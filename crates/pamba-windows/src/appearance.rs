//! How Windows draws now, and the colours of its theme the layer beneath does not name.
//!
//! `windows-reactor` names eight of Windows' theme brushes as [`ThemeBrush`], and XAML resolves
//! each for the person's colour mode and her contrast theme. Windows' theme holds more: its
//! signal colours for what is well and for caution, the lesser strengths of its text, the steps
//! of a window's ground. [`FillColor`] names those by Windows' own names, and
//! [`Appearance::brush`] resolves one for how Windows draws now. A program is told the
//! appearance through `Window::on_appearance`, at the start and whenever it changes, keeps it,
//! and draws with it.
//!
//! The colours are Windows' own, as WinUI's theme resources state them
//! (`microsoft-ui-xaml`, `Common_themeresources_any.xaml`): each member's line names its
//! resource there.

use windows_reactor::{Brush, Color, ColorScheme, ThemeBrush};

use crate::contrast::{self, Contrast};

/// How Windows draws now: in the person's light colours, in her dark ones, or by her contrast
/// theme.
///
/// A contrast theme states every colour itself, so under one there is no light or dark mode to
/// speak of: the layer beneath reports the light mode there whatever the theme's ground is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Appearance {
    /// Her colours are light, and no contrast theme is on.
    Light,
    /// Her colours are dark, and no contrast theme is on.
    Dark,
    /// Her contrast theme is on.
    Contrast(Contrast),
}

/// A colour of Windows' theme that [`ThemeBrush`] does not name, by Windows' own name for it:
/// `SystemSuccess` is the resource `SystemFillColorSuccess`, `TextSecondary` is
/// `TextFillColorSecondary`, `SolidBackgroundSecondary` is `SolidBackgroundFillColorSecondary`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FillColor {
    /// What is well, as a mark, a word or an edge.
    SystemSuccess,
    /// What asks for care, as a mark, a word or an edge.
    SystemCaution,
    /// What is neither, as a mark or an edge that lets the ground through.
    SystemNeutral,
    /// What is neither, as a mark or an edge that covers the ground.
    SystemSolidNeutral,
    /// The ground beneath what is well.
    SystemSuccessBackground,
    /// The ground beneath what asks for care.
    SystemCautionBackground,
    /// The ground beneath what is neither, which lets the window through.
    SystemNeutralBackground,
    /// The ground beneath what is neither, which covers the window.
    SystemSolidNeutralBackground,
    /// Words read second.
    TextSecondary,
    /// Words read third.
    TextTertiary,
    /// Words that are not active.
    TextDisabled,
    /// The window's ground, one step back.
    SolidBackgroundSecondary,
    /// The window's ground, one step forward.
    SolidBackgroundTertiary,
}

/// What a colour is for, which decides what a contrast theme puts in its place.
#[derive(Clone, Copy)]
enum Role {
    /// A mark, a word or an edge: the theme's colour for words.
    Words,
    /// A ground: the theme's colour for a window.
    Ground,
    /// Words that are not active: the theme's own colour for them.
    Inactive,
}

/// One colour as Windows states it for each mode.
#[derive(Clone, Copy)]
struct Stated {
    light: Color,
    dark: Color,
    role: Role,
}

impl FillColor {
    /// Every colour named here, for a program that draws or checks them all.
    pub const ALL: [Self; 13] = [
        Self::SystemSuccess,
        Self::SystemCaution,
        Self::SystemNeutral,
        Self::SystemSolidNeutral,
        Self::SystemSuccessBackground,
        Self::SystemCautionBackground,
        Self::SystemNeutralBackground,
        Self::SystemSolidNeutralBackground,
        Self::TextSecondary,
        Self::TextTertiary,
        Self::TextDisabled,
        Self::SolidBackgroundSecondary,
        Self::SolidBackgroundTertiary,
    ];

