//! Strains: an a–z jump list, and the strain detail with its lineage.

use nsh_core::prelude::*;
use nsh_metro::{ALPHABET, Leading, ListGroup, ListItem, jump_list, list_item, page_header, progress_bar, u};

use crate::app::App;
use crate::data::{Strain, StrainStatus};
use crate::nav::Page;
use crate::pages::{APP, PageView, fact, half, heading, missing, runs::run_item, scrolling, subtle, titled};

fn group_key(name: &str) -> String {
    match name.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => c.to_ascii_lowercase().to_string(),
        _ => "#".into(),
    }
}

fn strain_item(app: App, strain: &Strain) -> ListItem {
    let id = strain.id.clone();
    let item = list_item(strain.name.clone())
        .subtitle(format!("{} · {}", strain.code, strain.chassis.as_str()))
        .detail(format!("gen {} · fitness {} · {}", strain.generation, strain.fitness, strain.status.as_str()));
    let item = if strain.status == StrainStatus::Archived { item.subtle_subtitle() } else { item };
    item.on_click(move || app.push(Page::Strain { id: id.clone() }))
}

pub fn list(app: App) -> PageView {
    let groups = app.read(|s| {
        let mut strains: Vec<&Strain> = s.data.strains.iter().collect();
        strains.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.code.cmp(&b.code)));
        let mut groups: Vec<ListGroup> = Vec::new();
        for strain in strains {
            let key = group_key(&strain.name);
            let item = strain_item(app, strain).into_element();
            match groups.last_mut() {
                Some(g) if g.key.as_ref() == key.as_str() => g.items.push(item),
                _ => groups.push(ListGroup::new(key, [item])),
            }
        }
        groups
    });
    PageView::new(
        view()
            .size_full()
            .child(page_header("strains").app_title(APP))
            .child(view().flex_1().min_h(px(0.)).child(jump_list().keys(ALPHABET).groups(groups))),
    )
}

pub fn detail(app: App, id: &str) -> PageView {
    let Some(strain) = app.read(|s| s.strain(id).cloned()) else {
        return missing("strains", "strain");
    };

    let facts = view()
        .flex_row()
        .flex_wrap()
        .child(half(fact("code", strain.code.clone())))
        .child(half(fact("chassis", strain.chassis.as_str())))
        .child(half(fact("status", strain.status.as_str())))
        .child(half(fact("generation", strain.generation.to_string())))
        .child(half(fact("owner", strain.owner.clone())))
        .child(half(fact("updated", strain.updated.clone())));

    let fitness = view()
        .gap(u(6.))
        .child(heading("fitness"))
        .child(progress_bar(strain.fitness as f32 / 100.))
        .child(subtle(format!("{} of 100, growth-normalized", strain.fitness)));

    let (parent, children, constructs, runs, samples) = app.peek(|s| {
        let parent = strain.parent_id.as_deref().and_then(|p| s.strain(p)).map(|p| strain_item(app, p));
        let children: Vec<ListItem> = s.children_of(&strain.id).map(|c| strain_item(app, c)).collect();
        let constructs: Vec<ListItem> = strain
            .construct_ids
            .iter()
            .filter_map(|c| s.construct(c))
            .map(|c| {
                list_item(c.name.clone())
                    .leading(Leading::Glyph("p".into()))
                    .subtitle(format!("{} · {} bp · {} % GC · {}", c.code, c.bp, c.gc, c.status.as_str()))
                    .detail(c.parts.join(" → "))
            })
            .collect();
        let runs: Vec<ListItem> = s.runs_of(&strain.id).take(6).map(|r| run_item(app, r, s)).collect();
        let samples: Vec<String> = s
            .samples_of(&strain.id)
            .take(8)
            .map(|x| format!("{} · {} · frozen {}", x.code, x.form.as_str(), x.frozen_at))
            .collect();
        (parent, children, constructs, runs, samples)
    });

    let lineage = view()
        .child(heading("lineage"))
        .child(match parent {
            Some(p) => p.into_element(),
            None => subtle("founder strain — no parent on record").into_element(),
        })
        .children((!children.is_empty()).then(|| view().pt(u(6.)).child(subtle(format!("passaged into {}", children.len())))))
        .children(children);

    let constructs = view().child(heading("constructs")).children(constructs);

    let history = view()
        .children((!runs.is_empty()).then(|| heading("runs")))
        .children(runs)
        .children((!samples.is_empty()).then(|| heading("banked")))
        .children(samples.into_iter().map(|s| view().py(u(2.)).child(subtle(s))));

    PageView::new(titled(
        "strains",
        strain.name.clone(),
        scrolling(view().child(facts).child(fitness).child(lineage).child(constructs).child(history)),
    ))
}
