//! The controls a drawing names, and the keyboard stood at one of them.
//!
//! A program's logic names no control: it asks, as an effect, for the keyboard to stand at a place
//! of its own naming. The drawing gives that name to a control, and the ask is carried out once
//! the drawing that follows the step stands, since that drawing may be the one that first shows
//! the control. The names are kept for the window that stands on this thread.

use std::cell::RefCell;
use std::hash::Hash;

use pamba::Places;
use windows_reactor::ElementRef;

#[derive(Default)]
struct Named {
    places: Places<ElementRef>,
    /// Whether a control of the window has held the keyboard yet. Until one has, the window's
    /// content may not be on the screen, and no control of it can take the keyboard.
    taken: bool,
    /// The control of an ask Windows refused before any control had held the keyboard: asked once
    /// more when one first does.
    waits: Option<ElementRef>,
}

thread_local! {
    /// The names of the window that stands on this thread; none where no window does.
    static NAMED: RefCell<Option<Named>> = const { RefCell::new(None) };
}

/// The window stands on this thread.
pub(crate) fn stands() {
    NAMED.with_borrow_mut(|named| *named = Some(Named::default()));
}

/// The window has gone, and its names with it.
pub(crate) fn gone() {
    NAMED.with_borrow_mut(|named| *named = None);
}

/// Makes one drawing: the names `draw` gives are kept, and those the drawing before gave and this
/// one did not are dropped. Gives what `draw` made and the count of asks made so far.
pub(crate) fn drawing<T>(draw: impl FnOnce() -> T) -> (T, u64) {
    NAMED.with_borrow_mut(|named| {
        if let Some(named) = named {
            named.places.begin();
        }
    });
    // Not borrowed while the program draws: its drawing calls `named`.
    let drawn = draw();
    let asks = NAMED.with_borrow_mut(|named| {
        named.as_mut().map_or(0, |named| {
            named.places.end();
            named.places.asks()
        })
    });
    (drawn, asks)
}

/// Carries out the newest ask, where one waits for this drawing: the control the drawing gave
/// that name is asked to take the keyboard.
pub(crate) fn carry_out() {
    let asked = NAMED.with_borrow_mut(|named| {
        let named = named.as_mut()?;
        let control = named.places.take_asked()?;
        Some((control, named.taken))
    });
    if let Some((control, taken)) = asked {
        let again = control.clone();
        // Where the control is not on the screen the ask is not even queued, and there is nothing
        // to hear of.
        let _ = control.request_focus_result(move |answer| {
            answered(&again, taken, matches!(answer, Ok(true)));
        });
    }
}

/// Windows has answered an ask: the control took the keyboard, or it did not. An ask refused
/// before any control of the window had held the keyboard was made before the window's content
/// could take it: it is made once more when the content can.
fn answered(control: &ElementRef, taken_when_asked: bool, took: bool) {
    let again = NAMED.with_borrow_mut(|named| {
        let named = named.as_mut()?;
        if took {
            named.taken = true;
            return None;
        }
        if taken_when_asked || named.places.is_asked() {
            // The control cannot take the keyboard, or the program has asked for another since.
            return None;
        }
        if named.taken {
            return Some(control.clone());
        }
        named.waits = Some(control.clone());
        None
    });
    if let Some(control) = again {
        let _ = control.request_focus();
    }
}

/// A control of the window holds the keyboard. The first time, an ask that waited for the
/// window's content is carried out.
pub(crate) fn took() {
    let waited = NAMED.with_borrow_mut(|named| {
        let named = named.as_mut()?;
        named.taken = true;
        named.waits.take()
    });
    if let Some(control) = waited {
        let _ = control.request_focus();
    }
}

/// A reference for the control this drawing calls `name`, to give the control with
/// `element_ref`.
///
/// The name is the program's own: any value that can be compared and hashed, such as a case of an
/// enum of the places its logic speaks of. The control keeps the name for as long as each drawing
/// gives it. A drawing gives one name to one control: given to two, it stands for one of them
/// only.
///
/// Called while no window of the program is being drawn, it gives a reference that stands for no
/// control.
///
/// ```no_run
/// use pamba_windows::ui::Button;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// enum Place {
///     Pay,
/// }
///
/// // In the drawing: the press the keyboard is asked to stand at.
/// let pay = Button::new().content("Pay").element_ref(&pamba_windows::named(Place::Pay));
///
/// // Where the program's effect is performed.
/// pamba_windows::focus(Place::Pay);
/// ```
#[must_use]
pub fn named<K: Hash + Eq + 'static>(name: K) -> ElementRef {
    NAMED.with_borrow_mut(|named| match named {
        Some(named) => named.places.named(name, ElementRef::new),
        None => ElementRef::new(),
    })
}

/// Asks for the keyboard to stand at the control the drawing calls `name`: what a program's
/// effect comes to where its logic, having answered something the person did, says where she
/// goes on from.
///
/// Called while an effect is performed, on the window's thread. It is carried out once the
/// drawing that follows the step stands, so the control may be one that drawing shows for the
/// first time. An ask made as the program starts, by an effect of its construction, is carried
/// out when the window's content first takes the keyboard. Of several asks in one step the last
/// is carried out.
///
/// The keyboard stays where it is where no window of the program stands on this thread, where the
/// drawing gives no control that name, and where the control cannot take the keyboard.
pub fn focus<K: Hash + Eq + 'static>(name: K) {
    NAMED.with_borrow_mut(|named| {
        if let Some(named) = named {
            named.places.ask(name);
            named.waits = None;
        }
    });
}