    /// The colour as Windows states it.
    const fn stated(self) -> Stated {
        let (light, dark, role) = match self {
            Self::SystemSuccess => (
                Color::rgb(0x0F, 0x7B, 0x0F),
                Color::rgb(0x6C, 0xCB, 0x5F),
                Role::Words,
            ),
            Self::SystemCaution => (
                Color::rgb(0x9D, 0x5D, 0x00),
                Color::rgb(0xFC, 0xE1, 0x00),
                Role::Words,
            ),
            Self::SystemNeutral => (
                Color::argb(0x72, 0x00, 0x00, 0x00),
                Color::argb(0x8B, 0xFF, 0xFF, 0xFF),
                Role::Words,
            ),
            Self::SystemSolidNeutral => (
                Color::rgb(0x8A, 0x8A, 0x8A),
                Color::rgb(0x9D, 0x9D, 0x9D),
                Role::Words,
            ),
            Self::SystemSuccessBackground => (
                Color::rgb(0xDF, 0xF6, 0xDD),
                Color::rgb(0x39, 0x3D, 0x1B),
                Role::Ground,
            ),
            Self::SystemCautionBackground => (
                Color::rgb(0xFF, 0xF4, 0xCE),
                Color::rgb(0x43, 0x35, 0x19),
                Role::Ground,
            ),
            Self::SystemNeutralBackground => (
                Color::argb(0x06, 0x00, 0x00, 0x00),
                Color::argb(0x08, 0xFF, 0xFF, 0xFF),
                Role::Ground,
            ),
            Self::SystemSolidNeutralBackground => (
                Color::rgb(0xF3, 0xF3, 0xF3),
                Color::rgb(0x2E, 0x2E, 0x2E),
                Role::Ground,
            ),
            Self::TextSecondary => (
                Color::argb(0x9E, 0x00, 0x00, 0x00),
                Color::argb(0xC5, 0xFF, 0xFF, 0xFF),
                Role::Words,
            ),
            Self::TextTertiary => (
                Color::argb(0x72, 0x00, 0x00, 0x00),
                Color::argb(0x87, 0xFF, 0xFF, 0xFF),
                Role::Words,
            ),
            Self::TextDisabled => (
                Color::argb(0x5C, 0x00, 0x00, 0x00),
                Color::argb(0x5D, 0xFF, 0xFF, 0xFF),
                Role::Inactive,
            ),
            Self::SolidBackgroundSecondary => (
                Color::rgb(0xEE, 0xEE, 0xEE),
                Color::rgb(0x1C, 0x1C, 0x1C),
                Role::Ground,
            ),
            Self::SolidBackgroundTertiary => (
                Color::rgb(0xF9, 0xF9, 0xF9),
                Color::rgb(0x28, 0x28, 0x28),
                Role::Ground,
            ),
        };
        Stated { light, dark, role }
    }
}

impl Appearance {
    /// How Windows draws now, where the reactor last reported `mode`.
    pub(crate) fn now(mode: ColorScheme) -> Self {
        match (contrast::read(), mode) {
            (Some(theme), _) => Self::Contrast(theme),
            (None, ColorScheme::Light) => Self::Light,
            (None, ColorScheme::Dark) => Self::Dark,
        }
    }

