//! Progress: the determinate bar, the indeterminate dots that race across,
//! and the ring of dots.
//!
//! The dots and the ring are each one looping clock shared by their dots;
//! every dot maps the clock through its own (sampled) path, so the whole
//! indicator runs natively and is never rebuilt.

use nsh_core::prelude::*;
use nsh_core::TransformKind;

use crate::theme::{palette, u};

/// A bar filled to `value` (0–1; a value or a closure), or the racing dots.
pub struct ProgressBar {
    value: Option<Reactive<f32>>,
}

pub fn progress_bar(value: impl IntoReactive<f32>) -> ProgressBar {
    ProgressBar { value: Some(value.into_reactive()) }
}

/// The indeterminate bar: five dots that race in from the left, bunch up
/// in the middle, and race out.
pub fn progress_dots() -> ProgressBar {
    ProgressBar { value: None }
}

const DOTS: usize = 5;
const CYCLE_MS: f32 = 3200.;

/// Where dot `i` is (fraction of the bar) at cycle time `t`, or `None`
/// while it is off the bar.
pub fn dot_position(i: usize, t: f32) -> Option<f32> {
    let local = t - i as f32 * 0.055;
    if !(0. ..=0.72).contains(&local) {
        return None;
    }
    let u = local / 0.72;
    let x = if u < 0.3 {
        0.38 * (1. - (1. - u / 0.3).powi(2))
    } else if u < 0.7 {
        0.38 + 0.24 * ((u - 0.3) / 0.4)
    } else {
        0.62 + 0.45 * ((u - 0.7) / 0.3).powi(2)
    };
    Some(x)
}

/// Samples `f` over 0..=1 as an interpolation table.
fn sampled(n: usize, f: impl Fn(f32) -> f32) -> (Vec<f32>, Vec<f32>) {
    let input: Vec<f32> = (0..=n).map(|k| k as f32 / n as f32).collect();
    let output = input.iter().map(|&t| f(t)).collect();
    (input, output)
}

/// A clock running 0 → 1 every `ms`, forever.
fn clock(ms: f32) -> AnimatedValue {
    let t = animated_value(0.);
    let anim = Animation::timing(&t, 1., crate::motion::ambient(ms, Easing::Linear)).looped(None);
    let _ = anim.start();
    cleanup(move || anim.stop());
    t
}

impl IntoElement for ProgressBar {
    fn into_element(self) -> AnyElement {
        match self.value {
            Some(v) => view()
                .relative()
                .w_full()
                .h(u(4.))
                .bg(move || palette().accent.opacity(0.25))
                .child(
                    view()
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(move || relative(v.get().clamp(0., 1.)))
                        .bg(move || palette().accent),
                )
                .into_element(),
            None => {
                let t = clock(CYCLE_MS);
                view()
                    .relative()
                    .w_full()
                    .h(u(4.))
                    .overflow_hidden()
                    .children((0..DOTS).map(|i| {
                        // A full-width track per dot, moved by a fraction of
                        // its own width: the bar's.
                        let (input, x) = sampled(160, |t| dot_position(i, t).unwrap_or(if t < 0.5 { 0. } else { 1.1 }));
                        let (_, shown) = sampled(160, |t| if dot_position(i, t).is_some() { 1. } else { 0. });
                        view()
                            .absolute()
                            .top_0()
                            .left_0()
                            .w_full()
                            .h(u(4.))
                            .bind_transform(TransformKind::TranslateXFraction, &t.interpolate(&input, &x))
                            .bind_opacity(&t.interpolate(&input, &shown))
                            .child(view().size(u(4.)).bg(move || palette().accent))
                    }))
                    .into_element()
            }
        }
    }
}

/// The ring of dots that chase each other round.
pub struct ProgressRing {
    diameter: f32,
}

pub fn progress_ring() -> ProgressRing {
    ProgressRing { diameter: 40. }
}

