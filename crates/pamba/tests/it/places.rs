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
