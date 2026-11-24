//! Live tiles and the start-screen grid that packs them.
//!
//! A tile is a flat accent square with its name in the bottom-left corner
//! and, optionally, a count. A *live* tile has a back face; it periodically
//! slides up to show it (the WP7 hub-tile peek), then slides back.
//!
//! In gpui-metro every frame of a slide re-rendered the window, which is
//! what the perf incubator measured and what most of its optimisations
//! (keys, cached views, the shared ambient clock, painting faces directly)
//! were about. Here a tile's slide is one looping animation bound to a
//! transform, started once: the platform moves the layer, and the tile is
//! never built again.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use nsh_core::prelude::*;

use crate::components::text::label;

use crate::{
    Icon,
    button::ClickHandler,
    chrome::PAGE_MARGIN,
    motion::{self, EXP6_IN, EXP6_OUT, IN_OUT_CUBIC},
    theme::{FONT_FAMILY, LIGHT, SEMILIGHT, palette, reduced_motion, size, u},
};

/// Tile footprints in grid units (one unit = one small tile).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileSize {
    /// 1×1
    Small,
    /// 2×2
    Medium,
    /// 4×2
    Wide,
    /// 4×4
    Large,
}

impl TileSize {
    pub fn span(self) -> (usize, usize) {
        match self {
            TileSize::Small => (1, 1),
            TileSize::Medium => (2, 2),
            TileSize::Wide => (4, 2),
            TileSize::Large => (4, 4),
        }
    }

    /// Size in px for a given unit and gutter.
    pub fn pixels(self, unit: f32, gap: f32) -> (f32, f32) {
        let (c, r) = self.span();
        (c as f32 * unit + (c - 1) as f32 * gap, r as f32 * unit + (r - 1) as f32 * gap)
    }

    pub fn name(self) -> &'static str {
        match self {
            TileSize::Small => "small",
            TileSize::Medium => "medium",
            TileSize::Wide => "wide",
            TileSize::Large => "large",
        }
    }
}

/// Default WP8 metrics (scaled for a 480-wide page).
pub const TILE_UNIT: f32 = 99.;
pub const TILE_GAP: f32 = 12.;

/// Packs tiles the way the Windows Phone start screen does: the grid is made
/// of 2×2 blocks; medium and larger tiles align to block corners, and small
/// tiles fill a block's quadrants before moving to the next block. First
/// fit, in order. Returns `(column, row)` per tile, in grid units.
pub fn pack(sizes: &[TileSize], columns: usize) -> Vec<(usize, usize)> {
    let widest = sizes.iter().map(|s| s.span().0).max().unwrap_or(1);
    let columns = columns.max(widest).max(2);
    let mut occupied: Vec<Vec<bool>> = Vec::new();
    let free = |occ: &Vec<Vec<bool>>, c: usize, r: usize, w: usize, h: usize| {
        if c + w > columns {
            return false;
        }
        (r..r + h).all(|y| (c..c + w).all(|x| occ.get(y).is_none_or(|row| !row[x])))
    };
    let mut out = Vec::with_capacity(sizes.len());
    for &s in sizes {
        let (w, h) = s.span();
        let mut placed = None;
        'search: for by in 0.. {
            for bx in 0..columns.div_ceil(2) {
                let (c0, r0) = (bx * 2, by * 2);
                let candidates: &[(usize, usize)] =
                    if s == TileSize::Small { &[(0, 0), (1, 0), (0, 1), (1, 1)] } else { &[(0, 0)] };
                for &(dx, dy) in candidates {
                    let (c, r) = (c0 + dx, r0 + dy);
                    if free(&occupied, c, r, w, h) {
                        placed = Some((c, r));
                        break 'search;
                    }
                }
            }
        }
        let (c, r) = placed.expect("an empty row always fits");
        while occupied.len() < r + h {
            occupied.push(vec![false; columns]);
        }
        for row in occupied.iter_mut().skip(r).take(h) {
            for cell in row.iter_mut().skip(c).take(w) {
                *cell = true;
            }
        }
        out.push((c, r));
    }
    out
}

/// Total grid height in units for a packing.
pub fn packed_rows(sizes: &[TileSize], positions: &[(usize, usize)]) -> usize {
    sizes.iter().zip(positions).map(|(s, (_, r))| r + s.span().1).max().unwrap_or(0)
}

/// How long one slide between faces takes, in seconds, whatever the period.
pub const PEEK_SLIDE_SECS: f32 = 0.9;

/// Fraction of the face scrolled away (0 = front showing, 1 = back showing)
/// at cycle time `t`. `slide` is the fraction of the cycle each slide takes.
pub fn peek_offset(t: f32, slide: f32) -> f32 {
    let slide = slide.clamp(0.001, 0.5);
    let ease = |x: f32| if x < 0.5 { 4. * x * x * x } else { 1. - (-2. * x + 2.).powi(3) / 2. };
    match t {
        t if t < 0.5 - slide => 0.,
        t if t < 0.5 => ease((t - (0.5 - slide)) / slide),
        t if t < 1. - slide => 1.,
        t => 1. - ease((t - (1. - slide)) / slide),
    }
}

thread_local! {
    static EPOCH: Instant = Instant::now();
    static BEAT: std::cell::Cell<Option<f32>> = const { std::cell::Cell::new(None) };
}

