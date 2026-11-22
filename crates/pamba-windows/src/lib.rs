//! Pamba's Windows layer: a `pamba::Program` drawn with WinUI 3.
//!
//! `run` opens the program's window and runs its loop on the window's thread. What is drawn is a
//! function of the program, written with the controls of `ui`; a control's event becomes a
//! message of the program's through `Messages`. What a program's effect asks of a control
//! reaches it by a name the drawing gives it with `named`: `focus` stands the keyboard there.
//! What arrives in answer stands in a place the drawing gives with `place`: `bring_into_view`
//! scrolls it into sight, and `announce` says the answer to a screen reader.
//! What WinUI's reactor does not reach, Windows itself gives beside it: `copy` puts text on the
//! clipboard. Other threads ask work of the window's thread through `WindowThread`, which runs it
//! there with the window's `WindowHandle`: `bring_forward` brings the window in front of the
//! others. Where WinUI itself cannot start, so that no window of the program can say so,
//! `tell` says it through Windows. A window whose content does not fill it when it first stands is
//! mended, and the program is told through `Window::on_unfilled`. Microsoft's
//! `windows-reactor` draws beneath this crate, and a program names this crate alone: a fault of
//! the layer beneath is mended here, once, for every program.
//!
//! The crate is empty on every other platform, so a workspace that holds a Windows program beside
//! its platform-free logic checks everywhere.

#![cfg(windows)]
#![deny(unsafe_code)]

mod appearance;
mod arrive;
mod beside;
mod clipboard;
mod contrast;
mod days;
mod fill;
mod host;
mod keep;
mod named;
mod tell;
mod threads;
pub mod ui;
mod unfilled;
mod watch;
mod window;
mod window_thread;
mod xaml;

pub use arrive::{announce, bring_into_view, place};
pub use beside::{Behind, WindowHandle};
pub use clipboard::{Uncopied, copy};
pub use host::{Fault, Messages, run};
pub use named::{focus, named};
pub use tell::tell;
pub use unfilled::Unfilled;
pub use window::Window;
pub use window_thread::{Unrun, WindowThread};
