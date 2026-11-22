//! XAML itself, reached beside the reactor through bindings of Pamba's own.
//!
//! The reactor offers no call that brings an element into view. XAML has one, and what Pamba
//! calls of XAML for it, and to say words to a screen reader, is bound by `tools/bind` from the
//! metadata the reactor's own bindings are written from. This module is the only one that names
//! those bindings.
//!
//! It sets no property and no child of anything the reactor draws, and keeps no reference to one
//! from one call to the next: it calls only what the person's own input or her screen reader could
//! have caused, a scroll and words said, and, before a scroll, the layout XAML would make on its
//! next pass. What it keeps of XAML, the window's root and the registration of the handler that
//! learns it, is kept for the thread the window stands on, and let go with the window.

use std::cell::RefCell;

use windows_core::{EventRevoker, HSTRING, Interface as _};

use self::bindings::{
    AutomationNotificationKind, AutomationNotificationProcessing, AutomationPeer,
    AutomationProperties, BringIntoViewOptions, DependencyObject, FocusManager,
    FocusManagerGotFocusEventArgs, FrameworkElement, FrameworkElementAutomationPeer, Rect,
    ScrollView, ScrollViewer, UIElement, VisualTreeHelper, XamlRoot,
};

#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::unwrap_used,
    dead_code,
    elided_lifetimes_in_paths,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    trivial_casts,
    unreachable_pub,
    unsafe_code,
    reason = "written by Microsoft's generator from XAML's own metadata: see `tools/bind`"
)]
#[rustfmt::skip]
mod bindings;

/// The activity every word said is raised under, so that a screen reader or a listener can tell
/// them from what other programs say.
const ACTIVITY: &str = "pamba.announce";

/// What this thread's window has of XAML.
struct Held {
    /// The window's root, once a control of it has first taken the keyboard.
    root: Option<XamlRoot>,
    /// The handler of XAML's static event, removed when this is dropped.
    _listening: EventRevoker,
    /// Called the first time the root is known.
    reached: fn(),
}

thread_local! {
    /// What the window that stands on this thread has of XAML; nothing where no window does.
    static HELD: RefCell<Option<Held>> = const { RefCell::new(None) };
}

/// The window stands on this thread: the root of its content is learnt the first time a control
/// of it takes the keyboard, and `reached` is called then, once.
///
/// The argument-free read of the focused element gives nothing in a desktop program
/// (`microsoft-ui-xaml`, `FocusManager.cpp`, `GetFocusedElement`), so the root is learnt from the
/// static event XAML's core raises for the thread when a control takes the keyboard
/// (`DXamlCore.cpp`, `OnFocusManagerGotFocusEvent`).
pub(crate) fn stands(reached: fn()) {
    // XAML raises the event on this thread, and the handler reaches what is held through it alone.
    let listening = FocusManager::GotFocus(|_, focused| took(focused.as_ref()));
    HELD.with_borrow_mut(|held| {
        *held = listening.ok().map(|listening| Held {
            root: None,
            _listening: listening,
            reached,
        });
    });
}

/// The window has gone: the handler is removed and the root let go.
pub(crate) fn gone() {
    let held = HELD.with_borrow_mut(Option::take);
    // Dropped outside the borrow: removing the handler calls into XAML.
    drop(held);
}

fn took(focused: Option<&FocusManagerGotFocusEventArgs>) {
    let first = HELD.with_borrow_mut(|held| {
        let held = held.as_mut().filter(|held| held.root.is_none())?;
        let root = focused?
            .NewFocusedElement()
            .and_then(|element| element.cast::<UIElement>())
            .and_then(|element| element.XamlRoot())
            .ok()?;
        held.root = Some(root);
        Some(held.reached)
    });
    // Not borrowed while the layer carries out what waited: it reaches the root through here.
    if let Some(reached) = first {
        reached();
    }
}

fn root() -> Option<XamlRoot> {
    HELD.with_borrow(|held| held.as_ref()?.root.clone())
}

/// Walks the window's content and everything under it, in the tree's order, until `found` gives
/// an answer.
fn walk<T>(mut found: impl FnMut(&DependencyObject) -> Option<T>) -> Option<T> {
    let content = root()?.Content().ok()?.cast::<DependencyObject>().ok()?;
    let mut ahead = vec![content];
    while let Some(element) = ahead.pop() {
        if let Some(answer) = found(&element) {
            return Some(answer);
        }
        let count = VisualTreeHelper::GetChildrenCount(&element).unwrap_or(0);
        // Pushed last child first, so that the first child is the next taken.
        ahead.extend(
            (0..count)
                .rev()
                .filter_map(|index| VisualTreeHelper::GetChild(&element, index).ok()),
        );
    }
    None
}

