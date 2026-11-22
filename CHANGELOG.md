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
  drawing to the next, and retries an ask the platform cannot carry out yet.
- `pamba-windows`, the Windows layer. `run` opens a program's window, drawn with WinUI 3 through
  Microsoft's `windows-reactor`, whose controls are all under `ui`. `Messages` turns what a control
  reports into a message of the program's, and `Messages::routed` lets a program take a key or a
  pointer for itself. `named` gives a control a name of the program's, and `focus` stands the
  keyboard at it once the drawing after the step stands, asking again for a control Windows shows a
  moment later, such as a field in an expander the person has just opened. `copy` puts text on the
  clipboard. `tell` says a word through Windows itself where WinUI could not start.
  `Window::on_appearance` tells a program how Windows draws now, the person's colour mode and her
  contrast theme, and `ui::FillColor` names thirteen colours of Windows' theme that
  `windows-reactor` does not, each resolved for that appearance by `Appearance::brush`.
  `Appearance::accent` gives the accent as the ground of a filled mark together with the words that
  stand on it, which under a contrast theme are the theme's own pair for what is selected.
  `ui::CalendarDay` is a day of the calendar as a program holds one, and `ui::DayPicker` has
  Windows' date picker hold that day and tell the day a person picks, the same day in whichever
  time zone the computer is in.
- `pamba-windows-build`, what the build script of a program on the Windows layer calls, so that
  the program starts on a Windows computer that has no Windows App Runtime installed. It stages
  the runtime beside the executable, with the licence texts of its two packages in `licenses`,
  and takes each of its two downloads only where its SHA-256 is the one this version holds. A
  build at an earlier revision of this rewrite kept the packages in
  `%LOCALAPPDATA%\pamba\windows-app-runtime\`, which can be deleted once no program on the
  machine builds at such a revision.
