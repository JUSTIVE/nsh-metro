# Pitfalls (each one has cost real debugging time)

**A second window opens.** The root component returned `window()`. Return the content view instead, and set the title and size through `platform::AppOptions`.

**`text` is ambiguous.** Both `nsh_core::prelude` and nsh-metro export `text`, and nsh-metro's prelude leaves it out on purpose. Add `use nsh_metro::text;`. A name imported explicitly beats both globs.

**Something never updates.** You read a signal *outside* a closure, for example `.bg(palette().accent)`, so the value was taken once at build. Wrap the read in a closure: `.bg(move || palette().accent)`. The same applies to `text(…, format!(…))`: use `move || format!(…)`.

**An animation or value dies after a state change.** Values made inside an `effect` or `dynamic` are disposed each time that effect re-runs. Create animated values and signals in the component body. If they must be created later, create them under the long-lived owner: `let owner = Owner::current();` at build, then `owner.run(|| animated_value(0.))`.

**A whole control flickers or loses its animation state.** It was wrapped in `dynamic(move || control(...))` to change one option. Pass the signal to the option instead, e.g. `.disabled(sig)` or `.filled(sig)`. Keep `dynamic` for changes of structure.

**An overlay doesn't catch clicks.** An absolutely positioned layer has no size unless you give it one. Use `.absolute().inset_0()`.

**Exit animations don't play.** An element removed from the tree is gone immediately. Use the hosts that keep the old element until it has animated away: `page_host` for pages, `settings_flyout` for its own exit, and a tile grid's `.leave(..)`.

**Tiles jump instead of sliding.** Their keys changed between `source` runs. Keys must be stable: derive them from the data's id, never from the index.

**Two animations fight.** Starting a new `Animation::sequence`/`parallel` over the same values does not stop the previous composite; its delayed steps still fire. Stop the old one first. Inside the crate, `motion::Track` does this for you.

**Lengths are off by 16×.** gpui-metro's `u()` was rems; here `u(x)` is `px(x)`.

**`view()` stacks vertically.** It is a column by default. Use `.flex_row()` for a row.
