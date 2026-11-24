//! Buttons, selection controls, slider, list picker and text box.

use nsh_core::prelude::*;
use nsh_metro::{
    Accent, Icon, TextStyle, Theme, button, check_box, hyperlink, list_picker, radio_button, round_button,
    slider, text, text_box,
    toggle_switch, u,
};

use super::{DemoState, caption, page, stack};

/// A horizontal row of controls at their natural width.
fn row() -> View {
    view().flex_row().flex_wrap().items_center().gap(u(12.))
}

pub fn buttons(s: DemoState) -> AnyElement {
    let press = move |label: &'static str| {
        move || {
            s.button_taps.update(|n| *n += 1);
            s.last_button.set(label);
        }
    };
    page(
        "button",
        stack()
            .child(caption("normal · accent"))
            .child(
                row()
                    .child(button("send").on_click(press("send")))
                    .child(button("save").accent().on_click(press("save"))),
            )
            .child(caption("disabled"))
            .child(row().child(button("delete").disabled(true).on_click(press("delete"))))
            .child(caption("with a glyph"))
            .child(
                row()
                    .child(button("share").icon(Icon::Share).on_click(press("share")))
                    .child(button("upload").icon(Icon::Upload).accent().on_click(press("upload"))),
            )
            .child(caption("full width"))
            .child(button("sign in").full_width().on_click(press("sign in")))
            .child(text(TextStyle::Large, move || format!("{} taps · last: {}", s.button_taps.get(), s.last_button.get()))),
    )
}

pub fn round_buttons(s: DemoState) -> AnyElement {
    let play = dynamic(move || {
        round_button(if s.playing.get() { Icon::Pause } else { Icon::Play })
            .diameter(72.)
            .on_click(move || s.playing.update(|p| *p = !*p))
    });
    let transport = row().gap(u(20.)).child(round_button(Icon::Previous)).child(play).child(round_button(Icon::Next));
    let favorite = dynamic(move || {
        round_button(Icon::Favorite).filled(s.favorite.get()).on_click(move || s.favorite.update(|f| *f = !*f))
    });
    page(
        "round button",
        stack()
            .child(caption("transport — the big one toggles play / pause"))
            .child(transport)
            .child(text(TextStyle::Subtle, move || if s.playing.get() { "playing" } else { "paused" }))
            .child(caption("filled = toggled on"))
            .child(row().gap(u(20.)).child(favorite).child(text(TextStyle::Normal, move || {
                if s.favorite.get() { "favourite" } else { "not a favourite" }
            })))
            .child(caption("sizes · disabled"))
            .child(
                row()
                    .gap(u(20.))
                    .child(round_button(Icon::Add).diameter(36.))
                    .child(round_button(Icon::Add))
                    .child(round_button(Icon::Add).diameter(64.))
                    .child(round_button(Icon::Delete).disabled(true)),
            ),
    )
}

pub fn hyperlinks(s: DemoState) -> AnyElement {
    let links = ["privacy statement", "learn more about accents", "forgot your password?"];
    page(
        "hyperlink",
        stack()
            .child(text(TextStyle::Normal, "Find My Phone can ring, lock or erase this phone from the web."))
            .children(links.map(|label| hyperlink(label).on_click(move || s.last_link.set(Some(label)))))
            .child(text(TextStyle::Subtle, move || match s.last_link.get() {
                Some(l) => format!("opened: {l}"),
                None => "tap a link".into(),
            })),
    )
}

pub fn toggle_switches(s: DemoState) -> AnyElement {
    // The wi-fi switch is disabled under airplane mode: built again when
    // that changes (disabled is a build-time setting).
    let wifi = dynamic(move || {
        toggle_switch(move || s.wifi.get() && !s.airplane.get())
            .header("wi-fi networking")
            .disabled(s.airplane.get())
            .on_change(move |v| s.wifi.set(v))
    });
    page(
        "toggle switch",
        stack()
            .child(wifi)
            .child(
                toggle_switch(s.bluetooth)
                    .header("bluetooth")
                    .labels("Visible", "Hidden")
                    .on_change(move |v| s.bluetooth.set(v)),
            )
            .child(toggle_switch(s.airplane).header("airplane mode").on_change(move |v| s.airplane.set(v)))
            .child(caption("airplane mode disables the wi-fi switch"))
            .child(toggle_switch(true).header("location (managed by policy)").disabled(true)),
    )
}

