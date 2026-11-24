//! Deck: today's bookings, and the machines they run on.

use nsh_core::prelude::*;
use nsh_metro::{Accent, Icon, ListItem, list_item, palette, pivot};

use crate::app::App;
use crate::data::{DECK_TODAY, DeckBlock, DeckKind, InstrumentState};
use crate::nav::Page;
use crate::pages::{APP, PageView, alarm, scrolling, warn};
use crate::store::Store;

fn kind_color(kind: DeckKind) -> Hsla {
    match kind {
        DeckKind::Run => palette().accent,
        DeckKind::Cip => Accent::Teal.color(),
        DeckKind::Maintenance => warn(),
        DeckKind::Booked => Accent::Steel.color(),
    }
}

fn booking(app: App, block: &DeckBlock, store: &Store) -> ListItem {
    let machine = store.instrument(&block.instrument_id).map(|i| i.code.clone()).unwrap_or_default();
    let strain = block.strain_id.as_deref().and_then(|s| store.strain(s)).map(|s| format!(" · {}", s.name));
    let item = list_item(machine)
        .leading_icon(Icon::Calendar)
        .leading_color(kind_color(block.kind))
        .subtitle(format!("{}{}", block.label, strain.unwrap_or_default()))
        .detail(format!(
            "{} · day {} of {} · {}",
            block.kind.as_str(),
            DECK_TODAY - block.day + 1,
            block.days,
            block.operator
        ));
    match block.run_id.clone() {
        Some(id) => item.on_click(move || app.push(Page::Run { id: id.clone() })),
        None => item,
    }
}

fn pane(app: App, selected: usize) -> AnyElement {
    let rows: Vec<AnyElement> = app.read(|s| {
        if selected == 0 {
            s.deck_today().map(|b| booking(app, b, s).into_element()).collect()
        } else {
            s.data
                .instruments
                .iter()
                .map(|i| {
                    let tone = match i.state {
                        InstrumentState::Down => alarm(),
                        InstrumentState::Cip => warn(),
                        InstrumentState::Running => palette().accent,
                        InstrumentState::Ready => Accent::Emerald.color(),
                    };
                    list_item(i.code.clone())
                        .leading_icon(Icon::Gauge)
                        .leading_color(tone)
                        .subtitle(format!("{} for {} h", i.state.as_str(), i.since_hours))
                        .detail(format!("{} · {} cycles", i.kind, i.cycles))
                        .into_element()
                })
                .collect()
        }
    });
    scrolling(view().children(rows))
}

pub fn render(app: App, selected: usize) -> PageView {
    let sel = signal(selected);
    PageView::new(
        pivot(["today", "instruments"], move || sel.get())
            .title(APP)
            .on_select(move |ix| {
                sel.set(ix);
                app.set_selected(ix);
            })
            .panes((0..2).map(|i| move || pane(app, i))),
    )
}
