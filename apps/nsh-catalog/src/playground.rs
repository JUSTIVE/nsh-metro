//! The playground beside each page: a live preview of the component and a
//! panel with every option its builder takes. Changing an option rebuilds
//! the preview, so options the library takes as fixed values work too.
//! Where the combinations are few, a variant grid shows them all at once.

use std::rc::Rc;

use nsh_core::prelude::*;
use nsh_metro::{
    Accent, Icon, Leading, ListGroup, TextStyle, TileSize, app_bar, button, check_box, hyperlink, jump_list,
    label, list_item, list_picker, message_box, page_header, palette, panorama, pivot, progress_bar, progress_dots,
    progress_ring, radio_button, round_button, settings_flyout, slider, system_tray, text, text_box, tile, tile_grid,
    toast, toggle_switch, u,
    AppBarMode, FlyoutWidth,
    theme::{size, FONT_FAMILY},
};

use crate::pages::Page;

// ---------------------------------------------------------------- knobs ---

const KNOB_LABEL_W: f32 = 120.;

fn knob_row(name: &'static str, control: impl IntoElement) -> View {
    view()
        .flex_row()
        .items_start()
        .gap(u(12.))
        .child(
            label(name)
                .w(u(KNOB_LABEL_W))
                .flex_shrink_0()
                .pt(u(3.))
                .text_size(u(size::SMALL))
                .text_color(move || palette().subtle),
        )
        .child(view().flex_1().min_w(px(0.)).child(control))
}

fn chip(caption: impl Into<SharedString>, on: impl Fn() -> bool + 'static, press: impl Fn() + 'static) -> View {
    let on = Rc::new(on);
    let (a, b) = (on.clone(), on);
    view()
        .px(u(9.))
        .py(u(2.))
        .border(px(2.))
        .border_color(move || if a() { palette().accent } else { palette().disabled })
        .bg(move || if b() { palette().accent } else { palette().accent.opacity(0.) })
        .cursor_pointer()
        .on_press(move || press())
        .child(label(caption.into()).text_size(u(size::SMALL)).text_color(move || palette().foreground))
}

/// One of several options.
fn choice(name: &'static str, options: &[&'static str], sel: Signal<usize>) -> View {
    knob_row(
        name,
        view().flex_row().flex_wrap().gap(u(6.)).children(
            options.iter().enumerate().map(|(i, o)| chip(*o, move || sel.get() == i, move || sel.set(i))),
        ),
    )
}

/// On or off.
fn flag(name: &'static str, on: Signal<bool>) -> View {
    knob_row(
        name,
        view()
            .flex_row()
            .gap(u(6.))
            .child(chip("off", move || !on.get(), move || on.set(false)))
            .child(chip("on", move || on.get(), move || on.set(true))),
    )
}

/// A number between `min` and `max`.
fn number(name: &'static str, v: Signal<f32>, min: f32, max: f32, step: f32) -> View {
    let to01 = move |x: f32| (x - min) / (max - min);
    knob_row(
        name,
        view()
            .flex_row()
            .items_center()
            .gap(u(10.))
            .child(
                view().flex_1().child(
                    slider(move || to01(v.get()))
                        .step(step / (max - min))
                        .on_change(move |t| v.set(((min + t * (max - min)) / step).round() * step)),
                ),
            )
            .child(
                label(move || {
                    let x = v.get();
                    if step >= 1. { format!("{x:.0}") } else { format!("{x:.2}") }
                })
                .w(u(44.))
                .font_family("Menlo")
                .text_size(u(size::SMALL))
                .text_color(move || palette().foreground),
            ),
    )
}

/// Free text.
fn words(name: &'static str, v: Signal<String>) -> View {
    knob_row(name, text_box(v))
}

const ACCENTS: [&str; 6] = ["theme", "teal", "magenta", "orange", "crimson", "lime"];
fn accent_of(i: usize) -> Option<Hsla> {
    [None, Some(Accent::Teal), Some(Accent::Magenta), Some(Accent::Orange), Some(Accent::Crimson), Some(Accent::Lime)][i]
        .map(Accent::color)
}

const ICONS: [&str; 8] = ["mail", "phone", "camera", "calendar", "people", "clock", "share", "flask"];
fn icon_of(i: usize) -> Icon {
    [Icon::Mail, Icon::Phone, Icon::Camera, Icon::Calendar, Icon::People, Icon::Clock, Icon::Share, Icon::Flask][i]
}

// ----------------------------------------------------------- framing ----

