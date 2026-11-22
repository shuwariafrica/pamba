use std::cell::Cell;

use pamba::Places;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Place {
    Pay,
    Find,
}

/// Hands out references a drawer would make: each new one is the next number.
#[derive(Default)]
struct Made(Cell<u32>);

impl Made {
    fn next(&self) -> u32 {
        self.0.set(self.0.get() + 1);
        self.0.get()
    }
}

/// One drawing that gives each of `names`, and the reference each got.
fn draw(places: &mut Places<u32>, made: &Made, names: &[Place]) -> Vec<u32> {
    places.begin();
    let references = names
        .iter()
        .map(|name| places.named(*name, || made.next()))
        .collect();
    places.end();
    references
}

#[test]
fn a_name_keeps_its_reference_for_as_long_as_each_drawing_gives_it() {
    let (mut places, made) = (Places::new(), Made::default());

    let drawings = [
        draw(&mut places, &made, &[Place::Pay]),
        draw(&mut places, &made, &[Place::Pay]),
        draw(&mut places, &made, &[Place::Pay, Place::Find]),
    ];

    assert_eq!(drawings, [vec![1], vec![1], vec![1, 2]]);
}

#[test]
fn a_name_a_drawing_does_not_give_is_dropped_and_is_new_when_given_again() {
    let (mut places, made) = (Places::new(), Made::default());

    let first = draw(&mut places, &made, &[Place::Pay, Place::Find]);
    let without = draw(&mut places, &made, &[Place::Find]);
    let again = draw(&mut places, &made, &[Place::Pay, Place::Find]);

    assert_eq!((first, without), (vec![1, 2], vec![2]));
    assert_eq!(
        again,
        [3, 2],
        "the dropped name came back with its old reference"
    );
}

#[test]
fn an_ask_is_taken_once_with_the_reference_the_drawing_after_it_gave_its_name() {
    let (mut places, made) = (Places::new(), Made::default());
    draw(&mut places, &made, &[Place::Find]);
    let before = places.asks();

    places.ask(Place::Pay);
    assert_eq!(places.asks(), before + 1, "an ask is not counted");
    assert!(places.is_asked());
    // The drawing that follows is the first to show the place asked for.
    let drawn = draw(&mut places, &made, &[Place::Find, Place::Pay]);

    assert_eq!(drawn, [1, 2]);
    assert_eq!(places.take_asked(), Some(2));
    assert!(!places.is_asked());
    assert_eq!(places.take_asked(), None, "an ask is taken twice");
    assert_eq!(places.asks(), before + 1);
}

#[test]
fn an_ask_for_a_place_the_drawing_after_it_does_not_give_comes_to_nothing() {
    let (mut places, made) = (Places::new(), Made::default());

    places.ask(Place::Pay);
    draw(&mut places, &made, &[Place::Find]);
    let taken = places.take_asked();
    // A later drawing gives the name, when the person has long gone on.
    draw(&mut places, &made, &[Place::Find, Place::Pay]);

    assert_eq!(taken, None);
    assert_eq!(
        places.take_asked(),
        None,
        "a later drawing brought the ask back"
    );
}

#[test]
fn the_newest_ask_replaces_the_one_before_it() {
    let (mut places, made) = (Places::new(), Made::default());

    places.ask(Place::Pay);
    places.ask(Place::Find);
    let drawn = draw(&mut places, &made, &[Place::Pay, Place::Find]);

    assert_eq!(drawn, [1, 2]);
    assert_eq!(places.take_asked(), Some(2));
    assert_eq!(places.asks(), 2);
}

/// Places that retry `times` times, with `Pay` asked for, drawn and taken: its reference is 1.
fn taken(times: u8) -> (Places<u32>, Made) {
    let (mut places, made) = (Places::with_retries(times), Made::default());
    places.ask(Place::Pay);
    draw(&mut places, &made, &[Place::Pay]);
    assert_eq!(places.take_asked(), Some(1));
    (places, made)
}

#[test]
fn an_ask_the_platform_cannot_carry_out_yet_is_retried_as_often_as_the_drawer_said_and_no_more() {
    let (mut places, _made) = taken(2);

    assert_eq!(
        places.take_retry(),
        None,
        "a retry waits before one is asked for"
    );
    let mut retried = 0;
    while places.retry_later() {
        retried += 1;
        assert!(retried <= 2, "the ask is retried a third time of two");
        assert_eq!(places.take_retry(), Some(1));
        assert_eq!(places.take_retry(), None, "a retry is taken twice");
    }

    assert_eq!(retried, 2);
    assert_eq!(
        places.take_retry(),
        None,
        "an ask that is done with is retried"
    );
}

