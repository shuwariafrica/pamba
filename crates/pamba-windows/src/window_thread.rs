//! Work asked of the window's thread from any thread.
//!
//! The reactor carries to the window's thread, from another, only what a background task returns
//! when it ends, so work asked of the window is kept here, and the window's thread is woken by the
//! loop's own wake to take it. The reactor refuses work for a window beyond a bound of its own,
//! so what is kept goes to it as one hand-over at a time, which runs a bounded count of pieces.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::mem;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::beside::WindowHandle;

/// A piece of work asked of the window's thread.
pub(crate) type Work = Box<dyn for<'a> FnOnce(WindowHandle<'a>) + Send>;

/// How the window's thread is woken to take what was asked.
type Wake = Arc<dyn Fn() + Send + Sync>;

enum Stage {
    /// No window stands on the thread yet: work is kept until one does.
    Waiting,
    /// The window stands, and each ask wakes its thread.
    Stands(Wake),
    /// The window has gone, or its thread has ended: every ask is refused.
    Gone,
}

struct State {
    work: VecDeque<Work>,
    stage: Stage,
    /// A hand-over lives: it waits with the reactor, or runs.
    handing: bool,
}

/// What the askers share with the window's thread.
struct Kept {
    state: Mutex<State>,
}

impl Kept {
    fn new() -> Self {
        Self {
            state: Mutex::new(State {
                work: VecDeque::new(),
                stage: Stage::Waiting,
                handing: false,
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn ask(&self, work: Work) -> Result<(), Unrun> {
        let mut state = self.lock();
        let wake = match &state.stage {
            Stage::Waiting => None,
            Stage::Stands(wake) => Some(Arc::clone(wake)),
            Stage::Gone => {
                // The lock is let go first: the work's captures may ask again as they drop.
                drop(state);
                drop(work);
                return Err(Unrun);
            }
        };
        state.work.push_back(work);
        drop(state);
        if let Some(wake) = wake {
            wake();
        }
        Ok(())
    }

    fn stands(&self, wake: Wake) {
        self.lock().stage = Stage::Stands(wake);
    }

    /// A hand-over where one is owed: work waits, and no hand-over lives.
    fn owed(self: &Arc<Self>) -> Option<HandOver> {
        let mut state = self.lock();
        if state.handing || state.work.is_empty() {
            return None;
        }
        state.handing = true;
        drop(state);
        Some(HandOver(Arc::clone(self)))
    }

    /// Hands `to` the pieces asked first, `most` of them at most, in the order asked, with the
    /// lock let go: a piece that asks again while it runs is kept behind what was already asked.
    fn hand(&self, most: NonZeroUsize, mut to: impl FnMut(Work)) {
        let first: Vec<Work> = {
            let mut state = self.lock();
            let count = most.get().min(state.work.len());
            state.work.drain(..count).collect()
        };
        for piece in first {
            to(piece);
        }
    }

    fn gone(&self) {
        let work = {
            let mut state = self.lock();
            state.stage = Stage::Gone;
            mem::take(&mut state.work)
        };
        drop(work);
    }
}

/// One hand-over of what was asked to the window's thread, from when it is owed until it has run
/// or is dropped unrun.
///
/// While one lives no other is owed, so the reactor holds one at a time however much is asked,
/// and its bound on a window's work is never met by what is asked here. One the reactor refuses,
/// or drops because the window is going, loses nothing: what it would have handed stays kept.
pub(crate) struct HandOver(Arc<Kept>);

impl HandOver {
    /// Hands `to` the pieces asked first, `most` of them at most, in the order asked. What waits
    /// beyond them is owed a further hand-over once this one has ended.
    pub(crate) fn hand(self, most: NonZeroUsize, to: impl FnMut(Work)) {
        self.0.hand(most, to);
    }
}

impl Drop for HandOver {
    fn drop(&mut self) {
        self.0.lock().handing = false;
    }
}

/// The keeping of the thread it was made on, ended when that thread's storage is dropped, so a
/// thread that ends without a window lets go of what was asked of it.
struct Held(Arc<Kept>);

impl Drop for Held {
    fn drop(&mut self) {
        self.0.gone();
    }
}

thread_local! {
    /// What is kept for this thread's window, its next one included until it stands.
    static HELD: RefCell<Option<Held>> = const { RefCell::new(None) };
}

/// The keeping of the calling thread, made where it has none.
fn here() -> Arc<Kept> {
    HELD.try_with(|held| {
        Arc::clone(
            &held
                .borrow_mut()
                .get_or_insert_with(|| Held(Arc::new(Kept::new())))
                .0,
        )
    })
    .unwrap_or_else(|_| {
        // Asked while the thread's storage is dropped, as the thread ends: it serves no window.
        let ended = Kept::new();
        ended.gone();
        Arc::new(ended)
    })
}

/// The window stands on this thread: each ask from now wakes it by `wake`.
pub(crate) fn stands(wake: impl Fn() + Send + Sync + 'static) {
    here().stands(Arc::new(wake));
}

/// A hand-over of what is asked of this thread's window, where one is owed: work waits, and no
/// hand-over lives.
pub(crate) fn owed() -> Option<HandOver> {
    here().owed()
}

/// The window has gone: what was asked of it is dropped unrun, every `WindowThread` made for it
/// refuses from now, and one made after serves the thread's next window.
pub(crate) fn gone() {
    let held = HELD
        .try_with(|held| held.borrow_mut().take())
        .ok()
        .flatten();
    // Dropped once the thread's storage is let go: the work's captures may ask again as they drop.
    drop(held);
}

/// The thread a program's window stands on, as other threads ask work of it.
///
/// Work runs on the window's thread between the loop's turns, never inside the program's `update`
/// or a performer's `perform`. A piece of work must return at once, since the window draws and
/// answers the person only between pieces: what waits moves to another thread, and no piece of
/// work runs a message loop of its own.
///
/// A `WindowThread` serves the window that stands on its thread, or the next one to stand there.
/// Once that window has gone, it refuses, and a `WindowThread` made after serves the thread's next
/// window. One made on a thread that never runs a window lets go of its work when that thread ends,
/// except on the process's first thread, whose storage Rust does not promise to drop: there the
/// work may be kept until the process ends.
#[derive(Clone)]
pub struct WindowThread {
    kept: Arc<Kept>,
}

impl WindowThread {
    /// The calling thread: the one [`run`](crate::run) is, or will be, called on.
    ///
    /// # Examples
    ///
    /// A second launch knocks on a thread of its own, and the running program's window is brought
    /// forward on the window's thread:
    ///
    /// ```no_run
    /// use pamba_windows::WindowThread;
    ///
    /// // On the thread that calls `pamba_windows::run`, before it does.
    /// let window = WindowThread::current();
    /// std::thread::spawn(move || {
    ///     // A knock arrives here.
    ///     let _ = window.run(|handle| {
    ///         let _ = handle.bring_forward();
    ///     });
    /// });
    /// ```
    #[must_use]
    pub fn current() -> Self {
        Self { kept: here() }
    }

    /// Asks this thread to run `work` with Windows' handle of the window that stands on it.
    ///
    /// `Ok` says the work is kept to be run, not that it has run: where the window goes before
    /// the work's turn, the work is dropped unrun. Work asked before the window stands runs once
    /// it does, and work asked from inside work runs after what was asked before it, each in the
    /// order asked.
    ///
    /// # Errors
    ///
    /// [`Unrun`] once the window has gone, or its thread has ended: the work is dropped before
    /// this returns, never run.
    ///
    /// # Examples
    ///
    /// A worker that waits asks Windows for something over the window, which Windows takes on the
    /// window's thread alone:
    ///
    /// ```no_run
    /// use std::sync::mpsc;
    ///
    /// use pamba_windows::WindowThread;
    ///
    /// # fn ask_over(_: *mut core::ffi::c_void) {}
    /// # fn main() -> Result<(), pamba_windows::Unrun> {
    /// let window = WindowThread::current();
    /// let (asked, answer) = mpsc::channel();
    /// window.run(move |handle| {
    ///     ask_over(handle.as_raw());
    ///     let _ = asked.send(());
    /// })?;
    /// // Dropped unrun where the window goes first, and then the channel says so.
    /// let ran = answer.recv().is_ok();
    /// # let _ = ran;
    /// # Ok(())
    /// # }
    /// ```
    pub fn run(&self, work: impl FnOnce(WindowHandle<'_>) + Send + 'static) -> Result<(), Unrun> {
        self.kept.ask(Box::new(work))
    }
}

impl fmt::Debug for WindowThread {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WindowThread")
            .finish_non_exhaustive()
    }
}

/// Why work asked of a window's thread was not run: its window has gone, or its thread has ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unrun;

impl fmt::Display for Unrun {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the window has gone, or its thread has ended")
    }
}

impl Error for Unrun {}

#[cfg(test)]
mod tests {
    use std::ptr::null_mut;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::thread;

    use super::*;

    type Log = Arc<Mutex<Vec<u32>>>;

    fn noted(log: &Log) -> Vec<u32> {
        log.lock().unwrap().clone()
    }

    /// Work that notes `number` when it runs.
    fn noting(log: &Log, number: u32) -> Work {
        let log = Arc::clone(log);
        Box::new(move |_| log.lock().unwrap().push(number))
    }

    /// Sets its flag when dropped, so a case sees work let go.
    struct Dropped(Arc<AtomicBool>);

    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    /// Work that holds a `Dropped`, and fails its case if it ever runs.
    fn never(flag: &Arc<AtomicBool>) -> Work {
        let dropped = Dropped(Arc::clone(flag));
        Box::new(move |_| {
            let _held = &dropped;
            panic!("work let go of was run");
        })
    }

    fn handle() -> WindowHandle<'static> {
        WindowHandle::new(null_mut())
    }

    fn run_all(kept: &Kept) {
        kept.hand(NonZeroUsize::MAX, |piece| piece(handle()));
    }

    /// Runs the pieces asked first, `most` of them at most.
    fn run_first(kept: &Kept, most: usize) {
        kept.hand(NonZeroUsize::new(most).unwrap(), |piece| piece(handle()));
    }

    fn counting(wakes: &Arc<AtomicUsize>) -> Wake {
        let wakes = Arc::clone(wakes);
        Arc::new(move || {
            wakes.fetch_add(1, Ordering::SeqCst);
        })
    }

    #[test]
    fn work_asked_before_the_window_stands_runs_once_it_stands_in_the_order_asked() {
        let kept = Kept::new();
        let log = Log::default();
        for number in 1..=3 {
            kept.ask(noting(&log, number)).unwrap();
        }
        assert_eq!(noted(&log), Vec::<u32>::new());
        kept.stands(counting(&Arc::default()));
        run_all(&kept);
        assert_eq!(noted(&log), [1, 2, 3]);
    }

    #[test]
    fn work_asked_while_the_window_stands_wakes_its_thread_once_per_ask_and_runs_in_order() {
        let kept = Kept::new();
        let wakes = Arc::new(AtomicUsize::new(0));
        kept.stands(counting(&wakes));
        let log = Log::default();
        for number in 1..=3 {
            kept.ask(noting(&log, number)).unwrap();
            assert_eq!(
                wakes.load(Ordering::SeqCst),
                usize::try_from(number).unwrap()
            );
        }
        run_all(&kept);
        assert_eq!(noted(&log), [1, 2, 3]);
    }

    #[test]
    fn work_kept_when_the_window_goes_is_dropped_unrun() {
        let kept = Kept::new();
        kept.stands(counting(&Arc::default()));
        let dropped = Arc::new(AtomicBool::new(false));
        kept.ask(never(&dropped)).unwrap();
        kept.gone();
        assert!(dropped.load(Ordering::SeqCst));
        run_all(&kept);
    }

    #[test]
    fn an_ask_once_the_window_has_gone_is_refused_and_its_work_dropped_before_it_returns() {
        let kept = Kept::new();
        kept.stands(counting(&Arc::default()));
        kept.gone();
        let dropped = Arc::new(AtomicBool::new(false));
        assert_eq!(kept.ask(never(&dropped)), Err(Unrun));
        assert!(dropped.load(Ordering::SeqCst));
        run_all(&kept);
    }

    #[test]
    fn work_asked_from_inside_work_runs_after_the_work_already_asked_with_no_lock_held() {
        let kept = Arc::new(Kept::new());
        kept.stands(counting(&Arc::default()));
        let log = Log::default();
        let inner = Arc::clone(&kept);
        let inner_log = Arc::clone(&log);
        kept.ask(Box::new(move |_| {
            assert!(
                inner.state.try_lock().is_ok(),
                "the keeping is locked while work runs"
            );
            inner_log.lock().unwrap().push(1);
            inner.ask(noting(&inner_log, 3)).unwrap();
        }))
        .unwrap();
        kept.ask(noting(&log, 2)).unwrap();
        run_all(&kept);
        assert_eq!(noted(&log), [1, 2]);
        run_all(&kept);
        assert_eq!(noted(&log), [1, 2, 3]);
    }

    #[test]
    fn a_burst_beyond_one_hand_over_is_run_over_several_in_the_order_asked() {
        let kept = Kept::new();
        kept.stands(counting(&Arc::default()));
        let log = Log::default();
        for number in 1..=5 {
            kept.ask(noting(&log, number)).unwrap();
        }
        run_first(&kept, 2);
        assert_eq!(noted(&log), [1, 2]);
        run_first(&kept, 2);
        assert_eq!(noted(&log), [1, 2, 3, 4]);
        run_first(&kept, 2);
        assert_eq!(noted(&log), [1, 2, 3, 4, 5]);
    }

    #[test]
    fn one_hand_over_is_owed_at_a_time_and_one_dropped_unrun_loses_no_work() {
        let kept = Arc::new(Kept::new());
        kept.stands(counting(&Arc::default()));
        assert!(
            kept.owed().is_none(),
            "a hand-over is owed with no work asked"
        );
        let log = Log::default();
        kept.ask(noting(&log, 1)).unwrap();
        kept.ask(noting(&log, 2)).unwrap();
        let first = kept.owed();
        assert!(first.is_some(), "no hand-over is owed for work that waits");
        assert!(
            kept.owed().is_none(),
            "a second hand-over is owed while the first lives"
        );
        // As the reactor drops one it refuses, or one whose window is going.
        drop(first);
        assert_eq!(noted(&log), Vec::<u32>::new());
        kept.owed()
            .expect("no hand-over is owed once the first was dropped unrun")
            .hand(NonZeroUsize::MAX, |piece| piece(handle()));
        assert_eq!(noted(&log), [1, 2]);
        assert!(
            kept.owed().is_none(),
            "a hand-over is owed once all that was asked has run"
        );
    }

    #[test]
    fn a_threads_next_window_is_served_by_a_window_thread_made_after_the_earlier_one_went() {
        stands(|| {});
        let earlier = WindowThread::current();
        gone();
        let next = WindowThread::current();
        let dropped = Arc::new(AtomicBool::new(false));
        assert_eq!(earlier.run(never(&dropped)), Err(Unrun));
        let log = Log::default();
        assert_eq!(next.run(noting(&log, 1)), Ok(()));
        stands(|| {});
        owed()
            .expect("no hand-over is owed for the next window's work")
            .hand(NonZeroUsize::MAX, |piece| piece(handle()));
        assert_eq!(noted(&log), [1]);
    }

    #[test]
    fn a_thread_that_ends_without_a_window_drops_what_was_kept_and_refuses_after() {
        let (made, taken) = mpsc::channel();
        let (end, ended) = mpsc::channel::<()>();
        let thread = thread::spawn(move || {
            made.send(WindowThread::current()).unwrap();
            let _ = ended.recv();
        });
        let window = taken.recv().unwrap();
        let kept = Arc::new(AtomicBool::new(false));
        window.run(never(&kept)).unwrap();
        assert!(!kept.load(Ordering::SeqCst));
        end.send(()).unwrap();
        thread.join().unwrap();
        assert!(kept.load(Ordering::SeqCst), "the work outlived its thread");
        let later = Arc::new(AtomicBool::new(false));
        assert_eq!(window.run(never(&later)), Err(Unrun));
        assert!(later.load(Ordering::SeqCst));
    }
}
