//! The performance incubator, for nsh-metro: the same stress scenes as
//! `apps/perf-incubator` (gpui-metro), measured the same way, so the two
//! ports can be compared scene by scene.
//!
//! ```text
//! cargo run --release -p nsh-perf-incubator                  # the whole suite
//! cargo run --release -p nsh-perf-incubator -- live-tiles-60 # one scene
//! cargo run --release -p nsh-perf-incubator -- --show live-tiles-60
//! ```
//!
//! Options: `--secs N` (default 20), `--warmup N` (default 3), `--out FILE`,
//! `--beat N` (live tiles slide together every N seconds).
//!
//! CPU % is user + system time of this process over the window (100 % = one
//! core). On nsh the platform composites and animates the layers, and on
//! macOS that work lands partly in WindowServer, so the table also shows
//! WindowServer's CPU over the same window (both incubators report it).

use std::time::{Duration, Instant};

use nsh_core::{element::desktop::window, prelude::*};
use nsh_metro::{
    Icon, TextStyle, TileSize, list_item, panorama, pivot, progress_bar, progress_dots, progress_ring, text,
    theme::{palette, size as type_size, u},
    tile, tile_grid,
};
use nsh_platform_macos as platform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scene {
    Idle,
    StaticTiles(usize),
    LiveTiles(usize),
    /// Live tiles whose back is a closure face (gpui-metro's unkeyed path).
    LiveTilesUnkeyed(usize),
    Progress,
    PivotPaging,
    PanoramaPaging,
    List(usize),
}

impl Scene {
    const SUITE: [Scene; 10] = [
        Scene::Idle,
        Scene::StaticTiles(60),
        Scene::LiveTiles(12),
        Scene::LiveTiles(60),
        Scene::LiveTiles(120),
        Scene::LiveTilesUnkeyed(60),
        Scene::Progress,
        Scene::PivotPaging,
        Scene::PanoramaPaging,
        Scene::List(200),
    ];

    fn name(self) -> String {
        match self {
            Scene::Idle => "idle".into(),
            Scene::StaticTiles(n) => format!("static-tiles-{n}"),
            Scene::LiveTiles(n) => format!("live-tiles-{n}"),
            Scene::LiveTilesUnkeyed(n) => format!("live-tiles-{n}-unkeyed"),
            Scene::Progress => "progress".into(),
            Scene::PivotPaging => "pivot-paging".into(),
            Scene::PanoramaPaging => "panorama-paging".into(),
            Scene::List(n) => format!("list-{n}"),
        }
    }

    fn find(name: &str) -> Option<Scene> {
        Scene::SUITE.into_iter().find(|s| s.name() == name).or_else(|| {
            let (head, n) = name.rsplit_once('-')?;
            let n: usize = n.parse().ok()?;
            match head {
                "live-tiles" => Some(Scene::LiveTiles(n)),
                "static-tiles" => Some(Scene::StaticTiles(n)),
                "list" => Some(Scene::List(n)),
                _ => None,
            }
        })
    }
}

/// User + system CPU seconds this process has used.
fn cpu_seconds() -> f64 {
    // SAFETY: getrusage only writes into the struct we hand it.
    let mut u: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut u) };
    let t = |v: libc::timeval| v.tv_sec as f64 + v.tv_usec as f64 / 1e6;
    t(u.ru_utime) + t(u.ru_stime)
}

/// WindowServer's CPU seconds so far, from `ps` (0 if it can't be read).
fn window_server_seconds() -> f64 {
    let out = std::process::Command::new("ps").args(["-axco", "time=,comm="]).output();
    let Ok(out) = out else { return 0. };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.trim_end().ends_with("WindowServer"))
        .filter_map(|l| l.split_whitespace().next())
        .map(|t| {
            // [[hh:]mm:]ss.cc
            t.split(':').fold(0., |acc, part| acc * 60. + part.parse::<f64>().unwrap_or(0.))
        })
        .sum()
}

struct Options {
    scenes: Vec<Scene>,
    secs: u64,
    warmup: u64,
    out: Option<String>,
    show_only: bool,
    beat: Option<f32>,
}

fn options() -> Options {
    let mut o = Options { scenes: Vec::new(), secs: 20, warmup: 3, out: None, show_only: false, beat: None };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--secs" => o.secs = args.next().and_then(|v| v.parse().ok()).unwrap_or(o.secs),
            "--warmup" => o.warmup = args.next().and_then(|v| v.parse().ok()).unwrap_or(o.warmup),
            "--out" => o.out = args.next(),
            "--beat" => o.beat = args.next().and_then(|v| v.parse().ok()),
            "--show" => o.show_only = true,
            name => match Scene::find(name) {
                Some(s) => o.scenes.push(s),
                None => eprintln!("unknown scene {name:?}"),
            },
        }
    }
    if o.scenes.is_empty() {
        o.scenes = Scene::SUITE.to_vec();
    }
    o
}

const SIZES: [TileSize; 10] = [
    TileSize::Medium,
    TileSize::Small,
    TileSize::Small,
    TileSize::Small,
    TileSize::Small,
    TileSize::Wide,
    TileSize::Medium,
    TileSize::Small,
    TileSize::Small,
    TileSize::Medium,
];

const ICONS: [Icon; 6] = [Icon::Mail, Icon::Phone, Icon::Camera, Icon::Calendar, Icon::People, Icon::Clock];

