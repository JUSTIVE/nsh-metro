//! Metro motion on nsh: the turnstile, the pivot's line-by-line slide, the
//! swivel and the tilt, as native animations.
//!
//! gpui-metro drew every frame of these itself: it re-rendered the window,
//! and faked the 3D turn by clipping a flat page. Here each motion is an
//! [`AnimatedValue`] bound to a view's transform; the platform runs it on
//! its display link (Core Animation layers on macOS), so a moving page
//! costs no layout and no re-render, and the turnstile is a real rotation
//! in perspective.

use std::cell::Cell;
use std::rc::Rc;

use nsh_core::prelude::*;
use nsh_core::{TransformKind, element::{AnimationEnd, TransformOrigin}};

use crate::theme::reduced_motion;

/// Silverlight's `ExponentialEase` (EaseOut, exponent 6), the curve of every
/// WP Toolkit transition, as a cubic Bézier (within 1% of the exponential).
pub const EXP6_OUT: Easing = Easing::Bezier(0.15, 0.95, 0.4, 1.0);
/// The ease-in counterpart, for elements leaving.
pub const EXP6_IN: Easing = Easing::Bezier(0.6, 0.0, 0.85, 0.05);
/// Metro's ease for the live tile's slide.
pub const IN_OUT_CUBIC: Easing = Easing::Curve(Curve::Cubic, EaseMode::InOut);

/// The exponential ease-out itself, for code that samples it.
pub fn ease_out_exp6(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    (1. - (-6. * t).exp()) / (1. - (-6f32).exp())
}

/// A turnstile entrance, in ms.
pub const ENTER: f32 = 350.;
/// Per-item stagger in a turnstile feather, in ms.
pub const FEATHER: f32 = 40.;
/// How long a page takes to turnstile out, in ms.
pub const EXIT: f32 = 250.;
/// Where a forward entrance starts its swing: edge-on.
pub const FORWARD_IN_DEG: f32 = -90.;
/// Where a back entrance starts its swing.
pub const BACKWARD_IN_DEG: f32 = 90.;
/// Where a forward exit swings to.
pub const FORWARD_OUT_DEG: f32 = 90.;
/// Where a back exit swings to.
pub const BACKWARD_OUT_DEG: f32 = -90.;
/// Camera distance for the projection, in px (Silverlight's `PlaneProjection`).
pub const PERSPECTIVE: f32 = 1000.;

/// Which way navigation went; decides where a turnstile swings in from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    /// Forward navigation: swings in with its right edge toward the viewer.
    #[default]
    Forward,
    /// Back navigation: swings in with its right edge away from the viewer.
    Backward,
}

impl Direction {
    pub fn enter_from(self) -> f32 {
        if self == Direction::Forward { FORWARD_IN_DEG } else { BACKWARD_IN_DEG }
    }
    pub fn exit_to(self) -> f32 {
        if self == Direction::Forward { FORWARD_OUT_DEG } else { BACKWARD_OUT_DEG }
    }
    /// +1 forward, −1 back: which side a pane slides in from.
    pub fn sign(self) -> f32 {
        if self == Direction::Forward { 1. } else { -1. }
    }
}

/// A transform origin `hinge` px to the left of a view's left edge, at its
/// vertical middle: where the turnstile turns about.
pub fn hinge_origin(hinge: f32) -> TransformOrigin {
    TransformOrigin { x: px(-hinge).into(), y: relative(0.5).into(), z: Points::ZERO }
}

/// `child` turned about a vertical axis `hinge` px left of it by `angle`
/// (degrees), seen in perspective. The perspective sits on a wrapper: nsh,
/// like CSS, gives a view's children their depth.
pub fn turned(angle: &AnimatedValue, hinge: f32, child: impl IntoElement) -> View {
    view().perspective(px(PERSPECTIVE)).child(
        view()
            .flex_1()
            .transform_origin(hinge_origin(hinge))
            .bind_transform(TransformKind::RotateY, angle)
            .child(child),
    )
}

