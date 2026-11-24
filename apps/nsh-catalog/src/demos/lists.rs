//! List item variants and the a–z contacts jump list.

use nsh_core::prelude::*;
use nsh_metro::{ALPHABET, Accent, Icon, Leading, ListGroup, jump_list, list_item, page_header, palette, progress_ring, u};

use super::{DemoState, caption, page, stack};

pub fn list_items(s: DemoState) -> AnyElement {
    let chevron = || Icon::ChevronRight.render(22., move || palette().subtle);
    let choices = ["cellular data", "wi-fi", "bluetooth"];
    // The clickable group: rebuilt when the selection moves.
    let selectable = dynamic(move || {
        let selected = s.selected_row.get();
        choices
            .into_iter()
            .enumerate()
            .map(|(ix, title)| {
                let on = selected == Some(ix);
                let item = list_item(title)
                    .subtitle(if on { "selected" } else { "tap to select" })
                    .on_click(move || s.selected_row.set(Some(ix)));
                if on {
                    item.trailing(Icon::Check.render(24., move || palette().accent)).into_element()
                } else {
                    item.subtle_subtitle().into_element()
                }
            })
            .collect::<Vec<_>>()
    });

    page(
        "list item",
        stack()
            .gap(u(4.))
            .child(caption("title only"))
            .child(list_item("ringtones + sounds"))
            .child(caption("accent subtitle · subtle subtitle"))
            .child(list_item("kristin").subtitle("mobile"))
            .child(list_item("lock screen").subtitle("notifications, background").subtle_subtitle())
            .child(caption("third line"))
            .child(list_item("anna").subtitle("photos from the trip").detail("yesterday, 6:12 pm"))
            .child(caption("leading glyph · letters · element · colour"))
            .child(list_item("email + accounts").subtitle("3 accounts").leading_icon(Icon::Mail))
            .child(
                list_item("ben thompson")
                    .subtitle("work")
                    .leading(Leading::Glyph("bt".into()))
                    .leading_color(Accent::Teal.color()),
            )
            .child(
                list_item("backup")
                    .subtitle("in progress")
                    .leading(Leading::Element(progress_ring().diameter(32.).into_element()))
                    .leading_color(palette().chrome),
            )
            .child(
                list_item("battery saver")
                    .subtitle("off")
                    .subtle_subtitle()
                    .leading_icon(Icon::Battery)
                    .leading_color(Accent::Emerald.color()),
            )
            .child(caption("trailing element"))
            .child(list_item("about").subtitle("phone info").subtle_subtitle().trailing(chevron()))
            .child(caption("clickable"))
            .child(selectable),
    )
}

const CONTACTS: [&str; 34] = [
    "Aaron Burr",
    "Ada Lovelace",
    "Alan Turing",
    "Anna Karenina",
    "Barbara Liskov",
    "Ben Thompson",
    "Brian Kernighan",
    "Charles Babbage",
    "Claude Shannon",
    "Dennis Ritchie",
    "Donald Knuth",
    "Edsger Dijkstra",
    "Frances Allen",
    "Grace Hopper",
    "Guido van Rossum",
    "Hedy Lamarr",
    "Ivan Sutherland",
    "John McCarthy",
    "Ken Thompson",
    "Kristin Larsen",
    "Leslie Lamport",
    "Linus Torvalds",
    "Margaret Hamilton",
    "Niklaus Wirth",
    "Radia Perlman",
    "Rob Pike",
    "Robin Milner",
    "Sophie Wilson",
    "Tim Berners-Lee",
    "Tony Hoare",
    "Vint Cerf",
    "Whitfield Diffie",
    "Yukihiro Matsumoto",
    "8 Ball Pool Club",
];

/// Lower-case first letter, or `#` for anything that isn't a letter.
fn key_of(name: &str) -> String {
    match name.chars().next() {
        Some(c) if c.is_alphabetic() => c.to_lowercase().to_string(),
        _ => "#".into(),
    }
}

fn initials(name: &str) -> SharedString {
    name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_lowercase().into()
}

pub fn contacts() -> AnyElement {
    let groups = ALPHABET.into_iter().filter_map(|key| {
        let rows: Vec<AnyElement> = CONTACTS
            .iter()
            .enumerate()
            .filter(|(_, name)| key_of(name) == key)
            .map(|(ix, name)| {
                list_item(*name)
                    .leading(Leading::Glyph(initials(name)))
                    .leading_color(Accent::ALL[ix % Accent::ALL.len()].color())
                    .into_element()
            })
            .collect();
        (!rows.is_empty()).then(|| ListGroup::new(key, rows))
    });
    view()
        .size_full()
        .child(page_header("people").app_title("contacts"))
        .child(view().flex_1().min_h(u(0.)).child(jump_list().keys(ALPHABET).groups(groups)))
        .into_element()
}
