//! A contrast theme that is on, read through Windows itself.

#![allow(
    unsafe_code,
    reason = "a contrast theme is read through Windows' C interface, and nothing safe beneath this crate tells of one"
)]

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{from_mut, null_mut};

use windows_reactor::Color;

/// A contrast theme that is on, by what of it no brush beneath names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Contrast {
    /// The theme's colour for words that are not active.
    pub inactive: Color,
}

/// Windows' question for how the contrast theme stands (`SPI_GETHIGHCONTRAST`).
const HOW_CONTRAST_STANDS: u32 = 66;
/// The flag that says a contrast theme is on (`HCF_HIGHCONTRASTON`).
const ON: u32 = 1;
/// Windows' index for the colour of words that are not active (`COLOR_GRAYTEXT`).
const INACTIVE_WORDS: i32 = 17;

/// What Windows fills in when asked how the contrast theme stands (`HIGHCONTRASTW`).
#[repr(C)]
struct Standing {
    size: u32,
    flags: u32,
    scheme: *mut u16,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn SystemParametersInfoW(action: u32, parameter: u32, data: *mut c_void, update: u32) -> i32;
    fn GetSysColor(index: i32) -> u32;
}

/// The contrast theme that is on, where one is. Where Windows does not answer, none is taken to
/// be on.
pub(crate) fn read() -> Option<Contrast> {
    let size = u32::try_from(size_of::<Standing>()).ok()?;
    let mut standing = Standing {
        size,
        flags: 0,
        scheme: null_mut(),
    };
    // SAFETY: `standing` is a `HIGHCONTRASTW` that states its own size and lives through the
    // call; Windows writes within it alone. The name it points `scheme` at is Windows' own and
    // is never read here.
    let answered = unsafe {
        SystemParametersInfoW(HOW_CONTRAST_STANDS, size, from_mut(&mut standing).cast(), 0)
    };
    if answered == 0 || standing.flags & ON == 0 {
        return None;
    }
    // SAFETY: `GetSysColor` takes an index and returns a colour; it reads and writes no memory
    // of this program's.
    let inactive = unsafe { GetSysColor(INACTIVE_WORDS) };
    // Windows states a colour as red in its lowest byte, then green, then blue.
    let [red, green, blue, _] = inactive.to_le_bytes();
    Some(Contrast {
        inactive: Color::rgb(red, green, blue),
    })
}
