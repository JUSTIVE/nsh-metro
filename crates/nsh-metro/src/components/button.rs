//! Buttons: the bordered push button, the round app-bar button and the
//! hyperlink. Pressed, a button fills with the accent, as on the phone.

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::{
    Icon, motion,
    theme::{FONT_FAMILY, palette, size, u},
};

/// What a click runs.
pub type ClickHandler = Rc<dyn Fn()>;

/// `button("save").on_click(|| …)`
pub struct Button {
    label: SharedString,
    icon: Option<Icon>,
    on_click: Option<ClickHandler>,
    disabled: bool,
    accent: bool,
    full_width: bool,
}

pub fn button(label: impl Into<SharedString>) -> Button {
    Button { label: label.into(), icon: None, on_click: None, disabled: false, accent: false, full_width: false }
}

impl Button {
    pub fn on_click(mut self, f: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Filled with the accent: the primary action.
    pub fn accent(mut self) -> Self {
        self.accent = true;
        self
    }
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
}

impl IntoElement for Button {
    fn into_element(self) -> AnyElement {
        let disabled = self.disabled;
        let accent = self.accent && !disabled;
        let fg = move || {
            let p = palette();
            if disabled {
                p.disabled
            } else if accent {
                p.accent_ink
            } else {
                p.foreground
            }
        };
        let mut el = view()
            .flex_row()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .gap(u(8.))
            .min_w(u(108.))
            .h(u(52.))
            .px(u(20.))
            .border(px(3.))
            .border_color(move || {
                let p = palette();
                if accent { p.accent } else if disabled { p.disabled } else { p.foreground }
            })
            .bg(move || if accent { palette().accent } else { palette().accent.opacity(0.) })
            .when(self.full_width, |el| el.w_full())
            .when(!disabled, |el| el.cursor_pointer());
        if !disabled {
            el = el.active(|s| s.bg(palette().accent).border_color(palette().accent));
        }
        if let Some(icon) = self.icon {
            el = el.child(icon.render(20., fg));
        }
        el = el.child(
            text(self.label)
                .font_family(FONT_FAMILY)
                .text_size(u(size::MEDIUM))
                .text_color(fg),
        );
        if let (Some(f), false) = (self.on_click, disabled) {
            el = el.on_press(move || f());
        }
        el.into_element()
    }
}

/// The round icon button of the app bar and media controls.
pub struct RoundButton {
    icon: Icon,
    diameter: f32,
    on_click: Option<ClickHandler>,
    disabled: bool,
    filled: bool,
}

pub fn round_button(icon: Icon) -> RoundButton {
    RoundButton { icon, diameter: 48., on_click: None, disabled: false, filled: false }
}

impl RoundButton {
    pub fn on_click(mut self, f: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
    pub fn diameter(mut self, d: f32) -> Self {
        self.diameter = d;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }
}

impl IntoElement for RoundButton {
    fn into_element(self) -> AnyElement {
        let d = self.diameter;
        let (disabled, filled) = (self.disabled, self.filled);
        let color = move || if disabled { palette().disabled } else { palette().foreground };
        let ring = view()
            .items_center()
            .justify_center()
            .size(u(d))
            .rounded(px(d / 2.))
            .border(px(2.))
            .border_color(color)
            .bg(move || if filled { color() } else { color().opacity(0.) })
            .child(self.icon.render(d * 0.5, move || if filled { palette().background } else { color() }));
        if disabled {
            return ring.into_element();
        }
        let ring = ring.active(|s| s.bg(palette().accent).border_color(palette().accent)).cursor_pointer();
        let ring = match self.on_click {
            Some(f) => ring.on_press(move || f()),
            None => ring,
        };
        motion::tilt(ring).into_element()
    }
}

/// An underlined text link.
pub struct HyperlinkButton {
    label: SharedString,
    on_click: Option<ClickHandler>,
}

pub fn hyperlink(label: impl Into<SharedString>) -> HyperlinkButton {
    HyperlinkButton { label: label.into(), on_click: None }
}

impl HyperlinkButton {
    pub fn on_click(mut self, f: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl IntoElement for HyperlinkButton {
    fn into_element(self) -> AnyElement {
        let el = view().cursor_pointer().child(
            text(self.label)
                .font_family(FONT_FAMILY)
                .text_size(u(size::MEDIUM))
                .underline()
                .text_color(move || palette().foreground),
        );
        let el = el.active(|s| s.opacity(0.6));
        match self.on_click {
            Some(f) => el.on_press(move || f()).into_element(),
            None => el.into_element(),
        }
    }
}
