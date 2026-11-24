//! Horizontal paging input shared by the pivot and the panorama.
//!
//! * **Mouse**: Shift + wheel pages sideways (wheel down = next); three
//!   notches make a page.
//! * **Trackpad**: a horizontal two-finger swipe, one page per gesture.
//! * **Drag**: a horizontal drag past [`SWIPE_THRESHOLD`] pages.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use nsh_core::prelude::*;
use nsh_core::WheelDelta;

/// Horizontal travel (px) that turns a scroll into a page step.
pub const SWIPE_THRESHOLD: f32 = 60.;
/// Pixels per wheel notch.
const LINE: f32 = 20.;
/// Quiet time after which a trackpad gesture counts as over.
const GESTURE_IDLE: Duration = Duration::from_millis(160);

/// `+1` = next page, `-1` = previous page.
pub type StepHandler = Rc<dyn Fn(i32)>;

#[derive(Default)]
struct Swipe {
    accum: Cell<f32>,
    /// A trackpad gesture already paged; ignore it until it goes quiet.
    spent: Cell<bool>,
    epoch: Cell<u64>,
}

/// `el` turned into a paging surface: wheel, trackpad and drag input that
/// travels sideways calls `on_step`; vertical scrolling is left alone.
pub fn pageable<E: Interactive>(el: E, on_step: StepHandler) -> E {
    let swipe = Rc::new(Swipe::default());
    let wheel_step = on_step.clone();
    let el = el.on_wheel(move |e: &WheelEvent| {
        let (dx, dy, wheel) = match e.delta {
            WheelDelta::Points(d) => (f32::from(d.x), f32::from(d.y), false),
            WheelDelta::Lines(d) => (d.x * LINE, d.y * LINE, true),
        };
        let shift = e.modifiers.shift;
        let dx = if shift && dx == 0. { dy } else { dx };
        let dy = if shift { 0. } else { dy };
        if dx == 0. || dx.abs() < dy.abs() {
            return;
        }
        // Re-arm once the gesture goes quiet.
        let epoch = swipe.epoch.get() + 1;
        swipe.epoch.set(epoch);
        let s = swipe.clone();
        set_timeout(GESTURE_IDLE, move || {
            if s.epoch.get() == epoch {
                s.spent.set(false);
                s.accum.set(0.);
            }
        });
        if swipe.spent.get() {
            return;
        }
        let accum = swipe.accum.get() + dx;
        if accum.abs() > SWIPE_THRESHOLD {
            swipe.accum.set(0.);
            // A wheel has no gesture: let it keep paging.
            swipe.spent.set(!wheel);
            wheel_step(if accum < 0. { 1 } else { -1 });
        } else {
            swipe.accum.set(accum);
        }
    });
    el.on_pan_end(move |e: &PanEvent| {
        let dx = f32::from(e.translation.x);
        if dx.abs() > SWIPE_THRESHOLD && dx.abs() > f32::from(e.translation.y).abs() {
            on_step(if dx < 0. { 1 } else { -1 });
        }
    })
}
