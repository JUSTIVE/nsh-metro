//! Smoke test: a reflowing tile grid that pins a tile every 1.5 s.
use std::time::Duration;

use nsh_core::prelude::*;
use nsh_metro::{Accent, Icon, TileSize, tile, tile_grid};
use nsh_platform_macos as platform;

#[component]
pub fn App() -> impl IntoElement {
    let pinned = signal(0usize);
    let _t = set_interval(Duration::from_millis(1500), move || pinned.update(|n| *n = (*n + 1) % 4));
    let grid = tile_grid().source(move || {
        let n = pinned.get();
        eprintln!("source: pinned={n}");
        let mut tiles = vec![
            tile("mail", TileSize::Medium).icon(Icon::Mail),
            tile("phone", TileSize::Small).icon(Icon::Phone).color(Accent::Green.color()),
            tile("camera", TileSize::Small).icon(Icon::Camera).color(Accent::Magenta.color()),
            tile("clock", TileSize::Small).icon(Icon::Clock),
            tile("search", TileSize::Small).icon(Icon::Search).color(Accent::Orange.color()),
            tile("people", TileSize::Wide).icon(Icon::People).color(Accent::Crimson.color()),
        ];
        for i in (0..n).rev() {
            tiles.insert(1, tile(format!("pin{i}"), TileSize::Small).icon(Icon::Favorite).color(Accent::Teal.color()));
        }
        tiles
    });
    view().flex_1().bg(rgb(0x000000)).p(px(24.)).child(grid)
}

fn main() -> impl std::process::Termination {
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<App>()),
        platform::AppOptions { title: "nsh hello".to_owned(), size: (520, 700), ..Default::default() },
    )
}
