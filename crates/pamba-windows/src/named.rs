//! The controls and places a drawing names, and the keyboard stood at one of them.
//!
//! A program's logic names no control: it asks, as an effect, for the keyboard to stand at a place
//! of its own naming. The drawing gives that name to a control, or to a place round a part of the
//! window, and the ask is carried out once the drawing that follows the step stands, since that
//! drawing may be the one that first shows it. Each name keeps a reference for each kind of thing
//! that carries it, and the ask goes to the control where the drawing gives the name to one, else
//! to the place: the control is the more exact of the two. Asked the keyboard, a place's border
//! passes the ask to the first control inside it that can take it (`microsoft-ui-xaml`,
//! `focusmgr.cpp`, `SetFocusedElement`), so a place reaches a control the reactor gives no
//! reference, such as its date picker. The names are kept for the window that stands on this
//! thread.
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
    names: Names<ElementRef>,
    /// Whether a control of the window has held the keyboard yet. Until one has, the window's
    /// content may not be on the screen, and no control of it can take the keyboard.
    taken: bool,
    /// The control of an ask Windows refused before any control had held the keyboard: asked once
    /// more when one first does.
    waits: Option<ElementRef>,
    /// Asks the window to call [`retry`] after a [`PAUSE`].
    pause: Rc<dyn Fn()>,
}

/// The names one window's drawings give, each to a control, to a place, or to both, and the
/// keyboard's ask of one of them. `R` is what a drawing gives a control or a place to carry its
/// name: a reference on Windows, a plain value in a case.
struct Names<R> {
    given: Places<Given<R>>,
    /// The number of the newest drawing, as `given` counts them.
    drawing: u64,
}

/// What the drawings give one name. Shared with `given`, which hands back a clone of it.
struct Given<R>(Rc<RefCell<Carriers<R>>>);

impl<R> Clone for Given<R> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<R> Default for Given<R> {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(Carriers {
            control: None,
            place: None,
        })))
    }
}

/// The control and the place that carry one name, each with the newest drawing that gave it.
struct Carriers<R> {
    control: Option<Carried<R>>,
    place: Option<Carried<R>>,
}

struct Carried<R> {
    reference: R,
    drawn: u64,
}

impl<R: Clone> Carriers<R> {
    /// What the keyboard's ask goes to, once drawing `drawing` stands: the control where it gave
    /// the name to one, else the place where it gave the name to one.
    fn asked(&self, drawing: u64) -> Option<R> {
        let newest = |carried: &Option<Carried<R>>| {
            carried
                .as_ref()
                .filter(|carried| carried.drawn == drawing)
                .map(|carried| carried.reference.clone())
        };
        newest(&self.control).or_else(|| newest(&self.place))
    }
}

/// The reference `slot` carries in drawing `drawing`: the same one where the drawing before gave
/// it too, so that a control or a place keeps its name from drawing to drawing, else a new one.
fn carry<R: Clone>(slot: &mut Option<Carried<R>>, drawing: u64, new: impl FnOnce() -> R) -> R {
    match slot {
        Some(carried) if carried.drawn + 1 >= drawing => {
            carried.drawn = drawing;
            carried.reference.clone()
        }
        _ => slot
            .insert(Carried {
                reference: new(),
                drawn: drawing,
            })
            .reference
            .clone(),
    }
}

impl<R: Clone> Names<R> {
    fn new() -> Self {
        Self {
            given: Places::with_retries(RETRIES),
            drawing: 0,
        }
    }

    fn begin(&mut self) {
        self.given.begin();
        self.drawing += 1;
    }

    fn end(&mut self) {
        self.given.end();
    }

