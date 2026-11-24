//! The Metro theme: one ground (dark or light), one accent, one typeface.
//!
//! Windows Phone let the user pick exactly two things, background and
//! accent, and every app was expected to follow. The theme is a reactive
//! global of the surface: read it with [`palette`] inside a closure
//! (`.bg(move || palette().accent)`) and a change re-skins only the
//! properties that read it, natively, without rebuilding anything.

use nsh_core::prelude::*;

/// Dark or light background, the first of the phone's two theme choices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    /// Black ground, white type: the phone's default.
    Dark,
    /// White ground, black type.
    Light,
}

impl Mode {
    /// The other mode.
    pub fn toggled(self) -> Self {
        match self {
            Mode::Dark => Mode::Light,
            Mode::Light => Mode::Dark,
        }
    }

    /// The mode's lower-case name, as the phone's settings showed it (`"dark"`, `"light"`).
    pub fn label(self) -> &'static str {
        match self {
            Mode::Dark => "dark",
            Mode::Light => "light",
        }
    }
}

/// The twenty accent colours of Windows Phone 8.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Accent {
    /// Lime, `#A4C400`.
    Lime,
    /// Green, `#60A917`.
    Green,
    /// Emerald, `#008A00`.
    Emerald,
    /// Teal, `#00ABA9`.
    Teal,
    /// Cyan, `#1BA1E2`.
    Cyan,
    #[default]
    /// Cobalt, `#0050EF`: the default.
    Cobalt,
    /// Indigo, `#6A00FF`.
    Indigo,
    /// Violet, `#AA00FF`.
    Violet,
    /// Pink, `#F472D0`.
    Pink,
    /// Magenta, `#D80073`.
    Magenta,
    /// Crimson, `#A20025`.
    Crimson,
    /// Red, `#E51400`.
    Red,
    /// Orange, `#FA6800`.
    Orange,
    /// Amber, `#F0A30A`.
    Amber,
    /// Yellow, `#E3C800`.
    Yellow,
    /// Brown, `#825A2C`.
    Brown,
    /// Olive, `#6D8764`.
    Olive,
    /// Steel, `#647687`.
    Steel,
    /// Mauve, `#76608A`.
    Mauve,
    /// Taupe, `#87794E`.
    Taupe,
}

impl Accent {
    /// Every accent, in the order of the phone's accent picker.
    pub const ALL: [Accent; 20] = [
        Accent::Lime,
        Accent::Green,
        Accent::Emerald,
        Accent::Teal,
        Accent::Cyan,
        Accent::Cobalt,
        Accent::Indigo,
        Accent::Violet,
        Accent::Pink,
        Accent::Magenta,
        Accent::Crimson,
        Accent::Red,
        Accent::Orange,
        Accent::Amber,
        Accent::Yellow,
        Accent::Brown,
        Accent::Olive,
        Accent::Steel,
        Accent::Mauve,
        Accent::Taupe,
    ];

    /// The accent as `0xRRGGBB`.
    pub fn hex(self) -> u32 {
        match self {
            Accent::Lime => 0xA4C400,
            Accent::Green => 0x60A917,
            Accent::Emerald => 0x008A00,
            Accent::Teal => 0x00ABA9,
            Accent::Cyan => 0x1BA1E2,
            Accent::Cobalt => 0x0050EF,
            Accent::Indigo => 0x6A00FF,
            Accent::Violet => 0xAA00FF,
            Accent::Pink => 0xF472D0,
            Accent::Magenta => 0xD80073,
            Accent::Crimson => 0xA20025,
            Accent::Red => 0xE51400,
            Accent::Orange => 0xFA6800,
            Accent::Amber => 0xF0A30A,
            Accent::Yellow => 0xE3C800,
            Accent::Brown => 0x825A2C,
            Accent::Olive => 0x6D8764,
            Accent::Steel => 0x647687,
            Accent::Mauve => 0x76608A,
            Accent::Taupe => 0x87794E,
        }
    }

    /// The accent as a colour.
    pub fn color(self) -> Hsla {
        rgb(self.hex()).into()
    }

