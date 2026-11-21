use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use crate::mailbox::Dispatcher;
use crate::performer::Performer;
use crate::program::Program;
use crate::run::Loop;

/// A [`Dispatcher`] driven by hand, for running a program and its performer with no window and
/// no second thread: it notes a wake and keeps what is spawned until it is told to run it, on the
/// calling thread, so a case decides the order everything happens in.
#[derive(Default)]
pub struct Manual {
    woken: AtomicBool,
    work: Mutex<VecDeque<Box<dyn FnOnce() + Send>>>,
}

/// A program that was still asking for work after the steps a case allowed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsettled {
    /// How many steps were taken before the case gave up.
    pub stepped: usize,
}

impl fmt::Display for Unsettled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "still stepping after {} steps", self.stepped)
    }
}

impl Error for Unsettled {}

impl Manual {
    /// A dispatcher with nothing spawned and no wake noted.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Whether a wake was asked for since this was last read.
    pub fn woken(&self) -> bool {
        self.woken.swap(false, Ordering::AcqRel)
    }

    /// Runs what was spawned so far, in order, on the calling thread; returns how many ran.
    pub fn run(&self) -> usize {
        let mut ran = 0;
        loop {
            let next = self
                .work
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .pop_front();
            let Some(work) = next else {
                return ran;
            };
            work();
            ran += 1;
        }
    }

    /// Runs spawned work and takes queued messages, turn about, until neither is left; returns
    /// how many steps that took.
    ///
    /// # Errors
    ///
    /// [`Unsettled`] where `program` is still stepping after `most` steps: an effect whose
    /// message asks for the effect again never settles.
    pub fn settle<P: Program, W: Performer<P>>(
        &self,
        program: &mut Loop<P, W>,
        most: usize,
    ) -> Result<usize, Unsettled> {
        let mut stepped = 0;
        loop {
            let ran = self.run();
            let turn = program.turn(NonZeroUsize::MIN);
            stepped += turn.stepped;
            if stepped > most {
                return Err(Unsettled { stepped });
            }
            if ran == 0 && turn.stepped == 0 && !turn.left {
                self.woken.store(false, Ordering::Release);
                return Ok(stepped);
            }
        }
    }
}

impl Dispatcher for Manual {
    fn wake(&self) {
        self.woken.store(true, Ordering::Release);
    }

    fn spawn(&self, work: Box<dyn FnOnce() + Send>) {
        self.work
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push_back(work);
    }
}

impl fmt::Debug for Manual {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Manual").finish_non_exhaustive()
    }
}
