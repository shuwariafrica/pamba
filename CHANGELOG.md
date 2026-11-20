# Changelog

<!-- One section per release, newest first, headed by the version exactly as its tag names it
after the `v`: the tag v0.1.0 is "## 0.1.0". The release takes the section's text, to the next
"## " heading, as its notes; a tag with no section of its own is released with notes that say
so. -->

## 0.4.0

Pamba is written in Rust. The .NET runtime of 0.1.0 to 0.3.0 is gone, and nothing of its API
carries over.

- `pamba`, the loop. A `Program` answers each message by changing its state and returning its side
  effects as data; a `Performer` carries them out on one platform; a `Loop` runs the two on the
  thread that draws. `Manual` runs a program and its performer in a test with no window and no
  second thread.
- `pamba-windows`, the Windows layer. `run` opens a program's window, drawn with WinUI 3 through
  Microsoft's `windows-reactor`, whose controls are all under `ui`. `Messages` turns what a control
  reports into a message of the program's, and `Messages::routed` lets a program take a key or a
  pointer for itself. `copy` puts text on the clipboard. `tell` says a word through Windows itself
  where WinUI could not start.
- `pamba-windows-build`, what the build script of a program on the Windows layer calls, so that
  the program starts on a computer with nothing of Microsoft's installed.
