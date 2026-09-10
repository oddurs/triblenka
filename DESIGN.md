# Triblenka

**An Astro-shaped web framework for Rust.**
Server-first HTML, zero JavaScript by default, islands of interactivity, typed content
collections, and a single static binary at the end of it.

- Status: design document (v0.1, 2026-09-09)
- Working name: `triblenka`, CLI `tri`, component extension `.tri`
- Target: Rust 1.98+, edition 2024

---

## 1. What this is, and what it is not

Astro won the content-site category by making one bet: **most of a page is not interactive,
so most of a page should not ship JavaScript.** Everything else in Astro — the `.astro`
component language, content collections, adapters, server islands — is machinery in service
of that bet.

Triblenka takes the same bet and re-decides every implementation question for a compiled,
statically-typed language — and then makes a second bet that Astro cannot make.

**A content deploy should not be a code deploy.** In a JavaScript framework, content is compiled
into the bundle, so fixing a typo means rebuilding the world: the eight-minute CI run that defines
life on a large content site. Because our templates compile to a binary and our content is a typed
store, the two separate cleanly. A site is a *program plus a store*, not a pile of HTML. Content
changes are data changes — a store write and a millisecond re-render — and the binary they run
against never moves. Everything the design does with incrementality, provenance, and reproducible
output exists to make that split trustworthy.

And a third bet, which follows from the first two and is what keeps this from being a port:
**the server is fast enough to be the runtime.**

Almost every architectural fashion of the last decade is a workaround for server rendering being
slow and far away. Hydration exists because the server could not send a new page fast enough. Client
routers exist because navigation was a round trip. SPA state exists because refetching was
expensive. React Server Components exist to claw server rendering back after everything had been
built around its absence. When a render is ~200µs against an in-process store and the binary
deploys to the edge as a wasm component, that calculus inverts — and most of what people call
"interactivity" stops needing a client-side framework at all. §9 turns that into a ladder.

**Goals**

1. HTML-out by default. A page with no islands ships **zero** bytes of JS.
2. A component language that a designer can read: markup first, Rust in the frontmatter.
3. Content is *typed data*, validated once at build, queried with real Rust types.
4. Islands are real components — Leptos, Dioxus, or hand-written TS — hydrated individually.
5. Build output is a directory of files, or one self-contained binary, or one `wasm32-wasip2`
   component. No `node_modules` at runtime, ever.
6. Content edits must not invoke `rustc`. This is a hard constraint, not an aspiration (§6.3).

**Non-goals**

- Being a React/Vue/Svelte SSR host. We render *our* components and Rust/wasm islands
  server-side. JS-framework islands are supported **client-only** by default; server-rendering
  them requires an opt-in Node/Deno sidecar (§9.7). We will not embed V8 in the default build.
- Being a general SPA framework. Leptos and Dioxus already own that. Triblenka *uses* them
  for islands rather than competing with them.
- npm parity. We will not have 400 UI kits. We will have the 20 things a content site needs.

**Who it is for.** Docs sites, marketing sites, blogs, changelogs, and content-heavy product
surfaces built by teams that already run Rust in production and do not want a Node toolchain
in their build, their container, or their supply chain.

---

## 2. Research summary

### 2.1 What Astro actually does (the parts worth copying)

| Mechanism | How Astro does it | Verdict for Triblenka |
|---|---|---|
| Component language | `.astro` = TS frontmatter + JSX-ish template; compiler written in Go, shipped as WASM, emits a TS module whose default export renders HTML | Copy the *shape*. Emit Rust instead of TS (§5) |
| Islands | Component wrapped in an `<astro-island>` custom element that loads its own JS and hydrates in isolation | Copy exactly, including the custom-element boundary (§9.3) |
| Client directives | `client:load`, `client:idle`, `client:visible` (IntersectionObserver), `client:media`, `client:only` | Copy verbatim — the vocabulary is good and already learned |
| Server islands | `server:defer` renders a placeholder, fetches the real fragment from a dedicated route, props encrypted | Copy, and upgrade to true out-of-order streaming (§9.6) |
| Content | Content Layer API (v5+): *sources → loaders → cached data store → output*. Loaders get `store`, `meta`, `parseData()`, `generateDigest()`, `watcher`; entries update only when the digest changes | Copy the architecture; replace Zod with Rust types (§8) |
| Adapters | An integration that supplies a `serverEntrypoint` speaking `Request`/`Response`, plus a declared feature matrix and build-output rewriting | Copy, expressed as a Rust trait over `http::Request`/`Response` (§13) |
| Dev | Vite; from Astro 6 on the Environment API | Replace entirely — we have no JS module graph to serve (§12) |

Astro's own compiler README names the pain we should avoid inheriting: `.astro` output
depends on `astro/runtime/server/index.js`, so "other runtimes currently need to bring their
own rendering implementation." Our rendering runtime is a Rust crate that compiles anywhere
Rust compiles — including `wasm32-wasip2` and `wasm32-unknown-unknown`.

### 2.2 State of the Rust ecosystem (what we can stand on)

The pleasant surprise of this research: nearly every hard subsystem already exists as a
maintained Rust crate. We are assembling, not inventing.

| Need | Crate | Notes |
|---|---|---|
| Markdown | `comrak` | CommonMark + GFM, fast, extensible; `markdown-rs` as AST alternative |
| MDX-ish | `mdxjs` / `markdown-rs` | `mdxjs-rs` compiles MDX→JS via SWC; we want MDX→*Rust*, so we reuse the parser only (§8.4) |
| CSS | `lightningcss` | Parse, transform, minify, and — crucially — a visitor API for scoping selectors |
| JS/TS transform | `oxc_parser`, `oxc_transformer` | The parser under Rolldown; TS/JSX strip, target lowering |
| JS bundling | `rolldown` (1.2.x on crates.io, Rust-native) | Real bundler as a library. Only needed for TS islands and the tiny runtime shims |
| Incrementality | `salsa` | The rust-analyzer query engine, with durability tiers. Our build graph (§11) |
| Content store | `redb` | Pure-Rust embedded ACID KV. Digest-keyed entries, no C dependency |
| Templating precedent | `askama`, `maud` | Not used directly, but their compile-time codegen approach is the model |
| Islands (Rust) | `leptos` (islands mode, fine-grained, smallest wasm), `dioxus` (wasm-split lazy chunks, hot-patching) | First-party renderers for both (§9.5) |
| Wasm plumbing | `wasm-bindgen`, `walrus`, `wasm-opt`, `wasm-split` | Split one island binary into a shared core + lazily fetched per-island chunks |
| Images | `image`, `ravif`, `oxipng`, `resvg` | AVIF/WebP encode, resize, SVG raster. Replaces Sharp |
| Highlighting | `syntect` + `two-face`, or `inkjet` (tree-sitter) | Build-time only; zero client cost |
| HTTP | `hyper`, `axum`, `tower` | Middleware = `tower::Layer`. Adapters get this for free (§7.3) |
| Deploy targets | `workers-rs` (Cloudflare), `lambda_http` (AWS), Vercel's Rust runtime, `wasi:http` | All reachable from one `Adapter` trait |
| Diagnostics | `miette` + `annotate-snippets` | Source-span errors that point into `.tri`, not into generated code |

### 2.3 The two decisions that define the project

Everything below follows from these.

**Decision A — one semantics, compiled; two execution modes.**
*(Amended after design review — see Appendix A.)* The original framing was a false binary between
"compile everything" (slow template edits) and "interpret in dev" (a second, weaker language and
guaranteed semantic drift). Both poles are wrong, and Dioxus has already demonstrated the third
option in production: **expressions are always compiled Rust; structure is data.**

The compiler emits, alongside the generated render function, a *template descriptor* — a tree of
static chunks, attributes, slots, and expression slots keyed by index into a table of compiled
thunks. In release the descriptor is monomorphized into straight-line `sink` calls. In dev the
renderer walks it, and the dev server can swap in a new descriptor at runtime whenever an edit
does not change the *set of expressions*: markup, attributes, classes, nesting, and ordering all
reload in milliseconds with no `rustc`. Editing an expression or the frontmatter falls back to an
incremental rebuild. One language, one semantics, no drift, and the common template edit costs
roughly what it costs in Astro (§5.1).

**Decision B — content is data, pages are code, and the boundary is a directory.**
`content/**` is never compiled. It is loaded into a digest-keyed store and rendered by already-
compiled templates. `src/**` is compiled. This single rule is what makes a Rust content framework
viable: a writer changing a blog post gets a ~5 ms rebuild, because nothing they touched is code.

---

## 3. Project layout

