//! System tray, page header, and Metro's motions.

use nsh_core::prelude::*;
use nsh_metro::label;
use nsh_metro::{
    Icon, Leading, TextStyle, TileSize, list_item,
    motion::{Direction, FEATHER, PaneSlide, swivel_in, turnstile_in},
    page_header, palette, round_button, slider, system_tray, text, tile, toggle_switch, u,
};

use super::{DemoState, caption, page, stack};

/// A component shown inside an outlined box, as a specimen.
fn specimen(child: impl IntoElement) -> View {
    view().border_1().border_color(move || palette().disabled).child(child)
}

pub fn tray_page(s: DemoState) -> AnyElement {
    page(
        "system tray",
        stack()
            .child(caption("these drive the tray at the top of this phone"))
            .child(
                view()
                    .child(
                        label(move || format!("battery · {:.0}%", s.battery.get() * 100.))
                            .font_family(nsh_metro::theme::FONT_FAMILY)
                            .text_size(u(nsh_metro::theme::size::NORMAL))
                            .text_color(move || palette().subtle),
                    )
                    .child(slider(s.battery).on_change(move |v| s.battery.set(v))),
            )
            .child(
                toggle_switch(s.tray_opaque)
                    .header("opaque tray")
                    .labels("Opaque", "Transparent")
                    .on_change(move |v| s.tray_opaque.set(v)),
            )
            .child(caption("specimens"))
            .child(specimen(system_tray("9:41").battery(0.15)))
            .child(specimen(system_tray("23:59").battery(1.).opaque())),
    )
}

pub fn header_page() -> AnyElement {
    view()
        .size_full()
        .child(page_header("wi-fi").app_title("settings"))
        .child(page_header("inbox").app_title("outlook"))
        .child(page_header("no app title"))
        .into_element()
}

const ROWS: [(&str, &str, Icon); 2] = [("mail", "4 unread", Icon::Mail), ("calendar", "dentist at 3:30", Icon::Calendar)];

/// One original Metro motion: its name, what it is for, and the specimen.
fn motion_row(name: &'static str, use_: &'static str, specimen: impl IntoElement) -> View {
    view()
        .gap(u(8.))
        .child(text(TextStyle::GroupHeader, name))
        .child(caption(use_))
        .child(
            view()
                .relative()
                .h(u(150.))
                .overflow_hidden()
                .border_1()
                .border_color(move || palette().disabled.opacity(0.5))
                .child(specimen),
        )
}

/// A specimen built again (so replayed) on every beat of the motion loop.
fn looping<E: IntoElement>(s: DemoState, specimen: impl Fn() -> E + 'static) -> AnyElement {
    dynamic(move || {
        s.motion_loop.get();
        untracked(|| specimen().into_element())
    })
}

/// The specimen rows; with `feather`, each swings in after the one above.
fn rows(feather: bool) -> View {
    view().px(u(12.)).children(ROWS.into_iter().enumerate().map(|(ix, (title, sub, icon))| {
        let item = list_item(title).subtitle(sub).leading(Leading::Icon(icon));
        if feather {
            turnstile_in(Direction::Forward, (ix + 1) as f32 * FEATHER, 12., item).into_element()
        } else {
            item.into_element()
        }
    }))
}

fn inbox() -> AnyElement {
    view().px(u(12.)).child(text(TextStyle::Title2, "inbox").lowercase()).into_element()
}

/// Metro's own motions, and nothing else: each plays on its own, on a loop.
pub fn motion_page(s: DemoState) -> AnyElement {
    let turnstile = || turnstile_in(Direction::Forward, 0., 0., view().size_full().child(inbox()).child(rows(false)));
    let feather = || {
        view()
            .size_full()
            .child(turnstile_in(Direction::Forward, 0., 0., inbox()))
            .child(rows(true))
    };
    let swivel = || {
        swivel_in(
            view()
                .w_full()
                .bg(move || palette().chrome)
                .p(u(14.))
                .child(text(TextStyle::Large, "delete this message?"))
                .child(text(TextStyle::Normal, "it will be gone for good.")),
        )
    };
    let slide = || {
        // List items read the slide from the context and trail it line by line.
        let slide = PaneSlide::new();
        provide_context(slide);
        let pane = view().size_full().bind_translate_x(&slide.offset()).child(rows(false));
        let _ = slide.play(Direction::Forward);
        pane
    };
    let tilt = view()
            .flex_row()
            .gap(u(12.))
            .p(u(12.))
            .child(tile("m-tilt-tile", TileSize::Medium).icon(Icon::Mail).title("mail").on_click(|| {}))
            .child(round_button(Icon::Add).on_click(|| {}));

    page(
        "motion",
        stack()
            .child(motion_row("turnstile", "pages, on navigation: the page swings in about the left edge", looping(s, turnstile)))
            .child(motion_row("turnstile feather", "list pages: the header, then each item, swing in", looping(s, feather)))
            .child(motion_row("swivel", "message boxes and toasts swing in about their middle", looping(s, swivel)))
            .child(motion_row("slide", "pivot items: the pane slides in from the side", looping(s, slide)))
            .child(motion_row("tilt", "press and hold: the element sinks under the pointer", tilt)),
    )
}
