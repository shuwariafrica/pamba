//! What Windows gives a press's handler, taken on a desktop.
//!
//! It draws a press and counts its drawings: the press's callbacks carry the number of the drawing
//! they were made in. Each thing the program hears is a line in the file its first argument names,
//! with three clocks beside it: microseconds since the program started, Windows' tick count in
//! milliseconds since the computer started (`tick`), and the time Windows gives the message being
//! handled, on the same clock (`message`). The press is heard four ways: its click, a pointer going
//! down on it and a key going down or up on it while Windows routes them, and a double tap.
//!
//! Beside it: presses that arm the next press of the press to stop the window's thread for two
//! seconds, alone, or with the press removed or disabled by the drawing after; one that brings the
//! press back; and one that makes twenty drawings one after another, each written as its step ends,
//! as the reactor reports an ask made at that moment done, and at the next frame XAML renders.
//!
//! On a signed-in Windows desktop, start it with the path of a log, then run `facts.ps1` beside
//! this crate's manifest with a steps file, which writes its own acts into the same log.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!("presses takes facts of Windows' input, and runs on Windows alone");
}

#[cfg(windows)]
fn main() -> Result<(), pamba_windows::Fault> {
    shown::run()
}

#[cfg(windows)]
mod shown {
    use std::cell::{Cell, RefCell};
    use std::fs::OpenOptions;
    use std::io::Write as _;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{Duration, Instant};

