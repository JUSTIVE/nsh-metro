//! The catalog window: top bar, sidebar index, and the selected page with
//! its phone frame and usage snippet. All demo state lives in
//! [`DemoState`], created once, so it survives browsing.

use nsh_core::prelude::*;
use nsh_metro::{
    Accent, Icon, Mode, TextStyle, Theme, check_box,
    motion::{Direction, turnstile_in},
    palette, text, text_box,
    theme::{FONT_FAMILY, LIGHT, SEMILIGHT, size},
    u,
};

use crate::{
    demos::DemoState,
    pages::{Group, Page},
    phone,
};

/// Height of the top bar, in px.
const TOP_BAR: f32 = 64.;
/// Width of the sidebar, in px.
const SIDEBAR: f32 = 250.;
/// The main column's side padding.
const MAIN_PAD: f32 = 40.;

const MONO: &str = "Menlo";

/// A run of plain text in the Metro face.
fn label(content: impl IntoReactive<SharedString>, size: f32) -> Text {
    nsh_core::prelude::text(content).font_family(FONT_FAMILY).text_size(u(size))
}

pub fn catalog(page: Signal<Page>, demo: DemoState) -> AnyElement {
    view()
        .size_full()
        .bg(move || palette().background)
        .child(top_bar())
        .child(
            view()
                .flex_row()
                .flex_1()
                .min_h(u(0.))
                .child(sidebar(page))
                .child(main_area(page, demo)),
        )
        .into_element()
}

fn top_bar() -> AnyElement {
    let ground = |m: Mode| {
        label(m.label(), size::MEDIUM)
            .px(u(6.))
            .text_color(move || if Theme::get().mode == m { palette().foreground } else { palette().subtle })
            .cursor_pointer()
            .hover(|s| s.opacity(0.7))
            .on_press(move || Theme::set_mode(m))
    };
    let swatches = Accent::ALL.into_iter().map(|a| {
        view()
            .size(u(22.))
            .items_center()
            .justify_center()
            .bg(a.color())
            .border(px(2.))
            .border_color(move || if Theme::get().accent == a { palette().foreground } else { a.color() })
            .cursor_pointer()
            .hover(|s| s.opacity(0.8))
            .on_press(move || Theme::set_accent(a))
            .child(when(move || Theme::get().accent == a, move || Icon::Check.render(14., a.ink())))
    });
    view()
        .flex_row()
        .flex_none()
        .items_center()
        .gap(u(28.))
        .w_full()
        .h(u(TOP_BAR))
        .px(u(24.))
        .border_b_1()
        .border_color(move || palette().disabled.opacity(0.3))
        .child(
            view()
                .flex_row()
                .items_baseline()
                .gap(u(10.))
                .child(
                    label("metro", size::EXTRA_LARGE)
                        .font_weight(LIGHT)
                        .text_color(move || palette().foreground),
                )
                .child(text(TextStyle::Subtle, "nsh component catalog")),
        )
        .child(view().flex_1())
        .child(
            view()
                .flex_row()
                .items_center()
                .child(ground(Mode::Dark))
                .child(label("/", size::MEDIUM).text_color(move || palette().disabled))
                .child(ground(Mode::Light)),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(u(10.))
                .child(text(TextStyle::Small, move || Theme::get().accent.name()))
                .child(view().flex_row().gap(u(3.)).children(swatches)),
        )
        .child(
            check_box(move || Theme::get().reduced_motion)
                .label("reduced motion")
                .on_change(Theme::set_reduced_motion),
        )
        .into_element()
}

/// Pages whose name or group contains `query` (case-insensitive), by group.
fn matches(query: &str) -> Vec<(Group, Vec<Page>)> {
    let q = query.trim().to_lowercase();
    Group::ALL
        .into_iter()
        .filter_map(|g| {
            let pages: Vec<Page> = Page::ALL
                .into_iter()
                .filter(|p| p.group() == g && (q.is_empty() || p.name().contains(&q) || g.name().contains(&q)))
                .collect();
            (!pages.is_empty()).then_some((g, pages))
        })
        .collect()
}

fn sidebar(page: Signal<Page>) -> AnyElement {
    let query = signal(String::new());
    let index = move || {
        let groups = matches(&query.get());
        if groups.is_empty() {
            return text(TextStyle::Subtle, "no components match").into_element();
        }
        view()
            .children(groups.into_iter().map(|(g, pages)| {
                view().pb(u(14.)).child(view().pb(u(4.)).child(text(TextStyle::GroupHeader, g.name()))).children(
                    pages.into_iter().map(move |p| {
                        label(p.name(), size::MEDIUM_LARGE)
                            .font_weight(SEMILIGHT)
                            .py(u(3.))
                            .text_color(move || if page.get() == p { palette().accent } else { palette().foreground })
                            .cursor_pointer()
                            .hover(|s| s.opacity(0.65))
                            .on_press(move || page.set(p))
                    }),
                )
            }))
            .into_element()
    };
    view()
        .flex_none()
        .w(u(SIDEBAR))
        .h_full()
        .border_r_1()
        .border_color(move || palette().disabled.opacity(0.3))
        .child(view().px(u(20.)).pt(u(18.)).pb(u(12.)).child(text_box(query).placeholder("search components")))
        .child(scroll().flex_1().min_h(u(0.)).child(view().px(u(20.)).pb(u(24.)).child(dynamic(index))))
        .into_element()
}

fn heading(page: Page) -> AnyElement {
    view()
        .gap(u(6.))
        .child(text(TextStyle::AppTitle, page.group().name()).color(move || palette().accent))
        .child(
            label(page.name(), size::HUGE)
                .font_weight(LIGHT)
                .line_height(nsh_core::LineHeight::Points(px(64.)))
                .text_color(move || palette().foreground),
        )
        .child(
            label(page.description(), size::NORMAL)
                .max_w(u(860.))
                .line_height(nsh_core::LineHeight::Points(px(24.)))
                .text_color(move || palette().subtle),
        )
        .into_element()
}

fn snippet(page: Page) -> AnyElement {
    view()
        .gap(u(10.))
        .child(text(TextStyle::GroupHeader, "usage"))
        .child(
            view().p(u(18.)).bg(move || palette().chrome).children(page.snippet().lines().map(|line| {
                nsh_core::prelude::text(line.to_string())
                    .h(u(20.))
                    .font_family(MONO)
                    .text_size(u(13.))
                    .text_color(move || palette().foreground)
            })),
        )
        .into_element()
}

fn main_area(page: Signal<Page>, demo: DemoState) -> AnyElement {
    let screen = phone::frame(page, demo);
    scroll()
        .flex_1()
        .min_w(u(0.))
        .h_full()
        .child(
            view()
                .gap(u(28.))
                .px(u(MAIN_PAD))
                .pt(u(28.))
                .pb(u(48.))
                // The heading turnstiles in on each page change.
                .child(dynamic(move || {
                    let p = page.get();
                    turnstile_in(Direction::Forward, 0., MAIN_PAD, heading(p))
                }))
                .child(
                    view()
                        .flex_row()
                        .flex_wrap()
                        .items_start()
                        .gap(u(36.))
                        .child(screen)
                        .child(
                            view()
                                .flex_1()
                                .min_w(u(260.))
                                .max_w(u(520.))
                                .child(dynamic(move || snippet(page.get()))),
                        ),
                ),
        )
        .into_element()
}
