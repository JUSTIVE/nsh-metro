//! The message box (a chrome band across a dimmed page, swivelling in) and
//! the toast (an accent band at the top).

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::components::text::label;

use crate::{
    Icon,
    button::{ClickHandler, button},
    chrome::PAGE_MARGIN,
    motion::swivel_in,
    theme::{FONT_FAMILY, SEMIBOLD, palette, size, u},
};

/// Show it with `when(move || open.get(), move || message_box(…))`.
pub struct MessageBox {
    title: SharedString,
    message: Option<SharedString>,
    content: Option<AnyElement>,
    buttons: Vec<(SharedString, ClickHandler)>,
    on_dismiss: Option<ClickHandler>,
}

/// A message box with `title` in large semibold type.
pub fn message_box(title: impl Into<SharedString>) -> MessageBox {
    MessageBox { title: title.into(), message: None, content: None, buttons: Vec::new(), on_dismiss: None }
}

impl MessageBox {
    /// The body text under the title.
    pub fn message(mut self, m: impl Into<SharedString>) -> Self {
        self.message = Some(m.into());
        self
    }
    /// Custom content under the message, above the buttons.
    pub fn child(mut self, e: impl IntoElement) -> Self {
        self.content = Some(e.into_element());
        self
    }
    /// Adds a button to the bottom row; the buttons share the row equally.
    /// A tap runs `f` (close the box from it).
    pub fn button(mut self, label: impl Into<SharedString>, f: impl Fn() + 'static) -> Self {
        self.buttons.push((label.into(), Rc::new(f)));
        self
    }
    /// A tap on the dimmed page around the band.
    pub fn on_dismiss(mut self, f: impl Fn() + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl IntoElement for MessageBox {
    fn into_element(self) -> AnyElement {
        let fg = move || palette().foreground;
        let buttons = self
            .buttons
            .into_iter()
            .map(|(label, f)| view().flex_1().child(button(label).full_width().on_click(move || f())));
        let band = view()
            .w_full()
            .bg(move || palette().chrome)
            .px(u(PAGE_MARGIN))
            .pt(u(40.))
            .pb(u(20.))
            .gap(u(14.))
            // Taps on the band stay on the band.
            .on_press(|| {})
            .child(
                label(self.title)
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::LARGE))
                    .font_weight(SEMIBOLD)
                    .text_color(fg),
            )
            .children(self.message.map(|m| label(m).font_family(FONT_FAMILY).text_size(u(size::MEDIUM)).text_color(fg)))
            .children(self.content)
            .child(view().flex_row().gap(u(12.)).pt(u(8.)).children(buttons));
        let scrim = view().absolute().inset_0().bg(move || palette().scrim).child(swivel_in(band));
        match self.on_dismiss {
            Some(f) => scrim.on_press(move || f()).into_element(),
            None => scrim.on_press(|| {}).into_element(),
        }
    }
}

/// A toast notification: an accent band across the top.
pub struct Toast {
    title: SharedString,
    message: Option<SharedString>,
    icon: Option<Icon>,
    on_click: Option<ClickHandler>,
}

/// A toast with `title` in semibold, on the accent.
pub fn toast(title: impl Into<SharedString>) -> Toast {
    Toast { title: title.into(), message: None, icon: None, on_click: None }
}

impl Toast {
    /// Text after the title, on the same line, cut off with an ellipsis.
    pub fn message(mut self, m: impl Into<SharedString>) -> Self {
        self.message = Some(m.into());
        self
    }
    /// A 22 px glyph before the title.
    pub fn icon(mut self, i: Icon) -> Self {
        self.icon = Some(i);
        self
    }
    /// Runs `f` on a tap on the band.
    pub fn on_click(mut self, f: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl IntoElement for Toast {
    fn into_element(self) -> AnyElement {
        let ink = move || palette().accent_ink;
        let band = view()
            .w_full()
            .flex_row()
            .items_center()
            .gap(u(10.))
            .px(u(16.))
            .h(u(60.))
            .bg(move || palette().accent)
            .children(self.icon.map(|i| i.render(22., ink)))
            .child(
                label(self.title)
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::NORMAL))
                    .font_weight(SEMIBOLD)
                    .text_color(ink),
            )
            .children(self.message.map(|m| {
                label(m)
                    .flex_1()
                    .min_w(px(0.))
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::NORMAL))
                    .truncate()
                    .text_color(ink)
            }));
        let band = match self.on_click {
            Some(f) => band.cursor_pointer().on_press(move || f()),
            None => band,
        };
        view().absolute().top_0().left_0().right_0().child(swivel_in(band)).into_element()
    }
}
