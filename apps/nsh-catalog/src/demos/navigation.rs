//! Pivot, panorama and the application bar.

use nsh_core::prelude::*;
use nsh_metro::{
    Accent, AppBarMode, Icon, PAGE_MARGIN, TILE_GAP, TextStyle, TileSize, app_bar, list_item, page_header, palette,
    panorama, pivot, text,
    theme::{FONT_FAMILY, SEMILIGHT, size},
    tile, toggle_switch, u,
};

use super::{DemoState, caption, stack};

const FOLDERS: [&str; 4] = ["inbox", "unread", "flagged", "urgent"];

struct Mail {
    from: &'static str,
    subject: &'static str,
    preview: &'static str,
    unread: bool,
    flagged: bool,
    urgent: bool,
}

const MAIL: [Mail; 7] = [
    Mail {
        from: "kristin",
        subject: "dinner friday?",
        preview: "there's a new thai place on 15th",
        unread: true,
        flagged: false,
        urgent: false,
    },
    Mail {
        from: "build bot",
        subject: "nightly failed",
        preview: "3 tests failed on arm64",
        unread: true,
        flagged: true,
        urgent: true,
    },
    Mail {
        from: "anna",
        subject: "photos from the trip",
        preview: "finally got them off my camera",
        unread: false,
        flagged: true,
        urgent: false,
    },
    Mail {
        from: "landlord",
        subject: "water shut-off tuesday",
        preview: "between 9 and noon",
        unread: true,
        flagged: false,
        urgent: true,
    },
    Mail {
        from: "ben",
        subject: "re: metro catalog",
        preview: "the pivot looks great",
        unread: false,
        flagged: false,
        urgent: false,
    },
    Mail {
        from: "airline",
        subject: "check in now",
        preview: "SEA → SFO, seat 14A",
        unread: false,
        flagged: true,
        urgent: false,
    },
    Mail {
        from: "newsletter",
        subject: "this week in rust",
        preview: "edition 2024 is here",
        unread: false,
        flagged: false,
        urgent: false,
    },
];

/// One folder's pane: its mail as list items (which trail the pivot's
/// slide line by line).
fn folder_pane(folder: usize) -> AnyElement {
    let rows = MAIL
        .iter()
        .filter(move |m| match folder {
            1 => m.unread,
            2 => m.flagged,
            3 => m.urgent,
            _ => true,
        })
        .map(|m| {
            let item = list_item(m.from).subtitle(m.subject).detail(m.preview);
            if m.unread { item } else { item.subtle_subtitle() }
        });
    scroll().size_full().child(view().px(u(PAGE_MARGIN)).children(rows)).into_element()
}

pub fn pivot_page(s: DemoState) -> AnyElement {
    pivot(FOLDERS, s.pivot)
        .title("outlook")
        .on_select(move |ix| s.pivot.set(ix))
        .panes((0..FOLDERS.len()).map(|f| move || folder_pane(f)))
        .into_element()
}

fn big(label: &'static str) -> Text {
    nsh_core::prelude::text(label)
        .font_family(FONT_FAMILY)
        .text_size(u(size::EXTRA_LARGE))
        .font_weight(SEMILIGHT)
        .line_height(nsh_core::LineHeight::Points(px(52.)))
        .text_color(move || palette().foreground)
}

pub fn panorama_page(s: DemoState) -> AnyElement {
    let collection = || {
        view()
            .child(big("music"))
            .child(big("videos"))
            .child(big("podcasts"))
            .child(big("radio"))
            .child(view().pt(u(12.)).child(text(TextStyle::Subtle, "shift + scroll, swipe, or tap the peeking section")))
    };
    let history = || {
        let colors = [Accent::Orange, Accent::Magenta, Accent::Teal, Accent::Lime, Accent::Violet, Accent::Cobalt];
        view().flex_row().flex_wrap().gap(u(TILE_GAP)).children(colors.into_iter().enumerate().map(|(ix, a)| {
            tile(format!("pano-tile-{ix}"), TileSize::Medium).color(a.color()).icon(Icon::Play).metrics(85., TILE_GAP)
        }))
    };
    let new = || {
        view().children(
            [("random access memories", "daft punk"), ("lonerism", "tame impala"), ("settle", "disclosure")].into_iter().map(
                |(album, artist)| {
                    // The leading square takes a fixed colour: the accent now.
                    list_item(album).subtitle(artist).leading_icon(Icon::Play).leading_color(palette().accent)
                },
            ),
        )
    };
    panorama("music+videos", s.panorama)
        .section_width(432.)
        .on_select(move |ix| s.panorama.set(ix))
        .section("collection", collection)
        .section("history", history)
        .section("new", new)
        .into_element()
}

pub fn app_bar_page(s: DemoState) -> AnyElement {
    let bar = move || {
        let mode = if s.app_bar_minimized.get() { AppBarMode::Minimized } else { AppBarMode::Default };
        let log = move |label: &'static str| move || s.app_bar_log.set(label);
        app_bar()
            .mode(mode)
            .button(Icon::Add, "new", log("new"))
            .button(Icon::Sync, "sync", log("sync"))
            .button(Icon::Search, "search", log("search"))
            .disabled_button(Icon::Delete, "delete")
            .menu_item("settings", log("settings"))
            .menu_item("about", log("about"))
    };

    view()
        .relative()
        .size_full()
        .child(page_header("app bar").app_title("metro catalog"))
        .child(
            stack()
                .px(u(PAGE_MARGIN))
                .child(
                    toggle_switch(s.app_bar_minimized)
                        .header("minimized mode")
                        .on_change(move |v| s.app_bar_minimized.set(v)),
                )
                .child(caption("tap ··· to expand: labels and menu items appear"))
                .child(text(TextStyle::Large, move || format!("last action: {}", s.app_bar_log.get()))),
        )
        // The bar overlays the page bottom so expanding never reflows it.
        .child(view().absolute().left_0().right_0().bottom_0().child(dynamic(bar)))
        .into_element()
}
