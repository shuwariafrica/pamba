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
  drawing to the next, and retries an ask the platform cannot carry out yet. `Sender::wrapping`
  gives a part of a program a sender for its own messages, made from the whole's by a function.
- `pamba-host`, what a program asks of the computer it runs on, with no window and no drawing, on
  every platform. `session::switch_user` says once whether the computer offers to leave the screen
  to another person: the act, `SwitchUser`, whose `now` leaves the screen and answers `Stayed`
  where the platform refused, or `Unoffered`, which says why not. On Windows it is offered where the
  administrator has not hidden switching and another person can sign in: more than one enabled
  account, or a computer joined to a domain or to Entra.
- `pamba-windows`, the Windows layer. `run` opens a program's window, drawn with WinUI 3 through
  Microsoft's `windows-reactor`, whose controls are all under `ui`. A moment after the window first
  stands, the layer checks that its content fills it, and where it does not, has Windows tell the
  window its size again. The check reads the size the content is laid out at as well as the size
  of its child window, and `Window::on_unfilled` tells the program what it found and mended, as
  an `Unfilled` that says itself in one plain sentence for the program's log.
  `Messages` turns what a control
  reports into a message of the program's, and `Messages::routed` lets a program take a key or a
  pointer for itself. A pointer's press moves the keyboard only onto something that takes it:
  a press on the window's ground, on words or on its title band leaves the keyboard where it stood.
  `named` gives a control a name of the program's, and `focus` stands the
  keyboard at it once the drawing after the step stands, asking again for a control Windows shows a
  moment later, such as a field in an expander the person has just opened. `place` gives part of
  the drawing a name of the program's as the place where an answer arrives: `bring_into_view`
  moves every scrolling area it stands in as little as shows it, once the drawing after the step
  stands, and `announce` says words to a screen reader without moving the keyboard. `copy` puts
  text on the clipboard. `WindowThread` lets any thread ask work of the window's thread, which runs
  it there with the window's `WindowHandle` once the window stands, and refuses with `Unrun` once
  the window has gone. `WindowHandle::bring_forward` restores the window where it was minimised and
  asks Windows for the foreground, and answers `Behind` where Windows keeps it behind. `tell` says a
  word through Windows itself where WinUI could not start.
  `Window::on_appearance` tells a program how Windows draws now, the person's colour mode and her
  contrast theme, and `ui::FillColor` names thirteen colours of Windows' theme that
  `windows-reactor` does not, each resolved for that appearance by `Appearance::brush`.
  `Appearance::accent` gives the accent as the ground of a filled mark together with the words that
  stand on it, which under a contrast theme are the theme's own pair for what is selected.
  `ui::CalendarDay` is a day of the calendar as a program holds one, and `ui::DayPicker` has
  Windows' date picker hold that day and tell the day a person picks, the same day in whichever
  time zone the computer is in. `CalendarDay::new` says why three figures are no day (`NoDay`), and
  `ui::Days` holds the days a picker offers with the day it holds among them, refusing a day that is
  not offered (`NotOffered`): Windows would show another day in its place.
- `pamba-windows-build`, what the build script of a program on the Windows layer calls, so that
  the program starts on a Windows computer that has no Windows App Runtime installed. It stages
  the runtime beside the executable, with the licence texts of its two packages in `licenses`,
  and takes each of its two downloads only where its SHA-256 is the one this version holds. A
  build at an earlier revision of this rewrite kept the packages in
  `%LOCALAPPDATA%\pamba\windows-app-runtime\`, which can be deleted once no program on the
  machine builds at such a revision.
