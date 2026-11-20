//! The clipboard, through Windows itself.

#![allow(
    unsafe_code,
    reason = "the clipboard is reached through Windows' C interface, and nothing safe beneath this crate offers it"
)]

use std::error::Error;
use std::ffi::c_void;
use std::fmt;
use std::iter::once;
use std::thread::sleep;
use std::time::Duration;

use crate::beside;

/// Text in UTF-16, as the clipboard names it.
const UNICODE_TEXT: u32 = 13;
/// A block Windows may move, the kind the clipboard takes.
const MOVEABLE: u32 = 0x0002;
/// Windows' code for a clipboard another program holds open.
const ACCESS_DENIED: u32 = 5;
/// How often the clipboard is asked for while another program holds it, and the wait between:
/// forty milliseconds at the longest, on the window's thread.
const TRIES: u32 = 5;
const BETWEEN: Duration = Duration::from_millis(10);

#[link(name = "user32")]
unsafe extern "system" {
    fn OpenClipboard(owner: *mut c_void) -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(format: u32, data: *mut c_void) -> *mut c_void;
    fn CloseClipboard() -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> *mut c_void;
    fn GlobalLock(block: *mut c_void) -> *mut c_void;
    fn GlobalUnlock(block: *mut c_void) -> i32;
    fn GlobalFree(block: *mut c_void) -> *mut c_void;
    fn GetLastError() -> u32;
}

/// Why text was not put on the clipboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Uncopied {
    /// No window of the program stands on the thread that asked.
    NoWindow,
    /// Another program held the clipboard open for as long as it was asked for.
    Held,
    /// Windows refused, with its own code for why.
    Refused(u32),
}

impl fmt::Display for Uncopied {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoWindow => formatter.write_str("no window of the program stands on this thread"),
            Self::Held => formatter.write_str("another program holds the clipboard open"),
            Self::Refused(code) => {
                write!(formatter, "Windows refused the clipboard (error {code})")
            }
        }
    }
}

impl Error for Uncopied {}

fn refused() -> Uncopied {
    // SAFETY: the call reads this thread's last error and takes no argument.
    Uncopied::Refused(unsafe { GetLastError() })
}

/// Opens the clipboard for `window`, asking again for a moment while another program holds it.
fn open(window: *mut c_void) -> Result<(), Uncopied> {
    for attempt in 1..=TRIES {
        // SAFETY: `window` is a handle Windows gave for the window on this thread; one that is
        // no longer a window's is refused by the call, which keeps nothing of it.
        if unsafe { OpenClipboard(window) } != 0 {
            return Ok(());
        }
        let why = refused();
        if why != Uncopied::Refused(ACCESS_DENIED) {
            return Err(why);
        }
        if attempt < TRIES {
            sleep(BETWEEN);
        }
    }
    Err(Uncopied::Held)
}

/// Makes `wide` the whole content of the clipboard, which this thread holds open.
fn put(wide: &[u16]) -> Result<(), Uncopied> {
    // SAFETY: the block is allocated for exactly the units copied into it and is unlocked before
    // it is handed over; the clipboard owns it from `SetClipboardData` on, and it is freed here
    // only on the paths where Windows did not take it.
    unsafe {
        if EmptyClipboard() == 0 {
            return Err(refused());
        }
        let block = GlobalAlloc(MOVEABLE, size_of_val(wide));
        if block.is_null() {
            return Err(refused());
        }
        let to = GlobalLock(block).cast::<u16>();
        if to.is_null() {
            let why = refused();
            GlobalFree(block);
            return Err(why);
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), to, wide.len());
        GlobalUnlock(block);
        if SetClipboardData(UNICODE_TEXT, block).is_null() {
            let why = refused();
            GlobalFree(block);
            return Err(why);
        }
    }
    Ok(())
}

/// Puts `text` on the clipboard as its whole content, for the program's window.
///
/// It is called where a program's effects are performed: on the window's thread, while the
/// window stands. Where another program holds the clipboard open it asks again for at most
/// forty milliseconds.
///
/// ```no_run
/// # enum Effect { Copy(String) }
/// # enum Message { Copied(Result<(), pamba_windows::Uncopied>) }
/// # fn perform(effect: Effect) -> Message {
/// match effect {
///     Effect::Copy(text) => Message::Copied(pamba_windows::copy(&text)),
/// }
/// # }
/// ```
///
/// # Errors
///
/// [`Uncopied::NoWindow`] where no window of the program stands on this thread;
/// [`Uncopied::Held`] where another program kept the clipboard open throughout; and
/// [`Uncopied::Refused`] with Windows' code where it refused a step.
pub fn copy(text: &str) -> Result<(), Uncopied> {
    let window = beside::window().ok_or(Uncopied::NoWindow)?;
    let wide: Vec<u16> = text.encode_utf16().chain(once(0)).collect();
    open(window)?;
    let put = put(&wide);
    // SAFETY: this thread opened the clipboard just above and closes it once.
    unsafe {
        CloseClipboard();
    }
    put
}
