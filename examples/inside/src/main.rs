//! Where the keyboard goes when a window of Pamba's asks it of a place.
//!
//! Each press asks the keyboard, by `focus`, of a name the drawing gives: a place round a date
//! picker, which takes no name of its own; a place of words alone; a place whose first field is
//! disabled; a place round a field, asked again once the field holds the keyboard; a name given to
//! a field and to a place round another field; a place inside an expander that starts closed,
//! opened in the step that asks it; and a place the asking step draws for the first time. Each
//! step and each ask is appended, a line each, to the file its first argument names, with Windows'
//! own answer when the field already holding the keyboard is asked it again.
//!
//! On a signed-in Windows desktop, start it with the path of a log, then run the reach example's
//! `hand.ps1` with `steps.txt` beside this crate's manifest. What the hand read of the keyboard's
//! focus after each step is then `expected-answers.txt`: the date picker; the press, where the
//! words hold nothing to take it; the field after the disabled one; the field asked twice; the
//! field that carries the shared name itself; the field inside the expander; and the new field.
//! A place whose border carries no reference of its name leaves the keyboard on the press that
//! asked, so the first answer is the first to differ.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!("inside shows the keyboard asked of a place, and runs on Windows alone");
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
        Border, Button, CalendarDatePicker, ElementRef, Expander, FocusEventInfo, StackPanel,
        TextBlock, TextBox, Thickness, View, WindowPlacement, WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window};

    /// The names the drawing gives.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Name {
        Day,
        Words,
        Disabled,
        Field,
        Shared,
        Closed,
        New,
    }

    struct Inside {
        log: PathBuf,
        /// A reference of the example's own, round the asked-twice field inside its place, by
        /// which Windows' own answer is read.
        round_field: ElementRef,
        open: bool,
        new: bool,
        twice: bool,
    }

    #[derive(Clone, Copy, Debug)]
    enum Message {
        Ask(Name),
        Twice,
        Open,
        Draw,
        FieldTook,
    }

    #[derive(Debug)]
    enum Effect {
        Ask(Name),
        /// Windows is asked directly to stand the keyboard in the asked-twice field's place, and
        /// its answer is written.
        AnswerField,
    }

    impl Program for Inside {
        type Message = Message;
        type Effect = Effect;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
            write(&self.log, &format!("step: {message:?}"));
            match message {
                Message::Ask(name) => vec![Effect::Ask(name)],
                Message::Twice => {
                    self.twice = true;
                    vec![Effect::Ask(Name::Field)]
                }
                Message::FieldTook if self.twice => {
                    self.twice = false;
                    vec![Effect::AnswerField, Effect::Ask(Name::Field)]
                }
                Message::FieldTook => Vec::new(),
                Message::Open => {
                    self.open = true;
                    vec![Effect::Ask(Name::Closed)]
                }
                Message::Draw => {
                    self.new = true;
                    vec![Effect::Ask(Name::New)]
                }
            }
        }
    }

    struct Hand {
        log: PathBuf,
        round_field: ElementRef,
    }

    impl Performer<Inside> for Hand {
        fn moment(&mut self) {}

        fn perform(&mut self, effect: Effect, _messages: &Sender<Message>) {
            match effect {
                Effect::Ask(name) => {
                    write(&self.log, &format!("ask: {name:?}"));
                    pamba_windows::focus(name);
                }
                Effect::AnswerField => {
                    let log = self.log.clone();
                    let queued = self.round_field.request_focus_result(move |answer| {
                        write(&log, &format!("answer: the field {answer:?}"));
                    });
                    write(&self.log, &format!("answer asked: {queued}"));
                }
            }
        }

        fn subscribe(&mut self, subscription: &Never, _messages: Sender<Message>) -> Running {
            match *subscription {}
        }
    }

    /// Appends one line to the log in one write, so that no line is torn.
    fn write(log: &PathBuf, line: &str) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log) {
            let _ = file.write_all(format!("{line}\n").as_bytes());
        }
    }

    fn press(name: &str, message: Message, messages: &Messages<Message>) -> View {
        Button::new()
            .automation_name(name)
            .content(name)
            .on_click(messages.message(message))
            .into()
    }

    fn field(name: &str) -> TextBox {
        TextBox::new().automation_name(name)
    }

    fn view(inside: &Inside, messages: &Messages<Message>) -> Window {
        let presses = vec![
            press("Ask the day", Message::Ask(Name::Day), messages),
            press("Ask the words", Message::Ask(Name::Words), messages),
            press(
                "Ask past the disabled",
                Message::Ask(Name::Disabled),
                messages,
            ),
            press("Ask the field twice", Message::Twice, messages),
            press("Ask the shared name", Message::Ask(Name::Shared), messages),
            press("Open and ask", Message::Open, messages),
            press("Draw and ask", Message::Draw, messages),
        ];
        let mut places: Vec<View> = vec![
            pamba_windows::place(
                Name::Day,
                CalendarDatePicker::new().automation_name("The day"),
            )
            .into(),
            pamba_windows::place(Name::Words, TextBlock::new().text("Only words stand here"))
                .into(),
            pamba_windows::place(
                Name::Disabled,
                StackPanel::new().children(vec![
                    field("The disabled field").is_enabled(false).into(),
                    field("After the disabled").into(),
                ]),
            )
            .into(),
            // The field's taking the keyboard rises to the place's border, which hears it.
            pamba_windows::place(
                Name::Field,
                Border::new()
                    .element_ref(&inside.round_field)
                    .content(field("The field")),
            )
            .on_got_focus(messages.callback(|_: FocusEventInfo| Message::FieldTook))
            .into(),
            StackPanel::new()
                .children(vec![
                    field("The named field")
                        .element_ref(&pamba_windows::named(Name::Shared))
                        .into(),
                    pamba_windows::place(Name::Shared, field("The field in the place")).into(),
                ])
                .into(),
            Expander::new()
                .header(TextBlock::new().text("More"))
                .is_expanded(inside.open)
                .content(pamba_windows::place(
                    Name::Closed,
                    field("Inside the expander"),
                ))
                .into(),
        ];
        if inside.new {
            places.push(pamba_windows::place(Name::New, field("The new field")).into());
        }
        let content = StackPanel::new()
            .spacing(8.0)
            .margin(Thickness::uniform(16.0))
            .children(vec![
                StackPanel::new().spacing(4.0).children(presses).into(),
                StackPanel::new().spacing(8.0).children(places).into(),
            ]);
        Window::new(content)
            .title("Inside")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 200,
                y: 40,
                width: 760,
                height: 900,
                maximized: false,
            }))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let log = std::env::args_os()
            .nth(1)
            .map_or_else(|| PathBuf::from("inside.log"), PathBuf::from);
        write(&log, "started");
        let round_field = ElementRef::new();
        pamba_windows::run(
            Inside {
                log: log.clone(),
                round_field: round_field.clone(),
                open: false,
                new: false,
                twice: false,
            },
            Vec::new(),
            Hand { log, round_field },
            view,
        )
    }
}
