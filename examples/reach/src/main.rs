//! What a program on Pamba reaches of Windows, shown on a desktop.
//!
//! It draws a line that asks Windows' pointer for a line that is dragged, takes the keyboard and
//! follows the pointer and the arrow keys; a press that copies; a block that takes the pointer
//! inside a scroller; a window that opens at a place it names and says where it stands; and the
//! keyboard stood where the program asks: at the line when the program starts and when a press
//! asks it, and at a press that the drawing shows only once another has been pressed.
//! Every thing it observes is appended, a line each, to the file its first argument names.
//!
//! It is the proof a move of the reactor's commit owes. On a signed-in Windows desktop, start it
//! with the path of a log, then run `hand.ps1` with `steps.txt` (both beside this crate's
//! manifest; the steps name the screen places of a window at 300, 200 on a screen at 100 per
//! cent scale, and the process to bring to the front). The log is then `expected-log.txt` line
//! for line, and what the script read of the pointer, the clipboard and the keyboard's focus is
//! `expected-answers.txt`. Lines 19 to 22 of the log are a finger's: it drags the line, and
//! inside the scroller it is taken by the scroll before the block sees it move.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!("reach shows what is reached of Windows, and runs there alone");
}

#[cfg(windows)]
fn main() -> Result<(), pamba_windows::Fault> {
    shown::run()
}

#[cfg(windows)]
mod shown {
    use std::fs::OpenOptions;
    use std::io::Write as _;
    use std::path::PathBuf;

    use pamba::{Never, Performer, Program, Running, Sender};
    use pamba_windows::ui::{
        Border, Button, FocusEventInfo, HorizontalAlignment, InputModifiers,
        InputSystemCursorShape, KeyEventInfo, Orientation, PointerEventInfo, ScrollViewer,
        StackPanel, TextBlock, ThemeBrush, Thickness, View, VirtualKey, WindowPlacement,
        WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window};

    /// How far one press of an arrow key moves the line.
    const STEP: f64 = 10.0;

    struct Reach {
        /// How far the line stands from the window's left edge.
        line: f64,
        /// While the line is held: where it was taken, and where it then stood.
        taken: Option<(f64, f64)>,
        focused: bool,
        copies: u32,
        /// The moves seen since the pointer last went down in the scroller.
        moves: u32,
        /// Whether the press that opens has been pressed, so that what it opens is drawn.
        opened: bool,
    }

