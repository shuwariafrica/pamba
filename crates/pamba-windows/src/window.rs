use windows_reactor::{Callback, ColorScheme, View, WindowPlacement, WindowSize, WindowVisuals};

/// One window as a program's present state has it: what it holds, and what it says of itself.
#[derive(Debug)]
pub struct Window {
    pub(crate) content: View,
    pub(crate) title: Option<String>,
    pub(crate) visuals: Option<WindowVisuals>,
    pub(crate) sized: Option<Callback<WindowSize>>,
    pub(crate) scheme: Option<Callback<ColorScheme>>,
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
            scheme: None,
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

    /// Tells the program whether the person's colours are light or dark, at the start and
    /// whenever they change.
    #[must_use]
    pub fn on_color_scheme(mut self, scheme: Callback<ColorScheme>) -> Self {
        self.scheme = Some(scheme);
        self
    }
}
