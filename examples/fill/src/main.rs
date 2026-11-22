//! A window that opens filling the screen, read from Windows a moment after it stands.
//!
//! It asks, as Azkaban does, for a window restored at 40, 40 and 1,280 by 672 that opens
//! maximised, and fills it with one colour and a line of words at its middle, so that a picture
//! shows whether the content fills the window. Half a second after Windows first tells it the
//! window's size, and again three seconds after, it writes one line of what Windows holds:
//!
//! ```text
//! read 1: client 1920x1009 px, dpi 96, zoomed true, reported 1920x1009 dip (1920x1009 px), children [Microsoft.UI.Content.DesktopChildSiteBridge 1920x1009 px]; reported right, content right
//! ```
//!
//! "Reported" is the size the program was last told (`Window::on_size`), and "content" is the
//! client rectangle of the child window of class `Microsoft.UI.Content.DesktopChildSiteBridge`,
//! which holds the window's content. Either is wrong where it differs from the window's client
//! rectangle by more than two pixels either way.
//!
//! The first argument names the log the lines are appended to. A second, `--induce`, makes the
//! wrong state on purpose: just before the first line, the content's child window is given the
//! window's restored size while the window stands maximised.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    println!("fill reads a window from Windows, and runs there alone");
}

#[cfg(windows)]
fn main() -> Result<(), pamba_windows::Fault> {
    shown::run()
}

#[cfg(windows)]
#[allow(
    unsafe_code,
    reason = "the window is read and the wrong state made through Windows' C interface, which nothing safe beneath Pamba offers"
)]
mod windows {
    use std::ffi::c_void;
    use std::fmt::Write as _;

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Placement {
        length: u32,
        flags: u32,
        show: u32,
        least: Point,
        most: Point,
        normal: Rect,
    }

    type Found = unsafe extern "system" fn(*mut c_void, isize) -> i32;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumThreadWindows(thread: u32, found: Found, held: isize) -> i32;
        fn EnumChildWindows(parent: *mut c_void, found: Found, held: isize) -> i32;
        fn IsWindowVisible(window: *mut c_void) -> i32;
        fn GetClientRect(window: *mut c_void, rect: *mut Rect) -> i32;
        fn GetDpiForWindow(window: *mut c_void) -> u32;
        fn IsZoomed(window: *mut c_void) -> i32;
        fn GetClassNameW(window: *mut c_void, name: *mut u16, room: i32) -> i32;
        fn GetParent(window: *mut c_void) -> *mut c_void;
        fn GetWindowPlacement(window: *mut c_void, placement: *mut Placement) -> i32;
        fn GetWindowLongPtrW(window: *mut c_void, index: i32) -> isize;
        fn AdjustWindowRectExForDpi(
            rect: *mut Rect,
            style: u32,
            menu: i32,
            extended: u32,
            dpi: u32,
        ) -> i32;
        fn SetWindowPos(
            window: *mut c_void,
            after: *mut c_void,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            flags: u32,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentThreadId() -> u32;
    }

    const STYLE: i32 = -16;
    const EXTENDED_STYLE: i32 = -20;
    const NO_ZORDER_NO_MOVE_NO_ACTIVATE: u32 = 0x0004 | 0x0002 | 0x0010;

    unsafe extern "system" fn gather(window: *mut c_void, held: isize) -> i32 {
        // SAFETY: `held` is the address of the vector `windows` lends for the length of the call
        // that calls this, on this thread, and nothing else holds it meanwhile.
        let found = unsafe { &mut *(held as *mut Vec<*mut c_void>) };
        found.push(window);
        1
    }

    fn windows(of: Option<*mut c_void>) -> Vec<*mut c_void> {
        let mut found: Vec<*mut c_void> = Vec::new();
        let held = &raw mut found as isize;
        // SAFETY: `gather` writes only through `held`, which outlives the call, and the calls
        // keep nothing of it.
        unsafe {
            match of {
                Some(parent) => EnumChildWindows(parent, gather, held),
                None => EnumThreadWindows(GetCurrentThreadId(), gather, held),
            };
        }
        found
    }

    fn class(window: *mut c_void) -> String {
        let mut name = [0_u16; 128];
        // SAFETY: the buffer holds the 128 units the call is told it may write.
        let length = unsafe { GetClassNameW(window, name.as_mut_ptr(), 128) };
        String::from_utf16_lossy(
            name.get(..usize::try_from(length).unwrap_or(0))
                .unwrap_or(&[]),
        )
    }

