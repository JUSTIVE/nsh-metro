# AGENTS.md

This is the guide for coding agents working **in** this repository. If you are building an app **with** nsh-metro, read the skill instead: [`plugins/nsh-metro/skills/nsh-metro/SKILL.md`](plugins/nsh-metro/skills/nsh-metro/SKILL.md).

## What this is

nsh-metro is the Windows Phone "Metro" design system for [Native Shell](https://nsh.dev). It targets nsh-core 0.6.1 on the macOS AppKit backend, Rust 1.97+ and macOS 14+. It is a port of [gpui-metro](https://github.com/JUSTIVE/gpui-metro); [`crates/nsh-metro/MIGRATION.md`](crates/nsh-metro/MIGRATION.md) explains how it was moved.

## Index

| path | what |
|---|---|
| `crates/nsh-metro/src/theme.rs` | `Theme`, `Palette`, `Accent`, type ramp, `u()` |
| `crates/nsh-metro/src/motion.rs` | curves, durations, turnstile, tilt, swivel, pane slide, `Track` |
| `crates/nsh-metro/src/nav.rs` | `page_host` (turnstile page navigation) |
| `crates/nsh-metro/src/icons.rs` + `assets/icons/` | the `Icon` set (SVG, tinted) |
| `crates/nsh-metro/src/components/` | one file per component family: `tile.rs` (tiles, grids, live tiles, reflow), `pivot.rs`, `panorama.rs`, `app_bar.rs`, `flyout.rs`, `dialog.rs`, `context_menu.rs`, `list.rs`, `toggle.rs`, `button.rs`, `slider.rs`, `text_box.rs`, `progress.rs`, `chrome.rs`, `text.rs`, `pager.rs` (swipe) |
| `apps/nsh-catalog` | every component, with a per-page options playground (`CATALOG_PAGE=<page>`) |
| `apps/nsh-starter` | the template the skill hands out (a copy lives at `plugins/nsh-metro/skills/nsh-metro/assets/starter.rs`) |
| `apps/nsh-no-ativ` | a full app built on the library |
| `apps/nsh-perf-incubator` | stress scenes that report app and WindowServer CPU |
| `apps/nsh-hello` | smoke test: a reflowing grid |
| `plugins/nsh-metro/` | the Claude Code plugin (skill and references); `.claude-plugin/marketplace.json` makes this repo installable |

## Commands

```sh
cargo build --workspace
cargo clippy --workspace --all-targets      # must be warning-free (CI: -D warnings)
cargo test --workspace
cargo fmt --all
RUSTDOCFLAGS="-D warnings" cargo doc -p nsh-metro --no-deps
cargo deny check                            # licenses, advisories
cargo run --release -p nsh-catalog          # look at the result
```

CI (`.github/workflows/ci.yml`) runs all of these. It also checks that the skill's `starter.rs` copy matches `apps/nsh-starter/src/main.rs`.

## Rules

- Lints are strict: clippy pedantic, `missing_docs` on the library, `unsafe_code = "deny"`. Fix the code rather than adding an `allow`. If an `allow` is really needed, put it on the item and give the reason.
- Every public item needs a doc comment. Builders are `#[must_use]`.
- Options that can change take `impl IntoReactive<T>`. Options that change a control's structure go through `components::rebuilt_on`; the rest bind to their property.
- Start composite animations over shared values through a `motion::Track`, so a restart stops the previous run.
- Create animated values and signals under a long-lived owner, never inside an `effect` that re-runs (see `TileGrid`'s `Owner::current()`).
- Match gpui-metro's look and timing. The constants carry WP Toolkit values; change them only with a reason.
- Keep pure logic (packing, schedules, curves) in free functions with unit tests.
- After a visual change, run the catalog and look at it.
- When you change the public API, update `plugins/nsh-metro/skills/nsh-metro/references/components.md`. When you change the starter, copy it to `assets/starter.rs`.
- The downstream app claudmagi depends on `set_font_family`, `TileGrid::live_columns`, `TileGrid::on_tap` and `motion::set_ambient_fps`. Don't remove them without a replacement.
