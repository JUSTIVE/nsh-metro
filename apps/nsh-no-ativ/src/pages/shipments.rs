//! Shipments: cryo boxes leaving the floor, each with its stage progress.

use nsh_core::prelude::*;
use nsh_metro::{TextStyle, palette, pivot, progress_bar, text, u};

use crate::app::App;
use crate::data::{Shipment, ShipmentStage};
use crate::pages::{APP, PageView, alarm, raw, scrolling, subtle};
use crate::store::stage_progress;

const STAGES: [ShipmentStage; 5] = [
    ShipmentStage::Packed,
    ShipmentStage::PickedUp,
    ShipmentStage::InTransit,
    ShipmentStage::Customs,
    ShipmentStage::Delivered,
];

fn shipment_row(s: &Shipment) -> View {
    let current = s.stage;
    let stages = view().flex_row().justify_between().children(STAGES.iter().map(|&stage| {
        let reached = stage_progress(stage) <= stage_progress(current);
        raw(stage.as_str(), 11.).text_color(move || {
            let p = palette();
            if stage == current {
                p.accent
            } else if reached {
                p.foreground
            } else {
                p.disabled
            }
        })
    }));
    let excursions = (s.excursions > 0).then(|| {
        subtle(format!("{} excursion{}", s.excursions, if s.excursions == 1 { "" } else { "s" })).color(alarm())
    });
    view()
        .gap(u(4.))
        .py(u(10.))
        .child(text(TextStyle::Large, s.destination.to_lowercase()))
        .child(text(TextStyle::Accent, format!("{} · {}", s.code, s.eta)))
        .child(view().pt(u(6.)).child(progress_bar(stage_progress(s.stage))))
        .child(stages)
        .child(
            view()
                .flex_row()
                .gap(u(8.))
                .child(subtle(format!("{} · {} vials · {} kg dry ice", s.courier, s.vials, s.dry_ice_kg)))
                .children(excursions),
        )
}

fn pane(app: App, selected: usize) -> AnyElement {
    let delivered = selected == 1;
    let rows: Vec<View> = app.read(|s| {
        let mut shipments: Vec<&Shipment> =
            s.data.shipments.iter().filter(|sh| (sh.stage == ShipmentStage::Delivered) == delivered).collect();
        shipments.sort_by_key(|sh| if delivered { -sh.eta_days } else { sh.eta_days });
        shipments.into_iter().map(shipment_row).collect()
    });
    scrolling(view().children(rows))
}

pub fn render(app: App, selected: usize) -> PageView {
    let sel = signal(selected);
    PageView::new(
        pivot(["en route", "delivered"], move || sel.get())
            .title(APP)
            .on_select(move |ix| {
                sel.set(ix);
                app.set_selected(ix);
            })
            .panes((0..2).map(|i| move || pane(app, i))),
    )
}
