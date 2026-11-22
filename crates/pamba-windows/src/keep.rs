//! Where the keyboard stays when a pointer's press lands on what takes no keys.
//!
//! XAML keeps no keys for a press on bare ground or on words: the scrolling area it puts above
//! every window's content takes the keyboard for itself when the press is let go
//! (`microsoft-ui-xaml`, `ScrollViewer_Partial.cpp`, `OnPointerReleased`), and a person who
//! pressed beside a field finds her next keys gone nowhere she can see. XAML says, before the
//! keyboard leaves a control, what would take it and what moved it, and lets the leaving be
//! refused. This module decides from those facts alone; `xaml` reads them and refuses.

/// What moved the keyboard, as XAML says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Moved {
    /// A press of a mouse, a finger or a pen.
    Pointer,
    /// Tab, an arrow key or another key.
    Keys,
    /// A program's own ask or a screen reader's move.
    Program,
    /// The keyboard let go of with no move, as when the window loses the foreground.
    Released,
}

/// Where an element stands against the window's content: what the program drew.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Place {
    /// No element.
    Nothing,
    /// Above the content: what XAML puts around a window's content, which the program never drew.
    Holder,
    /// The content itself, the root of what the program drew.
    Content,
    /// Inside the content: a control the program drew.
    Drawn,
    /// Neither above nor inside it: a flyout, a menu or another popup, which XAML draws apart.
    Apart,
}

/// Whether the keyboard is kept where it stands: only where a pointer would move it from
/// something onto what holds the window's content. A key's move and a program's are never
/// refused, a press onto anything that is not above the content moves the keyboard there, and
/// where nothing held it, or only XAML's holder did, there is no place to keep.
pub(crate) fn keeps(moved: Moved, from: Place, to: Place) -> bool {
    moved == Moved::Pointer
        && to == Place::Holder
        && matches!(from, Place::Content | Place::Drawn | Place::Apart)
}

#[cfg(test)]
mod tests {
    use super::{Moved, Place, keeps};

    const MOVES: [Moved; 4] = [Moved::Pointer, Moved::Keys, Moved::Program, Moved::Released];
    const PLACES: [Place; 5] = [
        Place::Nothing,
        Place::Holder,
        Place::Content,
        Place::Drawn,
        Place::Apart,
    ];

    #[test]
    fn of_the_hundred_points_only_the_three_written_keep_the_keyboard() {
        let kept = [
            (Moved::Pointer, Place::Content, Place::Holder),
            (Moved::Pointer, Place::Drawn, Place::Holder),
            (Moved::Pointer, Place::Apart, Place::Holder),
        ];
        let mut points = 0;
        for moved in MOVES {
            for from in PLACES {
                for to in PLACES {
                    points += 1;
                    let given = kept.contains(&(moved, from, to));
                    assert_eq!(keeps(moved, from, to), given, "{moved:?} {from:?} {to:?}");
                }
            }
        }
        assert_eq!(points, 100);
    }

    #[test]
    fn a_press_beside_a_control_keeps_the_keyboard_on_it() {
        assert!(keeps(Moved::Pointer, Place::Drawn, Place::Holder));
    }

    #[test]
    fn a_key_never_has_its_move_refused() {
        for from in PLACES {
            for to in PLACES {
                assert!(!keeps(Moved::Keys, from, to), "{from:?} {to:?}");
            }
        }
    }

    #[test]
    fn a_program_or_a_screen_reader_never_has_its_move_refused() {
        for from in PLACES {
            for to in PLACES {
                assert!(!keeps(Moved::Program, from, to), "{from:?} {to:?}");
            }
        }
    }

    #[test]
    fn the_keyboard_let_go_with_the_window_is_not_held() {
        assert!(!keeps(Moved::Released, Place::Drawn, Place::Holder));
    }

    #[test]
    fn a_press_on_a_control_moves_the_keyboard_there() {
        for from in PLACES {
            assert!(!keeps(Moved::Pointer, from, Place::Drawn), "{from:?}");
        }
    }

    #[test]
    fn a_press_on_the_content_itself_moves_the_keyboard_there() {
        assert!(!keeps(Moved::Pointer, Place::Drawn, Place::Content));
    }

    #[test]
    fn a_press_into_a_flyout_moves_the_keyboard_there() {
        assert!(!keeps(Moved::Pointer, Place::Drawn, Place::Apart));
    }

    #[test]
    fn the_keyboard_in_a_flyout_stays_through_a_press_beside_it() {
        assert!(keeps(Moved::Pointer, Place::Apart, Place::Holder));
    }

    #[test]
    fn where_nothing_held_the_keyboard_nothing_is_kept() {
        assert!(!keeps(Moved::Pointer, Place::Nothing, Place::Holder));
    }

    #[test]
    fn where_only_the_holder_had_the_keyboard_nothing_is_kept() {
        assert!(!keeps(Moved::Pointer, Place::Holder, Place::Holder));
    }

    #[test]
    fn a_press_that_would_leave_the_keyboard_on_nothing_is_not_refused() {
        assert!(!keeps(Moved::Pointer, Place::Drawn, Place::Nothing));
    }
}
