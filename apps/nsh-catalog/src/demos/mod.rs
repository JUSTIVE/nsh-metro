//! One live demo per catalog page, grouped like the sidebar. Every value a
//! demo lets you change is a signal in [`DemoState`], created once by the
//! app, so toggles, sliders and selections persist while browsing.

mod chrome;
mod controls;
mod foundations;
mod lists;
mod navigation;
mod overlays;
mod progress;
mod tiles;

use std::time::Duration;

use nsh_core::prelude::*;
use nsh_metro::{PAGE_MARGIN, TextStyle, page_header, text, u};

use crate::{host::Host, pages::Page};

pub use tiles::Launched;

/// How often the motion page replays its specimens.
const MOTION_LOOP: Duration = Duration::from_millis(2600);

/// Every value the demos let you change.
#[derive(Clone, Copy)]
pub struct DemoState {
    // tiles
    pub mail_count: Signal<u32>,
    pub more_tiles: Signal<bool>,
    /// The app opened from the start-screen demo, if any.
    pub launched: Signal<Option<Launched>>,
    /// Turnstile demo: entrance replays and launches.
    pub turnstile_replays: Signal<usize>,
    /// Tiles pinned into the middle of the reflow demo's grid.
    pub pinned: Signal<usize>,
    pub turnstile_exits: Signal<usize>,
    // navigation
    pub pivot: Signal<usize>,
    pub panorama: Signal<usize>,
    pub app_bar_minimized: Signal<bool>,
    pub app_bar_log: Signal<&'static str>,
    // controls
    pub button_taps: Signal<u32>,
    pub last_button: Signal<&'static str>,
    pub favorite: Signal<bool>,
    pub playing: Signal<bool>,
    pub last_link: Signal<Option<&'static str>>,
    pub wifi: Signal<bool>,
    pub bluetooth: Signal<bool>,
    pub airplane: Signal<bool>,
    pub checks: [Signal<bool>; 3],
    pub route: Signal<usize>,
    pub volume: Signal<f32>,
    pub brightness: Signal<f32>,
    pub ringtone: Signal<usize>,
    pub name: Signal<String>,
    pub message: Signal<String>,
    pub submitted: Signal<Option<String>>,
    // progress
    pub progress: Signal<f32>,
    pub loading: Signal<bool>,
    // lists
    pub selected_row: Signal<Option<usize>>,
    // overlays
    pub message_open: Signal<bool>,
    pub dont_ask: Signal<bool>,
    pub message_result: Signal<&'static str>,
    /// The toast on screen, by the number it was sent as.
    pub toast: Signal<Option<usize>>,
    pub toasts_sent: Signal<usize>,
    pub toast_log: Signal<String>,
    pub flyout_open: Signal<bool>,
    pub flyout_wide: Signal<bool>,
    pub flyout_sync: Signal<bool>,
    pub flyout_volume: Signal<f32>,
    pub flyout_log: Signal<&'static str>,
    // chrome
    pub battery: Signal<f32>,
    pub tray_opaque: Signal<bool>,
    /// Bumps every few seconds so the motion page's specimens replay.
    pub motion_loop: Signal<usize>,
}

