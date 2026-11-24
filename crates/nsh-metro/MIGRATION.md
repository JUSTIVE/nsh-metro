# gpui → nsh migration notes

How gpui-metro was moved to [Native Shell](https://nsh.dev) (nsh 0.6.1,
macOS AppKit backend), written so another gpui project can follow the same
path. nsh-metro sits beside gpui-metro in this repository; compare any
component file by file.

## The model changes

| gpui | nsh |
|---|---|
| `Render::render` runs every frame a view is dirty; the tree is rebuilt | A `#[component]` body (or an `IntoElement::into_element`) runs **once**; values that change are closures bound to properties |
| `cx.notify()`, `Entity<T>`, `use_keyed_state` | `signal(v)`, `computed`, `effect`, `global::<T>()` (per surface) |
| Colours read from a global each render (`Theme::palette(cx)`) | Colours are bound: `.bg(move \|\| palette().accent)`; a theme change updates only those properties |
| `with_animation`, `request_animation_frame`, re-rendering each frame | `animated_value(x)` + `bind_translate_x / bind_scale / bind_opacity / bind_transform(TransformKind::RotateY, …)`; the platform runs it on Core Animation layers |
| `div()` (row when `.flex()`) | `view()` — a **column** by default; use `.flex_row()` |
| `.id(..)` everywhere for state | not needed: state lives in the closure that owns it |
| `on_click(\|e, window, cx\| …)` | `on_press(\|\| …)`, `on_press_down(\|e: &PressEvent\| …)`; handlers take no context |
| `.when(cond, …)` re-evaluated each frame | build-time `FluentBuilder::when`, or reactive `when(move \|\| cond.get(), \|\| …)` / `dynamic(move \|\| …)` |
| `children(iter)` rebuilt each frame | `each(items, key, render)` for keyed reactive lists |
| `svg().path("…")` + `AssetSource` | `svg(ImageSrc::svg_markup(…))`, tinted through `currentColor` + `.text_color(..)` (needs nsh-core's `svg` feature) |
| a 1,000-line custom `TextInput` | `input()`: the platform's own field |
| `ScrollHandle::scroll_to_top_of_item` | `element_ref()` + `.element_ref(&r)` + `r.reveal(..)` |
| `cx.spawn` + `timer` | `spawn(async { timer(d).await; … })`, `set_timeout`, `set_interval` |
| `rems` (`u(x)` = x/16 rem) | points (`u(x)` = `px(x)`) |

## What maps directly

- Style vocabulary: `p/px/gap/w/h/size/absolute/left/top/inset/bg/border/rounded/opacity/overflow_hidden/text_size/font_weight/font_family/text_color` all exist with the same names.
- Geometry types (`Point`, `Size`, `Bounds`, `Hsla`, `rgb`, `SharedString`) carry the same names.
- Pure logic (tile packing, schedules, header glides, curves) moved unchanged; only durations became `f32` ms.

## What got better

- **Real 3D.** gpui cannot transform a subtree, so gpui-metro faked the
  turnstile by clipping a flat page and the tilt by shrinking it. nsh has
  `rotate_y`, `rotate_x`, `perspective` and `transform_origin`: the
  turnstile is a rotation about the hinge (`transform_origin` at a negative
  x), and the tilt turns toward the pointer.
- **No per-frame work in the app.** A live tile's peek is one looping
  `Animation` started when it is built. The perf-incubator work on master
  (keys, cached views, a shared 60 fps clock, painting faces by hand, the
  slide beat) was all about avoiding re-renders; none of it is needed.
- **Native text input, scrolling and accessibility** for free.

## What is different or missing

- **Exit animations need a host.** An element removed from the tree is gone
  at once. `nav::page_host` keeps the old page until its turnstile-out
  finishes, then swaps; the flyout keeps itself mounted until it has slid
  out. Do the same for anything that animates away.
- **Context needs an owner.** `provide_context` applies to the current owner
  and its children; an `IntoElement` struct runs in its parent's owner, so
  provide inside a `dynamic(..)` (the pivot does this for its line slide).
- **Easing.** nsh has CSS-style curves and Béziers but no exponential; the
  WP Toolkit's `ExponentialEase(6)` is `Easing::Bezier(0.15, 0.95, 0.4, 1.0)`
  (within 1 %, see `motion::tests`).
- **Animated paths are sampled.** A dot that follows a piecewise path (the
  progress dots and ring) maps one clock through
  `AnimatedValue::interpolate` over a sampled table.
- **Window size** comes from `window().default_size(..)`, not
  `AppOptions::size`.
- **The compositor does the work.** CPU moves from the app into
  WindowServer: measure both (both incubators print WindowServer's CPU).

## Porting checklist for another project

1. Add `nsh-core`, `nsh-base` (`default-features = false`) and
   `nsh-platform-macos` (`features = ["svg"]`) from crates.io; Rust ≥ 1.97.
2. Replace the app entry with `platform::run_app(AppSource::Rust(Root::of::<App>()), …)`
   and a `#[component] fn App()` returning `window().content(..)`.
3. Turn each gpui view's fields into signals created in the component, and
   each `cx.notify()` into a `signal.set`.
4. Make every theme read a closure.
5. Replace per-frame animation with animated values; replace
   enter/exit-on-render with an effect that starts animations when a
   signal changes.
6. Measure with both incubators (app CPU and WindowServer CPU).