#[test]
fn places_made_plainly_carry_an_ask_out_once() {
    let (mut places, made) = (Places::new(), Made::default());
    places.ask(Place::Pay);
    draw(&mut places, &made, &[Place::Pay]);
    assert_eq!(places.take_asked(), Some(1));

    assert!(
        !places.retry_later(),
        "places that retry nothing kept an ask"
    );
    assert_eq!(places.take_retry(), None);
}

#[test]
fn a_newer_ask_ends_the_retrying_of_the_one_before_it() {
    let (mut places, made) = taken(3);
    assert!(places.retry_later());

    places.ask(Place::Find);

    assert_eq!(
        places.take_retry(),
        None,
        "the older ask is retried over the newer"
    );
    assert!(
        !places.retry_later(),
        "the older ask is kept after the newer"
    );
    // The newer ask is carried out as any other, and is retried in its own right.
    draw(&mut places, &made, &[Place::Pay, Place::Find]);
    assert_eq!(places.take_asked(), Some(2));
    assert!(places.retry_later());
    assert_eq!(places.take_retry(), Some(2));
}

#[test]
fn an_ask_the_drawer_gives_up_is_not_retried() {
    let (mut places, _made) = taken(3);
    assert!(places.retry_later());

    // The person has put the keyboard somewhere herself.
    places.give_up();

    assert_eq!(places.take_retry(), None, "a retry outlived the giving up");
    assert!(!places.retry_later(), "an ask given up is kept again");
}

#[test]
fn a_retry_reaches_what_the_newest_drawing_gives_the_name() {
    let (mut places, made) = taken(3);
    assert!(places.retry_later());
    // Two drawings are made during the pause: the first drops the name, the second gives it to
    // a new thing.
    draw(&mut places, &made, &[Place::Find]);
    let drawn = draw(&mut places, &made, &[Place::Find, Place::Pay]);

    assert_eq!(drawn, [2, 3]);
    assert_eq!(
        places.take_retry(),
        Some(3),
        "the retry reached a thing no longer drawn"
    );
}

#[test]
fn a_retry_for_a_name_the_newest_drawing_does_not_give_ends_the_ask() {
    let (mut places, made) = taken(3);
    assert!(places.retry_later());
    draw(&mut places, &made, &[Place::Find]);

    assert_eq!(places.take_retry(), None);
    // A later drawing gives the name again, when the person has long gone on.
    draw(&mut places, &made, &[Place::Find, Place::Pay]);
    assert!(!places.retry_later(), "an ask whose place went is kept");
    assert_eq!(
        places.take_retry(),
        None,
        "a later drawing brought the ask back"
    );
}

#[test]
fn an_ask_for_a_place_that_is_not_drawn_is_never_retried() {
    let (mut places, made) = (Places::with_retries(3), Made::default());
    places.ask(Place::Pay);
    draw(&mut places, &made, &[Place::Find]);
    assert_eq!(places.take_asked(), None);

    assert!(!places.retry_later(), "an ask that came to nothing is kept");
}

/// What one drawing gives a name.
#[derive(Clone, Copy)]
enum Gives {
    Control,
    Place,
    Both,
    PlaceThenControl,
    Nothing,
}

/// One drawing that gives `name` as `gives` says, and the references it got: the control's, then
/// the place's.
fn give(
    places: &mut Places<u32>,
    made: &Made,
    name: Place,
    gives: Gives,
) -> (Option<u32>, Option<u32>) {
    places.begin();
    let got = match gives {
        Gives::Control => (Some(places.named(name, || made.next())), None),
        Gives::Place => (None, Some(places.place(name, || made.next()))),
        Gives::Both => {
            let control = places.named(name, || made.next());
            (Some(control), Some(places.place(name, || made.next())))
        }
        Gives::PlaceThenControl => {
            let place = places.place(name, || made.next());
            (Some(places.named(name, || made.next())), Some(place))
        }
        Gives::Nothing => (None, None),
    };
    places.end();
    got
}

