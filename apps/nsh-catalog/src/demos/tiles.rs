//! Tiles in every size and face, and a packed start screen.

use nsh_core::prelude::*;
use nsh_metro::label;
use nsh_metro::{
    Accent, Icon, PAGE_MARGIN, TILE_GAP, TILE_UNIT, TextStyle, Tile, TileSize, button,
    motion::{Direction, turnstile_in},
    round_button, text,
    theme::{FONT_FAMILY, LIGHT, size},
    tile, tile_grid, toggle_switch, u,
};

use super::{DemoState, caption, navigation, page, stack};
use crate::{host::Host, phone::SCREEN_W};

/// A calendar-style front face: big date, next appointment.
fn agenda(_: TileSize) -> AnyElement {
    let ink = Hsla::from(rgb(0xFFFFFF));
    let line = |s: &'static str, size: f32| label(s).font_family(FONT_FAMILY).text_size(u(size)).text_color(ink);
    view()
        .relative()
        .size_full()
        .p(u(12.))
        .gap(u(2.))
        .child(line("dentist", size::NORMAL))
        .child(line("3:30 pm · capitol hill", size::NORMAL).opacity(0.8))
        .child(line("23", size::HUGE).font_weight(LIGHT).absolute().right(u(12.)).bottom(u(2.)))
        .into_element()
}

pub fn tile_page(s: DemoState) -> AnyElement {
    // The count is bound, not rebuilt: a new one pushes the glyph aside.
    let mail = tile("tile-mail", TileSize::Medium)
        .title("mail")
        .icon(Icon::Mail)
        .count(s.mail_count)
        .on_click(move || s.mail_count.update(|n| *n += 1));
    let smalls = view()
        .flex_row()
        .flex_wrap()
        .w(u(TILE_UNIT * 2. + TILE_GAP))
        .gap(u(TILE_GAP))
        .child(tile("tile-phone", TileSize::Small).icon(Icon::Phone).count(2))
        .child(tile("tile-camera", TileSize::Small).icon(Icon::Camera).color(Accent::Magenta.color()))
        .child(tile("tile-wifi", TileSize::Small).icon(Icon::Wifi).color(Accent::Teal.color()))
        .child(
            tile("tile-reset", TileSize::Small)
                .icon(Icon::Refresh)
                .color(Accent::Steel.color())
                .on_click(move || s.mail_count.set(0)),
        );

    page(
        "tile",
        stack()
            .child(caption("medium · small — tap mail to add, refresh to clear"))
            .child(view().flex_row().gap(u(TILE_GAP)).child(mail).child(smalls))
            .child(caption("wide · custom front face"))
            .child(tile("tile-calendar", TileSize::Wide).title("calendar").front(agenda))
            .child(caption("live · back faces peek every few seconds"))
            .child(
                view()
                    .flex_row()
                    .gap(u(TILE_GAP))
                    .child(
                        tile("tile-people", TileSize::Medium)
                            .title("people")
                            .icon(Icon::People)
                            .back_text("kristin", ["new photo"])
                            .period(10.),
                    )
                    .child(
                        tile("tile-music", TileSize::Medium)
                            .title("music")
                            .icon(Icon::Play)
                            .color(Accent::Orange.color())
                            .back_text("now playing", ["daft punk · get lucky"])
                            .period(10.)
                            .phase(0.5),
                    ),
            )
            .child(caption("large"))
            .child(
                tile("tile-weather", TileSize::Large)
                    .title("weather")
                    .icon(Icon::Snow)
                    .color(Accent::Cobalt.color())
                    .back_text("seattle −2°", ["snow through the evening, clearing overnight"])
                    .period(12.)
                    .phase(0.25),
            ),
    )
}

/// An app opened from the start-screen demo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Launched {
    /// outlook: a pivot of mail folders.
    Pivot,
    /// people: a panorama hub.
    Panorama,
}

/// The start screen's tiles, in pin order. people and outlook launch apps.
fn start_tiles(s: DemoState) -> Vec<Tile> {
    let launch = move |app: Launched| move || s.launched.set(Some(app));
    vec![
        tile("start-phone", TileSize::Medium).title("phone").icon(Icon::Phone).count(1),
        tile("start-sms", TileSize::Small).icon(Icon::Mail).count(3),
        tile("start-wifi", TileSize::Small).icon(Icon::Wifi),
        tile("start-camera", TileSize::Small).icon(Icon::Camera),
        tile("start-settings", TileSize::Small).icon(Icon::Settings),
        tile("start-people", TileSize::Wide)
            .title("people")
            .icon(Icon::People)
            .back_text("ben, kristin, anna", ["3 new posts"])
            .period(11.)
            .on_click(launch(Launched::Panorama)),
        tile("start-mail", TileSize::Medium)
            .title("outlook")
            .icon(Icon::Mail)
            .count(s.mail_count)
            .on_click(launch(Launched::Pivot)),
        tile("start-calendar", TileSize::Medium)
            .title("calendar")
            .icon(Icon::Calendar)
            .back_text("dentist", ["3:30 pm today"])
            .period(13.)
            .phase(0.4),
        tile("start-music", TileSize::Wide).title("music+videos").icon(Icon::Play).color(Accent::Orange.color()),
        tile("start-clock", TileSize::Small).icon(Icon::Clock),
        tile("start-search", TileSize::Small).icon(Icon::Search),
        tile("start-store", TileSize::Medium).title("store").icon(Icon::Download).color(Accent::Emerald.color()),
        tile("start-weather", TileSize::Large)
            .title("weather")
            .icon(Icon::Snow)
            .back_text("seattle −2°", ["snow"])
            .period(12.)
            .phase(0.7),
        tile("start-lock", TileSize::Small).icon(Icon::Lock),
        tile("start-favorite", TileSize::Small).icon(Icon::Favorite).color(Accent::Crimson.color()),
    ]
}

