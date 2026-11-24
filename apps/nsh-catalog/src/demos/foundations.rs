//! Accents and palette tokens, the type ramp, the icon set.

use nsh_core::prelude::*;
use nsh_metro::{
    Accent, Icon, Palette, TextStyle, Theme, palette, text,
    theme::{FONT_FAMILY, size},
    u,
};

use super::{caption, page, stack};

/// Four squares across the 432 px content column.
const CELL: f32 = 102.;

pub fn palette_page() -> AnyElement {
    let swatches = Accent::ALL.into_iter().map(|a| {
        view()
            .relative()
            .size(u(CELL))
            .bg(a.color())
            .cursor_pointer()
            .active(|s| s.opacity(0.8))
            .on_press(move || Theme::set_accent(a))
            .child(when(
                move || Theme::get().accent == a,
                move || view().absolute().top(u(6.)).right(u(6.)).child(Icon::Check.render(22., a.ink())),
            ))
            .child(
                nsh_core::prelude::text(a.name())
                    .absolute()
                    .left(u(8.))
                    .bottom(u(6.))
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::SMALL))
                    .text_color(a.ink()),
            )
    });

    let tokens: [(&str, fn(&Palette) -> Hsla); 12] = [
        ("background", |p| p.background),
        ("foreground", |p| p.foreground),
        ("subtle", |p| p.subtle),
        ("disabled", |p| p.disabled),
        ("chrome", |p| p.chrome),
        ("border", |p| p.border),
        ("input", |p| p.input),
        ("input focused", |p| p.input_focused),
        ("input ink", |p| p.input_ink),
        ("scrim", |p| p.scrim),
        ("accent", |p| p.accent),
        ("accent ink", |p| p.accent_ink),
    ];
    let rows = tokens.into_iter().map(|(name, pick)| {
        view()
            .flex_row()
            .items_center()
            .gap(u(14.))
            .child(view().size(u(32.)).bg(move || pick(&palette())).border_1().border_color(move || palette().subtle))
            .child(text(TextStyle::Normal, name))
    });

    page(
        "accents",
        stack()
            .child(caption(move || {
                let t = Theme::get();
                format!("{} on {} — tap a colour", t.accent.name(), t.mode.label())
            }))
            .child(view().flex_row().flex_wrap().gap(u(8.)).children(swatches))
            .child(text(TextStyle::GroupHeader, "palette"))
            .child(view().gap(u(8.)).children(rows)),
    )
}

const STYLES: [(TextStyle, &str, &str); 11] = [
    (TextStyle::Title1, "title 1", "settings"),
    (TextStyle::Title2, "title 2", "wi-fi networks"),
    (TextStyle::Title3, "title 3", "recent conversations"),
    (TextStyle::ExtraLarge, "extra large", "kristin"),
    (TextStyle::Large, "large", "mobile 425 555 0102"),
    (TextStyle::Normal, "normal", "Metro is type-first: content over chrome."),
    (TextStyle::Subtle, "subtle", "last synced two minutes ago"),
    (TextStyle::Accent, "accent", "3 new messages"),
    (TextStyle::Small, "small", "captions, dates and small print"),
    (TextStyle::AppTitle, "app title", "outlook"),
    (TextStyle::GroupHeader, "group header", "favourites"),
];

pub fn typography() -> AnyElement {
    page(
        "typography",
        stack().child(caption(format!("set in {FONT_FAMILY}"))).children(STYLES.into_iter().map(|(style, name, sample)| {
            view().child(caption(format!("{name} · {} px", style.size()))).child(text(style, sample))
        })),
    )
}

pub fn icons() -> AnyElement {
    let cells = Icon::ALL.iter().map(|&i| {
        view()
            .items_center()
            .justify_center()
            .gap(u(8.))
            .w(u(CELL))
            .h(u(92.))
            .child(i.render(36., move || palette().foreground))
            .child(
                nsh_core::prelude::text(i.name().replace('_', " "))
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::SMALL - 1.))
                    .text_color(move || palette().subtle),
            )
    });
    page(
        "icons",
        stack()
            .child(caption(format!("{} glyphs, tinted by text colour", Icon::ALL.len())))
            .child(view().flex_row().flex_wrap().gap(u(8.)).children(cells)),
    )
}
