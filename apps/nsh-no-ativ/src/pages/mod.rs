//! One module per destination. Every page is a function of the app handle
//! and its own back-stack entry, returning a [`PageView`]. Pages are built
//! once per visit; what changes while one is on screen is bound.

mod deck;
mod freezer;
mod incidents;
mod people;
mod reagents;
mod runs;
pub mod settings;
mod shipments;
mod start;
mod strains;
mod telemetry;

use nsh_core::prelude::*;
use nsh_metro::label;
use nsh_metro::{
    Accent, AppBar, PAGE_MARGIN, TextStyle, page_header, text,
    theme::FONT_FAMILY,
    u,
};

use crate::app::{App, LEFT_GUTTER};
use crate::nav::Page;

/// Lists and forms stop growing here on a wide window.
pub const MAX_LINE: f32 = 760.;

/// Application title above every page title.
pub const APP: &str = "no ativ";

/// What a page shows: its body, and optionally an application bar.
pub struct PageView {
    pub body: AnyElement,
    pub app_bar: Option<AppBar>,
}

impl PageView {
    pub fn new(body: impl IntoElement) -> Self {
        Self { body: body.into_element(), app_bar: None }
    }
    pub fn app_bar(mut self, bar: AppBar) -> Self {
        self.app_bar = Some(bar);
        self
    }
}

/// The page in the shell's frame: the body inset by the gutter, the app bar
/// along the bottom. The page host turns the whole frame.
fn frame(page: PageView) -> AnyElement {
    view()
        .size_full()
        .child(view().flex_1().min_h(px(0.)).pt(u(24.)).pl(u(LEFT_GUTTER)).overflow_hidden().child(page.body))
        .children(page.app_bar)
        .into_element()
}

pub fn render(app: App, page: Page) -> AnyElement {
    match page {
        Page::Start => frame(start::render(app)),
        Page::Runs { pivot } => frame(runs::list(app, pivot)),
        // Detail pages rebuild when the ledger changes under them (a
        // harvest, an incident moving on); live readings are bound.
        Page::Run { id } => dynamic(move || frame(runs::detail(app, &id))),
        Page::Telemetry { zone } => frame(telemetry::render(app, zone)),
        Page::Strains => frame(strains::list(app)),
        Page::Strain { id } => dynamic(move || frame(strains::detail(app, &id))),
        Page::Incidents { pivot } => frame(incidents::list(app, pivot)),
        Page::Incident { id } => dynamic(move || frame(incidents::detail(app, &id))),
        Page::NewIncident { run } => frame(incidents::form(app, run)),
        Page::Reagents { pivot } => frame(reagents::render(app, pivot)),
        Page::Shipments { pivot } => frame(shipments::render(app, pivot)),
        Page::Freezer => frame(freezer::render(app)),
        Page::Deck { pivot } => frame(deck::render(app, pivot)),
        Page::People { pivot } => frame(people::render(app, pivot)),
        Page::Settings => frame(settings::render(app)),
    }
}

/// Warning tone: amber, whatever the accent.
pub fn warn() -> Hsla {
    Accent::Amber.color()
}

/// Alarm tone: red, whatever the accent.
pub fn alarm() -> Hsla {
    Accent::Red.color()
}

/// A vertically scrolling page body with the standard margins.
pub fn scrolling(content: View) -> AnyElement {
    scroll()
        .size_full()
        .child(content.w_full().max_w(u(MAX_LINE)).px(u(PAGE_MARGIN)).pb(u(32.)))
        .into_element()
}

/// A page with the two-line header over a scrolling body.
pub fn titled(section: &str, title: impl Into<SharedString>, body: impl IntoElement) -> AnyElement {
    view()
        .size_full()
        .child(page_header(title).app_title(format!("{APP} · {section}")))
        .child(view().flex_1().min_h(px(0.)).child(body))
        .into_element()
}

/// A labelled value: subtle caption over the value.
pub fn fact(label: &'static str, value: impl IntoReactive<SharedString>) -> View {
    view().pb(u(10.)).child(text(TextStyle::Small, label)).child(text(TextStyle::Large, value))
}

/// Half the row, for facts laid out two by two.
pub fn half(el: View) -> View {
    el.w(relative(0.5))
}

/// A group heading inside a page body.
pub fn heading(label: impl Into<SharedString>) -> View {
    let label: SharedString = label.into();
    view().pt(u(18.)).pb(u(6.)).child(text(TextStyle::GroupHeader, label))
}

/// A subtle line of text.
pub fn subtle(s: impl IntoReactive<SharedString>) -> nsh_metro::Text {
    text(TextStyle::Subtle, s)
}

/// Plain text in the Metro face at `size`, for the few one-off sizes.
pub fn raw(s: impl IntoReactive<SharedString>, size: f32) -> nsh_core::prelude::Text {
    label(s).font_family(FONT_FAMILY).text_size(u(size))
}

/// What a detail page shows when its record has gone (e.g. a stale link).
pub fn missing(section: &str, what: &str) -> PageView {
    PageView::new(titled(section, "not found", scrolling(view().child(subtle(format!("No {what} with that id."))))))
}
