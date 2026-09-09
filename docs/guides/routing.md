# Routing

> Design-stage docs.

Every file in `src/pages/` becomes a route.

| File | Route |
|---|---|
| `src/pages/index.tri` | `/` |
| `src/pages/about.tri` | `/about` |
| `src/pages/blog/index.tri` | `/blog` |
| `src/pages/blog/[slug].tri` | `/blog/:slug` |
| `src/pages/docs/[...path].tri` | `/docs/*` |
| `src/pages/rss.xml.rs` | `/rss.xml` (endpoint) |

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

## Dynamic pages

A dynamic page in a static build must say which pages exist:

```html
---
// src/pages/blog/[slug].tri
use triblenka::prelude::*;

#[params]
pub struct Params { pub slug: String }

#[static_paths]
async fn paths() -> Result<Vec<Path<Params, Post>>> {
    Ok(content::blog()
        .published()
        .map(|post| Path::new(Params { slug: post.slug.clone() }).with_props(post))
        .collect())
}
---

<Base title={&props.title}>
  <h1>{ props.title }</h1>
  <Prose html={props.body.html()} />
</Base>
```

`with_props` hands the entry straight to the page, so you do not look it up twice.

In `output = "server"`, drop `#[static_paths]` and read `params.slug` at request time:

```html
---
#[prerender(false)]
let post = content::blog().by_slug(&params.slug).ok_or(Error::NotFound)?;
---
```

## Typed links

`route!` is generated from your page tree:

```html
<a href={route!(blog::post(&post.slug))}>{ post.title }</a>
<a href={route!(docs::path(["guides", "routing"]))}>Routing</a>
<a href={route!(index())}>Home</a>
```

Rename `[slug].tri`, and every call site fails to compile. Pass the wrong number or type of
params, and it fails to compile. There is no string-concatenation path that skips this — internal
links written as plain strings are reported by `tri check --strict`.

## Rendering mode per route

```rust
#[prerender]         // force build-time HTML, even in server output
#[prerender(false)]  // force per-request rendering, even in static output (needs an adapter)
```

The site-wide default comes from `output` in [`tri.toml`](../reference/config.md).

## Endpoints

```rust
// src/pages/api/search.rs
use triblenka::prelude::*;

pub async fn get(cx: Context) -> Result<Response> {
    let q = cx.query::<SearchQuery>()?;
    let hits = search::index().query(&q.term).take(20);
    Ok(Response::json(&hits))
}

pub async fn post(mut cx: Context) -> Result<Response> {
    let body: Feedback = cx.json().await?;
    db::save(body).await?;
    Ok(Response::no_content())
}
```

Exported `get`, `post`, `put`, `patch`, `delete`, `all` become methods. Anything not exported
returns `405` with a correct `Allow` header. Prerendered endpoints (`#[prerender]`) run at build
time and write a file — how `rss.xml` and `sitemap.xml` work.

## Redirects and 404s

```toml
# tri.toml
[redirects]
"/old-blog/:slug" = { to = "/blog/:slug", status = 301 }
```

`src/pages/404.tri` and `src/pages/500.tri` are used by the dev server, `tri preview`, and every
adapter that supports custom error documents.
