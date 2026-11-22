//! What the layer tells a program of a window whose content it did not find filling the window.

use std::fmt;

/// What the layer tells a program, once, where it did not find the window's content filling the
/// window when the window first stood. A program writes it in its own log or counts it: a
/// `ChildWindow` or a `Layout` is a start a person would have seen wrong had the layer not acted.
///
/// `Display` gives the line for a log: one sentence for whoever looks after the computer, which
/// names nothing of this crate's, starts in lower case and ends with no full stop, so that a
/// program can set its own words before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unfilled {
    /// The content's own child window did not fill the window.
    ChildWindow {
        /// How many times the layer mended the window, at most three.
        mends: u32,
        /// Whether the layer found the content filling the window at its last check: its child
        /// window, and what is laid out inside it. False also where the laid-out size was not
        /// given at that check, since the child window alone cannot say it.
        fills: bool,
    },
    /// The content's child window filled the window, and what is laid out inside it did not.
    Layout {
        /// How many times the layer mended the window, at most three.
        mends: u32,
        /// Whether the layer found the content filling the window at its last check, as
        /// [`ChildWindow`](Self::ChildWindow)'s says.
        fills: bool,
    },
    /// The child window filled the window, and the size the content is laid out at was never
    /// given, so the layer cannot say that the content fills the window. It mended nothing.
    Unread,
}

impl fmt::Display for Unfilled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (found, mends, fills) = match *self {
            Self::ChildWindow { mends, fills } => (
                "the area that holds the window's content stood smaller than the window",
                mends,
                fills,
            ),
            Self::Layout { mends, fills } => (
                "the window's content was laid out smaller than the window",
                mends,
                fills,
            ),
            Self::Unread => {
                return formatter.write_str(
                    "the size the window's content is laid out at was never given, so whether \
                     the content fills the window cannot be said; the window was not mended",
                );
            }
        };
        let times = if mends == 1 { "time" } else { "times" };
        let after = if fills {
            "the content was then found to fill it"
        } else {
            "the content was not then found to fill it"
        };
        write!(
            formatter,
            "{found}; the window was mended {mends} {times}, and {after}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every value the layer tells: each case, each count it can reach and each answer of `fills`.
    fn every() -> Vec<Unfilled> {
        let mut every = vec![Unfilled::Unread];
        for mends in 0..=3 {
            for fills in [false, true] {
                every.push(Unfilled::ChildWindow { mends, fills });
                every.push(Unfilled::Layout { mends, fills });
            }
        }
        every
    }

    #[test]
    fn a_child_window_found_smaller_is_said_as_the_area_that_holds_the_content() {
        assert_eq!(
            Unfilled::ChildWindow {
                mends: 1,
                fills: true
            }
            .to_string(),
            "the area that holds the window's content stood smaller than the window; the window \
             was mended 1 time, and the content was then found to fill it"
        );
    }

    #[test]
    fn a_layout_found_smaller_is_said_as_the_content_laid_out_smaller() {
        assert_eq!(
            Unfilled::Layout {
                mends: 2,
                fills: false
            }
            .to_string(),
            "the window's content was laid out smaller than the window; the window was mended 2 \
             times, and the content was not then found to fill it"
        );
    }

    #[test]
    fn an_unread_size_is_said_as_unknowable_and_unmended() {
        assert_eq!(
            Unfilled::Unread.to_string(),
            "the size the window's content is laid out at was never given, so whether the content \
             fills the window cannot be said; the window was not mended"
        );
    }

    #[test]
    fn content_found_filling_after_the_mends_is_said_to_fill() {
        for told in [
            Unfilled::ChildWindow {
                mends: 3,
                fills: true,
            },
            Unfilled::Layout {
                mends: 3,
                fills: true,
            },
        ] {
            let said = told.to_string();
            assert!(
                said.ends_with(", and the content was then found to fill it"),
                "{told:?}: {said}"
            );
        }
    }

    #[test]
    fn content_not_found_filling_is_said_not_to_fill() {
        for told in [
            Unfilled::ChildWindow {
                mends: 3,
                fills: false,
            },
            Unfilled::Layout {
                mends: 3,
                fills: false,
            },
        ] {
            let said = told.to_string();
            assert!(
                said.ends_with(", and the content was not then found to fill it"),
                "{told:?}: {said}"
            );
        }
    }

    #[test]
    fn one_mend_is_said_as_one_time() {
        for fills in [false, true] {
            for told in [
                Unfilled::ChildWindow { mends: 1, fills },
                Unfilled::Layout { mends: 1, fills },
            ] {
                let said = told.to_string();
                assert!(
                    said.contains("; the window was mended 1 time, and "),
                    "{told:?}: {said}"
                );
            }
        }
    }

    #[test]
    fn more_than_one_mend_is_said_as_times() {
        for mends in [2, 3] {
            for told in [
                Unfilled::ChildWindow { mends, fills: true },
                Unfilled::Layout { mends, fills: true },
            ] {
                let said = told.to_string();
                assert!(
                    said.contains(&format!("; the window was mended {mends} times, and ")),
                    "{told:?}: {said}"
                );
            }
        }
    }

    #[test]
    fn no_sentence_names_the_crate_or_opens_with_a_capital_or_ends_with_a_full_stop() {
        for told in every() {
            let said = told.to_string();
            for name in [
                "ChildWindow",
                "Layout",
                "Unread",
                "mends:",
                "fills:",
                "px",
                "{",
            ] {
                assert!(!said.contains(name), "{told:?} says {name:?}: {said}");
            }
            assert!(
                said.starts_with(|first: char| first.is_ascii_lowercase()),
                "{told:?}: {said}"
            );
            assert!(!said.ends_with('.'), "{told:?}: {said}");
        }
    }
}