impl DemoState {
    /// Creates the state and its timers; call from the app's component.
    pub fn new() -> Self {
        let s = Self {
            mail_count: signal(4),
            more_tiles: signal(false),
            launched: signal(None),
            turnstile_replays: signal(0),
            pinned: signal(0),
            turnstile_exits: signal(0),
            pivot: signal(0),
            panorama: signal(0),
            app_bar_minimized: signal(false),
            app_bar_log: signal("nothing yet"),
            button_taps: signal(0),
            last_button: signal("none"),
            favorite: signal(true),
            playing: signal(false),
            last_link: signal(None),
            wifi: signal(true),
            bluetooth: signal(false),
            airplane: signal(false),
            checks: [signal(true), signal(false), signal(true)],
            route: signal(0),
            volume: signal(0.6),
            brightness: signal(0.5),
            ringtone: signal(2),
            name: signal(String::new()),
            message: signal(String::new()),
            submitted: signal(None),
            progress: signal(0.42),
            loading: signal(true),
            selected_row: signal(Some(1)),
            message_open: signal(false),
            dont_ask: signal(false),
            message_result: signal("not asked yet"),
            toast: signal(None),
            toasts_sent: signal(0),
            toast_log: signal("no toasts yet".to_string()),
            flyout_open: signal(false),
            flyout_wide: signal(false),
            flyout_sync: signal(true),
            flyout_volume: signal(0.6),
            flyout_log: signal("closed"),
            battery: signal(0.8),
            tray_opaque: signal(false),
            motion_loop: signal(0),
        };
        // The motion page's specimens replay on their own, on a loop.
        let motion_loop = s.motion_loop;
        let _ = set_interval(MOTION_LOOP, move || motion_loop.update(|n| *n += 1));
        // A toast leaves on its own; a newer one restarts the clock (the
        // effect's re-run drops the old timer).
        let toast = s.toast;
        effect(move || {
            if let Some(n) = toast.get() {
                let _ = set_timeout(overlays::TOAST_FOR, move || {
                    if toast.get_untracked() == Some(n) {
                        toast.set(None);
                    }
                });
            }
        });
        s
    }
}

pub fn render(page: Page, s: DemoState, host: Host) -> AnyElement {
    match page {
        Page::Palette => foundations::palette_page(),
        Page::Typography => foundations::typography(),
        Page::Icons => foundations::icons(),
        Page::Tile => tiles::tile_page(s),
        Page::TileGrid => tiles::grid_page(s, host),
        Page::Turnstile => tiles::turnstile_page(s, host),
        Page::Pivot => navigation::pivot_page(s),
        Page::Panorama => navigation::panorama_page(s),
        Page::AppBar => navigation::app_bar_page(s),
        Page::Button => controls::buttons(s),
        Page::RoundButton => controls::round_buttons(s),
        Page::Hyperlink => controls::hyperlinks(s),
        Page::ToggleSwitch => controls::toggle_switches(s),
        Page::CheckBox => controls::check_boxes(s),
        Page::RadioButton => controls::radio_buttons(s),
        Page::Slider => controls::sliders(s),
        Page::ListPicker => controls::list_pickers(s),
        Page::TextBox => controls::text_boxes(s),
        Page::ProgressBar => progress::bar(s),
        Page::ProgressDots => progress::dots(s),
        Page::ProgressRing => progress::ring(),
        Page::ListItem => lists::list_items(s),
        Page::JumpList => lists::contacts(),
        Page::MessageBox => overlays::message_box_page(s),
        Page::ContextMenu => overlays::context_menu_page(),
        Page::Toast => overlays::toast_page(s),
        Page::SettingsFlyout => overlays::flyout_page(s),
        Page::SystemTray => chrome::tray_page(s),
        Page::Header => chrome::header_page(),
        Page::Motion => chrome::motion_page(s),
    }
}

/// What a page draws over the whole screen, tray included.
pub fn overlay(page: Page, s: DemoState) -> AnyElement {
    match page {
        Page::MessageBox => overlays::message_box_overlay(s),
        Page::Toast => overlays::toast_overlay(s),
        Page::SettingsFlyout => overlays::flyout_overlay(s),
        _ => view().into_element(),
    }
}

/// The usual page: `METRO CATALOG` over a lower-case title, then a
/// scrolling body.
fn page(title: &'static str, body: impl IntoElement) -> AnyElement {
    view()
        .size_full()
        .child(page_header(title).app_title("metro catalog"))
        .child(scroll().flex_1().min_h(u(0.)).child(view().px(u(PAGE_MARGIN)).pb(u(28.)).child(body)))
        .into_element()
}

/// A vertical stack with the standard gap between demo rows.
fn stack() -> View {
    view().gap(u(18.))
}

/// A small subtle label over a demo row.
fn caption(label: impl IntoReactive<SharedString>) -> AnyElement {
    text(TextStyle::Small, label).into_element()
}