    /// The accent's lower-case name (`"cobalt"`), as the phone's picker listed it.
    pub fn name(self) -> &'static str {
        match self {
            Accent::Lime => "lime",
            Accent::Green => "green",
            Accent::Emerald => "emerald",
            Accent::Teal => "teal",
            Accent::Cyan => "cyan",
            Accent::Cobalt => "cobalt",
            Accent::Indigo => "indigo",
            Accent::Violet => "violet",
            Accent::Pink => "pink",
            Accent::Magenta => "magenta",
            Accent::Crimson => "crimson",
            Accent::Red => "red",
            Accent::Orange => "orange",
            Accent::Amber => "amber",
            Accent::Yellow => "yellow",
            Accent::Brown => "brown",
            Accent::Olive => "olive",
            Accent::Steel => "steel",
            Accent::Mauve => "mauve",
            Accent::Taupe => "taupe",
        }
    }

    /// Text that sits on a filled accent. Metro always used white, even on
    /// yellow and lime; we keep that for fidelity.
    pub fn ink(self) -> Hsla {
        rgb(0xFFFFFF).into()
    }
}

/// The resolved colour tokens for a mode + accent pair. Names follow the
/// Windows Phone resource keys (`PhoneForegroundBrush`, `PhoneSubtleBrush`, …).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// The ground: black in dark mode, white in light (`PhoneBackgroundBrush`).
    pub background: Hsla,
    /// Primary text and glyphs: the opposite of the ground (`PhoneForegroundBrush`).
    pub foreground: Hsla,
    /// Secondary text: dates, captions, inactive pivot headers.
    pub subtle: Hsla,
    /// Disabled text and borders.
    pub disabled: Hsla,
    /// App bar, message box, context menu: the "chrome" layer.
    pub chrome: Hsla,
    /// Control borders (buttons, text boxes, check boxes).
    pub border: Hsla,
    /// Text box / list picker fill when idle.
    pub input: Hsla,
    /// Text box fill when focused (always the opposite of the ground).
    pub input_focused: Hsla,
    /// Ink inside a focused text box.
    pub input_ink: Hsla,
    /// Dim layer behind a message box.
    pub scrim: Hsla,
    /// The accent itself (`PhoneAccentBrush`).
    pub accent: Hsla,
    /// Text and glyphs on a filled accent (always white; see [`Accent::ink`]).
    pub accent_ink: Hsla,
}

fn alpha(hex: u32, a: f32) -> Hsla {
    let c: Hsla = rgb(hex).into();
    c.opacity(a)
}

impl Palette {
    /// The tokens for `mode` with `accent`.
    pub fn resolve(mode: Mode, accent: Accent) -> Self {
        let (accent, accent_ink) = (accent.color(), accent.ink());
        match mode {
            Mode::Dark => Palette {
                background: rgb(0x000000).into(),
                foreground: rgb(0xFFFFFF).into(),
                subtle: alpha(0xFFFFFF, 0.6),
                disabled: alpha(0xFFFFFF, 0.4),
                chrome: rgb(0x1F1F1F).into(),
                border: alpha(0xFFFFFF, 0.75),
                input: alpha(0xFFFFFF, 0.75),
                input_focused: rgb(0xFFFFFF).into(),
                input_ink: rgb(0x000000).into(),
                scrim: alpha(0x000000, 0.6),
                accent,
                accent_ink,
            },
            Mode::Light => Palette {
                background: rgb(0xFFFFFF).into(),
                foreground: rgb(0x000000).into(),
                subtle: alpha(0x000000, 0.4),
                disabled: alpha(0x000000, 0.26),
                chrome: rgb(0xDDDDDD).into(),
                border: alpha(0x000000, 0.6),
                input: alpha(0x000000, 0.15),
                input_focused: rgb(0xFFFFFF).into(),
                input_ink: rgb(0x000000).into(),
                scrim: alpha(0xFFFFFF, 0.6),
                accent,
                accent_ink,
            },
        }
    }
}

/// A length in Metro design px. nsh lays out in points, which is what the
/// design px were, so this is `px`; it keeps call sites the same as in
/// gpui-metro.
pub fn u(design_px: f32) -> Length {
    px(design_px).into()
}

