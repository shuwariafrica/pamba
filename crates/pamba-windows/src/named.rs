//! The controls a drawing names, and the keyboard stood at one of them.
//!
//! A program's logic names no control: it asks, as an effect, for the keyboard to stand at a place
//! of its own naming. The drawing gives that name to a control, and the ask is carried out once
//! the drawing that follows the step stands, since that drawing may be the one that first shows
//! the control. The names are kept for the window that stands on this thread.
//!
//! Windows does not always show a control in the drawing that holds it. What an expander holds
//! stays collapsed until the expander's opening begins, and that is after the expander has
//! told of being opened (`microsoft-ui-xaml`, `Expander.cpp`, `OnIsExpandedPropertyChanged`,
//! and the state `ExpandDown` of `Expander.xaml`). A control that is not shown cannot take the
//! keyboard, and Windows refuses the ask. So an ask Windows refuses is made again after a
//! [`PAUSE`], [`RETRIES`] times at most, and is given up as soon as any control takes the
//! keyboard or the program asks for another place.

use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;
use std::time::Duration;

use pamba::Places;
use windows_reactor::ElementRef;

/// How many times an ask Windows refuses is made again. With [`PAUSE`] it bounds the wait at about
/// a quarter of a second: Windows shows what was opened within a frame or two, a slow computer
/// takes several, and past that the person has gone on.
const RETRIES: u8 = 6;

/// How long after Windows refuses an ask it is made again: two or three frames.
pub(crate) const PAUSE: Duration = Duration::from_millis(40);

struct Named {
    places: Places<ElementRef>,
    /// Whether a control of the window has held the keyboard yet. Until one has, the window's
    /// content may not be on the screen, and no control of it can take the keyboard.
    taken: bool,
    /// The control of an ask Windows refused before any control had held the keyboard: asked once
    /// more when one first does.
    waits: Option<ElementRef>,
    /// Asks the window to call [`retry`] after a [`PAUSE`].
    pause: Rc<dyn Fn()>,
}

/// What follows Windows' answer to an ask.
enum Next {
    /// The control is asked once more, now.
    Now(ElementRef),
    /// The ask is made again after a pause.
    Pause(Rc<dyn Fn()>),
}

thread_local! {
    /// The names of the window that stands on this thread; none where no window does.
    static NAMED: RefCell<Option<Named>> = const { RefCell::new(None) };
}

/// The window stands on this thread. `pause` asks it to call [`retry`] after a [`PAUSE`].
pub(crate) fn stands(pause: impl Fn() + 'static) {
    NAMED.with_borrow_mut(|named| {
        *named = Some(Named {
            places: Places::with_retries(RETRIES),
            taken: false,
            waits: None,
            pause: Rc::new(pause),
        });
    });
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
/// could take it: it is made once more when the content can. An ask refused after that names a
/// control Windows does not show yet: it is made again after a pause, as often as the places
/// retry one.
fn answered(control: &ElementRef, taken_when_asked: bool, took: bool) {
    let next = NAMED.with_borrow_mut(|named| {
        let named = named.as_mut()?;
        if took {
            named.taken = true;
            return None;
        }
        if taken_when_asked {
            return named
                .places
                .retry_later()
                .then(|| Next::Pause(Rc::clone(&named.pause)));
        }
        if named.places.is_asked() {
            // The program has asked for another place since.
            return None;
        }
        if named.taken {
            return Some(Next::Now(control.clone()));
        }
        named.waits = Some(control.clone());
        None
    });
    match next {
        Some(Next::Now(control)) => {
            let _ = control.request_focus();
        }
        // Not borrowed while the window is told: it is told through the reactor.
        Some(Next::Pause(pause)) => pause(),
        None => {}
    }
}

/// The pause after a refused ask is over: the ask is made again, of the control the newest
/// drawing gives its name.
pub(crate) fn retry() {
    let control = NAMED.with_borrow_mut(|named| named.as_mut()?.places.take_retry());
    if let Some(control) = control {
        let again = control.clone();
        let _ = control.request_focus_result(move |answer| {
            answered(&again, true, matches!(answer, Ok(true)));
        });
    }
}

/// A control of the window holds the keyboard. The first time, an ask that waited for the
/// window's content is carried out. An ask that was to be made again is given up: the person, or
/// the ask itself, has put the keyboard somewhere, and nothing is to take it from there.
pub(crate) fn took() {
    let waited = NAMED.with_borrow_mut(|named| {
        let named = named.as_mut()?;
        named.taken = true;
        named.places.give_up();
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
/// The control may also be one Windows shows a moment after the step, as it shows what an
/// expander holds once the expander's opening begins. Windows refuses the keyboard to a control
/// it does not show, so a refused ask is made again 40 milliseconds later, six times at most.
/// It is given up as soon as any control takes the keyboard, since the person has then put it
/// somewhere herself, and when the program asks for another place.
///
/// The keyboard stays where it is where no window of the program stands on this thread, where the
/// drawing gives no control that name, and where the control cannot take the keyboard within
/// that time.
pub fn focus<K: Hash + Eq + 'static>(name: K) {
    NAMED.with_borrow_mut(|named| {
        if let Some(named) = named {
            named.places.ask(name);
            named.waits = None;
        }
    });
}
