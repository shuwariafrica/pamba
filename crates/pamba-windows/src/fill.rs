//! The window's content held to the window's whole client area from its first showing.
//!
//! Once in many starts a window that opens maximised stands with its content laid out at the
//! window's restored size, in its upper left, and the rest of the window bare. Either of two parts
//! can be small: the content's own child window, or what is laid out inside a child window that
//! fills the window. So a moment after the window first stands both are read and compared with the
//! window's client area, the window is mended where either does not fill it, lighter mends first,
//! and the program is told what was found and done.

#![allow(
    unsafe_code,
    reason = "the window and its content's child window are read and mended through Windows' C interface, which nothing safe beneath this crate offers"
)]

use std::ffi::c_void;
use std::time::Duration;

use crate::beside;
use crate::unfilled::Unfilled;

/// How long after the window first stands its content is checked: past the first layout, as a
/// person sees the window.
pub(crate) const FIRST: Duration = Duration::from_secs(1);
/// How long after a mend, or after a check that had no laid-out size to judge, the content is
/// checked again.
pub(crate) const BETWEEN: Duration = Duration::from_millis(250);
/// How many mends are made at most. After them the window is left as it is: a window that is
/// wrong and usable is better than one worked at without end.
const MENDS: u32 = 3;
/// How many checks in all wait for a laid-out size that has not been given yet.
const WAITS: u32 = 3;
/// How far either side of the content may differ from the client area and still fill it, in the
/// window's pixels.
const SLACK: i32 = 2;
/// The class of the child window that holds the window's content.
const CONTENT: &str = "Microsoft.UI.Content.DesktopChildSiteBridge";

/// The size the window's content is laid out at, in effective pixels, and the scale that gives
/// it in the window's own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LaidOut {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) scale: f64,
}

impl LaidOut {
    /// Whether the content laid out at this size fills `client`, the window's client size in its
    /// own pixels; none where the size has no area, as XAML gives it before the first layout.
    fn fills(self, client: (i32, i32)) -> Option<bool> {
        let (width, height) = (self.width * self.scale, self.height * self.scale);
        let near = |laid: f64, side: i32| (laid - f64::from(side)).abs() <= f64::from(SLACK);
        (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0)
            .then(|| near(width, client.0) && near(height, client.1))
    }
}

/// Which part of the content was found not filling the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Found {
    ChildWindow,
    Layout,
}

impl Found {
    fn told(self, mends: u32, fills: bool) -> Unfilled {
        match self {
            Self::ChildWindow => Unfilled::ChildWindow { mends, fills },
            Self::Layout => Unfilled::Layout { mends, fills },
        }
    }
}

/// How the window is mended, the lighter first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mend {
    /// Windows' size message, sent again with the window's present client size and state.
    Size,
    /// The window restored and maximised again.
    Restore,
}

/// How far the checking of the window has come since it first stood.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Sequence {
    mends: u32,
    waits: u32,
    /// What the first check that found the content wrong found.
    found: Option<Found>,
}

impl Sequence {
    /// What the program is told where the checking ends with nothing more judged.
    fn left(self) -> Option<Unfilled> {
        self.found.map(|found| found.told(self.mends, false))
    }
}

/// What one check of the rule comes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// No laid-out size has been given yet: checked again without a mend.
    Wait,
    /// The window is mended so, and checked again.
    Mend(Mend),
    /// The checking ends, and the program is told this, where anything.
    End(Option<Unfilled>),
}

/// What the host does after a check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Checked {
    /// The content is checked again after [`BETWEEN`].
    Again,
    /// The checking has ended, and the program is told this, where anything.
    Ended(Option<Unfilled>),
}

/// Whether `size` is within [`SLACK`] of `client` on each side.
fn within(size: (i32, i32), client: (i32, i32)) -> bool {
    (size.0 - client.0).abs() <= SLACK && (size.1 - client.1).abs() <= SLACK
}