/// The element of the window's content that holds the automation identifier `tag`.
fn tagged(tag: &str) -> Option<UIElement> {
    walk(|element| {
        if AutomationProperties::GetAutomationId(element).is_ok_and(|id| id == tag) {
            element.cast::<UIElement>().ok()
        } else {
            None
        }
    })
}

/// The smallest viewport, across and down, of the scrolling areas `element` stands in. An area
/// whose viewport reads as nothing is not yet laid out, and is left out.
fn viewport(element: &UIElement) -> (f64, f64) {
    let mut least = (f64::INFINITY, f64::INFINITY);
    let mut at = element
        .cast::<DependencyObject>()
        .and_then(|element| VisualTreeHelper::GetParent(&element))
        .ok();
    while let Some(element) = at {
        let seen = element
            .cast::<ScrollViewer>()
            .and_then(|area| Ok((area.ViewportWidth()?, area.ViewportHeight()?)))
            .or_else(|_| {
                element
                    .cast::<ScrollView>()
                    .and_then(|area| Ok((area.ViewportWidth()?, area.ViewportHeight()?)))
            });
        if let Ok((width, height)) = seen {
            if width > 0.0 {
                least.0 = least.0.min(width);
            }
            if height > 0.0 {
                least.1 = least.1.min(height);
            }
        }
        at = VisualTreeHelper::GetParent(&element).ok();
    }
    least
}

/// Brings the element tagged `tag` into view, where the window's content holds one. The rule is
/// stated for the scrolling area the layer's `ui` draws, XAML's `ScrollViewer`.
pub(crate) fn bring_into_view(tag: &str) {
    let Some(place) = tagged(tag) else {
        return;
    };
    // A place the drawing shows for the first time is in the tree but not yet laid out when the
    // ask is carried out: its size reads as nothing, and a rectangle of nothing moves no area.
    let _ = place.UpdateLayout();
    let size = place
        .cast::<FrameworkElement>()
        .and_then(|place| Ok((place.ActualWidth()?, place.ActualHeight()?)));
    let (width, height) = size.unwrap_or_default();
    let (across, down) = viewport(&place);
    // XAML moves each scrolling area as little as shows the rectangle it is given, and where the
    // rectangle is taller than the area it shows the head only of one below the area: one above
    // it is aligned at its foot, and one that spans it is not moved
    // (`ScrollContentPresenter_Partial.cpp:2521-2565`). An alignment ratio of 0 would show the
    // head, but it gives the rectangle the area's height (`ScrollContentPresenter_Partial.cpp:
    // 1324-1330`), so a small place below the area would be moved to its top. So the rectangle
    // is the place's own, each side clipped to the smallest viewport it stands in: the whole
    // place where it fits, and its head where it does not. The move is animated, and XAML leaves
    // the animation out where Windows' setting for animations is off
    // (`ScrollViewer_Partial.cpp:3662-3665`).
    #[allow(
        clippy::cast_possible_truncation,
        reason = "a size on the screen is far inside an `f32`'s range"
    )]
    let shown = Rect {
        X: 0.0,
        Y: 0.0,
        Width: width.min(across) as f32,
        Height: height.min(down) as f32,
    };
    if let Ok(options) = BringIntoViewOptions::new() {
        // Where Windows refuses, nothing moves, and there is nothing the program could do.
        let _ = options
            .SetTargetRect(Some(shown))
            .and_then(|()| place.StartBringIntoViewWithOptions(&options));
    }
}

/// The automation peer of `element`, where its kind has one.
fn peer(element: &UIElement) -> Option<AutomationPeer> {
    FrameworkElementAutomationPeer::CreatePeerForElement(element).ok()
}

/// Says `words` to a screen reader with no move of the keyboard, from the control that holds the
/// keyboard, or, where none does or it has no automation peer, from the first element of the
/// window's content in the tree's order that has one.
pub(crate) fn announce(words: &str) {
    let Some(root) = root() else {
        return;
    };
    let from = FocusManager::GetFocusedElementWithRoot(&root)
        .and_then(|element| element.cast::<UIElement>())
        .ok()
        .and_then(|element| peer(&element))
        .or_else(|| walk(|element| peer(&element.cast::<UIElement>().ok()?)));
    if let Some(peer) = from {
        // Where Windows refuses, nothing is said, and there is nothing the program could do.
        let _ = peer.RaiseNotificationEvent(
            AutomationNotificationKind::Other,
            AutomationNotificationProcessing::All,
            &HSTRING::from(words),
            &HSTRING::from(ACTIVITY),
        );
    }
}
