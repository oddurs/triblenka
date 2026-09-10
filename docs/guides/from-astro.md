# Migrating from Astro

> Design-stage docs.

Triblenka is deliberately Astro-shaped. Most concepts map one to one; the differences are where
Rust changes the right answer.

## Concept map

| Astro | Triblenka | Notes |
|---|---|---|
| `.astro` component | `.tri` component | TS frontmatter → Rust frontmatter |
| `---` frontmatter | `---` frontmatter | May be `async`; returns `Result` |
| `{expr}` | `{ expr }` | Escaped by default in both |
| `{cond && <p/>}` | `{#if cond}<p/>{/if}` | Rust has no truthiness, so blocks are explicit |
| `{items.map(i => …)}` | `{#for i in items}…{/for}` | |
| `set:html={s}` | `{ Html(s) }` | Typed newtype instead of a directive |
| `<slot />`, `<slot name="x"/>` | identical | |
| `Astro.props` | `props` | A declared struct, not a bag |
| `Astro.params` | `params` | Typed |
| `Astro.request` | `cx.request()` | |
| `getStaticPaths()` | `#[static_paths]` | Returns `Vec<Path<Params, Props>>` |
| `export const prerender` | `#[prerender]` | |
| `client:load/idle/visible/media/only` | identical, rung 3 only | the compiler usually picks the rung for you |
| — | `#[frame]` + `frame!()` | no Astro equivalent; see below |
| — | `#[handler]` | no Astro equivalent; resumable, no hydration |
| — | `rung:static\|frame\|resumable\|island` | override the inferred rung |
| `server:defer` | identical | Plus out-of-order streaming |
| `astro-island` element | `tri-island` element | Same idea, same isolation |
| `defineCollection` + Zod | `#[derive(Collection)]` + serde | Types are the schema |
| `getCollection('blog')` | `content::blog()` | Returns an `Iterator` |
| `getEntry('blog', slug)` | `content::blog().by_slug(slug)` | |
| Content Layer `loader` | `impl Loader` | Same store/digest/meta model |
| `astro:content` render | `post.body.html()` | |
| Integrations | `impl Integration` | Same hook shape |
| Adapters | `impl Adapter` | Same feature-declaration model |
| `<Image />` | `<Image />` | `alt` is required at compile time |
| `astro.config.mjs` | `tri.toml` + `src/site.rs` | Data in TOML, code in Rust |
| Vite plugins | — | No JS module graph; use integrations |
| MDX | `content/**/*.md` with registered components, or `*.md.tri` | See below |

## Interactivity is a ladder, not a switch

The biggest difference, and the one most likely to change how you build. Astro gives you static HTML
or a hydrated island. Triblenka gives you four rungs — platform, server frame, resumable handler,
island — and infers which one a component needs.

In practice, most Astro islands port to **frames**, not islands: a filter, a paginated list, a
search box, a form, a cart badge. They keep working with JavaScript disabled, and they cost ~2 KB
for the page rather than a runtime per island. Port them by moving the state to the server and
letting the frame re-render, rather than translating the component's `useState` into a signal.

Keep an island for what genuinely holds continuous local state: a canvas, an editor, a map, a data
grid. See [Interactivity](../concepts/interactivity.md).

## The three other real differences

**1. Content is data; templates are code.** Astro rebuilds everything through Vite. Triblenka
splits them: `content/` never invokes `rustc` (millisecond rebuilds), `src/` does (sub-second to a
few seconds). Keep prose in `content/`. Anything with a compiled component belongs in `src/`.

**2. There is no truthiness and no implicit coercion.** `{post.image && <img/>}` becomes
`{#if let Some(img) = &post.image}`. In exchange, a missing field is a build error rather than an
empty render.

**3. JS islands are client-only by default.** A React island runs with `client:only="react"`
without any JavaScript in your build. Server-rendering it needs the opt-in Node/Deno sidecar.

## Porting a page

```astro
---
// Astro
import Layout from '../layouts/Base.astro';
import { getCollection } from 'astro:content';
const posts = (await getCollection('blog'))
  .filter(p => !p.data.draft)
  .sort((a, b) => b.data.date - a.data.date);
---
<Layout title="Blog">
  {posts.map(p => (
    <a href={`/blog/${p.id}`}>{p.data.title}</a>
  ))}
</Layout>
```

```html
---
// Triblenka
use triblenka::prelude::*;
use crate::layouts::Base;

let posts = content::blog().published().sorted_by_key(|p| Reverse(p.date));
---
<Base title="Blog">
  {#for p in posts}
    <a href={route!(blog::post(&p.slug))}>{ p.title }</a>
  {/for}
</Base>
```

Note the link: the Astro version builds a URL from a string, the Triblenka version won't compile if
the route disappears.

## What you give up

- The npm ecosystem, except through `client:only` islands.
- MDX's full expressive range in `content/` (registered components only — or move the file to
  `src/pages/**/*.md.tri`).
- Sub-second rebuilds when you edit a *template*. Content edits are faster than Astro's; template
  edits are slower. Structure your site accordingly.

## What you gain

Typed content and typed links, no `node_modules` at runtime, a single deployable binary or wasm
component, and build times that stay flat as the site grows.
