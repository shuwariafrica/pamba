//! What a window is drawn with: WinUI 3's controls, layout and brushes, as values.
//!
//! Everything `windows-reactor` offers is here under its own name, so no capability of WinUI's
//! that it reaches is withheld. A name this module defines itself takes the place of the one
//! beneath: that is where a fault of the layer beneath is mended.
//!
//! Three names are this crate's own. `windows-reactor` names eight of Windows' theme brushes
//! and tells a window its colour mode alone. [`FillColor`] names the colours of Windows' theme
//! it does not, and [`Appearance`], with [`Contrast`], is how Windows draws now, by which one
//! is resolved.

pub use windows_reactor::*;

pub use crate::appearance::{Appearance, FillColor};
pub use crate::contrast::Contrast;
