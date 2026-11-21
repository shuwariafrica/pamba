//! The window as Windows itself knows it, for what is reached beside the reactor.
//!
//! The reactor hands a window's handle only for the length of one call. What Windows offers for
//! a window - the clipboard is opened for one - is asked for later, while an effect is
//! performed, so the handle is kept here for the thread its window stands on.

use std::cell::Cell;
use std::ffi::c_void;
use std::ptr::null_mut;

use windows_reactor::WindowHandle;

thread_local! {
    /// The window that stands on this thread, by Windows' handle; null where none does.
    static WINDOW: Cell<*mut c_void> = const { Cell::new(null_mut()) };
}

/// The window stands on this thread.
pub(crate) fn stands(window: &WindowHandle<'_>) {
    WINDOW.with(|held| held.set(window.as_raw()));
}

/// The window has gone: nothing is asked of Windows for it again.
pub(crate) fn gone() {
    WINDOW.with(|held| held.set(null_mut()));
}

/// The window that stands on this thread, where one does.
pub(crate) fn window() -> Option<*mut c_void> {
    let window = WINDOW.with(Cell::get);
    (!window.is_null()).then_some(window)
}
