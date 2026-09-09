# Directives reference

> Design-stage docs.

Directives are attributes with a namespace prefix. They are compiler instructions, never rendered.

## `client:*` — hydrate an island

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
