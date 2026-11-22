//! What arrives in answer, brought into view and said.
//!
//! A person does something and the program answers with something new: a refusal, a result. Its
//! logic decides that it arrived and in which words, and asks, as an effect, for the place it
//! names to be brought into view and for the words to be said; its drawing wraps what arrives in
//! that place. Both asks are carried out once the drawing that follows the step stands, after the
//! step's ask for the keyboard, so a screen reader hears where the keyboard went before it hears
//! the answer.

use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

use pamba::{Asks, Places};
use windows_reactor::{Border, View};

use crate::named;
use crate::xaml;

/// One thing done once a drawing stands.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Act {
    /// The place whose border holds this tag is brought into view.
    Bring(Rc<str>),
    /// These words are said.
    Say(String),
}

/// The places a window's drawings give, and what is asked of them, as this thread's window keeps
/// them.
struct Arrivals {
    /// Each place's tag, the same for a name from drawing to drawing.
    tags: Places<Rc<str>>,
    brought: Asks<()>,
    said: Vec<String>,
    /// How many times words have been asked to be said.
    sayings: u64,
    /// Whether Windows has told the layer where the window's content is.
    reached: bool,
    /// The number of the last tag made.
    last: u64,
}

impl Arrivals {
    fn new() -> Self {
        Self {
            tags: Places::new(),
            brought: Asks::new(),
            said: Vec::new(),
            sayings: 0,
            reached: false,
            last: 0,
        }
    }

    /// The tag of the place this drawing calls `name`: one the layer makes, unlike anything a
    /// program would write.
    fn tag<K: Hash + Eq + 'static>(&mut self, name: K) -> Rc<str> {
        let last = &mut self.last;
        self.tags.named(name, || {
            *last += 1;
            Rc::from(format!("pamba.place.{last}"))
        })
    }

    fn bring<K: Hash + Eq + 'static>(&mut self, name: K) {
        self.brought.ask(name, ());
    }

    fn say(&mut self, words: &str) {
        if !words.is_empty() {
            self.said.push(words.to_owned());
            self.sayings += 1;
        }
    }

    /// How many asks have been made: it changes exactly when there is a new one to carry out.
    fn made(&self) -> u64 {
        self.brought.made() + self.sayings
    }

    /// What is to be done once the drawing that follows the asks stands: nothing yet where
    /// Windows has not told where the window's content is, since until then nothing in it can be
    /// reached, and the asks wait.
    fn take(&mut self) -> Vec<Act> {
        if self.reached { self.due() } else { Vec::new() }
    }

    /// Windows has told where the window's content is: what waited, in order.
    fn reach(&mut self) -> Vec<Act> {
        self.reached = true;
        self.due()
    }

    /// Every place brought into view, then every word said, each kind in the order it was asked.
    /// A place the newest drawing does not give comes to nothing.
    fn due(&mut self) -> Vec<Act> {
        let brought = self.brought.take(&self.tags).into_iter();
        let said = std::mem::take(&mut self.said).into_iter();
        brought
            .filter_map(|(tag, ())| tag.map(Act::Bring))
            .chain(said.map(Act::Say))
            .collect()
    }
}

thread_local! {
    /// What the window that stands on this thread has asked; nothing where no window does.
    static ARRIVALS: RefCell<Option<Arrivals>> = const { RefCell::new(None) };
}

/// The window stands on this thread.
pub(crate) fn stands() {
    ARRIVALS.with_borrow_mut(|arrivals| *arrivals = Some(Arrivals::new()));
    xaml::stands(reached);
}

/// The window has gone, and what it asked with it.
pub(crate) fn gone() {
    xaml::gone();
    ARRIVALS.with_borrow_mut(|arrivals| *arrivals = None);
}

/// Makes one drawing: the places `draw` gives are kept, and those the drawing before gave and this
/// one did not are dropped. Gives what `draw` made and the count of asks made so far.
pub(crate) fn drawing<T>(draw: impl FnOnce() -> T) -> (T, u64) {
    ARRIVALS.with_borrow_mut(|arrivals| {
        if let Some(arrivals) = arrivals {
            arrivals.tags.begin();
        }
    });
    // Not borrowed while the program draws: its drawing calls `place`.
    let drawn = draw();
    let made = ARRIVALS.with_borrow_mut(|arrivals| {
        arrivals.as_mut().map_or(0, |arrivals| {
            arrivals.tags.end();
            arrivals.made()
        })
    });
    (drawn, made)
}

