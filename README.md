# nsh-metro

The Windows Phone "Metro" design language for [Native Shell](https://nsh.dev)
(nsh): type-first layouts, flat accent colour, live tiles, pivots, panoramas
and the rest, drawn by the platform's own views (AppKit on macOS) with their
motion on the platform's animation clock.

A port of [gpui-metro](https://github.com/JUSTIVE/gpui-metro). The motion is
native here: the turnstile and the tile tilt are real 3D rotations, a live
tile's peek is one looping animation started once, and nothing re-renders per
frame. See [`crates/nsh-metro/MIGRATION.md`](crates/nsh-metro/MIGRATION.md)
for how it was moved and what changed.

## Run

```sh
cargo run --release -p nsh-catalog          # every component, with a playground per page
cargo run --release -p nsh-no-ativ          # an app built on it
cargo run --release -p nsh-perf-incubator   # CPU (app + WindowServer) per scene
```

`CATALOG_PAGE=pivot` opens the catalog on a page. Requires Rust 1.97+ and
macOS 14+.

## Layout

| path | what |
|---|---|
| `crates/nsh-metro` | the design system: theme, icons, motion, components |
| `apps/nsh-catalog` | the component catalog |
| `apps/nsh-no-ativ` | a lab dashboard app |
| `apps/nsh-perf-incubator` | stress scenes measured by CPU |
| `apps/nsh-hello` | smoke test: a reflowing tile grid |
