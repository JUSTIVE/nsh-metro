//! Settings: the classic Windows Phone theme page, plus the one foundry
//! setting the phone owns (how early reagents start warning).

use nsh_core::prelude::*;
use nsh_metro::{Accent, Mode, TextStyle, Theme, list_picker, palette, slider, text, toggle_switch, u};

use crate::app::App;
use crate::pages::{PageView, heading, scrolling, subtle, titled};
use crate::store::EXPIRY_WINDOW;

/// The twenty accents as swatches; the current one is ringed.
pub fn accent_grid(swatch: f32) -> View {
    view().flex_row().flex_wrap().gap(u(10.)).children(Accent::ALL.iter().map(move |&accent| {
        view()
            .size(u(swatch))
            .bg(accent.color())
            .border(px(3.))
            .border_color(move || if Theme::get().accent == accent { palette().foreground } else { accent.color() })
            .cursor_pointer()
            .active(|s| s.opacity(0.7))
            .on_press(move || Theme::set_accent(accent))
    }))
}

pub fn render(app: App) -> PageView {
    let (first, last) = (*EXPIRY_WINDOW.start(), *EXPIRY_WINDOW.end());
    let span = (last - first) as f32;
    let (biosafety, opened, years) =
        app.peek(|s| (s.data.biosafety.as_str().to_string(), s.data.opened.clone(), s.data.years));
    let window = move || app.live(|s| s.expiry_window);

    let body = view()
        .gap(u(14.))
        .child(
            list_picker([Mode::Dark.label(), Mode::Light.label()], || usize::from(Theme::get().mode == Mode::Light))
                .header("background")
                .on_select(|ix| Theme::set_mode(if ix == 0 { Mode::Dark } else { Mode::Light })),
        )
        .child(
            view()
                .gap(u(8.))
                .child(subtle("accent colour"))
                .child(accent_grid(78.))
                .child(text(TextStyle::Accent, || Theme::get().accent.name())),
        )
        .child(toggle_switch(|| Theme::get().reduced_motion).header("reduce motion").on_change(Theme::set_reduced_motion))
        .child(
            view()
                .child(
                    slider(move || (window() - first) as f32 / span)
                        .header("warn about reagents expiring within")
                        .step(1. / span)
                        .on_change(move |v| app.set_expiry_window(first + (v * span).round() as i64)),
                )
                .child(text(TextStyle::Large, move || format!("{} days", window()))),
        )
        .child(
            view()
                .child(heading("about"))
                .child(text(TextStyle::Normal, format!("Seoul Foundry · {biosafety}")))
                .child(subtle(format!("logging runs since {opened} · {years} years"))),
        );
    PageView::new(titled("settings", "settings", scrolling(body)))
}
