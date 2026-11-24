//! Lists: the two- and three-line list item, the jump list with its letter
//! grid, and the list picker.

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::components::text::label;

use crate::{
    Icon,
    button::ClickHandler,
    chrome::PAGE_MARGIN,
    motion::{Direction, FEATHER, slide_line, turnstile_in},
    theme::{FONT_FAMILY, LIGHT, SEMILIGHT, palette, size, u},
};

/// What sits in a list item's leading square. (Built once and moved into
/// the item; the element variant's size doesn't matter.)
#[allow(clippy::large_enum_variant)]
pub enum Leading {
    /// A glyph from the set, 32 px.
    Icon(Icon),
    /// A short text, such as an initial, in large type.
    Glyph(SharedString),
    /// Any element.
    Element(AnyElement),
}

/// A list row: a big title, an accent subtitle and a subtle detail line.
/// Inside a sliding pivot pane its lines trail the pane, line by line.
#[must_use = "a builder does nothing until it is placed in the tree"]
pub struct ListItem {
    title: SharedString,
    subtitle: Option<SharedString>,
    detail: Option<SharedString>,
    subtitle_accent: bool,
    leading: Option<Leading>,
    leading_color: Option<Reactive<Hsla>>,
    trailing: Option<AnyElement>,
    on_click: Option<ClickHandler>,
}

/// A list item with `title` in large semilight type, and nothing else.
pub fn list_item(title: impl Into<SharedString>) -> ListItem {
    ListItem {
        title: title.into(),
        subtitle: None,
        detail: None,
        subtitle_accent: true,
        leading: None,
        leading_color: None,
        trailing: None,
        on_click: None,
    }
}

