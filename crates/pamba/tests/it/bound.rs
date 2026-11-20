use std::num::NonZeroUsize;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use pamba::{Loop, Manual, Unsent};

use crate::probe::{Message, Probe, Watch, World};

/// Long enough for a send that is not made to wait to have been heard of.
const HEARD: Duration = Duration::from_millis(200);
/// Long enough for anything that is going to happen: past it a case fails where it would hang.
const EVER: Duration = Duration::from_secs(10);

fn most(count: usize) -> NonZeroUsize {
    NonZeroUsize::new(count).unwrap()
}

#[test]
fn a_send_from_another_thread_waits_while_the_mailbox_is_full_and_goes_on_when_room_is_made() {
    let (world, _seen) = World::new();
    let mut probe =
        Loop::start_with_room(Probe::default(), Vec::new(), world, Manual::new(), most(1));
    let sender = probe.sender();
    let (sent, heard) = mpsc::channel();
    thread::spawn(move || {
        for name in ["first", "second"] {
            let outcome = sender.send(Message::Note(name));
            sent.send((name, outcome)).unwrap();
        }
    });

    assert_eq!(heard.recv_timeout(EVER), Ok(("first", Ok(()))));
    assert_eq!(
        heard.recv_timeout(HEARD),
        Err(mpsc::RecvTimeoutError::Timeout),
        "the second send waits: the mailbox holds one"
    );

    assert_eq!(probe.turn(most(1)).stepped, 1);
    assert_eq!(
        heard.recv_timeout(EVER),
        Ok(("second", Ok(()))),
        "room was made, and the send that waited goes on"
    );
    assert_eq!(probe.turn(most(8)).stepped, 1);
    assert_eq!(probe.program().told, [("first", 1), ("second", 2)]);
}

#[test]
fn a_send_from_the_loops_own_thread_never_waits() {
    let (done, heard) = mpsc::channel();
    // The loop's thread is the one that makes room, so a send it was made to wait on would
    // never return: the loop runs on a thread of its own and the case waits for its word.
    thread::spawn(move || {
        let (world, _seen) = World::new();
        let mut probe =
            Loop::start_with_room(Probe::default(), Vec::new(), world, Manual::new(), most(1));
        let sender = probe.sender();
        for name in ["first", "second", "third"] {
            sender.send(Message::Note(name)).unwrap();
        }
        done.send(probe.turn(most(8)).stepped).unwrap();
    });

    assert_eq!(heard.recv_timeout(EVER), Ok(3));
}

#[test]
fn a_sender_waiting_for_room_is_let_go_when_the_loop_ends() {
    let (world, _seen) = World::new();
    let probe = Loop::start_with_room(Probe::default(), Vec::new(), world, Manual::new(), most(1));
    let sender = probe.sender();
    sender.send(Message::Note("fills it")).unwrap();
    let (sent, heard) = mpsc::channel();
    thread::spawn(move || sent.send(sender.send(Message::Note("waits"))).unwrap());
    assert_eq!(
        heard.recv_timeout(HEARD),
        Err(mpsc::RecvTimeoutError::Timeout)
    );

    drop(probe);

    assert_eq!(heard.recv_timeout(EVER), Ok(Err(Unsent::Ended)));
}

#[test]
fn a_sender_waiting_for_room_is_let_go_when_its_subscription_is_stopped() {
    let (world, seen) = World::new();
    let mut probe =
        Loop::start_with_room(Probe::default(), Vec::new(), world, Manual::new(), most(1));
    probe.step(Message::Hold(vec![Watch::A]));
    let a = seen.borrow().senders[&Watch::A].clone();
    probe.sender().send(Message::Note("fills it")).unwrap();
    let (sent, heard) = mpsc::channel();
    thread::spawn(move || sent.send(a.send(Message::Note("waits"))).unwrap());
    assert_eq!(
        heard.recv_timeout(HEARD),
        Err(mpsc::RecvTimeoutError::Timeout)
    );

    // Stepped directly, so the mailbox stays full: only the stop can let the sender go.
    probe.step(Message::Hold(Vec::new()));

    assert_eq!(heard.recv_timeout(EVER), Ok(Err(Unsent::Stopped)));
}