/// What the playground of a page shows.
struct Playground {
    preview: AnyElement,
    options: Vec<View>,
    variants: Option<AnyElement>,
    /// Preview height, for overlays and paged components.
    height: Option<f32>,
}

fn pg(preview: impl Fn() -> AnyElement + 'static, options: Vec<View>) -> Playground {
    Playground { preview: dynamic(preview).into_element(), options, variants: None, height: None }
}

impl Playground {
    fn variants(mut self, v: impl IntoElement) -> Self {
        self.variants = Some(v.into_element());
        self
    }
    fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }
}

fn section_title(t: &'static str) -> AnyElement {
    text(TextStyle::GroupHeader, t).into_element()
}

/// The playground column for `page`.
pub fn render(page: Page) -> AnyElement {
    let Playground { preview, options, variants, height } = build(page);
    let stage = view()
        .relative()
        .overflow_hidden()
        .p(u(20.))
        .bg(move || palette().background)
        .border(px(1.))
        .border_color(move || palette().disabled.opacity(0.4))
        .child(preview);
    let stage = match height {
        Some(h) => stage.h(u(h)),
        None => stage,
    };
    view()
        .gap(u(14.))
        .child(section_title("playground"))
        .child(stage)
        .child(
            view()
                .gap(u(12.))
                .p(u(18.))
                .bg(move || palette().chrome)
                .child(label("options").text_size(u(size::MEDIUM)).text_color(move || palette().foreground))
                .children(options),
        )
        .children(variants.map(|v| view().gap(u(10.)).child(section_title("variants")).child(v)))
        .into_element()
}

fn row() -> View {
    view().flex_row().flex_wrap().items_start().gap(u(12.))
}

fn tagged(tag: impl Into<SharedString>, el: impl IntoElement) -> View {
    view()
        .gap(u(4.))
        .child(label(tag.into()).text_size(u(size::SMALL)).text_color(move || palette().subtle))
        .child(el)
}

// ------------------------------------------------------------- pages ----

