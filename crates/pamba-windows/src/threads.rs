use std::sync::{Condvar, Mutex, PoisonError};

use pamba::Dispatcher;

#[derive(Default)]
struct Asked {
    woken: bool,
    ended: bool,
}

/// The window's thread and Windows' thread pool, as the loop needs them.
///
/// The reactor delivers to the window's thread, from another, only what a background task returns
/// when it ends, so a wake is carried by one task that waits here for it; the window starts the
/// next when it has taken its turn.
#[derive(Default)]
pub(crate) struct Threads {
    asked: Mutex<Asked>,
    told: Condvar,
}

impl Threads {
    /// Waits until a wake is asked for, and takes it; returns at once when the window has gone.
    pub(crate) fn woken(&self) {
        let mut asked = self.asked.lock().unwrap_or_else(PoisonError::into_inner);
        while !asked.woken && !asked.ended {
            asked = self
                .told
                .wait(asked)
                .unwrap_or_else(PoisonError::into_inner);
        }
        asked.woken = false;
    }

    /// The window has gone: the task that waits for a wake is let go.
    pub(crate) fn end(&self) {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .ended = true;
        self.told.notify_all();
    }
}

impl Dispatcher for Threads {
    fn wake(&self) {
        self.asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .woken = true;
        self.told.notify_one();
    }

    fn spawn(&self, work: Box<dyn FnOnce() + Send>) {
        windows_threading::submit(work);
    }
}
