//! The nsh-metro component catalog: browse every component of the library
//! in a live phone frame, under any ground and accent. A port of
//! `apps/catalog` (gpui-metro) to Native Shell.
//!
//! Environment (handy for screenshots):
//! * `CATALOG_PAGE=pivot` opens straight onto a page (any page name; case,
//!   spaces and dashes are ignored).
//! * `CATALOG_MODE=light` starts on the light ground.

mod app;
mod demos;
mod host;
mod pages;
mod phone;
mod playground;

use nsh_core::prelude::*;
use nsh_metro::{Mode, Theme};
use nsh_platform_macos as platform;

use crate::pages::Page;

#[component]
fn App() -> impl IntoElement {
    // Live tiles slide together every 3 s; between waves nothing moves.
    nsh_metro::set_live_tile_beat(Some(3.));
    if std::env::var("CATALOG_MODE").is_ok_and(|m| m.eq_ignore_ascii_case("light")) {
        Theme::set_mode(Mode::Light);
    }
    let first = std::env::var("CATALOG_PAGE").ok().and_then(|p| Page::find(&p)).unwrap_or(Page::Palette);
    let page = signal(first);
    let demo = demos::DemoState::new();
    // The root's view is the main window's content (AppOptions sets its
    // title and size). Declaring `window()` here as well would open a
    // second, separate window beside the main one.
    view().flex_1().child(app::catalog(page, demo))
}

fn main() -> impl std::process::Termination {
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<App>()),
        platform::AppOptions { title: "nsh-metro catalog".to_owned(), size: (1280, 880), ..Default::default() },
    )
}