/// Puts every live tile's slides on a shared beat, `seconds` apart (`None`,
/// the default: each tile keeps its own time). Set it before the tiles are
/// built. In gpui-metro this was a CPU saving: tiles sliding together
/// share frames. On nsh slides cost almost nothing either way, so here it
/// is only the look: waves instead of a scatter.
pub fn set_live_tile_beat(seconds: Option<f32>) {
    BEAT.with(|b| b.set(seconds.map(|s| s.max(1.))));
}

/// A cycle of `period` seconds at `phase` (0–1), moved onto a beat so that
/// both of its slides start on one.
pub fn on_beat(period: f32, phase: f32, slide: f32, beat: f32) -> (f32, f32) {
    let period = (period / (2. * beat)).round().max(1.) * 2. * beat;
    let beats = (phase * period / beat).round();
    let phase = 0.5 - slide / period - beats * beat / period;
    (period, phase.rem_euclid(1.))
}

/// The peek as one looping animation of `y` (0 → −h → 0), starting where
/// the cycle is now so tiles keep their phases.
fn peek_animation(y: &AnimatedValue, h: f32, period: f32, phase: f32) -> Animation {
    let (period, _phase, start) = match BEAT.with(|b| b.get()) {
        Some(beat) => {
            let (p, ph) = on_beat(period, phase, PEEK_SLIDE_SECS, beat);
            let since = EPOCH.with(|e| e.elapsed().as_secs_f32());
            (p, ph, (since / p + ph).fract())
        }
        None => (period, phase, phase),
    };
    let ms = period * 1000.;
    let slide = (PEEK_SLIDE_SECS / period).clamp(0.001, 0.5);
    let rest = (0.5 - slide) * ms;
    let slide_ms = slide * ms;
    let up = || Animation::timing(y, -h, motion::ambient(slide_ms, IN_OUT_CUBIC));
    let down = || Animation::timing(y, 0., motion::ambient(slide_ms, IN_OUT_CUBIC));
    let cycle = Animation::sequence([Animation::delay(rest), up(), Animation::delay(rest), down()]).looped(None);
    // The rest of the current cycle, then the loop from its start.
    let t = start;
    let first: Vec<Animation> = if t < 0.5 - slide {
        y.set(0.);
        vec![Animation::delay((0.5 - slide - t) * ms), up(), Animation::delay(rest), down()]
    } else if t < 0.5 {
        y.set(-h * peek_offset(t, slide));
        vec![Animation::timing(y, -h, motion::ambient((0.5 - t) * ms, EXP6_OUT)), Animation::delay(rest), down()]
    } else if t < 1. - slide {
        y.set(-h);
        vec![Animation::delay((1. - slide - t) * ms), down()]
    } else {
        y.set(-h * peek_offset(t, slide));
        vec![Animation::timing(y, 0., motion::ambient((1. - t) * ms, EXP6_OUT))]
    };
    Animation::sequence(first.into_iter().chain([cycle]))
}

/// What a custom face draws, given the tile size.
pub type Face = Rc<dyn Fn(TileSize) -> AnyElement>;

/// A start-screen tile.
pub struct Tile {
    pub(crate) key: SharedString,
    pub(crate) size: TileSize,
    title: Option<SharedString>,
    icon: Option<Icon>,
    count: Option<Reactive<u32>>,
    color: Option<Hsla>,
    front: Option<Face>,
    back: Option<Face>,
    back_text: Option<(SharedString, Vec<SharedString>)>,
    /// Seconds for a full front→back→front cycle.
    period: f32,
    /// 0–1 offset into the cycle, so a screen of tiles doesn't flip in unison.
    phase: f32,
    unit: f32,
    gap: f32,
    pub(crate) on_click: Option<ClickHandler>,
}

pub fn tile(key: impl Into<SharedString>, size: TileSize) -> Tile {
    Tile {
        key: key.into(),
        size,
        title: None,
        icon: None,
        count: None,
        color: None,
        front: None,
        back: None,
        back_text: None,
        period: 12.,
        phase: 0.,
        unit: TILE_UNIT,
        gap: TILE_GAP,
        on_click: None,
    }
}

