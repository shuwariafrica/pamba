use std::fmt;

use crate::mailbox::Sender;
use crate::program::{Never, Program};

/// Where a program's side effects happen on one platform: it reads each step's moment, carries
/// out the effects the program asks for and runs its subscriptions.
///
/// The loop calls every method on its own thread, between steps, so none may block: work that
/// waits is moved off the thread with [`Sender::spawn`] or the performer's own means, and its
/// outcome sent as a message. What a performer holds - handles, connections, a store - is
/// invisible to the program.
pub trait Performer<P: Program> {
    /// The moment the next step runs at.
    fn moment(&mut self) -> P::Moment;

    /// Carries out `effect`. Whatever comes of it, a failure included, is sent through `messages`
    /// as a message of the program's; one sent before `perform` returns is taken after the
    /// present step's remaining effects.
    fn perform(&mut self, effect: P::Effect, messages: &Sender<P::Message>);

    /// Starts `subscription`, which sends through `messages` until the [`Running`] returned is
    /// dropped. A send after that is refused with [`Unsent::Stopped`](crate::Unsent::Stopped),
    /// which tells its source to end.
    fn subscribe(
        &mut self,
        subscription: &P::Subscription,
        messages: Sender<P::Message>,
    ) -> Running;
}

/// Performs for a program that asks for no effect, holds no subscription and reads no clock.
impl<P> Performer<P> for ()
where
    P: Program<Effect = Never, Subscription = Never, Moment = ()>,
{
    fn moment(&mut self) {}

    fn perform(&mut self, effect: Never, _messages: &Sender<P::Message>) {
        match effect {}
    }

    fn subscribe(&mut self, subscription: &Never, _messages: Sender<P::Message>) -> Running {
        match *subscription {}
    }
}

/// A subscription while it runs. The loop drops it to stop the subscription.
pub struct Running {
    stop: Option<Box<dyn FnOnce()>>,
}

impl Running {
    /// A subscription stopped by `stop`, which runs once, on the loop's thread, and must neither
    /// block nor panic: it raises a signal or closes a handle, and the source ends by itself.
    #[must_use]
    pub fn new(stop: impl FnOnce() + 'static) -> Self {
        Self {
            stop: Some(Box::new(stop)),
        }
    }

    /// A subscription that runs for as long as `held` lives, and is stopped by dropping it.
    #[must_use]
    pub fn holding<T: 'static>(held: T) -> Self {
        Self::new(move || drop(held))
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            stop();
        }
    }
}

impl fmt::Debug for Running {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Running").finish_non_exhaustive()
    }
}
