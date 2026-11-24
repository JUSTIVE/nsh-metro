//! The slider: a thick track filled with the accent up to a square thumb.

use std::cell::Cell;
use std::rc::Rc;

use nsh_core::prelude::*;

use crate::components::text::label;

use crate::theme::{FONT_FAMILY, palette, size, u};

/// What a slider runs, with the new value (0–1).
pub type ValueHandler = Rc<dyn Fn(f32)>;

pub struct Slider {
    value: Reactive<f32>,
    header: Option<Reactive<SharedString>>,
    step: Option<f32>,
    disabled: bool,
    on_change: Option<ValueHandler>,
}

pub fn slider(value: impl IntoReactive<f32>) -> Slider {
    Slider { value: value.into_reactive(), header: None, step: None, disabled: false, on_change: None }
}

impl Slider {
    /// The caption above; a value or a closure (`move || format!(…)`).
    pub fn header(mut self, header: impl IntoReactive<SharedString>) -> Self {
        self.header = Some(header.into_reactive());
        self
    }
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_change(mut self, f: impl Fn(f32) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

/// The value at `x` px into a track `width` wide, snapped to `step`.
pub fn value_at(x: f32, width: f32, step: Option<f32>) -> f32 {
    let v = (x / width.max(1.)).clamp(0., 1.);
    match step {
        Some(s) if s > 0. => ((v / s).round() * s).clamp(0., 1.),
        _ => v,
    }
}

impl IntoElement for Slider {
    fn into_element(self) -> AnyElement {
        let disabled = self.disabled;
        let value = self.value;
        let v = {
            let value = value.clone();
            move || value.get().clamp(0., 1.)
        };
        let width = Rc::new(Cell::new(1f32));
        let measure = width.clone();
        let step = self.step;
        let bar = view()
            .relative()
            .h(u(36.))
            .justify_center()
            .on_layout(move |b: &Bounds<Points>| measure.set(f32::from(b.size.width)))
            .child(
                view()
                    .relative()
                    .w_full()
                    .h(u(12.))
                    .bg(move || if disabled { palette().disabled.opacity(0.4) } else { palette().input.opacity(0.5) })
                    .child(
                        view()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left_0()
                            .w({
                                let v = v.clone();
                                move || relative(v())
                            })
                            .bg(move || if disabled { palette().disabled } else { palette().accent }),
                    ),
            )
            .child(
                view()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(move || relative(v()))
                    .ml(u(-6.))
                    .w(u(12.))
                    .bg(move || if disabled { palette().disabled } else { palette().foreground }),
            );
        let bar = match self.on_change.filter(|_| !disabled) {
            Some(f) => {
                let (w1, w2) = (width.clone(), width);
                let f2 = f.clone();
                bar.cursor_pointer()
                    .on_press_down(move |e: &PressEvent| f(value_at(f32::from(e.position.x), w1.get(), step)))
                    .on_mouse_move(move |e: &MouseEvent| {
                        if e.pressed_button.is_some() {
                            f2(value_at(f32::from(e.position.x), w2.get(), step));
                        }
                    })
            }
            None => bar,
        };
        view()
            .gap(u(6.))
            .children(self.header.map(|h| label(move || h.get()).font_family(FONT_FAMILY).text_size(u(size::NORMAL)).text_color(move || palette().subtle)))
            .child(bar)
            .into_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_maps_and_snaps() {
        assert_eq!(value_at(0., 200., None), 0.);
        assert_eq!(value_at(100., 200., None), 0.5);
        assert_eq!(value_at(300., 200., None), 1.);
        assert!((value_at(61., 200., Some(0.25)) - 0.25).abs() < 1e-6);
    }
}