/// Turnstile-in: a fresh angle that swings `child` in about a hinge `hinge`
/// px to its left, after `delay` ms. Starts as it is built.
pub fn turnstile_in(direction: Direction, delay: f32, hinge: f32, child: impl IntoElement) -> View {
    let angle = animated_value(if reduced_motion() { 0. } else { direction.enter_from() });
    if !reduced_motion() {
        let anim = Animation::sequence([Animation::delay(delay), Animation::timing(&angle, 0., Timing::new(ENTER, EXP6_OUT))]);
        let _ = anim.start();
    }
    turned(&angle, hinge, child)
}

/// The angle of a turnstile the caller drives: swing it with
/// [`swing_in`] / [`swing_out`] and bind it with [`turned`].
pub fn turnstile_angle() -> AnimatedValue {
    animated_value(0.)
}

/// Swings `angle` in from edge-on.
pub fn swing_in(angle: &AnimatedValue, direction: Direction, delay: f32) -> AnimationEnd {
    if reduced_motion() {
        angle.set(0.);
        return angle.timing(0., 0., Easing::Linear);
    }
    angle.set(direction.enter_from());
    Animation::sequence([Animation::delay(delay), Animation::timing(angle, 0., Timing::new(ENTER, EXP6_OUT))]).start()
}

/// Swings `angle` away to edge-on.
pub fn swing_out(angle: &AnimatedValue, direction: Direction, delay: f32) -> AnimationEnd {
    if reduced_motion() {
        return angle.timing(direction.exit_to(), 0., Easing::Linear);
    }
    Animation::sequence([Animation::delay(delay), Animation::timing(angle, direction.exit_to(), Timing::new(EXIT, EXP6_IN))])
        .start()
}

/// How long a pivot pane takes to slide in, in ms.
pub const PANE_IN: f32 = 400.;
/// How far (px) a pivot pane starts from where it settles.
pub const PANE_SHIFT: f32 = 110.;
/// Extra distance per line for the pivot's line-by-line slide (the WP
/// Toolkit's `SlideInEffect`): line 1 starts this much further out than
/// line 0, line 2 twice as far. All arrive together on the same ease, so the
/// further lines trail and the pane fans in.
pub const LINE_LEAD: f32 = 70.;
/// How long the outgoing pane takes to slip away, in ms.
pub const PANE_OUT: f32 = 130.;

/// A pane's slide: progress runs 1 → 0 as it arrives; `sign` is +1 when it
/// comes from the right. Lines inside the pane read it from the context and
/// bind their own offsets to it, so one animation moves them all.
#[derive(Clone, Copy)]
pub struct PaneSlide {
    pub progress: AnimatedValue,
    pub sign: AnimatedValue,
}

impl PaneSlide {
    pub fn new() -> Self {
        Self { progress: animated_value(0.), sign: animated_value(1.) }
    }

    /// Slides the pane in from `direction`'s side.
    pub fn play(&self, direction: Direction) -> AnimationEnd {
        self.sign.set(direction.sign());
        if reduced_motion() {
            self.progress.set(0.);
            return self.progress.timing(0., 0., Easing::Linear);
        }
        self.progress.set(1.);
        self.progress.timing(0., PANE_IN, EXP6_OUT)
    }

    /// The pane's own offset: [`PANE_SHIFT`] × progress × sign.
    pub fn offset(&self) -> AnimatedNode {
        self.progress.mul(PANE_SHIFT).mul(self.sign.node())
    }

    /// A line's extra trail: [`LINE_LEAD`] × line × progress × sign.
    pub fn line_offset(&self, line: usize) -> AnimatedNode {
        self.progress.mul(LINE_LEAD * line as f32).mul(self.sign.node())
    }
}

impl Default for PaneSlide {
    fn default() -> Self {
        Self::new()
    }
}

