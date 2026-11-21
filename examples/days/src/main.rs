//! Windows' date picker holding a day of the calendar, and telling the day a person picks.
//!
//! Three pickers stand in the window. The first holds the day given at the start, among any
//! days, and lets it go where a person clears it. The second holds it among the days of its own
//! month and never lets it go. The third holds the first one's day in a picker narrowed to its
//! glyph. Beside each of the first two stands the day the
//! program holds, as figures (`2026-11-21`), so that a picture shows Windows' own words for the
//! day against the figures it was given. Whatever a person picks or clears is appended, a line
//! each, to the file the first argument names. A run with the computer in one time zone and a
//! run with it in another write the same lines, which is what `DayPicker` is there for.
//!
//! On a signed-in Windows desktop: `days days.log 2026-11-21 2026-11-21`, the day held and
//! then the day the program takes to be today. Either may be left out.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!("days draws Windows' date picker, and runs there alone");
}

#[cfg(windows)]
fn main() -> Result<(), pamba_windows::Fault> {
    shown::run()
}

#[cfg(windows)]
mod shown {
    use std::ffi::OsString;
    use std::fs::OpenOptions;
    use std::io::Write as _;
    use std::path::PathBuf;

    use pamba::{Never, Performer, Program, Running, Sender};
    use pamba_windows::ui::{
        Border, Button, CalendarDatePicker, CalendarDay, DayPicker, Orientation, StackPanel,
        TextBlock, ThemeBrush, Thickness, VerticalAlignment, View, WindowPlacement, WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window};

    /// How wide a row's first words stand, so that every picker begins in one column.
    const LABEL: f64 = 200.0;
    /// How wide a picker stands.
    const PICKER: f64 = 220.0;

    struct Days {
        /// The day the first picker holds, among any days.
        any: Option<CalendarDay>,
        /// The day the second picker holds, among the days of one month.
        of_month: Option<CalendarDay>,
        /// The first and last day of that month; none where no day was given at the start.
        month: Option<(CalendarDay, CalendarDay)>,
        /// The day the program takes to be today.
        today: Option<CalendarDay>,
    }

    #[derive(Clone)]
    enum Message {
        Any(Option<CalendarDay>),
        OfMonth(Option<CalendarDay>),
        HoldNone,
    }

    #[derive(Debug)]
    enum Effect {
        Note(String),
    }

    impl Program for Days {
        type Message = Message;
        type Effect = Effect;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
            let note = match message {
                Message::Any(day) => {
                    self.any = day;
                    format!("any day: {}", figures(day))
                }
                // The second picker's day is never let go: pressed again in the calendar, Windows clears its own
                // picker and says so, and the program goes on holding the day and giving it.
                Message::OfMonth(None) => format!(
                    "a day of the month: none picked, still {}",
                    figures(self.of_month)
                ),
                Message::OfMonth(day) => {
                    self.of_month = day;
                    format!("a day of the month: {}", figures(day))
                }
                Message::HoldNone => {
                    self.any = None;
                    self.of_month = None;
                    "the program holds no day".to_owned()
                }
            };
            vec![Effect::Note(note)]
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

    impl Performer<Days> for Hand {
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

    /// A day as figures, or that there is none.
    fn figures(day: Option<CalendarDay>) -> String {
        day.map_or_else(
            || "no day".to_owned(),
            |day| format!("{:04}-{:02}-{:02}", day.year(), day.month(), day.day()),
        )
    }

    /// The day `2026-11-21` states, where it states one.
    fn stated(figures: Option<OsString>) -> Option<CalendarDay> {
        let figures = figures?.into_string().ok()?;
        let mut parts = figures.split('-');
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        CalendarDay::new(year, month, day)
    }

    /// The first and the last day of the month `day` is in.
    fn month_of(day: CalendarDay) -> Option<(CalendarDay, CalendarDay)> {
        let first = CalendarDay::new(day.year(), day.month(), 1)?;
        let last = (28..=31)
            .rev()
            .find_map(|of_month| CalendarDay::new(day.year(), day.month(), of_month))?;
        Some((first, last))
    }

    /// One picker's row: what it is, the picker under a name of its own, and the day the program holds as figures.
    /// The picker's name is not the label's words, so that whoever works the window by a control's name finds the
    /// picker and not the words beside it.
    fn row(label: &str, name: &str, picker: CalendarDatePicker, held: Option<CalendarDay>) -> View {
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(16.0)
            .children((
                TextBlock::new()
                    .text(label)
                    .width(LABEL)
                    .vertical_alignment(VerticalAlignment::Center),
                picker.width(PICKER).automation_name(name),
                TextBlock::new()
                    .text(figures(held))
                    .vertical_alignment(VerticalAlignment::Center),
            ))
            .into()
    }

    /// The first picker's day again, in a picker forty across: Windows draws the calendar's glyph alone there, with
    /// no words cut short, and the press opens the same calendar. It is the form for a program that says the day in
    /// words of its own beside the press.
    fn row_narrow(days: &Days, messages: &Messages<Message>) -> View {
        StackPanel::new()
            .orientation(Orientation::Horizontal)
            .spacing(16.0)
            .children((
                TextBlock::new()
                    .text("Narrow")
                    .width(LABEL)
                    .vertical_alignment(VerticalAlignment::Center),
                CalendarDatePicker::new()
                    .days(days.any, ..)
                    .on_day_changed(messages.callback(Message::Any))
                    .min_width(40.0)
                    .width(40.0)
                    .automation_name("Pick, narrow"),
            ))
            .into()
    }

    fn view(days: &Days, messages: &Messages<Message>) -> Window {
        let any = CalendarDatePicker::new()
            .placeholder_text("No day")
            .days(days.any, ..)
            .today(days.today)
            .on_day_changed(messages.callback(Message::Any));
        let of_month = CalendarDatePicker::new().placeholder_text("No day");
        let of_month = match days.month {
            Some((first, last)) => of_month.days(days.of_month, first..=last),
            None => of_month.days(days.of_month, ..),
        }
        .today(days.today)
        .on_day_changed(messages.callback(Message::OfMonth));
        let content = StackPanel::new().spacing(12.0).children((
            row("Any day", "Pick any day", any, days.any),
            row(
                "A day of the month",
                "Pick a day of the month",
                of_month,
                days.of_month,
            ),
            TextBlock::new().text(format!("Today, to the program: {}", figures(days.today))),
            row_narrow(days, messages),
            Button::new()
                .automation_name("Hold no day")
                .on_click(messages.message(Message::HoldNone))
                .content("Hold no day"),
        ));
        let content = Border::new()
            .background(ThemeBrush::SolidBackground)
            .padding(Thickness::uniform(16.0))
            .content(content);
        Window::new(content)
            .title("Days")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 100,
                y: 100,
                width: 640,
                height: 560,
                maximized: false,
            }))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let mut arguments = std::env::args_os().skip(1);
        let log = arguments
            .next()
            .map_or_else(|| PathBuf::from("days.log"), PathBuf::from);
        let held = stated(arguments.next());
        let today = stated(arguments.next());
        let hand = Hand { log };
        hand.note(&format!(
            "days starts: holds {}, today is {}",
            figures(held),
            figures(today)
        ));
        let days = Days {
            any: held,
            of_month: held,
            month: held.and_then(month_of),
            today,
        };
        pamba_windows::run(days, Vec::new(), hand, view)
    }
}
