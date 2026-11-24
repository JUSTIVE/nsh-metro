//! The start screen: one live tile per destination, each showing a real
//! reading from the store. Telemetry ticks every two seconds; the tiles
//! that quote it bind their faces, so they follow without a rebuild.

use std::rc::Rc;

use nsh_core::prelude::*;
use nsh_metro::{Accent, Icon, PAGE_MARGIN, TextStyle, TileSize, app_bar, text, tile, tile_grid, u};

use crate::app::{App, LEFT_GUTTER};
use crate::data::{LotState, SensorState};
use crate::nav::Page;
use crate::pages::PageView;
use crate::store::reading;
use crate::widgets::{Line, line, live_back, live_front};

/// Distance from the window edge to the first group (the shell's gutter
/// plus the page margin), for the turnstile hinge.
const START_INSET: f32 = LEFT_GUTTER + PAGE_MARGIN;

/// What the tiles show that only a ledger change moves (the start screen
/// is rebuilt on every visit, so these are read once).
struct Counts {
    live: usize,
    open: usize,
    first_incident: Option<(String, String, String)>,
    en_route: usize,
    next_shipment: Option<(String, String, String)>,
    expiring: usize,
    first_expiring: Option<(String, String, String)>,
    filled: usize,
    slots: usize,
    strains: usize,
    deck: usize,
    people: usize,
}

