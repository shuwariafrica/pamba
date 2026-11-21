//! The smallest program on Pamba's Windows layer.

#![cfg_attr(windows, windows_subsystem = "windows")]

use pamba::{Never, Program};

/// How many times the button was pressed.
struct Counter {
    count: u32,
}

#[derive(Clone)]
enum Message {
    Pressed,
}

impl Program for Counter {
    type Message = Message;
    type Effect = Never;
    type Subscription = Never;
    type Moment = ();

    fn update(&mut self, message: Message, _now: &()) -> Vec<Never> {
        match message {
            Message::Pressed => self.count += 1,
        }
        Vec::new()
    }
}

#[cfg(windows)]
fn view(counter: &Counter, messages: &pamba_windows::Messages<Message>) -> pamba_windows::Window {
    use pamba_windows::ui::{Button, StackPanel, TextBlock};

    let content = StackPanel::new().spacing(8.0).children((
        TextBlock::new().text(format!("Pressed {} times", counter.count)),
        Button::new()
            .on_click(messages.message(Message::Pressed))
            .content("Press"),
    ));
    pamba_windows::Window::new(content).title("Counter")
}

#[cfg(windows)]
fn main() -> Result<(), pamba_windows::Fault> {
    pamba_windows::run(Counter { count: 0 }, Vec::new(), (), view)
}

#[cfg(not(windows))]
fn main() {
    // The program is platform-free and its window is Windows': elsewhere it is only stepped.
    let mut counter = Counter { count: 0 };
    counter.update(Message::Pressed, &());
    println!("Pressed {} times", counter.count);
}
