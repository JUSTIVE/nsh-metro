//! Telemetry: a panorama with one section per zone. Values tick live: each
//! reading is bound to the store, so a tick changes text, not the tree.

use nsh_core::prelude::*;
use nsh_metro::{PAGE_MARGIN, TextStyle, palette, panorama, text, theme::LIGHT, theme::size, u};

use crate::app::{App, LEFT_GUTTER};
use crate::data::{Sensor, SensorState};
use crate::pages::{PageView, alarm, raw, subtle, warn};
use crate::store::reading;

/// The window's width as it opens (gpui measured the viewport each frame;
/// a panorama's section width is fixed when it is built).
const WINDOW_WIDTH: f32 = 1280.;

fn sensor_card(app: App, ix: usize) -> View {
    let sensor = move |f: fn(&Sensor) -> String| move || app.live(|s| s.data.sensors.get(ix).map(f).unwrap_or_default());
    let state = move || app.live(|s| s.data.sensors.get(ix).map(|x| x.state).unwrap_or(SensorState::Ok));
    let tone = move || match state() {
        SensorState::Ok => palette().foreground,
        SensorState::Warn => warn(),
        SensorState::Alarm => alarm(),
    };
    view()
        .w(relative(0.5))
        .pb(u(18.))
        .child(text(TextStyle::Small, sensor(|s| s.kind.as_str().to_string())))
        .child(
            view()
                .flex_row()
                .items_baseline()
                .gap(u(6.))
                .child(raw(sensor(reading), size::EXTRA_LARGE + 6.).font_weight(LIGHT).text_color(tone))
                .child(subtle(sensor(|s| s.unit.clone()))),
        )
        .child(
            subtle(sensor(|s| format!("{}–{} · {}", s.low, s.high, s.state.as_str())))
                .color(move || if state() == SensorState::Ok { palette().subtle } else { tone() }),
        )
        .child(raw(sensor(|s| s.code.clone()), size::SMALL - 2.).text_color(move || palette().disabled))
}

pub fn render(app: App, zone: usize) -> PageView {
    let zones: Vec<(String, Vec<usize>)> = app.peek(|s| {
        let mut zones: Vec<(String, Vec<usize>)> = Vec::new();
        for (ix, sensor) in s.data.sensors.iter().enumerate() {
            match zones.iter_mut().find(|(z, _)| *z == sensor.zone) {
                Some((_, members)) => members.push(ix),
                None => zones.push((sensor.zone.clone(), vec![ix])),
            }
        }
        zones
    });
    // A section is a bit under half the window, so the next one peeks in.
    let page_width = WINDOW_WIDTH - LEFT_GUTTER;
    let section = ((page_width - 2. * PAGE_MARGIN) / 2.2).clamp(380., 620.);
    let sel = signal(zone);
    let mut pano = panorama("telemetry", move || sel.get()).section_width(section).on_select(move |ix| {
        sel.set(ix);
        app.set_selected(ix);
    });
    for (name, members) in zones {
        pano = pano.section(name.to_lowercase(), move || {
            scroll()
                .size_full()
                .child(view().flex_row().flex_wrap().children(members.iter().map(|&ix| sensor_card(app, ix))))
        });
    }
    PageView::new(pano)
}
