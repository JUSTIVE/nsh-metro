//! Incidents: a pivot by status, the incident detail, and the form that files
//! a new one.

use nsh_core::prelude::*;
use nsh_metro::{
    Icon, ListItem, TextStyle, app_bar, button, check_box, list_item, list_picker, palette, pivot, text,
    toggle_switch, u,
};

use crate::app::App;
use crate::data::{Incident, IncidentSeverity, IncidentStatus};
use crate::nav::{AREAS, Dialog, Page, SEVERITIES};
use crate::pages::{APP, PageView, alarm, fact, half, heading, missing, runs::run_item, scrolling, subtle, titled, warn};
use crate::store::{IncidentDraft, StoreError};
use crate::widgets::text_box;

const STATUSES: [IncidentStatus; 4] =
    [IncidentStatus::Open, IncidentStatus::Investigating, IncidentStatus::Capa, IncidentStatus::Closed];

pub fn severity_color(severity: IncidentSeverity) -> Hsla {
    match severity {
        IncidentSeverity::Critical => alarm(),
        IncidentSeverity::Major => warn(),
        IncidentSeverity::Minor => palette().accent,
    }
}

fn incident_item(app: App, incident: &Incident) -> ListItem {
    let id = incident.id.clone();
    list_item(incident.title.clone())
        .leading_icon(Icon::Alert)
        .leading_color(severity_color(incident.severity))
        .subtitle(format!("{} · {} · {}", incident.code, incident.severity.as_str(), incident.area.as_str()))
        .detail(format!("opened {} · {}", incident.opened, incident.owner))
        .on_click(move || app.push(Page::Incident { id: id.clone() }))
}

fn pane(app: App, selected: usize) -> AnyElement {
    let status = STATUSES[selected.min(STATUSES.len() - 1)];
    let items: Vec<AnyElement> = app.read(|s| {
        s.data.incidents.iter().filter(|i| i.status == status).map(|i| incident_item(app, i).into_element()).collect()
    });
    let empty = items.is_empty();
    scrolling(view().children(empty.then(|| subtle("nothing in this state"))).children(items))
}

pub fn list(app: App, selected: usize) -> PageView {
    let sel = signal(selected);
    let body = pivot(STATUSES.map(|s| s.as_str()), move || sel.get())
        .title(APP)
        .on_select(move |ix| {
            sel.set(ix);
            app.set_selected(ix);
        })
        .panes((0..STATUSES.len()).map(|i| move || pane(app, i)));
    PageView::new(body).app_bar(app_bar().button(Icon::Add, "add", move || app.new_incident(None)))
}

pub fn detail(app: App, id: &str) -> PageView {
    let Some(incident) = app.read(|s| s.incident(id).cloned()) else {
        return missing("incidents", "incident");
    };
    let (instrument, lot, linked) = app.peek(|s| {
        (
            incident.instrument_id.as_deref().and_then(|i| s.instrument(i)).map(|i| format!("{} · {}", i.code, i.state.as_str())),
            incident.lot_id.as_deref().and_then(|l| s.lot(l)).map(|l| l.code.clone()),
            incident.run_id.as_deref().and_then(|r| s.run(r)).map(|run| run_item(app, run, s)),
        )
    });

    let severity = incident.severity;
    let summary = view().gap(u(4.)).child(text(TextStyle::Large, incident.title.clone())).child(
        text(TextStyle::Accent, format!("{} · {}", incident.severity.as_str(), incident.status.as_str()))
            .color(move || severity_color(severity)),
    );

    let facts = view()
        .flex_row()
        .flex_wrap()
        .pt(u(14.))
        .child(half(fact("area", incident.area.as_str())))
        .child(half(fact("owner", incident.owner.clone())))
        .child(half(fact("opened", incident.opened.clone())))
        .child(half(fact("age", format!("{} days", incident.age))))
        .children(instrument.map(|i| half(fact("instrument", i))))
        .children(lot.map(|l| half(fact("lot", l))));

    let linked = linked.map(|item| view().child(heading("run")).child(item));

    let timeline = view().child(heading("timeline")).children(incident.timeline.iter().map(|e| {
        view()
            .flex_row()
            .gap(u(12.))
            .py(u(6.))
            .child(view().w(u(3.)).flex_shrink_0().bg(move || palette().accent))
            .child(view().child(text(TextStyle::Normal, e.text.clone())).child(subtle(e.at.clone())))
    }));

    let body = titled(
        "incidents",
        incident.code.clone(),
        scrolling(view().child(summary).child(facts).children(linked).child(timeline)),
    );

    let bar = app_bar();
    let bar = if incident.status == IncidentStatus::Closed {
        bar.disabled_button(Icon::Next, "advance")
    } else {
        let id = incident.id.clone();
        bar.button(Icon::Next, "advance", move || {
            let moved =
                app.mutate(|s| s.advance_incident(&id).map(|status| (s.incident(&id).map(|i| i.code.clone()), status)));
            if let Some((Some(code), status)) = moved {
                app.toast("incident-moved", Icon::Next, code, format!("moved to {}", status.as_str()));
            }
        })
    };
    PageView::new(body).app_bar(bar)
}

