//! The phone's page host: navigation the Metro way. The page on screen
//! turnstiles out, then the new one turnstiles in. A start screen (a tile
//! grid with its own turnstile) is different: its tiles swing away by
//! themselves in a wave, and swing in by themselves when it appears.
//!
//! `nsh_metro::page_host` turns every page as a whole, so it can't let a
//! page do its own exit; this is a local version of it that can (see the
//! library gaps in the port notes).

use std::{cell::Cell, rc::Rc, time::Duration};

use nsh_core::prelude::*;
use nsh_metro::motion::{Direction, swing_in, swing_out, turned};

use crate::pages::Page;

/// Pages that are a start screen: their tiles turnstile themselves.
pub fn is_start_screen(page: Page) -> bool {
    matches!(page, Page::TileGrid | Page::Turnstile)
}

/// A start screen's exit: the longest tile wave plus one tile's swing.
const GRID_EXIT: Duration = Duration::from_millis(620);

/// What the host tells the page it shows.
#[derive(Clone, Copy)]
pub struct Host {
    /// The page on screen (lags `current` while the old page leaves).
    pub shown: Signal<Page>,
    /// True while a start screen's tiles are swinging away.
    pub leaving: Signal<bool>,
}

/// Shows `render(page)` for the page on screen, turning between pages as
/// `current` changes. `hinge` is the distance from the turn's axis to the
/// host's left edge.
pub fn page_host<E: IntoElement>(
    current: Signal<Page>,
    hinge: f32,
    render: impl Fn(Page, Host) -> E + 'static,
) -> (AnyElement, Host) {
    let first = current.get_untracked();
    let host = Host { shown: signal(first), leaving: signal(false) };
    let angle = animated_value(0.);
    if !is_start_screen(first) {
        let _ = swing_in(&angle, Direction::Forward, 0.);
    }
    // Bumps on every navigation, so a late exit doesn't swap in a stale page.
    let epoch = Rc::new(Cell::new(0u64));
    effect(move || {
        let page = current.get();
        untracked(|| {
            let n = epoch.get() + 1;
            epoch.set(n);
            let old = host.shown.get();
            if old == page {
                // Back to the page that was leaving: cancel its exit.
                if n > 1 {
                    host.leaving.set(false);
                    if !is_start_screen(page) {
                        let _ = swing_in(&angle, Direction::Forward, 0.);
                    }
                }
                return;
            }
            // Swaps in the new page and starts its entrance.
            let arrive = move || {
                batch(|| {
                    host.shown.set(page);
                    host.leaving.set(false);
                });
                if is_start_screen(page) {
                    angle.set(0.);
                } else {
                    let _ = swing_in(&angle, Direction::Forward, 0.);
                }
            };
            if nsh_metro::theme::reduced_motion() {
                arrive();
                return;
            }
            let epoch = epoch.clone();
            if is_start_screen(old) {
                // The tiles swing away by themselves; wait for the wave.
                host.leaving.set(true);
                spawn(async move {
                    timer(GRID_EXIT).await;
                    if epoch.get() == n {
                        arrive();
                    }
                });
            } else {
                let end = swing_out(&angle, Direction::Forward, 0.);
                spawn(async move {
                    end.await;
                    if epoch.get() == n {
                        arrive();
                    }
                });
            }
        });
    });
    let render = Rc::new(render);
    let el = view()
        .size_full()
        .child(turned(&angle, hinge, dynamic(move || render(host.shown.get(), host).into_element())).size_full())
        .into_element();
    (el, host)
}
