//! Pamba's Windows layer: a `pamba::Program` drawn with WinUI 3.
//!
//! `run` opens the program's window and runs its loop on the window's thread. What is drawn is a
//! function of the program, written with the controls of `ui`; a control's event becomes a
//! message of the program's through `Messages`. What a program's effect asks of a control
//! reaches it by a name the drawing gives it with `named`: `focus` stands the keyboard there.
//! What WinUI's reactor does not reach, Windows itself gives beside it: `copy` puts text on the
//! clipboard. Where WinUI itself cannot start, so that no window of the program can say so,
//! `tell` says it through Windows. Microsoft's
//! `windows-reactor` draws beneath this crate, and a program names this crate alone: a fault of
//! the layer beneath is mended here, once, for every program.
//!
//! The crate is empty on every other platform, so a workspace that holds a Windows program beside
//! its platform-free logic checks everywhere.

#![cfg(windows)]
#![deny(unsafe_code)]

mod beside;
mod clipboard;
mod host;
mod named;
mod tell;
mod threads;
pub mod ui;
mod window;

pub use clipboard::{Uncopied, copy};
pub use host::{Fault, Messages, run};
pub use named::{focus, named};
pub use tell::tell;
pub use window::Window;