    /// The controls the program asks the keyboard to stand at.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Place {
        Line,
        Opened,
    }

    #[derive(Clone, Copy, Debug)]
    enum Key {
        Left,
        Right,
        Copy,
    }

    #[derive(Clone, Debug)]
    enum Message {
        Key(Key),
        Focus(bool),
        Taken(f64),
        At(f64),
        Let(&'static str),
        Placed(WindowPlacement),
        Pressed,
        Scroller(&'static str),
        ScrollerMoved,
        Stand,
        Open,
    }

    #[derive(Debug)]
    enum Effect {
        Note(String),
        Copy(String),
        Stand(Place),
    }

    fn note(said: &str) -> Vec<Effect> {
        vec![Effect::Note(said.to_owned())]
    }

    impl Reach {
        fn copy(&mut self, by: &str) -> Vec<Effect> {
            self.copies += 1;
            vec![
                Effect::Note(format!("asked to copy by {by}")),
                Effect::Copy(format!("pamba reach {}", self.copies)),
            ]
        }
    }

    impl Program for Reach {
        type Message = Message;
        type Effect = Effect;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
            match message {
                Message::Key(Key::Left) => {
                    self.line -= STEP;
                    note("the left arrow key")
                }
                Message::Key(Key::Right) => {
                    self.line += STEP;
                    note("the right arrow key")
                }
                Message::Key(Key::Copy) => self.copy("Ctrl+C on the line"),
                Message::Pressed => self.copy("the press"),
                Message::Focus(focused) => {
                    self.focused = focused;
                    note(if focused {
                        "the line holds the keyboard"
                    } else {
                        "the line let the keyboard go"
                    })
                }
                Message::Taken(at) => {
                    self.taken = Some((at, self.line));
                    note("the line is taken")
                }
                Message::At(at) => {
                    if let Some((from, stood)) = self.taken {
                        self.line = stood + (at - from);
                    }
                    Vec::new()
                }
                Message::Let(how) => match self.taken.take() {
                    Some(_) => vec![Effect::Note(format!(
                        "the line is let go ({how}) and stands at {:.0}",
                        self.line
                    ))],
                    None => Vec::new(),
                },
                Message::Placed(placed) => vec![Effect::Note(format!(
                    "the window stands at {} {}, {} by {}, maximized {}",
                    placed.x, placed.y, placed.width, placed.height, placed.maximized
                ))],
                Message::Scroller("pressed") => {
                    self.moves = 0;
                    note("the block in the scroller is pressed")
                }
                Message::Scroller(what) => {
                    let moved = if self.moves > 0 {
                        "having followed the pointer"
                    } else {
                        "having seen no move"
                    };
                    vec![Effect::Note(format!(
                        "the block in the scroller: {what}, {moved}"
                    ))]
                }
                Message::ScrollerMoved => {
                    self.moves += 1;
                    Vec::new()
                }
                Message::Stand => vec![
                    Effect::Note("asked to stand the keyboard at the line".to_owned()),
                    Effect::Stand(Place::Line),
                ],
                Message::Open => {
                    self.opened = true;
                    vec![
                        Effect::Note(
                            "asked to open, and to stand the keyboard at what opens".to_owned(),
                        ),
                        Effect::Stand(Place::Opened),
                    ]
                }
            }
        }
    }

    struct Hand {
        log: PathBuf,
    }

    impl Hand {
        fn note(&self, line: &str) {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.log) {
                let _ = writeln!(file, "{line}");
            }
        }
    }

    impl Performer<Reach> for Hand {
        fn moment(&mut self) {}

        fn perform(&mut self, effect: Effect, _messages: &Sender<Message>) {
            match effect {
                Effect::Note(line) => self.note(&line),
                Effect::Copy(text) => match pamba_windows::copy(&text) {
                    Ok(()) => self.note(&format!("copied \"{text}\"")),
                    Err(why) => self.note(&format!("not copied: {why}")),
                },
                Effect::Stand(place) => pamba_windows::focus(place),
            }
        }

        fn subscribe(&mut self, subscription: &Never, _messages: Sender<Message>) -> Running {
            match *subscription {}
        }
    }

    /// The line: dragged by the pointer, moved by the arrow keys, and a place the keyboard is
    /// stood at.
    fn line(reach: &Reach, messages: &Messages<Message>) -> Border {
        Border::new()
            .width(12.0)
            .height(120.0)
            .horizontal_alignment(HorizontalAlignment::Left)
            .margin(Thickness::new(reach.line, 0.0, 0.0, 0.0))
            .background(if reach.focused || reach.taken.is_some() {
                ThemeBrush::Accent
            } else {
                ThemeBrush::CardStroke
            })
            .pointer_cursor(InputSystemCursorShape::SizeWestEast)
            .is_tab_stop(true)
            .automation_name("The line")
            .element_ref(&pamba_windows::named(Place::Line))
            .capture_pointer_on_press(true)
            .on_got_focus(messages.callback(|_: FocusEventInfo| Message::Focus(true)))
            .on_lost_focus(messages.callback(|_: FocusEventInfo| Message::Focus(false)))
            .on_preview_key_down(messages.routed(|key: KeyEventInfo| {
                if key.key == VirtualKey::LEFT {
                    Some(Message::Key(Key::Left))
                } else if key.key == VirtualKey::RIGHT {
                    Some(Message::Key(Key::Right))
                } else if key.key == VirtualKey::C
                    && key.modifiers.contains(InputModifiers::CONTROL)
                {
                    Some(Message::Key(Key::Copy))
                } else {
                    None
                }
            }))
            .on_pointer_pressed(
                messages.callback(|at: PointerEventInfo| Message::Taken(at.window_x)),
            )
            .on_pointer_moved(messages.callback(|at: PointerEventInfo| Message::At(at.window_x)))
            .on_pointer_released(messages.callback(|_: PointerEventInfo| Message::Let("released")))
            .on_pointer_capture_lost(messages.message(Message::Let("capture lost")))
            .on_pointer_canceled(messages.message(Message::Let("canceled")))
    }

    /// A block that takes the pointer, inside a scroller that scrolls.
    fn scroller(messages: &Messages<Message>) -> ScrollViewer {
        let held = Border::new()
            .height(120.0)
            .background(ThemeBrush::CardBackground)
            .automation_name("Held in the scroller")
            .capture_pointer_on_press(true)
            .on_pointer_pressed(
                messages.callback(|_: PointerEventInfo| Message::Scroller("pressed")),
            )
            .on_pointer_moved(messages.callback(|_: PointerEventInfo| Message::ScrollerMoved))
            .on_pointer_released(
                messages.callback(|_: PointerEventInfo| Message::Scroller("released")),
            )
            .on_pointer_capture_lost(messages.message(Message::Scroller("capture lost")))
            .on_pointer_canceled(messages.message(Message::Scroller("canceled")))
            .content(TextBlock::new().text("Held in the scroller"));
        let tall = StackPanel::new().children((
            held,
            Border::new()
                .height(1200.0)
                .content(TextBlock::new().text("What the scroller scrolls")),
        ));
        ScrollViewer::new().height(200.0).content(tall)
    }

    /// The presses that ask where the keyboard stands, and what one of them opens.
    fn asks(reach: &Reach, messages: &Messages<Message>) -> StackPanel {
        let ask = |name: &'static str, message: Message| {
            Button::new()
                .width(160.0)
                .height(32.0)
                .automation_name(name)
                .on_click(messages.message(message))
                .content(name)
        };
        // What opens is drawn only once it has been asked for: the drawing that first shows it
        // is the one the keyboard is stood in. The reactor tells of the keyboard's coming for a
        // `Border` alone, so where it stands is read from Windows by the script.
        let mut pressed: Vec<View> = vec![
            ask("Stand at the line", Message::Stand).into(),
            ask("Open", Message::Open).into(),
        ];
        if reach.opened {
            pressed.push(
                Button::new()
                    .width(160.0)
                    .height(32.0)
                    .automation_name("Opened")
                    .element_ref(&pamba_windows::named(Place::Opened))
                    .content("Opened")
                    .into(),
            );
        }
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(12.0)
            .children(pressed)
    }

    fn view(reach: &Reach, messages: &Messages<Message>) -> Window {
        let content = StackPanel::new()
            .spacing(12.0)
            .margin(Thickness::uniform(16.0))
            .children((
                TextBlock::new().text(format!(
                    "The line stands at {:.0}; asked to copy {} times",
                    reach.line, reach.copies
                )),
                Button::new()
                    .automation_name("Copy")
                    .on_click(messages.message(Message::Pressed))
                    .content("Copy"),
                line(reach, messages),
                scroller(messages),
                asks(reach, messages),
            ));
        Window::new(content)
            .title("Reach")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 300,
                y: 200,
                width: 900,
                height: 640,
                maximized: false,
            }))
            .on_placement(messages.callback(Message::Placed))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let log = std::env::args_os()
            .nth(1)
            .map_or_else(|| PathBuf::from("reach.log"), PathBuf::from);
        let hand = Hand { log };
        hand.note("reach starts");
        let reach = Reach {
            line: 100.0,
            taken: None,
            focused: false,
            copies: 0,
            moves: 0,
            opened: false,
        };
        // The program starts with the keyboard at the line, which is not the first control that
        // can take it.
        pamba_windows::run(reach, vec![Effect::Stand(Place::Line)], hand, view)
    }
}