/// Carries out, once this drawing stands, the step's ask for the keyboard and then what was asked
/// of its places.
///
/// The reactor stands the keyboard from a queue of its own, after this returns. Raised at once,
/// the words would be heard before the screen reader heard where the keyboard went, so what was
/// asked here follows Windows' answer to the keyboard's ask, where the step made one.
pub(crate) fn carry_out() {
    let due = ARRIVALS.with_borrow_mut(|arrivals| arrivals.as_mut().map(Arrivals::take));
    let due = due.unwrap_or_default();
    named::carry_out(move || perform(&due));
}

/// Windows has first told where the window's content is.
fn reached() {
    let due = ARRIVALS.with_borrow_mut(|arrivals| arrivals.as_mut().map(Arrivals::reach));
    perform(&due.unwrap_or_default());
}

fn perform(due: &[Act]) {
    for act in due {
        match act {
            Act::Bring(tag) => xaml::bring_into_view(tag),
            Act::Say(words) => xaml::announce(words),
        }
    }
}

/// `content` as the place the program calls `name`, for an effect to bring into view with
/// [`bring_into_view`].
///
/// The name is the program's own, as [`named`](crate::named) takes one. A drawing gives one name
/// to one place. The border draws nothing - no padding, no thickness, no background - and is
/// placed as any border is, in a grid's row or with a margin. It holds the place in its automation
/// identifier, so giving it another undoes the place; no screen reader is shown it, since a border
/// has no automation peer.
///
/// Called while no window of the program is being drawn, it gives a border that is no place.
///
/// ```no_run
/// use pamba_windows::ui::TextBlock;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// enum Place {
///     Refusal,
/// }
///
/// // In the drawing: what the program answered, where it is brought into view.
/// let notice = pamba_windows::place(Place::Refusal, TextBlock::new().text("That day is full"));
/// ```
pub fn place<K: Hash + Eq + 'static>(name: K, content: impl Into<View>) -> Border {
    let tag =
        ARRIVALS.with_borrow_mut(|arrivals| arrivals.as_mut().map(|arrivals| arrivals.tag(name)));
    let border = Border::new().content(content);
    match tag {
        Some(tag) => border.automation_id(&*tag),
        None => border,
    }
}

/// Asks for the place the drawing calls `name` to be brought into view: what a program's effect
/// comes to where its logic answered something the person did with something new.
///
/// Called while an effect is performed, on the window's thread. It is carried out once the drawing
/// that follows the step stands, so the place may be one that drawing shows for the first time,
/// and after the step's ask for the keyboard. Every scrolling area the place stands in is moved as
/// little as shows the whole place; where the place is taller than an area, its head shows. The
/// rule is stated for the scrolling area of [`ui`](crate::ui), `ScrollViewer`. Windows animates
/// the move unless the person has turned animations off. The keyboard does not move.
///
/// Where the drawing that follows gives no place that name, the ask comes to nothing, and a later
/// drawing does not bring it back. Until a control of the window first takes the keyboard, the
/// layer cannot reach the window's content: asks made before then wait, in order, and are carried
/// out at that moment, and in a window where nothing ever takes the keyboard they are never carried
/// out.
///
/// ```no_run
/// # #[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// # enum Place { Refusal }
/// # enum Effect { Arrived { place: Place, said: String } }
/// # fn perform(effect: Effect) {
/// match effect {
///     Effect::Arrived { place, said } => {
///         pamba_windows::bring_into_view(place);
///         pamba_windows::announce(&said);
///     }
/// }
/// # }
/// ```
pub fn bring_into_view<K: Hash + Eq + 'static>(name: K) {
    ARRIVALS.with_borrow_mut(|arrivals| {
        if let Some(arrivals) = arrivals {
            arrivals.bring(name);
        }
    });
}