```
my-site/
├── Cargo.toml
├── tri.toml               # non-code config: base url, output mode, image defaults
├── src/
│   ├── site.rs            # site definition: collections, integrations, adapter
│   ├── pages/             # file-based routing
│   │   ├── index.tri
│   │   ├── about.tri
│   │   ├── blog/
│   │   │   ├── index.tri
│   │   │   └── [slug].tri
│   │   ├── rss.xml.rs     # endpoint: fn get(ctx) -> Response
│   │   └── api/
│   │       └── search.rs
│   ├── layouts/
│   │   └── base.tri
│   ├── components/
│   │   ├── card.tri
│   │   └── counter.rs     # #[island] — a Leptos/Dioxus component
│   ├── middleware.rs      # tower layers
│   └── content.rs         # collection definitions (types + loaders)
├── content/               # DATA. never compiled.
│   ├── blog/*.md
│   └── authors/*.yaml
├── public/                # copied verbatim
└── styles/
    └── global.css
```

`Cargo.toml` declares one dependency and one build script:

```toml
[dependencies]
triblenka = { version = "0.1", features = ["islands-vanilla"] }

[build-dependencies]
triblenka-build = "0.1"     # runs the .tri compiler, emits Rust into OUT_DIR
```

---

## 4. The `.tri` component language

### 4.1 Anatomy

```html
---
// Rust frontmatter. Runs once per render, on the server.
use crate::components::{Card, Prose};
use triblenka::prelude::*;

#[props]
fn (title: String, subtitle: Option<String> = None);

let posts = content::blog()
    .published()
    .sorted_by_key(|p| Reverse(p.date))
    .take(5);
---

<style>
  h1 { font-size: var(--step-4); }
  .grid { display: grid; gap: 1rem; }
</style>

<section class="grid">
  <h1>{ props.title }</h1>
  {#if let Some(sub) = &props.subtitle}
    <p class="sub">{ sub }</p>
  {/if}

  {#for post in posts}
    <Card title={&post.title} href={route!(blog::post(&post.slug))}>
      <Prose html={post.excerpt()} />
    </Card>
  {/for}

  <Counter start={0} client:visible />
</section>
```

### 4.2 Rules

- **Frontmatter** is a Rust block. Its `use` statements, `let` bindings, and items are lifted
  into the generated module. It may be `async`; the codegen infers this and makes the component's
  render fn async.
- **`{ expr }`** interpolates. The value must implement `Render`. Strings are HTML-escaped.
  Raw HTML requires the `Html` newtype: `{ Html(post.body) }` — explicit, greppable, and typed,
  rather than a magic `set:html` directive.
- **Block forms** are the only non-Rust syntax: `{#if}` / `{:else if}` / `{:else}` / `{/if}`,
  `{#for x in xs}`, `{#match e}{:case Pat}`, `{#let x = e}`. Conditions and patterns are real
  Rust, parsed by `syn` and pasted through.
  *Why not Leptos-style `view!` with nested Rust blocks?* Because the primary reader of a page
  template is often not the person who wrote the Rust. Markup-first with a fixed, tiny set of
  block forms keeps templates skimmable; the moment logic gets interesting it belongs in the
  frontmatter or a helper, not in the markup.
- **Components** are capitalized tags. Attributes map to the `Props` struct by name; a bare
  `{name}` shorthand fills the field of the same name. Missing required props are a compile
  error, with the span pointing at the tag.
- **Slots.** `<slot />` for children, `<slot name="footer" />` for named; callers pass
  `<Card><p slot="footer">…</p></Card>`. Slots are `impl Fn(&mut Sink) -> Result<()>` under the
  hood — no allocation, no intermediate string.
- **`<style>`** is scoped by default: the compiler hashes the component, tags matching elements
  with `data-t-a1b2c3`, and rewrites selectors through the `lightningcss` visitor.
  `<style is:global>` opts out. Styles are hoisted, deduped, and emitted per-page (§10.1).
- **`<script>`** is *not* a hydration mechanism. A bare `<script>` is bundled as a page-level
  module by `oxc` + `rolldown`; interactivity that needs state uses an island.
- **Directives** on component tags: `client:load|idle|visible|media|only`, `server:defer`,
  plus `is:global` on styles and `is:raw` on any element to suppress interpolation.

### 4.3 Typed routes

`route!(blog::post(&slug))` is generated from `src/pages/**` at build time. It is a function
call, so a renamed or deleted page is a **compile error at the link site**, not a 404 found by a
crawler three weeks later. This is the single most-liked feature of typed web frameworks and it
falls out of file-based routing for free.

---

## 5. Compilation model

```
src/**/*.tri ──► tri-compiler ──► $OUT_DIR/tri_generated/*.rs ──► rustc ──► site lib
                     │                        │
                     │                        └── span map: (gen file, line, col) → (.tri, line, col)
                     └── parse (html5-ish tolerant parser + syn for embedded Rust)
```

`build.rs` calls `triblenka_build::compile()`, which walks `src/`, parses every `.tri`, and emits
one Rust module per component:

```rust
// generated from src/components/card.tri
pub struct Card;
#[derive(triblenka::Props)]
pub struct Props<'a> { pub title: &'a str, pub href: Route }

impl triblenka::Component for Card {
    type Props<'a> = Props<'a>;
    const SCOPE: ScopeId = ScopeId(0xa1b2c3);
    const STYLE: &'static str = "[data-t-a1b2c3] h1{font-size:var(--step-4)}";

    async fn render(props: Props<'_>, slots: Slots<'_>, sink: &mut Sink<'_>) -> Result<()> {
        sink.raw("<article data-t-a1b2c3 class=\"card\"><h2>")?;
        sink.escaped(props.title)?;          // ← span-mapped back to card.tri:9:12
        sink.raw("</h2>")?;
        slots.default(sink).await?;
        sink.raw("</article>")?;
        Ok(())
    }
}
```

Notes on the shape:

- Static markup becomes `sink.raw(&'static str)` calls over pre-concatenated literals. The
  fast path is a `memcpy` per run of static HTML.
- There is **no virtual DOM and no intermediate tree** on the server. Rendering is a
  depth-first write into a `Sink`. This is the same trick `maud` and `askama` use, and it is why
  a Rust renderer can beat a JS one by an order of magnitude rather than a factor.
- `async fn render` is only generated when the component (or any descendant) needs it. Sync
  components stay sync and cost nothing.

### 5.1 Two render modes, one source of truth

```
.tri ──► generated Rust ──┬── release: straight-line sink.raw(…) / sink.escaped(expr)
                          └── dev:     descriptor + compiled expression thunks
```

```rust
static TEMPLATE: Template = Template(&[
    Node::Static("<article class=\"card\" data-t-a1b2c3><h2>"),
    Node::Expr(0),                       // → thunk table
    Node::Static("</h2>"),
    Node::Slot(SlotId::Default),
    Node::Static("</article>"),
]);

static THUNKS: &[Thunk<Props>] = &[
    |p, sink| sink.escaped(p.title),     // compiled, type-checked, span-mapped
];
```

The thunk table is ordinary compiled code, so expressions keep full Rust semantics and full type
checking in *both* modes — the descriptor never interprets Rust, it only orders it. Dev overhead
is one `match` per node, invisible next to I/O.

The dev server watches `.tri`, re-parses on change, and diffs the expression set. Unchanged set →
ship a descriptor over the HMR socket and re-render in place (5–30 ms). Changed set → codegen and
an incremental `rustc` build. This is exactly the mechanism and exactly the limitation of Dioxus's
RSX hot reload: structure and styling are instant, logic needs a rebuild.

A useful side effect: because the descriptor is data, `tri build --emit-templates` can export it,
which is how a future CMS-editable-template mode or a template linter would work without a second
parser.

### 5.2 Diagnostics

