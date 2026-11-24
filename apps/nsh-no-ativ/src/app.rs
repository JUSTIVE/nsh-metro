//! The desktop shell, Windows 8 style: the page on top of the back stack in
//! the full window (turnstiling between pages), a round back button in the
//! left margin, the page's app bar along the bottom, and whatever is
//! interrupting (message box, toast, the settings flyout).
//!
//! gpui kept all of this in one entity that re-rendered on every change.
//! Here it is [`App`], a `Copy` handle of signals: the store sits behind
//! two revision signals, `rev` (the ledger changed: a harvest, a filing)
//! and `tick` (telemetry moved), so a page rebuilds only for the first and
//! binds the second to the few properties that show live readings.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use nsh_core::prelude::*;
use nsh_metro::{
    FlyoutWidth, Icon, Mode, PAGE_MARGIN, TextStyle, Theme, list_picker, message_box,
    motion::{Direction, swivel_in},
    page_host, palette, round_button, settings_flyout, text,
    theme::{FONT_FAMILY, SEMIBOLD, size},
    toggle_switch, u,
};

use crate::data::{seed, to_fixed};
use crate::nav::{ActiveToast, Dialog, Page, Shown};
use crate::pages;
use crate::store::Store;

/// Telemetry interval.
pub const TICK: Duration = Duration::from_secs(2);
pub const TOAST_FOR: Duration = Duration::from_secs(3);
/// Windows 8 pages start content 120 px in; the components already pad by
/// the phone margin, so the shell adds the rest. The back button sits here.
pub const LEFT_GUTTER: f32 = 120. - PAGE_MARGIN;

/// The app's state, as signals: copy it into any handler.
#[derive(Clone, Copy)]
pub struct App {
    store: Stored<Rc<RefCell<Store>>>,
    /// Bumps when the ledger changes (not on telemetry ticks).
    rev: Signal<u64>,
    /// Bumps on every telemetry tick.
    tick: Signal<u64>,
    stack: Stored<Vec<Page>>,
    serial: Stored<usize>,
    pub depth: Signal<usize>,
    pub current: Signal<(Shown, Direction)>,
    pub dialog: Signal<Option<Dialog>>,
    pub toast: Signal<Option<ActiveToast>>,
    toasts: Stored<usize>,
    /// The quick-settings flyout on the start screen.
    pub flyout: Signal<bool>,
    /// A text box has the keyboard focus: back dismisses the keyboard first.
    pub editing: Signal<bool>,
    /// The app's own owner: timers started from a page's handler outlive it.
    owner: Owner,
}

impl App {
    /// The app, with `boot` (for `NO_ATIV_PAGE`) opened over the start screen.
    pub fn new(boot: Option<&str>) -> Self {
        let app = App {
            store: stored(Rc::new(RefCell::new(Store::new(seed())))),
            rev: signal(0),
            tick: signal(0),
            stack: stored(vec![Page::Start]),
            serial: stored(0),
            depth: signal(1),
            current: signal((Shown { serial: 0, page: Page::Start }, Direction::Forward)),
            dialog: signal(None),
            toast: signal(None),
            toasts: stored(0),
            flyout: signal(false),
            editing: signal(false),
            owner: Owner::current(),
        };
        if let Some(page) = boot.and_then(|name| app.boot_page(name)) {
            app.push(page);
        }
        app
    }

    pub fn boot_page(&self, name: &str) -> Option<Page> {
        self.peek(|store| {
            let first_live = store.live_runs().next().map(|r| r.id.clone());
            Some(match name {
                "runs" => Page::Runs { pivot: 0 },
                "run" => Page::Run { id: first_live? },
                "telemetry" => Page::Telemetry { zone: 0 },
                "strains" => Page::Strains,
                "strain" => Page::Strain { id: store.data.strains.first()?.id.clone() },
                "incidents" => Page::Incidents { pivot: 0 },
                "incident" => Page::Incident { id: store.open_incidents().next()?.id.clone() },
                "new-incident" => Page::NewIncident { run: None },
                "reagents" => Page::Reagents { pivot: 0 },
                "shipments" => Page::Shipments { pivot: 0 },
                "freezer" => Page::Freezer,
                "deck" => Page::Deck { pivot: 0 },
                "people" => Page::People { pivot: 0 },
                "settings" => Page::Settings,
                _ => return None,
            })
        })
    }

    /* the store ----------------------------------------------------------- */

    /// Reads the store without subscribing.
    pub fn peek<R>(&self, f: impl FnOnce(&Store) -> R) -> R {
        let store = self.store.get();
        let s = store.borrow();
        f(&s)
    }

    /// Reads the store; the running computation re-runs when the ledger changes.
    pub fn read<R>(&self, f: impl FnOnce(&Store) -> R) -> R {
        self.rev.with(|_| ());
        self.peek(f)
    }