fn back_face(i: usize) -> impl Fn(TileSize) -> AnyElement + 'static {
    move |s| {
        let big = if s == TileSize::Small { type_size::MEDIUM } else { type_size::LARGE };
        view()
            .size_full()
            .p(u(8.))
            .child(nsh_core::prelude::text(format!("R-{}", 4700 + i)).text_size(u(big)).text_color(rgb(0xFFFFFF)))
            .child(nsh_core::prelude::text(format!("OD {}.{} · log", i % 12, i % 10)).text_size(u(type_size::SMALL)).text_color(rgb(0xFFFFFF)))
            .child(nsh_core::prelude::text("BR-2K · 2 L").text_size(u(type_size::SMALL)).text_color(rgb(0xFFFFFF)))
            .into_element()
    }
}

fn tiles(n: usize, live: bool, keyed: bool) -> AnyElement {
    let grid = tile_grid().columns(16).metrics(64., 8.).tiles((0..n).map(|i| {
        let t = tile(format!("t{i}"), SIZES[i % SIZES.len()])
            .title(format!("tile {i}"))
            .icon(ICONS[i % ICONS.len()])
            .count((i % 7) as u32);
        if live {
            let t = t.period(9. + (i % 7) as f32).phase((i as f32 * 0.37).fract());
            if keyed {
                t.back_text(format!("R-{}", 4700 + i), [format!("OD {}.{} · log", i % 12, i % 10), "BR-2K · 2 L".into()])
            } else {
                t.back(back_face(i))
            }
        } else {
            t
        }
    }));
    view().size_full().overflow_hidden().p(u(24.)).child(grid).into_element()
}

fn mail_pane(folder: usize) -> AnyElement {
    view()
        .size_full()
        .px(u(24.))
        .children((0..12).map(|i| {
            list_item(format!("sender {i}"))
                .subtitle(format!("subject line {i} in folder {folder}"))
                .detail("a preview of the message body that runs on")
        }))
        .into_element()
}

fn scene(s: Scene, page: Signal<usize>) -> AnyElement {
    match s {
        Scene::Idle => view().size_full().p(u(24.)).child(text(TextStyle::Title1, "idle")).into_element(),
        Scene::StaticTiles(n) => tiles(n, false, false),
        Scene::LiveTiles(n) => tiles(n, true, true),
        Scene::LiveTilesUnkeyed(n) => tiles(n, true, false),
        Scene::Progress => view()
            .size_full()
            .p(u(24.))
            .gap(u(32.))
            .child(progress_dots())
            .child(progress_bar(0.4))
            .child(progress_ring().diameter(64.))
            .into_element(),
        Scene::PivotPaging => pivot(["inbox", "unread", "flagged", "urgent"], move || page.get() % 4)
            .title("perf")
            .panes((0..4).map(|i| move || mail_pane(i)))
            .into_element(),
        Scene::PanoramaPaging => {
            let mut p = panorama("incubator", move || page.get() % 4).section_width(560.);
            for i in 0..4 {
                p = p.section(format!("section {i}"), move || mail_pane(i));
            }
            p.into_element()
        }
        Scene::List(n) => view()
            .size_full()
            .overflow_hidden()
            .px(u(24.))
            .children((0..n).map(|i| list_item(format!("row {i}")).subtitle("subtitle").detail("detail")))
            .into_element(),
    }
}

#[component]
fn Lab() -> impl IntoElement {
    let o = options();
    nsh_metro::set_live_tile_beat(o.beat);
    let current = signal(o.scenes[0]);
    let page = signal(0usize);
    // Pivot and panorama scenes page on their own every second.
    let _pager = set_interval(Duration::from_secs(1), move || page.update(|p| *p += 1));
    if !o.show_only {
        spawn(async move {
            let mut table = String::from("| scene | CPU % | WindowServer CPU % |\n|---|---:|---:|\n");
            for s in o.scenes.clone() {
                current.set(s);
                timer(Duration::from_secs(o.warmup)).await;
                let (c0, w0, t0) = (cpu_seconds(), window_server_seconds(), Instant::now());
                timer(Duration::from_secs(o.secs)).await;
                let (c1, w1, dt) = (cpu_seconds(), window_server_seconds(), t0.elapsed().as_secs_f64());
                let (cpu, ws) = ((c1 - c0) / dt * 100., (w1 - w0) / dt * 100.);
                eprintln!("{:<22} {:>6.1} % CPU {:>6.1} % WindowServer", s.name(), cpu, ws);
                table += &format!("| {} | {:.1} | {:.1} |\n", s.name(), cpu, ws);
            }
            println!("{table}");
            if let Some(path) = &o.out {
                std::fs::write(path, &table).ok();
            }
            std::process::exit(0);
        });
    }
    window().id("main").title("nsh-metro perf incubator").default_size(Size { width: px(1280.), height: px(900.) }).content(move || {
        view()
            .flex_1()
            .bg(move || palette().background)
            .child(dynamic(move || scene(current.get(), page)))
    })
}

fn main() -> impl std::process::Termination {
    platform::run_app(
        platform::AppSource::Rust(platform::Root::of::<Lab>()),
        platform::AppOptions { title: "nsh-metro perf incubator".to_owned(), size: (1280, 900), ..Default::default() },
    )
}
