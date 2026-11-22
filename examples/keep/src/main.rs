//! Where a window of Pamba's keeps the keyboard when a pointer's press lands on what takes no keys.
//!
//! It draws a sentence, a press, a field, a second press and one that takes every control away,
//! on bare ground, and the program asks for nothing about the keyboard. Every press of the first
//! press is appended, a line each, to the file its first argument names.
//!
//! On a signed-in Windows desktop, start it with the path of a log, then run the reach example's
//! `hand.ps1` with `steps.txt` beside this crate's manifest; the steps find each place by its name
//! or as a share of the window, so they hold at any scale. What the hand read of the keyboard's
//! focus after each step is then `expected-answers.txt`: the press keeps it through a press on the
//! ground, on the sentence and on the title band, the field takes it when it is pressed, Tab moves
//! it on, a press on the press takes it back and is written to the log, and once nothing the
//! program drew can hold it a press on the ground leaves it where it was.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!("keep shows where a window keeps the keyboard, and runs on Windows alone");
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
        Border, Button, StackPanel, TextBlock, TextBox, ThemeBrush, Thickness, WindowPlacement,
        WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window};

    struct Keep {
        presses: u32,
        bare: bool,
    }

    #[derive(Clone, Copy, Debug)]
    enum Message {
        Pressed,
        Bared,
    }

    impl Program for Keep {
        type Message = Message;
        type Effect = u32;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<u32> {
            match message {
                Message::Pressed => {
                    self.presses += 1;
                    vec![self.presses]
                }
                Message::Bared => {
                    self.bare = true;
                    Vec::new()
                }
            }
        }
    }

    struct Hand {
        log: PathBuf,
    }

    impl Performer<Keep> for Hand {
        fn moment(&mut self) {}

        fn perform(&mut self, presses: u32, _messages: &Sender<Message>) {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.log) {
                let _ = writeln!(file, "pressed {presses}");
            }
        }

        fn subscribe(&mut self, subscription: &Never, _messages: Sender<Message>) -> Running {
            match *subscription {}
        }
    }

    fn view(keep: &Keep, messages: &Messages<Message>) -> Window {
        let sentence = TextBlock::new().text(format!(
            "A sentence, which takes no keys. Pressed {} times.",
            keep.presses
        ));
        let content = StackPanel::new()
            .spacing(12.0)
            .margin(Thickness::uniform(16.0));
        let content = if keep.bare {
            content.children((sentence,))
        } else {
            content.children((
                sentence,
                Button::new()
                    .width(160.0)
                    .automation_name("Press")
                    .on_click(messages.message(Message::Pressed))
                    .content("Press"),
                TextBox::new().width(320.0).automation_name("Field"),
                Button::new()
                    .width(160.0)
                    .automation_name("After the field")
                    .content("After the field"),
                Button::new()
                    .automation_name("Take every control away")
                    .on_click(messages.message(Message::Bared))
                    .content("Take every control away"),
            ))
        };
        // The ground: the rest of the window, painted, under what is drawn on it.
        let ground = Border::new()
            .background(ThemeBrush::CardBackground)
            .content(content);
        Window::new(ground)
            .title("Keep")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 300,
                y: 200,
                width: 900,
                height: 640,
                maximized: false,
            }))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let log = std::env::args_os()
            .nth(1)
            .map_or_else(|| PathBuf::from("keep.log"), PathBuf::from);
        pamba_windows::run(
            Keep {
                presses: 0,
                bare: false,
            },
            Vec::new(),
            Hand { log },
            view,
        )
    }
}