pub fn grid_page(s: DemoState, host: Host) -> AnyElement {
    dynamic(move || match s.launched.get() {
        Some(app) => {
            let body = match app {
                Launched::Pivot => navigation::pivot_page(s),
                Launched::Panorama => navigation::panorama_page(s),
            };
            view()
                .size_full()
                .child(view().flex_1().min_h(u(0.)).child(turnstile_in(Direction::Forward, 0., 0., body).size_full()))
                .child(
                    view()
                        .flex_row()
                        .flex_none()
                        .items_center()
                        .gap(u(12.))
                        .px(u(PAGE_MARGIN))
                        .py(u(10.))
                        .child(round_button(Icon::Back).diameter(40.).on_click(move || s.launched.set(None)))
                        .child(caption("back to start")),
                )
                .into_element()
        }
        // Coming back from an app, the grid's own turnstile swings the tiles in.
        None => start_screen(s, host),
    })
}

/// The turnstile on its own: tap any tile to watch it leave and come back.
pub fn turnstile_page(s: DemoState, host: Host) -> AnyElement {
    page(
        "turnstile",
        stack()
            .child(caption("tap a tile: the tiles swing out, then back in"))
            .child(
                view()
                    .flex_row()
                    .gap(u(12.))
                    .child(button("replay entrance").on_click(move || s.turnstile_replays.update(|n| *n += 1)))
                    .child(button("pin a tile").on_click(move || s.pinned.update(|n| *n += 1)))
                    .child(button("unpin").on_click(move || s.pinned.update(|n| *n = n.saturating_sub(1)))),
            )
            .child(
                tile_grid()
                    .turnstile(true)
                    .leave(host.leaving.read_only())
                    .replay(s.turnstile_replays.read_only())
                    .source(move || turnstile_tiles(s, s.pinned.get())),
            )
            .child(caption(move || format!("{} launches", s.turnstile_exits.get()))),
    )
}

/// The turnstile page's tiles, with `pinned` tiles after mail: pinning one
/// slides the rest along to make room, and the new tile turns in.
fn turnstile_tiles(s: DemoState, pinned: usize) -> Vec<nsh_metro::Tile> {
    let again = move |key: &'static str, size: TileSize, icon: Icon, color: Option<Accent>| {
        let t = tile(key, size).icon(icon).on_click(move || {
            s.turnstile_exits.update(|n| *n += 1);
            s.turnstile_replays.update(|n| *n += 1);
        });
        match color {
            Some(c) => t.color(c.color()),
            None => t,
        }
    };
    const PINNED: [(TileSize, Icon); 4] =
        [(TileSize::Small, Icon::Favorite), (TileSize::Medium, Icon::Camera), (TileSize::Small, Icon::Wifi), (TileSize::Wide, Icon::Flask)];
    let mut tiles = vec![
        again("ts-mail", TileSize::Medium, Icon::Mail, None).title("mail"),
        again("ts-phone", TileSize::Small, Icon::Phone, Some(Accent::Teal)),
        again("ts-camera", TileSize::Small, Icon::Camera, Some(Accent::Magenta)),
        again("ts-clock", TileSize::Small, Icon::Clock, None),
        again("ts-search", TileSize::Small, Icon::Search, Some(Accent::Orange)),
        again("ts-people", TileSize::Wide, Icon::People, Some(Accent::Cobalt)).title("people"),
        again("ts-music", TileSize::Medium, Icon::Play, Some(Accent::Crimson)).title("music"),
        again("ts-calendar", TileSize::Medium, Icon::Calendar, None).title("calendar"),
    ];
    for n in (0..pinned).rev() {
        let (size, icon) = PINNED[n % PINNED.len()];
        tiles.insert(1, tile(format!("ts-pinned-{n}"), size).icon(icon).color(Accent::ALL[(n * 7 + 3) % Accent::ALL.len()].color()));
    }
    tiles
}

fn start_screen(s: DemoState, host: Host) -> AnyElement {
    let grid = move || {
        let more = s.more_tiles.get();
        let (columns, gap) = if more { (6, 8.) } else { (4, TILE_GAP) };
        let width = SCREEN_W - 2. * PAGE_MARGIN;
        let unit = (width - (columns - 1) as f32 * gap) / columns as f32;
        view()
            .gap(u(16.))
            .child(
                tile_grid()
                    .turnstile(true)
                    .leave(host.leaving.read_only())
                    .columns(columns)
                    .metrics(unit, gap)
                    .tiles(start_tiles(s)),
            )
            .child(text(TextStyle::Small, format!("{columns} columns · unit {unit:.0} px · gap {gap:.0} px")))
    };
    scroll()
        .size_full()
        .child(
            view()
                .px(u(PAGE_MARGIN))
                .pt(u(16.))
                .pb(u(40.))
                .gap(u(16.))
                .child(
                    toggle_switch(s.more_tiles)
                        .header("show more tiles")
                        .labels("6 columns", "4 columns")
                        .on_change(move |v| s.more_tiles.set(v)),
                )
                .child(caption("tap people (panorama) or outlook (pivot) to launch"))
                .child(dynamic(grid)),
        )
        .into_element()
}
