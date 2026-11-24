//! Page chrome: the page header and the phone's system tray.

use nsh_core::prelude::*;

use crate::{
    Icon, TextStyle, text,
    theme::{palette, u},
};

/// The left margin of every Metro page.
pub const PAGE_MARGIN: f32 = 24.;

/// An app title in small caps over a huge lower-case page title.
#[must_use = "a builder does nothing until it is placed in the tree"]
pub struct PageHeader {
    app_title: Option<SharedString>,
    title: SharedString,
}

/// A page header with `title`, set lower-case in [`TextStyle::Title1`].
pub fn page_header(title: impl Into<SharedString>) -> PageHeader {
    PageHeader { app_title: None, title: title.into() }
}

impl PageHeader {
    /// The app's name above the title, in upper-case [`TextStyle::AppTitle`].
    pub fn app_title(mut self, app: impl Into<SharedString>) -> Self {
        self.app_title = Some(app.into());
        self
    }
}

impl IntoElement for PageHeader {
    fn into_element(self) -> AnyElement {
        view()
            .pt(u(12.))
            .pb(u(20.))
            .px(u(PAGE_MARGIN))
            .children(self.app_title.map(|t| text(TextStyle::AppTitle, t)))
            .child(view().ml(u(-3.)).child(text(TextStyle::Title1, self.title).lowercase()))
            .into_element()
    }
}

/// The phone's status bar: signal, wifi, battery and the time.
#[must_use = "a builder does nothing until it is placed in the tree"]
pub struct SystemTray {
    time: Reactive<SharedString>,
    battery: Reactive<f32>,
    transparent: bool,
}

/// A system tray showing `time` (a value, or a signal or closure for a clock).
/// Transparent over the page, with the battery at 80%.
pub fn system_tray(time: impl IntoReactive<SharedString>) -> SystemTray {
    SystemTray { time: time.into_reactive(), battery: Reactive::Static(0.8), transparent: true }
}

impl SystemTray {
    /// The battery's charge, 0–1 (default 0.8).
    pub fn battery(mut self, level: impl IntoReactive<f32>) -> Self {
        self.battery = level.into_reactive();
        self
    }

    /// Fills the tray with the chrome colour instead of letting the page show through.
    pub fn opaque(mut self) -> Self {
        self.transparent = false;
        self
    }
}

impl IntoElement for SystemTray {
    fn into_element(self) -> AnyElement {
        let battery = self.battery.clone();
        let fg = move || palette().foreground;
        let transparent = self.transparent;
        let time = self.time;
        view()
            .flex_row()
            .flex_shrink_0()
            .items_center()
            .justify_between()
            .h(u(32.))
            .px(u(12.))
            .bg(move || if transparent { palette().chrome.opacity(0.) } else { palette().chrome })
            .child(
                view()
                    .flex_row()
                    .gap(u(6.))
                    .items_center()
                    .child(Icon::Signal.render(16., fg))
                    .child(Icon::Wifi.render(16., fg)),
            )
            .child(
                view()
                    .flex_row()
                    .gap(u(10.))
                    .items_center()
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .child(
                                view()
                                    .w(u(22.))
                                    .h(u(11.))
                                    .border_1()
                                    .border_color(fg)
                                    .p(u(1.5))
                                    .child(view().h_full().w(move || relative(battery.get().clamp(0., 1.))).bg(fg)),
                            )
                            .child(view().w(u(2.)).h(u(5.)).bg(fg)),
                    )
                    .child(text(TextStyle::Small, move || time.get()).color(fg)),
            )
            .into_element()
    }
}
