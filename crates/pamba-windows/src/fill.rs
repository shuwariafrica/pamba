//! The window's content held to the window's whole client area from its first showing.
//!
//! Once in many starts a window that opens maximised stands with its content laid out at the
//! window's restored size, in its upper left, and the rest of the window bare. The content's own
//! child window is then smaller than the window's client area, while the size the reactor reports
//! reads right. So a moment after the window first stands the two are read from Windows itself,
//! and where they differ the window is made to take its size again.

#![allow(
    unsafe_code,
    reason = "the window and its content's child window are read and mended through Windows' C interface, which nothing safe beneath this crate offers"
)]

use std::ffi::c_void;
use std::time::Duration;

use crate::beside;

/// How long after the window first stands its content is checked: past the first layout, as a
/// person sees the window.
pub(crate) const FIRST: Duration = Duration::from_secs(1);
/// How long after a mend the content is checked again.
pub(crate) const BETWEEN: Duration = Duration::from_millis(250);
/// How many mends are made at most. After them the window is left as it is: a window that is
/// wrong and usable is better than one worked at without end.
const MENDS: u32 = 3;
/// How far either side of the content may differ from the client area and still fill it.
const SLACK: i32 = 2;
/// The class of the child window that holds the window's content.
const CONTENT: &str = "Microsoft.UI.Content.DesktopChildSiteBridge";

/// What a check of the window's content comes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Checked {
    /// The content fills the window, or the window has no child of the content's class, so that
    /// nothing can be judged: nothing is done.
    Stands,
    /// The content is smaller or larger than the window: the window is mended, and checked again.
    Mended,
    /// The content is still wrong after every mend allowed: the window is left as it is.
    Left,
}

/// Decides from `content`, the client size of the content's child window where there is one,
/// `client`, the window's client size, both in Windows' pixels, and `mends`, the mends made so
/// far, whether the content stands, is mended, or is left as it is.
fn judge(content: Option<(i32, i32)>, client: (i32, i32), mends: u32) -> Checked {
    let Some((width, height)) = content else {
        return Checked::Stands;
    };
    if (width - client.0).abs() <= SLACK && (height - client.1).abs() <= SLACK {
        Checked::Stands
    } else if mends < MENDS {
        Checked::Mended
    } else {
        Checked::Left
    }
}

#[repr(C)]
#[derive(Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

type Found = unsafe extern "system" fn(*mut c_void, isize) -> i32;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetClientRect(window: *mut c_void, rect: *mut Rect) -> i32;
    fn EnumChildWindows(parent: *mut c_void, found: Found, held: isize) -> i32;
    fn GetClassNameW(window: *mut c_void, name: *mut u16, room: i32) -> i32;
    fn IsIconic(window: *mut c_void) -> i32;
    fn IsZoomed(window: *mut c_void) -> i32;
    fn SendMessageW(window: *mut c_void, message: u32, first: usize, second: isize) -> isize;
}

/// Windows' message that tells a window its client area took a new size.
const SIZE: u32 = 0x0005;
/// The state that message gives for a window Windows holds maximised.
const MAXIMISED: usize = 2;
/// The state it gives for a window at its own size.
const RESTORED: usize = 0;

/// The client size of `window` in its own pixels; none where Windows refuses the read.
fn client(window: *mut c_void) -> Option<(i32, i32)> {
    let mut rect = Rect::default();
    // SAFETY: `rect` is written by the call alone, which keeps nothing of it; a handle that is no
    // longer a window's is refused.
    (unsafe { GetClientRect(window, &raw mut rect) } != 0)
        .then_some((rect.right - rect.left, rect.bottom - rect.top))
}

/// Keeps the first child window Windows lists of the content's class, and stops the listing there.
unsafe extern "system" fn content_of(child: *mut c_void, held: isize) -> i32 {
    let mut name = [0_u16; 64];
    // SAFETY: the buffer holds the 64 units the call is told it may write; `child` is a handle
    // Windows is listing now.
    let length = unsafe { GetClassNameW(child, name.as_mut_ptr(), 64) };
    let named = usize::try_from(length)
        .ok()
        .and_then(|length| name.get(..length))
        .is_some_and(|name| name.iter().copied().eq(CONTENT.encode_utf16()));
    if !named {
        return 1;
    }
    // SAFETY: `held` is the address `content` lends for the length of the listing, on this
    // thread, and nothing else reaches it meanwhile.
    unsafe { *(held as *mut *mut c_void) = child };
    0
}