    /// Reads the store; re-runs on ledger changes and telemetry ticks.
    pub fn live<R>(&self, f: impl FnOnce(&Store) -> R) -> R {
        self.tick.with(|_| ());
        self.read(f)
    }

    /// Changes the ledger.
    pub fn mutate<R>(&self, f: impl FnOnce(&mut Store) -> R) -> R {
        let r = {
            let store = self.store.get();
            let mut s = store.borrow_mut();
            f(&mut s)
        };
        self.rev.update(|v| *v += 1);
        r
    }

    /// One telemetry interval.
    pub fn tick(&self) {
        self.store.get().borrow_mut().tick();
        self.tick.update(|v| *v += 1);
    }

    pub fn set_expiry_window(&self, days: i64) {
        if self.peek(|s| s.expiry_window) != days {
            self.mutate(|s| s.set_expiry_window(days));
        }
    }

    /* navigation --------------------------------------------------------- */

    pub fn top(&self) -> Page {
        self.stack.with(|s| s.last().cloned().expect("the start screen is never popped"))
    }

    pub fn push(&self, page: Page) {
        self.stack.with(|s| s.push(page));
        self.navigated(Direction::Forward);
    }

    pub fn pop(&self) {
        self.pop_with(|_| ());
    }

    /// Pops, then lets `f` adjust the page it uncovers before it shows.
    pub fn pop_with(&self, f: impl FnOnce(&mut Page)) {
        let popped = self.stack.with(|s| {
            if s.len() > 1 {
                s.pop();
                if let Some(top) = s.last_mut() {
                    f(top);
                }
                true
            } else {
                false
            }
        });
        if popped {
            self.navigated(Direction::Backward);
        }
    }

    /// The hardware back button: dismisses the keyboard, then a message box
    /// or the flyout, then the page.
    pub fn back(&self) {
        if self.editing.get_untracked() {
            if platform_runs() {
                dismiss_keyboard();
            }
            self.editing.set(false);
        } else if self.dialog.get_untracked().is_some() {
            self.dialog.set(None);
        } else if self.flyout.get_untracked() {
            self.flyout.set(false);
        } else {
            self.pop();
        }
    }

    fn navigated(&self, direction: Direction) {
        let serial = self.serial.with(|n| {
            *n += 1;
            *n
        });
        let depth = self.stack.with(|s| s.len());
        self.editing.set(false);
        self.flyout.set(false);
        self.depth.set(depth);
        self.current.set((Shown { serial, page: self.top() }, direction));
    }

    /// Remembers the selected pivot item or panorama section on the top
    /// entry, so coming back to it restores it (without navigating).
    pub fn set_selected(&self, ix: usize) {
        self.stack.with(|s| {
            if let Some(sel) = s.last_mut().and_then(Page::selected_mut) {
                *sel = ix;
            }
        });
    }

    /// Opens the new-incident form, optionally pre-linked to a run.
    pub fn new_incident(&self, run_id: Option<String>) {
        self.push(Page::NewIncident { run: run_id });
    }

    /* interruptions ------------------------------------------------------ */

    pub fn show(&self, dialog: Dialog) {
        self.dialog.set(Some(dialog));
    }

    pub fn dismiss(&self) {
        self.dialog.set(None);
    }

    /// The message box's "harvest": closes it, harvests, and says so.
    pub fn harvest(&self, run_id: &str) {
        self.dialog.set(None);
        let result = self.mutate(|s| s.harvest(run_id).map(|r| (r.code.clone(), r.titer.clone())));
        match result {
            Ok((code, titer)) => self.toast("harvest", Icon::Flask, code, format!("harvested · titer {titer}")),
            Err(e) => self.show(Dialog::Notice { title: "can't harvest".into(), message: e.to_string() }),
        }
    }

    pub fn toast(&self, kind: &'static str, icon: Icon, title: impl Into<String>, message: impl Into<String>) {
        let serial = self.toasts.with(|n| {
            *n += 1;
            *n
        });
        self.toast.set(Some(ActiveToast { kind, title: title.into(), message: message.into(), icon, serial }));
        let toast = self.toast;
        if !platform_runs() {
            return;
        }
        self.owner.run(|| {
            set_timeout(TOAST_FOR, move || {
                if toast.get_untracked().is_some_and(|t| t.serial == serial) {
                    toast.set(None);
                }
            })
        });
    }

    pub fn copy(&self, text: String) {
        set_clipboard(text.clone());
        self.toast("copied", Icon::Check, "copied", text);
    }

    /// Starts the telemetry ticker for as long as the app lives.
    pub fn start_ticker(&self) {
        let app = *self;
        self.owner.run(|| set_interval(TICK, move || app.tick()));
    }
}

/// Whether a platform engine is running (not in the headless tests, where
/// timers and the keyboard aren't there).
fn platform_runs() -> bool {
    try_os().is_some()
}

