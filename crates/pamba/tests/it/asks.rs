use pamba::{Asks, Places};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Paper {
    Receipt,
    Slip,
}

/// One drawing that gives each name the reference beside it.
fn draw(places: &mut Places<u32>, names: &[(Paper, u32)]) {
    places.begin();
    for (name, reference) in names {
        places.named(*name, || *reference);
    }
    places.end();
}

#[test]
fn an_ask_is_taken_once_with_the_reference_the_drawing_after_it_gave_its_name() {
    let (mut places, mut asks) = (Places::new(), Asks::new());

    asks.ask(Paper::Receipt, "print");
    // The drawing that follows is the first to show the thing asked of.
    draw(&mut places, &[(Paper::Receipt, 7)]);

    assert_eq!(asks.take(&places), [(Some(7), "print")]);
    assert_eq!(asks.take(&places), [], "an ask is taken twice");
}

#[test]
fn an_ask_of_a_thing_the_drawing_after_it_does_not_give_is_taken_with_no_reference() {
    let (mut places, mut asks) = (Places::new(), Asks::new());

    asks.ask(Paper::Receipt, "print");
    draw(&mut places, &[(Paper::Slip, 1)]);
    let taken = asks.take(&places);
    // A later drawing gives the name, when the person has long gone on.
    draw(&mut places, &[(Paper::Receipt, 2)]);

    assert_eq!(
        taken,
        [(None, "print")],
        "an ask nothing answers is kept from its asker"
    );
    assert_eq!(
        asks.take(&places),
        [],
        "a later drawing brought the ask back"
    );
}

#[test]
fn every_ask_is_taken_in_the_order_it_was_made_and_none_replaces_another() {
    let (mut places, mut asks) = (Places::new(), Asks::new());

    asks.ask(Paper::Receipt, "first");
    asks.ask(Paper::Slip, "second");
    asks.ask(Paper::Receipt, "third");
    draw(&mut places, &[(Paper::Receipt, 1), (Paper::Slip, 2)]);

    assert_eq!(
        asks.take(&places),
        [(Some(1), "first"), (Some(2), "second"), (Some(1), "third")]
    );
}

#[test]
fn each_ask_is_counted_and_taking_them_counts_nothing() {
    let (mut places, mut asks) = (Places::new(), Asks::new());
    let before = asks.made();

    asks.ask(Paper::Receipt, ());
    assert_eq!(asks.made(), before + 1, "an ask is not counted");
    asks.ask(Paper::Receipt, ());
    assert_eq!(
        asks.made(),
        before + 2,
        "a second ask of the same thing is not counted"
    );
    draw(&mut places, &[(Paper::Receipt, 1)]);
    let taken = asks.take(&places);

    assert_eq!(taken.len(), 2);
    assert_eq!(
        asks.made(),
        before + 2,
        "taking the asks changed how many were made"
    );
}

#[test]
fn an_ask_of_a_name_given_to_a_control_and_a_place_is_taken_with_the_controls_reference() {
    let (mut places, mut asks) = (Places::new(), Asks::new());

    asks.ask(Paper::Receipt, "print");
    places.begin();
    places.place(Paper::Receipt, || 3);
    places.named(Paper::Receipt, || 7);
    places.end();

    assert_eq!(
        asks.take(&places),
        [(Some(7), "print")],
        "the place's reference was given over the control's"
    );
}
