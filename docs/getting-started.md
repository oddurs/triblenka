# Getting started

> Design-stage docs. The commands below describe the intended v1 CLI.

## Requirements

- Rust 1.98 or newer (edition 2024)
- For islands: the `wasm32-unknown-unknown` target and `wasm-bindgen-cli`

```sh
rustup target add wasm32-unknown-unknown
cargo install triblenka-cli wasm-bindgen-cli
```

## Create a site

```sh
tri new my-site --template blog
cd my-site
tri dev
```

`tri dev` serves on <http://localhost:4321> and watches your files. Editing anything under
`content/` re-renders in milliseconds; editing a `.tri` template triggers an incremental Rust
build and a hot HTML swap that preserves the state of already-hydrated islands.

## What you get

```
my-site/
├── Cargo.toml
├── tri.toml            # base URL, output mode, image defaults
├── content/
│   └── blog/
│       └── hello-world.md
├── public/
│   └── favicon.svg
├── styles/
│   └── global.css
└── src/
    ├── site.rs         # collections, integrations, adapter
    ├── content.rs      # collection types
    ├── layouts/
    │   └── base.tri
    ├── components/
    │   └── card.tri
    └── pages/
        ├── index.tri
        └── blog/
            ├── index.tri
            └── [slug].tri
```

## Your first page

`src/pages/index.tri`:

```html
---
use triblenka::prelude::*;
use crate::layouts::Base;

let year = 2026;
---

<Base title="Hello">
  <h1>Hello, world</h1>
  <p>Built in { year }.</p>
</Base>
```

Everything above the second `---` is Rust that runs on the server. Everything below is markup.
`{ ... }` interpolates a Rust expression and HTML-escapes it.

## Your first content collection

Content is data, described by a Rust type. `src/content.rs`:

```rust
use triblenka::prelude::*;

#[derive(Collection, Deserialize)]
#[collection(name = "blog", loader = Glob::new("content/blog/**/*.md"))]
pub struct Post {
    pub title: String,
    pub date: Date,
    pub description: String,
    #[serde(default)]
    pub draft: bool,
    #[content]
    pub body: Markdown,
}
```

Register it in `src/site.rs`:

```rust
pub fn site() -> Site {
    Site::new().collection::<Post>()
}
```

Now `content::blog()` is a typed query anywhere in your site, and a markdown file missing
`description` fails the build with the file name, the line, and the expected type.

## Your first interaction

Most interactivity on a content site does not need a component runtime. Start with a **server
frame** — a region the server re-renders and the browser swaps in:

```html
---
// src/components/post-list.tri
#[frame(id = "posts")]
#[props]
fn (tag: Option<String> = None);

let posts = content::blog().published().by_tag_opt(&tag);
---

{#for t in content::blog().tags()}
  <a href={frame!(posts(tag = t))}>{ t }</a>
{/for}

{#for post in posts}
  <Card post={post} />
{/for}
```

Clicking a tag re-renders the list on the server and swaps it in — ~2 KB of JavaScript for the whole
page, no component runtime, and with JavaScript disabled the same links are ordinary navigation.

Reach past this only when the state is genuinely local and continuous. See
[Interactivity](concepts/interactivity.md) for the four rungs and how the compiler picks one.

## Your first island

`src/components/counter.rs`:

```rust
use triblenka::island;
use leptos::prelude::*;

#[island]
#[component]
pub fn Counter(#[prop(default = 0)] start: i32) -> impl IntoView {
    let (n, set_n) = signal(start);
    view! { <button on:click=move |_| set_n.update(|n| *n += 1)>"Clicked " {n} " times"</button> }
}
```

Use it from any `.tri` file:

```html
<Counter start={3} client:visible />
```

The button renders on the server with `3` already in it. Its WebAssembly loads only when it
scrolls into view.

That example needs the `islands-leptos` feature. The **default** island renderer is TypeScript,
which is smaller for UI-shaped widgets and needs no Node toolchain — see
[Islands](concepts/islands.md) for both forms and for when each one is the right call.

## Build and preview

```sh
tri build          # writes dist/ (and a server binary in hybrid/server mode)
tri preview        # serves the build output exactly as an adapter would
tri check          # types, dead links, missing images, island size budgets
```

## Next

- [Components](guides/components.md) — slots, props, conditionals, loops
- [Routing](guides/routing.md) — dynamic pages and `route!`
- [Deployment](guides/deployment.md) — static, standalone, Cloudflare, Vercel, Lambda, WASI
