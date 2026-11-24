//! Determinate bar, indeterminate dots, and the ring.

use nsh_core::prelude::*;
use nsh_metro::{TextStyle, progress_bar, progress_dots, progress_ring, slider, text, toggle_switch, u};

use super::{DemoState, caption, page, stack};

pub fn bar(s: DemoState) -> AnyElement {
    page(
        "progress bar",
        stack()
            .child(text(TextStyle::Normal, move || {
                format!("downloading updates · {} of 7", (s.progress.get() * 7.).round() as u32)
            }))
            .child(progress_bar(s.progress))
            .child(text(TextStyle::Subtle, move || format!("{:.0}%", s.progress.get() * 100.)))
            .child(slider(s.progress).header("drive the bar").on_change(move |v| s.progress.set(v)))
            .child(caption("empty and full"))
            .child(progress_bar(0.))
            .child(progress_bar(1.)),
    )
}

pub fn dots(s: DemoState) -> AnyElement {
    let feed = view()
        .gap(u(6.))
        .child(text(TextStyle::Large, "what's new"))
        .child(text(TextStyle::Subtle, move || if s.loading.get() { "fetching your feed…" } else { "up to date" }));
    page(
        "progress dots",
        stack()
            .child(when(s.loading, progress_dots).otherwise(|| view().h(u(4.))))
            .child(feed)
            .child(toggle_switch(s.loading).header("loading").on_change(move |v| s.loading.set(v)))
            .child(caption("the caller decides when the dots show")),
    )
}

pub fn ring() -> AnyElement {
    let sizes = [24., 48., 96.];
    page(
        "progress ring",
        stack()
            .child(caption("24 · 48 · 96 px"))
            .child(view().flex_row().items_center().gap(u(40.)).children(sizes.map(|d| progress_ring().diameter(d))))
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap(u(16.))
                    .pt(u(20.))
                    .child(progress_ring().diameter(32.))
                    .child(text(TextStyle::Normal, "signing in…")),
            ),
    )
}
