# Directives reference

> Design-stage docs.

Directives are attributes with a namespace prefix. They are compiler instructions, never rendered.

## `rung:*` — override the inferred rung

The compiler picks a rung from the [interactivity ladder](../concepts/interactivity.md); these
override it when you disagree.

| Directive | Effect |
|---|---|
| `rung:static` | assert the component has no interactivity — a compile error if it does |
| `rung:frame` | force a server frame even where a resumable handler was inferred |
| `rung:resumable` | force a resumable handler |
| `rung:island` | force a full island |

```html
<Filters rung:frame />
<Chart rung:island />
<Nav rung:static />
```

An override that contradicts what the component actually does is a compile error, not a silent
downgrade: `rung:static` on a component with a handler fails, and so does `rung:resumable` on one
holding continuous local state.

## `#[frame]` — a server-rendered fragment

Declared in frontmatter rather than as a tag attribute, because a frame has an identity and a URL.

```html
---
#[frame(id = "posts")]
#[props]
fn (tag: Option<String>, page: usize = 1);
---
```

| Argument | Meaning |
|---|---|
| `id` | stable identity, used in the swap target and the frame URL |
| `cache` | optional cache policy; tags are derived from the provenance graph, not written by hand |

`frame!(posts(tag = t))` builds the URL for a frame with a given set of props, the same way
`route!` builds one for a page. The rendered element is a link or a form first — the enhancement
layers on top, which is what makes `require(no_js_fallback)` provable.

## `#[handler]` — a resumable event handler

```rust
#[handler]
fn increment(count: Signal<i32>) { count += 1; }
```

Becomes a `wasm-split` point. Captured state is serialized into the document; the code is fetched
on first interaction. Parameters must be `Serialize + DeserializeOwned`.

## `client:*` — hydrate an island

Rung 3 only. On a `.tri` component the compiler errors and suggests `#[island]`; on a component the
compiler inferred as a frame or resumable, it errors and points at `rung:island`.

| Directive | Trigger |
|---|---|
| `client:load` | immediately after the document parses |
| `client:idle` | `requestIdleCallback`, falling back to a timeout |
| `client:visible` | `IntersectionObserver` fires |
| `client:visible={"200px"}` | as above, with a root margin |
| `client:media={"(min-width: 60rem)"}` | `matchMedia` matches, now or later |
| `client:only` / `client:only="react"` | never server-rendered; mounted client-side |

Only valid on `#[island]` components. On a `.tri` component the compiler errors and suggests the
conversion. Without any directive, a component is server-only and ships nothing.

## `server:defer` — server island

```html
<CartBadge server:defer><span class="skeleton">—</span></CartBadge>
```

Renders a fallback immediately; the real fragment is produced on the server and streamed in (or
fetched, on adapters without streaming). Props are encrypted with a per-build key.

Requires `output = "server"` or `"hybrid"`. Using it on a prerendered route is a build error.

## `is:*` — element behavior

| Directive | Effect |
|---|---|
| `is:global` | on `<style>`: do not scope |
| `is:inline` | on `<style>`/`<script>`: emit verbatim, skip processing and bundling |
| `is:raw` | on any element: do not interpolate `{ }` in its children |

## Attribute helpers

| Syntax | Meaning |
|---|---|
| `class:name={cond}` | add `name` when `cond` |
| `class={classes![…]}` | build a class list from strings and `Option`s |
| `style:prop={value}` | set one inline style property |
| `attrs={map}` | spread a `Attrs` map of extra attributes |
| `{name}` | shorthand for `name={name}` |
| `checked` | bare boolean attribute, equivalent to `checked={true}` |

Attribute values are escaped. `None` removes the attribute entirely rather than rendering
`="None"`.

## Component-tag rules

- Capitalized tags resolve to components in scope; lowercase tags are HTML elements.
- Unknown capitalized tag → compile error listing components in scope, with a spelling suggestion.
- Unknown *lowercase* tag → warning unless it contains a hyphen (custom elements are fine).
- Void elements are checked; `<div />` is an error, `<br />` is not.