impl ProgressRing {
    pub fn diameter(mut self, d: f32) -> Self {
        self.diameter = d;
        self
    }
}

pub const RING_DOTS: usize = 5;
const RING_CYCLE: f32 = 5.5;
const RING_STAGGER: f32 = 0.24;

#[derive(Clone, Copy)]
enum Ease {
    Linear,
    Out,
    InOut,
}

impl Ease {
    fn apply(self, t: f32) -> f32 {
        match self {
            Ease::Linear => t,
            Ease::Out => 1. - (1. - t) * (1. - t),
            Ease::InOut => t * t * (3. - 2. * t),
        }
    }
}

const RING_KEYS: [(f32, f32, Ease); 6] = [
    (0.00, 225., Ease::Out),
    (0.07, 345., Ease::Linear),
    (0.30, 455., Ease::InOut),
    (0.39, 690., Ease::Linear),
    (0.70, 815., Ease::Out),
    (0.75, 945., Ease::Linear),
];

/// A ring dot's angle (degrees clockwise from 12 o'clock) at cycle time
/// `t`, or `None` while it is hidden.
pub fn ring_dot(t: f32) -> Option<f32> {
    let t = t.rem_euclid(1.);
    if t >= 0.76 {
        return None;
    }
    let (end, end_deg, _) = RING_KEYS[RING_KEYS.len() - 1];
    if t >= end {
        return Some(end_deg);
    }
    let i = RING_KEYS.iter().rposition(|k| k.0 <= t).unwrap_or(0);
    let (t0, a0, ease) = RING_KEYS[i];
    let (t1, a1, _) = RING_KEYS[i + 1];
    Some(a0 + (a1 - a0) * ease.apply((t - t0) / (t1 - t0)))
}

impl IntoElement for ProgressRing {
    fn into_element(self) -> AnyElement {
        let d = self.diameter;
        let dot = (d / 8.).max(3.);
        let total = RING_CYCLE + RING_STAGGER * (RING_DOTS - 1) as f32;
        let t = clock(total * 1000.);
        view()
            .relative()
            .flex_shrink_0()
            .size(u(d))
            .children((0..RING_DOTS).map(|i| {
                // Dot i's own time, in cycles, at clock time `t`.
                let local = move |t: f32| (t * total - i as f32 * RING_STAGGER) / RING_CYCLE;
                let (input, angle) = sampled(240, |t| {
                    let l = local(t);
                    if (0. ..1.).contains(&l) { ring_dot(l).unwrap_or(945.) } else { 225. }
                });
                let (_, shown) = sampled(240, |t| {
                    let l = local(t);
                    if (0. ..1.).contains(&l) && ring_dot(l).is_some() { 1. } else { 0. }
                });
                // The dot sits at the top of a ring-sized frame, which turns.
                view()
                    .absolute()
                    .left_0()
                    .top_0()
                    .size(u(d))
                    .items_center()
                    .bind_rotate(&t.interpolate(&input, &angle))
                    .bind_opacity(&t.interpolate(&input, &shown))
                    .child(view().size(u(dot)).rounded(px(dot / 2.)).bg(move || palette().accent))
            }))
            .into_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dots_cross_left_to_right_in_order() {
        for step in 0..100 {
            let t = step as f32 / 100.;
            let xs: Vec<f32> = (0..DOTS).filter_map(|i| dot_position(i, t)).collect();
            assert!(xs.windows(2).all(|w| w[0] >= w[1]), "t={t}: {xs:?}");
        }
    }

    #[test]
    fn ring_dots_only_ever_go_clockwise_then_vanish() {
        let angles: Vec<f32> = (0..76).filter_map(|i| ring_dot(i as f32 / 100.)).collect();
        assert_eq!(angles.len(), 76);
        assert!(angles.windows(2).all(|w| w[1] >= w[0] - 1e-3));
        assert_eq!(ring_dot(0.8), None);
    }
}
