//! Type is the interface: the Metro text styles.

use nsh_core::prelude::*;

use crate::theme::{FONT_FAMILY, LIGHT, SEMIBOLD, SEMILIGHT, palette, size, u};

/// A Metro text style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextStyle {
    /// `PhoneTextTitle1Style`: page titles, 56 light.
    Title1,
    /// `PhoneTextTitle2Style`: 36 light.
    Title2,
    /// `PhoneTextTitle3Style`: 27 semilight.
    Title3,
    /// `PhoneTextExtraLargeStyle`: 36 semilight.
    ExtraLarge,
    /// `PhoneTextLargeStyle`: 27 semilight.
    Large,
    /// `PhoneTextNormalStyle`: 17 normal.
    Normal,
    /// `PhoneTextSubtleStyle`: 17, subtle colour.
    Subtle,
    /// `PhoneTextAccentStyle`: 17 semibold, accent colour.
    Accent,
    /// `PhoneTextSmallStyle`: 14, subtle colour.
    Small,
    /// Application title above a page title: 14 semibold, upper-case.
    AppTitle,
    /// Group header in lists: 22 semibold accent.
    GroupHeader,
}

impl TextStyle {
    pub fn size(self) -> f32 {
        match self {
            TextStyle::Title1 => size::HUGE,
            TextStyle::Title2 | TextStyle::ExtraLarge => size::EXTRA_LARGE,
            TextStyle::Title3 | TextStyle::Large => size::LARGE,
            TextStyle::Normal | TextStyle::Subtle | TextStyle::Accent => size::NORMAL,
            TextStyle::Small | TextStyle::AppTitle => size::SMALL,
            TextStyle::GroupHeader => size::MEDIUM_LARGE,
        }
    }

    pub fn weight(self) -> FontWeight {
        match self {
            TextStyle::Title1 | TextStyle::Title2 => LIGHT,
            TextStyle::Title3 | TextStyle::ExtraLarge | TextStyle::Large => SEMILIGHT,
            TextStyle::Normal | TextStyle::Subtle | TextStyle::Small => FontWeight::NORMAL,
            TextStyle::Accent | TextStyle::AppTitle | TextStyle::GroupHeader => SEMIBOLD,
        }
    }

    /// Its colour now (tracked).
    pub fn color(self) -> Hsla {
        let p = palette();
        match self {
            TextStyle::Subtle | TextStyle::Small => p.subtle,
            TextStyle::Accent | TextStyle::GroupHeader => p.accent,
            _ => p.foreground,
        }
    }

    pub fn line_height(self) -> f32 {
        (self.size() * if self.size() > 30. { 1.1 } else { 1.3 }).round()
    }
}

/// `text(TextStyle::Large, "strains")`: a styled run of text. The content
/// may be a value or a closure (`move || format!(…)`).
pub struct Text {
    style: TextStyle,
    content: Reactive<SharedString>,
    color: Option<Reactive<AdaptiveColor>>,
    lowercase: bool,
    nowrap: bool,
}

pub fn text(style: TextStyle, content: impl IntoReactive<SharedString>) -> Text {
    Text { style, content: content.into_reactive(), color: None, lowercase: false, nowrap: false }
}

impl Text {
    pub fn color(mut self, color: impl IntoReactive<AdaptiveColor>) -> Self {
        self.color = Some(color.into_reactive());
        self
    }

    /// Metro set headings in lower case. Opt in per call.
    pub fn lowercase(mut self) -> Self {
        self.lowercase = true;
        self
    }

    /// One line, cut off with an ellipsis.
    pub fn truncate(mut self) -> Self {
        self.nowrap = true;
        self
    }
}

impl IntoElement for Text {
    fn into_element(self) -> AnyElement {
        let style = self.style;
        let lowercase = self.lowercase;
        let content = self.content;
        let shown = move || {
            let s = content.get();
            match (style, lowercase) {
                (TextStyle::AppTitle, _) => SharedString::from(s.to_uppercase()),
                (_, true) => SharedString::from(s.to_lowercase()),
                _ => s,
            }
        };
        let el = nsh_core::prelude::text(shown)
            .font_family(FONT_FAMILY)
            .text_size(u(style.size()))
            .font_weight(style.weight())
            .line_height(nsh_core::LineHeight::Points(px(style.line_height())));
        let el = match self.color {
            Some(c) => el.text_color(move || c.get()),
            None => el.text_color(move || style.color()),
        };
        if self.nowrap { el.truncate().into_element() } else { el.into_element() }
    }
}
