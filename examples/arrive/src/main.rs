//! What arrives in answer, brought into view and said, shown on a desktop.
//!
//! A window 480 rows high holds a row of presses and a field above a column that scrolls. The
//! column holds a tall place at its head, then enough lines to push the answer's place below the
//! window's foot, and a second scrolling area with a place of its own. Each press answers as a
//! program's logic would, with an effect that names a place and the words to say:
//!
//! - "Answer" draws the answer in a place below the foot, for the first time, and asks for it to
//!   be brought into view and said, with the keyboard left on the press;
//! - "Answer in the field" asks the same and stands the keyboard in the field;
//! - "Inner" asks for the place inside the second scrolling area;
//! - "Tall" asks for the tall place, which is taller than the column, with no words;
//! - "Missing" asks for a place no drawing gives.
//!
//! Every thing it observes is appended, a line each, to the file its first argument names, and
//! the last line says how the window ended. A second argument changes what it is:
//! `--without-calls` performs the effect without the two calls, for the case that shows what the
//! calls do; `--measure` draws a column of two thousand lines with the answer's place last.
//!
//! On a signed-in Windows desktop at 100 per cent scale, start it with the path of a log, then
//! `listen.ps1` with its process's number, then `hand.ps1` with `steps.txt` (all beside this
//! crate's manifest). The log is then `expected-log.txt` line for line, what the hand read is
//! `expected-answers.txt`, and what the listener heard is `expected-heard.txt`. Under `--measure`
//! the hand works `measure-steps.txt` instead: twenty presses of "Answer", the wheel turned back
//! between them so that each press moves the column.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!(
        "arrive shows what arrives in answer brought into view and said, and runs on Windows alone"
    );
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
        Border, Button, Orientation, ScrollViewer, StackPanel, TextBlock, TextBox, Thickness, View,
        WindowPlacement, WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window};

    /// What the logic says when an answer arrives: the same words each time.
    const ANSWERED: &str = "The answer has arrived";

    /// The places the logic names.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Place {
        /// Where the answer is drawn, below the window's foot.
        Answer,
        /// The place inside the second scrolling area.
        Inner,
        /// A place taller than the column.
        Tall,
        /// A place no drawing gives.
        Missing,
        /// The field, where the keyboard is asked to stand.
        Field,
    }

    /// How the example was started.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Run {
        /// As a program on Pamba would.
        Calls,
        /// With the effect performed but neither call made.
        WithoutCalls,
        /// With two thousand lines before the answer's place.
        Measure,
    }

    struct Arrive {
        answers: u32,
        run: Run,
    }

    #[derive(Clone, Copy, Debug)]
    enum Message {
        Answer,
        AnswerInField,
        Inner,
        Tall,
        Missing,
    }

    #[derive(Debug)]
    enum Effect {
        Note(String),
        Arrived { place: Place, said: String },
        Focus(Place),
    }

    fn arrived(place: Place, said: &str) -> Effect {
        Effect::Arrived {
            place,
            said: said.to_owned(),
        }
    }

    impl Program for Arrive {
        type Message = Message;
        type Effect = Effect;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
            match message {
                Message::Answer => {
                    self.answers += 1;
                    vec![
                        Effect::Note(format!("answer {} arrives", self.answers)),
                        arrived(Place::Answer, ANSWERED),
                    ]
                }
                Message::AnswerInField => {
                    self.answers += 1;
                    vec![
                        Effect::Note(format!(
                            "answer {} arrives, and the keyboard is asked to the field",
                            self.answers
                        )),
                        Effect::Focus(Place::Field),
                        arrived(Place::Answer, ANSWERED),
                    ]
                }
                Message::Inner => vec![
                    Effect::Note("the inner answer arrives".to_owned()),
                    arrived(Place::Inner, "The inner answer has arrived"),
                ],
                Message::Tall => vec![
                    Effect::Note("the tall place is asked for, with no words".to_owned()),
                    arrived(Place::Tall, ""),
                ],
                Message::Missing => vec![
                    Effect::Note("a place no drawing gives is asked for".to_owned()),
                    arrived(Place::Missing, ""),
                ],
            }
        }
    }

    struct Hand {
        log: PathBuf,
        run: Run,
    }

    impl Hand {
        fn note(&self, line: &str) {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.log) {
                let _ = writeln!(file, "{line}");
            }
        }
    }

    impl Performer<Arrive> for Hand {
        fn moment(&mut self) {}

        fn perform(&mut self, effect: Effect, _messages: &Sender<Message>) {
            match effect {
                Effect::Note(line) => self.note(&line),
                Effect::Arrived { place, said } => {
                    if self.run != Run::WithoutCalls {
                        pamba_windows::bring_into_view(place);
                        pamba_windows::announce(&said);
                    }
                }
                Effect::Focus(place) => pamba_windows::focus(place),
            }
        }

        fn subscribe(&mut self, subscription: &Never, _messages: Sender<Message>) -> Running {
            match *subscription {}
        }
    }

    fn lines(prefix: &str, count: u32) -> impl Iterator<Item = View> + '_ {
        (1..=count).map(move |n| TextBlock::new().text(format!("{prefix} {n}")).into())
    }

    fn presses(messages: &Messages<Message>) -> StackPanel {
        let press = |name: &'static str, message: Message| -> View {
            Button::new()
                .automation_name(name)
                .on_click(messages.message(message))
                .content(name)
                .into()
        };
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(8.0)
            .children(vec![
                press("Answer", Message::Answer),
                press("Answer in the field", Message::AnswerInField),
                press("Inner", Message::Inner),
                press("Tall", Message::Tall),
                press("Missing", Message::Missing),
                TextBox::new()
                    .width(160.0)
                    .automation_name("The field")
                    .element_ref(&pamba_windows::named(Place::Field))
                    .into(),
            ])
    }

    /// The column: the tall place, lines, the second scrolling area, lines, and the answer once
    /// one has arrived.
    fn column(arrive: &Arrive) -> ScrollViewer {
        let tall = pamba_windows::place(
            Place::Tall,
            StackPanel::new().children((
                TextBlock::new().text("The tall place's head"),
                Border::new().height(600.0),
                TextBlock::new().text("The tall place's foot"),
            )),
        );
        let inner: Vec<View> = lines("Inner line", 20)
            .chain([
                pamba_windows::place(Place::Inner, TextBlock::new().text("The inner place")).into(),
            ])
            .collect();
        let mut held: Vec<View> = vec![tall.into()];
        held.extend(lines("Line", 40));
        held.push(
            ScrollViewer::new()
                .height(120.0)
                .content(StackPanel::new().children(inner))
                .into(),
        );
        held.extend(lines("After", 40));
        if arrive.run == Run::Measure {
            held.extend(lines("Measured", 2000));
        }
        if arrive.answers > 0 {
            held.push(pamba_windows::place(Place::Answer, TextBlock::new().text(ANSWERED)).into());
        }
        ScrollViewer::new()
            .height(330.0)
            .content(StackPanel::new().children(held))
    }

    fn view(arrive: &Arrive, messages: &Messages<Message>) -> Window {
        let content = StackPanel::new()
            .spacing(12.0)
            .margin(Thickness::uniform(16.0))
            .children((presses(messages), column(arrive)));
        Window::new(content)
            .title("Arrive")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 300,
                y: 200,
                width: 900,
                height: 480,
                maximized: false,
            }))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let mut arguments = std::env::args_os().skip(1);
        let log = arguments
            .next()
            .map_or_else(|| PathBuf::from("arrive.log"), PathBuf::from);
        let run = match arguments.next() {
            Some(switch) if switch == "--without-calls" => Run::WithoutCalls,
            Some(switch) if switch == "--measure" => Run::Measure,
            _ => Run::Calls,
        };
        let hand = Hand {
            log: log.clone(),
            run,
        };
        hand.note(&format!("arrive starts: {run:?}"));
        let ended = pamba_windows::run(Arrive { answers: 0, run }, Vec::new(), hand, view);
        let closing = Hand { log, run };
        closing.note(&match &ended {
            Ok(()) => "the window has closed".to_owned(),
            Err(fault) => format!("the window ended: {fault}"),
        });
        ended
    }
}
