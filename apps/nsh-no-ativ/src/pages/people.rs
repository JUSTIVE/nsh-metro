//! People: the roster, the alumni, and what each role may do.

use nsh_core::prelude::*;
use nsh_metro::{Leading, ListItem, TextStyle, list_item, pivot, text, u};

use crate::app::App;
use crate::data::{Member, ROLES};
use crate::pages::{APP, PageView, scrolling, subtle};

fn initials(name: &str) -> String {
    name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_lowercase()
}

fn role_name(member: &Member) -> &'static str {
    ROLES.iter().find(|r| r.id == member.role).map(|r| r.name).unwrap_or("")
}

fn member_item(member: &Member) -> ListItem {
    let since = match &member.left {
        Some(left) => format!("{} – {}", member.joined, left),
        None => format!("since {}", member.joined),
    };
    list_item(member.name.clone())
        .leading(Leading::Glyph(initials(&member.name).into()))
        .subtitle(format!("{} · {}", role_name(member).to_lowercase(), since))
        .detail(member.email.clone())
}

fn pane(app: App, selected: usize) -> AnyElement {
    let rows: Vec<AnyElement> = app.read(|s| match selected {
        0 | 1 => {
            let alumni = selected == 1;
            s.data.members.iter().filter(|m| m.left.is_some() == alumni).map(|m| member_item(m).into_element()).collect()
        }
        _ => ROLES
            .iter()
            .map(|role| {
                let count = s.data.members.iter().filter(|m| m.role == role.id && m.left.is_none()).count();
                view()
                    .py(u(8.))
                    .child(text(TextStyle::Large, format!("{} · {count}", role.name.to_lowercase())))
                    .child(subtle(role.description))
                    .into_element()
            })
            .collect(),
    });
    scrolling(view().children(rows))
}

pub fn render(app: App, selected: usize) -> PageView {
    let sel = signal(selected);
    PageView::new(
        pivot(["roster", "alumni", "roles"], move || sel.get())
            .title(APP)
            .on_select(move |ix| {
                sel.set(ix);
                app.set_selected(ix);
            })
            .panes((0..3).map(|i| move || pane(app, i))),
    )
}