    /// The child window of `window` that holds its content. Windows lists the window's children
    /// in no fixed order: under load a child with no size of its own came first.
    fn content(window: *mut c_void) -> Option<*mut c_void> {
        windows(Some(window))
            .into_iter()
            .find(|child| class(*child) == "Microsoft.UI.Content.DesktopChildSiteBridge")
    }

    fn client(window: *mut c_void) -> (i32, i32) {
        let mut rect = Rect::default();
        // SAFETY: `rect` is written by the call alone; a handle that is no window is refused.
        unsafe {
            GetClientRect(window, &raw mut rect);
        }
        (rect.right - rect.left, rect.bottom - rect.top)
    }

    /// The visible top-level window of this thread: the program's.
    fn top() -> Option<*mut c_void> {
        // SAFETY: each handle is one Windows gave for this thread just now.
        windows(None)
            .into_iter()
            .find(|window| unsafe { IsWindowVisible(*window) != 0 && GetParent(*window).is_null() })
    }

    fn wrong(seen: (i32, i32), client: (i32, i32)) -> &'static str {
        if (seen.0 - client.0).abs() > 2 || (seen.1 - client.1).abs() > 2 {
            "wrong"
        } else {
            "right"
        }
    }

    /// One line of what Windows holds of the program's window, beside the size it was last told.
    pub(super) fn reading(read: u32, reported: Option<(f64, f64)>) -> String {
        let Some(window) = top() else {
            return format!("read {read}: no visible window on this thread");
        };
        let (width, height) = client(window);
        // SAFETY: `window` is the handle Windows just gave; the calls read it and keep nothing.
        let (dpi, zoomed) = unsafe { (GetDpiForWindow(window), IsZoomed(window) != 0) };
        let scale = f64::from(dpi.max(96)) / 96.0;
        let children = windows(Some(window));
        let mut line =
            format!("read {read}: client {width}x{height} px, dpi {dpi}, zoomed {zoomed}, ");
        let told = reported.map(|(w, h)| {
            #[allow(
                clippy::cast_possible_truncation,
                reason = "a size on the screen fits an `i32`"
            )]
            let px = ((w * scale).round() as i32, (h * scale).round() as i32);
            let _ = write!(line, "reported {w:.0}x{h:.0} dip ({}x{} px), ", px.0, px.1);
            px
        });
        if told.is_none() {
            line.push_str("reported nothing, ");
        }
        line.push_str("children [");
        for (index, child) in children.iter().enumerate() {
            let (w, h) = client(*child);
            let _ = write!(
                line,
                "{}{} {w}x{h} px",
                if index == 0 { "" } else { ", " },
                class(*child)
            );
        }
        line.push_str("]; ");
        let _ = write!(
            line,
            "reported {}, content {}",
            told.map_or("unknown", |px| wrong(px, (width, height))),
            content(window).map_or("unknown", |child| wrong(client(child), (width, height)))
        );
        line
    }

    /// Gives the content's child window the window's restored client size while it stands
    /// maximised, and says what it did.
    pub(super) fn induce() -> String {
        let Some(window) = top() else {
            return "induce: no visible window on this thread".to_owned();
        };
        let Some(child) = content(window) else {
            return "induce: the window has no child window that holds its content".to_owned();
        };
        let mut placement = Placement {
            length: u32::try_from(size_of::<Placement>()).unwrap_or(0),
            ..Placement::default()
        };
        let mut frame = Rect::default();
        // SAFETY: each call writes only the structure it is handed, sized as it expects, and reads
        // the window by a handle Windows just gave.
        let moved = unsafe {
            GetWindowPlacement(window, &raw mut placement);
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a window's styles are 32 bits"
            )]
            AdjustWindowRectExForDpi(
                &raw mut frame,
                GetWindowLongPtrW(window, STYLE) as u32,
                0,
                GetWindowLongPtrW(window, EXTENDED_STYLE) as u32,
                GetDpiForWindow(window),
            );
            let width =
                (placement.normal.right - placement.normal.left) - (frame.right - frame.left);
            let height =
                (placement.normal.bottom - placement.normal.top) - (frame.bottom - frame.top);
            let moved = SetWindowPos(
                child,
                std::ptr::null_mut(),
                0,
                0,
                width,
                height,
                NO_ZORDER_NO_MOVE_NO_ACTIVATE,
            );
            (width, height, moved != 0)
        };
        format!(
            "induce: {} given {}x{} px, the restored client size: done {}",
            class(child),
            moved.0,
            moved.1,
            moved.2
        )
    }
}

