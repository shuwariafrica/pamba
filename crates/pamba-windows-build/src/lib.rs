//! What the build script of a program on Pamba's Windows layer calls.
//!
//! ```no_run
//! // build.rs
//! pamba_windows_build::self_contained();
//! ```

#![forbid(unsafe_code)]

use std::env;

/// Stages the Windows App Runtime beside the program's executable and marks the executable to
/// load it from there, so the program starts on a machine with nothing of Microsoft's installed.
///
/// It does nothing where the target is not Windows, so a program's build script calls it on
/// every platform its workspace is checked on. It does nothing either where the machine that
/// builds is not Windows, which cannot stage the runtime: there the Windows target is checked,
/// and a program built there starts only where the runtime is installed.
///
/// # Panics
///
/// Where Cargo's build-script environment is missing, or the target's toolchain is neither
/// MSVC nor LLVM's GNU.
pub fn self_contained() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") || !cfg!(windows) {
        return;
    }
    windows_reactor_setup::as_self_contained();
}