pub fn render(app: App) -> PageView {
    let c = app.peek(|s| {
        let open: Vec<_> = s.open_incidents().collect();
        let mut expiring: Vec<_> = s.lots_in(LotState::Expiring).collect();
        expiring.sort_by_key(|l| l.shelf_days);
        let (filled, slots) = s.freezer_occupancy();
        Counts {
            live: s.live_runs().count(),
            open: open.len(),
            first_incident: open.first().map(|i| (i.code.clone(), i.title.clone(), i.severity.as_str().to_string())),
            en_route: s.data.shipments.iter().filter(|s| s.eta_days > 0).count(),
            next_shipment: s.next_shipment().map(|sh| {
                (
                    format!("{} {}", sh.code, sh.eta),
                    format!("→ {}", sh.destination),
                    format!("{} · {} · {} vials", sh.stage.as_str(), sh.courier, sh.vials),
                )
            }),
            expiring: expiring.len(),
            first_expiring: expiring.first().map(|l| (l.expires.clone(), l.name.clone(), l.code.clone())),
            filled,
            slots,
            strains: s.active_strains(),
            deck: s.deck_today().count(),
            people: s.active_members(),
        }
    });

    // The first live culture's phase and OD move with every tick.
    let runs = {
        let t = tile("runs", TileSize::Medium).title("runs").icon(Icon::Flask).count(c.live as u32);
        let first = move |f: fn(&crate::data::Run) -> String| {
            line(move || app.live(|s| s.live_runs().next().map(f).unwrap_or_default()))
        };
        if c.live > 0 {
            let head = first(|r| r.code.clone());
            let lines = vec![first(|r| format!("{} · OD {}", r.phase.as_str(), r.od600)), first(|r| r.vessel.clone())];
            t.back(move |size| live_back(size, head.clone(), lines.clone(), None)).phase(0.05)
        } else {
            t
        }
    };

    let incidents = {
        let t = tile("incidents", TileSize::Medium)
            .title("incidents")
            .icon(Icon::Alert)
            .count(c.open as u32)
            .color(Accent::Crimson.color());
        match c.first_incident {
            Some((code, title, severity)) => t.back_text(code, [title, severity]).phase(0.35),
            None => t,
        }
    };

    let shipments = {
        let t = tile("shipments", TileSize::Wide).title("shipments").icon(Icon::Truck).count(c.en_route as u32);
        match c.next_shipment {
            Some((head, to, how)) => t.back_text(head, [to, how]).phase(0.6).period(15.),
            None => t,
        }
    };

    // Telemetry ticks every two seconds: the alarm count, the tile's
    // colour and the worst sensor on its back all follow.
    let telemetry = {
        let alarms = move || app.live(|s| s.sensors_in(SensorState::Alarm)) as u32;
        let bg: Rc<dyn Fn() -> Hsla> =
            Rc::new(move || if alarms() > 0 { Accent::Red.color() } else { Accent::Teal.color() });
        let worst = move |f: fn(&crate::data::Sensor) -> String| -> Line {
            line(move || app.live(|s| s.worst_sensor().map(f).unwrap_or_default()))
        };
        let head = worst(|s| format!("{} {}", reading(s), s.unit));
        let lines = vec![worst(|s| s.code.clone()), worst(|s| s.zone.to_lowercase())];
        let front_bg = bg.clone();
        tile("telemetry", TileSize::Medium)
            .title("telemetry")
            .color(Accent::Teal.color())
            .front(move |size| live_front(size, Icon::Gauge, Rc::new(alarms), front_bg.clone()))
            .back(move |size| live_back(size, head.clone(), lines.clone(), Some(bg.clone())))
            .phase(0.18)
            .period(12.)
    };

    let reagents = {
        let t = tile("reagents", TileSize::Medium).title("reagents").icon(Icon::Clock).count(c.expiring as u32);
        match c.first_expiring {
            Some((expires, name, code)) => t.back_text(expires, [name, code]).phase(0.8),
            None => t,
        }
    };

    let small = |key: &'static str, icon: Icon, count: usize| {
        tile(key, TileSize::Small).title(key).icon(icon).count(count as u32)
    };
    let freezer = small("freezer", Icon::Snow, 0)
        .back_text(format!("{} %", c.filled * 100 / c.slots.max(1)), [format!("{} vials", c.filled)])
        .phase(0.45)
        .period(10.);
    let strains = small("strains", Icon::Dna, c.strains);
    let deck = small("deck", Icon::Calendar, c.deck);
    let people = small("people", Icon::People, c.people);
    let settings = small("settings", Icon::Settings, 0);

    let to = move |page: Page| move || app.push(page.clone());
    // The Windows 8 start screen: named groups side by side, scrolling
    // sideways. Tapping a tile swings the tiles away, the tapped one last.
    let grid = tile_grid()
        .turnstile(true)
        .hinge(START_INSET)
        .group(
            "floor",
            [
                runs.on_click(to(Page::Runs { pivot: 1 })),
                telemetry.on_click(to(Page::Telemetry { zone: 0 })),
                incidents.on_click(to(Page::Incidents { pivot: 0 })),
                deck.on_click(to(Page::Deck { pivot: 0 })),
            ],
        )
        .group("biology", [strains.on_click(to(Page::Strains))])
        .group(
            "inventory",
            [
                shipments.on_click(to(Page::Shipments { pivot: 0 })),
                freezer.on_click(to(Page::Freezer)),
                reagents.on_click(to(Page::Reagents { pivot: 0 })),
            ],
        )
        .group("org", [people.on_click(to(Page::People { pivot: 0 })), settings.on_click(to(Page::Settings))]);

    let body = view()
        .size_full()
        .child(view().px(u(PAGE_MARGIN)).pb(u(28.)).child(text(TextStyle::Title1, "start").lowercase()))
        .child(
            scroll()
                .horizontal()
                .flex_1()
                .min_h(px(0.))
                .child(view().flex_row().px(u(PAGE_MARGIN)).pb(u(48.)).child(grid).child(view().w(u(96.)))),
        );
    PageView::new(body).app_bar(
        app_bar()
            .button(Icon::Settings, "settings", move || app.flyout.set(true))
            .button(Icon::Refresh, "refresh", move || {
                app.tick();
                app.toast("refreshed", Icon::Refresh, "refreshed", "latest readings pulled from the floor");
            })
            .menu_item("new incident", move || app.new_incident(None))
            .menu_item("all settings", move || app.push(Page::Settings)),
    )
}