/// The child window of `window` that holds its content: Windows lists the window's children in no
/// fixed order, so it is found by its class.
fn content(window: *mut c_void) -> Option<*mut c_void> {
    let mut found: *mut c_void = std::ptr::null_mut();
    // SAFETY: `content_of` writes only through the address lent, which outlives the call, and the
    // call keeps nothing of it. The listing's own answer says nothing a child found does not.
    unsafe { EnumChildWindows(window, content_of, &raw mut found as isize) };
    (!found.is_null()).then_some(found)
}

/// Checks the content of the window that stands on this thread, after `mends` mends, and mends
/// the window where the content does not fill it.
///
/// A window minimised meanwhile has no client area to fill, and stands.
pub(crate) fn check(mends: u32) -> Checked {
    let Some(window) = beside::window() else {
        return Checked::Stands;
    };
    // SAFETY: the call reads the window's state alone; a handle that is no longer a window's
    // reads as not minimised, and the read of its client area below is then refused.
    if unsafe { IsIconic(window) } != 0 {
        return Checked::Stands;
    }
    let Some(size) = client(window) else {
        return Checked::Stands;
    };
    let checked = judge(content(window).and_then(client), size, mends);
    if checked == Checked::Mended {
        mend(window, size);
    }
    checked
}

/// Has `window` take its size again: Windows' size message is sent once more, with `size`, the
/// window's present client size, and its present state, and the content's child window is laid
/// out to it as at any change of size. Nothing of it shows where the content already fills the
/// window.
fn mend(window: *mut c_void, (width, height): (i32, i32)) {
    // A client area too large for the message's two halves cannot be told in it.
    let (Ok(width), Ok(height)) = (u16::try_from(width), u16::try_from(height)) else {
        return;
    };
    let Ok(size) = isize::try_from((u32::from(height) << 16) | u32::from(width)) else {
        return;
    };
    // SAFETY: `window` is the handle of the window that stands on this thread, read from
    // `beside` on this thread, so the message is handled here before the call returns and
    // nothing of the call is kept. Its answer says nothing.
    unsafe {
        let state = if IsZoomed(window) != 0 {
            MAXIMISED
        } else {
            RESTORED
        };
        SendMessageW(window, SIZE, state, size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIENT: (i32, i32) = (1920, 1009);

    #[test]
    fn content_as_large_as_the_client_area_stands_and_is_not_mended() {
        assert_eq!(judge(Some(CLIENT), CLIENT, 0), Checked::Stands);
    }

    #[test]
    fn content_within_two_pixels_either_way_stands() {
        for content in [(1918, 1007), (1922, 1011), (1918, 1011), (1922, 1007)] {
            assert_eq!(
                judge(Some(content), CLIENT, 0),
                Checked::Stands,
                "{content:?}"
            );
        }
    }

    #[test]
    fn content_three_pixels_off_on_either_side_is_mended() {
        for content in [(1917, 1009), (1923, 1009), (1920, 1006), (1920, 1012)] {
            assert_eq!(
                judge(Some(content), CLIENT, 0),
                Checked::Mended,
                "{content:?}"
            );
        }
    }

    #[test]
    fn content_at_the_restored_size_is_mended_until_the_last_mend_allowed() {
        for mends in 0..MENDS {
            assert_eq!(judge(Some((1264, 633)), CLIENT, mends), Checked::Mended);
        }
        assert_eq!(judge(Some((1264, 633)), CLIENT, MENDS), Checked::Left);
    }

    #[test]
    fn a_window_with_no_child_of_the_contents_class_stands() {
        assert_eq!(judge(None, CLIENT, 0), Checked::Stands);
    }

    #[test]
    fn content_that_never_fills_the_window_is_mended_three_times_and_then_left() {
        let mut mends = 0;
        loop {
            match judge(Some((1264, 633)), CLIENT, mends) {
                Checked::Mended => {
                    mends += 1;
                    assert!(mends <= MENDS, "mended {mends} times");
                }
                Checked::Left => break,
                Checked::Stands => panic!("content at the restored size stood"),
            }
        }
        assert_eq!(mends, MENDS);
    }

    #[test]
    fn content_that_fills_the_window_after_a_mend_stands() {
        assert_eq!(judge(Some(CLIENT), CLIENT, 1), Checked::Stands);
    }
}
