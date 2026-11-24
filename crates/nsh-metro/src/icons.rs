//! Metro glyphs: geometric, single-weight, square caps. Embedded into the
//! binary as SVG markup and drawn as vectors by the platform, tinted by the
//! element's text colour (`currentColor`).

use nsh_core::{element::ImageSrc, prelude::*};

macro_rules! icons {
    ($($variant:ident => $file:literal),+ $(,)?) => {
        /// Every glyph shipped with nsh-metro.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Icon { $($variant),+ }

        impl Icon {
            pub const ALL: &'static [Icon] = &[$(Icon::$variant),+];

            pub fn name(self) -> &'static str {
                match self { $(Icon::$variant => $file),+ }
            }

            /// The glyph's SVG markup as shipped (white strokes).
            pub fn markup(self) -> &'static str {
                match self { $(Icon::$variant => include_str!(concat!("../assets/icons/", $file, ".svg"))),+ }
            }
        }
    };
}

icons! {
    Add => "add",
    Alert => "alert",
    Back => "back",
    Battery => "battery",
    Calendar => "calendar",
    Camera => "camera",
    Check => "check",
    ChevronDown => "chevron_down",
    ChevronRight => "chevron_right",
    Clock => "clock",
    Close => "close",
    Delete => "delete",
    Dna => "dna",
    Download => "download",
    Edit => "edit",
    Favorite => "favorite",
    Filter => "filter",
    Flask => "flask",
    Forward => "forward",
    Gauge => "gauge",
    Grid => "grid",
    Home => "home",
    List => "list",
    Lock => "lock",
    Mail => "mail",
    More => "more",
    Next => "next",
    Pause => "pause",
    People => "people",
    Phone => "phone",
    Pin => "pin",
    Play => "play",
    Previous => "previous",
    Refresh => "refresh",
    Save => "save",
    Search => "search",
    Settings => "settings",
    Share => "share",
    Signal => "signal",
    Snow => "snow",
    Sync => "sync",
    Thermometer => "thermometer",
    Truck => "truck",
    Upload => "upload",
    Wave => "wave",
    Wifi => "wifi",
}

impl Icon {
    /// The markup with its strokes and fills in `currentColor`, so the
    /// element's text colour tints it.
    pub fn tintable(self) -> String {
        self.markup().replace("#fff", "currentColor").replace("#FFF", "currentColor")
    }

    /// An `svg()` element for this glyph, `size` px square, tinted `color`
    /// (a value or a closure).
    pub fn render(self, size: f32, color: impl IntoReactive<AdaptiveColor>) -> AnyElement {
        svg(ImageSrc::svg_markup(&self.tintable())).size(px(size)).flex_shrink_0().text_color(color).into_element()
    }
}

/// Shorthand for [`Icon::render`].
pub fn icon(icon: Icon, size: f32, color: impl IntoReactive<AdaptiveColor>) -> AnyElement {
    icon.render(size, color)
}
