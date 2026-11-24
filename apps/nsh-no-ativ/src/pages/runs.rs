//! Runs: a pivot over the ledger, and the run detail with harvest.

use nsh_core::prelude::*;
use nsh_metro::{Accent, Icon, Leading, ListItem, TextStyle, app_bar, list_item, palette, pivot, progress_bar, text, u};

use crate::app::App;
use crate::data::{Run, RunPhase, Verdict, od_series, run_log, to_fixed};
use crate::nav::{Dialog, Page};
use crate::pages::{APP, PageView, alarm, fact, half, heading, missing, raw, scrolling, subtle, titled};
use crate::store::{PEAK_OD, Store, growth, is_live};

const FILTERS: [&str; 4] = ["all", "live", "harvested", "failed"];

fn matches(filter: usize, phase: RunPhase) -> bool {
    match filter {
        1 => is_live(phase),
        2 => phase == RunPhase::Harvested,
        3 => phase == RunPhase::Failed,
        _ => true,
    }
}

pub fn phase_color(phase: RunPhase) -> Hsla {
    match phase {
        RunPhase::Lag => Accent::Steel.color(),
        RunPhase::Log => palette().accent,
        RunPhase::Stationary => Accent::Teal.color(),
        RunPhase::Harvested => Accent::Emerald.color(),
        RunPhase::Failed => alarm(),
    }
}

/// A run as a list row, for any page that lists runs.
pub fn run_item(app: App, run: &Run, store: &Store) -> ListItem {
    let strain = store.strain(&run.strain_id).map(|s| format!("{} {}", s.code, s.name)).unwrap_or_default();
    let initial = run.phase.as_str()[..1].to_string();
    let id = run.id.clone();
    list_item(run.code.clone())
        .leading(Leading::Glyph(initial.into()))
        .leading_color(phase_color(run.phase))
        .subtitle(format!("{} · {}", run.phase.as_str(), strain))
        .detail(format!("{} · OD {} · {}", run.vessel, to_fixed(run.od600, 1), run.started))
        .on_click(move || app.push(Page::Run { id: id.clone() }))
}

fn pane(app: App, filter: usize) -> AnyElement {
    let items: Vec<AnyElement> = app.read(|s| {
        s.data.runs.iter().filter(|r| matches(filter, r.phase)).map(|r| run_item(app, r, s).into_element()).collect()
    });
    let empty = items.is_empty();
    scrolling(view().children(empty.then(|| subtle("nothing here right now"))).children(items))
}

pub fn list(app: App, selected: usize) -> PageView {
    let sel = signal(selected);
    let body = pivot(FILTERS, move || sel.get())
        .title(APP)
        .on_select(move |ix| {
            sel.set(ix);
            app.set_selected(ix);
        })
        .panes((0..FILTERS.len()).map(|i| move || pane(app, i)));
    PageView::new(body).app_bar(
        app_bar()
            .button(Icon::Alert, "incident", move || app.new_incident(None))
            .button(Icon::Thermometer, "telemetry", move || app.push(Page::Telemetry { zone: 0 }))
            .menu_item("freezer", move || app.push(Page::Freezer))
            .menu_item("deck bookings", move || app.push(Page::Deck { pivot: 0 })),
    )
}

/// Twenty-four bars of OD600 history; redrawn on every tick.
fn growth_curve(app: App, id: String) -> AnyElement {
    dynamic(move || {
        let Some((phase, series)) = app.live(|s| s.run(&id).map(|r| (r.phase, od_series(r)))) else {
            return view().into_element();
        };
        let max = series.iter().copied().fold(PEAK_OD * 0.25, f64::max);
        view()
            .flex_row()
            .items_end()
            .gap(u(3.))
            .h(u(56.))
            .pt(u(8.))
            .children(series.into_iter().map(move |v| {
                view().flex_1().h(relative((v / max) as f32)).bg(move || phase_color(phase))
            }))
            .into_element()
    })
}