    /// The reference a control carries for `name` in this drawing.
    fn control<K: Hash + Eq + 'static>(&mut self, name: K, new: impl FnOnce() -> R) -> R {
        let given = self.given.named(name, Given::default);
        carry(&mut given.0.borrow_mut().control, self.drawing, new)
    }

    /// The reference a place's border carries for `name` in this drawing.
    fn place<K: Hash + Eq + 'static>(&mut self, name: K, new: impl FnOnce() -> R) -> R {
        let given = self.given.named(name, Given::default);
        carry(&mut given.0.borrow_mut().place, self.drawing, new)
    }

    fn ask<K: Hash + Eq + 'static>(&mut self, name: K) {
        self.given.ask(name);
    }

    fn asks(&self) -> u64 {
        self.given.asks()
    }

    fn is_asked(&self) -> bool {
        self.given.is_asked()
    }

    /// What the newest ask goes to in the newest drawing; nothing where it gives the name to
    /// nothing.
    fn take_asked(&mut self) -> Option<R> {
        let drawing = self.drawing;
        self.given.take_asked()?.0.borrow().asked(drawing)
    }

    fn retry_later(&mut self) -> bool {
        self.given.retry_later()
    }

    /// What a refused ask, made again, goes to in the newest drawing: which of the control and the
    /// place is asked is settled again, since the drawing may have changed since.
    fn take_retry(&mut self) -> Option<R> {
        let drawing = self.drawing;
        self.given.take_retry()?.0.borrow().asked(drawing)
    }

    fn give_up(&mut self) {
        self.given.give_up();
    }
}

/// What follows Windows' answer to the keyboard's ask once a drawing stands: taken once.
type Then = Rc<RefCell<Option<Box<dyn FnOnce()>>>>;

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
            names: Names::new(),
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
            named.names.begin();
        }
    });
    // Not borrowed while the program draws: its drawing calls `named` and `place`.
    let drawn = draw();
    let asks = NAMED.with_borrow_mut(|named| {
        named.as_mut().map_or(0, |named| {
            named.names.end();
            named.names.asks()
        })
    });
    (drawn, asks)
}

/// Carries out the newest ask, where one waits for this drawing: the control the drawing gave
/// that name is asked to take the keyboard, else the place. `then` follows Windows' answer to the ask, whichever
/// it is, and follows at once where no ask is made.
pub(crate) fn carry_out(then: impl FnOnce() + 'static) {
    let asked = NAMED.with_borrow_mut(|named| {
        let named = named.as_mut()?;
        let control = named.names.take_asked()?;
        Some((control, named.taken))
    });
    let Some((control, taken)) = asked else {
        then();
        return;
    };
    // Shared with the answer, which the reactor may call more than once and may never call.
    let then: Then = Rc::new(RefCell::new(Some(Box::new(then))));
    let after = Rc::clone(&then);
    let again = control.clone();
    let queued = control.request_focus_result(move |answer| {
        answered(&again, taken, matches!(answer, Ok(true)));
        let then = after.borrow_mut().take();
        if let Some(then) = then {
            then();
        }
    });
    // Where the control is not on the screen the ask is not even queued, and there is no answer
    // to follow.
    if !queued {
        let then = then.borrow_mut().take();
        if let Some(then) = then {
            then();
        }
    }
}

