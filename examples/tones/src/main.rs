//! The colours of Windows' theme that Pamba names, each drawn as Windows draws now.
//!
//! Every member of `FillColor` stands in a row of its own: its name, then the colour as words, as
//! an edge and as a ground, on the window's own ground, resolved for the appearance the window
//! was last told. Until an appearance is told nothing is drawn but a line that says so.
//! Every appearance the program is told is appended, a line each, to the file its first argument
//! names, so a run under each colour mode and each contrast theme shows what the program was
//! told and when.
//!
//! On a signed-in Windows desktop: `tones tones.log`.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!("tones draws Windows' theme colours, and runs there alone");
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
        Appearance, Border, FillColor, Orientation, StackPanel, TextBlock, ThemeBrush, Thickness,
        VerticalAlignment, View, WindowPlacement, WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window};

    /// How wide a colour's name stands, so that every row's samples begin in one column.
    const NAME: f64 = 280.0;
    /// How wide and how tall one sample stands.
    const SAMPLE: (f64, f64) = (96.0, 28.0);

    struct Tones {
        /// How Windows draws, as last told.
        appearance: Option<Appearance>,
        /// How many times the program has been told.
        told: u32,
    }

    #[derive(Clone)]
    enum Message {
        Appeared(Appearance),
    }

    #[derive(Debug)]
    enum Effect {
        Note(String),
    }

    impl Program for Tones {
        type Message = Message;
        type Effect = Effect;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
            match message {
                Message::Appeared(appearance) => {
                    self.told += 1;
                    self.appearance = Some(appearance);
                    vec![Effect::Note(format!("told {}: {appearance:?}", self.told))]
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

    impl Performer<Tones> for Hand {
        fn moment(&mut self) {}

        fn perform(&mut self, effect: Effect, _messages: &Sender<Message>) {
            match effect {
                Effect::Note(line) => self.note(&line),
            }
        }

        fn subscribe(&mut self, subscription: &Never, _messages: Sender<Message>) -> Running {
            match *subscription {}
        }
    }

    /// One colour's row: its name, then the colour as words, as an edge and as a ground.
    fn row(appearance: Appearance, colour: FillColor) -> View {
        let (width, height) = SAMPLE;
        let brush = appearance.brush(colour);
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(16.0)
            .children((
                TextBlock::new()
                    .text(format!("{colour:?}"))
                    .width(NAME)
                    .vertical_alignment(VerticalAlignment::Center),
                TextBlock::new()
                    .text("Words")
                    .width(width)
                    .foreground(brush)
                    .vertical_alignment(VerticalAlignment::Center),
                Border::new()
                    .width(width)
                    .height(height)
                    .border_brush(brush)
                    .border_thickness(Thickness::uniform(2.0)),
                Border::new().width(width).height(height).background(brush),
            ))
            .into()
    }

    fn view(tones: &Tones, messages: &Messages<Message>) -> Window {
        let content: View = match tones.appearance {
            None => TextBlock::new()
                .text("Not yet told how Windows draws")
                .into(),
            Some(appearance) => StackPanel::new()
                .spacing(8.0)
                .children(FillColor::ALL.map(|colour| row(appearance, colour)))
                .into(),
        };
        let content = Border::new()
            .background(ThemeBrush::SolidBackground)
            .padding(Thickness::uniform(16.0))
            .content(content);
        Window::new(content)
            .title("Tones")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 100,
                y: 100,
                width: 760,
                height: 620,
                maximized: false,
            }))
            .on_appearance(messages.callback(Message::Appeared))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let log = std::env::args_os()
            .nth(1)
            .map_or_else(|| PathBuf::from("tones.log"), PathBuf::from);
        let hand = Hand { log };
        hand.note("tones starts");
        let tones = Tones {
            appearance: None,
            told: 0,
        };
        pamba_windows::run(tones, Vec::new(), hand, view)
    }
}