/// Asks for `words` to be said to a screen reader, with no move of the keyboard: what a program's
/// effect comes to where its logic has an answer to tell, such as "Copied" or a refusal.
///
/// Called while an effect is performed, on the window's thread. It is carried out once the drawing
/// that follows the step stands, after the step's ask for the keyboard and after each
/// [`bring_into_view`] of the step, so a screen reader hears where the keyboard went first. Each
/// is said whole, in the order asked, and none interrupts what is being said. The same words
/// asked again are said again. Empty words are not said. It waits, as [`bring_into_view`] does,
/// until a control of the window first takes the keyboard.
///
/// ```no_run
/// pamba_windows::announce("Copied");
/// ```
pub fn announce(words: &str) {
    ARRIVALS.with_borrow_mut(|arrivals| {
        if let Some(arrivals) = arrivals {
            arrivals.say(words);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{Act, Arrivals};

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    enum Place {
        Refusal,
        Result,
        Gone,
    }

    fn drawn(arrivals: &mut Arrivals, places: &[Place]) {
        arrivals.tags.begin();
        for place in places {
            arrivals.tag(*place);
        }
        arrivals.tags.end();
    }

    fn reached() -> Arrivals {
        let mut arrivals = Arrivals::new();
        assert_eq!(arrivals.reach(), []);
        arrivals
    }

    #[test]
    fn places_are_brought_into_view_before_words_are_said_each_kind_in_the_order_asked() {
        let mut arrivals = reached();
        arrivals.say("first");
        arrivals.bring(Place::Result);
        arrivals.say("second");
        arrivals.bring(Place::Refusal);
        drawn(&mut arrivals, &[Place::Refusal, Place::Result]);

        assert_eq!(
            arrivals.take(),
            [
                Act::Bring("pamba.place.2".into()),
                Act::Bring("pamba.place.1".into()),
                Act::Say("first".to_owned()),
                Act::Say("second".to_owned()),
            ]
        );
        assert_eq!(
            arrivals.take(),
            [],
            "what was carried out is carried out again"
        );
    }

    #[test]
    fn asks_made_before_the_content_is_reached_wait_and_are_carried_out_in_order_when_it_is() {
        let mut arrivals = Arrivals::new();
        arrivals.bring(Place::Refusal);
        arrivals.say("refused");
        drawn(&mut arrivals, &[Place::Refusal]);
        assert_eq!(
            arrivals.take(),
            [],
            "an ask was carried out before the content was reached"
        );
        arrivals.say("again");
        drawn(&mut arrivals, &[Place::Refusal]);
        assert_eq!(
            arrivals.take(),
            [],
            "an ask was carried out before the content was reached"
        );

        assert_eq!(
            arrivals.reach(),
            [
                Act::Bring("pamba.place.1".into()),
                Act::Say("refused".to_owned()),
                Act::Say("again".to_owned()),
            ]
        );
        assert_eq!(
            arrivals.take(),
            [],
            "what waited is carried out a second time"
        );
    }

    #[test]
    fn once_the_content_is_reached_an_ask_is_carried_out_at_the_drawing_after_it() {
        let mut arrivals = reached();
        drawn(&mut arrivals, &[Place::Result]);
        arrivals.bring(Place::Result);
        drawn(&mut arrivals, &[Place::Result]);

        assert_eq!(arrivals.take(), [Act::Bring("pamba.place.1".into())]);
    }

    #[test]
    fn an_ask_for_a_place_the_drawing_after_it_does_not_give_comes_to_nothing_and_is_not_kept() {
        let mut arrivals = reached();
        arrivals.bring(Place::Gone);
        arrivals.say("said all the same");
        drawn(&mut arrivals, &[Place::Refusal]);
        assert_eq!(arrivals.take(), [Act::Say("said all the same".to_owned())]);

        drawn(&mut arrivals, &[Place::Gone]);
        assert_eq!(arrivals.take(), [], "an ask that came to nothing came back");
    }

    #[test]
    fn empty_words_are_not_said_and_are_no_ask() {
        let mut arrivals = reached();
        arrivals.say("");

        assert_eq!(arrivals.made(), 0, "empty words were counted as an ask");
        assert_eq!(arrivals.take(), []);
    }

    #[test]
    fn the_same_words_asked_twice_are_said_twice_and_each_ask_is_counted() {
        let mut arrivals = reached();
        arrivals.say("Copied");
        arrivals.say("Copied");
        arrivals.bring(Place::Result);

        assert_eq!(arrivals.made(), 3);
        assert_eq!(
            arrivals.take(),
            [Act::Say("Copied".to_owned()), Act::Say("Copied".to_owned())]
        );
    }

    #[test]
    fn a_place_keeps_its_tag_from_drawing_to_drawing_and_another_place_has_another() {
        let mut arrivals = Arrivals::new();
        arrivals.tags.begin();
        let refusal = arrivals.tag(Place::Refusal);
        let result = arrivals.tag(Place::Result);
        arrivals.tags.end();
        arrivals.tags.begin();
        let again = arrivals.tag(Place::Refusal);
        arrivals.tags.end();

        assert_eq!(&*refusal, "pamba.place.1");
        assert_eq!(&*result, "pamba.place.2");
        assert_eq!(again, refusal, "a place's tag changed between drawings");
    }
}
