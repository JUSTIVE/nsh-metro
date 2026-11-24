//! Page navigation with the turnstile: the old page swings away about the
//! left edge, then the new one swings in.

use std::cell::Cell;
use std::rc::Rc;

use nsh_core::prelude::*;

use crate::motion::{Direction, swing_in, swing_out, turned};

/// Shows `render(key)` for the current page, `current` being the page and
/// which way navigation went. A change turnstiles out the page on screen
/// (it stays as it was, scroll position and all, until it has gone), then
/// builds the new one and turnstiles it in. `hinge` is the distance from
/// the window's left edge to the host's.
pub fn page_host<K, F, E>(current: impl IntoReactive<(K, Direction)>, hinge: f32, render: F) -> AnyElement
where
    K: Clone + PartialEq + 'static,
    F: Fn(K) -> E + 'static,
    E: IntoElement,
{
    let current = current.into_reactive();
    let (first, _) = untracked(|| current.get());
    let shown = signal_with(first.clone(), SignalEquals::Never);
    let angle = animated_value(0.);
    let _ = swing_in(&angle, Direction::Forward, 0.);
    let on_screen = Rc::new(std::cell::RefCell::new(first));
    // Bumps on every navigation, so a late exit doesn't swap in a stale page.
    let epoch = Rc::new(Cell::new(0u64));
    effect(move || {
        let (key, direction) = current.get();
        if *on_screen.borrow() == key {
            return;
        }
        *on_screen.borrow_mut() = key.clone();
        let n = epoch.get() + 1;
        epoch.set(n);
        let epoch = epoch.clone();
        untracked(|| {
            let end = swing_out(&angle, direction, 0.);
            spawn(async move {
                end.await;
                if epoch.get() == n {
                    shown.set(key);
                    let _ = swing_in(&angle, direction, 0.);
                }
            });
        });
    });
    let render = Rc::new(render);
    view()
        .size_full()
        .child(turned(&angle, hinge, dynamic(move || render(shown.get()).into_element())).size_full())
        .into_element()
}
