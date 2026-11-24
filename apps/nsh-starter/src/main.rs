//! The nsh-metro starter: a start screen of tiles that opens a pivot app,
//! turnstiles between pages, and a settings flyout that switches the theme.
//! The agent docs and the Claude Code skill hand this file out as the
//! template, so CI keeps the documented code compiling.

use nsh_core::prelude::*;
use nsh_metro::prelude::*;
// `text` is named in both preludes; importing it by name picks Metro's.
use nsh_metro::text;
use nsh_platform_macos as platform;

/// The app's pages; `page_host` turnstiles between them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Page {
    Start,
    Mail,
}

#[component]
fn App() -> impl IntoElement {
    // State is signals, made once: a component body runs once, and only the
    // closures bound to properties run again when what they read changes.
    let page = signal((Page::Start, Direction::Forward));
    let unread = signal(3u32);
    let folder = signal(0usize);
    let settings_open = signal(false);
    let dark = signal(true);

    let content = page_host(page, 0., move |p| match p {
        Page::Start => start(unread, move || page.set((Page::Mail, Direction::Forward))),
        Page::Mail => mail(folder, unread, move || page.set((Page::Start, Direction::Backward))),
    });

    let settings = settings_flyout(settings_open)
        .title("settings")
        .child(move || {
            toggle_switch(dark).header("dark theme").on_change(move |on| {
                dark.set(on);
                Theme::set_mode(if on { Mode::Dark } else { Mode::Light });
            })
        })
        .on_dismiss(move || settings_open.set(false));

    // The root's view is the window's content (AppOptions sizes the window);
    // returning `window()` here would open a second one.
    view()
        .relative()
        .flex_1()
        .bg(move || palette().background)
        .child(view().flex_1().min_h(u(0.)).child(content))
        .child(app_bar().button(Icon::Settings, "settings", move || settings_open.set(true)))
        .child(settings)
}

/// The start screen. Tapping a tile swings the tiles away (the turnstile),
/// then runs the tile's handler.
fn start(unread: Signal<u32>, open_mail: impl Fn() + 'static) -> AnyElement {
    view()
        .p(u(PAGE_MARGIN))
        .gap(u(24.))
        .child(text(TextStyle::Title1, "start").lowercase())
        .child(tile_grid().turnstile(true).tiles([
            tile("mail", TileSize::Medium).title("mail").icon(Icon::Mail).count(unread).on_click(open_mail),
            tile("phone", TileSize::Small).icon(Icon::Phone),
            tile("camera", TileSize::Small).icon(Icon::Camera).color(Accent::Magenta.color()),
            tile("people", TileSize::Wide).title("people").icon(Icon::People).back_text("kristin", ["new photo"]),
        ]))
        .into_element()
}

/// A pivot app: swipe, or tap a header, to change pane.
fn mail(folder: Signal<usize>, unread: Signal<u32>, back: impl Fn() + 'static) -> AnyElement {
    view()
        .size_full()
        .child(
            pivot(["inbox", "sent"], folder)
                .title("mail")
                .child(move || {
                    view()
                        .child(
                            list_item("anna")
                                .subtitle("lunch?")
                                .detail("2:14 pm")
                                .on_click(move || unread.update(|n| *n = n.saturating_sub(1))),
                        )
                        .child(list_item("ben").subtitle("the build is green").detail("11:02 am"))
                })
                .child(|| text(TextStyle::Subtle, "nothing sent yet"))
                .on_select(move |i| folder.set(i)),
        )
        .child(view().px(u(PAGE_MARGIN)).py(u(12.)).child(button("back").on_click(back)))
        .into_element()
}

fn main() -> impl std::process::Termination {
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<App>()),
        platform::AppOptions { title: "nsh-metro starter".to_owned(), size: (480, 800), ..Default::default() },
    )
}
