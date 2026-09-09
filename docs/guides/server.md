# Server: middleware, context, endpoints

> Design-stage docs. Applies to `output = "server"` and `"hybrid"`.

## Middleware is tower

```rust
// src/middleware.rs
use triblenka::prelude::*;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub fn middleware() -> impl Layer<Router> {
    ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::overriding(CSP, csp()))
        .layer(auth::require_session().only(path("/admin/*")))
}
```

Every [`tower-http`](https://docs.rs/tower-http) layer works: compression, CORS, timeouts, rate
limiting, request IDs, tracing. Register it once:

```rust
Site::new().middleware(middleware::middleware())
```

## Context

Pages and endpoints receive a `Context`:

```rust
cx.request()            // &http::Request
cx.params::<Params>()?  // typed route params
cx.query::<Q>()?        // typed query string
cx.json::<T>().await?   // typed body
cx.cookies()            // read/write cookies
cx.locals::<Session>()  // values inserted by middleware
cx.url()                // absolute URL of this request
cx.client_address()     // as reported by the adapter
```

In a `.tri` page, `cx`, `params`, and `props` are in scope in the frontmatter.

## Responses

```rust
Response::html(body)
Response::json(&value)
Response::redirect(route!(blog::index()), 302)
Response::not_found()
Response::text("ok").header(CACHE_CONTROL, "no-store")
```

Returning `Err(Error::NotFound)` from a page renders `src/pages/404.tri` with a 404 status.

## Caching and revalidation

```rust
cx.cache().max_age(60).stale_while_revalidate(600);
cx.cache().tag("blog").tag(format!("post:{}", post.slug));
```

Adapters that support tag-based purging (Cloudflare, Vercel) map these onto their own mechanism;
others fall back to `Cache-Control`. `revalidate::tag("post:hello").await?` from an endpoint purges
a tag after a CMS webhook.

## Sessions and secrets

```rust
let key = secret!("STRIPE_KEY");   // resolved by the adapter at runtime, not baked into the build
```

`secret!` reads from the adapter's secret store where one exists (Workers bindings, Vercel env,
Lambda env) and from the process environment otherwise. It fails the build if the name is not
declared in `tri.toml`, so a missing production secret is caught before deploy rather than at 3am.
