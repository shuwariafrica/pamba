//! What a native program asks of the computer it runs on, beside the window it draws.
//!
//! A service here needs no window and no drawing library, so a program with no window takes it as
//! well as one drawn on a platform's layer. Each is called with no `cfg` of the program's: the
//! crate builds on every platform, and each platform's side answers by that platform's own means.
//!
//! [`session`] holds what concerns the session the program runs in: leaving the screen to another
//! person, where the computer has someone to leave it to.

#![deny(unsafe_code)]

pub mod session;
