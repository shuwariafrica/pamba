//! What the layer tells a program of a window whose content it did not find filling the window.

/// What the layer tells a program, once, where it did not find the window's content filling the
/// window when the window first stood. A program writes it in its own log or counts it: a
/// `ChildWindow` or a `Layout` is a start a person would have seen wrong had the layer not acted.
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