/* the shell ------------------------------------------------------------------ */

/// The whole window.
pub fn shell(app: App) -> AnyElement {
    // Escape and backspace are the phone's back button (backspace only
    // while no text field is using it).
    on_key("escape", move |_| app.back());
    on_key("backspace", move |_| app.back());

    let back = move || {
        view()
            .absolute()
            .left(u(40.))
            .top(u(44.))
            .child(round_button(Icon::Back).diameter(40.).on_click(move || app.back()))
    };
    view()
        .relative()
        .flex_1()
        .bg(move || palette().background)
        .child(page_host(move || app.current.get(), 0., move |shown: Shown| pages::render(app, shown.page)))
        .child(when(move || app.depth.get() > 1, back))
        .child(quick_settings(app))
        .child(dynamic(move || dialog(app)))
        .child(toast_host(app))
        .into_element()
}

fn dialog(app: App) -> AnyElement {
    let Some(dialog) = app.dialog.get() else { return view().into_element() };
    match dialog {
        Dialog::Harvest { run_id } => {
            let Some((code, vessel, od)) = app.peek(|s| s.run(&run_id).map(|r| (r.code.clone(), r.vessel.clone(), r.od600)))
            else {
                return view().into_element();
            };
            message_box(format!("harvest {code}?"))
                .message(format!(
                    "The culture leaves {vessel} at OD600 {} and goes downstream. This can't be undone.",
                    to_fixed(od, 1)
                ))
                .button("harvest", move || app.harvest(&run_id))
                .button("cancel", move || app.dismiss())
                .on_dismiss(move || app.dismiss())
                .into_element()
        }
        Dialog::Notice { title, message } => message_box(title)
            .message(message)
            .button("ok", move || app.dismiss())
            .on_dismiss(move || app.dismiss())
            .into_element(),
    }
}

/// A desktop toast: a box in the top-right corner. A new toast of the same
/// kind changes the text of the one showing; a new kind swivels in.
fn toast_host(app: App) -> AnyElement {
    let toast = app.toast;
    let kind = computed(move || toast.with(|t| t.as_ref().map(|t| t.kind)));
    let ink = move || palette().accent_ink;
    let field = move |f: fn(&ActiveToast) -> String| move || toast.with(|t| t.as_ref().map(f).unwrap_or_default());
    dynamic(move || {
        if kind.get().is_none() {
            return view().into_element();
        }
        let icon = toast.get_untracked().map(|t| t.icon).unwrap_or(Icon::Check);
        let band = view()
            .w_full()
            .flex_row()
            .items_center()
            .gap(u(10.))
            .px(u(16.))
            .h(u(60.))
            .bg(move || palette().accent)
            .cursor_pointer()
            .on_press(move || toast.set(None))
            .child(icon.render(22., ink))
            .child(
                nsh_core::prelude::text(field(|t| t.title.clone()))
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::NORMAL))
                    .font_weight(SEMIBOLD)
                    .text_color(ink),
            )
            .child(
                nsh_core::prelude::text(field(|t| t.message.clone()))
                    .flex_1()
                    .min_w(px(0.))
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::NORMAL))
                    .truncate()
                    .text_color(ink),
            );
        view().absolute().top(u(24.)).right(u(24.)).w(u(380.)).h(u(60.)).child(swivel_in(band)).into_element()
    })
}

/// The settings charm: background, accent and motion, from the start screen.
fn quick_settings(app: App) -> AnyElement {
    settings_flyout(move || app.flyout.get())
        .title("settings")
        .width(FlyoutWidth::Narrow)
        .on_dismiss(move || app.flyout.set(false))
        .child(|| {
            list_picker([Mode::Dark.label(), Mode::Light.label()], move || usize::from(Theme::get().mode == Mode::Light))
                .header("background")
                .on_select(|ix| Theme::set_mode(if ix == 0 { Mode::Dark } else { Mode::Light }))
        })
        .child(|| {
            view()
                .gap(u(8.))
                .child(text(TextStyle::Subtle, "accent colour"))
                .child(pages::settings::accent_grid(52.))
                .child(text(TextStyle::Accent, || Theme::get().accent.name()))
        })
        .child(|| {
            toggle_switch(|| Theme::get().reduced_motion)
                .header("reduce motion")
                .on_change(Theme::set_reduced_motion)
        })
        .child(move || {
            view().pt(u(8.)).child(nsh_metro::hyperlink("all settings").on_click(move || {
                app.flyout.set(false);
                app.push(Page::Settings);
            }))
        })
        .into_element()
}

#[cfg(test)]
mod tests {
    use nsh_core::element::{Runtime, create_root};

    use super::*;
    use crate::data::RunPhase;

