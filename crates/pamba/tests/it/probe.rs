use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use pamba::{Performer, Program, Running, Sender};

/// A program that keeps what it was told, holds the subscriptions it was last told to hold and
/// asks for the effects a message carries.
#[derive(Debug, Default)]
pub(crate) struct Probe {
    /// Each message's name with the moment its step ran at.
    pub(crate) told: Vec<(&'static str, u32)>,
    pub(crate) held: Vec<Watch>,
}

#[derive(Debug)]
pub(crate) enum Message {
    Note(&'static str),
    Hold(Vec<Watch>),
    Ask(Vec<Effect>),
    /// Holds and asks in one step.
    Both(Vec<Watch>, Vec<Effect>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Effect {
    /// Written down and nothing more.
    Say(&'static str),
    /// Answered at once, on the loop's thread, with a note of the same name.
    Echo(&'static str),
    /// Answered by spawned work with a note of the same name.
    Later(&'static str),
    /// Answered at once with a message that asks for it again.
    Again,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Watch {
    A,
    B,
    C,
}

impl Program for Probe {
    type Message = Message;
    type Effect = Effect;
    type Subscription = Watch;
    type Moment = u32;

    fn update(&mut self, message: Message, now: &u32) -> Vec<Effect> {
        match message {
            Message::Note(name) => {
                self.told.push((name, *now));
                Vec::new()
            }
            Message::Hold(held) => {
                self.told.push(("hold", *now));
                self.held = held;
                Vec::new()
            }
            Message::Ask(effects) => {
                self.told.push(("ask", *now));
                effects
            }
            Message::Both(held, effects) => {
                self.told.push(("both", *now));
                self.held = held;
                effects
            }
        }
    }

    fn subscriptions(&self) -> Vec<Watch> {
        self.held.clone()
    }
}

/// How often [`Effect::Again`] may be performed before the performer itself fails the case: a
/// settling that does not end fails here, inside its loop, and never hangs.
pub(crate) const AGAIN: usize = 16;

/// What the performer was asked, in the order it was asked.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Asked {
    Performed(Effect),
    Subscribed(Watch),
    Stopped(Watch),
}

/// What a case reads of the performer after the loop has taken it.
#[derive(Debug, Default)]
pub(crate) struct Seen {
    pub(crate) asked: Vec<Asked>,
    /// The sender each subscription was started with, kept after it is stopped.
    pub(crate) senders: BTreeMap<Watch, Sender<Message>>,
}

#[derive(Debug, Default)]
pub(crate) struct World {
    moments: u32,
    again: usize,
    seen: Rc<RefCell<Seen>>,
}

impl World {
    pub(crate) fn new() -> (Self, Rc<RefCell<Seen>>) {
        let world = Self::default();
        let seen = Rc::clone(&world.seen);
        (world, seen)
    }
}

impl Performer<Probe> for World {
    fn moment(&mut self) -> u32 {
        self.moments += 1;
        self.moments
    }

    fn perform(&mut self, effect: Effect, messages: &Sender<Message>) {
        self.seen
            .borrow_mut()
            .asked
            .push(Asked::Performed(effect.clone()));
        match effect {
            Effect::Say(_) => {}
            Effect::Echo(name) => messages.send(Message::Note(name)).unwrap(),
            Effect::Later(name) => messages.spawn(move || Message::Note(name)),
            Effect::Again => {
                self.again += 1;
                assert!(
                    self.again <= AGAIN,
                    "asked again past the step a case may not pass"
                );
                messages.send(Message::Ask(vec![Effect::Again])).unwrap();
            }
        }
    }

    fn subscribe(&mut self, subscription: &Watch, messages: Sender<Message>) -> Running {
        let watch = *subscription;
        let mut seen = self.seen.borrow_mut();
        seen.asked.push(Asked::Subscribed(watch));
        seen.senders.insert(watch, messages);
        let seen = Rc::clone(&self.seen);
        Running::new(move || seen.borrow_mut().asked.push(Asked::Stopped(watch)))
    }
}
