//! The loop a native program runs on.
//!
//! A program is written as a [`Program`]: state that answers each message by changing and by
//! returning the side effects it wants, as data, and that says what it stays subscribed to. It
//! names no platform. A [`Performer`] carries the effects out on one platform, and a [`Loop`]
//! runs the two together under that platform's window: one message at a time, each step's effects
//! performed before the next message is taken, nothing delivered from a subscription once it is
//! stopped.
//!
//! Because effects are values, a program's every rule is a function call and a comparison, with
//! no window, thread or clock. A program with its real performer runs the same way under
//! [`Manual`].
//!
//! The Windows layer, which draws a program with WinUI 3, is the crate `pamba-windows`.

#![forbid(unsafe_code)]

mod mailbox;
mod manual;
mod performer;
mod program;
mod run;

pub use mailbox::{Dispatcher, Sender, Unsent};
pub use manual::{Manual, Unsettled};
pub use performer::{Performer, Running};
pub use program::{Never, Program};
pub use run::{Loop, ROOM, Turn};
