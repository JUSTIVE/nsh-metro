//! The device the demos run in: a 480×800 screen in a bezel, with the
//! system tray on top and the three hardware buttons below.

use nsh_core::prelude::*;
use nsh_metro::{Icon, palette, system_tray, theme::FONT_FAMILY, u};

use crate::{demos, demos::DemoState, host, pages::Page};

pub const SCREEN_W: f32 = 480.;
const SCREEN_H: f32 = 800.;

pub fn frame(page: Signal<Page>, demo: DemoState) -> AnyElement {
    // The tray follows the system-tray page's controls.
    let tray = dynamic(move || {
        let t = system_tray("12:38").battery(demo.battery.get());
        if demo.tray_opaque.get() { t.opaque() } else { t }
    });
    // The page turns about the screen's left edge.
    let (body, host) = host::page_host(page, 0., move |p, host| demos::render(p, demo, host));
    // Overlays are children of the screen, so they cover the phone only
    // (tray included); they follow the page on screen.
    let overlay = dynamic(move || demos::overlay(host.shown.get(), demo));

    let screen = view()
        .relative()
        .overflow_hidden()
        .flex_none()
        .w(u(SCREEN_W))
        .h(u(SCREEN_H))
        .bg(move || palette().background)
        .text_color(move || palette().foreground)
        .font_family(FONT_FAMILY)
        .child(tray)
        .child(view().relative().flex_1().min_h(u(0.)).child(body))
        .child(overlay);

    let hardware = view()
        .flex_row()
        .justify_around()
        .items_center()
        .h(u(56.))
        .children([Icon::Back, Icon::Home, Icon::Search].map(|i| i.render(22., move || palette().subtle)));

    view()
        .flex_none()
        .px(u(14.))
        .pt(u(30.))
        .rounded(u(34.))
        .bg(move || palette().chrome)
        .border_1()
        .border_color(move || palette().disabled.opacity(0.5))
        .child(screen)
        .child(hardware)
        .into_element()
}
