//! A word to the person through Windows itself.

#![allow(
    unsafe_code,
    reason = "Windows' own dialog is reached through its C interface, and nothing safe beneath this crate offers it"
)]

use std::ffi::c_void;
use std::iter::once;

/// `MB_OK | MB_ICONERROR | MB_SETFOREGROUND`: one press to close it, the mark of a failure, and in front.
const KIND: u32 = 0x0001_0010;

#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBoxW(owner: *mut c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(once(0)).collect()
}

/// Tells the person `message` in a dialog of Windows' own, titled `title`, and returns once it is
/// closed.
///
/// It is for the one moment no window of the program can speak: [`run`](crate::run) has returned
/// a [`Fault`](crate::Fault), so WinUI did not start. A program that then ends without a word
/// leaves the person who pressed it with nothing. The words are the program's own.
pub fn tell(title: &str, message: &str) {
    let (text, caption) = (wide(message), wide(title));
    // SAFETY: `text` and `caption` are NUL-terminated UTF-16 that outlive the call, which reads
    // them and keeps neither; a null owner is the documented way to show a dialog no window owns.
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), KIND);
    }
}
