# Islands

> Design-stage docs.

A Triblenka page is static HTML by default. An **island** is a component that is allowed to run in
the browser: it renders on the server like everything else, then hydrates on a trigger you choose,
loading only its own code.

```
┌──────────────────────────────────────────┐
│  header (HTML, 0 KB JS)                  │
│  ┌────────────────┐                      │
│  │ <Search        │   ← island, hydrates on click
│  │   client:idle> │                      │
│  └────────────────┘                      │
│  article body (HTML, 0 KB JS)            │
│  ┌────────────────┐                      │
│  │ <Cart          │   ← island, hydrates when visible
│  │  client:visible>│                     │
│  └────────────────┘                      │
│  footer (HTML, 0 KB JS)                  │
└──────────────────────────────────────────┘
```

## Writing one

The default renderer is TypeScript. The markup is a `.tri` file, the behavior is a class:

```html
<!-- src/components/counter.tri -->
---
#[island(vanilla)]
#[props]
fn (start: i32 = 0, label: String);
---
<button data-count={start}>{ label }: { start }</button>
```

```ts
// src/components/counter.ts
export default class extends Island<{ start: number; label: string }> {
  mount(el: HTMLElement, props) {
    let n = props.start;
    el.onclick = () => (el.textContent = `${props.label}: ${++n}`);
  }
}
```

Or in Rust, with `islands-leptos` enabled — same directives, same protocol:

```rust
// src/components/counter.rs
use triblenka::island;
use leptos::prelude::*;

#[island]
#[component]
pub fn Counter(#[prop(default = 0)] start: i32, label: String) -> impl IntoView {
    let (n, set_n) = signal(start);
    view! {
        <button on:click=move |_| set_n.update(|n| *n += 1)>
            {label} ": " {n}
        </button>
    }
}
```

```html
<Counter start={3} label="Clicks" client:visible />
```

`#[island]` registers the component, generates the server-render shim and the client hydrate shim,
and derives the props serialization used to move `start` and `label` across the boundary.

**Props are one type, compiled twice.** The server serializes the struct; the browser deserializes
the same struct. A shape mismatch cannot happen — unlike hand-written JSON contracts, it is a
compile error.

Props must be `Serialize + DeserializeOwned` and owned. Passing a borrowed `&str` to an island is
a compile error suggesting `String`.

## Client directives

| Directive | Hydrates when | Use for |
|---|---|---|
| `client:load` | immediately on page load | above-the-fold controls |
| `client:idle` | `requestIdleCallback` | important but not urgent |
| `client:visible` | enters the viewport (`IntersectionObserver`) | **the default choice** |
| `client:visible={"200px"}` | within a root margin | pre-warm just before it is needed |
| `client:media={"(min-width: 60rem)"}` | media query matches | desktop-only widgets |
| `client:only` | never renders on the server | anything touching `window` at construction |

A directive is required. A component without one never ships JavaScript, and a `.tri` component
*with* one is a compile error telling you to make it an `#[island]`.

## What the browser receives

```html
<tri-island id="i0" isl="a1b2c3" on="visible"
            core="/_tri/core.a91f.wasm" mod="/_tri/isl_a1b2.wasm">
  <button data-t-9f2>Clicks: 3</button>
  <script type="application/json">{"start":3,"label":"Clicks"}</script>
</tri-island>
```

Plus one ~1.1 KB inline shim per page, which arms the trigger, fetches the chunks, and calls
`hydrate`. If the wasm fails to load, the server-rendered markup stays on the page — a broken
island degrades to static content rather than a blank box.

## Code size

Islands are compiled into **one** wasm binary per site, then split: a shared core (framework
runtime, fetched once and cached across every page) plus a lazily fetched chunk per island.

| Page | JS | Wasm |
|---|---|---|
| no islands | **0 B** | 0 B |
| one island | ~1.1 KB gz | ~45 KB gz (core, cached) + a few KB for the island |
| ten islands, one visible | ~1.1 KB gz | core + one chunk |

`tri check` enforces per-page budgets and fails CI when a page crosses them:

```toml
[budgets]
js_per_page = "5kb"
wasm_per_page = "60kb"
```

A JavaScript island (Preact, ~10 KB) is smaller than a wasm one for trivial widgets. That is a
real trade-off; see [JavaScript islands](#javascript-islands) below.

## Nesting: static content inside interactive shells

```html
<Tabs client:visible>
  <ExpensiveMarkdownTable />   <!-- server-rendered, never shipped as JS -->
</Tabs>
```

Children of an island are rendered on the server and handed to the island as an opaque fragment
that hydration does not touch. The interactive shell costs its own bytes; its content costs none.

## Server islands

Sometimes a fragment is dynamic but not interactive — a cart badge, a personalized greeting, a
live price. Those want a *server* island, not a client one:

```html
<CartBadge server:defer>
  <span class="skeleton">—</span>   <!-- fallback shown until it resolves -->
</CartBadge>
```

The page is cached and served instantly with the fallback; the badge renders on the server and
arrives separately. Props are encrypted with a per-build key, so a client cannot tamper with the
inputs of a deferred render.

Adapters that support streaming deliver server islands **out of order on the same response** — no
second round trip. Adapters that do not fall back to one fetch per island, with a build warning.

## Choosing a renderer

| Renderer | Feature flag | Good at |
|---|---|---|
| **Vanilla (default)** | `islands-vanilla` | A TypeScript class bundled by rolldown. 1–2 KB. No wasm core |
| Leptos | `islands-leptos` | Fine-grained reactivity, smallest wasm of the Rust options |
| Dioxus | `islands-dioxus` | RSX ergonomics, hot-patching in dev, code shared with native apps |

Renderers can be mixed in one site; each contributes a shared core chunk only if it is used.

**The default is TypeScript on purpose.** A theme toggle or a nav drawer is 1–2 KB of TS against
~45 KB of shared wasm core. A framework built to ship less to the browser should not default to
the heavier option out of language loyalty — and because the bundler is a Rust library, TS islands
still need no Node toolchain.

Choose a Rust island when the island shares real logic with the server (validation, parsing,
formatting — write it once), when the work is compute-bound (search, diffing, image processing,
SQLite in the browser), or when it is large enough that wasm's size curve wins — empirically past
about 4–6 KB of TypeScript. `tri build --stats` prints what each island actually costs.

## JavaScript islands

An npm widget with no Rust equivalent is still reachable:

```html
<ReactCalendar client:only="react" events={&events} />
```

`client:only` bundles the component with rolldown and mounts it in the browser. Because it never
renders on the server, no JavaScript engine is needed in your build or your deployment.

Server-rendering a JS component requires the opt-in `sidecar` feature, which runs Node or Deno
alongside the build. It is off by default and absent from the shipped binary.
