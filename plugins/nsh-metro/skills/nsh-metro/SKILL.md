---
name: nsh-metro
description: Build native macOS apps in Rust in the Windows Phone "Metro" design language with the nsh-metro crate on Native Shell (nsh, nsh-core 0.6.1). Use when creating or changing an app that depends on nsh-metro or asks for a Metro / Windows Phone / live-tile / pivot / panorama / turnstile UI on nsh, and when porting a gpui-metro app to nsh.
---

# nsh-metro

nsh-metro is the Windows Phone Metro design system for [Native Shell](https://nsh.dev).
It provides type-first pages, flat accent colours, live tiles, pivots, panoramas and turnstile navigation. Everything is drawn by AppKit views, and the motion runs on Core Animation.

- **Platform:** macOS 14+, Rust 1.97+.
- **Source:** https://github.com/JUSTIVE/nsh-metro

Look things up in this order:
1. Start from the starter for a new app: `assets/starter.rs`.
2. Check `references/components.md` for every component and its options.
3. Read `references/pitfalls.md` before you debug anything odd.

## Set up

```toml
[dependencies]
nsh-core = "0.6.1"
nsh-platform-macos = { version = "0.6.1", default-features = false, features = ["svg"] }
nsh-metro = { git = "https://github.com/JUSTIVE/nsh-metro" }
```

Then copy `assets/starter.rs` to `src/main.rs`. CI compiles it on every push as `apps/nsh-starter`. It shows:
- a start screen of tiles
- a tile tap that turnstiles into a pivot app
- a back button
- an app bar
- a settings flyout that switches the theme

```rust
use nsh_core::prelude::*;
use nsh_metro::prelude::*;
use nsh_metro::text; // `text` exists in both preludes; naming it picks Metro's
use nsh_platform_macos as platform;
```

## The model, in five rules

1. **A component body runs once.** `#[component] fn App()` and every `into_element` build the tree a single time. Nothing re-renders per frame. A value that changes is a *closure bound to a property*, for example `.bg(move || palette().background)` or `text(TextStyle::Normal, move || format!("{} new", n.get()))`. Only that closure re-runs.
2. **State is signals.**
   - Create state with `signal(v)`. Read it with `.get()` (tracked) and change it with `.set` or `.update`.
   - Signals are `Copy`, so you can `move` them into as many closures as you need.
   - Use `effect(move || …)` for side effects.
   - Use `dynamic(move || …)` only when the *structure* has to change. A plain property change never needs it.
3. **Options that can change take `impl IntoReactive<T>`.** You can pass a plain value, a `Signal`/`ReadSignal`, or a `move ||` closure. Examples: `toggle_switch(on)`, `.disabled(airplane)`, `.count(unread)`, `.color(move || …)`. Don't rebuild a control to change one of these options.
4. **The root returns content, not `window()`.** The window's title and size come from `platform::AppOptions { title, size, .. }`. Returning `window()` opens a second window.
5. **Colours come from the theme.**
   - Always read them through `palette()` inside a closure: `palette().foreground / .background / .accent / .subtle / .chrome / .disabled …`. Then `Theme::set_mode(Mode::Light)` and `Theme::set_accent(Accent::Teal)` re-skin the whole app live.
   - Use `Accent::X.color()` for a fixed tile colour.
   - Use `u(px)` for every length.

## The nsh-core you need

| call | what |
|---|---|
| `signal(v) -> Signal<T>` | state; `Copy`; `.get()` tracked, `.get_untracked()`, `.set(v)`, `.update(\|v\| …)`, `.read_only()` |
| `effect(move \|\| …)` | re-runs when what it read changes |
| `when(cond: R<bool>, \|\| el) -> If` | shows `el` while `cond` holds (`.child(when(..))`); the element is built each time it appears |
| `dynamic(move \|\| el)` | rebuilds `el` whenever what the closure read changes: for structure, not for a property |
| `each(items: R<Vec<T>>, \|item: &T\| key, move \|item: ReadSignal<T>, index: ReadSignal<usize>\| el)` | a keyed list that follows a `Signal<Vec<T>>` (`T: Clone + PartialEq`): rows are kept by key, not rebuilt |
| `view()` / `scroll()` | a column (`.flex_row()` for a row) / a scrolling column |
| `set_timeout(dur, f)` / `set_interval(dur, f)` | runs while the owner it was made under (the component) lives; `.cancel()` on the returned handle stops it early |
| `untracked(\|\| …)` | reads without subscribing |

Handlers everywhere are `impl Fn(..) + 'static`: they run many times, so
capture `Copy` signals or clone owned data into them (`let name = name.clone();
move || play(&name)`). A helper that builds UI and names its return type
returns `AnyElement` (`….into_element()`); otherwise `impl IntoElement`.

## Building blocks at a glance

| need | use |
|---|---|
| page title / app title | `page_header("inbox").app_title("mail")`, `text(TextStyle::Title1, …)` |
| type ramp | `text(TextStyle::{Title1,Title2,Title3,ExtraLarge,Large,Normal,Subtle,Accent,Small,AppTitle,GroupHeader}, s)`; raw: `label(s)` |
| start screen | `tile_grid().turnstile(true).tiles([...])` or `.source(move \|\| vec![...])` for tiles that change |
| a tile | `tile("unique-key", TileSize::{Small,Medium,Wide,Large}).title(..).icon(Icon::Mail).count(sig).color(..).back_text(..).on_click(..)` |
| a list that changes | `each(songs, \|s: &Song\| s.id, move \|s, _\| list_item(s.get().title))` |
| page navigation | `page_host(sig_of_(Page, Direction), 0., move \|p\| …)`: turnstile out, then in |
| tabs | `pivot(["a","b"], sel).title(..).child(\|\| paneA).child(\|\| paneB).on_select(move \|i\| sel.set(i))` |
| hub | `panorama("title", sel).section("one", \|\| …).on_select(move \|i\| sel.set(i))` |
| commands | `app_bar().button(Icon::Add, "new", f).menu_item("settings", f)` |
| settings pane | `settings_flyout(open).title(..).child(\|\| …).on_dismiss(move \|\| open.set(false))` |
| dialog | `when(move \|\| open.get(), move \|\| message_box("title").message(..).button("ok", f).on_dismiss(f))` |
| controls | `button`, `round_button`, `hyperlink`, `toggle_switch`, `check_box`, `radio_button`, `slider`, `text_box(Signal<String>)`, `list_picker` |
| lists | `list_item("title").subtitle(..).detail(..).leading_icon(..)`, `jump_list().groups(..)` |
| right-click menu | `with_context_menu(list_item(..), context_menu().item("play", f).item("delete", g))`, per row |
| progress | `progress_bar(value)`, `progress_dots()`, `progress_ring()` |

## Working on it

- Look at the live catalog for any component: `cargo run --release -p nsh-catalog`. `CATALOG_PAGE=pivot` opens a given page, and each page has a playground with every option.
- Use real nsh-metro components rather than hand-rolling look-alikes. They carry the Metro motion (tilt on press, turnstile, swivel, pane slide), which is the part that is easy to get wrong.
- Use lower case for Metro titles, headers and commands (`"settings"`, not `"Settings"`).
- Every tile in a grid needs a unique, stable key. A reflowing grid matches tiles by key to slide them.
- To verify a UI change, build, run the app and take a screenshot. Type-checking is not enough for layout and motion.
