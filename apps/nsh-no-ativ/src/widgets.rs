//! Small local stand-ins for what nsh-metro doesn't (yet) offer.

use std::rc::Rc;

use nsh_core::{element::input, prelude::*};
use nsh_metro::{
    Icon, TileSize, palette,
    theme::{FONT_FAMILY, LIGHT, size},
    u,
};

/// nsh-metro's text box, but it reports focus into `editing`, so the back
/// key can dismiss the keyboard before it navigates.
pub fn text_box(value: Signal<String>, header: &str, placeholder: &str, editing: Signal<bool>) -> AnyElement {
    let focused = signal(false);
    let field = input()
        .value(move || value.get())
        .on_change(move |v: &str| value.set(v.to_string()))
        .on_focus(move || {
            focused.set(true);
            editing.set(true);
        })
        .on_blur(move || {
            focused.set(false);
            editing.set(false);
        })
        .placeholder(placeholder.to_string())
        .flex_1()
        .font_family(FONT_FAMILY)
        .text_size(u(size::MEDIUM))
        .text_color(move || palette().input_ink)
        .caret_color(Hsla::from(rgb(0x000000)))
        .selection_color(move || palette().accent.opacity(0.45));
    view()
        .gap(u(4.))
        .child(
            text(SharedString::from(header.to_string()))
                .font_family(FONT_FAMILY)
                .text_size(u(size::NORMAL))
                .text_color(move || palette().subtle),
        )
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

/// A line of a live tile's face that follows the store.
pub type Line = Rc<dyn Fn() -> SharedString>;

pub fn line(f: impl Fn() -> String + 'static) -> Line {
    Rc::new(move || SharedString::from(f()))
}

/// The usual back face (a headline over a few lines), with text that
/// updates in place: nsh-metro's `back_text` takes fixed strings.
pub fn live_back(size: TileSize, headline: Line, lines: Vec<Line>, bg: Option<Rc<dyn Fn() -> Hsla>>) -> AnyElement {
    let big = if size == TileSize::Small { size::MEDIUM } else { size::LARGE };
    let ink = move || palette().accent_ink;
    let face = view().size_full().p(u(10.)).pb(u(26.)).gap(u(2.)).overflow_hidden();
    let face = match bg {
        Some(bg) => face.bg(move || bg()),
        None => face,
    };
    face.child(
        text(move || headline())
            .font_family(FONT_FAMILY)
            .text_size(u(big))
            .line_height(nsh_core::LineHeight::Points(px(big + 4.)))
            .text_color(ink),
    )
    .children(lines.into_iter().map(move |l| {
        text(move || l()).font_family(FONT_FAMILY).text_size(u(size::SMALL)).truncate().text_color(ink)
    }))
    .into_element()
}

/// The default front face (icon + count) with a count and fill that follow
/// the store: nsh-metro's takes a fixed count and colour.
pub fn live_front(size: TileSize, icon: Icon, count: Rc<dyn Fn() -> u32>, bg: Rc<dyn Fn() -> Hsla>) -> AnyElement {
    let ink = move || palette().accent_ink;
    let icon_px = if size == TileSize::Small { 40. } else { 64. };
    let label = move || {
        let c = count();
        SharedString::from(if c > 0 { c.to_string() } else { String::new() })
    };
    view()
        .size_full()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(u(8.))
        .bg(move || bg())
        .child(icon.render(icon_px, ink))
        .child(text(label).font_family(FONT_FAMILY).text_size(u(size::HUGE * 0.8)).font_weight(LIGHT).text_color(ink))
        .into_element()
}
