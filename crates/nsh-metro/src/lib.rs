//! # nsh-metro
//!
//! The Windows Phone "Metro" design language for [Native Shell] (nsh):
//! type-first layouts, flat accent colour, live tiles, pivots and
//! panoramas, drawn by the platform's own views (AppKit on macOS) with
//! their motion on the platform's animation clock.
//!
//! A port of gpui-metro; see `MIGRATION.md` for what changed.
//!
//! [Native Shell]: https://nsh.dev

mod components;
pub mod icons;
pub mod motion;
pub mod nav;
pub mod theme;

pub use components::*;
pub use icons::{Icon, icon};
pub use nav::page_host;
pub use theme::{Accent, Mode, Palette, Theme, palette, u};

/// Everything an app usually needs in scope.
pub mod prelude {
    pub use crate::{
        Accent, Icon, Mode, Palette, Theme, TextStyle, TileSize, app_bar, button, check_box, hyperlink, jump_list,
        list_item, list_picker, message_box, motion::Direction, page_header, page_host, palette, panorama, pivot,
        progress_bar, progress_dots, progress_ring, radio_button, round_button, settings_flyout, slider,
        system_tray, text, text_box, tile, tile_grid, toast, toggle_switch, u,
    };
}