impl Tile {
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// The notification count (hidden at 0, as on the phone): a value, or
    /// a signal or closure for a count that changes. When it appears the
    /// glyph slides aside to make room and the number fades in beside it.
    pub fn count(mut self, count: impl IntoReactive<u32>) -> Self {
        self.count = Some(count.into_reactive());
        self
    }
    /// Override the accent background.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
    /// Replace the default front face (icon + count).
    pub fn front<E: IntoElement>(mut self, f: impl Fn(TileSize) -> E + 'static) -> Self {
        self.front = Some(Rc::new(move |s| f(s).into_element()));
        self
    }
    /// Make the tile live with a back face.
    pub fn back<E: IntoElement>(mut self, f: impl Fn(TileSize) -> E + 'static) -> Self {
        self.back = Some(Rc::new(move |s| f(s).into_element()));
        self
    }
    /// Make the tile live with the usual back face: a headline over a few lines.
    pub fn back_text(
        mut self,
        headline: impl Into<SharedString>,
        lines: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.back_text = Some((headline.into(), lines.into_iter().map(Into::into).collect()));
        self
    }
    pub fn period(mut self, seconds: f32) -> Self {
        self.period = seconds.max(6.);
        self
    }
    pub fn phase(mut self, phase: f32) -> Self {
        self.phase = phase.rem_euclid(1.);
        self
    }
    pub fn metrics(mut self, unit: f32, gap: f32) -> Self {
        self.unit = unit;
        self.gap = gap;
        self
    }
    pub fn on_click(mut self, f: impl Fn() + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl IntoElement for Tile {
    fn into_element(self) -> AnyElement {
        let (w, h) = self.size.pixels(self.unit, self.gap);
        let size = self.size;
        let color = self.color;
        let bg = move || color.unwrap_or(palette().accent);
        let ink = move || palette().accent_ink;
        let title = self.title.clone();
        let face = move |content: AnyElement| {
            view()
                .relative()
                .w(u(w))
                .h(u(h))
                .flex_shrink_0()
                .overflow_hidden()
                .child(content)
                .children(title.clone().map(|t| {
                    label(t)
                        .absolute()
                        .left(u(8.))
                        .bottom(u(6.))
                        .font_family(FONT_FAMILY)
                        .text_size(u(size::SMALL))
                        .text_color(ink)
                }))
        };
        let (icon, count) = (self.icon, self.count);
        let front = match self.front {
            Some(f) => f(size),
            None => default_front(size, icon, count),
        };
        let back = match (self.back, self.back_text) {
            (Some(f), _) => Some(f(size)),
            (None, Some((headline, lines))) => Some(back_text_face(size, headline, lines)),
            (None, None) => None,
        };
        let body = match back {
            Some(back) if !reduced_motion() => {
                let y = animated_value(0.);
                let anim = peek_animation(&y, h, self.period, self.phase);
                let _ = anim.start();
                cleanup(move || anim.stop());
                // The strip holds both faces stacked; it rides up by one face.
                view()
                    .absolute()
                    .left_0()
                    .top_0()
                    .w(u(w))
                    .h(u(h * 2.))
                    .bind_translate_y(&y)
                    .child(face(front))
                    .child(face(back))
                    .into_element()
            }
            _ => face(front).into_element(),
        };
        let el = view().relative().w(u(w)).h(u(h)).flex_shrink_0().overflow_hidden().bg(bg).child(body);
        match self.on_click {
            // A tappable tile tilts under the pointer.
            Some(f) => motion::tilt(el.cursor_pointer().on_press(move || f())).into_element(),
            None => el.into_element(),
        }
    }
}

/// The [`Tile::back_text`] face: a headline over a few lines.
fn back_text_face(size: TileSize, headline: SharedString, lines: Vec<SharedString>) -> AnyElement {
    let big = if size == TileSize::Small { size::MEDIUM } else { size::LARGE };
    let ink = move || palette().accent_ink;
    view()
        .size_full()
        .p(u(10.))
        .pb(u(26.))
        .gap(u(2.))
        .overflow_hidden()
        .child(
            label(headline)
                .font_family(FONT_FAMILY)
                .text_size(u(big))
                .line_height(nsh_core::LineHeight::Points(px(big + 4.)))
                .text_color(ink),
        )
        .children(
            lines
                .into_iter()
                .map(|l| label(l).font_family(FONT_FAMILY).text_size(u(size::SMALL)).truncate().text_color(ink)),
        )
        .into_element()
}

/// How long the glyph takes to move aside for a count (or back), in ms.
pub const COUNT_SHIFT: f32 = 350.;
/// Gap between the glyph and the count beside it.
const COUNT_GAP: f32 = 8.;
/// How far the count slides in from as it fades in.
const COUNT_SLIDE: f32 = 14.;

fn default_front(size: TileSize, icon: Option<Icon>, count: Option<Reactive<u32>>) -> AnyElement {
    let ink = move || palette().accent_ink;
    let icon_px = match size {
        TileSize::Small => 40.,
        TileSize::Medium | TileSize::Wide => 64.,
        TileSize::Large => 96.,
    };
    let count = count.unwrap_or(Reactive::Static(0));
    let initial = untracked(|| count.get()) > 0;
    // 0 = no count showing, 1 = shown; eases between on a change.
    let shown = animated_value(if initial { 1. } else { 0. });
    // The last count shown stays while it fades out at 0.
    let last = Rc::new(std::cell::Cell::new(untracked(|| count.get())));
    let digits = {
        let count = count.clone();
        move || {
            let c = count.get();
            if c > 0 {
                last.set(c);
            }
            SharedString::from(match last.get() {
                0 => String::new(),
                n => n.to_string(),
            })
        }
    };
    {
        let count = count.clone();
        effect(move || {
            let on = count.get() > 0;
            untracked(|| {
                let to = if on { 1. } else { 0. };
                if reduced_motion() {
                    shown.set(to);
                } else {
                    let _ = shown.timing(to, COUNT_SHIFT, EXP6_OUT);
                }
            });
        });
    }
    match size {
        // Small: the glyph stays put; the count fades in in the corner.
        TileSize::Small => view()
            .size_full()
            .items_center()
            .justify_center()
            .children(icon.map(|i| i.render(icon_px, ink)))
            .child(
                label(digits)
                    .absolute()
                    .top(u(4.))
                    .right(u(8.))
                    .font_family(FONT_FAMILY)
                    .text_size(u(size::NORMAL))
                    .text_color(ink)
                    .bind_opacity(&shown),
            )
            .into_element(),
        // Larger: the glyph sits centred alone; a count pushes it left by
        // half the count's width (and the gap), so the pair is centred.
        _ => {
            let width = animated_value(0.);
            let measured = Rc::new(std::cell::Cell::new(0f32));
            let half = shown.mul(&width.add(COUNT_GAP)).mul(0.5);
            let has_icon = icon.is_some();
            let glyph_w = if has_icon { icon_px } else { 0. };
            let number = label(digits)
                .font_family(FONT_FAMILY)
                .text_size(u(size::HUGE * 0.8))
                .font_weight(LIGHT)
                .number_of_lines(1u32)
                .text_color(ink)
                .on_layout(move |b: &Bounds<Points>| {
                    let w = f32::from(b.size.width);
                    if (measured.replace(w) - w).abs() > 0.5 {
                        if reduced_motion() {
                            width.set(w);
                        } else {
                            let _ = width.timing(w, COUNT_SHIFT, EXP6_OUT);
                        }
                    }
                });
            view()
                .size_full()
                .relative()
                .children(icon.map(|i| {
                    view()
                        .absolute()
                        .inset_0()
                        .items_center()
                        .justify_center()
                        .bind_translate_x(&half.mul(-1.))
                        .child(i.render(icon_px, ink))
                }))
                .child(
                    // The count starts just right of the glyph's resting
                    // centre and rides left with it.
                    view()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(relative(0.5))
                        .ml(u(glyph_w / 2. + if has_icon { COUNT_GAP } else { 0. }))
                        .justify_center()
                        .bind_translate_x(&half.mul(-1.).add(&shown.interpolate(&[0., 1.], &[COUNT_SLIDE, 0.])))
                        .bind_opacity(&shown)
                        .child(number),
                )
                .into_element()
        }
    }
}

/// How long each tile takes to swing out, in ms.
pub const EXIT_TILE: f32 = 250.;
/// Gap between rows in the wave (compressed on tall grids).
const EXIT_STEP: f32 = 45.;
/// The whole wave (before the tapped tile) never takes longer than this.
const EXIT_WAVE_MAX: f32 = 260.;
/// The tapped tile lingers this long after the last of the others starts.
const EXIT_HOLD: f32 = 90.;
/// Where a tile turns in from when the start screen appears: edge-on.
const ENTER_FROM_DEG: f32 = 90.;
/// Gap between rows as they turn in.
const ENTER_STEP: f32 = 40.;
const ENTER_WAVE_MAX: f32 = 240.;

/// Rank of each tile's row among the rows in use (0 = top), so a feather
/// moves whole rows: tiles side by side turn together and never cross.
fn row_ranks(positions: &[(usize, usize)], skip: usize) -> (Vec<usize>, usize) {
    let mut rows: Vec<usize> = positions.iter().enumerate().filter(|&(i, _)| i != skip).map(|(_, p)| p.1).collect();
    rows.sort_unstable();
    rows.dedup();
    let ranks = positions.iter().map(|p| rows.binary_search(&p.1).unwrap_or(0)).collect();
    (ranks, rows.len())
}

/// Start delays (ms) for the turnstile-out, per tile, plus the total time
/// until the last tile has gone. Rows leave top to bottom; the tapped tile
/// goes last, on its own.
pub fn exit_schedule(positions: &[(usize, usize)], tapped: usize) -> (Vec<f32>, f32) {
    let (ranks, rows) = row_ranks(positions, tapped);
    let step = match rows {
        0 | 1 => EXIT_STEP,
        n => EXIT_STEP.min(EXIT_WAVE_MAX / (n as f32 - 1.)),
    };
    let mut delays: Vec<f32> = ranks.iter().map(|&r| step * r as f32).collect();
    let last_row = rows.checked_sub(1).map(|r| step * r as f32);
    let tapped_delay = last_row.map_or(0., |d| d + EXIT_HOLD);
    if tapped < delays.len() {
        delays[tapped] = tapped_delay;
    }
    (delays, tapped_delay + EXIT_TILE)
}

/// Start delays (ms) for the turnstile-in: rows top to bottom.
pub fn enter_schedule(positions: &[(usize, usize)]) -> Vec<f32> {
    let (ranks, rows) = row_ranks(positions, usize::MAX);
    let step = match rows {
        0 | 1 => ENTER_STEP,
        n => ENTER_STEP.min(ENTER_WAVE_MAX / (n as f32 - 1.)),
    };
    ranks.iter().map(|&r| step * r as f32).collect()
}

/// Space between groups, and the height of the group title row.
pub const GROUP_GAP: f32 = 56.;
pub const GROUP_HEADER: f32 = 44.;

/// The start screen: tiles packed by [`pack`] and positioned absolutely, in
/// one column of blocks (the phone) or several named groups side by side
/// (Windows 8).
pub struct TileGrid {
    tiles: Vec<Tile>,
    group_of: Vec<usize>,
    groups: Vec<Option<SharedString>>,
    columns: usize,
    unit: f32,
    gap: f32,
    turnstile: bool,
    replay: Option<ReadSignal<usize>>,
    leave: Option<ReadSignal<bool>>,
    hinge: f32,
    source: Option<Rc<dyn Fn() -> Vec<Tile>>>,
}

pub fn tile_grid() -> TileGrid {
    TileGrid {
        tiles: Vec::new(),
        group_of: Vec::new(),
        groups: vec![None],
        columns: 4,
        unit: TILE_UNIT,
        gap: TILE_GAP,
        turnstile: false,
        replay: None,
        leave: None,
        hinge: PAGE_MARGIN,
        source: None,
    }
}

impl TileGrid {
    /// Adds a tile to the current (last) group.
    pub fn tile(mut self, tile: Tile) -> Self {
        self.group_of.push(self.groups.len() - 1);
        self.tiles.push(tile);
        self
    }
    pub fn tiles(self, tiles: impl IntoIterator<Item = Tile>) -> Self {
        tiles.into_iter().fold(self, |g, t| g.tile(t))
    }
    /// Starts a new named group to the right of the previous one.
    pub fn group(mut self, title: impl Into<SharedString>, tiles: impl IntoIterator<Item = Tile>) -> Self {
        let title = Some(title.into());
        if self.tiles.is_empty() && self.groups.len() == 1 {
            self.groups[0] = title;
        } else {
            self.groups.push(title);
        }
        self.tiles(tiles)
    }
    /// Distance from the turnstile's hinge (the screen's left edge) to the
    /// grid's left edge, in px.
    pub fn hinge(mut self, distance: f32) -> Self {
        self.hinge = distance;
        self
    }
    /// Grid width in small-tile units.
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }
    pub fn metrics(mut self, unit: f32, gap: f32) -> Self {
        self.unit = unit;
        self.gap = gap;
        self
    }
    /// The start screen's turnstile: tiles swing in row by row when the grid
    /// appears; a tapped tile swings everything away in a wave, itself
    /// last, and only then runs its `on_click`.
    pub fn turnstile(mut self, on: bool) -> Self {
        self.turnstile = on;
        self
    }
    /// Replays the entrance whenever this changes.
    pub fn replay(mut self, key: impl Into<ReadSignal<usize>>) -> Self {
        self.replay = Some(key.into());
        self
    }
    /// Swings every tile away while this is true, as when the start screen is
    /// left without a tile being tapped; back in when it turns false.
    pub fn leave(mut self, leave: impl Into<ReadSignal<bool>>) -> Self {
        self.leave = Some(leave.into());
        self
    }

    /// Tiles that change: `f` is read reactively, and whenever what it read
    /// changes the grid re-packs. Tiles are known by their key: one that
    /// moved slides to its new place (rows nearest the change first), one
    /// that is new waits edge-on for room and then turns in, one that went
    /// is removed. A tile's faces are built when it first appears. One
    /// group; `turnstile`, `replay` and `leave` work as on a static grid.
    pub fn source(mut self, f: impl Fn() -> Vec<Tile> + 'static) -> Self {
        self.source = Some(Rc::new(f));
        self
    }

    /// The width of one group, in px.
    pub fn group_width(&self) -> f32 {
        self.columns as f32 * self.unit + (self.columns.saturating_sub(1)) as f32 * self.gap
    }

    /// The width this grid occupies.
    pub fn width(&self) -> f32 {
        let n = self.group_count() as f32;
        n * self.group_width() + (n - 1.).max(0.) * GROUP_GAP
    }

    fn group_count(&self) -> usize {
        self.group_of.iter().max().map_or(1, |g| g + 1)
    }
}

impl IntoElement for TileGrid {
    fn into_element(self) -> AnyElement {
        if let Some(source) = self.source.clone() {
            return self.reflowing(source);
        }
        let step = self.unit + self.gap;
        let width = self.width();
        let (unit, gap, hinge) = (self.unit, self.gap, self.hinge);
        let group_count = self.group_count();
        let titled = self.groups.iter().any(Option::is_some);
        let top = if titled { GROUP_HEADER } else { 0. };
        let group_w = self.group_width();
        let group_x = move |g: usize| g as f32 * (group_w + GROUP_GAP);

        let mut positions = vec![(0, 0); self.tiles.len()];
        let mut origins = vec![(0., 0.); self.tiles.len()];
        let mut rows = 0;
        for g in 0..group_count {
            let members: Vec<usize> = (0..self.tiles.len()).filter(|&i| self.group_of[i] == g).collect();
            let sizes: Vec<TileSize> = members.iter().map(|&i| self.tiles[i].size).collect();
            let packed = pack(&sizes, self.columns);
            rows = rows.max(packed_rows(&sizes, &packed));
            for (&i, &(c, r)) in members.iter().zip(&packed) {
                positions[i] = (g * (self.columns + 1) + c, r);
                origins[i] = (group_x(g) + c as f32 * step, top + r as f32 * step);
            }
        }
        let height = top + rows as f32 * step - if rows > 0 { gap } else { 0. };
        let headers: Vec<AnyElement> = self
            .groups
            .iter()
            .enumerate()
            .filter_map(|(g, title)| {
                let title = title.clone()?;
                Some(
                    label(SharedString::from(title.to_lowercase()))
                        .absolute()
                        .left(u(group_x(g)))
                        .top_0()
                        .font_family(FONT_FAMILY)
                        .text_size(u(size::MEDIUM_LARGE))
                        .font_weight(SEMILIGHT)
                        .text_color(move || palette().foreground)
                        .into_element(),
                )
            })
            .collect();

        let turnstile = self.turnstile && !reduced_motion();
        let n = self.tiles.len();
        let angles: Vec<AnimatedValue> =
            (0..n).map(|_| animated_value(if turnstile { ENTER_FROM_DEG } else { 0. })).collect();
        let positions = Rc::new(positions);
        let enter = {
            let (angles, positions) = (angles.clone(), positions.clone());
            move || {
                let delays = enter_schedule(&positions);
                for a in &angles {
                    a.set(ENTER_FROM_DEG);
                }
                Animation::parallel(angles.iter().zip(&delays).map(|(a, &d)| {
                    Animation::sequence([Animation::delay(d), Animation::timing(a, 0., Timing::new(motion::ENTER, EXP6_OUT))])
                }))
                .start()
            }
        };
        let exit = {
            let (angles, positions) = (angles.clone(), positions.clone());
            move |tapped: usize| {
                let (delays, _) = exit_schedule(&positions, tapped);
                Animation::parallel(angles.iter().zip(&delays).map(|(a, &d)| {
                    Animation::sequence([Animation::delay(d), Animation::timing(a, 90., Timing::new(EXIT_TILE, EXP6_IN))])
                }))
                .start()
            }
        };
        let busy = Rc::new(std::cell::Cell::new(false));
        if turnstile {
            let _ = enter();
            if let Some(replay) = self.replay {
                let enter = enter.clone();
                let first = RefCell::new(true);
                effect(move || {
                    replay.get();
                    if !std::mem::replace(&mut *first.borrow_mut(), false) {
                        let _ = untracked(|| enter());
                    }
                });
            }
            if let Some(leave) = self.leave {
                let (enter, exit) = (enter.clone(), exit.clone());
                let first = RefCell::new(true);
                effect(move || {
                    let away = leave.get();
                    if std::mem::replace(&mut *first.borrow_mut(), false) && !away {
                        return;
                    }
                    let _ = untracked(|| if away { exit(usize::MAX) } else { enter() });
                });
            }
        }

        let tiles: Vec<AnyElement> = self
            .tiles
            .into_iter()
            .zip(origins)
            .enumerate()
            .map(|(ix, (mut t, (x, y)))| {
                // Route the click through the exit: animate first, launch after.
                if turnstile && let Some(launch) = t.on_click.take() {
                    let (exit, busy) = (exit.clone(), busy.clone());
                    t.on_click = Some(Rc::new(move || {
                        if busy.replace(true) {
                            return;
                        }
                        let end = exit(ix);
                        let (launch, busy) = (launch.clone(), busy.clone());
                        spawn(async move {
                            end.await;
                            // The tiles stay away: the app is opening. A grid
                            // that stays on screen comes back with `replay`.
                            busy.set(false);
                            launch();
                        });
                    }));
                }
                let el = t.metrics(unit, gap).into_element();
                let placed = view().absolute().left(u(x)).top(u(y));
                if turnstile {
                    placed.child(motion::turned(&angles[ix], x + hinge, el)).into_element()
                } else {
                    placed.child(el).into_element()
                }
            })
            .collect();

        view()
            .relative()
            .flex_shrink_0()
            .w(u(width))
            .h(u(height))
            .children(headers)
            .children(tiles)
            .into_element()
    }
}

/// How long a tile takes to slide to a new place when the grid reflows, ms.
pub const REFLOW: f32 = 400.;
/// Gap between rows as they slide: the push travels down from the change.
const REFLOW_STEP: f32 = 30.;
const REFLOW_WAVE_MAX: f32 = 150.;
/// A new tile waits this long for the others to start making room.
const ARRIVE_AFTER: f32 = 160.;
/// How long a new tile takes to swing open into place, in ms.
const ARRIVE: f32 = 520.;
/// Where a new tile swings open from: turned away about its left edge.
const ARRIVE_FROM_DEG: f32 = 75.;

/// Start delays (ms) for a reflow: the moving tiles' rows, nearest the
/// change (`origin`) first.
pub fn reflow_schedule(rows: &[usize], origin: usize) -> Vec<f32> {
    let dist: Vec<usize> = rows.iter().map(|&r| r.abs_diff(origin)).collect();
    let mut ranks = dist.clone();
    ranks.sort_unstable();
    ranks.dedup();
    let step = match ranks.len() {
        0 | 1 => REFLOW_STEP,
        n => REFLOW_STEP.min(REFLOW_WAVE_MAX / (n as f32 - 1.)),
    };
    dist.iter().map(|d| step * ranks.binary_search(d).unwrap_or(0) as f32).collect()
}

/// A tile's place in a reflowing grid: where it is drawn (animated), how
/// far it has turned in, and its tile until it is first built.
struct Slot {
    x: AnimatedValue,
    y: AnimatedValue,
    angle: AnimatedValue,
    /// A new tile's flip into place about its own left edge (degrees).
    arrive: AnimatedValue,
    at: (f32, f32),
    /// Place in grid units, for the turnstile's row order.
    cell: (usize, usize),
    tile: Option<Tile>,
}

type Slots = Rc<RefCell<std::collections::HashMap<SharedString, Slot>>>;

/// The turnstile-in over a reflowing grid's tiles, in `keys` order.
fn slots_enter(slots: &Slots, keys: &[SharedString]) {
    let slots = slots.borrow();
    let tiles: Vec<&Slot> = keys.iter().filter_map(|k| slots.get(k)).collect();
    let cells: Vec<(usize, usize)> = tiles.iter().map(|s| s.cell).collect();
    let delays = enter_schedule(&cells);
    for t in &tiles {
        t.angle.set(ENTER_FROM_DEG);
    }
    let _ = Animation::parallel(tiles.iter().zip(&delays).map(|(t, &d)| {
        Animation::sequence([Animation::delay(d), Animation::timing(&t.angle, 0., Timing::new(motion::ENTER, EXP6_OUT))])
    }))
    .start();
}

/// The turnstile-out, `tapped` (an index into `keys`, or none) last.
fn slots_exit(slots: &Slots, keys: &[SharedString], tapped: usize) -> nsh_core::element::AnimationEnd {
    let slots = slots.borrow();
    let tiles: Vec<&Slot> = keys.iter().filter_map(|k| slots.get(k)).collect();
    let cells: Vec<(usize, usize)> = tiles.iter().map(|s| s.cell).collect();
    let (delays, _) = exit_schedule(&cells, tapped);
    Animation::parallel(tiles.iter().zip(&delays).map(|(t, &d)| {
        Animation::sequence([Animation::delay(d), Animation::timing(&t.angle, 90., Timing::new(EXIT_TILE, EXP6_IN))])
    }))
    .start()
}

impl TileGrid {
    fn reflowing(self, source: Rc<dyn Fn() -> Vec<Tile>>) -> AnyElement {
        let (unit, gap, columns, hinge) = (self.unit, self.gap, self.columns, self.hinge);
        let step = unit + gap;
        let width = self.group_width();
        let slots: Slots = Rc::default();
        let keys = signal(Vec::<SharedString>::new());
        let height = signal(0f32);
        let mounted = Rc::new(std::cell::Cell::new(false));
        let turnstile = self.turnstile && !reduced_motion();
        let busy = Rc::new(std::cell::Cell::new(false));
        // Tiles' animated values belong to the grid, not to the effect: an
        // effect disposes what it made on its last run each time it runs.
        let owner = Owner::current();
        {
            let slots = slots.clone();
            let slots_c = slots.clone();
            let busy = busy.clone();
            effect(move || {
                let tiles = source();
                untracked(|| {
                    let sizes: Vec<TileSize> = tiles.iter().map(|t| t.size).collect();
                    let packed = pack(&sizes, columns);
                    let rows = packed_rows(&sizes, &packed);
                    height.set(rows as f32 * step - if rows > 0 { gap } else { 0. });
                    let reduced = reduced_motion();
                    let first = !mounted.replace(true);
                    let mut slots = slots.borrow_mut();
                    // What moved, and the topmost row that changed.
                    let mut moved: Vec<(SharedString, (f32, f32), usize)> = Vec::new();
                    let mut origin = usize::MAX;
                    let mut new_keys = Vec::with_capacity(tiles.len());
                    for (t, &(c, r)) in tiles.into_iter().zip(&packed) {
                        let to = (c as f32 * step, r as f32 * step);
                        let key = t.key.clone();
                        new_keys.push(key.clone());
                        match slots.get_mut(&key) {
                            Some(slot) => {
                                slot.cell = (c, r);
                                if slot.at != to {
                                    slot.at = to;
                                    moved.push((key, to, r));
                                    origin = origin.min(r);
                                }
                            }
                            None => {
                                let arriving = !first && !reduced;
                                let start = if first && turnstile { ENTER_FROM_DEG } else { 0. };
                                let (angle, arrive, x, y) = owner.run(|| {
                                    (
                                        animated_value(start),
                                        animated_value(if arriving { ARRIVE_FROM_DEG } else { 0. }),
                                        animated_value(to.0),
                                        animated_value(to.1),
                                    )
                                });
                                // On a turnstile grid a tap swings the tiles
                                // away first, the tapped one last.
                                let mut t = t;
                                if turnstile && let Some(launch) = t.on_click.take() {
                                    let (slots_c, busy) = (slots_c.clone(), busy.clone());
                                    let me = key.clone();
                                    t.on_click = Some(Rc::new(move || {
                                        if busy.replace(true) {
                                            return;
                                        }
                                        let order = keys.get_untracked();
                                        let ix = order.iter().position(|k| *k == me).unwrap_or(usize::MAX);
                                        let end = slots_exit(&slots_c, &order, ix);
                                        let (launch, busy) = (launch.clone(), busy.clone());
                                        spawn(async move {
                                            end.await;
                                            busy.set(false);
                                            launch();
                                        });
                                    }));
                                }
                                if arriving {
                                    origin = origin.min(r);
                                    let _ = Animation::sequence([
                                        Animation::delay(ARRIVE_AFTER),
                                        Animation::timing(&arrive, 0., Timing::new(ARRIVE, EXP6_OUT)),
                                    ])
                                    .start();
                                }
                                let slot = Slot { x, y, angle, arrive, at: to, cell: (c, r), tile: Some(t) };
                                slots.insert(key, slot);
                            }
                        }
                    }
                    let rows: Vec<usize> = moved.iter().map(|m| m.2).collect();
                    let delays = reflow_schedule(&rows, if origin == usize::MAX { 0 } else { origin });
                    for ((key, to, _), delay) in moved.iter().zip(delays) {
                        let slot = &slots[key];
                        if reduced {
                            slot.x.set(to.0);
                            slot.y.set(to.1);
                        } else {
                            let _ = Animation::sequence([
                                Animation::delay(delay),
                                Animation::parallel([
                                    Animation::timing(&slot.x, to.0, Timing::new(REFLOW, EXP6_OUT)),
                                    Animation::timing(&slot.y, to.1, Timing::new(REFLOW, EXP6_OUT)),
                                ]),
                            ])
                            .start();
                        }
                    }
                    slots.retain(|k, _| new_keys.contains(k));
                    drop(slots);
                    keys.set(new_keys.clone());
                    if first && turnstile {
                        slots_enter(&slots_c, &new_keys);
                    }
                });
            });
        }
        if turnstile {
            if let Some(replay) = self.replay {
                let slots = slots.clone();
                let first = RefCell::new(true);
                effect(move || {
                    replay.get();
                    if !std::mem::replace(&mut *first.borrow_mut(), false) {
                        untracked(|| slots_enter(&slots, &keys.get()));
                    }
                });
            }
            if let Some(leave) = self.leave {
                let slots = slots.clone();
                let first = RefCell::new(true);
                effect(move || {
                    let away = leave.get();
                    if std::mem::replace(&mut *first.borrow_mut(), false) && !away {
                        return;
                    }
                    untracked(|| {
                        if away {
                            let _ = slots_exit(&slots, &keys.get(), usize::MAX);
                        } else {
                            slots_enter(&slots, &keys.get());
                        }
                    });
                });
            }
        }
        let tiles = each(
            move || keys.get(),
            |k: &SharedString| k.clone(),
            move |key: ReadSignal<SharedString>, _| {
                let key = untracked(|| key.get());
                let mut slots = slots.borrow_mut();
                let Some(slot) = slots.get_mut(&key) else { return view().into_element() };
                let tile = slot.tile.take().map(|t| t.metrics(unit, gap).into_element());
                view()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bind_translate_x(&slot.x)
                    .bind_translate_y(&slot.y)
                    .child(motion::turned(
                        &slot.angle,
                        slot.at.0 + hinge,
                        // A new tile swings open about its own left edge,
                        // like a door, fading in as it comes square.
                        view()
                            .bind_opacity(&slot.arrive.interpolate(&[0., 30., ARRIVE_FROM_DEG], &[1., 0.85, 0.]))
                            .child(motion::turned(&slot.arrive, 0., tile.unwrap_or_else(|| view().into_element()))),
                    ))
                    .into_element()
            },
        );
        view().relative().flex_shrink_0().w(u(width)).h(move || u(height.get())).child(tiles).into_element()
    }
}

#[cfg(test)]
mod tests {
    use super::TileSize::*;
    use super::*;

