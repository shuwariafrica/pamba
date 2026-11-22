use std::num::NonZeroUsize;

use pamba::{Loop, Manual, Unsent};

use crate::probe::{Message, Probe, Watch, World};

fn most(count: usize) -> NonZeroUsize {
    NonZeroUsize::new(count).unwrap()
}

fn names(probe: &Probe) -> Vec<&'static str> {
    probe.told.iter().map(|(name, _)| *name).collect()
}

#[test]
fn what_a_part_sends_reaches_the_program_wrapped_in_the_order_sent_among_the_whole_senders() {
    let (world, _seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());
    let whole = probe.sender();
    let part = whole.wrapping(Message::Note);

    whole.send(Message::Note("first")).unwrap();
    part.send("second").unwrap();
    whole.send(Message::Note("third")).unwrap();
    part.send("fourth").unwrap();
    probe.turn(most(8));

    assert_eq!(
        names(probe.program()),
        ["first", "second", "third", "fourth"]
    );
}

#[test]
fn a_part_of_a_part_reaches_the_program_wrapped_by_each() {
    let (world, _seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());
    let part = probe.sender().wrapping(Message::Note);
    let inner = part.wrapping(|long: bool| if long { "long" } else { "short" });

    inner.send(true).unwrap();
    part.send("between").unwrap();
    inner.send(false).unwrap();
    probe.turn(most(8));

    assert_eq!(names(probe.program()), ["long", "between", "short"]);
}

#[test]
fn a_part_of_a_stopped_subscriptions_sender_is_refused_as_that_sender_is() {
    let (world, seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());
    probe.step(Message::Hold(vec![Watch::A]));
    let part = seen.borrow().senders[&Watch::A].wrapping(Message::Note);

    assert_eq!(part.send("while it runs"), Ok(()));
    probe.step(Message::Hold(Vec::new()));

    assert_eq!(part.send("after its stop"), Err(Unsent::Stopped));
    probe.turn(most(8));
    assert!(!names(probe.program()).contains(&"after its stop"));
}

#[test]
fn a_part_is_refused_once_the_loop_has_ended() {
    let (world, _seen) = World::new();
    let probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());
    let part = probe.sender().wrapping(Message::Note);

    drop(probe);

    assert_eq!(part.send("too late"), Err(Unsent::Ended));
}

#[test]
fn work_spawned_through_a_part_runs_off_the_loop_and_what_it_returns_arrives_wrapped() {
    let (world, _seen) = World::new();
    let threads = Manual::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, threads.clone());
    let part = probe.sender().wrapping(Message::Note);

    part.spawn(|| "worked");

    // Nothing ran where it was asked: the work waits for a thread that is not the loop's.
    probe.turn(most(8));
    assert_eq!(names(probe.program()), Vec::<&str>::new());
    assert_eq!(threads.run(), 1);
    probe.turn(most(8));
    assert_eq!(names(probe.program()), ["worked"]);
}