pub fn check_boxes(s: DemoState) -> AnyElement {
    let labels = ["sync contacts", "sync calendar", "sync photos"];
    let boxes = labels.into_iter().zip(s.checks).map(|(label, c)| check_box(c).label(label).on_change(move |v| c.set(v)));
    page(
        "check box",
        stack()
            .child(view().children(boxes))
            .child(check_box(true).label("sync mail (required)").disabled(true))
            .child(text(TextStyle::Subtle, move || {
                let count = s.checks.iter().filter(|c| c.get()).count();
                format!("{count} of {} selected", labels.len())
            })),
    )
}

const ROUTES: [&str; 3] = ["fastest", "shortest", "avoid tolls"];

pub fn radio_buttons(s: DemoState) -> AnyElement {
    let radios = ROUTES
        .into_iter()
        .enumerate()
        .map(|(ix, label)| radio_button(move || s.route.get() == ix).label(label).on_select(move || s.route.set(ix)));
    page(
        "radio button",
        stack()
            .child(caption("route preference"))
            .child(
                view()
                    .children(radios)
                    .child(radio_button(false).label("scenic (needs a data connection)").disabled(true)),
            )
            .child(text(TextStyle::Accent, move || format!("routing: {}", ROUTES[s.route.get()]))),
    )
}

/// A slider header that follows its value. The library's header is fixed
/// text, and rebuilding the slider mid-drag would end the drag.
pub fn sliders(s: DemoState) -> AnyElement {
    page(
        "slider",
        stack()
            .child(
                slider(s.volume)
                    .header(move || format!("ringer volume · {:.0}", s.volume.get() * 30.))
                    .on_change(move |v| s.volume.set(v)),
            )
            .child(
                slider(s.brightness)
                    .header(move || format!("brightness · step 0.1 · {:.0}%", s.brightness.get() * 100.))
                    .step(0.1)
                    .on_change(move |v| s.brightness.set(v)),
            )
            .child(slider(0.3).header("locked by policy").disabled(true))
            .child(caption("drag past the ends: the thumb keeps tracking")),
    )
}

const RINGTONES: [&str; 6] = ["chimes", "cloud", "glimmer", "minimal", "pulse", "xylophone"];

pub fn list_pickers(s: DemoState) -> AnyElement {
    let current = move || {
        let accent = Theme::get().accent;
        Accent::ALL.iter().position(|&a| a == accent).unwrap_or(0)
    };
    page(
        "list picker",
        stack()
            .child(list_picker(RINGTONES, s.ringtone).header("ringtone").on_select(move |ix| s.ringtone.set(ix)))
            .child(text(TextStyle::Subtle, move || format!("plays “{}” for calls", RINGTONES[s.ringtone.get()])))
            .child(
                list_picker(Accent::ALL.map(Accent::name), current)
                    .header("accent colour (changes the theme)")
                    .on_select(|ix| Theme::set_accent(Accent::ALL[ix])),
            ),
    )
}

pub fn text_boxes(s: DemoState) -> AnyElement {
    let focused = signal(false);
    let greeting = move || {
        let name = s.name.get();
        let name = name.trim();
        if name.is_empty() { "hello, stranger".to_string() } else { format!("hello, {name}") }
    };
    page(
        "text box",
        stack()
            .child(text_box(s.name).header("name").placeholder("your name").on_focus_change(move |f| focused.set(f)))
            .child(text(TextStyle::Large, greeting))
            .child(caption(move || if focused.get() { "focused: white field, accent border" } else { "click the field to focus" }))
            .child(text_box(s.message).header("message").placeholder("type and press enter").on_submit(move |v| {
                let sent = v.trim().to_string();
                if !sent.is_empty() {
                    s.submitted.set(Some(sent));
                    s.message.set(String::new());
                }
            }))
            .child(text(TextStyle::Accent, move || match s.submitted.get() {
                Some(m) => format!("sent: {m}"),
                None => "nothing sent yet".into(),
            }))
            .child(caption("⌥← ⌥→ by word · ⌘A · ⌘Z · double-click selects a word")),
    )
}