/// The Metro type ramp (WP8 `PhoneFontSize*` resources), in px.
pub mod size {
    /// `PhoneFontSizeSmall`
    pub const SMALL: f32 = 14.;
    /// `PhoneFontSizeNormal`
    pub const NORMAL: f32 = 17.;
    /// `PhoneFontSizeMedium`
    pub const MEDIUM: f32 = 20.;
    /// `PhoneFontSizeMediumLarge`
    pub const MEDIUM_LARGE: f32 = 22.;
    /// `PhoneFontSizeLarge`
    pub const LARGE: f32 = 27.;
    /// `PhoneFontSizeExtraLarge`: pivot headers.
    pub const EXTRA_LARGE: f32 = 36.;
    /// `PhoneFontSizeExtraExtraLarge`: page titles.
    pub const HUGE: f32 = 56.;
    /// Panorama title.
    pub const PANORAMA: f32 = 112.;
}

/// Metro is set in Segoe WP; the closest face on macOS that still carries
/// light weights. An app with a face of its own puts it in its place with
/// [`set_font_family`]; components read it through this, so call sites keep
/// `.font_family(FONT_FAMILY)`.
pub const FONT_FAMILY: fn() -> SharedString = font_family;

/// The default face, before an app sets its own.
pub const DEFAULT_FONT_FAMILY: &str = "Helvetica Neue";

thread_local! {
    static FAMILY: std::cell::RefCell<SharedString> = std::cell::RefCell::new(SharedString::from(DEFAULT_FONT_FAMILY));
}

/// The face every component is set in.
pub fn font_family() -> SharedString {
    FAMILY.with(|f| f.borrow().clone())
}

/// Sets the face every component is set in (registered with the platform
/// first), as gpui-metro's `Theme::font_family` did. Call it before the
/// first component is built: a component reads it when it is built.
pub fn set_font_family(family: impl Into<SharedString>) {
    FAMILY.with(|f| *f.borrow_mut() = family.into());
}

/// Weight of the big Metro type (page titles, pivot headers).
pub const LIGHT: FontWeight = FontWeight::LIGHT;
/// `PhoneFontFamilySemiLight`.
pub const SEMILIGHT: FontWeight = FontWeight::NORMAL;
/// `PhoneFontFamilySemiBold`.
pub const SEMIBOLD: FontWeight = FontWeight::SEMIBOLD;

/// The surface's theme. Change it with [`Theme::set_mode`] /
/// [`Theme::set_accent`]; everything bound to [`palette`] follows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    /// Dark or light ground.
    pub mode: Mode,
    /// The accent colour.
    pub accent: Accent,
    /// When true, components skip their entrance/transition motion.
    pub reduced_motion: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self { mode: Mode::Dark, accent: Accent::Cobalt, reduced_motion: false }
    }
}

impl Theme {
    /// The theme now (tracked: a closure reading it re-runs on a change).
    pub fn get() -> Theme {
        global::<Theme>().get()
    }

    /// Switches the ground; everything bound to [`palette`] re-skins.
    pub fn set_mode(mode: Mode) {
        global::<Theme>().update(|t| t.mode = mode);
    }

    /// Switches the accent; everything bound to [`palette`] re-skins.
    pub fn set_accent(accent: Accent) {
        global::<Theme>().update(|t| t.accent = accent);
    }

    /// Turns motion down (`true`) or back up. Components read it when they are built,
    /// so it applies to what is built after the change.
    pub fn set_reduced_motion(reduced: bool) {
        global::<Theme>().update(|t| t.reduced_motion = reduced);
    }

    /// This theme's colour tokens.
    pub fn palette(&self) -> Palette {
        Palette::resolve(self.mode, self.accent)
    }
}

/// The palette now (tracked).
pub fn palette() -> Palette {
    Theme::get().palette()
}

/// Whether motion is turned down, read once (untracked): components decide
/// at build time whether to animate.
pub fn reduced_motion() -> bool {
    global::<Theme>().get_untracked().reduced_motion
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_accent_has_a_unique_colour_and_name() {
        let mut hexes: Vec<u32> = Accent::ALL.iter().map(|a| a.hex()).collect();
        hexes.sort_unstable();
        hexes.dedup();
        assert_eq!(hexes.len(), 20);
        let mut names: Vec<&str> = Accent::ALL.iter().map(|a| a.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 20);
    }

    #[test]
    fn grounds_are_inverted() {
        let d = Palette::resolve(Mode::Dark, Accent::Red);
        let l = Palette::resolve(Mode::Light, Accent::Red);
        assert_eq!(d.background, l.foreground);
        assert_eq!(d.foreground, l.background);
        assert_eq!(d.accent, l.accent);
    }
}