/// The new-incident form's state, one signal per field.
#[derive(Clone, Copy)]
struct Form {
    title: Signal<String>,
    area: Signal<usize>,
    severity: Signal<usize>,
    link_run: Signal<bool>,
    run: Signal<usize>,
    quarantine: Signal<bool>,
    page_lead: Signal<bool>,
}

fn save(app: App, form: Form, run_choices: &[String]) {
    let draft = IncidentDraft {
        title: form.title.get_untracked(),
        area: AREAS[form.area.get_untracked()],
        severity: SEVERITIES[form.severity.get_untracked()],
        run_id: if form.link_run.get_untracked() { run_choices.get(form.run.get_untracked()).cloned() } else { None },
        quarantine: form.quarantine.get_untracked(),
        page_lead: form.page_lead.get_untracked(),
    };
    match app.mutate(|s| s.file_incident(draft)) {
        Ok(code) => {
            app.pop_with(|page| {
                if let Page::Incidents { pivot } = page {
                    *pivot = 0;
                }
            });
            app.toast("incident-filed", Icon::Alert, code, "filed · open");
        }
        Err(e) => {
            let title = if e == StoreError::EmptyTitle { "missing title" } else { "can't file this" };
            app.show(Dialog::Notice { title: title.into(), message: e.to_string() });
        }
    }
}

/// A blank form, or one pre-linked to `run_id` (flagging from a run).
pub fn form(app: App, run_id: Option<String>) -> PageView {
    let mut run_choices: Vec<String> = app.peek(|s| s.live_runs().map(|r| r.id.clone()).collect());
    if let Some(id) = &run_id
        && !run_choices.iter().any(|r| r == id)
    {
        run_choices.insert(0, id.clone());
    }
    let run = run_id.as_ref().and_then(|id| run_choices.iter().position(|r| r == id)).unwrap_or(0);
    let form = Form {
        title: signal(String::new()),
        area: signal(0),
        severity: signal(if run_id.is_some() { 1 } else { 0 }),
        link_run: signal(run_id.is_some()),
        run: signal(run),
        quarantine: signal(false),
        page_lead: signal(false),
    };
    let runs: Vec<String> = app.peek(|s| {
        run_choices.iter().filter_map(|id| s.run(id)).map(|r| format!("{} · {}", r.code, r.phase.as_str())).collect()
    });
    let no_runs = runs.is_empty();
    let choices = std::rc::Rc::new(run_choices);
    let save_now = {
        let choices = choices.clone();
        move || save(app, form, &choices)
    };

    let fields = view()
        .gap(u(18.))
        .child(text_box(form.title, "title", "what happened?", app.editing))
        .child(list_picker(AREAS.map(|a| a.as_str()), form.area).header("area").on_select(move |ix| form.area.set(ix)))
        .child(
            list_picker(SEVERITIES.map(|s| s.as_str()), form.severity)
                .header("severity")
                .on_select(move |ix| form.severity.set(ix)),
        )
        .child(
            view()
                .gap(u(8.))
                .child(
                    toggle_switch(form.link_run)
                        .header("link to a run")
                        .disabled(no_runs)
                        .on_change(move |on| form.link_run.set(on)),
                )
                .child(when(move || form.link_run.get() && !no_runs, move || {
                    list_picker(runs.clone(), form.run).on_select(move |ix| form.run.set(ix))
                })),
        )
        .child(
            view()
                .child(check_box(form.quarantine).label("quarantine the area").on_change(move |on| form.quarantine.set(on)))
                .child(check_box(form.page_lead).label("page the floor lead").on_change(move |on| form.page_lead.set(on))),
        )
        .child(button("save").accent().full_width().on_click(save_now.clone()));

    PageView::new(titled("incidents", "new incident", scrolling(fields))).app_bar(
        app_bar().button(Icon::Save, "save", save_now).button(Icon::Close, "cancel", move || app.pop()),
    )
}

#[cfg(test)]
mod tests {
    use nsh_core::element::{Runtime, create_root};

    use super::*;

    #[test]
    fn save_validates_then_files_and_returns() {
        let runtime = Runtime::default();
        let _entered = runtime.enter();
        let (owner, (app, form)) = create_root(|| {
            let app = App::new(None);
            app.push(Page::Incidents { pivot: 2 });
            app.new_incident(None);
            let form = Form {
                title: signal(String::new()),
                area: signal(0),
                severity: signal(0),
                link_run: signal(false),
                run: signal(0),
                quarantine: signal(true),
                page_lead: signal(false),
            };
            (app, form)
        });
        let open = || app.peek(|s| s.open_incidents().count());
        let before = open();

        save(app, form, &[]);
        assert_eq!(app.depth.get_untracked(), 3, "an empty title keeps the form open");
        assert!(app.dialog.get_untracked().is_some(), "…and explains why");
        app.back();
        assert!(app.dialog.get_untracked().is_none(), "back closes the message box first");
        assert_eq!(app.depth.get_untracked(), 3);

        form.title.set("chiller alarm on rack B".into());
        save(app, form, &[]);
        assert_eq!(app.depth.get_untracked(), 2, "filed, back to where we came from");
        assert_eq!(app.current.get_untracked().0.page, Page::Incidents { pivot: 0 }, "on the open pivot");
        assert_eq!(open(), before + 1);
        assert_eq!(app.peek(|s| s.data.incidents[0].title.clone()), "chiller alarm on rack B");
        owner.dispose();
    }
}
