//! Reagents: every lot with its shelf life, expiring and expired stock toned.

use nsh_core::prelude::*;
use nsh_metro::{TextStyle, palette, pivot, progress_bar, text, u};

use crate::app::App;
use crate::data::{LotState, ReagentLot, to_fixed};
use crate::pages::{APP, PageView, alarm, scrolling, subtle, warn};

fn tone(state: LotState) -> Hsla {
    match state {
        LotState::Expired => alarm(),
        LotState::Expiring | LotState::Low => warn(),
        LotState::Ok => palette().subtle,
    }
}

fn lot_row(lot: &ReagentLot, state: LotState) -> View {
    let shelf = (lot.shelf_days.max(0) as f32 / lot.shelf_total.max(1) as f32).clamp(0., 1.);
    let expiry = match state {
        LotState::Expired => lot.expires.clone(),
        LotState::Ok => format!("expires {}", lot.expires),
        _ => format!("{} · expires {}", state.as_str(), lot.expires),
    };
    view()
        .gap(u(4.))
        .py(u(10.))
        .child(text(TextStyle::Large, lot.name.clone()))
        .child(subtle(format!("{} · {} · {} · {}", lot.code, lot.vendor, lot.grade, lot.location)))
        .child(view().pt(u(4.)).child(progress_bar(shelf)))
        .child(
            view()
                .flex_row()
                .justify_between()
                .child(text(TextStyle::Normal, expiry).color(move || tone(state)))
                .child(
                    text(TextStyle::Normal, format!("{} / {} {}", to_fixed(lot.qty, 2), lot.capacity, lot.unit))
                        .color(move || if state == LotState::Low { warn() } else { palette().subtle }),
                ),
        )
        .children(lot.reserved_for.clone().map(|run| subtle(format!("reserved for {run}"))))
}

fn pane(app: App, selected: usize) -> AnyElement {
    let (window, rows) = app.read(|s| {
        let mut lots: Vec<(&ReagentLot, LotState)> = s.data.lots.iter().map(|l| (l, s.lot_state(l))).collect();
        if selected == 0 {
            lots.retain(|(_, st)| *st != LotState::Ok);
            lots.sort_by_key(|(l, _)| l.shelf_days);
        }
        (s.expiry_window, lots.into_iter().map(|(l, st)| lot_row(l, st)).collect::<Vec<_>>())
    });
    scrolling(
        view().child(subtle(format!("warning {window} days before expiry · change it in settings"))).children(rows),
    )
}

pub fn render(app: App, selected: usize) -> PageView {
    let sel = signal(selected);
    PageView::new(
        pivot(["attention", "all lots"], move || sel.get())
            .title(APP)
            .on_select(move |ix| {
                sel.set(ix);
                app.set_selected(ix);
            })
            .panes((0..2).map(|i| move || pane(app, i))),
    )
}