impl ListItem {
    /// The second line, in the accent (or subtle, with [`ListItem::subtle_subtitle`]).
    pub fn subtitle(mut self, s: impl Into<SharedString>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    /// The third line, in the subtle colour.
    pub fn detail(mut self, s: impl Into<SharedString>) -> Self {
        self.detail = Some(s.into());
        self
    }
    /// Sets the subtitle in the subtle colour instead of the accent.
    pub fn subtle_subtitle(mut self) -> Self {
        self.subtitle_accent = false;
        self
    }
    /// A 62 px square before the text, filled with the accent (or
    /// [`ListItem::leading_color`]).
    pub fn leading(mut self, leading: Leading) -> Self {
        self.leading = Some(leading);
        self
    }
    /// Shorthand for a [`Leading::Icon`].
    pub fn leading_icon(self, icon: Icon) -> Self {
        self.leading(Leading::Icon(icon))
    }
    /// The leading square's fill, instead of the accent.
    pub fn leading_color(mut self, c: impl IntoReactive<Hsla>) -> Self {
        self.leading_color = Some(c.into_reactive());
        self
    }
    /// An element at the end of the row, such as a count or a chevron.
    pub fn trailing(mut self, e: impl IntoElement) -> Self {
        self.trailing = Some(e.into_element());
        self
    }
    /// Runs `f` on a tap; the row dims while pressed.
    pub fn on_click(mut self, f: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl IntoElement for ListItem {
    fn into_element(self) -> AnyElement {
        let ink = move || palette().accent_ink;
        let leading_color = self.leading_color;
        let leading = self.leading.map(|l| {
            view()
                .flex_shrink_0()
                .items_center()
                .justify_center()
                .size(u(62.))
                .bg({
                    let lc = leading_color.clone();
                    move || match &lc {
                        Some(c) => c.get(),
                        None => palette().accent,
                    }
                })
                .child(match l {
                    Leading::Icon(i) => i.render(32., ink),
                    Leading::Glyph(g) => {
                        label(g).font_family(FONT_FAMILY).text_size(u(size::LARGE)).text_color(ink).into_element()
                    }
                    Leading::Element(e) => e,
                })
        });
        let accent = self.subtitle_accent;
        let line = |s: SharedString, color: Rc<dyn Fn() -> Hsla>| {
            label(s).font_family(FONT_FAMILY).text_size(u(size::NORMAL)).truncate().text_color(move || color())
        };
        let body = view()
            .flex_1()
            .min_w(px(0.))
            .child(
                label(self.title)
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::LARGE + 3.))
                    .font_weight(SEMILIGHT)
                    .line_height(nsh_core::LineHeight::Points(px(34.)))
                    .truncate()
                    .text_color(move || palette().foreground),
            )
            .children(self.subtitle.map(|s| {
                slide_line(1, line(s, Rc::new(move || if accent { palette().accent } else { palette().subtle })))
            }))
            .children(self.detail.map(|s| slide_line(2, line(s, Rc::new(move || palette().subtle)))));
        let row = view()
            .flex_row()
            .items_center()
            .gap(u(14.))
            .py(u(8.))
            .children(leading)
            .child(body)
            .children(self.trailing);
        match self.on_click {
            Some(f) => row.cursor_pointer().active(|s| s.opacity(0.6)).on_press(move || f()).into_element(),
            None => row.into_element(),
        }
    }
}

/// A run of items under one key (a letter).
pub struct ListGroup {
    /// The key shown on the group's header square and in the jump grid.
    key: SharedString,
    /// The group's rows.
    items: Vec<AnyElement>,
}

impl ListGroup {
    /// A group of `items` under `key`.
    pub fn new(key: impl Into<SharedString>, items: impl IntoIterator<Item = AnyElement>) -> Self {
        Self { key: key.into(), items: items.into_iter().collect() }
    }
    /// The group's key.
    pub fn key(&self) -> &SharedString {
        &self.key
    }
    /// Adds a row to the end of the group.
    pub fn push(&mut self, item: impl IntoElement) {
        self.items.push(item.into_element());
    }
}

/// The keys of a people-hub jump grid.
pub const ALPHABET: [&str; 27] = [
    "#", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v",
    "w", "x", "y", "z",
];

/// A grouped list whose group headers open a grid of keys; a key jumps to
/// its group.
#[must_use = "a builder does nothing until it is placed in the tree"]
pub struct JumpList {
    groups: Vec<ListGroup>,
    keys: Option<Vec<SharedString>>,
}

/// An empty jump list.
pub fn jump_list() -> JumpList {
    JumpList { groups: Vec::new(), keys: None }
}

impl JumpList {
    /// Adds a group, after those already there.
    pub fn group(mut self, group: ListGroup) -> Self {
        self.groups.push(group);
        self
    }
    /// Adds several groups, in order.
    pub fn groups(mut self, groups: impl IntoIterator<Item = ListGroup>) -> Self {
        self.groups.extend(groups);
        self
    }
    /// The keys the jump grid shows, in order (such as [`ALPHABET`]); a key
    /// with no group is greyed out. Defaults to the groups' own keys.
    pub fn keys<S: Into<SharedString>>(mut self, keys: impl IntoIterator<Item = S>) -> Self {
        self.keys = Some(keys.into_iter().map(Into::into).collect());
        self
    }
}

impl IntoElement for JumpList {
    fn into_element(self) -> AnyElement {
        let grid_open = signal(false);
        let group_keys: Vec<SharedString> = self.groups.iter().map(|g| g.key.clone()).collect();
        let keys = self.keys.unwrap_or_else(|| group_keys.clone());
        let refs: Vec<ElementRef> = self.groups.iter().map(|_| element_ref()).collect();
        let header = |key: SharedString, r: &ElementRef| {
            view()
                .element_ref(r)
                .justify_end()
                .size(u(52.))
                .p(u(6.))
                .mt(u(14.))
                .mb(u(6.))
                .bg(move || palette().accent)
                .cursor_pointer()
                .on_press(move || grid_open.set(true))
                .child(
                    label(key)
                        .font_family(FONT_FAMILY)
                        .text_size(u(size::LARGE + 3.))
                        .text_color(move || palette().accent_ink),
                )
        };
        let list = scroll().size_full().child(
            view()
                .px(u(PAGE_MARGIN))
                .children(
                    self.groups.into_iter().zip(&refs).map(|(g, r)| view().child(header(g.key, r)).children(g.items)),
                )
                .child(view().h(u(80.))),
        );
        let refs = Rc::new(refs);
        let grid = move || {
            let cells = keys.iter().cloned().enumerate().map(|(ix, key)| {
                let target = group_keys.iter().position(|k| k == &key);
                let enabled = target.is_some();
                let refs = refs.clone();
                let cell = view()
                    .justify_end()
                    .size(u(96.))
                    .p(u(8.))
                    .bg(move || if enabled { palette().accent } else { palette().chrome })
                    .child(
                        label(key)
                            .font_family(FONT_FAMILY)
                            .text_size(u(size::EXTRA_LARGE + 6.))
                            .font_weight(LIGHT)
                            .text_color(move || if enabled { palette().accent_ink } else { palette().disabled }),
                    );
                let cell = match target {
                    Some(t) => cell.cursor_pointer().on_press(move || {
                        let mut o = RevealOptions::default();
                        o.animated = false;
                        refs[t].reveal(o);
                        grid_open.set(false);
                    }),
                    None => cell,
                };
                // The keys swing in one after another.
                turnstile_in(Direction::Forward, ix.min(12) as f32 * FEATHER, 0., cell)
            });
            view()
                .absolute()
                .inset_0()
                .bg(move || palette().background.opacity(0.92))
                .on_press(move || grid_open.set(false))
                .child(
                    view()
                        .flex_row()
                        .flex_wrap()
                        .gap(u(10.))
                        .p(u(PAGE_MARGIN))
                        .pt(u(40.))
                        .children(cells.collect::<Vec<_>>()),
                )
        };
        view().relative().size_full().child(list).child(when(move || grid_open.get(), grid)).into_element()
    }
}

/// A field that opens in place into its list of choices.
#[must_use = "a builder does nothing until it is placed in the tree"]
pub struct ListPicker {
    header: Option<SharedString>,
    items: Vec<SharedString>,
    selected: Reactive<usize>,
    on_select: Option<Rc<dyn Fn(usize)>>,
}

/// A list picker of `items` showing the one at `selected` (a value, or a
/// signal or closure).
pub fn list_picker<S: Into<SharedString>>(
    items: impl IntoIterator<Item = S>,
    selected: impl IntoReactive<usize>,
) -> ListPicker {
    ListPicker {
        header: None,
        items: items.into_iter().map(Into::into).collect(),
        selected: selected.into_reactive(),
        on_select: None,
    }
}

impl ListPicker {
    /// The caption above the field, in the subtle colour.
    pub fn header(mut self, h: impl Into<SharedString>) -> Self {
        self.header = Some(h.into());
        self
    }
    /// Runs `f` with the index chosen; the picker closes. Set `selected` from it.
    pub fn on_select(mut self, f: impl Fn(usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl IntoElement for ListPicker {
    fn into_element(self) -> AnyElement {
        let open = signal(false);
        let items = Rc::new(self.items);
        let selected = self.selected;
        let on_select = self.on_select;
        let field = {
            let items = items.clone();
            let selected = selected.clone();
            move || {
                let sel = selected.get().min(items.len().saturating_sub(1));
                if open.get() {
                    view()
                        .children(items.iter().cloned().enumerate().map(|(ix, item)| {
                            let f = on_select.clone();
                            view()
                                .px(u(10.))
                                .py(u(6.))
                                .cursor_pointer()
                                .on_press(move || {
                                    open.set(false);
                                    if let Some(f) = &f {
                                        f(ix);
                                    }
                                })
                                .child(label(item).text_color(move || {
                                    if ix == sel { palette().accent } else { palette().foreground }
                                }))
                        }))
                        .into_element()
                } else {
                    view()
                        .px(u(10.))
                        .py(u(6.))
                        .cursor_pointer()
                        .on_press(move || open.set(true))
                        .child(
                            label(items.get(sel).cloned().unwrap_or_default()).text_color(move || palette().foreground),
                        )
                        .into_element()
                }
            }
        };
        view()
            .gap(u(4.))
            .children(self.header.map(|h| {
                label(h).font_family(FONT_FAMILY).text_size(u(size::NORMAL)).text_color(move || palette().subtle)
            }))
            .child(
                view()
                    .border(px(2.))
                    .border_color(move || if open.get() { palette().accent } else { palette().border })
                    .bg(move || if open.get() { palette().background } else { palette().input.opacity(0.2) })
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::MEDIUM))
                    .py(u(2.))
                    .child(dynamic(field)),
            )
            .into_element()
    }
}
