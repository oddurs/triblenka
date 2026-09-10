# Interactivity: the ladder

> Design-stage docs.

Most frameworks give you two choices: static HTML, or ship a component runtime to the browser. That
is a reasonable ladder when a server round trip costs 50 ms and a runtime costs 8 KB. Ours costs are
the other way round — a render is ~200µs, and a wasm runtime is 45 KB — so Triblenka offers four
rungs and **the compiler picks the lowest one that works.**

| Rung | What it is | Cost | Works with JS off |
|---|---|---|---|
| **0** | The platform | 0 B | yes |
| **1** | Server frame | ~2 KB JS | yes |
| **2** | Resumable handler | ~1 KB + a chunk per interaction | yes, paired with a frame |
| **3** | Island | 45 KB wasm, or ~2 KB TypeScript | no |

## Rung 0 — the platform

Before reaching for any of the rest, check whether the browser already does it. In 2026 it usually
does: `<details>` for disclosure, `popover` for menus and dialogs, `:has()` for parent-conditional
styling, anchor positioning for tooltips, scroll-driven animations, and cross-document view
transitions for page-to-page animation.

Triblenka ships no component library for these, on purpose. A tabs component that costs 4 KB to
re-implement `:target` is a tax, not a feature.

## Rung 1 — server frames

A frame is a region of the page that the **server** re-renders and the browser swaps in.

```html
---
// src/components/post-list.tri
#[frame(id = "posts")]
#[props]
fn (tag: Option<String>, page: usize = 1);

let posts = content::blog().published().by_tag_opt(&tag).paginate(12, page)?;
---

<div class="filters">
  {#for t in content::blog().tags()}
    <a href={frame!(posts(tag = t))}>{ t }</a>
  {/for}
</div>

{#for post in posts.items}
  <Card post={post} />
{/for}

<a href={frame!(posts(tag, page + 1))} rel="next">Next</a>
```

Clicking a filter fetches the re-rendered frame and swaps it — no page reload, no client state, no
component runtime. With JavaScript disabled the same links are ordinary navigation and the same
frame renders as part of a full page. **You do not write the fallback; the fallback is the primary
path with an enhancement layered on top.**

This is the rung most content sites need for most of what they call interactivity: filters,
pagination, sorting, search, forms, live prices, cart badges. It costs ~2 KB of JavaScript for the
whole page, regardless of how many frames it has.

### Why this works here and not elsewhere

The idea is not new — it is htmx's, and htmx is right. What htmx cannot assume is a server fast
enough to make the round trip invisible. A frame re-render is a query against an in-process store
and a depth-first write into a buffer: **microseconds of compute**, plus whatever the network costs.
Deployed at the edge, the swap is faster than most client-side state updates.

The honest caveat: this depends on the origin being near the reader. `tri check` warns when the
adapter puts frames a continent away from the audience the site is configured for.

## Rung 2 — resumable handlers

Sometimes the interaction is genuinely local — a disclosure that needs to remember a value, a
counter, an input mask — but nowhere near enough to justify booting a framework.

```rust
#[handler]
fn increment(count: Signal<i32>) {
    count += 1;
}
```

Rather than hydrating, the page **resumes**: the handler's captured state is serialized into the
document, one delegated listener is attached for the whole page, and the code for `increment` is
fetched only when someone actually clicks. Nothing runs before that. There is no framework boot, no
component tree reconstruction, no re-execution of what the server already did.

This is [Qwik](https://qwik.dev)'s idea, and it lands more naturally in Rust than in JavaScript: a
JavaScript compiler has to discover what a closure captured, while `rustc` already knows, as a typed
struct. The serialization is derived from the type rather than inferred from the source.

> **Status:** rung 2 is the most speculative part of the design. It is a spike with a kill criterion
> — if serializing captured state costs more than it saves, it does not ship and this page changes.

## Rung 3 — islands

For continuous local state: a canvas, a text editor, a map, a diagram tool, a data grid. Here the
component runtime earns its bytes.

```rust
#[island]
#[component]
pub fn Editor(initial: String) -> impl IntoView { /* … */ }
```

See [Islands](islands.md) for the full mechanism, the renderers, and the size budgets. After rungs 1
and 2, this should be rare — and when it is not rare, `tri build --stats` will tell you.

## The compiler picks

You do not normally choose a rung. The compiler infers it:

- no handlers, no signals → **static**
- interaction is a submit or a link → **frame**
- handlers close over serializable state → **resumable**
- continuous local state → **island**

Override when you disagree:

```html
<Filters rung:frame />        <!-- force a frame even though it looks resumable -->
<Chart rung:island />         <!-- force an island -->
<Nav rung:static />           <!-- assert no interactivity; a compile error if wrong -->
```

Inference is a deliberate bet: implicit behaviour is delightful when it is right and maddening when
it is wrong. It is paired with accountability you cannot skip — every build prints the rung and the
bytes for every interactive region:

```
$ tri build --stats
route                    rung    js      wasm    region
/blog                    frame   2.0 KB      0    PostList
/blog/hello              static  0 B         0    —
/tools/diff              island  1.1 KB  44 KB    DiffViewer
```

— and the budgets in `tri.toml` fail the build rather than warn.

## Proving the no-JavaScript path

Rungs 0, 1 and 2 all have a working fallback by construction, so it is a property the build can
*prove* rather than a claim you test by hand:

```rust
#![require(no_js_fallback)]   // in a page, or site-wide in src/site.rs
```

A rung-3 island cannot satisfy it, and the build says which component broke the contract and why.
See [Contracts](contracts.md).