    #[test]
    fn a_reflow_pushes_out_from_the_change() {
        let d = reflow_schedule(&[2, 2, 4, 6], 2);
        assert_eq!(d[0], d[1]);
        assert!(d[0] < d[2] && d[2] < d[3], "{d:?}");
        assert!(reflow_schedule(&(0..40).collect::<Vec<_>>(), 0).iter().all(|&x| x <= REFLOW_WAVE_MAX));
    }

    #[test]
    fn smalls_fill_a_block_before_moving_on() {
        let p = pack(&[Small, Small, Small, Small, Small], 4);
        assert_eq!(p, vec![(0, 0), (1, 0), (0, 1), (1, 1), (2, 0)]);
    }

    #[test]
    fn medium_and_wide_align_to_blocks() {
        let p = pack(&[Medium, Small, Wide, Medium, Medium], 4);
        assert_eq!(p, vec![(0, 0), (2, 0), (0, 2), (0, 4), (2, 4)]);
    }

    #[test]
    fn tiles_never_overlap() {
        let sizes = [Wide, Small, Medium, Small, Small, Large, Small, Medium, Wide, Small, Small, Small];
        for cols in [4, 6, 8] {
            let p = pack(&sizes, cols);
            let mut cells = std::collections::HashSet::new();
            for (s, (c, r)) in sizes.iter().zip(&p) {
                let (w, h) = s.span();
                assert!(c + w <= cols);
                for y in *r..r + h {
                    for x in *c..c + w {
                        assert!(cells.insert((x, y)), "overlap at {x},{y} with {cols} columns");
                    }
                }
            }
        }
    }