#[test]
fn a_name_given_to_a_control_alone_asks_the_control() {
    let (mut places, made) = (Places::new(), Made::default());
    places.ask(Place::Pay);
    let (control, _) = give(&mut places, &made, Place::Pay, Gives::Control);

    assert!(control.is_some());
    assert_eq!(places.take_asked(), control);
}

#[test]
fn a_name_given_to_a_place_alone_asks_the_place() {
    let (mut places, made) = (Places::new(), Made::default());
    places.ask(Place::Pay);
    let (_, place) = give(&mut places, &made, Place::Pay, Gives::Place);

    assert!(place.is_some());
    assert_eq!(places.take_asked(), place);
}

#[test]
fn a_name_given_to_a_control_and_a_place_asks_the_control_in_either_order_of_giving() {
    for gives in [Gives::Both, Gives::PlaceThenControl] {
        let (mut places, made) = (Places::new(), Made::default());
        places.ask(Place::Pay);
        let (control, place) = give(&mut places, &made, Place::Pay, gives);

        assert_ne!(
            control, place,
            "the control and the place share a reference"
        );
        assert_eq!(
            places.take_asked(),
            control,
            "the place was asked over the control"
        );
    }
}

#[test]
fn a_name_the_newest_drawing_gives_to_nothing_asks_nothing() {
    let (mut places, made) = (Places::with_retries(3), Made::default());
    give(&mut places, &made, Place::Pay, Gives::Both);
    places.ask(Place::Pay);
    give(&mut places, &made, Place::Pay, Gives::Nothing);

    assert_eq!(places.take_asked(), None);
    assert!(
        !places.retry_later(),
        "an ask of nothing is kept to be retried"
    );
}

#[test]
fn the_newest_drawing_decides_whether_the_control_or_the_place_is_asked() {
    let (mut places, made) = (Places::new(), Made::default());
    give(&mut places, &made, Place::Pay, Gives::Both);
    let (_, place) = give(&mut places, &made, Place::Pay, Gives::Place);
    places.ask(Place::Pay);
    assert_eq!(
        places.take_asked(),
        place,
        "a control an earlier drawing gave was asked"
    );

    let (control, _) = give(&mut places, &made, Place::Pay, Gives::Control);
    places.ask(Place::Pay);
    assert_eq!(
        places.take_asked(),
        control,
        "a place an earlier drawing gave was asked"
    );
}

#[test]
fn a_retry_settles_whether_the_control_or_the_place_is_asked_against_the_newest_drawing() {
    let (mut places, made) = (Places::with_retries(3), Made::default());
    places.ask(Place::Pay);
    let (_, place) = give(&mut places, &made, Place::Pay, Gives::Place);
    assert_eq!(places.take_asked(), place);

    assert!(places.retry_later());
    let (control, _) = give(&mut places, &made, Place::Pay, Gives::Both);
    assert_eq!(
        places.take_retry(),
        control,
        "the retry went to what the first drawing gave"
    );

    assert!(places.retry_later());
    give(&mut places, &made, Place::Pay, Gives::Nothing);
    assert_eq!(places.take_retry(), None);
}

#[test]
fn an_ask_of_a_place_is_retried_as_often_as_the_drawer_said_and_no_more() {
    let (mut places, made) = (Places::with_retries(2), Made::default());
    places.ask(Place::Pay);
    let (_, place) = give(&mut places, &made, Place::Pay, Gives::Place);
    assert_eq!(places.take_asked(), place);

    let mut retried = 0;
    while places.retry_later() {
        retried += 1;
        assert!(
            retried <= 2,
            "the ask of a place is retried a third time of two"
        );
        assert_eq!(places.take_retry(), place);
    }

    assert_eq!(retried, 2);
}

#[test]
fn each_kind_keeps_its_reference_while_each_drawing_gives_it_and_takes_a_new_one_after_a_gap() {
    let (mut places, made) = (Places::new(), Made::default());
    let first = give(&mut places, &made, Place::Find, Gives::Both);
    assert_eq!(
        give(&mut places, &made, Place::Find, Gives::Both),
        first,
        "a reference changed between drawings that each gave it"
    );

    let (_, place) = give(&mut places, &made, Place::Find, Gives::Place);
    assert_eq!(place, first.1, "the place lost its reference while given");
    let (control, again) = give(&mut places, &made, Place::Find, Gives::Both);
    assert_ne!(
        control, first.0,
        "a control a drawing left out came back with its old reference"
    );
    assert_eq!(again, first.1);
}
