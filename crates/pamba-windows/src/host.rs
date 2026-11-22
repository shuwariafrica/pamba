use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;
use std::rc::Rc;
use std::sync::Arc;

use pamba::{Dispatcher, Loop, Performer, Program};
use windows_reactor::{
    App, Border, Callback, Component, ComponentContext, ComponentTimer, FocusEventInfo,
    LocalSender, RoutedCallback, View, ViewContext,
};

use crate::arrive;
use crate::beside::{self, WindowHandle};
use crate::fill::{self, Checked};
use crate::named;
use crate::threads::Threads;
use crate::watch;
use crate::window::Window;
use crate::window_thread;

/// How many queued messages one turn takes, and how many pieces of work asked of the window's
/// thread one hand-over runs, before the window is drawn and its own events are seen to: a burst
/// beyond it is taken over several.
const TURN: NonZeroUsize = match NonZeroUsize::new(64) {
    Some(turn) => turn,
    None => unreachable!(),
};

/// The window could not be opened, or WinUI ended it: Windows' own code and words for why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// The `HRESULT` Windows gave.
    pub code: i32,
    /// Windows' words for it, where it has any.
    pub message: String,
}

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} (0x{:08X})", self.message, self.code)
    }
}

impl Error for Fault {}

/// Turns what a control reports into a message of the program's.
pub struct Messages<M> {
    sender: LocalSender<Event<M>>,
}

impl<M: 'static> Messages<M> {
    /// A callback that sends what `map` makes of the value a control reports.
    ///
    /// Give `map` no captures where the value alone says what happened: such a callback is the
    /// same one at every drawing, and its control is left as it is.
    pub fn callback<T, F>(&self, map: F) -> Callback<T>
    where
        F: Fn(T) -> M + 'static,
    {
        self.sender.callback(move |value| Event::Did(map(value)))
    }

    /// A callback Windows asks while it still routes the event, for a key or a pointer the
    /// program may take for itself: where `map` gives a message it is sent and the event goes no
    /// further, and where it gives none the event goes on to whatever else would take it.
    pub fn routed<T, F>(&self, map: F) -> RoutedCallback<T>
    where
        F: Fn(T) -> Option<M> + 'static,
    {
        let sender = self.sender.clone();
        RoutedCallback::new(move |value| {
            map(value).is_some_and(|message| sender.send(Event::Did(message)))
        })
    }

    /// A callback that sends `message` each time it is called.
    pub fn message(&self, message: M) -> Callback<()>
    where
        M: Clone,
    {
        self.sender.callback(move |()| Event::Did(message.clone()))
    }
}

impl<M> fmt::Debug for Messages<M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Messages").finish_non_exhaustive()
    }
}

pub(crate) enum Event<M> {
    /// What a person did in the window.
    Did(M),
    /// The loop's mailbox holds messages.
    Woken,
    /// The window stands.
    Opened,
    /// Windows refused the keyboard to a named control it does not show yet: the ask is made
    /// again after a pause.
    Pause,
    /// That pause is over.
    Paused,
    /// The window's content is to be checked against the window.
    Check,
    /// A hand-over of work asked of the window's thread has run.
    Ran,
}

struct Parts<P: Program, W, V> {
    program: P,
    effects: Vec<P::Effect>,
    performer: W,
    view: V,
}

/// What the window's one component is made from, handed over once: `windows-reactor` asks an
/// input to be cloned and compared, and this one is the same input wherever it is the same cell.
struct Seed<P: Program, W, V>(Rc<RefCell<Option<Parts<P, W, V>>>>);

impl<P: Program, W, V> Clone for Seed<P, W, V> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<P: Program, W, V> PartialEq for Seed<P, W, V> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

struct Host<P: Program, W: Performer<P>, V> {
    running: Loop<P, W>,
    view: V,
    threads: Arc<Threads>,
    /// Told whenever a control of the window takes the keyboard; the same callback at every
    /// drawing.
    took: Callback<FocusEventInfo>,
    /// The pause before a refused ask of a named control is made again, while it runs: dropped,
    /// it is cancelled.
    pause: Option<ComponentTimer>,
    /// The pause before the window's content is next checked, while it runs.
    fill: Option<ComponentTimer>,
    /// How many times the window was made to take its size again.
    mends: u32,
}

impl<P, W, V> Host<P, W, V>
where
    P: Program + 'static,
    P::Message: Send + 'static,
    W: Performer<P> + 'static,
    V: Fn(&P, &Messages<P::Message>) -> Window + 'static,
{
    /// Starts the one task that waits for the loop's next wake.
    fn wait(threads: &Arc<Threads>, context: &ComponentContext<'_, Self>) {
        let threads = Arc::clone(threads);
        let _ = context.spawn_background(move |_| {
            threads.woken();
            Event::Woken
        });
    }

    /// Where work is asked of the window's thread and no hand-over waits already, queues one with
    /// the reactor, which runs a turn's worth of that work with the window's handle, in the order
    /// asked.
    fn hand(context: &ComponentContext<'_, Self>) {
        let Some(owed) = window_thread::owed() else {
            return;
        };
        // The reactor drops what it refuses, and what waits when the window goes. Either way the
        // work stays kept: the next wake offers it again, and the window's going drops it unrun.
        let _ = context.run_window(move |window| {
            let handle = WindowHandle::new(window.as_raw());
            owed.hand(TURN, |work| {
                // Work run before this in the hand-over may have closed the window. Work is
                // handed no handle Windows no longer knows as a window's: it is dropped unrun.
                if handle.is_window() {
                    work(handle);
                }
            });
            Event::Ran
        });
    }
}

