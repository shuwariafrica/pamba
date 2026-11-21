use std::fmt;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::mailbox::{Dispatcher, Mailbox, Sender};
use crate::performer::{Performer, Running};
use crate::program::Program;

/// How many messages a mailbox holds unless [`Loop::start_with_room`] says otherwise. A burst a
/// connection or a file watch delivers at once is in the hundreds; past this a source waits.
pub const ROOM: NonZeroUsize = match NonZeroUsize::new(1024) {
    Some(room) => room,
    None => unreachable!(),
};

struct Held<S> {
    subscription: S,
    stopped: Arc<AtomicBool>,
    // Dropped to stop it.
    _running: Running,
}

/// What one call of [`Loop::turn`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turn {
    /// How many messages reached [`update`](Program::update): where it is none, nothing the
    /// program shows has changed.
    pub stepped: usize,
    /// Messages are still queued, and another turn is already asked for.
    pub left: bool,
}

/// A program running: one message at a time through [`update`](Program::update), each step's
/// effects performed and its subscriptions brought into line before the next message is taken.
///
/// The loop owns no thread. A platform's layer creates it on the thread that draws, calls
/// [`step`](Self::step) for what a person does there and [`turn`](Self::turn) whenever its
/// [`Dispatcher`] is woken, and draws from [`program`](Self::program) afterwards. Dropping it
/// stops every subscription and refuses every later send.
pub struct Loop<P: Program, W: Performer<P>> {
    program: P,
    performer: W,
    mailbox: Arc<Mailbox<P::Message>>,
    replies: Sender<P::Message>,
    running: Vec<Held<P::Subscription>>,
}

impl<P: Program, W: Performer<P>> Loop<P, W> {
    /// Starts `program` on the calling thread, which is the loop's from then on: its
    /// subscriptions are started, then `effects`, the ones its construction asked for, are
    /// performed.
    pub fn start(
        program: P,
        effects: Vec<P::Effect>,
        performer: W,
        dispatcher: Arc<dyn Dispatcher>,
    ) -> Self {
        Self::start_with_room(program, effects, performer, dispatcher, ROOM)
    }

    /// As [`start`](Self::start), with a mailbox that holds `room` messages before a source on
    /// another thread waits.
    pub fn start_with_room(
        program: P,
        effects: Vec<P::Effect>,
        performer: W,
        dispatcher: Arc<dyn Dispatcher>,
        room: NonZeroUsize,
    ) -> Self {
        let mailbox = Arc::new(Mailbox::new(dispatcher, room.get()));
        let replies = Sender::new(Arc::clone(&mailbox), None);
        let mut started = Self {
            program,
            performer,
            mailbox,
            replies,
            running: Vec::new(),
        };
        started.settle(effects);
        started
    }

    /// The program as its last step left it: what a platform's layer draws from.
    pub fn program(&self) -> &P {
        &self.program
    }

    /// A sender for a source the platform's layer runs itself, outside any subscription.
    pub fn sender(&self) -> Sender<P::Message> {
        self.replies.clone()
    }

    /// Takes `message` now, ahead of what is queued: for what a person does on the loop's own
    /// thread.
    pub fn step(&mut self, message: P::Message) {
        let now = self.performer.moment();
        let effects = self.program.update(message, &now);
        self.settle(effects);
    }

    /// Takes up to `most` queued messages in order, and asks for another turn where more are
    /// left, so a long burst never holds the thread that draws.
    pub fn turn(&mut self, most: NonZeroUsize) -> Turn {
        self.mailbox.begin();
        let mut stepped = 0;
        for _ in 0..most.get() {
            let Some(envelope) = self.mailbox.take() else {
                break;
            };
            // Read here and not when it was sent: an earlier step of this turn may have stopped
            // its subscription.
            if envelope
                .stopped
                .is_some_and(|stopped| stopped.load(Ordering::Acquire))
            {
                continue;
            }
            self.step(envelope.message);
            stepped += 1;
        }
        Turn {
            stepped,
            left: self.mailbox.rest(),
        }
    }

    /// Brings what runs into line with the state, then performs `effects`: an effect therefore
    /// finds exactly the subscriptions of the state that asked for it.
    fn settle(&mut self, effects: Vec<P::Effect>) {
        self.subscribe();
        for effect in effects {
            self.performer.perform(effect, &self.replies);
        }
    }

    fn subscribe(&mut self) {
        let wanted = self.program.subscriptions();
        let unchanged = wanted.len() == self.running.len()
            && wanted
                .iter()
                .zip(&self.running)
                .all(|(wanted, held)| *wanted == held.subscription);
        if unchanged {
            return;
        }
        let (mut kept, dropped): (Vec<_>, Vec<_>) = std::mem::take(&mut self.running)
            .into_iter()
            .partition(|held| wanted.contains(&held.subscription));
        if !dropped.is_empty() {
            for held in dropped.into_iter().rev() {
                held.stopped.store(true, Ordering::Release);
            }
            self.mailbox.stopped();
        }
        let mut next: Vec<Held<P::Subscription>> = Vec::with_capacity(wanted.len());
        for subscription in wanted {
            if next.iter().any(|held| held.subscription == subscription) {
                continue;
            }
            let running = kept
                .iter()
                .position(|held| held.subscription == subscription);
            next.push(match running {
                Some(at) => kept.swap_remove(at),
                None => self.begin(subscription),
            });
        }
        self.running = next;
    }

    fn begin(&mut self, subscription: P::Subscription) -> Held<P::Subscription> {
        let stopped = Arc::new(AtomicBool::new(false));
        let messages = Sender::new(Arc::clone(&self.mailbox), Some(Arc::clone(&stopped)));
        let running = self.performer.subscribe(&subscription, messages);
        Held {
            subscription,
            stopped,
            _running: running,
        }
    }
}

impl<P: Program, W: Performer<P>> Drop for Loop<P, W> {
    fn drop(&mut self) {
        self.mailbox.end();
        // Last started, first stopped.
        while self.running.pop().is_some() {}
    }
}

impl<P: Program, W: Performer<P>> fmt::Debug for Loop<P, W> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Loop")
            .field("running", &self.running.len())
            .finish_non_exhaustive()
    }
}