    #[test]
    fn exit_waves_in_reading_order_and_the_tapped_tile_goes_last() {
        let sizes = [Wide, Medium, Small, Small, Small, Small];
        let positions = pack(&sizes, 4);
        let tapped = 1;
        let (delays, total) = exit_schedule(&positions, tapped);
        let others: Vec<usize> = (0..sizes.len()).filter(|&i| i != tapped).collect();
        assert!(others.iter().all(|&i| delays[i] < delays[tapped]), "{delays:?}");
        assert_eq!(total, delays[tapped] + EXIT_TILE);
    }

    #[test]
    fn tiles_turn_in_top_left_first() {
        let positions = pack(&[Wide, Medium, Small, Small], 4);
        let d = enter_schedule(&positions);
        assert_eq!(d[0], 0.);
        assert!(d[1] > d[0]);
        assert!(enter_schedule(&vec![(0, 0); 80]).iter().all(|&x| x <= ENTER_WAVE_MAX));
    }

    #[test]
    fn peek_rests_on_each_face() {
        let slide = PEEK_SLIDE_SECS / 12.;
        assert_eq!(peek_offset(0.1, slide), 0.);
        assert_eq!(peek_offset(0.7, slide), 1.);
        assert!(peek_offset(0.47, slide) > 0. && peek_offset(0.47, slide) < 1.);
    }

    #[test]
    fn a_cycle_on_a_beat_starts_both_slides_on_it() {
        let (beat, slide) = (3., 0.9);
        for (period, phase) in [(9., 0.), (12., 0.37), (15., 0.9), (7., 0.5)] {
            let (p, ph) = on_beat(period, phase, slide, beat);
            assert_eq!((p / (2. * beat)).fract(), 0.);
            for at in [0.5 - slide / p, 1. - slide / p] {
                let secs = (at - ph).rem_euclid(1.) * p;
                let off = (secs / beat).round() * beat - secs;
                assert!(off.abs() < 1e-3);
            }
        }
    }
}
