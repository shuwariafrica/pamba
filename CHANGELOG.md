# Changelog

What each release changes for a program built on Pamba. The format is that of Keep a Changelog, and versions follow
Semantic Versioning.

## Unreleased

### Changed

- Pamba is written in Rust. The .NET runtime of 0.1.0 to 0.3.0 is gone, and nothing of its API carries over.

### Added

- `pamba`: the loop. A `Program` answers each message by changing its state and returning its side effects as data; a
  `Performer` carries them out on one platform; a `Loop` runs the two on the thread that draws. `Manual` runs a
  program and its performer in a test with no window and no second thread.
- `pamba-windows`: the Windows layer. `run` opens a program's window, drawn with WinUI 3.
- `pamba-windows-build`: what the build script of a program on the Windows layer calls.
