# Styling

> Design-stage docs.

## Scoped by default

A `<style>` block in a `.tri` file applies to that component only.

```html
<style>
  .card { border: 1px solid var(--line); border-radius: 8px; }
  h3 { margin: 0 0 .5rem; }
</style>

<article class="card"><h3>{ props.title }</h3><slot /></article>
```

The compiler hashes the component, tags its elements with `data-t-<hash>`, and rewrites the
selectors through [Lightning CSS](https://lightningcss.dev). Nested selectors, custom media, and
modern syntax are compiled down for your browser targets; the output is minified.

Child components are *not* styled by their parent's rules. To reach into one deliberately:

```css
.card :global(.prose a) { text-decoration-thickness: 2px; }
```

## Global styles

```html
<style is:global>
  :root { --line: color-mix(in oklab, canvastext 15%, canvas); }
</style>
```

Or import a stylesheet in the frontmatter — usually once, in your layout:

```html
---
use_style!("styles/global.css");
---
```

## Conditional classes

```html
<article class="card" class:featured={post.featured} class:draft={post.draft}>
<a href="#" class={classes!["btn", size.as_class(), active.then_some("is-active")]}>
```

## How CSS is delivered

Rendering a page visits exactly the components that page uses, so Triblenka knows precisely which
styles it needs. Per page:

- under the inline threshold (default 12 KB) → inlined into `<head>`, zero extra requests;
- over it → a fingerprinted stylesheet plus an inlined critical subset.

```toml
[css]
inline_threshold = "12kb"
targets = "defaults and not dead"
```

There is no global stylesheet accumulating every rule on the site, and no manual bookkeeping
about which page needs which file.

## Tailwind and other toolchains

```rust
Site::new().integration(tailwind::default())
```

The integration scans `.tri` and `.rs` sources for class names and generates the stylesheet during
the build. Sass is available through `integration(sass::default())` (via `grass`, pure Rust).
Both plug into the same per-page delivery logic above.
