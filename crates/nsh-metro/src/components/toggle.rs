//! The toggle switch, the check box and the radio button.

use std::rc::Rc;

use nsh_core::prelude::*;

use crate::components::text::label;
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
#[must_use = "a builder does nothing until it is placed in the tree"]
#[derive(Clone)]
pub struct ToggleSwitch {
    on: Reactive<bool>,
    header: Option<SharedString>,
    labels: (SharedString, SharedString),
    disabled: Reactive<bool>,
    on_change: Option<ToggleHandler>,
}

/// A toggle switch showing `on` (a value, or a signal or closure), labelled
/// "On" / "Off".
pub fn toggle_switch(on: impl IntoReactive<bool>) -> ToggleSwitch {
    ToggleSwitch {
        on: on.into_reactive(),
        header: None,
        labels: ("On".into(), "Off".into()),
        disabled: Reactive::Static(false),
        on_change: None,
    }
}

impl ToggleSwitch {
    /// The caption above, in the subtle colour.
    pub fn header(mut self, header: impl Into<SharedString>) -> Self {
        self.header = Some(header.into());
        self
    }
    /// The labels shown beside the track when on and when off.
    pub fn labels(mut self, on: impl Into<SharedString>, off: impl Into<SharedString>) -> Self {
        self.labels = (on.into(), off.into());
        self
    }
    /// Greys the switch out and ignores taps.
    pub fn disabled(mut self, disabled: impl IntoReactive<bool>) -> Self {
        self.disabled = disabled.into_reactive();
        self
    }
    /// Runs `f` with the state a tap asks for; the thumb slides when `on`
    /// changes. Set `on` from it.
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
        let disabled = self.disabled.clone();
        super::rebuilt_on(disabled, move |d| self.clone().build(d))
    }
}

impl ToggleSwitch {
    fn build(self, disabled: bool) -> AnyElement {
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
        let shown_label = {
            let on = on.clone();
            move || if on.get() { on_label.clone() } else { off_label.clone() }
        };
        let left = TransformOrigin { x: px(0.).into(), y: relative(0.5), z: Points::ZERO };
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
            .child(label(shown_label).font_family(FONT_FAMILY).text_size(u(size::LARGE)).text_color(fg))
            .child(track);
        let el = view()
            .gap(u(4.))
            .children(self.header.map(|h| {
                label(h).font_family(FONT_FAMILY).text_size(u(size::NORMAL)).text_color(move || palette().subtle)
            }))
            .child(row);
        match self.on_change.filter(|_| !disabled) {
            Some(f) => el.cursor_pointer().on_press(move || f(!untracked(|| on.get()))).into_element(),
            None => el.into_element(),
        }
    }
}

/// A square box with a check mark.
#[must_use = "a builder does nothing until it is placed in the tree"]
#[derive(Clone)]
pub struct CheckBox {
    checked: Reactive<bool>,
    label: Option<SharedString>,
    disabled: Reactive<bool>,
    on_change: Option<ToggleHandler>,
}

/// A check box showing `checked` (a value, or a signal or closure).
pub fn check_box(checked: impl IntoReactive<bool>) -> CheckBox {
    CheckBox { checked: checked.into_reactive(), label: None, disabled: Reactive::Static(false), on_change: None }
}

impl CheckBox {
    /// The text beside the box.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Greys the box out and ignores taps.
    pub fn disabled(mut self, disabled: impl IntoReactive<bool>) -> Self {
        self.disabled = disabled.into_reactive();
        self
    }
    /// Runs `f` with the state a tap asks for. Set `checked` from it.
    pub fn on_change(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl IntoElement for CheckBox {
    fn into_element(self) -> AnyElement {
        let disabled = self.disabled.clone();
        super::rebuilt_on(disabled, move |d| self.clone().build(d))
    }
}

impl CheckBox {
    fn build(self, disabled: bool) -> AnyElement {
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
            .children(self.label.map(|l| label(l).font_family(FONT_FAMILY).text_size(u(size::MEDIUM)).text_color(fg)));
        match self.on_change.filter(|_| !disabled) {
            Some(f) => el
                .cursor_pointer()
                .active(|s| s.opacity(0.7))
                .on_press(move || f(!untracked(|| checked.get())))
                .into_element(),
            None => el.into_element(),
        }
    }
}

/// A round box with a dot.
#[must_use = "a builder does nothing until it is placed in the tree"]
#[derive(Clone)]
pub struct RadioButton {
    selected: Reactive<bool>,
    label: Option<SharedString>,
    disabled: Reactive<bool>,
    on_select: Option<Rc<dyn Fn()>>,
}

/// A radio button showing `selected` (a value, or a signal or closure).
pub fn radio_button(selected: impl IntoReactive<bool>) -> RadioButton {
    RadioButton { selected: selected.into_reactive(), label: None, disabled: Reactive::Static(false), on_select: None }
}

impl RadioButton {
    /// The text beside the button.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Greys the button out and ignores taps.
    pub fn disabled(mut self, disabled: impl IntoReactive<bool>) -> Self {
        self.disabled = disabled.into_reactive();
        self
    }
    /// Runs `f` when the button is chosen. Named as the pivot's and the
    /// list picker's are: `on_select` is one of many chosen, `on_change` a
    /// state toggled (a check box, a switch).
    pub fn on_select(mut self, f: impl Fn() + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl IntoElement for RadioButton {
    fn into_element(self) -> AnyElement {
        let disabled = self.disabled.clone();
        super::rebuilt_on(disabled, move |d| self.clone().build(d))
    }
}

impl RadioButton {
    fn build(self, disabled: bool) -> AnyElement {
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
                    .child(
                        view()
                            .size(u(14.))
                            .rounded(px(7.))
                            .bg(fg)
                            .opacity(move || if selected.get() { 1f32 } else { 0f32 }),
                    ),
            )
            .children(self.label.map(|l| label(l).font_family(FONT_FAMILY).text_size(u(size::MEDIUM)).text_color(fg)));
        match self.on_select.filter(|_| !disabled) {
            Some(f) => el.cursor_pointer().active(|s| s.opacity(0.7)).on_press(move || f()).into_element(),
            None => el.into_element(),
        }
    }
}