impl<P, W, V> Component for Host<P, W, V>
where
    P: Program + 'static,
    P::Message: Send + 'static,
    W: Performer<P> + 'static,
    V: Fn(&P, &Messages<P::Message>) -> Window + 'static,
{
    type Input = Seed<P, W, V>;
    type Message = Event<P::Message>;

    fn create(input: &Self::Input, context: &ComponentContext<'_, Self>) -> Self {
        let Some(parts) = input.0.borrow_mut().take() else {
            unreachable!("a window's root is created once")
        };
        let threads = Arc::new(Threads::default());
        let dispatcher: Arc<dyn pamba::Dispatcher> = threads.clone();
        // Before the loop starts: an effect of the program's construction may ask where the
        // keyboard stands at the start.
        let sender = context.sender();
        named::stands(move || {
            // Refused only once the window has gone, and its names with it.
            let _ = sender.send(Event::Pause);
        });
        arrive::stands();
        let running = Loop::start(parts.program, parts.effects, parts.performer, dispatcher);
        Self::wait(&threads, context);
        // What Pamba reaches through Windows beside the reactor is reached for this window.
        let _ = context.run_window(|window| {
            beside::stands(&window);
            Event::Opened
        });
        // After the handle is kept: work asked before the window stood runs after it, in order.
        let waking = Arc::clone(&threads);
        window_thread::stands(move || waking.wake());
        Self::hand(context);
        watch::stands();
        Self {
            running,
            view: parts.view,
            threads,
            took: Callback::new(|_: FocusEventInfo| named::took()),
            pause: None,
            fill: None,
            mends: 0,
        }
    }

    fn update(&mut self, message: Self::Message, context: &ComponentContext<'_, Self>) {
        match message {
            Event::Did(message) => self.running.step(message),
            Event::Woken => {
                Self::hand(context);
                self.running.turn(TURN);
                Self::wait(&self.threads, context);
            }
            Event::Opened => {
                self.fill = Some(context.set_timeout(fill::FIRST, Event::Check));
            }
            Event::Check => {
                self.fill = match fill::check(self.mends) {
                    Checked::Mended => {
                        self.mends += 1;
                        Some(context.set_timeout(fill::BETWEEN, Event::Check))
                    }
                    Checked::Stands | Checked::Left => None,
                };
            }
            // More may wait: a burst beyond one hand-over, or work asked from inside work.
            Event::Ran => Self::hand(context),
            // One ask is retried at a time: a pause still running is for an ask since replaced.
            Event::Pause => {
                self.pause = Some(context.set_timeout(named::PAUSE, Event::Paused));
            }
            Event::Paused => {
                self.pause = None;
                named::retry();
            }
        }
    }

    fn view(&self, _input: &Self::Input, context: &mut ViewContext<'_, Self>) -> View {
        let messages = Messages {
            sender: context.sender(),
        };
        let ((window, asks), arrivals) =
            arrive::drawing(|| named::drawing(|| (self.view)(self.running.program(), &messages)));
        // The reactor runs this once this drawing stands, with every control and place it names
        // in place, and only where an ask of either was made since the drawing before.
        context.use_effect(
            "pamba-windows: what was asked of a named control",
            (asks, arrivals),
            || {
                arrive::carry_out();
                None
            },
        );
        if let Some(title) = window.title {
            context.window_title(title);
        }
        if let Some(visuals) = window.visuals {
            context.window_visuals(visuals);
        }
        if let Some(sized) = window.sized {
            context.on_window_size(sized);
        }
        // The reactor reports the colour mode alone, so the mode goes to the watch, which reads
        // what else decides how Windows draws and tells the program of the two as one fact.
        let asks = window.appeared.is_some();
        watch::asks(window.appeared);
        if asks {
            context.on_color_scheme(Callback::new(watch::mode));
        }
        if let Some(placed) = window.placed {
            context.on_window_placement(placed);
        }
        // The reactor tells of the keyboard's coming through a `Border` alone, and a control's
        // taking it is told to every element it stands in. So the content stands in one that
        // draws nothing, and the first control to take the keyboard is how the window's content
        // is known to be on the screen: an ask made before then is carried out at that.
        Border::new()
            .on_got_focus(self.took.clone())
            .content(window.content)
            .into()
    }
}

impl<P: Program, W: Performer<P>, V> Drop for Host<P, W, V> {
    fn drop(&mut self) {
        beside::gone();
        watch::gone();
        named::gone();
        arrive::gone();
        window_thread::gone();
        self.threads.end();
    }
}

/// Opens `program`'s window and runs it until the window is closed.
///
/// `effects` are the ones the program's construction asked for. `view` is called after every
/// step, and what it returns is brought onto the screen by changing only what differs.
///
/// # Errors
///
/// [`Fault`] where WinUI could not start - the Windows App Runtime is neither beside the
/// executable nor installed - or ended the window itself.
pub fn run<P, W, V>(program: P, effects: Vec<P::Effect>, performer: W, view: V) -> Result<(), Fault>
where
    P: Program + 'static,
    P::Message: Send + 'static,
    W: Performer<P> + 'static,
    V: Fn(&P, &Messages<P::Message>) -> Window + 'static,
{
    let seed = Seed(Rc::new(RefCell::new(Some(Parts {
        program,
        effects,
        performer,
        view,
    }))));
    App::run_component::<Host<P, W, V>>(seed).map_err(|error| {
        // Where WinUI could not start, no window stood to end what was asked of it.
        window_thread::gone();
        Fault {
            code: error.code().0,
            message: error.message(),
        }
    })
}