    const PAGES: [&str; 14] = [
        "runs",
        "run",
        "telemetry",
        "strains",
        "strain",
        "incidents",
        "incident",
        "new-incident",
        "reagents",
        "shipments",
        "freezer",
        "deck",
        "people",
        "settings",
    ];

    /// Runs `check` on an app made in a fresh runtime (no window: the state
    /// and navigation only).
    fn with_app(boot: Option<&str>, check: impl FnOnce(App)) {
        let runtime = Runtime::default();
        let _entered = runtime.enter();
        let (owner, app) = create_root(|| App::new(boot));
        check(app);
        owner.dispose();
    }

    #[test]
    fn every_page_boots_and_back_returns_to_start() {
        for page in PAGES {
            with_app(Some(page), |app| {
                assert_eq!(app.depth.get_untracked(), 2, "{page} boots over the start screen");
                assert_ne!(app.current.get_untracked().0.page, Page::Start);
                app.back();
                assert_eq!(app.depth.get_untracked(), 1, "back pops {page}");
                assert_eq!(app.current.get_untracked().0.page, Page::Start);
                app.back();
                assert_eq!(app.depth.get_untracked(), 1, "the start screen is never popped");
            });
        }
    }

    #[test]
    fn back_dismisses_the_keyboard_then_the_dialog_then_the_page() {
        with_app(Some("new-incident"), |app| {
            app.editing.set(true);
            app.show(Dialog::Notice { title: "t".into(), message: "m".into() });
            app.back();
            assert!(!app.editing.get_untracked(), "the first back dismisses the keyboard");
            assert_eq!(app.depth.get_untracked(), 2);
            app.back();
            assert!(app.dialog.get_untracked().is_none(), "then the message box");
            assert_eq!(app.depth.get_untracked(), 2);
            app.back();
            assert_eq!(app.depth.get_untracked(), 1, "then the page");
        });
    }

    #[test]
    fn the_pivot_is_remembered_on_the_stack() {
        with_app(None, |app| {
            app.push(Page::Runs { pivot: 0 });
            app.set_selected(2);
            app.push(Page::Freezer);
            app.pop();
            assert_eq!(app.current.get_untracked().0.page, Page::Runs { pivot: 2 });
        });
    }

    #[test]
    fn each_navigation_is_a_new_key() {
        with_app(None, |app| {
            app.push(Page::Freezer);
            let first = app.current.get_untracked().0;
            app.pop();
            app.push(Page::Freezer);
            assert_ne!(app.current.get_untracked().0, first, "the same page turns in again");
        });
    }

    #[test]
    fn harvest_from_the_message_box() {
        with_app(Some("run"), |app| {
            let Page::Run { id } = app.top() else { unreachable!() };
            app.show(Dialog::Harvest { run_id: id.clone() });
            let rev = app.rev.get_untracked();
            app.harvest(&id);
            assert!(app.dialog.get_untracked().is_none());
            assert_eq!(app.peek(|s| s.run(&id).unwrap().phase), RunPhase::Harvested);
            assert!(app.rev.get_untracked() > rev, "the ledger changed");
            assert!(app.toast.get_untracked().is_some(), "a toast confirms it");
            app.harvest(&id);
            assert!(matches!(app.dialog.get_untracked(), Some(Dialog::Notice { .. })), "twice is refused");
        });
    }

    #[test]
    fn a_toast_of_the_same_kind_updates_in_place() {
        with_app(None, |app| {
            app.toast("refreshed", Icon::Refresh, "refreshed", "first");
            app.toast("refreshed", Icon::Refresh, "refreshed", "second");
            let t = app.toast.get_untracked().expect("a toast is up");
            assert_eq!((t.kind, t.message.as_str()), ("refreshed", "second"));
        });
    }

    #[test]
    fn a_tick_moves_telemetry_but_not_the_ledger() {
        with_app(None, |app| {
            let (rev, tick) = (app.rev.get_untracked(), app.tick.get_untracked());
            let before = app.peek(|s| s.run("r1").map(|r| r.od600)).unwrap();
            app.tick();
            assert_eq!(app.rev.get_untracked(), rev);
            assert_eq!(app.tick.get_untracked(), tick + 1);
            let (od, phase) = app.peek(|s| s.run("r1").map(|r| (r.od600, r.phase))).unwrap();
            assert!(od > before || phase != RunPhase::Log, "the ticker grew the culture");
        });
    }

    #[test]
    fn the_expiry_window_only_writes_on_a_change() {
        with_app(None, |app| {
            let rev = app.rev.get_untracked();
            let days = app.peek(|s| s.expiry_window);
            app.set_expiry_window(days);
            assert_eq!(app.rev.get_untracked(), rev);
            app.set_expiry_window(days + 5);
            assert_eq!(app.peek(|s| s.expiry_window), days + 5);
            assert_eq!(app.rev.get_untracked(), rev + 1);
        });
    }
}
