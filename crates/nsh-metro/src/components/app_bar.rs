//! The application bar: up to four round icon buttons, a "…" that expands
//! it to show their labels and a menu, and a minimized mode.

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::{
    Icon,
    button::{ClickHandler, round_button},
    motion::EXP6_OUT,
    theme::{FONT_FAMILY, palette, reduced_motion, size, u},
};

struct BarButton {
    icon: Icon,
    label: SharedString,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

struct MenuItem {
    label: SharedString,
    on_click: Option<ClickHandler>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppBarMode {
    Default,
    Minimized,
}

pub struct AppBar {
    buttons: Vec<BarButton>,
    items: Vec<MenuItem>,
    mode: AppBarMode,
    opacity: f32,
}

pub fn app_bar() -> AppBar {
    AppBar { buttons: Vec::new(), items: Vec::new(), mode: AppBarMode::Default, opacity: 1. }
}

impl AppBar {
    pub fn button(mut self, icon: Icon, label: impl Into<SharedString>, f: impl Fn() + 'static) -> Self {
        self.buttons.push(BarButton { icon, label: label.into(), disabled: false, on_click: Some(Rc::new(f)) });
        self
    }
    pub fn disabled_button(mut self, icon: Icon, label: impl Into<SharedString>) -> Self {
        self.buttons.push(BarButton { icon, label: label.into(), disabled: true, on_click: None });
        self
    }
    pub fn menu_item(mut self, label: impl Into<SharedString>, f: impl Fn() + 'static) -> Self {
        self.items.push(MenuItem { label: label.into(), on_click: Some(Rc::new(f)) });
        self
    }
    pub fn mode(mut self, mode: AppBarMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0., 1.);
        self
    }
}

/// The phone allowed four buttons.
pub const MAX_BUTTONS: usize = 4;

impl IntoElement for AppBar {
    fn into_element(self) -> AnyElement {
        let open = signal(false);
        let minimized_mode = self.mode == AppBarMode::Minimized;
        let opacity = self.opacity;
        // Opening lifts the bar's content in and brightens it.
        let lift = animated_value(0.);
        effect(move || {
            if open.get() && !untracked(reduced_motion) {
                lift.set(1.);
                let _ = lift.timing(0., 260., EXP6_OUT);
            }
        });
        let buttons = Rc::new(self.buttons.into_iter().take(MAX_BUTTONS).collect::<Vec<_>>());
        let items = Rc::new(self.items);
        let body = move || {
            let expanded = open.get();
            let minimized = minimized_mode && !expanded;
            let row = view()
                .flex_row()
                .justify_center()
                .pt(u(8.))
                .pb(u(if expanded { 4. } else { 10. }))
                .children(buttons.iter().map(|b| {
                    let mut rb = round_button(b.icon).diameter(44.).disabled(b.disabled);
                    if let Some(f) = b.on_click.clone() {
                        rb = rb.on_click(move || {
                            open.set(false);
                            f()
                        });
                    }
                    let disabled = b.disabled;
                    view().items_center().w(u(76.)).gap(u(4.)).child(rb).children(expanded.then(|| {
                        text(SharedString::from(b.label.to_lowercase()))
                            .font_family(FONT_FAMILY)
                            .text_size(u(size::SMALL - 2.))
                            .text_color(move || if disabled { palette().disabled } else { palette().foreground })
                    }))
                }));
            let menu = (expanded && !items.is_empty()).then(|| {
                view().pt(u(10.)).pb(u(12.)).px(u(24.)).gap(u(14.)).children(items.iter().map(|item| {
                    let f = item.on_click.clone();
                    view()
                        .cursor_pointer()
                        .active(|s| s.opacity(0.6))
                        .on_press(move || {
                            open.set(false);
                            if let Some(f) = &f {
                                f();
                            }
                        })
                        .child(
                            text(SharedString::from(item.label.to_lowercase()))
                                .font_family(FONT_FAMILY)
                                .text_size(u(size::MEDIUM_LARGE))
                                .text_color(move || palette().foreground),
                        )
                }))
            });
            let el = view().bind_translate_y(&lift.mul(24.)).bind_opacity(&lift.mul(-0.6).add(1.));
            let el = if minimized { el.h(u(30.)) } else { el.child(row) };
            el.children(menu).into_element()
        };
        let toggle = view()
            .absolute()
            .top_0()
            .right_0()
            .w(u(56.))
            .h(move || u(if minimized_mode && !open.get() { 30. } else { 36. }))
            .items_center()
            .pt(u(2.))
            .cursor_pointer()
            .on_press(move || open.update(|o| *o = !*o))
            .child(Icon::More.render(26., move || palette().foreground));
        view()
            .relative()
            .w_full()
            .flex_shrink_0()
            .bg(move || palette().chrome.opacity(opacity))
            .child(dynamic(body))
            .child(toggle)
            .into_element()
    }
}