fn build(page: Page) -> Playground {
    match page {
        Page::Palette => {
            let accent = signal(5usize);
            let preview = move || {
                let a = Accent::ALL[accent.get()];
                view()
                    .flex_row()
                    .gap(u(12.))
                    .child(view().size(u(96.)).bg(a.color()).justify_end().p(u(6.)).child(
                        label(a.name()).text_size(u(size::SMALL)).text_color(a.ink()),
                    ))
                    .child(button("apply to catalog").accent().on_click(move || nsh_metro::Theme::set_accent(a)))
                    .into_element()
            };
            let names: Vec<&'static str> = Accent::ALL.iter().map(|a| a.name()).collect();
            let names: &'static [&'static str] = Box::leak(names.into_boxed_slice());
            pg(preview, vec![choice("accent", names, accent)])
        }
        Page::Typography => {
            let style = signal(0usize);
            let content = signal("strains".to_string());
            let lower = signal(false);
            let truncate = signal(false);
            const STYLES: [&str; 11] = [
                "title 1", "title 2", "title 3", "extra large", "large", "normal", "subtle", "accent", "small",
                "app title", "group header",
            ];
            let of = |i: usize| {
                [
                    TextStyle::Title1,
                    TextStyle::Title2,
                    TextStyle::Title3,
                    TextStyle::ExtraLarge,
                    TextStyle::Large,
                    TextStyle::Normal,
                    TextStyle::Subtle,
                    TextStyle::Accent,
                    TextStyle::Small,
                    TextStyle::AppTitle,
                    TextStyle::GroupHeader,
                ][i]
            };
            let preview = move || {
                let mut t = text(of(style.get()), content.get());
                if lower.get() {
                    t = t.lowercase();
                }
                if truncate.get() {
                    t = t.truncate();
                }
                t.into_element()
            };
            let variants = view().gap(u(6.)).children((0..STYLES.len()).map(move |i| {
                view().flex_row().items_center().gap(u(12.)).child(
                    label(STYLES[i]).w(u(110.)).text_size(u(size::SMALL)).text_color(move || palette().subtle),
                ).child(text(of(i), "the quick fox"))
            }));
            pg(
                preview,
                vec![
                    choice("style", &STYLES, style),
                    words("text", content),
                    flag("lowercase", lower),
                    flag("one line", truncate),
                ],
            )
            .variants(variants)
        }
        Page::Icons => {
            let ic = signal(0usize);
            let px_ = signal(48f32);
            let color = signal(0usize);
            let preview = move || {
                let c = accent_of(color.get());
                icon_of(ic.get()).render(px_.get(), move || c.unwrap_or(palette().foreground))
            };
            let variants = row().children(Icon::ALL.iter().map(|i| {
                tagged(i.name(), i.render(28., move || palette().foreground))
            }));
            pg(
                preview,
                vec![choice("icon", &ICONS, ic), number("size", px_, 12., 128., 1.), choice("colour", &ACCENTS, color)],
            )
            .variants(variants)
        }
        Page::Tile => {
            let sz = signal(1usize);
            let title = signal("mail".to_string());
            let ic = signal(0usize);
            let has_icon = signal(true);
            let count = signal(3f32);
            let color = signal(0usize);
            let live = signal(true);
            let period = signal(9f32);
            let phase = signal(0f32);
            let tappable = signal(true);
            let headline = signal("ben, kristin".to_string());
            let taps = signal(0u32);
            const SIZES: [&str; 4] = ["small", "medium", "wide", "large"];
            let size_of = |i: usize| [TileSize::Small, TileSize::Medium, TileSize::Wide, TileSize::Large][i];
            let preview = move || {
                let mut t = tile("pg-tile", size_of(sz.get())).count(move || count.get() as u32).metrics(80., 10.);
                let tt = title.get();
                if !tt.is_empty() {
                    t = t.title(tt);
                }
                if has_icon.get() {
                    t = t.icon(icon_of(ic.get()));
                }
                if let Some(c) = accent_of(color.get()) {
                    t = t.color(c);
                }
                if live.get() {
                    t = t.back_text(headline.get(), ["3 new posts", "and 2 photos"]).period(period.get()).phase(phase.get());
                }
                if tappable.get() {
                    t = t.on_click(move || taps.update(|n| *n += 1));
                }
                view().gap(u(10.)).child(t).child(label(move || format!("{} taps", taps.get())).text_size(u(size::SMALL)).text_color(move || palette().subtle)).into_element()
            };
            let variants = row().children((0..4).flat_map(|s| {
                [false, true].into_iter().map(move |l| {
                    let mut t = tile(format!("v-{s}-{l}"), size_of(s)).icon(Icon::Mail).count(2).title("mail").metrics(44., 6.);
                    if l {
                        t = t.back_text("2 new", ["from ben"]).period(6. + s as f32);
                    }
                    tagged(format!("{} {}", SIZES[s], if l { "live" } else { "static" }), t)
                })
            }));
            pg(
                preview,
                vec![
                    choice("size", &SIZES, sz),
                    words("title", title),
                    flag("icon", has_icon),
                    choice("glyph", &ICONS, ic),
                    number("count", count, 0., 99., 1.),
                    choice("colour", &ACCENTS, color),
                    flag("live (back face)", live),
                    words("back headline", headline),
                    number("period (s)", period, 6., 20., 1.),
                    number("phase", phase, 0., 1., 0.05),
                    flag("tappable (tilt)", tappable),
                ],
            )
            .variants(variants)
        }
        Page::TileGrid | Page::Turnstile => {
            let columns = signal(0usize);
            let unit = signal(60f32);
            let gap = signal(8f32);
            let n = signal(10f32);
            let turnstile = signal(true);
            let groups = signal(false);
            let replay = signal(0usize);
            let pinned = signal(0usize);
            let preview = move || {
                let cols = [4, 6, 8][columns.get()];
                let make = move |i: usize| {
                    let s = [TileSize::Medium, TileSize::Small, TileSize::Small, TileSize::Wide, TileSize::Small, TileSize::Medium][i % 6];
                    tile(format!("g{i}"), s).icon(icon_of(i % 8)).color(Accent::ALL[(i * 7) % 20].color())
                };
                let count = n.get() as usize;
                let mut g = tile_grid().columns(cols).metrics(unit.get(), gap.get()).hinge(20.).turnstile(turnstile.get()).replay(replay.read_only());
                if groups.get() {
                    let half = count / 2;
                    g = g.group("first", (0..half).map(make)).group("second", (half..count).map(make));
                } else {
                    g = g.tiles((0..count).map(make));
                }
                view()
                    .gap(u(14.))
                    .child(g)
                    .child(label("reflowing grid: pin tiles in after the first").text_size(u(size::SMALL)).text_color(move || palette().subtle))
                    .child(
                        view().flex_row().gap(u(8.))
                            .child(button("pin a tile").on_click(move || pinned.update(|p| *p += 1)))
                            .child(button("unpin").on_click(move || pinned.update(|p| *p = p.saturating_sub(1)))),
                    )
                    .child(tile_grid().columns(cols).metrics(unit.get(), gap.get()).source(move || {
                        let mut v: Vec<_> = (0..6).map(make).collect();
                        for p in (0..pinned.get()).rev() {
                            v.insert(1, tile(format!("pin{p}"), TileSize::Small).icon(Icon::Favorite).color(Accent::Teal.color()));
                        }
                        v
                    }))
                    .into_element()
            };
            pg(
                preview,
                vec![
                    choice("columns", &["4", "6", "8"], columns),
                    number("unit (px)", unit, 32., 99., 1.),
                    number("gap (px)", gap, 2., 16., 1.),
                    number("tiles", n, 1., 24., 1.),
                    flag("turnstile", turnstile),
                    flag("two groups", groups),
                    knob_row("entrance", button("replay").on_click(move || replay.update(|r| *r += 1))),
                ],
            )
        }
        Page::Pivot => {
            let headers = signal(4f32);
            let has_title = signal(true);
            let sel = signal(0usize);
            const NAMES: [&str; 6] = ["inbox", "unread", "flagged", "urgent", "sent", "drafts"];
            let preview = move || {
                let k = headers.get() as usize;
                let mut p = pivot(NAMES[..k].iter().copied(), move || sel.get().min(k - 1))
                    .on_select(move |i| sel.set(i))
                    .panes((0..k).map(|i| {
                        move || {
                            view().px(u(24.)).children((0..5).map(move |r| {
                                list_item(format!("{} {r}", NAMES[i])).subtitle("subtitle line").detail("detail line")
                            }))
                        }
                    }));
                if has_title.get() {
                    p = p.title("outlook");
                }
                p.into_element()
            };
            pg(preview, vec![number("headers", headers, 1., 6., 1.), flag("title", has_title), choice("selected", &NAMES, sel)])
                .height(440.)
        }
        Page::Panorama => {
            let sections = signal(4f32);
            let width = signal(380f32);
            let sel = signal(0usize);
            let title = signal("music + videos".to_string());
            let preview = move || {
                let k = sections.get() as usize;
                let mut p = panorama(title.get(), move || sel.get().min(k - 1)).section_width(width.get()).on_select(move |i| sel.set(i));
                for i in 0..k {
                    p = p.section(format!("section {i}"), move || {
                        view().children((0..4).map(move |r| list_item(format!("item {i}.{r}")).subtitle("artist")))
                    });
                }
                p.into_element()
            };
            pg(
                preview,
                vec![
                    words("title", title),
                    number("sections", sections, 1., 6., 1.),
                    number("section width", width, 260., 560., 10.),
                    choice("selected", &["0", "1", "2", "3", "4", "5"], sel),
                ],
            )
            .height(440.)
        }
        Page::AppBar => {
            let mode = signal(0usize);
            let buttons = signal(3f32);
            let disabled = signal(true);
            let items = signal(2f32);
            let opacity = signal(1f32);
            let log = signal(String::from("—"));
            let preview = move || {
                let mut b = app_bar()
                    .mode(if mode.get() == 0 { AppBarMode::Default } else { AppBarMode::Minimized })
                    .opacity(opacity.get());
                for i in 0..buttons.get() as usize {
                    let name = ICONS[i];
                    b = b.button(icon_of(i), name, move || log.set(format!("tapped {name}")));
                }
                if disabled.get() {
                    b = b.disabled_button(Icon::Delete, "delete");
                }
                for i in 0..items.get() as usize {
                    b = b.menu_item(format!("menu item {i}"), move || log.set(format!("menu item {i}")));
                }
                view()
                    .h_full()
                    .justify_end()
                    .child(label(move || log.get()).p(u(12.)).text_color(move || palette().subtle))
                    .child(b)
                    .into_element()
            };
            pg(
                preview,
                vec![
                    choice("mode", &["default", "minimized"], mode),
                    number("buttons", buttons, 0., 4., 1.),
                    flag("disabled button", disabled),
                    number("menu items", items, 0., 5., 1.),
                    number("opacity", opacity, 0., 1., 0.05),
                ],
            )
            .height(320.)
        }
        Page::Button => {
            let caption = signal("send".to_string());
            let has_icon = signal(false);
            let ic = signal(6usize);
            let accent = signal(false);
            let disabled = signal(false);
            let full = signal(false);
            let taps = signal(0u32);
            let preview = move || {
                let mut b = button(caption.get()).on_click(move || taps.update(|n| *n += 1));
                if has_icon.get() {
                    b = b.icon(icon_of(ic.get()));
                }
                if accent.get() {
                    b = b.accent();
                }
                if full.get() {
                    b = b.full_width();
                }
                view()
                    .gap(u(10.))
                    .child(b.disabled(disabled.get()))
                    .child(label(move || format!("{} taps", taps.get())).text_size(u(size::SMALL)).text_color(move || palette().subtle))
                    .into_element()
            };
            let variants = view().gap(u(10.)).children([false, true].into_iter().map(|dis| {
                row().children([(false, false), (true, false), (false, true), (true, true)].into_iter().map(move |(acc, ic)| {
                    let mut b = button(if acc { "save" } else { "send" }).disabled(dis);
                    if acc {
                        b = b.accent();
                    }
                    if ic {
                        b = b.icon(Icon::Share);
                    }
                    tagged(format!("{}{}{}", if acc { "accent" } else { "normal" }, if ic { " + glyph" } else { "" }, if dis { ", disabled" } else { "" }), b)
                }))
            }));
            pg(
                preview,
                vec![
                    words("label", caption),
                    flag("glyph", has_icon),
                    choice("which glyph", &ICONS, ic),
                    flag("accent", accent),
                    flag("disabled", disabled),
                    flag("full width", full),
                ],
            )
            .variants(variants)
        }
        Page::RoundButton => {
            let ic = signal(1usize);
            let d = signal(48f32);
            let filled = signal(false);
            let disabled = signal(false);
            let taps = signal(0u32);
            let preview = move || {
                view()
                    .flex_row()
                    .items_center()
                    .gap(u(14.))
                    .child(
                        round_button(icon_of(ic.get()))
                            .diameter(d.get())
                            .filled(filled.get())
                            .disabled(disabled.get())
                            .on_click(move || taps.update(|n| *n += 1)),
                    )
                    .child(label(move || format!("{} taps", taps.get())).text_size(u(size::SMALL)).text_color(move || palette().subtle))
                    .into_element()
            };
            let variants = row().children([(false, false), (true, false), (false, true), (true, true)].into_iter().map(|(f, dis)| {
                tagged(format!("{}{}", if f { "filled" } else { "outline" }, if dis { ", disabled" } else { "" }), round_button(Icon::Play).filled(f).disabled(dis))
            }));
            pg(
                preview,
                vec![choice("glyph", &ICONS, ic), number("diameter", d, 24., 96., 1.), flag("filled", filled), flag("disabled", disabled)],
            )
            .variants(variants)
        }
        Page::Hyperlink => {
            let caption = signal("privacy statement".to_string());
            let taps = signal(0u32);
            let preview = move || {
                view()
                    .gap(u(10.))
                    .child(hyperlink(caption.get()).on_click(move || taps.update(|n| *n += 1)))
                    .child(label(move || format!("{} taps", taps.get())).text_size(u(size::SMALL)).text_color(move || palette().subtle))
                    .into_element()
            };
            pg(preview, vec![words("label", caption)])
        }
        Page::ToggleSwitch => {
            let on = signal(true);
            let header = signal("wi-fi networking".to_string());
            let on_l = signal("On".to_string());
            let off_l = signal("Off".to_string());
            let disabled = signal(false);
            let preview = move || {
                let mut t = toggle_switch(on).labels(on_l.get(), off_l.get()).disabled(disabled.get()).on_change(move |v| on.set(v));
                let h = header.get();
                if !h.is_empty() {
                    t = t.header(h);
                }
                view().w(u(380.)).child(t).into_element()
            };
            let variants = view().w(u(380.)).gap(u(14.)).children([(true, false), (false, false), (true, true), (false, true)].into_iter().map(|(v, dis)| {
                toggle_switch(v).header(format!("{}{}", if v { "on" } else { "off" }, if dis { ", disabled" } else { "" })).disabled(dis)
            }));
            pg(
                preview,
                vec![
                    flag("on", on),
                    words("header", header),
                    words("on label", on_l),
                    words("off label", off_l),
                    flag("disabled", disabled),
                ],
            )
            .variants(variants)
        }
        Page::CheckBox => {
            let checked = signal(true);
            let caption = signal("send usage data".to_string());
            let disabled = signal(false);
            let preview = move || {
                let mut c = check_box(checked).disabled(disabled.get()).on_change(move |v| checked.set(v));
                let l = caption.get();
                if !l.is_empty() {
                    c = c.label(l);
                }
                c.into_element()
            };
            let variants = view().gap(u(4.)).children([(true, false), (false, false), (true, true), (false, true)].into_iter().map(|(v, dis)| {
                check_box(v).label(format!("{}{}", if v { "checked" } else { "unchecked" }, if dis { ", disabled" } else { "" })).disabled(dis)
            }));
            pg(preview, vec![flag("checked", checked), words("label", caption), flag("disabled", disabled)]).variants(variants)
        }
        Page::RadioButton => {
            let sel = signal(0usize);
            let disabled = signal(false);
            let count = signal(3f32);
            let preview = move || {
                view()
                    .children((0..count.get() as usize).map(|i| {
                        radio_button(move || sel.get() == i)
                            .label(format!("option {i}"))
                            .disabled(disabled.get())
                            .on_select(move || sel.set(i))
                    }))
                    .into_element()
            };
            let variants = view().gap(u(4.)).children([(true, false), (false, false), (true, true), (false, true)].into_iter().map(|(v, dis)| {
                radio_button(v).label(format!("{}{}", if v { "selected" } else { "not selected" }, if dis { ", disabled" } else { "" })).disabled(dis)
            }));
            pg(preview, vec![number("options", count, 1., 6., 1.), choice("selected", &["0", "1", "2", "3", "4", "5"], sel), flag("disabled", disabled)])
                .variants(variants)
        }
        Page::Slider => {
            let v = signal(0.4f32);
            let header = signal("volume".to_string());
            let stepped = signal(false);
            let step = signal(0.25f32);
            let disabled = signal(false);
            let preview = move || {
                let mut s = slider(v).disabled(disabled.get()).on_change(move |x| v.set(x));
                let h = header.get();
                if !h.is_empty() {
                    s = s.header(h);
                }
                if stepped.get() {
                    s = s.step(step.get());
                }
                view()
                    .w(u(380.))
                    .gap(u(6.))
                    .child(s)
                    .child(label(move || format!("value {:.2}", v.get())).text_size(u(size::SMALL)).text_color(move || palette().subtle))
                    .into_element()
            };
            pg(
                preview,
                vec![
                    number("value", v, 0., 1., 0.01),
                    words("header", header),
                    flag("stepped", stepped),
                    number("step", step, 0.05, 0.5, 0.05),
                    flag("disabled", disabled),
                ],
            )
        }
        Page::ListPicker => {
            let n = signal(4f32);
            let sel = signal(0usize);
            let header = signal("ringtone".to_string());
            const ITEMS: [&str; 6] = ["chimes", "bells", "ripple", "xylophone", "marimba", "nothing"];
            let preview = move || {
                let k = n.get() as usize;
                let mut p = list_picker(ITEMS[..k].iter().copied(), move || sel.get().min(k - 1)).on_select(move |i| sel.set(i));
                let h = header.get();
                if !h.is_empty() {
                    p = p.header(h);
                }
                view().w(u(380.)).child(p).into_element()
            };
            pg(preview, vec![number("items", n, 1., 6., 1.), choice("selected", &ITEMS, sel), words("header", header)])
        }
        Page::TextBox => {
            let value = signal(String::new());
            let header = signal("name".to_string());
            let placeholder = signal("type here".to_string());
            let masked = signal(false);
            let submitted = signal(String::from("—"));
            let preview = move || {
                let mut t = text_box(value).placeholder(placeholder.get()).on_submit(move |s| submitted.set(s));
                let h = header.get();
                if !h.is_empty() {
                    t = t.header(h);
                }
                if masked.get() {
                    t = t.masked();
                }
                view()
                    .w(u(380.))
                    .gap(u(8.))
                    .child(t)
                    .child(label(move || format!("submitted: {}", submitted.get())).text_size(u(size::SMALL)).text_color(move || palette().subtle))
                    .into_element()
            };
            pg(preview, vec![words("header", header), words("placeholder", placeholder), flag("password", masked), words("value", value)])
        }
        Page::ProgressBar | Page::ProgressDots | Page::ProgressRing => {
            let kind = signal(match page {
                Page::ProgressBar => 0,
                Page::ProgressDots => 1,
                _ => 2,
            });
            let v = signal(0.4f32);
            let d = signal(48f32);
            let preview = move || match kind.get() {
                0 => view().w(u(380.)).child(progress_bar(v)).into_element(),
                1 => view().w(u(380.)).child(progress_dots()).into_element(),
                _ => progress_ring().diameter(d.get()).into_element(),
            };
            let variants = view()
                .w(u(380.))
                .gap(u(16.))
                .child(tagged("bar 0%", progress_bar(0.)))
                .child(tagged("bar 60%", progress_bar(0.6)))
                .child(tagged("bar 100%", progress_bar(1.)))
                .child(tagged("dots", progress_dots()))
                .child(row().child(tagged("ring 24", progress_ring().diameter(24.))).child(tagged("ring 40", progress_ring())).child(tagged("ring 64", progress_ring().diameter(64.))));
            pg(
                preview,
                vec![choice("kind", &["bar", "dots", "ring"], kind), number("bar value", v, 0., 1., 0.01), number("ring diameter", d, 16., 128., 1.)],
            )
            .variants(variants)
        }
        Page::ListItem => {
            let title = signal("ben thompson".to_string());
            let sub = signal(true);
            let detail = signal(true);
            let subtle = signal(false);
            let leading = signal(1usize);
            let color = signal(0usize);
            let trailing = signal(false);
            let tappable = signal(true);
            let taps = signal(0u32);
            let preview = move || {
                let mut it = list_item(title.get());
                if sub.get() {
                    it = it.subtitle("mobile");
                }
                if detail.get() {
                    it = it.detail("seattle");
                }
                if subtle.get() {
                    it = it.subtle_subtitle();
                }
                match leading.get() {
                    1 => it = it.leading_icon(Icon::People),
                    2 => it = it.leading(Leading::Glyph("bt".into())),
                    3 => it = it.leading(Leading::Element(view().size(u(30.)).rounded(px(15.)).bg(rgb(0xFFFFFF)).into_element())),
                    _ => {}
                }
                if let Some(c) = accent_of(color.get()) {
                    it = it.leading_color(c);
                }
                if trailing.get() {
                    it = it.trailing(Icon::ChevronRight.render(24., move || palette().subtle));
                }
                if tappable.get() {
                    it = it.on_click(move || taps.update(|n| *n += 1));
                }
                view()
                    .w(u(420.))
                    .gap(u(6.))
                    .child(it)
                    .child(label(move || format!("{} taps", taps.get())).text_size(u(size::SMALL)).text_color(move || palette().subtle))
                    .into_element()
            };
            pg(
                preview,
                vec![
                    words("title", title),
                    flag("subtitle", sub),
                    flag("detail line", detail),
                    flag("subtle subtitle", subtle),
                    choice("leading", &["none", "glyph", "letters", "element"], leading),
                    choice("leading colour", &ACCENTS, color),
                    flag("trailing", trailing),
                    flag("tappable", tappable),
                ],
            )
        }
        Page::JumpList => {
            let groups = signal(6f32);
            let all_keys = signal(true);
            let preview = move || {
                let k = groups.get() as usize;
                let letters = ["a", "b", "c", "f", "k", "m", "s", "t", "w", "z"];
                let mut j = jump_list().groups(letters[..k].iter().map(|l| {
                    ListGroup::new(*l, (0..3).map(|i| list_item(format!("{l}{i} contact")).subtitle("mobile").into_element()))
                }));
                if all_keys.get() {
                    j = j.keys(nsh_metro::ALPHABET);
                }
                j.into_element()
            };
            pg(preview, vec![number("groups", groups, 1., 10., 1.), flag("whole alphabet", all_keys)]).height(440.)
        }
        Page::MessageBox => {
            let open = signal(true);
            let title = signal("delete this run?".to_string());
            let message = signal("R-4712 and its logs will be gone.".to_string());
            let buttons = signal(2f32);
            let dismissable = signal(true);
            let preview = move || {
                let shown = view().child(button("show message box").on_click(move || open.set(true)));
                let mut m = message_box(title.get());
                let msg = message.get();
                if !msg.is_empty() {
                    m = m.message(msg);
                }
                for i in 0..buttons.get() as usize {
                    m = m.button(["delete", "cancel", "later"][i], move || open.set(false));
                }
                if dismissable.get() {
                    m = m.on_dismiss(move || open.set(false));
                }
                let m = Rc::new(std::cell::RefCell::new(Some(m)));
                view()
                    .size_full()
                    .child(shown)
                    .child(when(move || open.get(), move || m.borrow_mut().take().map(IntoElement::into_element).unwrap_or_else(|| view().into_element())))
                    .into_element()
            };
            pg(
                preview,
                vec![
                    words("title", title),
                    words("message", message),
                    number("buttons", buttons, 0., 3., 1.),
                    flag("tap outside dismisses", dismissable),
                ],
            )
            .height(360.)
        }
        Page::Toast => {
            let title = signal("no ativ".to_string());
            let message = signal("R-4712 harvested".to_string());
            let has_icon = signal(true);
            let ic = signal(0usize);
            let shown = signal(0usize);
            let preview = move || {
                shown.get();
                let mut t = toast(title.get());
                let m = message.get();
                if !m.is_empty() {
                    t = t.message(m);
                }
                if has_icon.get() {
                    t = t.icon(icon_of(ic.get()));
                }
                view()
                    .size_full()
                    .justify_end()
                    .child(button("show again").on_click(move || shown.update(|n| *n += 1)))
                    .child(t)
                    .into_element()
            };
            pg(preview, vec![words("title", title), words("message", message), flag("icon", has_icon), choice("glyph", &ICONS, ic)]).height(220.)
        }
        Page::SettingsFlyout => {
            let open = signal(true);
            let wide = signal(0usize);
            let title = signal("options".to_string());
            let back = signal(false);
            let log = signal(String::from("—"));
            let preview = move || {
                let mut f = settings_flyout(open)
                    .title(title.get())
                    .width(if wide.get() == 0 { FlyoutWidth::Narrow } else { FlyoutWidth::Wide })
                    .on_dismiss(move || open.set(false))
                    .child(|| toggle_switch(true).header("sync"))
                    .child(|| slider(0.5).header("volume"));
                if back.get() {
                    f = f.on_back(move || log.set("back pressed".into()));
                }
                view()
                    .size_full()
                    .relative()
                    .gap(u(8.))
                    .child(button("open flyout").on_click(move || open.set(true)))
                    .child(label(move || log.get()).text_size(u(size::SMALL)).text_color(move || palette().subtle))
                    .child(f)
                    .into_element()
            };
            pg(
                preview,
                vec![
                    flag("open", open),
                    choice("width", &["narrow 346", "wide 646"], wide),
                    words("title", title),
                    flag("own back handler", back),
                ],
            )
            .height(440.)
        }
        Page::SystemTray => {
            let battery = signal(0.8f32);
            let opaque = signal(false);
            let time = signal("12:38".to_string());
            let preview = move || {
                let mut t = system_tray(time.get()).battery(battery.get());
                if opaque.get() {
                    t = t.opaque();
                }
                view().w(u(420.)).child(t).into_element()
            };
            pg(preview, vec![number("battery", battery, 0., 1., 0.05), flag("opaque", opaque), words("time", time)])
        }
        Page::Header => {
            let title = signal("strains".to_string());
            let app = signal("no ativ".to_string());
            let preview = move || {
                let mut h = page_header(title.get());
                let a = app.get();
                if !a.is_empty() {
                    h = h.app_title(a);
                }
                h.into_element()
            };
            pg(preview, vec![words("title", title), words("app title", app)])
        }
        Page::Motion => {
            use nsh_metro::motion::{Direction, swivel_in, tilt, turnstile_in};
            let kind = signal(0usize);
            let dir = signal(0usize);
            let again = signal(0usize);
            let preview = move || {
                again.get();
                let d = if dir.get() == 0 { Direction::Forward } else { Direction::Backward };
                let card = || view().w(u(260.)).h(u(120.)).bg(move || palette().accent).p(u(12.)).child(label("metro").text_size(u(size::LARGE)).text_color(rgb(0xFFFFFF)));
                let el = match kind.get() {
                    0 => turnstile_in(d, 0., 20., card()).into_element(),
                    1 => view().gap(u(8.)).children((0..4).map(|i| turnstile_in(d, i as f32 * nsh_metro::motion::FEATHER, 20., card().h(u(40.))))).into_element(),
                    2 => swivel_in(card()).into_element(),
                    _ => tilt(card().cursor_pointer().on_press(|| {})).into_element(),
                };
                view().gap(u(12.)).child(button("play again").on_click(move || again.update(|n| *n += 1))).child(el).into_element()
            };
            pg(
                preview,
                vec![
                    choice("motion", &["turnstile", "feather", "swivel", "tilt (press it)"], kind),
                    choice("direction", &["forward", "backward"], dir),
                ],
            )
            .height(360.)
        }
    }
}

#[allow(dead_code)]
fn _font() -> &'static str {
    FONT_FAMILY
}
