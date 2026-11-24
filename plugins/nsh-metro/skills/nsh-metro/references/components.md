# nsh-metro component reference

All builders are `#[must_use]`: a builder does nothing until it is placed in
the tree (`.child(…)`). "R<T>" below means `impl IntoReactive<T>`: a plain
value, a `Signal<T>` / `ReadSignal<T>`, or a `move || …` closure, so the
option follows state without a rebuild. Lengths are px through `u(px)`.
Plain text arguments (`label`, `title`, `header`, `message`, list items) are
`impl Into<SharedString>`: `&'static str` or `String`, fixed once built; use
an `R<SharedString>` option, or rebuild with `when` / `dynamic`, for text
that changes. Every handler is `impl Fn(..) + 'static`.

The rustdoc (`cargo doc -p nsh-metro --open`) documents every item; this is
the map.

## Theme — `nsh_metro::theme`

| item | notes |
|---|---|
| `palette()` | the current `Palette` (tracked): `background foreground subtle disabled chrome border input input_focused input_ink scrim accent accent_ink`. Read it inside closures. |
| `Theme::set_mode(Mode::Dark \| Mode::Light)` | re-skins everything bound to `palette()` |
| `Theme::set_accent(Accent::Cobalt)` | `Accent` is `Copy`; `.name()` is its lower-case name. 20 accents: Lime Green Emerald Teal Cyan Cobalt Indigo Violet Pink Magenta Crimson Red Orange Amber Yellow Brown Olive Steel Mauve Taupe; `Accent::ALL`, `.color()`, `.name()` |
| `Theme::set_reduced_motion(bool)` | components built afterwards skip entrance motion |
| `Theme::get()` | the theme now (tracked) |
| `set_font_family("Segoe UI")` | call before building; default Helvetica Neue |
| `size::{SMALL 14, NORMAL 17, MEDIUM 20, MEDIUM_LARGE 22, LARGE 27, EXTRA_LARGE 36, HUGE 56, PANORAMA 112}` | the WP type ramp |
| `LIGHT`, `SEMILIGHT`, `SEMIBOLD`, `FONT_FAMILY()` | weights and the face |
| `u(px)` | a length |
| `PAGE_MARGIN` (24) | the page's side margin |

## Text

| builder | options |
|---|---|
| `text(TextStyle, R<SharedString>)` | `.color(R<AdaptiveColor>)`, `.lowercase()`, `.truncate()`. Styles: `Title1 Title2 Title3 ExtraLarge Large Normal Subtle Accent Small AppTitle GroupHeader`. Import by name: `use nsh_metro::text;` |
| `label(R<SharedString>)` | an nsh `Text` in the Metro face and line height, unstyled otherwise |
| `page_header(title)` | `.app_title(app)`: the small upper-case line above |

## Icons

`Icon::{Add Alert Back Battery Calendar Camera Check ChevronDown ChevronRight Clock Close Delete Dna Download Edit Favorite Filter Flask Forward Gauge Grid Home List Lock Mail More Next Pause People Phone Pin Play Previous Refresh Save Search Settings Share Signal Snow Sync Thermometer Truck Upload Wave Wifi}`;
`icon(Icon, size, R<AdaptiveColor>)` draws one; `Icon::ALL` lists them.

## Tiles

`tile(key, TileSize::{Small, Medium, Wide, Large})` — key unique in its grid.

| option | |
|---|---|
| `.title(R<SharedString>)` | bottom-left label |
| `.icon(Icon)` | centred glyph |
| `.count(R<u32>)` | badge beside the glyph; a change pushes the glyph aside |
| `.color(R<Hsla>)` | default: the accent |
| `.front(\|size\| el)` / `.back(\|size\| el)` | custom faces; a back face makes it a live tile |
| `.back_text(title, lines)` | a text back face |
| `.period(secs)` / `.phase(0..1)` | live-tile peek cycle |
| `.on_click(f)` | tap (tilts toward the pointer) |

`set_live_tile_beat(Some(3.))` puts every live tile on a shared beat.

`tile_grid()`:

| option | |
|---|---|
| `.tiles([..])` / `.tile(t)` | a fixed grid |
| `.group(title, [..])` | horizontal groups, as on the Windows 8 start screen |
| `.source(move \|\| vec![..])` | tiles that change: re-packs, slides moved tiles, turns new ones in |
| `.columns(n)` / `.live_columns(R<usize>)` | default 4 |
| `.metrics(unit, gap)` | default 99 / 12 (`TILE_UNIT`, `TILE_GAP`) |
| `.turnstile(true)` | entrance row by row; a tap swings everything away, the tapped tile last, then runs its `on_click` |
| `.replay(ReadSignal<usize>)` | replays the entrance when it changes |
| `.leave(ReadSignal<bool>)` | swings away while true |
| `.on_tap(f)` | runs at the tap, before the exit |
| `.hinge(px)` | distance from the screen's left edge to the grid (turnstile axis) |