/// Decides one check from `client`, the window's client size, `child`, the client size of the
/// content's child window where there is one, both in the window's pixels, `laid`, the last
/// laid-out size given, and `maximised`, whether Windows holds the window maximised, and keeps in
/// `sequence` what the checking has come to.
///
/// The child window is judged first, and the laid-out size only where the child fills the window.
/// What the first wrong check found is what is told, whatever a later check finds.
fn judge(
    client: (i32, i32),
    child: Option<(i32, i32)>,
    laid: Option<LaidOut>,
    maximised: bool,
    sequence: &mut Sequence,
) -> Step {
    // A window with no child of the content's class has nothing of the content to judge.
    let Some(child) = child else {
        return Step::End(sequence.left());
    };
    let wrong = if within(child, client) {
        match laid.and_then(|laid| laid.fills(client)) {
            Some(false) => Some(Found::Layout),
            None if sequence.waits < WAITS => {
                sequence.waits += 1;
                return Step::Wait;
            }
            // No laid-out size came, and nothing was found wrong before: the child window alone
            // cannot say that the content fills the window.
            None if sequence.found.is_none() => return Step::End(Some(Unfilled::Unread)),
            // No laid-out size came after a mend: the layer cannot say that the content fills the
            // window now, and does not mend further what it cannot read.
            None => return Step::End(sequence.left()),
            Some(true) => None,
        }
    } else {
        Some(Found::ChildWindow)
    };
    let Some(wrong) = wrong else {
        return Step::End(sequence.found.map(|found| found.told(sequence.mends, true)));
    };
    let found = *sequence.found.get_or_insert(wrong);
    if sequence.mends < MENDS {
        let mend = if sequence.mends > 0 && maximised {
            Mend::Restore
        } else {
            Mend::Size
        };
        sequence.mends += 1;
        Step::Mend(mend)
    } else {
        Step::End(Some(found.told(sequence.mends, false)))
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

type Listed = unsafe extern "system" fn(*mut c_void, isize) -> i32;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetClientRect(window: *mut c_void, rect: *mut Rect) -> i32;
    fn EnumChildWindows(parent: *mut c_void, listed: Listed, held: isize) -> i32;
    fn GetClassNameW(window: *mut c_void, name: *mut u16, room: i32) -> i32;
    fn IsIconic(window: *mut c_void) -> i32;
    fn IsZoomed(window: *mut c_void) -> i32;
    fn SendMessageW(window: *mut c_void, message: u32, first: usize, second: isize) -> isize;
    fn ShowWindow(window: *mut c_void, show: i32) -> i32;
}

/// Windows' message that tells a window its client area took a new size.
const SIZE: u32 = 0x0005;
/// The state that message gives for a window Windows holds maximised.
const MAXIMISED: usize = 2;
/// The state it gives for a window at its own size.
const RESTORED: usize = 0;
/// The command that has Windows restore a window from maximised.
const SHOW_RESTORE: i32 = 9;
/// The command that has Windows maximise a window.
const SHOW_MAXIMISED: i32 = 3;

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

/// Checks the content of the window that stands on this thread against `laid`, the last
/// laid-out size given, keeps in `sequence` what the checking has come to, and mends the window
/// where the content does not fill it. A start whose content fills the window at the first check
/// gets reads alone.
///
/// A window minimised meanwhile has no client area to fill, and the checking ends there.
pub(crate) fn check(sequence: &mut Sequence, laid: Option<LaidOut>) -> Checked {
    let Some(window) = beside::window() else {
        return Checked::Ended(sequence.left());
    };
    // SAFETY: the call reads the window's state alone; a handle that is no longer a window's
    // reads as not minimised, and the read of its client area below is then refused.
    if unsafe { IsIconic(window) } != 0 {
        return Checked::Ended(sequence.left());
    }
    let Some(size) = client(window) else {
        return Checked::Ended(sequence.left());
    };
    // SAFETY: the call reads the window's state alone, and a handle that is no longer a window's
    // reads as not maximised.
    let maximised = unsafe { IsZoomed(window) } != 0;
    match judge(
        size,
        content(window).and_then(client),
        laid,
        maximised,
        sequence,
    ) {
        Step::Wait => Checked::Again,
        Step::Mend(Mend::Size) => {
            resize(window, size, maximised);
            Checked::Again
        }
        Step::Mend(Mend::Restore) => {
            // SAFETY: `window` is the handle of the window that stands on this thread, read from
            // `beside` on this thread, so each command is carried out here before its call
            // returns and nothing of either call is kept. Their answers say how the window stood
            // before, which the next check reads for itself.
            unsafe {
                ShowWindow(window, SHOW_RESTORE);
                ShowWindow(window, SHOW_MAXIMISED);
            }
            Checked::Again
        }
        Step::End(told) => Checked::Ended(told),
    }
}

/// Has `window` take its size again: Windows' size message is sent once more, with `size`, the
/// window's present client size, and its present state, and the content is laid out to it as at
/// any change of size.
fn resize(window: *mut c_void, (width, height): (i32, i32), maximised: bool) {
    // A client area too large for the message's two halves cannot be told in it.
    let (Ok(width), Ok(height)) = (u16::try_from(width), u16::try_from(height)) else {
        return;
    };
    let Ok(size) = isize::try_from((u32::from(height) << 16) | u32::from(width)) else {
        return;
    };
    let state = if maximised { MAXIMISED } else { RESTORED };
    // SAFETY: `window` is the handle of the window that stands on this thread, read from
    // `beside` on this thread, so the message is handled here before the call returns and
    // nothing of the call is kept. Its answer says nothing.
    unsafe { SendMessageW(window, SIZE, state, size) };
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIENT: (i32, i32) = (1920, 1009);
    /// The window's restored size, at which the content stands in a wrong start.
    const RESTORED_SIZE: (i32, i32) = (1264, 633);

    fn laid(width: f64, height: f64, scale: f64) -> LaidOut {
        LaidOut {
            width,
            height,
            scale,
        }
    }

    /// A layout that fills `CLIENT` at a scale of one.
    const FILLING: LaidOut = LaidOut {
        width: 1920.0,
        height: 1009.0,
        scale: 1.0,
    };

    /// A layout at the restored size, at a scale of one.
    const SMALL: LaidOut = LaidOut {
        width: 1264.0,
        height: 633.0,
        scale: 1.0,
    };

    /// The sequence after `mends` mends of a child window found wrong.
    fn after(mends: u32) -> Sequence {
        Sequence {
            mends,
            waits: 0,
            found: (mends > 0).then_some(Found::ChildWindow),
        }
    }

    #[test]
    fn content_as_large_as_the_client_area_stands_and_is_not_mended() {
        let step = judge(
            CLIENT,
            Some(CLIENT),
            Some(FILLING),
            true,
            &mut Sequence::default(),
        );
        assert_eq!(step, Step::End(None));
    }

    #[test]
    fn content_within_two_pixels_either_way_stands() {
        for child in [(1918, 1007), (1922, 1011), (1918, 1011), (1922, 1007)] {
            let step = judge(
                CLIENT,
                Some(child),
                Some(FILLING),
                true,
                &mut Sequence::default(),
            );
            assert_eq!(step, Step::End(None), "{child:?}");
        }
    }

    #[test]
    fn content_three_pixels_off_on_either_side_is_mended() {
        for child in [(1917, 1009), (1923, 1009), (1920, 1006), (1920, 1012)] {
            let step = judge(
                CLIENT,
                Some(child),
                Some(FILLING),
                true,
                &mut Sequence::default(),
            );
            assert_eq!(step, Step::Mend(Mend::Size), "{child:?}");
        }
    }

    #[test]
    fn content_at_the_restored_size_is_mended_until_the_last_mend_allowed() {
        for mends in 0..MENDS {
            let step = judge(
                CLIENT,
                Some(RESTORED_SIZE),
                Some(SMALL),
                true,
                &mut after(mends),
            );
            assert!(
                matches!(step, Step::Mend(_)),
                "after {mends} mends: {step:?}"
            );
        }
        let step = judge(
            CLIENT,
            Some(RESTORED_SIZE),
            Some(SMALL),
            true,
            &mut after(MENDS),
        );
        assert_eq!(
            step,
            Step::End(Some(Unfilled::ChildWindow {
                mends: MENDS,
                fills: false
            }))
        );
    }

    #[test]
    fn a_window_with_no_child_of_the_contents_class_stands() {
        let step = judge(CLIENT, None, Some(FILLING), true, &mut Sequence::default());
        assert_eq!(step, Step::End(None));
    }

    #[test]
    fn content_that_never_fills_the_window_is_mended_three_times_and_then_left() {
        let mut sequence = Sequence::default();
        loop {
            match judge(
                CLIENT,
                Some(RESTORED_SIZE),
                Some(SMALL),
                true,
                &mut sequence,
            ) {
                Step::Mend(_) => {
                    assert!(sequence.mends <= MENDS, "mended {} times", sequence.mends);
                }
                Step::End(_) => break,
                Step::Wait => panic!("a check with a laid-out size waited"),
            }
        }
        assert_eq!(sequence.mends, MENDS);
    }

    #[test]
    fn content_that_fills_the_window_after_a_mend_stands() {
        let step = judge(CLIENT, Some(CLIENT), Some(FILLING), true, &mut after(1));
        assert_eq!(
            step,
            Step::End(Some(Unfilled::ChildWindow {
                mends: 1,
                fills: true
            }))
        );
    }

    #[test]
    fn a_child_that_fills_the_window_with_a_layout_that_does_not_is_found_as_layout_and_mended() {
        let mut sequence = Sequence::default();
        let step = judge(CLIENT, Some(CLIENT), Some(SMALL), true, &mut sequence);
        assert_eq!(step, Step::Mend(Mend::Size));
        assert_eq!(sequence.found, Some(Found::Layout));
    }

    #[test]
    fn a_child_that_does_not_fill_the_window_is_found_as_child_window_whatever_the_layout_reads() {
        for layout in [Some(FILLING), Some(SMALL), None] {
            let mut sequence = Sequence::default();
            let step = judge(CLIENT, Some(RESTORED_SIZE), layout, true, &mut sequence);
            assert_eq!(step, Step::Mend(Mend::Size), "{layout:?}");
            assert_eq!(sequence.found, Some(Found::ChildWindow), "{layout:?}");
        }
    }

    #[test]
    fn a_child_and_a_layout_that_both_fill_the_window_stand_at_any_scale() {
        for (client, layout) in [
            (CLIENT, Some(FILLING)),
            ((2400, 1261), Some(laid(1920.0, 1009.0, 1.25))),
        ] {
            let step = judge(client, Some(client), layout, true, &mut Sequence::default());
            assert_eq!(step, Step::End(None), "{layout:?}");
        }
    }

    #[test]
    fn a_layout_within_two_pixels_stands_at_a_scale_of_one_and_of_one_and_a_quarter() {
        // Sizes in quarters of a pixel, which a scale of 1.25 multiplies exactly.
        let cases = [
            (CLIENT, Some(laid(1918.0, 1011.0, 1.0)), true),
            (CLIENT, Some(laid(1917.0, 1009.0, 1.0)), false),
            (CLIENT, Some(laid(1920.0, 1012.0, 1.0)), false),
            ((2400, 1261), Some(laid(1918.5, 1010.25, 1.25)), true),
            ((2400, 1261), Some(laid(1917.5, 1009.0, 1.25)), false),
            ((2400, 1261), Some(laid(1920.0, 1011.25, 1.25)), false),
        ];
        for (client, layout, stands) in cases {
            let step = judge(client, Some(client), layout, true, &mut Sequence::default());
            let expected = if stands {
                Step::End(None)
            } else {
                Step::Mend(Mend::Size)
            };
            assert_eq!(step, expected, "{client:?} {layout:?}");
        }
    }

    #[test]
    fn what_the_first_wrong_check_found_is_told_where_a_later_check_finds_the_other() {
        let mut sequence = Sequence::default();
        judge(
            CLIENT,
            Some(RESTORED_SIZE),
            Some(FILLING),
            true,
            &mut sequence,
        );
        judge(CLIENT, Some(CLIENT), Some(SMALL), true, &mut sequence);
        let step = judge(CLIENT, Some(CLIENT), Some(FILLING), true, &mut sequence);
        assert_eq!(
            step,
            Step::End(Some(Unfilled::ChildWindow {
                mends: 2,
                fills: true
            }))
        );

        let mut sequence = Sequence::default();
        judge(CLIENT, Some(CLIENT), Some(SMALL), true, &mut sequence);
        judge(
            CLIENT,
            Some(RESTORED_SIZE),
            Some(FILLING),
            true,
            &mut sequence,
        );
        let step = judge(CLIENT, Some(CLIENT), Some(FILLING), true, &mut sequence);
        assert_eq!(
            step,
            Step::End(Some(Unfilled::Layout {
                mends: 2,
                fills: true
            }))
        );
    }

    #[test]
    fn the_size_message_is_the_first_mend_and_the_restore_follows_only_for_a_maximised_window() {
        for (maximised, expected) in [
            (true, [Mend::Size, Mend::Restore, Mend::Restore]),
            (false, [Mend::Size, Mend::Size, Mend::Size]),
        ] {
            let mut sequence = Sequence::default();
            for (made, mend) in expected.into_iter().enumerate() {
                let step = judge(
                    CLIENT,
                    Some(RESTORED_SIZE),
                    Some(SMALL),
                    maximised,
                    &mut sequence,
                );
                assert_eq!(step, Step::Mend(mend), "mend {made}, maximised {maximised}");
            }
        }
    }

    #[test]
    fn a_check_with_no_laid_out_size_waits_and_mends_nothing() {
        for layout in [None, Some(laid(0.0, 0.0, 1.0))] {
            let mut sequence = Sequence::default();
            let step = judge(CLIENT, Some(CLIENT), layout, true, &mut sequence);
            assert_eq!(step, Step::Wait, "{layout:?}");
            assert_eq!((sequence.waits, sequence.mends), (1, 0), "{layout:?}");
        }
    }

    #[test]
    fn a_child_that_fills_the_window_with_no_laid_out_size_after_three_waits_is_told_unread() {
        let mut sequence = Sequence {
            waits: WAITS,
            ..Sequence::default()
        };
        let step = judge(CLIENT, Some(CLIENT), None, true, &mut sequence);
        assert_eq!(step, Step::End(Some(Unfilled::Unread)));
    }

    #[test]
    fn a_layout_never_given_is_waited_for_three_times_and_then_told_unread() {
        let mut sequence = Sequence::default();
        let told = loop {
            match judge(CLIENT, Some(CLIENT), None, true, &mut sequence) {
                Step::Wait => {
                    assert!(sequence.waits <= WAITS, "waited {} times", sequence.waits);
                }
                Step::End(told) => break told,
                Step::Mend(mend) => panic!("a window with no laid-out size was mended: {mend:?}"),
            }
        };
        assert_eq!((told, sequence.waits), (Some(Unfilled::Unread), WAITS));
    }

    #[test]
    fn after_a_mend_a_layout_never_given_is_not_told_as_filling_and_is_mended_no_more() {
        for found in [Found::ChildWindow, Found::Layout] {
            let mut sequence = Sequence {
                mends: 1,
                waits: WAITS,
                found: Some(found),
            };
            let step = judge(CLIENT, Some(CLIENT), None, true, &mut sequence);
            assert_eq!(step, Step::End(Some(found.told(1, false))), "{found:?}");
            assert_eq!(sequence.mends, 1, "{found:?}");
        }
    }

    #[test]
    fn each_ending_tells_what_it_found_or_nothing() {
        let endings = [
            (Some(CLIENT), Some(FILLING), after(0), None),
            (
                Some(CLIENT),
                Some(FILLING),
                after(1),
                Some(Unfilled::ChildWindow {
                    mends: 1,
                    fills: true,
                }),
            ),
            (
                Some(RESTORED_SIZE),
                Some(SMALL),
                after(MENDS),
                Some(Unfilled::ChildWindow {
                    mends: MENDS,
                    fills: false,
                }),
            ),
            (
                Some(CLIENT),
                None,
                Sequence {
                    waits: WAITS,
                    ..Sequence::default()
                },
                Some(Unfilled::Unread),
            ),
        ];
        for (child, layout, mut sequence, told) in endings {
            let step = judge(CLIENT, child, layout, true, &mut sequence);
            assert_eq!(step, Step::End(told), "{child:?} {layout:?}");
        }
    }
}
