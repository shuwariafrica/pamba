use std::num::NonZeroUsize;

use pamba::{Loop, Manual, Never, Program, Turn, Unsent, Unsettled};

use crate::probe::{Asked, Effect, Message, Probe, Watch, World};

fn most(count: usize) -> NonZeroUsize {
    NonZeroUsize::new(count).unwrap()
}

#[test]
fn a_step_runs_at_the_moment_read_for_it_and_performs_its_effects_in_the_order_returned() {
    let (world, seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());

    probe.step(Message::Ask(vec![Effect::Say("one"), Effect::Say("two")]));
    probe.step(Message::Note("after"));

    assert_eq!(probe.program().told, [("ask", 1), ("after", 2)]);
    assert_eq!(
        seen.borrow().asked,
        [
            Asked::Performed(Effect::Say("one")),
            Asked::Performed(Effect::Say("two")),
        ]
    );
}

#[test]
fn the_effects_a_programs_construction_asked_for_are_performed_at_the_start_after_its_subscriptions()
 {
    let (world, seen) = World::new();
    let program = Probe {
        held: vec![Watch::A],
        ..Probe::default()
    };

    let _probe = Loop::start(program, vec![Effect::Say("begun")], world, Manual::new());

    assert_eq!(
        seen.borrow().asked,
        [
            Asked::Subscribed(Watch::A),
            Asked::Performed(Effect::Say("begun")),
        ]
    );
}

#[test]
fn a_message_sent_while_an_effect_is_performed_is_taken_after_the_steps_remaining_effects() {
    let (world, seen) = World::new();
    let threads = Manual::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, threads.clone());

    probe.step(Message::Ask(vec![
        Effect::Echo("echo"),
        Effect::Say("rest"),
    ]));

    // The echo is queued and the step it came from is over: nothing of it is told yet.
    assert_eq!(probe.program().told, [("ask", 1)]);
    assert_eq!(
        seen.borrow().asked,
        [
            Asked::Performed(Effect::Echo("echo")),
            Asked::Performed(Effect::Say("rest")),
        ]
    );
    assert_eq!(threads.settle(&mut probe, 8), Ok(1));
    assert_eq!(probe.program().told, [("ask", 1), ("echo", 2)]);
}

#[test]
fn messages_from_one_sender_are_taken_in_the_order_sent() {
    let (world, _seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());
    let sender = probe.sender();

    for name in ["first", "second", "third"] {
        sender.send(Message::Note(name)).unwrap();
    }
    let turn = probe.turn(most(8));

    assert_eq!(
        turn,
        Turn {
            stepped: 3,
            left: false
        }
    );
    assert_eq!(
        probe.program().told,
        [("first", 1), ("second", 2), ("third", 3)]
    );
}

#[test]
fn a_turn_takes_no_more_than_it_is_allowed_and_asks_for_another_where_more_are_left() {
    let (world, _seen) = World::new();
    let threads = Manual::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, threads.clone());
    let sender = probe.sender();
    for name in ["first", "second", "third"] {
        sender.send(Message::Note(name)).unwrap();
    }
    assert!(threads.woken());

    let turn = probe.turn(most(2));

    assert_eq!(
        turn,
        Turn {
            stepped: 2,
            left: true
        }
    );
    assert_eq!(probe.program().told, [("first", 1), ("second", 2)]);
    assert!(
        threads.woken(),
        "the turn that left a message asks for another"
    );

    let turn = probe.turn(most(2));

    assert_eq!(
        turn,
        Turn {
            stepped: 1,
            left: false
        }
    );
    assert!(!threads.woken(), "an emptied mailbox asks for nothing");
}

#[test]
fn a_wake_is_asked_once_until_a_turn_begins() {
    let (world, _seen) = World::new();
    let threads = Manual::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, threads.clone());
    let sender = probe.sender();

    sender.send(Message::Note("first")).unwrap();
    assert!(threads.woken(), "a send to an empty mailbox wakes the loop");
    sender.send(Message::Note("second")).unwrap();
    assert!(!threads.woken(), "a second send before any turn does not");

    probe.turn(most(8));
    sender.send(Message::Note("third")).unwrap();
    assert!(
        threads.woken(),
        "a send after a turn has begun wakes it again"
    );
}

#[test]
fn a_message_stepped_directly_is_taken_ahead_of_what_is_queued() {
    let (world, _seen) = World::new();
    let threads = Manual::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, threads.clone());

    probe.sender().send(Message::Note("queued")).unwrap();
    probe.step(Message::Note("pressed"));
    threads.settle(&mut probe, 8).unwrap();

    assert_eq!(probe.program().told, [("pressed", 1), ("queued", 2)]);
}

#[test]
fn spawned_work_runs_off_the_step_in_the_order_spawned_and_what_it_returns_is_sent() {
    let (world, _seen) = World::new();
    let threads = Manual::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, threads.clone());

    probe.step(Message::Ask(vec![
        Effect::Later("one"),
        Effect::Later("two"),
    ]));

    assert_eq!(probe.program().told, [("ask", 1)], "the work has not run");
    assert_eq!(threads.run(), 2);
    assert_eq!(probe.turn(most(8)).stepped, 2);
    assert_eq!(probe.program().told, [("ask", 1), ("one", 2), ("two", 3)]);
}

#[test]
fn a_program_that_never_settles_is_given_up_on_at_the_step_it_may_not_pass() {
    let (world, seen) = World::new();
    let threads = Manual::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, threads.clone());

    probe.step(Message::Ask(vec![Effect::Again]));
    let settled = threads.settle(&mut probe, 5);

    assert_eq!(settled, Err(Unsettled { stepped: 6 }));
    // One step before the settling and the six it took, and not one more.
    assert_eq!(seen.borrow().asked.len(), 7);
}

#[test]
fn once_the_loop_has_ended_a_send_is_refused() {
    let (world, _seen) = World::new();
    let probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());
    let sender = probe.sender();

    drop(probe);

    assert_eq!(sender.send(Message::Note("late")), Err(Unsent::Ended));
}

/// A program with no effect, no subscription and no clock takes `()` as its performer.
struct Counter(u32);

impl Program for Counter {
    type Message = u32;
    type Effect = Never;
    type Subscription = Never;
    type Moment = ();

    fn update(&mut self, by: u32, _now: &()) -> Vec<Never> {
        self.0 += by;
        Vec::new()
    }
}

#[test]
fn a_program_of_state_alone_runs_with_no_performer_of_its_own() {
    let mut counter = Loop::start(Counter(0), Vec::new(), (), Manual::new());

    counter.step(2);
    counter.sender().send(3).unwrap();
    counter.turn(most(8));

    assert_eq!(counter.program().0, 5);
}
