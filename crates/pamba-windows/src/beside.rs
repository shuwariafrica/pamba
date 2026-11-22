//! The window as Windows itself knows it, for what is reached beside the reactor.
//!
//! The reactor hands a window's handle only for the length of one call. What Windows offers for
//! a window - the clipboard is opened for one - is asked for later, while an effect is
//! performed, so the handle is kept here for the thread its window stands on. Work asked of the
//! window's thread is handed the reactor's own handle for its call, as a `WindowHandle`.

#![allow(
    unsafe_code,
    reason = "the window is brought forward through Windows' C interface, and nothing safe beneath this crate offers it"
)]

use std::cell::Cell;
use std::error::Error;
use std::ffi::c_void;
use std::fmt;
use std::marker::PhantomData;
use std::ptr::null_mut;

/// `ShowWindow`'s ask to restore a minimised window to the size and position it had.
const RESTORE: i32 = 9;

#[link(name = "user32")]
unsafe extern "system" {
    fn IsWindow(window: *mut c_void) -> i32;
    fn IsIconic(window: *mut c_void) -> i32;
    fn ShowWindow(window: *mut c_void, show: i32) -> i32;
    fn SetForegroundWindow(window: *mut c_void) -> i32;
}

thread_local! {
    /// The window that stands on this thread, by Windows' handle; null where none does.
    static WINDOW: Cell<*mut c_void> = const { Cell::new(null_mut()) };
}

/// The window stands on this thread.
pub(crate) fn stands(window: &windows_reactor::WindowHandle<'_>) {
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

/// Windows' own handle of a program's window (its `HWND`), for as long as the work it is handed to
/// runs.
///
/// Work asked through [`WindowThread::run`](crate::WindowThread::run) is handed one, on the
/// window's thread, which the handle cannot leave.
#[derive(Debug, Clone, Copy)]
pub struct WindowHandle<'a> {
    raw: *mut c_void,
    /// The reactor holds the handle good for one call.
    call: PhantomData<&'a ()>,
}

impl WindowHandle<'_> {
    /// The handle the reactor gave for the call now running.
    pub(crate) fn new(raw: *mut c_void) -> Self {
        Self {
            raw,
            call: PhantomData,
        }
    }

    /// Whether Windows still knows the handle as a window's: no longer once the window is
    /// destroyed, as where work run before this in one hand-over closed it. Asked on the
    /// window's own thread, where nothing else destroys the window between the answer and its
    /// use.
    pub(crate) fn is_window(self) -> bool {
        // SAFETY: `IsWindow` takes any value for a handle and keeps nothing of it.
        unsafe { IsWindow(self.raw) != 0 }
    }

    /// The handle as Windows' own interfaces take it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn ask_over(_: *mut core::ffi::c_void) {}
    /// # fn main() -> Result<(), pamba_windows::Unrun> {
    /// pamba_windows::WindowThread::current().run(|handle| ask_over(handle.as_raw()))?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn as_raw(self) -> *mut c_void {
        self.raw
    }

    /// Brings the window forward: restored where it was minimised, in front of the other
    /// windows, with the keyboard.
    ///
    /// Windows gives the foreground only to a process it allows to take it, such as one the
    /// foreground process started or handed the right to with `AllowSetForegroundWindow`, and
    /// that right is lost at the person's next input elsewhere. So [`Behind`] is an answer to
    /// expect, not a fault. Where Windows refuses, it flashes the window's taskbar button
    /// instead.
    ///
    /// # Errors
    ///
    /// [`Behind`] where Windows did not give the window the foreground.
    ///
    /// # Examples
    ///
    /// A second launch knocks, and the running window comes forward on its own thread:
    ///
    /// ```no_run
    /// # fn main() -> Result<(), pamba_windows::Unrun> {
    /// pamba_windows::WindowThread::current().run(|handle| {
    ///     if let Err(behind) = handle.bring_forward() {
    ///         eprintln!("{behind}");
    ///     }
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn bring_forward(self) -> Result<(), Behind> {
        // SAFETY: `self.raw` is the handle the reactor gave for the window on this thread, for the
        // call now running; a handle no longer a window's is refused by each of the three calls,
        // which keep nothing of it.
        if unsafe { IsIconic(self.raw) } != 0 {
            // `ShowWindow` answers whether the window was visible before, not whether it was
            // restored, so its answer tells the caller nothing.
            // SAFETY: as above.
            let _ = unsafe { ShowWindow(self.raw, RESTORE) };
        }
        // SAFETY: as above.
        if unsafe { SetForegroundWindow(self.raw) } == 0 {
            Err(Behind)
        } else {
            Ok(())
        }
    }
}

/// The window was not brought forward: Windows kept it behind the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Behind;

impl fmt::Display for Behind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Windows kept the window behind the others")
    }
}

impl Error for Behind {}