#[cfg(windows)]
mod shown {
    use std::fs::OpenOptions;
    use std::io::Write as _;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    use pamba::{Never, Performer, Program, Running, Sender};
    use pamba_windows::ui::{
        Border, HorizontalAlignment, TextBlock, ThemeBrush, VerticalAlignment, WindowPlacement,
        WindowSize, WindowVisuals,
    };
    use pamba_windows::{Fault, Messages, Window};

    use crate::windows::{induce, reading};

    /// When the lines are written, after Windows first tells the window's size.
    const FIRST: Duration = Duration::from_millis(500);
    const SECOND: Duration = Duration::from_millis(2500);

    struct Fill {
        reported: Option<WindowSize>,
        reads: u32,
        induce: bool,
    }

    #[derive(Clone, Copy, Debug)]
    enum Message {
        Sized(WindowSize),
        Read,
    }

    #[derive(Debug)]
    enum Effect {
        Later(Duration),
        Induce,
        Read(u32, Option<(f64, f64)>),
    }

    impl Program for Fill {
        type Message = Message;
        type Effect = Effect;
        type Subscription = Never;
        type Moment = ();

        fn update(&mut self, message: Message, _now: &()) -> Vec<Effect> {
            match message {
                Message::Sized(size) => {
                    let first = self.reported.is_none();
                    self.reported = Some(size);
                    if first {
                        vec![Effect::Later(FIRST)]
                    } else {
                        Vec::new()
                    }
                }
                Message::Read => {
                    self.reads += 1;
                    let reported = self.reported.map(|size| (size.width, size.height));
                    let mut effects = Vec::new();
                    if self.reads == 1 && self.induce {
                        effects.push(Effect::Induce);
                    }
                    effects.push(Effect::Read(self.reads, reported));
                    if self.reads == 1 {
                        effects.push(Effect::Later(SECOND));
                    }
                    effects
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

    impl Performer<Fill> for Hand {
        fn moment(&mut self) {}

        fn perform(&mut self, effect: Effect, messages: &Sender<Message>) {
            match effect {
                Effect::Later(wait) => {
                    let messages = messages.clone();
                    thread::spawn(move || {
                        thread::sleep(wait);
                        // Refused only once the window has gone.
                        let _ = messages.send(Message::Read);
                    });
                }
                Effect::Induce => self.note(&induce()),
                Effect::Read(read, reported) => {
                    // The induced size reaches the content a moment after the call.
                    if read == 1 {
                        thread::sleep(Duration::from_millis(200));
                    }
                    self.note(&reading(read, reported));
                }
            }
        }

        fn subscribe(&mut self, subscription: &Never, _messages: Sender<Message>) -> Running {
            match *subscription {}
        }
    }

    fn view(_fill: &Fill, messages: &Messages<Message>) -> Window {
        let content = Border::new().background(ThemeBrush::Accent).content(
            TextBlock::new()
                .text("The middle of the content")
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center),
        );
        Window::new(content)
            .title("Fill")
            .visuals(WindowVisuals::new().initial_placement(WindowPlacement {
                x: 40,
                y: 40,
                width: 1280,
                height: 672,
                maximized: true,
            }))
            .on_size(messages.callback(Message::Sized))
    }

    pub(super) fn run() -> Result<(), Fault> {
        let mut arguments = std::env::args_os().skip(1);
        let log = arguments
            .next()
            .map_or_else(|| PathBuf::from("fill.log"), PathBuf::from);
        let induce = arguments.next().is_some_and(|switch| switch == "--induce");
        let hand = Hand { log: log.clone() };
        hand.note(&format!(
            "fill starts{}",
            if induce {
                ", to induce the wrong state"
            } else {
                ""
            }
        ));
        let ended = pamba_windows::run(
            Fill {
                reported: None,
                reads: 0,
                induce,
            },
            Vec::new(),
            hand,
            view,
        );
        Hand { log }.note(&match &ended {
            Ok(()) => "the window has closed".to_owned(),
            Err(fault) => format!("the window ended: {fault}"),
        });
        ended
    }
}