/// Windows has answered an ask: the control took the keyboard, or it did not. An ask refused
/// before any control of the window had held the keyboard was made before the window's content
/// could take it: it is made once more when the content can. An ask refused after that names a
/// control Windows does not show yet: it is made again after a pause, [`RETRIES`] times at most.
fn answered(control: &ElementRef, taken_when_asked: bool, took: bool) {
    let next = NAMED.with_borrow_mut(|named| {
        let named = named.as_mut()?;
        if took {
            named.taken = true;
            return None;
        }
        if taken_when_asked {
            return named
                .names
                .retry_later()
                .then(|| Next::Pause(Rc::clone(&named.pause)));
        }
        if named.names.is_asked() {
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
/// drawing gives its name, else the place.
pub(crate) fn retry() {
    let control = NAMED.with_borrow_mut(|named| named.as_mut()?.names.take_retry());
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
        named.names.give_up();
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
/// only. The same name may also be given to a [`place`](crate::place), and [`focus`] then asks the
/// control.
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
        Some(named) => named.names.control(name, ElementRef::new),
        None => ElementRef::new(),
    })
}

/// The reference a place's border carries for the place this drawing calls `name`; one that
/// stands for nothing while no window of the program is being drawn.
pub(crate) fn place<K: Hash + Eq + 'static>(name: K) -> ElementRef {
    NAMED.with_borrow_mut(|named| match named {
        Some(named) => named.names.place(name, ElementRef::new),
        None => ElementRef::new(),
    })
}

/// Asks for the keyboard to stand at what the drawing calls `name`: what a program's effect comes
/// to where its logic, having answered something the person did, says where she goes on from.
///
/// Where the drawing gives the name to a control with [`named`], that control is asked. Where it
/// gives the name to no control and to a [`place`](crate::place), the keyboard goes to the first
/// control inside the place that takes it, in Windows' own order: a place reaches a control the
/// layer gives no reference, such as [`CalendarDatePicker`](crate::ui::CalendarDatePicker), and a
/// part of the window whose first control is the drawing's to decide.
///
/// Called while an effect is performed, on the window's thread. It is carried out once the
/// drawing that follows the step stands, so the control or the place may be one that drawing
/// shows for the first time. An ask made as the program starts, by an effect of its
/// construction, is carried out when the window's content first takes the keyboard. Of several
/// asks in one step the last is carried out. An ask of a place whose control already holds the
/// keyboard leaves it there.
///
/// The control may also be one Windows shows a moment after the step, as it shows what an
/// expander holds once the expander's opening begins. Windows refuses the keyboard to a control
/// it does not show, so a refused ask is made again 40 milliseconds later, six times at most, of
/// what the newest drawing then gives the name. It is given up as soon as any control takes the
/// keyboard, since the person has then put it somewhere herself, and when the program asks for
/// another place.
///
/// The keyboard stays where it is where no window of the program stands on this thread, where the
/// drawing gives the name to nothing, where a place holds nothing that takes the keyboard, and
/// where nothing asked can take it within that time.
///
/// ```no_run
/// use pamba_windows::ui::CalendarDatePicker;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// enum Place {
///     Expiry,
/// }
///
/// // In the drawing: a day's control, which takes no name of its own, in a place.
/// let expiry = pamba_windows::place(Place::Expiry, CalendarDatePicker::new());
///
/// // Where the program's effect is performed: the keyboard goes to the day's control.
/// pamba_windows::focus(Place::Expiry);
/// ```
pub fn focus<K: Hash + Eq + 'static>(name: K) {
    NAMED.with_borrow_mut(|named| {
        if let Some(named) = named {
            named.names.ask(name);
            named.waits = None;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{Names, RETRIES};

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    enum Place {
        Expiry,
        Note,
    }

    /// What a drawing gives a name.
    #[derive(Clone, Copy)]
    enum Gives {
        Control,
        Place,
        Both,
        PlaceThenControl,
        Nothing,
    }

    /// Names whose references are numbered in the order they are made, from 1.
    struct Drawn {
        names: Names<u32>,
        made: u32,
    }

    impl Drawn {
        fn new() -> Self {
            Self {
                names: Names::new(),
                made: 0,
            }
        }

        /// One drawing, giving `name` as `gives` says; the references it carried, the control's
        /// then the place's.
        fn draw(&mut self, name: Place, gives: Gives) -> (Option<u32>, Option<u32>) {
            let Self { names, made } = self;
            let mut new = || {
                *made += 1;
                *made
            };
            names.begin();
            let carried = match gives {
                Gives::Control => (Some(names.control(name, &mut new)), None),
                Gives::Place => (None, Some(names.place(name, &mut new))),
                Gives::Both => {
                    let control = names.control(name, &mut new);
                    (Some(control), Some(names.place(name, &mut new)))
                }
                Gives::PlaceThenControl => {
                    let place = names.place(name, &mut new);
                    (Some(names.control(name, &mut new)), Some(place))
                }
                Gives::Nothing => (None, None),
            };
            names.end();
            carried
        }
    }

    #[test]
    fn a_name_given_to_a_control_alone_asks_the_control() {
        let mut drawn = Drawn::new();
        drawn.names.ask(Place::Expiry);
        let (control, _) = drawn.draw(Place::Expiry, Gives::Control);

        assert_eq!(drawn.names.take_asked(), control);
        assert!(control.is_some());
    }

    #[test]
    fn a_name_given_to_a_place_alone_asks_the_place() {
        let mut drawn = Drawn::new();
        drawn.names.ask(Place::Expiry);
        let (_, place) = drawn.draw(Place::Expiry, Gives::Place);

        assert_eq!(drawn.names.take_asked(), place);
        assert!(place.is_some());
    }

    #[test]
    fn a_name_given_to_a_control_and_a_place_asks_the_control_in_either_order() {
        for gives in [Gives::Both, Gives::PlaceThenControl] {
            let mut drawn = Drawn::new();
            drawn.names.ask(Place::Expiry);
            let (control, place) = drawn.draw(Place::Expiry, gives);

            assert_ne!(
                control, place,
                "the control and the place share a reference"
            );
            assert_eq!(
                drawn.names.take_asked(),
                control,
                "the place was asked over the control"
            );
        }
    }

    #[test]
    fn a_name_the_drawing_gives_to_nothing_asks_nothing() {
        let mut drawn = Drawn::new();
        drawn.draw(Place::Expiry, Gives::Both);
        drawn.names.ask(Place::Expiry);
        drawn.draw(Place::Expiry, Gives::Nothing);

        assert_eq!(drawn.names.take_asked(), None);
        assert!(
            !drawn.names.retry_later(),
            "an ask of nothing is made again"
        );
    }

    #[test]
    fn the_newest_drawing_decides_which_is_asked() {
        let mut drawn = Drawn::new();
        drawn.draw(Place::Expiry, Gives::Both);
        let (_, place) = drawn.draw(Place::Expiry, Gives::Place);
        drawn.names.ask(Place::Expiry);
        assert_eq!(
            drawn.names.take_asked(),
            place,
            "a control an earlier drawing gave was asked"
        );

        let (control, _) = drawn.draw(Place::Expiry, Gives::Control);
        drawn.names.ask(Place::Expiry);
        assert_eq!(
            drawn.names.take_asked(),
            control,
            "a place an earlier drawing gave was asked"
        );
    }

    #[test]
    fn an_ask_made_again_settles_which_is_asked_against_the_newest_drawing() {
        let mut drawn = Drawn::new();
        drawn.names.ask(Place::Expiry);
        let (_, place) = drawn.draw(Place::Expiry, Gives::Place);
        assert_eq!(drawn.names.take_asked(), place);

        assert!(drawn.names.retry_later());
        let (control, _) = drawn.draw(Place::Expiry, Gives::Both);
        assert_eq!(
            drawn.names.take_retry(),
            control,
            "the ask made again went to what the first drawing gave"
        );

        assert!(drawn.names.retry_later());
        drawn.draw(Place::Expiry, Gives::Nothing);
        assert_eq!(drawn.names.take_retry(), None);
    }

    #[test]
    fn a_refused_ask_of_a_place_is_made_again_six_times_at_most() {
        let mut drawn = Drawn::new();
        drawn.names.ask(Place::Expiry);
        let (_, place) = drawn.draw(Place::Expiry, Gives::Place);
        assert_eq!(drawn.names.take_asked(), place);

        let mut again = 0;
        while drawn.names.retry_later() {
            again += 1;
            assert!(again <= RETRIES, "made again {again} times");
            assert_eq!(drawn.names.take_retry(), place);
        }
        assert_eq!(again, RETRIES);
    }

    #[test]
    fn each_kind_keeps_its_reference_while_each_drawing_gives_it_and_gets_a_new_one_after_a_gap() {
        let mut drawn = Drawn::new();
        let first = drawn.draw(Place::Note, Gives::Both);
        assert_eq!(
            drawn.draw(Place::Note, Gives::Both),
            first,
            "a reference changed between drawings"
        );

        let (_, place) = drawn.draw(Place::Note, Gives::Place);
        assert_eq!(place, first.1);
        let (control, _) = drawn.draw(Place::Note, Gives::Both);
        assert_ne!(
            control, first.0,
            "a control a drawing left out came back with its old reference"
        );
    }
}