/// One line of content (`line` 0 = the first) that trails a pane sliding
/// in around it by [`LINE_LEAD`] per line. Outside a sliding pane it is
/// left as is.
pub fn slide_line<E: Styled + IntoElement>(line: usize, el: E) -> AnyElement {
    match use_context::<PaneSlide>() {
        Some(slide) if line > 0 => view().bind_translate_x(&slide.line_offset(line)).child(el).into_element(),
        _ => el.into_element(),
    }
}

/// Where a swivel starts: turned this far about its horizontal axis.
pub const SWIVEL_IN_DEG: f32 = 70.;

/// Swivel-in (the WP Toolkit's `SwivelTransition`): `child` swings in about
/// a horizontal axis through its middle, from nearly edge-on.
pub fn swivel_in(child: impl IntoElement) -> View {
    let angle = animated_value(if reduced_motion() { 0. } else { SWIVEL_IN_DEG });
    if !reduced_motion() {
        let _ = angle.timing(0., ENTER, EXP6_OUT);
    }
    view().perspective(px(PERSPECTIVE)).child(view().bind_transform(TransformKind::RotateX, &angle).child(child))
}

/// How far a pressed element sinks.
pub const TILT_SCALE: f32 = 0.96;
/// How far (degrees) a press at an edge tilts the element toward it.
pub const TILT_DEG: f32 = 12.;

/// The tilt effect (the Toolkit's `TiltEffect`): pressed, `el` tilts toward
/// the pointer, its pressed side pushed back, and springs back on release.
/// gpui-metro could only shrink it; here it turns in 3D.
pub fn tilt<E: Styled + Interactive + IntoElement>(el: E) -> View {
    if reduced_motion() {
        return view().child(el);
    }
    let (rx, ry, scale) = (animated_value(0.), animated_value(0.), animated_value(1.));
    let size = Rc::new(Cell::new((1f32, 1f32)));
    let measure = size.clone();
    let press = move |e: &PressEvent| {
        let (w, h) = size.get();
        let dx = (f32::from(e.position.x) / w.max(1.) - 0.5) * 2.;
        let dy = (f32::from(e.position.y) / h.max(1.) - 0.5) * 2.;
        let _ = ry.timing(dx.clamp(-1., 1.) * TILT_DEG, 60., EXP6_OUT);
        let _ = rx.timing(-dy.clamp(-1., 1.) * TILT_DEG, 60., EXP6_OUT);
        let _ = scale.timing(TILT_SCALE, 60., EXP6_OUT);
    };
    let release = move || {
        for v in [rx, ry] {
            let _ = v.timing(0., 180., EXP6_OUT);
        }
        let _ = scale.timing(1., 180., EXP6_OUT);
    };
    view()
        .perspective(px(PERSPECTIVE))
        .on_layout(move |b: &Bounds<Points>| measure.set((f32::from(b.size.width), f32::from(b.size.height))))
        .child(
            el.bind_transform(TransformKind::RotateX, &rx)
                .bind_transform(TransformKind::RotateY, &ry)
                .bind_scale(&scale)
                .on_press_down(press)
                .on_press_up(release),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp6_bezier_tracks_the_exponential() {
        // Sample the Bézier the way a timing animation does.
        let Easing::Bezier(x1, y1, x2, y2) = EXP6_OUT else { unreachable!() };
        let bez = |x: f32| {
            let (mut lo, mut hi) = (0f32, 1f32);
            for _ in 0..40 {
                let t = (lo + hi) / 2.;
                let bx = 3. * (1. - t).powi(2) * t * x1 + 3. * (1. - t) * t * t * x2 + t.powi(3);
                if bx < x { lo = t } else { hi = t }
            }
            let t = (lo + hi) / 2.;
            3. * (1. - t).powi(2) * t * y1 + 3. * (1. - t) * t * t * y2 + t.powi(3)
        };
        for i in 0..=20 {
            let x = i as f32 / 20.;
            assert!((bez(x) - ease_out_exp6(x)).abs() < 0.012, "at {x}");
        }
    }
}