    /// The brush that draws `colour` now: Windows' own colour for the mode, and under a contrast
    /// theme what Windows itself puts in that colour's place.
    #[must_use]
    pub const fn brush(self, colour: FillColor) -> Brush {
        let stated = colour.stated();
        match (self, stated.role) {
            (Self::Light, _) => Brush::Solid(stated.light),
            (Self::Dark, _) => Brush::Solid(stated.dark),
            (Self::Contrast(_), Role::Words) => Brush::Theme(ThemeBrush::PrimaryText),
            (Self::Contrast(_), Role::Ground) => Brush::Theme(ThemeBrush::SolidBackground),
            (Self::Contrast(theme), Role::Inactive) => Brush::Solid(theme.inactive),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two appearances in which Windows' own colours for a mode are drawn.
    const MODES: [Appearance; 2] = [Appearance::Light, Appearance::Dark];

    /// A contrast theme whose colour for inactive words is no colour Windows states for a mode.
    const THEMED: Appearance = Appearance::Contrast(Contrast {
        inactive: Color::rgb(0x3F, 0xF2, 0x3F),
    });

    /// The window's own ground in each mode, as Windows states `SolidBackgroundFillColorBase`.
    const fn window(mode: Appearance) -> Color {
        match mode {
            Appearance::Light => Color::rgb(0xF3, 0xF3, 0xF3),
            Appearance::Dark => Color::rgb(0x20, 0x20, 0x20),
            Appearance::Contrast(_) => unreachable!(),
        }
    }

    /// `ink` as it stands once laid over `ground`, which is opaque.
    fn over(ink: Color, ground: Color) -> [f64; 3] {
        let strength = f64::from(ink.a) / 255.0;
        let lay = |above: u8, beneath: u8| {
            f64::from(above) * strength + f64::from(beneath) * (1.0 - strength)
        };
        [
            lay(ink.r, ground.r),
            lay(ink.g, ground.g),
            lay(ink.b, ground.b),
        ]
    }

    /// How bright a colour is, as WCAG 2 reckons it.
    fn luminance(colour: [f64; 3]) -> f64 {
        let linear = |channel: f64| {
            let part = channel / 255.0;
            if part <= 0.04045 {
                part / 12.92
            } else {
                ((part + 0.055) / 1.055).powf(2.4)
            }
        };
        let [red, green, blue] = colour;
        0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue)
    }

    /// How strongly `ink` stands on `ground`, to one.
    fn stands(ink: Color, ground: Color) -> f64 {
        let ink = luminance(over(ink, ground));
        let ground = luminance(over(ground, ground));
        (ink.max(ground) + 0.05) / (ink.min(ground) + 0.05)
    }

    #[test]
    fn under_a_contrast_theme_no_colour_of_a_mode_is_drawn() {
        let Appearance::Contrast(theme) = THEMED else {
            unreachable!()
        };
        for colour in FillColor::ALL {
            let brush = THEMED.brush(colour);
            let stated = colour.stated();
            assert!(
                brush != Brush::Solid(stated.light) && brush != Brush::Solid(stated.dark),
                "{colour:?} keeps a mode's colour under a contrast theme"
            );
            assert!(
                matches!(
                    brush,
                    Brush::Theme(ThemeBrush::PrimaryText | ThemeBrush::SolidBackground)
                ) || brush == Brush::Solid(theme.inactive),
                "{colour:?} is drawn in {brush:?}, which the theme does not hold"
            );
        }
    }

    #[test]
    fn with_no_contrast_theme_each_colour_is_solid_and_differs_between_the_modes() {
        for colour in FillColor::ALL {
            let light = Appearance::Light.brush(colour);
            let dark = Appearance::Dark.brush(colour);
            assert!(
                matches!(light, Brush::Solid(_)),
                "{colour:?} is {light:?} in the light mode"
            );
            assert!(
                matches!(dark, Brush::Solid(_)),
                "{colour:?} is {dark:?} in the dark mode"
            );
            assert_ne!(light, dark, "{colour:?} is one colour in both modes");
        }
    }

    #[test]
    fn inactive_words_take_the_contrast_themes_own_colour_for_them() {
        let Appearance::Contrast(theme) = THEMED else {
            unreachable!()
        };
        assert_eq!(
            THEMED.brush(FillColor::TextDisabled),
            Brush::Solid(theme.inactive)
        );
    }

    #[test]
    fn each_colour_read_as_words_or_as_a_mark_keeps_its_floor_on_the_window() {
        // Four and a half to one for what is read as words; three to one for what Windows itself
        // gives a lesser strength, which is the floor of a mark.
        let floors = [
            (FillColor::SystemSuccess, 4.5),
            (FillColor::SystemCaution, 4.5),
            (FillColor::TextSecondary, 4.5),
            (FillColor::SystemSolidNeutral, 3.0),
            (FillColor::TextTertiary, 3.0),
        ];
        for mode in MODES {
            for (colour, floor) in floors {
                let Brush::Solid(ink) = mode.brush(colour) else {
                    unreachable!("with no contrast theme a colour is solid")
                };
                let stands = stands(ink, window(mode));
                assert!(
                    stands >= floor,
                    "{colour:?} in {mode:?} stands {stands:.2} to 1, under {floor}"
                );
            }
        }
    }
}
