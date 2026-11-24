//! The context menu (the WP Toolkit's `ContextMenu`): right-click, or press
//! and hold, an element and a chrome panel of lower-case commands swivels
//! open at the pointer over a lightly dimmed page. A command closes the
//! menu and runs; a click outside closes it.

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::{
    button::ClickHandler,
    components::text::label,
    motion::{EXP6_OUT, ENTER, SWIVEL_IN_DEG},
    theme::{FONT_FAMILY, palette, reduced_motion, size, u},
};

struct Entry {
    label: SharedString,
    on_click: Option<ClickHandler>,
}

/// `context_menu().item("pin to start", || ..).item("delete", || ..)`
pub struct ContextMenu {
    entries: Vec<Entry>,
    on_open: Option<Rc<dyn Fn(bool)>>,
}

pub fn context_menu() -> ContextMenu {
    ContextMenu { entries: Vec::new(), on_open: None }
}

impl ContextMenu {
    pub fn item(mut self, label: impl Into<SharedString>, f: impl Fn() + 'static) -> Self {
        self.entries.push(Entry { label: label.into(), on_click: Some(Rc::new(f)) });
        self
    }
    /// A command shown greyed out.
    pub fn disabled_item(mut self, label: impl Into<SharedString>) -> Self {
        self.entries.push(Entry { label: label.into(), on_click: None });
        self
    }
    /// Told when the menu opens (`true`) and closes.
    pub fn on_open(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_open = Some(Rc::new(f));
        self
    }
}

/// Width of the panel.
pub const MENU_W: f32 = 300.;
/// Height of one command.
const ROW_H: f32 = 46.;
const PAD_Y: f32 = 10.;

/// Where the panel goes for a press at `at` in a window `window` big: at
/// the pointer, or flipped left / above it when it would run off an edge.
pub fn place(at: (f32, f32), menu: (f32, f32), window: (f32, f32)) -> (f32, f32) {
    let x = if at.0 + menu.0 > window.0 - 8. { (at.0 - menu.0).max(8.) } else { at.0 };
    let y = if at.1 + menu.1 > window.1 - 8. { (at.1 - menu.1).max(8.) } else { at.1 };
    (x, y)
}

/// Which menu is open in the window, and where: one at a time.
#[derive(Clone, Copy, Default, PartialEq)]
struct OpenMenu(Option<(u64, (f32, f32))>);

thread_local! {
    static NEXT_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

/// `el` with `menu` on right-click and press-and-hold. Opening it closes
/// any other context menu in the window.
pub fn with_context_menu(el: impl IntoElement, menu: ContextMenu) -> AnyElement {
    let id = NEXT_ID.with(|n| n.replace(n.get() + 1));
    let shared = global::<OpenMenu>();
    let open = Opener { id, shared };
    let on_open = menu.on_open.clone();
    effect(move || {
        let o = open.get().is_some();
        if let Some(f) = &on_open {
            untracked(|| f(o));
        }
    });
    let entries = Rc::new(menu.entries);
    let menu_h = entries.len() as f32 * ROW_H + 2. * PAD_Y;
    let window = signal((0f32, 0f32));
    let target = view()
        .child(el)
        .on_context_menu(move |e: &ContextMenuEvent| open.set(Some((f32::from(e.page_position.x), f32::from(e.page_position.y)))))
        .on_long_press(move |e: &PressEvent| open.set(Some((f32::from(e.page_position.x), f32::from(e.page_position.y)))));
    let layer = move || {
        let Some(at) = open.get() else { return view().into_element() };
        let entries = entries.clone();
        let (x, y) = {
            let w = window.get();
            if w.0 > 0. { place(at, (MENU_W, menu_h), w) } else { at }
        };
        let angle = animated_value(if reduced_motion() { 0. } else { -SWIVEL_IN_DEG });
        let fade = animated_value(if reduced_motion() { 1. } else { 0. });
        if !reduced_motion() {
            let _ = angle.timing(0., ENTER, EXP6_OUT);
            let _ = fade.timing(1., ENTER * 0.6, EXP6_OUT);
        }
        let rows = entries.iter().map(|entry| {
            let enabled = entry.on_click.is_some();
            let f = entry.on_click.clone();
            let row = view()
                .h(u(ROW_H))
                .px(u(20.))
                .justify_center()
                .child(
                    label(SharedString::from(entry.label.to_lowercase()))
                        .text_size(u(size::LARGE - 3.))
                        .text_color(move || if enabled { palette().foreground } else { palette().disabled }),
                );
            match f {
                Some(f) => row
                    .cursor_pointer()
                    .hover(|s| s.bg(palette().accent.opacity(0.18)))
                    .active(|s| s.bg(palette().accent))
                    .on_press(move || {
                        open.set(None);
                        f();
                    })
                    .into_element(),
                None => row.into_element(),
            }
        });
        // The panel swivels open about its top edge, out of the page.
        let panel = view()
            .absolute()
            .left(u(x))
            .top(u(y))
            .w(u(MENU_W))
            .perspective(px(crate::motion::PERSPECTIVE))
            .child(
                view()
                    .py(u(PAD_Y))
                    .bg(move || palette().chrome)
                    .border(px(2.))
                    .border_color(move || palette().border.opacity(0.35))
                    .font_family(FONT_FAMILY)
                    .transform_origin(nsh_core::element::TransformOrigin {
                        x: relative(0.5).into(),
                        y: px(0.).into(),
                        z: Points::ZERO,
                    })
                    .bind_transform(nsh_core::TransformKind::RotateX, &angle)
                    .bind_opacity(&fade)
                    .on_press(|| {})
                    .children(rows.collect::<Vec<_>>()),
            );
        // The overlay is the window's size only when told to cover it.
        overlay()
            .absolute()
            .inset_0()
            .child(
                view()
                    .absolute()
                    .inset_0()
                    .bg(move || palette().background.opacity(0.35))
                    .bind_opacity(&fade)
                    .on_layout(move |b: &Bounds<Points>| window.set((f32::from(b.size.width), f32::from(b.size.height))))
                    .on_press(move || open.set(None))
                    .on_context_menu(move |_: &ContextMenuEvent| open.set(None)),
            )
            .child(panel)
            .into_element()
    };
    view().child(target).child(dynamic(layer)).into_element()
}

/// This menu's view of the window's one open menu.
#[derive(Clone, Copy)]
struct Opener {
    id: u64,
    shared: Signal<OpenMenu>,
}

impl Opener {
    /// Where this menu is open (tracked), if it is the one open.
    fn get(&self) -> Option<(f32, f32)> {
        match self.shared.get().0 {
            Some((id, at)) if id == self.id => Some(at),
            _ => None,
        }
    }
    fn set(&self, at: Option<(f32, f32)>) {
        let id = self.id;
        match at {
            Some(at) => self.shared.set(OpenMenu(Some((id, at)))),
            // Closing closes this menu only, not one opened since.
            None => self.shared.update(|m| {
                if m.0.is_some_and(|(open, _)| open == id) {
                    m.0 = None;
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_opens_at_the_pointer_and_flips_at_the_edges() {
        let w = (1000., 800.);
        assert_eq!(place((100., 100.), (300., 200.), w), (100., 100.));
        assert_eq!(place((900., 100.), (300., 200.), w), (600., 100.), "flips left at the right edge");
        assert_eq!(place((100., 700.), (300., 200.), w), (100., 500.), "opens above at the bottom");
    }
}