pub fn detail(app: App, id: &str) -> PageView {
    let Some(run) = app.read(|s| s.run(id).cloned()) else {
        return missing("runs", "run");
    };
    let (strain, instrument, lots, assays) = app.peek(|s| {
        (
            s.strain(&run.strain_id).map(|st| format!("{} · {}", st.code, st.name)),
            run.instrument_id.as_deref().and_then(|i| s.instrument(i)).map(|i| i.code.clone()),
            run.lot_ids.iter().filter_map(|l| s.lot(l)).map(|l| l.name.clone()).collect::<Vec<_>>(),
            s.assays_for(&run.id).cloned().collect::<Vec<_>>(),
        )
    });
    // Readings that move with the telemetry ticks.
    let live = {
        let id = run.id.clone();
        move |f: fn(&Run) -> String| {
            let id = id.clone();
            move || app.live(|s| s.run(&id).map(f).unwrap_or_default())
        }
    };
    let phase = {
        let id = run.id.clone();
        move || app.live(|s| s.run(&id).map(|r| r.phase).unwrap_or(RunPhase::Failed))
    };
    let growth_now = {
        let id = run.id.clone();
        move || app.live(|s| s.run(&id).map(growth).unwrap_or(0.))
    };

    let summary = view()
        .child(text(TextStyle::ExtraLarge, live(|r| r.phase.as_str().to_string())).color(move || phase_color(phase())))
        .children(strain.map(subtle));

    let growth_block = view()
        .gap(u(6.))
        .child(heading("growth"))
        .child(progress_bar(growth_now))
        .child(subtle(live(|r| format!("OD600 {} of {PEAK_OD:.0} · titer {}", to_fixed(r.od600, 2), r.titer))))
        .child(growth_curve(app, run.id.clone()));

    let dash = || "—".to_string();
    let facts = view()
        .flex_row()
        .flex_wrap()
        .pt(u(12.))
        .child(half(fact("vessel", run.vessel.clone())))
        .child(half(fact("instrument", instrument.unwrap_or_else(dash))))
        .child(half(fact("operator", run.operator.clone())))
        .child(half(fact("started", run.started.clone())))
        .child(half(fact("duration", run.duration.clone())))
        .child(half(fact("charge", run.charge_l.map(|l| format!("{} L", to_fixed(l, 2))).unwrap_or_else(dash))))
        .child(fact("draws", if lots.is_empty() { dash() } else { lots.join(", ") }));

    let assay_block = view().children((!assays.is_empty()).then(|| heading("assays"))).children(assays.into_iter().map(|a| {
        let flagged = a.verdict == Verdict::Flag;
        view()
            .flex_row()
            .justify_between()
            .py(u(4.))
            .child(
                view()
                    .child(text(TextStyle::Normal, a.metric.clone()))
                    .child(subtle(format!("{} · {}", a.kind.as_str(), a.verdict.as_str()))),
            )
            .child(text(TextStyle::Large, a.value.clone()).color(move || if flagged { alarm() } else { palette().foreground }))
    }));

    let log = view().child(heading("log")).child(
        view().p(u(10.)).bg(move || palette().chrome).children(
            run_log(&run)
                .into_iter()
                .map(|l| raw(l, 12.).font_family("Menlo").text_color(move || palette().subtle)),
        ),
    );

    let body = titled(
        "runs",
        run.code.clone(),
        scrolling(view().child(summary).child(growth_block).child(facts).child(assay_block).child(log)),
    );

    let (run_id, code, strain_id) = (run.id.clone(), run.code.clone(), run.strain_id.clone());
    let bar = app_bar();
    let bar = if matches!(run.phase, RunPhase::Log | RunPhase::Stationary) {
        let id = run_id.clone();
        bar.button(Icon::Check, "harvest", move || app.show(Dialog::Harvest { run_id: id.clone() }))
    } else {
        bar.disabled_button(Icon::Check, "harvest")
    };
    let bar = bar
        .button(Icon::Alert, "flag", move || app.new_incident(Some(run_id.clone())))
        .button(Icon::Refresh, "refresh", move || {
            app.tick();
            app.toast("refreshed", Icon::Refresh, "refreshed", "latest readings pulled from the floor");
        })
        .menu_item("open strain", move || app.push(Page::Strain { id: strain_id.clone() }))
        .menu_item("copy run code", move || app.copy(code.clone()));
    PageView::new(body).app_bar(bar)
}
