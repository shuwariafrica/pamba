use windows_reactor::{Callback, View, WindowPlacement, WindowSize, WindowVisuals};

use crate::appearance::Appearance;

/// One window as a program's present state has it: what it holds, and what it says of itself.
#[derive(Debug)]
pub struct Window {
    pub(crate) content: View,
    pub(crate) title: Option<String>,
    pub(crate) visuals: Option<WindowVisuals>,
    pub(crate) sized: Option<Callback<WindowSize>>,
    pub(crate) appeared: Option<Callback<Appearance>>,
    pub(crate) placed: Option<Callback<WindowPlacement>>,
}

impl Window {
    /// A window that holds `content`.
    pub fn new(content: impl Into<View>) -> Self {
        Self {
            content: content.into(),
            title: None,
            visuals: None,
            sized: None,
            appeared: None,
            placed: None,
        }
    }

    /// What the title bar, the taskbar and a screen reader call the window.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The window's backdrop, theme, icon, the place and size it first takes and the sizes it may
    /// take.
    #[must_use]
    pub fn visuals(mut self, visuals: WindowVisuals) -> Self {
        self.visuals = Some(visuals);
        self
    }

    /// Tells the program the size of the window's inside, in device-independent pixels, at the
    /// start and whenever it changes.
    #[must_use]
    pub fn on_size(mut self, sized: Callback<WindowSize>) -> Self {
        self.sized = Some(sized);
        self
    }

    /// Tells the program where the window stands on the screen and how large it is, in the
    /// screen's own pixels, at the start and whenever she moves or sizes it: what a program keeps
    /// to open its window where she left it.
    #[must_use]
    pub fn on_placement(mut self, placed: Callback<WindowPlacement>) -> Self {
        self.placed = Some(placed);
        self
    }

    /// Tells the program how Windows draws now - in the person's light colours, in her dark
    /// ones, or by her contrast theme - at the start and whenever it changes. A program keeps
    /// what it is told and resolves a [`FillColor`](crate::ui::FillColor) with it.
    #[must_use]
    pub fn on_appearance(mut self, appeared: Callback<Appearance>) -> Self {
        self.appeared = Some(appeared);
        self
    }
}
