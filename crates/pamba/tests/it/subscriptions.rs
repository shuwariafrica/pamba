use std::num::NonZeroUsize;

use std::cell::Cell;
use std::rc::Rc;

use pamba::{Loop, Manual, Running, Unsent};

use crate::probe::{Asked, Effect, Message, Probe, Watch, World};

fn most(count: usize) -> NonZeroUsize {
    NonZeroUsize::new(count).unwrap()
}

#[test]
fn what_runs_is_brought_into_line_with_the_state_before_the_steps_effects_are_performed() {
    let (world, seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());

    probe.step(Message::Both(vec![Watch::A], vec![Effect::Say("with a")]));
    probe.step(Message::Both(Vec::new(), vec![Effect::Say("without")]));

    assert_eq!(
        seen.borrow().asked,
        [
            Asked::Subscribed(Watch::A),
            Asked::Performed(Effect::Say("with a")),
            Asked::Stopped(Watch::A),
            Asked::Performed(Effect::Say("without")),
        ]
    );
}

#[test]
fn a_subscription_still_held_keeps_running_and_only_the_difference_is_started_and_stopped() {
    let (world, seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());

    probe.step(Message::Hold(vec![Watch::A, Watch::B]));
    probe.step(Message::Hold(vec![Watch::B, Watch::C]));
    probe.step(Message::Note("nothing changes"));

    assert_eq!(
        seen.borrow().asked,
        [
            Asked::Subscribed(Watch::A),
            Asked::Subscribed(Watch::B),
            Asked::Stopped(Watch::A),
            Asked::Subscribed(Watch::C),
        ]
    );
}

#[test]
fn subscriptions_start_in_the_order_stated_once_each_and_stop_in_the_reverse_of_it() {
    let (world, seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());

    probe.step(Message::Hold(vec![Watch::B, Watch::A, Watch::B, Watch::C]));
    probe.step(Message::Hold(Vec::new()));

    assert_eq!(
        seen.borrow().asked,
        [
            Asked::Subscribed(Watch::B),
            Asked::Subscribed(Watch::A),
            Asked::Subscribed(Watch::C),
            Asked::Stopped(Watch::C),
            Asked::Stopped(Watch::A),
            Asked::Stopped(Watch::B),
        ]
    );
}

#[test]
fn a_subscription_let_go_and_taken_up_again_is_started_anew() {
    let (world, seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());

    probe.step(Message::Hold(vec![Watch::A]));
    let first = seen.borrow().senders[&Watch::A].clone();
    probe.step(Message::Hold(Vec::new()));
    probe.step(Message::Hold(vec![Watch::A]));
    let second = seen.borrow().senders[&Watch::A].clone();

    assert_eq!(first.send(Message::Note("old")), Err(Unsent::Stopped));
    assert_eq!(second.send(Message::Note("new")), Ok(()));
    probe.turn(most(8));
    assert_eq!(probe.program().told.last(), Some(&("new", 4)));
}

#[test]
fn nothing_a_stopped_subscription_sent_reaches_the_program() {
    let (world, seen) = World::new();
    let threads = Manual::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, threads.clone());
    probe.step(Message::Hold(vec![Watch::A, Watch::B]));
    let (a, b) = {
        let seen = seen.borrow();
        (
            seen.senders[&Watch::A].clone(),
            seen.senders[&Watch::B].clone(),
        )
    };

    // Queued while A still runs, behind the message whose step stops it.
    b.send(Message::Hold(vec![Watch::B])).unwrap();
    a.send(Message::Note("from a, queued before its stop"))
        .unwrap();
    b.send(Message::Note("from b")).unwrap();
    threads.settle(&mut probe, 8).unwrap();

    assert_eq!(
        probe.program().told,
        [("hold", 1), ("hold", 2), ("from b", 3)]
    );
    assert_eq!(
        a.send(Message::Note("from a, after its stop")),
        Err(Unsent::Stopped)
    );
}

#[test]
fn ending_the_loop_stops_what_runs_in_the_reverse_of_the_order_started() {
    let (world, seen) = World::new();
    let mut probe = Loop::start(Probe::default(), Vec::new(), world, Manual::new());
    probe.step(Message::Hold(vec![Watch::A, Watch::B]));
    let a = seen.borrow().senders[&Watch::A].clone();

    drop(probe);

    assert_eq!(
        seen.borrow().asked,
        [
            Asked::Subscribed(Watch::A),
            Asked::Subscribed(Watch::B),
            Asked::Stopped(Watch::B),
            Asked::Stopped(Watch::A),
        ]
    );
    assert_eq!(a.send(Message::Note("late")), Err(Unsent::Ended));
}

#[test]
fn a_subscription_held_by_a_value_is_stopped_by_dropping_the_value() {
    struct Source(Rc<Cell<bool>>);
    impl Drop for Source {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    let ended = Rc::new(Cell::new(false));

    let running = Running::holding(Source(Rc::clone(&ended)));
    assert!(!ended.get());
    drop(running);

    assert!(ended.get());
}
