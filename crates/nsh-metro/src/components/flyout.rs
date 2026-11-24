//! The settings flyout (Windows 8's `SettingsFlyout`): a full-height panel
//! that slides in from the right edge over the page, under an accent header
//! with a back button and a title. Tapping outside it dismisses it.
//!
//! Give it whether it should be open; it slides in when that turns true and
//! back out by itself when it turns false, staying on screen until it has
//! gone.

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::{
    Icon,
    button::ClickHandler,
    motion::{EXP6_IN, EXP6_OUT},
    theme::{FONT_FAMILY, SEMILIGHT, palette, reduced_motion, size, u},
};

/// The two widths Windows 8 allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlyoutWidth {
    /// 346 px: the usual settings pane.
    Narrow,
    /// 646 px: room for richer content.
    Wide,
}

impl FlyoutWidth {
    pub fn px(self) -> f32 {
        match self {
            FlyoutWidth::Narrow => 346.,
            FlyoutWidth::Wide => 646.,
        }
    }
}

/// Height of the accent header.
pub const FLYOUT_HEADER: f32 = 80.;
/// Sliding in, in ms: WinJS `showPanel`, a long hard deceleration.
pub const FLYOUT_IN: f32 = 550.;
/// Sliding out, in ms: quicker, accelerating away.
pub const FLYOUT_OUT: f32 = 250.;

/// A settings flyout. Place it as the last child of a `relative()` root.
pub struct SettingsFlyout {
    open: Reactive<bool>,
    title: SharedString,
    width: FlyoutWidth,
    content: Vec<Rc<dyn Fn() -> AnyElement>>,
    on_dismiss: Option<ClickHandler>,
    on_back: Option<ClickHandler>,
}

pub fn settings_flyout(open: impl IntoReactive<bool>) -> SettingsFlyout {
    SettingsFlyout {
        open: open.into_reactive(),
        title: SharedString::default(),
        width: FlyoutWidth::Narrow,
        content: Vec::new(),
        on_dismiss: None,
        on_back: None,
    }
}

impl SettingsFlyout {
    /// The header title (rendered lower-case, as Metro set it).
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }
    pub fn width(mut self, width: FlyoutWidth) -> Self {
        self.width = width;
        self
    }
    /// Content under the header (built each time it opens); it scrolls if
    /// it runs long.
    pub fn child<E: IntoElement>(mut self, e: impl Fn() -> E + 'static) -> Self {
        self.content.push(Rc::new(move || e().into_element()));
        self
    }
    /// Light dismiss: a tap outside the panel. Set `open` false in response.
    pub fn on_dismiss(mut self, f: impl Fn() + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
    /// The header's back button. Without it, back dismisses.
    pub fn on_back(mut self, f: impl Fn() + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }
}

impl IntoElement for SettingsFlyout {
    fn into_element(self) -> AnyElement {
        let w = self.width.px();
        let open = self.open;
        let initially = untracked(|| open.get());
        // Off the edge by this many px (0 = in place).
        let x = animated_value(if initially { 0. } else { w });
        let mounted = signal(initially);
        let is_open = signal(initially);
        effect(move || {
            let o = open.get();
            untracked(|| {
                is_open.set(o);
                if o {
                    mounted.set(true);
                    if reduced_motion() {
                        x.set(0.);
                    } else {
                        x.set(w);
                        let _ = x.timing(0., FLYOUT_IN, EXP6_OUT);
                    }
                } else if reduced_motion() {
                    mounted.set(false);
                } else {
                    let end = x.timing(w, FLYOUT_OUT, EXP6_IN);
                    spawn(async move {
                        if end.await && !is_open.get_untracked() {
                            mounted.set(false);
                        }
                    });
                }
            });
        });
        let back = self.on_back.or_else(|| self.on_dismiss.clone());
        let title = SharedString::from(self.title.to_lowercase());
        let content = Rc::new(self.content);
        let dismiss = self.on_dismiss;
        let panel = move || {
            let ink = move || palette().accent_ink;
            let back_button = view()
                .flex_shrink_0()
                .items_center()
                .justify_center()
                .size(u(32.))
                .rounded(px(16.))
                .border(px(2.))
                .border_color(ink)
                .cursor_pointer()
                .active(|s| s.opacity(0.6))
                .child(Icon::Back.render(16., ink));
            let back_button = match back.clone() {
                Some(f) => back_button.on_press(move || f()),
                None => back_button,
            };
            let header = view()
                .flex_row()
                .flex_shrink_0()
                .items_center()
                .gap(u(14.))
                .h(u(FLYOUT_HEADER))
                .px(u(24.))
                .bg(move || palette().accent)
                .child(back_button)
                .child(
                    text(title.clone())
                        .font_family(FONT_FAMILY)
                        .text_size(u(size::LARGE))
                        .font_weight(SEMILIGHT)
                        .truncate()
                        .text_color(ink),
                );
            let panel = view()
                .absolute()
                .top_0()
                .bottom_0()
                .right_0()
                .w(u(w))
                .bg(move || palette().background)
                .border_l_1()
                .border_color(move || palette().accent)
                .bind_translate_x(&x)
                .on_press(|| {})
                .child(header)
                .child(
                    scroll().flex_1().child(view().px(u(24.)).py(u(20.)).gap(u(18.)).children(content.iter().map(|c| c()))),
                );
            // While open, a press outside the panel closes it.
            let dismiss = dismiss.clone();
            let catcher = move || {
                let dismiss = dismiss.clone();
                view().absolute().inset_0().on_press(move || {
                    if let Some(f) = &dismiss {
                        f();
                    }
                })
            };
            view()
                .absolute()
                .inset_0()
                .child(when(move || is_open.get(), catcher))
                .child(panel)
                .into_element()
        };
        dynamic(move || if mounted.get() { panel() } else { view().into_element() }).into_element()
    }
}