    use pamba::{Never, Performer, Program, Running, Sender};
    use pamba_windows::ui::{
        Border, Button, ElementObservation, ElementRef, KeyEventInfo, PointerEventInfo, StackPanel,
        SwapChainPanel, SwapChainPanelEvent, TextBlock, Thickness, View, WindowPlacement,
        WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window};

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetMessageTime() -> i32;
        fn GetDoubleClickTime() -> u32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetTickCount64() -> u64;
    }

    /// How many drawings make the chain of B3.
    const CHAIN: u32 = 20;
    /// How long an armed press stops the window's thread.
    const FREEZE: Duration = Duration::from_secs(2);

    thread_local! {
        static LOG: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
        static STARTED: Instant = Instant::now();
        /// The number of the drawing being made, counted as the view is asked for one.
        static DRAWING: Cell<u64> = const { Cell::new(0) };
        static PANEL: ElementRef<SwapChainPanel> = ElementRef::new();
        static RENDERING: RefCell<Option<ElementObservation>> = const { RefCell::new(None) };
        /// The step of the chain whose first rendered frame is awaited, and the sender to ask the
        /// next one with.
        static AWAITED: RefCell<Option<(u32, Sender<Message>)>> = const { RefCell::new(None) };
    }

    /// Appends one line to the log, with the three clocks read now.
    fn write(what: &str) {
        let micros = STARTED.with(Instant::elapsed).as_micros();
        // SAFETY: GetTickCount64 takes nothing and reads a counter.
        let tick = unsafe { GetTickCount64() };
        // SAFETY: GetMessageTime takes nothing and reads the thread's last message's time.
        let message = unsafe { GetMessageTime() };
        // One write for the whole line: the hand appends to the same file, and a line written in
        // pieces can be torn by its line.
        let line = format!("t={micros} tick={tick} message={message} {what}\n");
        LOG.with_borrow(|log| {
            if let Some(log) = log
                && let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log)
            {
                let _ = file.write_all(line.as_bytes());
            }
        });
    }

    fn double_click_time() -> u32 {
        // SAFETY: GetDoubleClickTime takes nothing and reads a setting.
        unsafe { GetDoubleClickTime() }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Arm {
        Nothing,
        Freeze,
        Remove,
        Disable,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Shown {
        Enabled,
        Disabled,
        Removed,
    }

    struct Presses {
        armed: Arm,
        shown: Shown,
    }

    #[derive(Clone, Copy, Debug)]
    enum Message {
        Pressed { drawing: u64 },
        Doubled { drawing: u64 },
        Armed(Arm),
        Restored,
        Chained(u32),
    }

    #[derive(Debug)]
    enum Effect {
        Freeze,
        Chain(u32),
    }

    impl Program for Presses {
        type Message = Message;
        type Effect = Effect;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
            let now = DRAWING.get();
            match message {
                Message::Pressed { drawing } => {
                    write(&format!(
                        "step: pressed, carried drawing {drawing}, now drawing {now}, armed {:?}",
                        self.armed
                    ));
                    let armed = std::mem::replace(&mut self.armed, Arm::Nothing);
                    match armed {
                        Arm::Nothing => Vec::new(),
                        Arm::Freeze => vec![Effect::Freeze],
                        Arm::Remove => {
                            self.shown = Shown::Removed;
                            vec![Effect::Freeze]
                        }
                        Arm::Disable => {
                            self.shown = Shown::Disabled;
                            vec![Effect::Freeze]
                        }
                    }
                }
                Message::Doubled { drawing } => {
                    write(&format!(
                        "step: double tapped, carried drawing {drawing}, now drawing {now}"
                    ));
                    Vec::new()
                }
                Message::Armed(arm) => {
                    write(&format!("step: armed {arm:?}"));
                    self.armed = arm;
                    Vec::new()
                }
                Message::Restored => {
                    write("step: restored");
                    self.shown = Shown::Enabled;
                    Vec::new()
                }
                Message::Chained(step) => vec![Effect::Chain(step)],
            }
        }
    }

    struct Hand;

    impl Performer<Presses> for Hand {
        fn moment(&mut self) {}

        fn perform(&mut self, effect: Effect, messages: &Sender<Message>) {
            match effect {
                Effect::Freeze => {
                    write("freeze: begins");
                    thread::sleep(FREEZE);
                    write("freeze: ends");
                }
                Effect::Chain(step) => {
                    write(&format!("chain {step}: step ended"));
                    AWAITED.with_borrow_mut(|awaited| *awaited = Some((step, messages.clone())));
                    let asked = PANEL.with(|panel| {
                        panel.request_surface_frame(move |done| {
                            write(&format!("chain {step}: ask done {done:?}"));
                        })
                    });
                    if !asked {
                        write(&format!("chain {step}: ask refused"));
                    }
                }
            }
        }

        fn subscribe(&mut self, subscription: &Never, _messages: Sender<Message>) -> Running {
            match *subscription {}
        }
    }

    /// The first frame XAML renders after a step of the chain: written, and the next step asked.
    fn rendered(event: SwapChainPanelEvent) {
        if event != SwapChainPanelEvent::Rendering {
            return;
        }
        let Some((step, messages)) = AWAITED.with_borrow_mut(Option::take) else {
            return;
        };
        write(&format!("chain {step}: frame rendered"));
        if step < CHAIN {
            let _ = messages.send(Message::Chained(step + 1));
        }
    }

    fn view(presses: &Presses, messages: &Messages<Message>) -> Window {
        let drawing = DRAWING.get() + 1;
        DRAWING.set(drawing);
        RENDERING.with_borrow_mut(|rendering| {
            if rendering.is_none() {
                *rendering = Some(PANEL.with(|panel| panel.observe_surface(rendered)));
            }
        });
        let press = Button::new()
            .width(200.0)
            .automation_name("Press")
            .is_enabled(presses.shown == Shown::Enabled)
            .on_click(messages.callback(move |()| {
                write(&format!(
                    "click: drawing {drawing}, double-click time {}",
                    double_click_time()
                ));
                Message::Pressed { drawing }
            }))
            .content("Press");
        let arm = |name: &'static str, arm: Arm| {
            Button::new()
                .automation_name(name)
                .on_click(messages.message(Message::Armed(arm)))
                .content(name)
        };
        // The reactor gives a button no handler for a pointer going down, a key or a double tap,
        // so the border around the press hears what rises to it from the press, and a key going
        // down before the press does.
        let press = Border::new()
            .on_double_tapped(messages.callback(move |()| {
                write(&format!("double tap: drawing {drawing}"));
                Message::Doubled { drawing }
            }))
            .on_pointer_pressed(messages.routed(move |pointer: PointerEventInfo| {
                write(&format!(
                    "pointer down: drawing {drawing}, pointer {}, left {}",
                    pointer.pointer_id, pointer.is_left_button_pressed
                ));
                None
            }))
            .on_preview_key_down(messages.routed(move |key: KeyEventInfo| {
                write(&format!(
                    "key down: drawing {drawing}, {:?}, {:?}",
                    key.key, key.status
                ));
                None
            }))
            .on_key_up(messages.routed(move |key: KeyEventInfo| {
                write(&format!(
                    "key up: drawing {drawing}, {:?}, {:?}",
                    key.key, key.status
                ));
                None
            }))
            .content(press);
        let mut children: Vec<View> = vec![
            TextBlock::new().text(format!("Drawing {drawing}")).into(),
            arm("Arm a freeze", Arm::Freeze).into(),
            arm("Arm a removal", Arm::Remove).into(),
            arm("Arm a disabling", Arm::Disable).into(),
            Button::new()
                .automation_name("Bring the press back")
                .on_click(messages.message(Message::Restored))
                .content("Bring the press back")
                .into(),
            Button::new()
                .automation_name("Twenty drawings")
                .on_click(messages.message(Message::Chained(1)))
                .content("Twenty drawings")
                .into(),
            SwapChainPanel::new()
                .width(8.0)
                .height(8.0)
                .element_ref(&PANEL.with(Clone::clone))
                .into(),
        ];
        // The press stands last, so that removing it moves nothing else: a press waiting in the
        // queue then lands where the press stood, not on a control that moved into its place.
        if presses.shown != Shown::Removed {
            children.push(press.into());
        }
        let content = StackPanel::new()
            .spacing(12.0)
            .margin(Thickness::uniform(16.0))
            .children(children);
        Window::new(content)
            .title("Presses")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 300,
                y: 120,
                width: 700,
                height: 720,
                maximized: false,
            }))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let log = std::env::args_os()
            .nth(1)
            .map_or_else(|| PathBuf::from("presses.log"), PathBuf::from);
        LOG.with_borrow_mut(|held| *held = Some(log));
        STARTED.with(|_| ());
        write("started");
        pamba_windows::run(
            Presses {
                armed: Arm::Nothing,
                shown: Shown::Enabled,
            },
            Vec::new(),
            Hand,
            view,
        )
    }
}
