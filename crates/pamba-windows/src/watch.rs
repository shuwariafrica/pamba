//! How Windows draws, watched for a program: told at the start and whenever it changes.
//!
//! The reactor reports a window's colour mode and nothing of a contrast theme, and under a
//! contrast theme it reports the light mode whatever the theme's ground. So a contrast theme
//! turned on or off from the light mode changes nothing the reactor reports. Windows tells every
//! top-level window of such a change by a message, and the reactor's own window is closed to
//! this crate. So a window of this crate's own stands beside it on the same thread, is never
//! shown, and hears those messages. At each of them, and at each mode the reactor reports, how
//! Windows draws is read again, and the program is told where that differs from what it was
//! last told.

#![allow(
    unsafe_code,
    reason = "a window that hears Windows' messages is made through Windows' C interface, and the reactor closes its own"
)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::iter::once;
use std::mem::{replace, size_of};
use std::ptr::{null, null_mut};

use windows_reactor::{Callback, ColorScheme};

use crate::appearance::Appearance;

/// A setting of Windows' changed, a contrast theme's among them (`WM_SETTINGCHANGE`).
const SETTING_CHANGED: u32 = 26;
/// Windows' own colours changed (`WM_SYSCOLORCHANGE`).
const COLOURS_CHANGED: u32 = 21;
/// The theme changed (`WM_THEMECHANGED`).
const THEME_CHANGED: u32 = 794;

/// What Windows calls this crate's kind of window.
const CLASS: &str = "pamba-windows: how Windows draws";

/// A window's procedure, as Windows calls one.
type Procedure = unsafe extern "system" fn(*mut c_void, u32, usize, isize) -> isize;

/// A kind of window, as Windows registers one (`WNDCLASSEXW`).
#[repr(C)]
struct Class {
    size: u32,
    style: u32,
    procedure: Option<Procedure>,
    extra: i32,
    window_extra: i32,
    instance: *mut c_void,
    icon: *mut c_void,
    cursor: *mut c_void,
    background: *mut c_void,
    menu_name: *const u16,
    name: *const u16,
    small_icon: *mut c_void,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassExW(class: *const Class) -> u16;
    fn CreateWindowExW(
        extended: u32,
        class: *const u16,
        name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: *mut c_void,
        menu: *mut c_void,
        instance: *mut c_void,
        parameter: *mut c_void,
    ) -> *mut c_void;
    fn DefWindowProcW(window: *mut c_void, message: u32, word: usize, long: isize) -> isize;
    fn DestroyWindow(window: *mut c_void) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
}

/// What is watched on one thread.
struct Watch {
    /// This crate's own window on this thread; null where Windows made none.
    window: *mut c_void,
    /// The colour mode the reactor last reported, once it has reported one.
    mode: Option<ColorScheme>,
    /// What the program was last told.
    told: Option<Appearance>,
    /// Where the program asks to be told.
    appeared: Option<Callback<Appearance>>,
}

thread_local! {
    static WATCH: RefCell<Watch> = const {
        RefCell::new(Watch {
            window: null_mut(),
            mode: None,
            told: None,
            appeared: None,
        })
    };
}

/// The program's window stands on this thread: Windows' messages are heard here from now. Where
/// Windows makes no window, what the reactor reports is still told.
pub(crate) fn stands() {
    let Ok(size) = u32::try_from(size_of::<Class>()) else {
        return;
    };
    let name: Vec<u16> = CLASS.encode_utf16().chain(once(0)).collect();
    // SAFETY: a null name asks for the module the process started from, whose handle lives as
    // long as the process.
    let instance = unsafe { GetModuleHandleW(null()) };
    let class = Class {
        size,
        style: 0,
        procedure: Some(heard),
        extra: 0,
        window_extra: 0,
        instance,
        icon: null_mut(),
        cursor: null_mut(),
        background: null_mut(),
        menu_name: null(),
        name: name.as_ptr(),
        small_icon: null_mut(),
    };
    // SAFETY: `class` is a `WNDCLASSEXW` that states its own size, and `name` is UTF-16 ending
    // in a nought that lives through both calls; Windows copies what it keeps of each. `heard`
    // is a window procedure for the life of the process. A class this process registered for an
    // earlier window is refused here and still names the window made next, so the first call's
    // answer is not read. The window has no size, no parent and is never shown.
    let window = unsafe {
        RegisterClassExW(&raw const class);
        CreateWindowExW(
            0,
            name.as_ptr(),
            null(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        )
    };
    WATCH.with_borrow_mut(|watch| {
        watch.window = window;
        watch.mode = None;
        watch.told = None;
    });
}

/// The program's window has gone: nothing more is heard or told on this thread.
pub(crate) fn gone() {
    let window = WATCH.with_borrow_mut(|watch| {
        watch.mode = None;
        watch.told = None;
        watch.appeared = None;
        replace(&mut watch.window, null_mut())
    });
    if !window.is_null() {
        // SAFETY: `window` is the window `stands` made on this thread, and nothing else
        // destroys it. The watch is not borrowed while Windows calls `heard` for its end.
        unsafe {
            DestroyWindow(window);
        }
    }
}

/// Where the program asks to be told, as the drawing in hand has it; `None` where it does not
/// ask. A program that stops asking and asks again is told afresh, as the reactor reports the
/// mode afresh to a new subscription.
pub(crate) fn asks(appeared: Option<Callback<Appearance>>) {
    WATCH.with_borrow_mut(|watch| {
        if appeared.is_none() {
            watch.told = None;
        }
        watch.appeared = appeared;
    });
}

/// The reactor reports the window's colour mode.
pub(crate) fn mode(mode: ColorScheme) {
    WATCH.with_borrow_mut(|watch| watch.mode = Some(mode));
    tell();
}

/// Reads how Windows draws now and tells the program where that is not what it was last told.
/// Before the reactor has reported a mode nothing is told: the mode is half of what is read.
fn tell() {
    let due = WATCH.with_borrow_mut(|watch| {
        let now = Appearance::now(watch.mode?);
        if watch.told == Some(now) {
            return None;
        }
        let appeared = watch.appeared.clone()?;
        watch.told = Some(now);
        Some((appeared, now))
    });
    // The watch is no longer borrowed: what the program does on being told may come back here.
    if let Some((appeared, now)) = due {
        appeared.call(now);
    }
}

/// This crate's window procedure: a change in how Windows draws is read and told, and every
/// message is then Windows' own to answer.
unsafe extern "system" fn heard(
    window: *mut c_void,
    message: u32,
    word: usize,
    long: isize,
) -> isize {
    if matches!(message, SETTING_CHANGED | COLOURS_CHANGED | THEME_CHANGED) {
        tell();
    }
    // SAFETY: the four values are those Windows called this procedure with, for a window of
    // this crate's own class.
    unsafe { DefWindowProcW(window, message, word, long) }
}
