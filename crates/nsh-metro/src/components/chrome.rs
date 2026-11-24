//! Page chrome: the page header and the phone's system tray.

use nsh_core::prelude::*;

use crate::{
    Icon, TextStyle, text,
    theme::{palette, u},
};

/// The left margin of every Metro page.
pub const PAGE_MARGIN: f32 = 24.;

/// An app title in small caps over a huge lower-case page title.
pub struct PageHeader {
    app_title: Option<SharedString>,
    title: SharedString,
}

pub fn page_header(title: impl Into<SharedString>) -> PageHeader {
    PageHeader { app_title: None, title: title.into() }
}

impl PageHeader {
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
pub struct SystemTray {
    time: Reactive<SharedString>,
    battery: f32,
    transparent: bool,
}

pub fn system_tray(time: impl IntoReactive<SharedString>) -> SystemTray {
    SystemTray { time: time.into_reactive(), battery: 0.8, transparent: true }
}

impl SystemTray {
    pub fn battery(mut self, level: f32) -> Self {
        self.battery = level.clamp(0., 1.);
        self
    }

    pub fn opaque(mut self) -> Self {
        self.transparent = false;
        self
    }
}

impl IntoElement for SystemTray {
    fn into_element(self) -> AnyElement {
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
            .child(view().flex_row().gap(u(6.)).items_center().child(Icon::Signal.render(16., fg)).child(Icon::Wifi.render(16., fg)))
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
                                    .child(view().h_full().w(relative(self.battery)).bg(fg)),
                            )
                            .child(view().w(u(2.)).h(u(5.)).bg(fg)),
                    )
                    .child(text(TextStyle::Small, move || time.get()).color(fg)),
            )
            .into_element()
    }
}
