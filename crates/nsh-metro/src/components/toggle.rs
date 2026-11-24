//! The toggle switch, the check box and the radio button.

use std::rc::Rc;

use nsh_core::prelude::*;
use nsh_core::{TransformKind, element::TransformOrigin};

use crate::{
    Icon,
    motion::EXP6_OUT,
    theme::{FONT_FAMILY, palette, reduced_motion, size, u},
};

/// What a toggle runs, with its new state.
pub type ToggleHandler = Rc<dyn Fn(bool)>;

/// The WP8 toggle: a label (On/Off) and a square track whose thumb slides
/// across, the accent filling in behind it.
pub struct ToggleSwitch {
    on: Reactive<bool>,
    header: Option<SharedString>,
    labels: (SharedString, SharedString),
    disabled: bool,
    on_change: Option<ToggleHandler>,
}

pub fn toggle_switch(on: impl IntoReactive<bool>) -> ToggleSwitch {
    ToggleSwitch { on: on.into_reactive(), header: None, labels: ("On".into(), "Off".into()), disabled: false, on_change: None }
}

impl ToggleSwitch {
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = Some(header.into());
        self
    }
    pub fn labels(mut self, on: impl Into<SharedString>, off: impl Into<SharedString>) -> Self {
        self.labels = (on.into(), off.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_change(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

const TRACK_W: f32 = 76.;
const TRACK_H: f32 = 32.;
const THUMB_W: f32 = 20.;
const TRAVEL: f32 = TRACK_W - THUMB_W;

impl IntoElement for ToggleSwitch {
    fn into_element(self) -> AnyElement {
        let disabled = self.disabled;
        let fg = move || if disabled { palette().disabled } else { palette().foreground };
        let fill = move || if disabled { palette().disabled } else { palette().accent };
        let on = self.on;
        let x = animated_value(if untracked(|| on.get()) { TRAVEL } else { 0. });
        {
            let on = on.clone();
            effect(move || {
                let to = if on.get() { TRAVEL } else { 0. };
                untracked(|| {
                    if reduced_motion() {
                        x.set(to);
                    } else {
                        let _ = x.timing(to, 220., EXP6_OUT);
                    }
                });
            });
        }
        let (on_label, off_label) = self.labels;
        let label = {
            let on = on.clone();
            move || if on.get() { on_label.clone() } else { off_label.clone() }
        };
        let left = TransformOrigin { x: px(0.).into(), y: relative(0.5).into(), z: Points::ZERO };
        let track = view()
            .relative()
            .w(u(TRACK_W))
            .h(u(TRACK_H))
            .border(px(2.))
            .border_color(fg)
            .child(
                // The fill behind the thumb, stretched from the left.
                view()
                    .absolute()
                    .top(u(4.))
                    .bottom(u(4.))
                    .left(u(4.))
                    .w(u(TRAVEL - 2.))
                    .bg(fill)
                    .transform_origin(left)
                    .bind_transform(TransformKind::ScaleX, &x.interpolate(&[0., 2., TRAVEL], &[0., 0., 1.])),
            )
            .child(view().absolute().top_0().bottom_0().left_0().w(u(THUMB_W)).bg(fg).bind_translate_x(&x));
        let row = view()
            .flex_row()
            .items_center()
            .justify_between()
            .child(text(label).font_family(FONT_FAMILY).text_size(u(size::LARGE)).text_color(fg))
            .child(track);
        let el = view()
            .gap(u(4.))
            .children(self.header.map(|h| text(h).font_family(FONT_FAMILY).text_size(u(size::NORMAL)).text_color(move || palette().subtle)))
            .child(row);
        match self.on_change.filter(|_| !disabled) {
            Some(f) => el.cursor_pointer().on_press(move || f(!untracked(|| on.get()))).into_element(),
            None => el.into_element(),
        }
    }
}

/// A square box with a check mark.
pub struct CheckBox {
    checked: Reactive<bool>,
    label: Option<SharedString>,
    disabled: bool,
    on_change: Option<ToggleHandler>,
}

pub fn check_box(checked: impl IntoReactive<bool>) -> CheckBox {
    CheckBox { checked: checked.into_reactive(), label: None, disabled: false, on_change: None }
}

impl CheckBox {
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_change(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl IntoElement for CheckBox {
    fn into_element(self) -> AnyElement {
        let disabled = self.disabled;
        let fg = move || if disabled { palette().disabled } else { palette().foreground };
        let checked = self.checked;
        let mark = {
            let checked = checked.clone();
            move || if checked.get() { 1f32 } else { 0f32 }
        };
        let el = view()
            .flex_row()
            .items_center()
            .gap(u(14.))
            .py(u(6.))
            .child(
                view()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .size(u(32.))
                    .border(px(2.))
                    .border_color(fg)
                    .child(view().opacity(mark).child(Icon::Check.render(22., fg))),
            )
            .children(self.label.map(|l| text(l).font_family(FONT_FAMILY).text_size(u(size::MEDIUM)).text_color(fg)));
        match self.on_change.filter(|_| !disabled) {
            Some(f) => el.cursor_pointer().active(|s| s.opacity(0.7)).on_press(move || f(!untracked(|| checked.get()))).into_element(),
            None => el.into_element(),
        }
    }
}

/// A round box with a dot.
pub struct RadioButton {
    selected: Reactive<bool>,
    label: Option<SharedString>,
    disabled: bool,
    on_select: Option<Rc<dyn Fn()>>,
}

pub fn radio_button(selected: impl IntoReactive<bool>) -> RadioButton {
    RadioButton { selected: selected.into_reactive(), label: None, disabled: false, on_select: None }
}

impl RadioButton {
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_select(mut self, f: impl Fn() + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl IntoElement for RadioButton {
    fn into_element(self) -> AnyElement {
        let disabled = self.disabled;
        let fg = move || if disabled { palette().disabled } else { palette().foreground };
        let selected = self.selected;
        let el = view()
            .flex_row()
            .items_center()
            .gap(u(14.))
            .py(u(6.))
            .child(
                view()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .size(u(32.))
                    .rounded(px(16.))
                    .border(px(2.))
                    .border_color(fg)
                    .child(view().size(u(14.)).rounded(px(7.)).bg(fg).opacity(move || if selected.get() { 1f32 } else { 0f32 })),
            )
            .children(self.label.map(|l| text(l).font_family(FONT_FAMILY).text_size(u(size::MEDIUM)).text_color(fg)));
        match self.on_select.filter(|_| !disabled) {
            Some(f) => el.cursor_pointer().active(|s| s.opacity(0.7)).on_press(move || f()).into_element(),
            None => el.into_element(),
        }
    }
}
