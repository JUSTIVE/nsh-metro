//! Freezer: occupancy per rack and box.

use nsh_core::prelude::*;
use nsh_metro::{TextStyle, progress_bar, text, u};

use crate::app::App;
use crate::pages::{PageView, heading, scrolling, subtle, titled};

pub fn render(app: App) -> PageView {
    let (summary, racks) = app.read(|store| {
        let (filled, slots) = store.freezer_occupancy();
        let summary = view()
            .child(text(TextStyle::ExtraLarge, format!("{} %", filled * 100 / slots.max(1))))
            .child(subtle(format!("{filled} of {slots} slots across {} racks", store.data.racks.len())));
        let racks: Vec<View> = store
            .data
            .racks
            .iter()
            .map(|rack| {
                let boxes = rack.boxes.iter().map(|b| {
                    let used = b.slots.iter().filter(|s| s.is_some()).count();
                    view()
                        .flex_row()
                        .items_center()
                        .gap(u(12.))
                        .py(u(5.))
                        .child(view().w(u(36.)).child(text(TextStyle::Normal, b.label.clone())))
                        .child(view().flex_1().child(progress_bar(used as f32 / b.slots.len().max(1) as f32)))
                        .child(view().w(u(56.)).items_end().child(subtle(format!("{used}/{}", b.slots.len()))))
                });
                view().child(heading(format!("{} · {}", rack.name.to_lowercase(), rack.temp))).children(boxes)
            })
            .collect();
        (summary, racks)
    });
    PageView::new(titled("freezer", "freezer", scrolling(view().child(summary).children(racks))))
}
