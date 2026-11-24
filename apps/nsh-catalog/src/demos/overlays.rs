//! Message box, toast and settings flyout, all drawn over the phone screen.
//! Each page has a body (inside the page host) and an overlay (a child of
//! the screen, over the tray too).

use std::time::Duration;

use nsh_core::prelude::*;
use nsh_metro::{
    FlyoutWidth, Icon, TextStyle, button, check_box, message_box, settings_flyout, slider, text, toast, toggle_switch,
};

use super::{DemoState, caption, page, stack};

/// How long a toast stays up.
pub const TOAST_FOR: Duration = Duration::from_secs(3);

pub fn message_box_page(s: DemoState) -> AnyElement {
    page(
        "message box",
        stack()
            .child(text(TextStyle::Normal, "Remove a saved network from this phone."))
            .child(view().flex_row().child(button("forget network").on_click(move || s.message_open.set(true))))
            .child(text(TextStyle::Accent, move || s.message_result.get()))
            .child(caption(move || if s.dont_ask.get() { "\"don't ask again\" is ticked" } else { "will ask every time" })),
    )
}

pub fn message_box_overlay(s: DemoState) -> AnyElement {
    let answer = move |result: &'static str| {
        move || {
            s.message_result.set(result);
            s.message_open.set(false);
        }
    };
    when(s.message_open, move || {
        message_box("forget home-5g?")
            .message("You'll need the password to join this network again.")
            .child(check_box(s.dont_ask).label("don't ask again").on_change(move |v| s.dont_ask.set(v)))
            .button("forget", answer("forgot home-5g"))
            .button("cancel", answer("kept home-5g"))
            .on_dismiss(answer("dismissed by tapping outside"))
    })
    .into_element()
}

pub fn toast_page(s: DemoState) -> AnyElement {
    let send = move || {
        s.toasts_sent.update(|n| *n += 1);
        s.toast.set(Some(s.toasts_sent.get_untracked()));
    };
    page(
        "toast",
        stack()
            .child(text(TextStyle::Normal, "Toasts interrupt from the top edge and leave on their own."))
            .child(view().flex_row().child(button("send a toast").on_click(send)))
            .child(text(TextStyle::Accent, move || s.toast_log.get()))
            .child(caption(move || {
                format!("{} sent · auto-dismiss after {} s · tap to open", s.toasts_sent.get(), TOAST_FOR.as_secs())
            })),
    )
}

pub fn toast_overlay(s: DemoState) -> AnyElement {
    // One kind of toast: a newer one replaces it (and swivels in again).
    dynamic(move || match s.toast.get() {
        Some(n) => toast("Kristin")
            .icon(Icon::Mail)
            .message(format!("are we still on for tonight? ({n})"))
            .on_click(move || {
                s.toast.set(None);
                s.toast_log.set(format!("opened message {n}"));
            })
            .into_element(),
        None => view().into_element(),
    })
}

pub fn flyout_page(s: DemoState) -> AnyElement {
    page(
        "settings flyout",
        stack()
            .child(text(TextStyle::Normal, "Settings live in a panel that slides in from the right edge."))
            .child(view().flex_row().child(button("options").on_click(move || {
                s.flyout_open.set(true);
                s.flyout_log.set("open");
            })))
            .child(check_box(s.flyout_wide).label("wide (646 px)").on_change(move |v| s.flyout_wide.set(v)))
            .child(text(TextStyle::Accent, move || s.flyout_log.get()))
            .child(caption(move || {
                format!(
                    "sync {} · volume {:.0} %",
                    if s.flyout_sync.get() { "on" } else { "off" },
                    s.flyout_volume.get() * 100.
                )
            })),
    )
}

pub fn flyout_overlay(s: DemoState) -> AnyElement {
    let close = move |how: &'static str| {
        move || {
            s.flyout_open.set(false);
            s.flyout_log.set(how);
        }
    };
    // Built once per width: it slides itself in and out as `open` changes.
    dynamic(move || {
        let width = if s.flyout_wide.get() { FlyoutWidth::Wide } else { FlyoutWidth::Narrow };
        settings_flyout(s.flyout_open)
            .title("options")
            .width(width)
            .child(move || {
                toggle_switch(s.flyout_sync).header("sync settings").on_change(move |v| s.flyout_sync.set(v))
            })
            .child(move || slider(s.flyout_volume).header("volume").on_change(move |v| s.flyout_volume.set(v)))
            .child(|| text(TextStyle::Subtle, "changes apply at once; there is no save button in a flyout."))
            .on_back(close("closed with back"))
            .on_dismiss(close("dismissed by tapping outside"))
            .into_element()
    })
    .into_element()
}
