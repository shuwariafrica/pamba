//! Work asked of the window's thread from other threads, and the window brought forward, shown on
//! a desktop.
//!
//! Before `run` is called, a thread of its own asks one piece of work of the thread that will run
//! the window. Once the window stands, four presses each start what a program would:
//!
//! - "Ask three" asks three numbered pieces from a thread of its own;
//! - "Ask a burst" asks twenty thousand pieces at once from a thread of its own, far more than
//!   `windows-reactor` queues for one window, and notes how many ran, in which order, and how many
//!   were dropped unrun;
//! - "Bring forward soon" waits five seconds on a thread of its own, as a later launch's knock
//!   would arrive, then asks for the window to be brought forward;
//! - "Close with work kept" runs a piece that asks one more and then closes the window within its
//!   own call, with a second piece asked straight behind it: neither of the two it leaves is run
//!   once Windows no longer knows the window.
//!
//! Once the window has closed, a `WindowThread` made before `run` asks once more.
//!
//! Every thing it observes is appended, a line each, to the file its first argument names. On a
//! signed-in Windows desktop at 100 per cent scale, start it with the path of a log, then
//! `hand.ps1` with `steps.txt` (both beside this crate's manifest). The log is then
//! `expected-log.txt` line for line, and what the hand read is `expected-answers.txt`. The second
//! and third brings are asked while another program's window stands in front and no right to the
//! foreground was handed on, the second with the window minimised and the third with it open.
//! Windows' documentation promises nothing of the second, and of the third only that Windows may
//! refuse: their lines in both files are what the first run on that desktop saw.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!(
        "forward shows work asked of the window's thread and the window brought forward, and runs on Windows alone"
    );
}

#[cfg(windows)]
fn main() -> Result<(), pamba_windows::Fault> {
    shown::run()
}

