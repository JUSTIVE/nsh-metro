//! no ativ — a cyberpunk bio-foundry console, as a Windows 8-style desktop
//! app, on Native Shell with nsh-metro. A port of `apps/no-ativ` (gpui).
//!
//! `NO_ATIV_PAGE=<page>` boots straight into a page over the start screen
//! (runs, run, telemetry, strains, strain, incidents, incident, new-incident,
//! reagents, shipments, freezer, deck, people, settings).

mod app;
mod data;
mod nav;
mod pages;
mod store;
mod widgets;

use nsh_core::prelude::*;
use nsh_platform_macos as platform;

#[component]
fn NoAtiv() -> impl IntoElement {
    // Live tiles slide together every 3 s; between waves nothing moves.
    nsh_metro::set_live_tile_beat(Some(3.));
    let boot = std::env::var("NO_ATIV_PAGE").ok();
    let app = app::App::new(boot.as_deref());
    app.start_ticker();
    // The root's view is the main window's content (AppOptions sets its
    // title and size); a `window()` here would open a second window.
    view().flex_1().child(app::shell(app))
}

fn main() -> impl std::process::Termination {
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<NoAtiv>()),
        platform::AppOptions { title: "no ativ".to_owned(), size: (1280, 820), ..Default::default() },
    )
}
