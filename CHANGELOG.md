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
  second thread. `Places` keeps, for a platform's layer, the places a program names from one
  drawing to the next.
- `pamba-windows`, the Windows layer. `run` opens a program's window, drawn with WinUI 3 through
  Microsoft's `windows-reactor`, whose controls are all under `ui`. `Messages` turns what a control
  reports into a message of the program's, and `Messages::routed` lets a program take a key or a
  pointer for itself. `named` gives a control a name of the program's, and `focus` stands the
  keyboard at it once the drawing after the step stands. `copy` puts text on the clipboard. `tell`
  says a word through Windows itself where WinUI could not start.
- `pamba-windows-build`, what the build script of a program on the Windows layer calls, so that
  the program starts on a Windows computer that has no Windows App Runtime installed. It stages
  the runtime beside the executable, and takes each of its two downloads only where its SHA-256
  is the one this version holds.