The known failure mode of file-based template compilers (Askama's chief complaint) is that
`rustc` errors point at generated code. Two mitigations, both required:

1. **Span map.** The compiler records, for every emitted byte range, the originating `.tri`
   span. `tri build` runs cargo with `--message-format=json-diagnostic-rendered-ansi`, remaps
   every span through the table, and re-renders the diagnostic with `miette` against the `.tri`
   source. Generated files are never shown to the user.
2. **Pre-flight checks.** Unknown component, missing required prop, unclosed block, unknown
   directive, and slot-name typos are caught in the compiler itself with exact spans, before
   `rustc` ever sees the file. These are ~80% of real-world template errors.

An escape hatch exists for people who want perfect spans and inline components:
`triblenka::component! { ... }` inside a `.rs` file, same grammar, proc-macro spans.

---

## 6. Rendering runtime

### 6.1 The sink

```rust
pub trait Sink {
    fn raw(&mut self, s: &str) -> Result<()>;
    fn escaped(&mut self, v: impl Render) -> Result<()>;
    /// Reserve an out-of-order slot; resolved later by a streaming island.
    fn defer(&mut self, id: IslandId) -> Result<()>;
}
```

Three implementations: `StringSink` (static builds, `String` with a size hint from the last
build — the hint is cached, so re-renders allocate exactly once), `StreamSink` (writes
`Bytes` chunks into an `http_body` stream), and `HashSink` (renders to a content hash only,
used by the build graph to decide whether an output actually changed).

### 6.2 Page lifecycle

```
request ──► router ──► middleware (tower) ──► page component
                                                   │
                        ┌──────────────────────────┼─────────────────────────┐
                        ▼                          ▼                         ▼
                  frontmatter                  render tree             island collection
                  (data access)            (write into Sink)      (props → hydration script)
                        │                          │                         │
                        └──────────────► response head + streamed body ◄─────┘
```

For prerendered routes the same path runs at build time with a synthetic request, and the
resulting bytes are written to `dist/`. **Static and server rendering share one code path** —
there is no second implementation to drift.

### 6.3 The compile boundary (Decision B, in detail)

| You changed | What reruns | Typical latency |
|---|---|---|
| `content/**` (markdown, yaml, json) | loader → store → re-render affected pages | **3–20 ms** |
| a `.tri` template — markup, attributes, nesting, order | descriptor swap over HMR, no `rustc` | **5–30 ms** |
| a `.tri` template — new or changed expression | codegen → `rustc` incremental → re-render | 0.4–2 s (cranelift dev profile) |
| a `.rs` non-island file | `rustc` incremental → re-render | 0.5–3 s |
| an `#[island]` component | wasm rebuild + `wasm-bindgen` + split | 3–15 s (only that island's chunk changes) |
| `tri.toml` | full reload of config, no recompile | ~50 ms |

The top row is the one that matters. It is why `content/` cannot be a Rust module, why
collections are deserialized at runtime rather than `include_str!`'d, and why we accept the
loss of `const`-folding content into the binary.

---

## 7. Routing, endpoints, middleware

### 7.1 Files to routes

`src/pages/blog/[slug].tri` → `/blog/:slug`. `[...path].tri` → catch-all.
Matching precedence is a total order compared **segment by segment**, not whole-route. Verified
against Astro's comparator (`core/routing/priority.ts`) and adopted with two deliberate
differences:

1. more segments beat fewer — an `index` page counts as one segment more than the URL it serves;
2. within a segment: fully static beats partially dynamic (`game-[title]`) beats fully dynamic
   (`[title]`) beats rest (`[...path]`);
3. `/foo` is more specific than `/foo/[...rest]` when the lengths differ by exactly one;
4. prerendered beats on-demand; endpoints beat pages;
5. ties break **byte-wise, not by locale**.

Rule 5 is our first divergence: Astro tiebreaks with `localeCompare`, which is locale-dependent, so
the same source can in principle order two routes differently on two machines. For a project whose
incremental check byte-compares build outputs, that is not acceptable.

The second divergence: only *identical* route patterns are a build error. The first draft made any
two routes matching one URL an error, which would reject `game-[title]` alongside `[title]` — a
legitimate pair. Ambiguity is resolved by the order above and explained by `tri explain`, not
forbidden.

See Appendix B.5 for the comparator in full.

Dynamic prerendered pages declare their params in the frontmatter:

```rust
#[static_paths]
async fn paths() -> Vec<Path<Params, Props>> {
    content::blog().published()
        .map(|p| Path::new(Params { slug: p.slug.clone() }).with_props(p))
        .collect()
}
```

Per-route rendering mode: `#[prerender]` / `#[prerender(false)]` in frontmatter; the site-level
default comes from `tri.toml` (`output = "static" | "server" | "hybrid"`).

### 7.2 Endpoints

`src/pages/rss.xml.rs`:

```rust
pub async fn get(ctx: Context) -> Result<Response> {
    let feed = rss::channel(content::blog().take(20));
    Ok(Response::xml(feed))
}
```

Exported `get`/`post`/`put`/`delete`/`all` become methods. Prerendered endpoints run at build
time and write a file.

### 7.3 Middleware is `tower`

```rust
pub fn middleware() -> impl Layer<Router> {
    ServiceBuilder::new()
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::overriding(CSP, csp_for_islands()))
        .layer(auth::require_session().only(path("/admin/*")))
}
```

This is a large, unglamorous win. Every `tower-http` layer — compression, CORS, tracing, rate
limiting, timeouts — works on day one, and every adapter target already speaks `tower`.

---

## 8. Content layer

### 8.1 Collections are types

```rust
// src/content.rs
#[derive(Collection, Deserialize)]
#[collection(name = "blog", loader = Glob::new("content/blog/**/*.md"))]
pub struct Post {
    pub title: String,
    pub date: Date,
    #[serde(default)]
    pub draft: bool,
    pub author: Ref<Author>,          // referential integrity, checked at load
    pub tags: Vec<String>,
    #[content] pub body: Markdown,    // the document body
}
```

The derive generates: a serde impl, a JSON-Schema for editors and CMS round-tripping, a
digest function over the parsed value, and a typed query surface `content::blog()` returning
`Query<Post>` (an iterator with `.published()`, `.by_tag()`, `.sorted_by_key()`, `.page(n)`).

Frontmatter that fails to deserialize is a **build error with a file/line span into the
markdown file**, listing the offending field and the expected type. A missing `Ref<Author>`
target is an error, not `undefined`.

To be precise about a claim that is easy to overstate: the *type* is compile-time, the
*conformance check* is build-time. Because content is loaded rather than compiled (Decision B),
a malformed document is caught when the loader runs, exactly as Zod catches it in Astro. What Rust
adds is that the type is the single source of truth — there is no schema to drift from the
interface — and that every downstream use of `post.date` is checked by `rustc`.

### 8.2 The loader trait

Deliberately isomorphic to Astro's Content Layer, because that design is correct: sources →
loaders → digest-keyed store → output.

```rust
#[async_trait]
pub trait Loader: Send + Sync {
    fn name(&self) -> &str;

    /// Populate or refresh the store. Called at build, and on watcher events in dev.
    async fn load(&self, cx: &LoaderContext) -> Result<()>;

    /// Optional: files/globs to watch in dev.
    fn watch(&self) -> Vec<Pattern> { vec![] }
}

pub struct LoaderContext {
    pub store: Store,          // scoped to this collection
    pub meta: MetaStore,       // sync tokens, cursors, etl checkpoints
    pub logger: Logger,
    pub config: Arc<Config>,
    pub refresh: Option<RefreshData>,   // webhook payload that triggered this run
}

impl Store {
    pub fn set<T: Serialize>(&self, id: &str, value: &T, digest: Digest) -> Result<Changed>;
    pub fn get<T: DeserializeOwned>(&self, id: &str) -> Result<Option<T>>;
    pub fn delete(&self, id: &str) -> Result<()>;
    pub fn keys(&self) -> impl Iterator<Item = String>;
}
```

`set` returns `Changed::{Yes, No}` by comparing digests — unchanged entries do not dirty the
build graph, so a loader that refetches an entire CMS still only rebuilds the pages whose
content actually moved.

First-party loaders: `Glob` (markdown/mdx/yaml/json/toml), `File` (one file → many entries),
`Http` (any JSON API, with ETag/`If-Modified-Since` in `meta`), `Sql` (via `sqlx`), and
`Git` (derives `lastModified`/authors from history — the sort of thing that is miserable in
Node and trivial with `gix`).

### 8.3 The store

`redb` at `.tri/store.redb`. Tables: `entries(collection, id) → (digest, msgpack blob)`,
`meta(collection, key) → bytes`, `assets(hash) → path`. Chosen over SQLite because it is pure
Rust (no `cc` in the dependency tree, and it cross-compiles to wasm targets for
edge-side live collections), and over plain JSON because incremental writes must not rewrite
the world. Queries are served from an in-memory index built at startup — for the sizes that
matter (10⁴–10⁵ entries) this is microseconds and simpler than pushing predicates into storage.

The store sits behind a `Store` trait. When live collections and full-text search justify real SQL,
SQLite or [Turso](https://turso.tech) (the Rust SQLite rewrite — beta as of 2026, not yet at
SQLite-level reliability) can replace it without touching a single loader. The honest cost of a
binary KV file is that you cannot inspect it with the tools you already have, so `tri store dump`
prints entries as JSON.

### 8.4 Markdown and `.mdx`-alikes

Pipeline: `comrak` → AST → transform passes → `Html`. Build-time passes: syntax highlighting
(`syntect`), heading slugs + `TableOfContents` extraction, link rewriting and **dead internal
link detection** (a build error under `--strict`), image references resolved into the asset
pipeline (so markdown images get the same AVIF/WebP treatment as `<Image>`), and smart typography.

For markdown with components — the MDX use case — we split the difference honestly:

- `content/**/*.md` may embed components **from a registered whitelist**:
  `mdx.component::<Callout>("Callout")`. The markdown stays *data*; component tags are resolved
  at render time through a `&dyn ComponentRegistry` and dispatched dynamically. Fast rebuilds,
  no `rustc`, small runtime dispatch cost.
- `src/pages/**/*.md.tri` is markdown *as code*: any component in scope, full frontmatter,
  compiled. Slower to change, unrestricted.

That is a real trade and users should be able to see it in the file extension.

---

## 9. Interactivity

Astro offers two rungs: static HTML, or a hydrated island. That is the right shape for a JavaScript
framework, where a server round trip costs tens of milliseconds and a component runtime is a few
kilobytes. Our costs are the other way round — a render is microseconds, and a wasm runtime is 45 KB
— so copying that ladder would inherit its ergonomics *and* its worst constant factor.

**The ladder has four rungs, and the compiler picks the lowest one that works.**

| Rung | Mechanism | Cost | Works without JS |
|---|---|---|---|
| 0 | The platform — `<details>`, `popover`, `:has()`, anchor positioning, view transitions | 0 B | yes |
| 1 | **Server frame** — a fragment re-rendered on the server and swapped in | ~2 KB JS | yes, via form/link |
| 2 | **Resumable handler** — one closure, its captured state serialized, its code fetched on demand | ~1 KB + a chunk per interaction | yes, when paired with a frame |
| 3 | **Island** — continuous local state: canvas, editor, map | 45 KB wasm core, or ~2 KB TS | no |

Rung 1 is the one most content sites need and the one no compiled framework has taken seriously.
Filters, pagination, sort, search, forms, live prices: htmx was right that these belong on the
server, and what htmx lacks is a server fast enough to make the round trip invisible. At 200µs a
fragment render is cheaper than most client-side state updates.

Rung 2 is Qwik's idea — do not hydrate, *resume* — and it is more natural in Rust than in
JavaScript. Qwik needs a compiler to discover what a closure captured; `rustc` already knows, as a
typed struct. A `#[handler]` becomes a `wasm-split` point, its captured state is serialized into the
document with serde, and an interaction fetches one small chunk instead of booting a runtime. This
is the direct answer to our worst structural weakness (§15).

Rung 3 is what the rest of this section describes. It is where wasm's size curve actually wins, and
after rungs 1 and 2 it should be rare.

**The compiler infers the rung.** A component with no handlers and no signals is static; one whose
only interaction is a submit is a frame; one whose handlers close over serializable state is
resumable; one with continuous local state is an island. Marko has done automatic partial hydration
for years while everyone else asks you to type `client:visible` — a directive is a workaround for a
compiler that cannot infer, and ours can. Directives remain, as an override when you disagree.

The risk of inference is that payload becomes surprising, so it is paired with accountability that
cannot be skipped: `tri build --stats` prints the rung and the bytes for every interactive region,
and the budgets in §9.2 fail the build rather than warn.

### 9.1 Declaring an island

```rust
// src/components/counter.rs
use triblenka::island;

#[island]
#[component]                       // leptos::component
pub fn Counter(#[prop(default = 0)] start: i32) -> impl IntoView {
    let (n, set_n) = signal(start);
    view! { <button on:click=move |_| set_n.update(|n| *n += 1)>{n}</button> }
}
```

`#[island]` does four things: registers the component in a distributed slice (`linkme`) keyed by
a stable id (crate path + name, hashed); derives `Serialize`/`Deserialize` glue for its props;
emits a server-side render shim used for the initial HTML; and emits a wasm-side hydrate shim.

### 9.2 The build: one binary, split into chunks

The naive approach — one wasm module per island — duplicates the framework runtime in every
module and is a non-starter (each copy is 40–100 KB gzipped). Instead:

```
islands crate (all #[island] fns + registry)
   └─ cargo build --target wasm32-unknown-unknown --profile island
        └─ wasm-bindgen
             └─ wasm-split  ──► core.wasm     (runtime + reactivity, shared)
                            ──► isl_a1b2.wasm (counter)
                            ──► isl_c3d4.wasm (search box)
                  └─ wasm-opt -Oz
```

The registry's dispatch fn is the split point set: each island entry is marked as a lazy
boundary, so a page with one `client:visible` island fetches `core.wasm` + that island's chunk
and nothing else. Leptos islands mode keeps the shared core small (fine-grained reactivity, no
VDOM); Dioxus provides the same via `wasm-split`.

**Budget we hold ourselves to:** a page with a single trivial island ships ≤ 1.2 KB of JS
(the loader shim) and ≤ 45 KB gzipped wasm. A page with zero islands ships **0 bytes**.

### 9.3 The hydration protocol

Server output for `<Counter start={3} client:visible />`:

```html
<tri-island id="i0" isl="a1b2c3" on="visible" core="/_tri/core.a91f.wasm" mod="/_tri/isl_a1b2.wasm">
  <button data-t-9f2>3</button>
  <script type="application/json">{"start":3}</script>
</tri-island>
```

The loader shim (inlined once per page, ~1.1 KB min+gz) is a custom element that:

1. reads `on` and arms the corresponding trigger — `load` (immediate), `idle`
   (`requestIdleCallback`), `visible` (`IntersectionObserver`), `media` (`matchMedia`);
2. on trigger, `import()`s the wasm-bindgen JS glue for `core`, then the island chunk;
3. calls `hydrate(islandId, propsJson, hostElement)`, which deserializes props with serde and
   attaches reactive listeners to the *existing* DOM (no re-render);
4. removes itself from the DOM observer set. Errors are reported once, to `console.error`, and
   leave the static HTML intact — a failed island degrades to the server-rendered markup.

`client:only="leptos"` skips server rendering entirely and renders into an empty host — the
escape hatch for anything that touches `window` at construction.

**Props are typed on both ends.** The server serializes `Props` with serde; the client
deserializes the same struct. A mismatch is impossible: it is one type, compiled twice.
Astro's equivalent contract is `JSON.stringify` and hope.

### 9.4 Nesting and slots

An island may contain server-rendered children (`<Counter client:load><ExpensiveChart /></Counter>`).
Children are rendered on the server, and the client shim passes the existing subtree to the island
as an opaque `Fragment` that hydration must not touch. This preserves the "static content inside an
interactive shell ships no JS" property, which is the whole point of islands and which most
frameworks quietly lose at the second level of nesting.

### 9.5 Renderers

```rust
pub trait IslandRenderer {
    const NAME: &'static str;
    type Component<P>;
    fn render_to_sink<P: Props>(c: &Self::Component<P>, p: &P, s: &mut Sink) -> Result<()>;
    fn client_entry() -> ClientEntry;      // chunk + hydrate symbol
}
```

Shipping in v1: **`vanilla` (the default)** — a TypeScript class bundled in-process by oxc +
rolldown, server-rendered from a `.tri` template and hydrated in `connectedCallback` — plus
`leptos` and `dioxus`. A site may mix renderers; each contributes at most one shared core chunk,
and only if used.

*(The default was reversed in design review; see Appendix A.)* The reasoning, stated plainly: a
theme toggle, a nav drawer, or a copy button is 1–2 KB of TypeScript against ~45 KB of shared wasm
core. A framework whose entire thesis is "ship less to the browser" cannot default to the heavier
option out of language loyalty. Because the bundler is a Rust library, TypeScript islands still
need no Node toolchain — the "no npm in CI" property is untouched.

Reach for a Rust island when one of three things is true: the island shares non-trivial logic with
the server (validation, parsing, formatting — write it once); the work is compute-bound (search,
diffing, image or audio processing, SQLite in the browser); or the island is big enough that
wasm's size curve wins, empirically past roughly 4–6 KB of TypeScript. `tri build --stats` prints
the payload of every island, so this is a measurement, not a preference.

### 9.6 Server islands, streaming, out of order

`<Cart server:defer />` emits a placeholder and a fetch:

```html
<tri-server id="s1" src="/_tri/island/cart" k="AES-GCM(props)"><!--fallback slot--></tri-server>
```

Props are encrypted with AES-GCM so a deferred island's inputs cannot be tampered with from the
client — same threat model as Astro, and the same algorithm choice for the same reason (GCM
authenticates, so it detects manipulation rather than merely hiding the plaintext).

*(Corrected from "per-build key" after reading `core/encryption.ts`.)* A per-build key is wrong
twice over: it breaks horizontally scaled deployments, where instance B cannot decrypt what
instance A encrypted, and it invalidates the incremental build cache on every build. Astro's answer
is a key that can be supplied out of band — `ASTRO_KEY`, a base64 raw AES key — plus a one-way hash
of it (`hashCryptoKey`) recorded in the build cache so a *changed* key invalidates exactly the
ciphertext that depends on it. We copy this: `TRI_KEY`, generated by `tri key gen`, required by
`tri check` whenever a route uses `server:defer`, and hashed into the memo table's input digests.
Note that the component export is encrypted alongside the props, not just the props.

Where Astro fetches each server island with a separate request, we *also* support **single-response
out-of-order streaming**: the page body streams immediately with placeholders; each island, as it
resolves, is appended to the same response as a `<template data-tri-slot="s1">` plus a one-line
inline script that moves it into place. One connection, no waterfall, no extra round trip.

*(Demoted to opt-in in design review; the stated reason was then corrected by reading Astro's
source — see Appendix B.4.)* The original claim was that streaming forces per-response nonces and
therefore an uncacheable page. That is wrong. Astro's CSP support is **hash-based, not
nonce-based**: it digests each inline script's content and accumulates the hashes into the policy
(`runtime/server/render/csp.ts`), and its server-island implementation computes that digest in
`init()` — during the head phase, *before* the body streams
(`runtime/server/render/server-islands.ts`).

The real constraint is narrower and more useful: **every inline script must be content-stable and
known at head-flush time.** A fixed mover shim with the island id substituted satisfies that; a
per-island generated script does not. So the rule for us is a design rule, not a trade-off — the
streaming shim is one constant string, hashed once at build, and the page stays fully cacheable.

Streaming stays opt-in for a different and better reason: it complicates the response path for a
benefit only pages with several slow islands can measure, and the fetch strategy is the one that
degrades cleanly on adapters without streaming.

### 9.7 JavaScript-framework islands

Supported, with an honest boundary:

- **`client:only`** — a React/Svelte/Vue component is bundled by `rolldown` and mounted into an
  empty host. No server rendering, therefore **no JS engine in our build**. This covers the large
  majority of real "I need this one npm widget" cases.
- **SSR of JS components** — opt-in `sidecar` feature. `tri` spawns a Node or Deno process; the
  render pipeline calls it over a Unix socket for those components only. Off by default, absent
  from the shipped binary, and clearly marked as the slow path.

Embedding V8 or QuickJS in the default build was considered and rejected: it doubles binary size,
breaks `wasm32` targets, and re-imports the ecosystem we are trying to leave.

---

## 10. Asset pipeline

### 10.1 CSS

Component `<style>` blocks are scoped (§4.2), collected per route by the renderer (the set of
components a page actually touched is known exactly, because rendering visits them), deduped by
scope hash, then run through `lightningcss` for nesting, custom media, autoprefixing, and minify.

Three specifics taken from Astro's compiler rather than invented (Appendix B.6). The scope id is a
short hash rendered in base32 and truncated — short enough to sit on every element without bloating
the HTML. Scoping wraps the added selector in **`:where(...)`** so it contributes zero specificity;
a scheme that silently raises specificity breaks user overrides in ways that are miserable to
debug. And a fixed set of elements is **never** scoped: `head`, `title`, `base`, `link`, `meta`,
`script`, `style`, `noscript`, `slot`, the frame elements, and `Fragment` — plus the `:root`
selector.

Emission policy, per page: if the page's total CSS is under a threshold (default 12 KB) it is
inlined into `<head>`; otherwise it is emitted as a fingerprinted file with a `<link>` and an
inlined critical subset. No FOUC, one fewer round trip, and no "which CSS file does this page
need" bookkeeping for the user.

### 10.2 JavaScript

The only JS we ever ship: the island loader shim (~1.1 KB), the optional view-transition client
router (~2 KB), wasm-bindgen glue for used renderers, and whatever the user puts in `<script>` or
a `vanilla` island. All of it goes through `oxc` (TS strip, target lowering) and `rolldown`
(bundle, treeshake, chunk) as **library calls in-process** — no subprocess, no `node`.

### 10.3 Images

`<Image src={...} width={800} alt="…" />` at build time: decode (`image`), resize (Lanczos3),
encode to AVIF (`ravif`) + WebP + fallback, emit `<picture>` with a `srcset` and intrinsic
dimensions to reserve layout. LQIP blur placeholder is a 20-byte base64 data URL computed from a
4×4 DCT. In server mode, an `/_tri/image` endpoint does the same work on demand with a signed
parameter set and an on-disk cache. SVGs pass through `resvg`-based sanitization.

### 10.4 Fonts

`font(Inter, [400, 700], subset = latin)` in config downloads, subsets (`fonttools`-equivalent via
`allsorts`/`skrifa`), self-hosts, fingerprints, and emits `@font-face` + preload hints. Third-party
font CDNs are a render-blocking cross-origin dependency and we should make the good path the easy one.

---

## 11. Build system

The build is a digest-keyed memo table — not a phase pipeline, and *(after design review)* not
`salsa` yet.

```
  file_text(path)            ─┐
  parsed_template(path)      ─┤ derived queries, memoized on input digests
  component_deps(id)         ─┤
  route_table()              ─┤
  collection_entries(name)   ─┤
  rendered_page(route, params)
  page_assets(route)
  emitted_file(path)
```

Every query records the input digests it read; a query re-runs when one of them moves. Inputs are
tiered by volatility — `content/**` changes constantly, `src/**` occasionally, config and vendored
crates almost never — so a content edit invalidates a handful of `rendered_page` nodes and nothing
above them. The table is persisted to `.tri/cache/graph.bin`.

**Why not `salsa`?** It is the right shape, and the dev server is precisely the long-lived process
it assumes. But it is an invasive, macro-heavy dependency to adopt *before* we know our graph is
hard, and ours is eight node types with a shape we already understand. We start hand-rolled, keep
every query boundary salsa-shaped so adoption stays a drop-in, and revisit at M4.

**The risk this creates** is the classic one: hand-rolled incremental builds are where stale-output
bugs live, and a stale output is worse than a slow build because nobody notices. The mitigation is
mandatory and cheap: `tri build --verify-incremental` runs the incremental build, then a clean
build, and byte-compares every output. It runs on every CI build of the framework itself, and
ships to users for their own pipelines.

Page rendering is `rayon`-parallel across routes; the store is read-only during render, so there is
no lock contention. Output writing compares against `HashSink` results and skips unchanged files,
which keeps `rsync`/CDN invalidation surfaces small.

`.tri/cache/` persists the store, image derivatives, highlight caches, and the last build's
digest map, so CI with a warm cache rebuilds only what a PR touched.

**Targets** (10,000-page site, M-series laptop, warm cache): cold build < 8 s; content-only
rebuild < 50 ms; single-page dev re-render < 10 ms. These are load-bearing numbers — if the
design cannot hit them, the design is wrong.

---

## 12. Dev server

`tri dev` runs `axum` on 4321 (Astro's port; muscle memory is worth respecting) with:

- `notify` watching `content/`, `src/`, `public/`, `styles/`;
- a change classifier that routes each event to the cheapest possible response (§6.3 table);
- a WebSocket HMR channel.

Three update kinds over the wire:

| Kind | Trigger | Client action |
|---|---|---|
| `css` | style block or `.css` change | swap `<style>`/`<link>`, no reload |
| `html` | content or template change | fetch new page HTML, morph the DOM in place, **preserve island state** by leaving hydrated `<tri-island>` subtrees untouched when their id and props hash are unchanged |
| `wasm` | island code change | reload island chunks and re-hydrate affected hosts only |

Rust rebuilds in dev use a dedicated profile: `opt-level=0`, `debug=1`, `codegen-backend=cranelift`
where available, `split-debuginfo=unpacked`, and `lto=false`. Where the island crate is built with
Dioxus, `subsecond` hot-patching can replace island function bodies without a full wasm rebuild.

The error overlay renders `miette` diagnostics with the `.tri` source, the span, and the frame of
the render stack that failed — not a Rust backtrace through generated modules.

---

## 13. Output modes and adapters

`output = "static" | "server" | "hybrid"`, with `#[prerender]` overriding per route.

```rust
pub trait Adapter {
    fn name(&self) -> &str;
    fn features(&self) -> Features;   // streaming, edge middleware, on-demand images, kv, secrets
    /// Rewrite/emit build output: platform config files, function bundles, redirects.
    fn finalize(&self, out: &BuildOutput) -> Result<()>;
}
```

The runtime half of every adapter is a thin shim over one shared core:
`triblenka::App::handle(Request<Body>) -> Response<Body>`, which is a `tower::Service`.

| Adapter | Shape | Notes |
|---|---|---|
| `static` | files in `dist/` | Default. No server. |
| `standalone` | one binary, `hyper` | ~8 MB stripped; scratch container; no runtime deps |
| `wasi` | `wasm32-wasip2`, `wasi:http` | One artifact for Fastly, Spin, wasmCloud, and anything else WASI 0.2 |
| `cloudflare` | `workers-rs` on `wasm32` | KV/R2/D1 bindings exposed to loaders and endpoints |
| `vercel` | Vercel Rust runtime | Fluid Compute; streaming supported; ISR via revalidate headers |
| `lambda` | `lambda_http` | Function URL or API Gateway |

A feature the adapter does not support is a **build error naming the route and the feature**
("`/cart` uses `server:defer` streaming; adapter `lambda` reports `streaming: unsupported`"),
never a silent runtime degradation. Where a safe fallback exists (streaming → fetch-per-island),
it is applied with a warning.

---

## 14. Integrations

```rust
#[async_trait]
pub trait Integration: Send + Sync {
    fn name(&self) -> &str;
    async fn config_setup(&self, cx: &mut ConfigCtx) -> Result<()>;   // add routes, deps, defaults
    async fn routes_resolved(&self, cx: &mut RouteCtx) -> Result<()>;
    async fn page_rendered(&self, cx: &mut PageCtx<'_>) -> Result<()>; // transform HTML
    async fn build_done(&self, cx: &BuildCtx) -> Result<()>;           // sitemaps, feeds, reports
}
```

First-party: `sitemap`, `rss`, `og-image` (render OG cards with `resvg` from a `.tri` template),
`search` (build a `pagefind`-compatible index — the index format is language-agnostic and the
client is 30 KB), `analytics`, `mdx-components`, `redirects`, `robots`, `check-links`.

Site definition:

```rust
// src/site.rs
pub fn site() -> Site {
    Site::new()
        .collection::<Post>()
        .collection::<Author>()
        .integration(sitemap::default())
        .integration(og_image::from_template::<templates::OgCard>())
        .adapter(adapters::Cloudflare::new())
        .middleware(middleware::middleware())
}
```

---

## 15. Risks, and what we do about them

| Risk | Severity | Mitigation |
|---|---|---|
| Rust compile times destroy the authoring loop | **Highest** | Decision B (content ≠ code); cranelift dev profile; islands in a separate crate; subsecond hot-patching; the §6.3 latency table is a test, run in CI |
| `rustc` errors surfacing in generated code | High | Span remapping + pre-flight semantic checks (§5.1); generated code never shown |
| Wasm islands are heavier than JS islands | Low–Medium *(was Medium; reduced by the ladder)* | True in isolation: ~45 KB gz against ~10 KB for a Preact island. The ladder is the real answer — rung 1 costs ~2 KB and rung 2 costs a chunk per interaction, so a page reaches rung 3 only when it holds continuous local state, where the size curve favours wasm anyway. Zero-interaction pages remain free, and `--stats` publishes the numbers rather than hiding them |
| No npm-scale component ecosystem | Medium | Lean on headless/CSS-only patterns; `client:only` JS islands as the pressure valve; ship the 20 components a docs site needs |
| Four rungs are more to learn than Astro's two | Medium | Mitigated by inference: the ladder is a *cost model* to understand, not an API to memorise, because the compiler picks the rung and `--stats` shows what it picked. The failure mode to watch is a surprising payload, which is why the stats column and failing budgets ship *with* inference rather than after it (§9). If users end up writing `rung:*` on most components, inference has failed and we should say so |
| Inventing a template language nobody wants | Medium | Grammar is deliberately Astro-shaped; block forms are a closed set of five; everything else is Rust |
| Scope explosion (this document describes a lot of software) | High | The §16 roadmap is ordered so that M1 is independently useful, and each milestone ships something a real site can run on |
| `salsa`, `rolldown`, `wasm-split` are moving targets | Low–Medium | All are versioned crates; wrap each behind an internal trait so a swap is local |

---

## 16. Roadmap

*The milestone rationale is below; the item-level plan lives in `cairn` — 87 items across six
milestones, generated into [`ROADMAP.md`](ROADMAP.md). Do not restate it here, and do not keep a
parallel list anywhere else: `cairn next` is the source of truth for what to work on.*


**M0 — Spike (2–3 weeks).** Prove the two boundaries the whole design rests on. `.tri` parser +
codegen for text, interpolation, `{#if}`, `{#for}`, and components with props; the template
descriptor and dev-mode walker; `StringSink`; a `Glob` loader over markdown into `redb`; a static
build of a 500-post blog. Measure both loops.
**Kill criteria — two, and either one stops the project:** a content-only rebuild must be under
50 ms, and a structure-only template edit must be under 100 ms end to end. If descriptor swapping
does not work in practice, the framework is a slow Zola and should not exist.

**Result — GO (2026-09-10).** Measured on the generated fixture, release profile, 50 iterations:

| posts | content edit → page on disk | template markup edit → page on disk |
|---|---|---|
| 500 | p50 3.9 ms · p99 9.1 ms | p50 57 µs · p99 243 µs |
| 2,000 | p50 4.0 ms · p99 8.4 ms | p50 37 µs · p99 208 µs |
| 10,000 | p50 3.9 ms · p99 10.0 ms | p50 40 µs · p99 72 µs |

Both budgets are met at every size, and the content path is *flat* with site size rather than
linear, because the watcher names the changed file and no directory is rescanned. The descriptor
mechanism — re-parse, lower, verify the expression table is unchanged, re-render — costs tens of
microseconds, which is roughly 1,400× under its budget.

Two honest caveats, both filed rather than buried. The *full rescan* path, which a watcher never
takes but a cold start does, fails at 10,000 posts (p99 824 ms) and needs an mtime pre-filter and
parallel hashing. And the codegen path emits Rust but nothing compiles it yet, so its half of §5.1
is asserted rather than demonstrated.

**M1 — A static site generator people would actually use.** Slots, scoped CSS + lightningcss,
layouts, file routing + `route!`, `#[static_paths]`, markdown pipeline with highlighting and TOC,
image pipeline, dev server with content + template reload, `miette` diagnostics with span mapping,
`--verify-incremental`, `static` adapter. Plus three items the ladder work added here rather than
later, because each is cheap and shapes what comes after: platform-first navigation (view
transitions and speculation rules, no client router), the accessibility contracts, and
machine-readable diagnostics for the second user (Appendix C.5).

**And the editor tooling, in M1, not later** — a tree-sitter grammar, a `tri-lsp` (completion for
components and props, go-to-definition, diagnostics from the compiler), and `tri fmt`. This was
missing from the first draft of the roadmap and it is the single largest unbudgeted cost in the
project: a bespoke template language without an LSP is a worse experience than a macro that
rust-analyzer already understands, no matter how good the language is. Roughly a third of M1.

*M1 alone competes with Zola, with types. It is the deliverable the project is judged on.*

**M2 — The ladder.** Rung 1 first: server frames, the swap protocol, and the no-JS fallback proof —
smaller than islands, more useful, and the thing that decides how much of rung 3 we ever need. Then
rung inference with `--stats` accountability, automatic cache tags from the provenance graph, and
the resumable-handler spike. Rung 3 lands last and smallest: `#[island]` + registry, the TypeScript
renderer, hydration shim, wasm build and split, size budgets in CI.

The sequencing is the opinion. Building islands first would produce Astro; building frames first
produces a framework whose interactivity story is its own.

**M3 — Server.** `server` and `hybrid` output, tower middleware, endpoints, `server:defer` with
encrypted props and out-of-order streaming, `standalone` + `wasi` adapters.

**M4 — Platform.** Integration trait + first-party set, Cloudflare/Vercel/Lambda adapters,
`Http`/`Sql`/`Git` loaders, live collections, `salsa`-backed incremental CI cache, Dioxus and
`vanilla` renderers.

**M5 — Ecosystem.** `client:only` JS islands via rolldown, optional Node sidecar SSR, CMS
loaders, view-transition client router, `tri new` templates, docs site (dogfooded).

---

## 17. Resolved questions

Everything below was open in the first draft and was closed in design review (Appendix A).

1. **Async frontmatter everywhere?** *Yes*, including on prerendered pages — it matches Astro and
   the ergonomics are worth it. `tri check` warns when a prerendered page performs network I/O,
   because that silently couples your build to a third party's uptime.
2. **Slot type erasure.** `impl Fn` by default; `BoxedSlot` as an explicit opt-in for the rare
   component that stores or reorders its slots.
3. **A `Query<T>` DSL?** *No.* `Iterator` plus a handful of extension traits. Building an ORM by
   accident is a known failure mode of content frameworks.
4. **Borrowed island props?** *No* in v1 — owned, `Serialize + DeserializeOwned`. A `Cow`-based
   derive is not worth the complexity before anyone has complained.
5. **Naming.** Settled: `triblenka`, CLI `tri`, extension `.tri`. It is cheap now and expensive
   after M1, so it is decided now rather than deferred.
6. **`.md` with whitelisted components vs `.md.tri`.** Both stay, because the trade is real and
   should be visible in the file extension. The compiler's error for an unregistered component in
   `content/` names both fixes explicitly.
7. **Props: struct or signature?** *Signature*, with the struct form retained for reuse:
   `#[props] fn (title: &str, href: Route, featured: bool = false)`. Declaring a struct for every
   component is friction Leptos already showed we do not need.

## 18. Sources

Astro: [islands](https://docs.astro.build/en/concepts/islands/) ·
[content loader API](https://docs.astro.build/en/reference/content-loader-reference/) ·
[adapter API](https://docs.astro.build/en/reference/adapter-reference/) ·
[content layer deep dive](https://astro.build/blog/content-layer-deep-dive/) ·
[compiler](https://github.com/withastro/compiler) ·
[island internals](https://softwaremill.com/astro-island-architecture-demystified/) ·
[patterns.dev on islands](https://www.patterns.dev/vanilla/islands-architecture/)

Rust ecosystem: [Leptos islands & SSR](https://rustify.rs/articles/rust-leptos-fullstack-web-2026) ·
[Leptos vs Dioxus 2026](https://rustify.rs/articles/leptos-vs-dioxus-rust-frontend-2026) ·
[Dioxus 0.7: wasm-split, hot-patching](https://dioxuslabs.com/blog/release-070/) ·
[rolldown crate](https://crates.io/crates/rolldown) ·
[oxc_parser](https://crates.io/crates/oxc_parser) · [oxc_transformer](https://crates.io/crates/oxc_transformer) ·
[lightningcss](https://crates.io/crates/lightningcss) · [salsa](https://crates.io/crates/salsa) ·
[rust-analyzer durable incrementality](https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html) ·
[askama](https://github.com/askama-rs/askama) · [mdxjs-rs](https://github.com/wooorm/mdxjs-rs) ·
[Zola](https://www.getzola.org/) ·
[workers-rs](https://developers.cloudflare.com/workers/languages/rust/) ·
[Vercel Rust runtime](https://github.com/vercel-community/rust) ·
[Vite 8 / Rolldown / Oxc](https://deepwiki.com/vitejs/vite/3.3-rolldown-and-oxc-integration)


---

## Appendix A — Design review (2026-09-09)

The first draft was argued against, motion by motion. Nine of eleven positions survived; two were
reversed and several were sharpened. Recorded here so the reasoning is not lost when someone asks
"why is it like this" in a year.

| # | Motion | Verdict |
|---|---|---|
| 1 | Templates must compile; an interpreter is a trap | **Superseded.** Both poles were wrong. Expressions compile, structure is a swappable descriptor (§5.1) |
| 2 | Content is data, pages are code | **Upheld**, claim narrowed: this wins for prose edits, not schema or layout edits. Typed-content rhetoric corrected in §8.1 |
| 3 | Islands should default to Leptos | **Reversed.** Default is TypeScript; Rust islands for shared logic or compute (§9.5) |
| 4 | A bespoke `.tri` file beats a `view!` macro | **Upheld on the audience argument**, but the tooling tail (grammar, LSP, formatter) is now budgeted into M1 as its largest single cost |
| 5 | Use `salsa` for the build graph | **Reversed.** Hand-rolled digest memo table, salsa-shaped boundaries, `--verify-incremental` as the safety net (§11) |
| 6 | `redb` over SQLite | **Upheld**, behind a `Store` trait, with `tri store dump` conceding the inspectability loss (§8.3) |
| 7 | Out-of-order streaming is a differentiator | **Demoted to opt-in.** Streaming, strict CSP, and full-page caching are a pick-two (§9.6) |
| 8 | Build on Leptos SSR instead of a bespoke renderer | **Rejected.** The straight-line sink renderer is the differentiator and is small; Leptos stays an island renderer, not a foundation |
| 9 | Props as a declared struct | **Reversed.** Signature form, struct retained for reuse (§4.2) |
| 10 | The roadmap is achievable | **Rejected as written.** M0–M5 is multiple person-years. M1 is the product; M2+ is a bet placed only if M1 finds users |
| 11 | There is a market for this | **Unproven, and stated as such.** The defensible wedge is Rust-project docs sites and teams who want Node out of CI. Anything beyond that is speculation |

### The two reversals worth reading in full

**Islands (motion 3).** The argument that broke the original position: a framework whose thesis is
"ship less to the browser" cannot default its interactivity story to a ~45 KB wasm core when the
median island — a theme toggle, a drawer, a copy button — is 1–2 KB of TypeScript. Defaulting to
Rust would have been language loyalty overriding the product thesis. The "no Node in CI" property
is what makes this painless: oxc and rolldown are Rust libraries, so TypeScript islands cost
nothing in toolchain terms. Rust islands remain the right answer where they are actually better,
and `--stats` makes that a measurement rather than an argument.

**Build graph (motion 5).** `salsa` is the right *shape* and the wrong *commitment*. Adopting an
invasive, macro-heavy dependency before the graph has proven hard is the kind of choice that is
cheap to make and expensive to unmake. The counter-risk — hand-rolled incremental builds silently
serving stale output — is real, and is answered by `--verify-incremental` rather than by a
dependency.

### What this review did not resolve

The scope question is genuine and remains open: this document describes several person-years of
software, and the honest gate is M0's kill criteria followed by M1 shipping as a standalone,
usable static site generator. If M1 does not find users, M2 should not be built.

---

## Appendix B — Astro mechanics, verified from source (2026-09-09)

Read against a shallow clone of `withastro/astro` and `withastro/compiler` at main. Paths below are
relative to `packages/astro/src/` unless marked *(compiler)*. This appendix exists because the
first draft was written from documentation, and documentation describes intent while source
describes what actually had to be handled.

### B.1 The island custom element — `runtime/server/astro-island.ts`

Seven mechanisms, six of which the first draft missed entirely. All apply to `<tri-island>`.

1. **Children may not exist when the element connects.** Under HTML streaming, `connectedCallback`
   can fire before the island's server-rendered children are parsed. Astro emits an `await-children`
   attribute and a trailing `<!--astro:end-->` marker comment, watches for it with a
   `MutationObserver`, and falls back to `DOMContentLoaded` in case the marker was stripped.
   Without this, a streamed island hydrates against a partial subtree.
2. **Hydration is top-down.** An island whose ancestor is still un-hydrated (`astro-island[ssr]`)
   defers and re-runs on the parent's `astro:hydrate` event. Bottom-up hydration would let a parent
   re-create and discard a child that had already mounted.
3. **`display: contents` forces the observer onto the children.** Because the host element has no
   box, `IntersectionObserver` on the host never fires — `client/visible.ts` observes
   `el.children` instead. Exactly the kind of detail that costs a day to rediscover.
4. **Slots are recovered from the DOM**, from `<astro-slot>` elements and
   `<template data-astro-template>`, each guarded by `closest(tagName).isSameNode(this)` so a
   nested island does not steal its parent's slots.
5. **Failed dynamic imports are retried with a query-parameter cache-buster**, not a hash, because
   the browser caches the failed fetch in its module map keyed by URL.
6. **Errors dispatch a cancelable `astro:hydration-error` event** before logging, so an application
   can suppress or report them; import failures are caught inside the loader so directives never
   leak rejections.
7. **`observedAttributes = ['props']`** — writing new props to the element re-hydrates it. This is
   the seam view transitions and server islands use.

*Our delta:* items 1–4 and 6 are adopted as-is; they are correctness, not taste. Item 5 does not
apply to us in the same form (we fetch wasm chunks, not ES modules) but the underlying lesson —
a failed fetch must be retried under a different URL — does.

### B.2 Props serialization — `runtime/server/serialize.ts`

Astro cannot use plain JSON, because JSON loses types. It ships a tagged-tuple encoding —
`[typeId, value]` with ids for object, array, RegExp, Date, Map, Set, BigInt, URL, three typed
array kinds, and ±Infinity — and revives it **top-down** on the client, since `JSON.parse`'s
reviver runs bottom-up. Cyclic references are detected with a `WeakSet` and raise a build error
naming the component and its directive.

*Our delta:* this entire mechanism is a workaround for an untyped boundary, and we do not need it.
The island's props are one Rust struct serialized by serde and deserialized by serde, so types
survive by construction. What we should copy is the **error**: a cyclic or non-serializable prop
must fail at build time naming the component and the offending field, not silently at runtime.

### B.3 Server islands — `runtime/server/render/server-islands.ts`, `core/encryption.ts`

- AES-GCM, chosen because it authenticates rather than merely conceals — tampering is detected.
- **Both the props and the component export are encrypted**, then passed as query parameters
  `e`, `p`, `s` (slots), with an explicit check that the URL stays under 2048 characters.
- The key is supplied out of band (`ASTRO_KEY`, base64 raw key), *not* generated per build, and a
  one-way SHA-256 of it is recorded so the build cache invalidates when the key changes.
- The host id is a random UUID per render; the fragment arrives via an inline
  `<script type="module" data-astro-rerun data-island-id="…">`.

*Our delta:* adopted wholesale, including `TRI_KEY` and the key hash as a build-cache input. §9.6
was corrected accordingly — a per-build key breaks both horizontal scaling and incremental builds.

### B.4 CSP is hash-based — `runtime/server/render/csp.ts`, `core/csp/common.ts`

Astro digests inline script and style *content* and accumulates the hashes into the policy; it
does not mint per-response nonces. Server islands compute their digest during `init()`, i.e. in the
head phase before the body streams. The renderer also handles the `-elem`/`-attr` narrowing
correctly (a browser will not fall back from a narrower directive to the baseline) and suppresses
hashes when `'unsafe-inline'` is present, since the spec makes browsers ignore one in the presence
of the other.

*Our delta:* this refuted the trilemma asserted in the first draft. The real constraint is that
**inline script content must be stable and known at head-flush time** — which a fixed shim with a
substituted id satisfies. §9.6 rewritten.

### B.5 Route priority — `core/routing/priority.ts`

A segment-by-segment comparator, not the whole-route heuristic the first draft assumed. Notably it
handles **partially dynamic segments** (`game-[title]` outranks `[title]`), treats an `index` file
as one segment deeper than its URL, and contains a special case making `/foo` more specific than
`/foo/[...bar]` when the lengths differ by exactly one. Ties fall through to `localeCompare`.

*Our delta:* adopted, except the tiebreak, which becomes byte-wise — `localeCompare` is
locale-dependent and therefore a source of machine-dependent build output. And route ambiguity is
ordered rather than rejected; only identical patterns are an error. §7.1 rewritten.

### B.6 Style scoping — *(compiler)* `internal/hash.go`, `internal/transform/scope-{html,css}.go`

- Scope id: xxhash → base32 → lowercased → first 8 characters.
- Default strategy wraps the injected selector in **`:where(...)`**, contributing zero specificity;
  `class` and `attribute` strategies exist as opt-ins.
- A hard-coded never-scoped set: `Fragment`, `base`, `font`, `frame`, `frameset`, `head`, `link`,
  `meta`, `noframes`, `noscript`, `script`, `style`, `slot`, `title`, plus the `:root` selector.
- In dev, elements are annotated with `data-astro-source-file`, which is what powers the
  click-to-source dev toolbar.

*Our delta:* all four adopted (§10.1). The `:where()` default in particular is a correctness
property, not a preference. The dev-only source annotation is a cheap way to get click-to-source in
our error overlay and is now an M1 item.

### B.7 The content store — `content/mutable-data-store.ts`

Not a database: an in-memory `Map` persisted as **chunked JavaScript modules** with a manifest,
written atomically (temp file + rename, with retry) and debounced. Three things worth stealing:

1. `set()` returns `false` when the incoming digest matches the stored one — the design's
   `Changed::No`, confirmed as the mechanism that keeps a full refetch from rebuilding a site.
2. **Asset references inside entry data are recorded by path.** Image fields are prefixed during
   schema parsing; the store strips the prefix, records the *location* of each such field, and
   read-time resolution rewrites only those paths instead of traversing or cloning the whole entry.
3. A corrupt or unreadable cache **warns and rebuilds from scratch** rather than failing the build.

*Our delta:* (1) is already the design. (2) is new and is now an M1 item — it is how markdown
image references get into the asset pipeline without a full walk of every entry on every read.
(3) becomes a rule for `.tri/cache`: a cache that cannot be read is a warning and a cold build,
never an error. redb gives us (atomicity, durability) for free.

### B.8 The adapter runtime — `core/app/base.ts`

`RenderOptions` is the whole contract between an adapter and the framework, and it is small:
`addCookieHeader`, `clientAddress`, `locals`, `prerenderedErrorPageFetch`, `waitUntil`, and an
advanced `routeData` override.

*Our delta:* two of these were missing from §13's `Features` matrix and are now added.
`waitUntil` — keeping background work alive after the response is sent — is required for cache
writes on edge platforms, and an adapter that lacks it must be prevented from claiming
write-behind caching. `prerenderedErrorPageFetch` matters because a dynamic route that 404s in a
hybrid build still needs to serve the *prerendered* 404, which the server cannot read from disk on
every platform.

### B.9 Summary: adopt, adapt, diverge

| Mechanism | Decision |
|---|---|
| Streaming-safe children marker, top-down hydration, observer-on-children, slot nesting guard | **Adopt** — correctness, learned the hard way |
| Cancelable hydration-error event | **Adopt** |
| Tagged-tuple prop encoding | **Skip** — serde makes it unnecessary; keep the build-time error |
| AES-GCM server islands, out-of-band key, key hash in the cache | **Adopt**, corrects §9.6 |
| Hash-based CSP with head-phase digests | **Adopt**, refutes the first draft's trilemma |
| Segment-wise route comparator incl. partial-dynamic segments | **Adopt** |
| `localeCompare` tiebreak | **Diverge** — byte-wise, for reproducible builds |
| Any-ambiguity-is-an-error routing | **Diverge** — order it, explain it, only reject exact duplicates |
| `:where()` scoping, never-scoped element set, base32 short hash | **Adopt** |
| Recorded asset-field paths in stored entries | **Adopt** |
| Corrupt cache warns and rebuilds | **Adopt** |
| `waitUntil` and `prerenderedErrorPageFetch` in the adapter contract | **Adopt** |

---

## Appendix C — Beyond Astro (2026-09-10)

Appendix B established what Astro does. This appendix is the wider survey: the ideas worth taking
from everywhere else, and the ones worth refusing. It exists because a design that only reads one
prior framework produces a port, and a port's best possible review is "impressively faithful".

### C.1 What we take

| Idea | From | What we take |
|---|---|---|
| **Resumability** | Qwik | Rung 2 (§9). Serialize captured state, attach one delegated listener, fetch a closure on demand. Better in Rust: `rustc` already knows a closure's captured environment as a typed struct, so the serialization is derived rather than discovered |
| **Automatic partial hydration** | Marko | Rung inference. Directives are a workaround for a compiler that cannot infer; ours can, with directives kept as an override |
| **Hypermedia over the wire** | htmx | Rung 1 (§9). htmx's analysis is correct and its constraint is a slow server. Ours is 200µs |
| **Forms that work without JavaScript** | Remix | Progressive enhancement as a *checked* property, not a convention — see C.3 |
| **Typed server functions** | Leptos `#[server]`, tRPC | If we do a function-call boundary, it is typed on both sides. `"use server"` is a string where a type belongs |
| **Cross-document view transitions, speculation rules** | The platform | We do not ship a client router. Ever. Most frameworks cannot take this position because their state lives in the JS heap; ours does not |
| **Content-addressed remote build cache** | Bazel, Turborepo | Falls out of determinism (§11) for free: inputs fully determine outputs, so a shared cache keyed on input digests is sound |
| **Build attestation** | SLSA | Also falls out of determinism. The archival and regulated-publishing case is the one nobody else can serve |
| **The component model** | WASI 0.2 | A site as a composable `wasm32-wasip2` component rather than a container image |
| **Client-side typed queries** | Pagefind, local-first sync | Ship a compact subset of the content index for instant search, filter, and sort with no round trip. Generalizes Pagefind because our content is typed |

### C.2 What we refuse, and why

- **A client-side router.** The platform has one now. Shipping ours would be re-implementing 2019.
- **An SPA mode.** Leptos and Dioxus own that problem and do it well.
- **Being a sync engine.** Read-only content sync is in scope; collaborative mutable state is a
  different product (Electric, Zero, Jazz). Saying so is worth more than hedging.
- **Stringly-typed server boundaries**, CSS-in-JS, a client data-fetching library, and JS SSR.
- **A configuration knob for the ladder.** The compiler picks; you override per component. A knob
  here would be a decision we refused to make, exported to the user.

Identity comes from refusals. Every item above is something a reasonable person would ask for.

### C.3 Guarantees as compile errors

The family the zero-JavaScript contract belongs to. A guarantee that is not enforced is a slogan,
and every framework's performance and accessibility promises are slogans.

```rust
#![deny(javascript)]         // no byte of JS may reach this page
#![require(no_js_fallback)]  // every interaction must work with JS disabled
#![deny(external_requests)]  // no third-party origin on the critical path
```

Plus the budgets already in `tri.toml`, and the accessibility checks (`alt` required at compile
time, heading order, label association, `lang`) promoted from lints to contracts a page can opt
into. Rungs 0–2 satisfy `no_js_fallback` by construction, so the compiler can *prove* the property
rather than test for it; rung 3 cannot, and the build says so by name.

### C.4 Automatic cache tags

Every other framework makes you write `revalidateTag("post-123")` by hand and then debug the one you
forgot. We already know, per fragment, exactly which content fields a render read (§ incrementality)
— so the cache tags **are** the dependency edges. They are derived, not typed, and a frame's cache
policy falls out of the graph that already exists for incremental builds.

This is the clearest case in the design of one mechanism paying for itself three times: incremental
rebuilds, deploy plans, and cache invalidation are the same graph.

### C.5 The second user

In 2026 a large share of the code in any repository is written by an agent, and every framework
still has exactly one designed user: a person in an editor.

We are further along here than anyone, mostly by accident: deterministic builds an agent can verify,
provenance it can query, typed content instead of `any`, and a backlog that lives in the repository.
Finishing the thought is cheap — every diagnostic emittable as JSON, `tri explain --json`, a
machine-readable project schema, and error messages written to be *acted on* rather than read.

"The first framework designed for agents as a first-class user" is a defensible position that is
currently unoccupied, and the prerequisites are already in the design for other reasons.

### C.6 What is speculative

Stated plainly, because this appendix is the most speculative part of the document:

- **Rung 2 is unproven.** Qwik's serialization is subtle, and we do not know what a captured
  reference costs when it crosses into the document. It is a spike with a kill criterion, not a
  commitment.
- **Rung inference is an ergonomic bet.** Implicit behaviour delights when right and infuriates when
  wrong. It survives only if `--stats` and failing budgets make the implicit visible every build.
- **The frame round trip depends on the origin being close.** At the edge it is invisible; from one
  region to a reader on another continent it is not. The honest scope is: rung 1 is the default when
  the adapter puts the origin near the reader, and `tri check` should say so when it does not.
