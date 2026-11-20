use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};

/// How one platform runs something on the loop's thread and off it. A platform's layer
/// implements it once, for every program.
pub trait Dispatcher: Send + Sync {
    /// Asks that the loop's thread call [`Loop::turn`](crate::Loop::turn) soon. Called from any
    /// thread, and not again until a turn has begun.
    fn wake(&self);

    /// Runs `work` on a thread that is not the loop's.
    fn spawn(&self, work: Box<dyn FnOnce() + Send>);
}

/// Why a message was not taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unsent {
    /// The loop has ended.
    Ended,
    /// The subscription this sender was given to has been stopped.
    Stopped,
}

impl fmt::Display for Unsent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Ended => "the loop has ended",
            Self::Stopped => "the subscription has been stopped",
        })
    }
}

impl Error for Unsent {}

pub(crate) struct Envelope<M> {
    pub(crate) message: M,
    /// Raised once the subscription that sent it is stopped.
    pub(crate) stopped: Option<Arc<AtomicBool>>,
}

struct Waiting<M> {
    queue: VecDeque<Envelope<M>>,
    ended: bool,
    /// A wake is asked for and no turn has begun since.
    woken: bool,
}

pub(crate) struct Mailbox<M> {
    waiting: Mutex<Waiting<M>>,
    /// Signalled when the queue has room, a subscription is stopped or the loop ends.
    moved: Condvar,
    dispatcher: Arc<dyn Dispatcher>,
    home: ThreadId,
    room: usize,
}

impl<M> Mailbox<M> {
    /// A mailbox for a loop on the calling thread.
    pub(crate) fn new(dispatcher: Arc<dyn Dispatcher>, room: usize) -> Self {
        Self {
            waiting: Mutex::new(Waiting {
                queue: VecDeque::new(),
                ended: false,
                woken: false,
            }),
            moved: Condvar::new(),
            dispatcher,
            home: thread::current().id(),
            room,
        }
    }

    // A panic while the lock is held leaves the queue whole: every section under it is a push,
    // a pop or a flag.
    fn waiting(&self) -> MutexGuard<'_, Waiting<M>> {
        self.waiting.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A turn begins: a send from here on asks for another.
    pub(crate) fn begin(&self) {
        self.waiting().woken = false;
    }

    pub(crate) fn take(&self) -> Option<Envelope<M>> {
        let taken = self.waiting().queue.pop_front();
        if taken.is_some() {
            self.moved.notify_one();
        }
        taken
    }

    /// Asks for another turn where messages are left and none is asked for; says whether any are
    /// left.
    pub(crate) fn rest(&self) -> bool {
        let (left, wake) = {
            let mut waiting = self.waiting();
            let left = !waiting.queue.is_empty();
            let wake = left && !waiting.woken;
            waiting.woken |= wake;
            (left, wake)
        };
        if wake {
            self.dispatcher.wake();
        }
        left
    }

    /// Lets every sender waiting for room look again: one of them may now be stopped.
    pub(crate) fn stopped(&self) {
        self.moved.notify_all();
    }

    pub(crate) fn end(&self) {
        let mut waiting = self.waiting();
        waiting.ended = true;
        waiting.queue.clear();
        drop(waiting);
        self.moved.notify_all();
    }

    fn put(&self, message: M, stopped: Option<&Arc<AtomicBool>>) -> Result<(), Unsent> {
        let gone = || stopped.is_some_and(|stopped| stopped.load(Ordering::Acquire));
        let mut waiting = self.waiting();
        loop {
            if waiting.ended {
                return Err(Unsent::Ended);
            }
            if gone() {
                return Err(Unsent::Stopped);
            }
            // The loop's own thread is the one that makes room, so it is never made to wait.
            if waiting.queue.len() < self.room || thread::current().id() == self.home {
                break;
            }
            waiting = self
                .moved
                .wait(waiting)
                .unwrap_or_else(PoisonError::into_inner);
        }
        waiting.queue.push_back(Envelope {
            message,
            stopped: stopped.cloned(),
        });
        let wake = !waiting.woken;
        waiting.woken = true;
        drop(waiting);
        if wake {
            self.dispatcher.wake();
        }
        Ok(())
    }
}

/// Sends messages to a program's loop, from any thread.
///
/// Messages from one sender reach [`update`](crate::Program::update) in the order sent. The
/// mailbox holds a bounded number: a send from another thread waits while it is full, so a source
/// that outruns the loop is slowed and nothing is dropped. A send from the loop's own thread
/// never waits.
pub struct Sender<M> {
    mailbox: Arc<Mailbox<M>>,
    stopped: Option<Arc<AtomicBool>>,
}

impl<M> Sender<M> {
    pub(crate) fn new(mailbox: Arc<Mailbox<M>>, stopped: Option<Arc<AtomicBool>>) -> Self {
        Self { mailbox, stopped }
    }
}

impl<M> Clone for Sender<M> {
    fn clone(&self) -> Self {
        Self {
            mailbox: Arc::clone(&self.mailbox),
            stopped: self.stopped.clone(),
        }
    }
}

impl<M> fmt::Debug for Sender<M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Sender").finish_non_exhaustive()
    }
}

impl<M: Send + 'static> Sender<M> {
    /// Queues `message` for the loop.
    ///
    /// # Errors
    ///
    /// [`Unsent::Ended`] once the loop has ended, and [`Unsent::Stopped`] once the subscription
    /// this sender was given to has been stopped. Either is final: the source ends.
    pub fn send(&self, message: M) -> Result<(), Unsent> {
        self.mailbox.put(message, self.stopped.as_ref())
    }

    /// Runs `work` off the loop's thread and sends what it returns.
    pub fn spawn(&self, work: impl FnOnce() -> M + Send + 'static) {
        let sender = self.clone();
        self.mailbox.dispatcher.spawn(Box::new(move || {
            // A loop that has ended, or a subscription stopped, wants nothing of the outcome.
            let _ = sender.send(work());
        }));
    }
}