## Navigation

| builder | |
|---|---|
| `page_host(R<(K, Direction)>, hinge, \|k\| el)` | shows the page for `k`; a change turnstiles the old page out (`Direction::Forward` / `Backward`) and the new one in. `K: Clone + PartialEq`. |
| `pivot(headers, R<usize>)` | `.title(app)`, `.child(\|\| pane)` per header (or `.panes([..])`), `.on_select(\|i\|)`. Panes slide and their lines trail; headers glide. Swipe works when `on_select` is set. |
| `panorama(title, R<usize>)` | fills its parent (give it a sized parent, e.g. `view().flex_1().min_h(u(0.))`); `.section(header, \|\| el)` (each section is a column, its content clipped to the height; put long content in a `scroll()`), `.section_width(px)`, `.on_select(\|i\|)`. Parallax title; wraps round with 3+ sections. |
| `app_bar()` | place it last in a full-height column (it sits at the bottom, expanding upward over the page); `.button(Icon, label, f)` (max 4), `.disabled_button(..)`, `.menu_item(label, f)`, `.mode(R<AppBarMode::{Default, Minimized}>)`, `.opacity(f32)`; "…" expands labels and the menu |
| `system_tray(R<time>)` | `.battery(R<f32>)`, `.opaque()` |

## Overlays

| builder | |
|---|---|
| `settings_flyout(R<bool>)` | `.title`, `.width(FlyoutWidth::{Narrow, Wide})`, `.child(\|\| el)`, `.on_dismiss(f)`, `.on_back(f)`; slides in from the right and stays mounted until it has slid out |
| `message_box(title)` | `.message(s)`, `.child(el)`, `.button(label, f)`, `.on_dismiss(f)`; show with `when(move \|\| open.get(), move \|\| message_box(..))` |
| `toast(title)` | `.message(s)`, `.icon(Icon)`, `.on_click(f)`; show with `when` like the message box |
| `with_context_menu(el, context_menu().item(label, f).disabled_item(label).on_open(\|open\|))` | right-click or press-and-hold; one open at a time; a click outside closes |

## Controls

| builder | options |
|---|---|
| `button(label)` | `.on_click`, `.icon(Icon)`, `.accent()`, `.full_width()`, `.disabled(R<bool>)` |
| `round_button(Icon)` | `.on_click`, `.diameter(px)` (48), `.filled(R<bool>)`, `.disabled(R<bool>)` |
| `hyperlink(label)` | `.on_click` |
| `toggle_switch(R<bool>)` | `.header`, `.labels(on, off)`, `.on_change(\|bool\|)`, `.disabled(R<bool>)` |
| `check_box(R<bool>)` | `.label`, `.on_change(\|bool\|)`, `.disabled(R<bool>)` |
| `radio_button(R<bool>)` | `.label`, `.on_select(\|\|)` (one of many chosen), `.disabled(R<bool>)` |
| `slider(R<f32>)` (0–1) | `.header(R<SharedString>)`, `.step(f32)`, `.on_change(\|f32\|)`, `.disabled(R<bool>)` |
| `text_box(Signal<String>)` | `.header`, `.placeholder`, `.masked()`, `.on_focus_change(\|bool\|)`, `.on_submit(\|String\|)`; the platform's own field |
| `list_picker(items, R<usize>)` | items: `impl IntoIterator<Item: Into<SharedString>>`; `.header`, `.on_select(\|i\|)` (write the index back: the picker shows what you pass); expands in place |

Controls are controlled: they show the state you pass in and never change
it themselves, so write it back in the handler (`toggle_switch(wifi).on_change(move |v| wifi.set(v))`).

## Lists

| builder | |
|---|---|
| `list_item(title)` | `.subtitle`, `.detail`, `.subtle_subtitle()`, `.leading(Leading::{Icon, Glyph, Element})`, `.leading_icon(Icon)`, `.leading_color(R<Hsla>)`, `.trailing(el)`, `.on_click` |
| `jump_list()` | `.group(ListGroup::new(key, items))`, `.groups(..)`, `.keys(..)`: tap a group header for the key grid |

## Progress

`progress_bar(R<f32>)` (determinate), `progress_dots()` (indeterminate dots),
`progress_ring()` with `.diameter(px)`.

## Motion — `nsh_metro::motion`

For custom elements that should move like the rest:
`tilt(el)` (press tilt), `turnstile_in(Direction, delay, hinge, el)`,
`swivel_in(el)`, `turnstile_angle()` + `swing_in` / `swing_out`,
`turned(&angle, hinge, el)`, `PaneSlide` + `slide_line(line, el)` for lines
that trail a sliding pane. Curves: `EXP6_OUT`, `EXP6_IN` (WP's
`ExponentialEase(6)`); durations `ENTER`, `EXIT`, `PANE_IN`, … in ms.
`set_ambient_fps(None)` lifts the 60 fps cap on looping motion.