#[cfg(windows)]
mod shown {
    use std::ffi::c_void;
    use std::fs::OpenOptions;
    use std::io::Write as _;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, OnceLock};
    use std::thread::{self, ThreadId};
    use std::time::{Duration, Instant};

    use pamba::{Never, Performer, Program, Running, Sender};
    use pamba_windows::ui::{
        Button, Orientation, StackPanel, Thickness, WindowPlacement, WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window, WindowHandle, WindowThread};

    /// The message the close button sends a window.
    const CLOSE: u32 = 0x0010;
    /// How long a knock takes to arrive after "Bring forward soon" is pressed: longer than the hand
    /// takes to minimise the window, or to put another program in front.
    const KNOCK: Duration = Duration::from_secs(5);
    /// How many pieces "Ask a burst" asks at once.
    const BURST: usize = 20_000;
    /// How long the burst's asker waits for every piece to have run or been dropped.
    const SETTLE: Duration = Duration::from_secs(20);

    #[link(name = "user32")]
    unsafe extern "system" {
        fn IsWindow(window: *mut c_void) -> i32;
        fn GetWindowThreadProcessId(window: *mut c_void, process: *mut u32) -> u32;
        fn SendMessageW(window: *mut c_void, message: u32, first: usize, second: isize) -> isize;
    }

    /// Where the log is, and the thread `run` is called on, for every thread to read.
    static LOG: OnceLock<PathBuf> = OnceLock::new();
    static RUN_ON: OnceLock<ThreadId> = OnceLock::new();

    fn note(line: &str) {
        if let Some(log) = LOG.get()
            && let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log)
        {
            let _ = writeln!(file, "{line}");
        }
    }

    fn on_the_window_thread() -> bool {
        RUN_ON.get() == Some(&thread::current().id())
    }

    /// Whether Windows knows `handle` as a window of this process.
    fn of_this_process(handle: WindowHandle<'_>) -> bool {
        let raw = handle.as_raw();
        let mut process = 0;
        // SAFETY: `raw` is the handle the layer handed this work, on the window's thread, for the
        // call now running; both calls keep nothing of it, and `process` outlives the second.
        let (window, thread) = unsafe {
            (
                IsWindow(raw),
                GetWindowThreadProcessId(raw, &raw mut process),
            )
        };
        window != 0 && thread != 0 && process == std::process::id()
    }

    /// Held by a piece of work, and notes when it is dropped whether that piece ran.
    struct Kept {
        what: &'static str,
        ran: bool,
    }

    impl Drop for Kept {
        fn drop(&mut self) {
            let became = if self.ran { "ran" } else { "was dropped unrun" };
            note(&format!("(e) {} {became}", self.what));
        }
    }

    /// What became of a burst's pieces.
    struct Tally {
        ran: AtomicUsize,
        dropped: AtomicUsize,
        in_order: AtomicBool,
    }

    /// Held by one piece of a burst, and counts the piece as dropped where it never ran.
    struct Counted {
        tally: Arc<Tally>,
        ran: bool,
    }

    impl Drop for Counted {
        fn drop(&mut self) {
            if !self.ran {
                self.tally.dropped.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    struct Forward {
        knocks: u32,
    }

    #[derive(Clone, Copy, Debug)]
    enum Message {
        AskThree,
        AskBurst,
        BringForwardSoon,
        CloseWithWorkKept,
    }

    #[derive(Debug)]
    enum Effect {
        AskThree,
        AskBurst,
        BringForwardSoon(u32),
        CloseWithWorkKept,
    }

    impl Program for Forward {
        type Message = Message;
        type Effect = Effect;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
            match message {
                Message::AskThree => vec![Effect::AskThree],
                Message::AskBurst => vec![Effect::AskBurst],
                Message::BringForwardSoon => {
                    self.knocks += 1;
                    vec![Effect::BringForwardSoon(self.knocks)]
                }
                Message::CloseWithWorkKept => vec![Effect::CloseWithWorkKept],
            }
        }
    }

    struct Hand;

    fn ask(
        window: &WindowThread,
        what: &str,
        work: impl FnOnce(WindowHandle<'_>) + Send + 'static,
    ) {
        if let Err(unrun) = window.run(work) {
            note(&format!("{what} was refused: {unrun}"));
        }
    }

    impl Performer<Forward> for Hand {
        fn moment(&mut self) {}

        fn perform(&mut self, effect: Effect, _messages: &Sender<Message>) {
            let window = WindowThread::current();
            match effect {
                Effect::AskThree => {
                    thread::spawn(move || {
                        for piece in 1..=3 {
                            ask(&window, "(b) a piece", move |_| {
                                note(&format!(
                                    "(b) piece {piece} ran on the window's thread: {}",
                                    on_the_window_thread()
                                ));
                            });
                        }
                    });
                }
                Effect::AskBurst => {
                    thread::spawn(move || {
                        let tally = Arc::new(Tally {
                            ran: AtomicUsize::new(0),
                            dropped: AtomicUsize::new(0),
                            in_order: AtomicBool::new(true),
                        });
                        for piece in 0..BURST {
                            let counted = Counted {
                                tally: Arc::clone(&tally),
                                ran: false,
                            };
                            ask(&window, "(f) a piece of the burst", move |_| {
                                let mut counted = counted;
                                counted.ran = true;
                                // In the order asked, this piece is the next to run.
                                if counted.tally.ran.fetch_add(1, Ordering::SeqCst) != piece {
                                    counted.tally.in_order.store(false, Ordering::SeqCst);
                                }
                            });
                        }
                        let asked = Instant::now();
                        let settled = || {
                            tally.ran.load(Ordering::SeqCst) + tally.dropped.load(Ordering::SeqCst)
                        };
                        while settled() < BURST && asked.elapsed() < SETTLE {
                            thread::sleep(Duration::from_millis(20));
                        }
                        note(&format!(
                            "(f) of {BURST} pieces asked at once: {} ran, {} were dropped unrun, in the order asked: {}",
                            tally.ran.load(Ordering::SeqCst),
                            tally.dropped.load(Ordering::SeqCst),
                            tally.in_order.load(Ordering::SeqCst)
                        ));
                    });
                }
                Effect::BringForwardSoon(knock) => {
                    thread::spawn(move || {
                        thread::sleep(KNOCK);
                        ask(&window, "bringing forward", move |handle| {
                            let answer = match handle.bring_forward() {
                                Ok(()) => "Ok".to_owned(),
                                Err(behind) => format!("Behind ({behind})"),
                            };
                            note(&format!("bring forward {knock} answered {answer}"));
                        });
                    });
                }
                Effect::CloseWithWorkKept => {
                    let again = window.clone();
                    ask(&window, "(e) the closing piece", move |handle| {
                        let kept = Kept {
                            what: "the kept work",
                            ran: false,
                        };
                        ask(&again, "(e) the kept piece", move |_| {
                            let mut kept = kept;
                            kept.ran = true;
                        });
                        // The window is closed within this piece, so no turn hands over the piece
                        // just asked: a posted close leaves a turn between for that.
                        // SAFETY: `handle` is the window's, on its own thread, for the call now
                        // running; the call keeps nothing of the handle.
                        let _ = unsafe { SendMessageW(handle.as_raw(), CLOSE, 0, 0) };
                    });
                    // Asked straight behind the closing piece, so one hand-over holds the two.
                    let behind = Kept {
                        what: "the work behind the closing piece",
                        ran: false,
                    };
                    ask(&window, "(e) the piece behind", move |handle| {
                        let mut behind = behind;
                        behind.ran = true;
                        note(&format!(
                            "(e) the work behind the closing piece was handed a window of this process: {}",
                            of_this_process(handle)
                        ));
                    });
                }
            }
        }

        fn subscribe(&mut self, subscription: &Never, _messages: Sender<Message>) -> Running {
            match *subscription {}
        }
    }

    fn view(_forward: &Forward, messages: &Messages<Message>) -> Window {
        let press = |name: &'static str, message: Message| {
            Button::new()
                .automation_name(name)
                .on_click(messages.message(message))
                .content(name)
        };
        let content = StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(8.0)
            .margin(Thickness::uniform(16.0))
            .children((
                press("Ask three", Message::AskThree),
                press("Ask a burst", Message::AskBurst),
                press("Bring forward soon", Message::BringForwardSoon),
                press("Close with work kept", Message::CloseWithWorkKept),
            ));
        Window::new(content)
            .title("Forward")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 300,
                y: 200,
                width: 700,
                height: 300,
                maximized: false,
            }))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let log = std::env::args_os()
            .nth(1)
            .map_or_else(|| PathBuf::from("forward.log"), PathBuf::from);
        let _ = LOG.set(log);
        let _ = RUN_ON.set(thread::current().id());
        note("forward starts");
        let before = WindowThread::current();
        let from_before = before.clone();
        let asked = thread::spawn(move || {
            from_before.run(|handle| {
                note(&format!(
                    "(a) work asked before run ran on the thread run was called on: {}",
                    on_the_window_thread()
                ));
                note(&format!(
                    "(a) Windows knows its handle as a window of this process: {}",
                    of_this_process(handle)
                ));
            })
        })
        .join();
        note(&match asked {
            Ok(Ok(())) => "(a) asked from another thread before run: kept".to_owned(),
            Ok(Err(unrun)) => format!("(a) asked from another thread before run: {unrun}"),
            Err(_) => "(a) the asking thread panicked".to_owned(),
        });
        let ended = pamba_windows::run(Forward { knocks: 0 }, Vec::new(), Hand, view);
        note(&match &ended {
            Ok(()) => "the window has closed".to_owned(),
            Err(fault) => format!("the window ended: {fault}"),
        });
        note(&format!(
            "(e) an ask after the window closed: {:?}",
            before.run(|_| note("(e) work asked after the window closed ran"))
        ));
        ended
    }
}
