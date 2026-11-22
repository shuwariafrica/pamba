//! Whether this computer offers to leave the screen to another person, and the act itself.
//!
//! With no argument it writes one line of what `pamba_host::session::switch_user` gave: the act,
//! with the three facts it was offered on, or why it is not offered.
//!
//! ```text
//! offered: SwitchUser { hidden: no, accounts: 2, joined: no }
//! not offered: nobody else can sign in to this computer (NobodyElse)
//! ```
//!
//! With the argument `now` it then acts where the act is offered, and writes what `now` gave. The
//! session leaves the screen a moment after `now: done` is written.
//!
//! ```text
//! now: done
//! now: stayed: the platform did not leave the screen to another person: ...
//! ```
//!
//! It names no account. It exits 0 where it ran, and 2 where it was given another argument.

use std::env;
use std::process::ExitCode;

use pamba_host::session;

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    let acts = match (arguments.next(), arguments.next()) {
        (None, None) => false,
        (Some(argument), None) if argument == "now" => true,
        _ => {
            eprintln!("usage: switch [now]");
            return ExitCode::from(2);
        }
    };
    match session::switch_user() {
        Ok(switch) => {
            println!("offered: {switch:?}");
            if acts {
                match switch.now() {
                    Ok(()) => println!("now: done"),
                    Err(stayed) => println!("now: stayed: {stayed} ({stayed:?})"),
                }
            }
        }
        Err(unoffered) => {
            println!("not offered: {unoffered} ({unoffered:?})");
            if acts {
                println!("now: not asked, since the act is not offered");
            }
        }
    }
    ExitCode::SUCCESS
}
