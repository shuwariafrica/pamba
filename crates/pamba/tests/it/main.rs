//! The loop's rules, each held by a case over one small program whose performer writes down
//! everything it is asked.

#![allow(
    clippy::unwrap_used,
    reason = "a case fails by panicking, in its helpers and its performer as in its body"
)]

mod bound;
mod order;
mod places;
mod probe;
mod subscriptions;
