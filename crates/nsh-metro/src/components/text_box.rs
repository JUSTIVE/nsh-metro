//! The text box: a native text field in Metro dress. Idle it is a pale
//! filled box; focused it turns white with an accent border.
//!
//! gpui-metro had to write its own text input (1,000 lines: caret,
//! selection, IME, key bindings). nsh's `input()` is the platform's own
//! field, so this is only the styling.

use std::rc::Rc;

use nsh_core::{element::input, prelude::*};

use crate::components::text::label;

use crate::theme::{FONT_FAMILY, palette, size, u};

pub struct TextBox {
    value: Signal<String>,
    header: Option<SharedString>,
    placeholder: Option<SharedString>,
    masked: bool,
    on_submit: Option<Rc<dyn Fn(String)>>,
}

/// A text box editing `value`.
pub fn text_box(value: Signal<String>) -> TextBox {
    TextBox { value, header: None, placeholder: None, masked: false, on_submit: None }
}

impl TextBox {
    pub fn header(mut self, h: impl Into<SharedString>) -> Self {
        self.header = Some(h.into());
        self
    }
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = Some(p.into());
        self
    }
    /// A password box.
    pub fn masked(mut self) -> Self {
        self.masked = true;
        self
    }
    /// Return pressed, with the text.
    pub fn on_submit(mut self, f: impl Fn(String) + 'static) -> Self {
        self.on_submit = Some(Rc::new(f));
        self
    }
}

impl IntoElement for TextBox {
    fn into_element(self) -> AnyElement {
        let focused = signal(false);
        let value = self.value;
        let mut field = input()
            .value(move || value.get())
            .on_change(move |v: &str| value.set(v.to_string()))
            .on_focus(move || focused.set(true))
            .on_blur(move || focused.set(false))
            .flex_1()
            .font_family(FONT_FAMILY)
            .text_size(u(size::MEDIUM))
            .text_color(move || palette().input_ink)
            .caret_color(Hsla::from(rgb(0x000000)))
            .selection_color(move || palette().accent.opacity(0.45));
        if let Some(p) = self.placeholder {
            field = field.placeholder(p.to_string());
        }
        if self.masked {
            field = field.masked(true);
        }
        if let Some(f) = self.on_submit {
            field = field.on_submit(move |v: &str| f(v.to_string()));
        }
        view()
            .gap(u(4.))
            .children(self.header.map(|h| label(h).font_family(FONT_FAMILY).text_size(u(size::NORMAL)).text_color(move || palette().subtle)))
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .h(u(48.))
                    .px(u(10.))
                    .bg(move || if focused.get() { palette().input_focused } else { palette().input })
                    .border(px(2.))
                    .border_color(move || if focused.get() { palette().accent } else { palette().input })
                    .child(field),
            )
            .into_element()
    }
}
