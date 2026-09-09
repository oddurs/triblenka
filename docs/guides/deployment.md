# Deployment

> Design-stage docs.

## Output modes

```toml
# tri.toml
output = "static"    # every route prerendered  (default)
output = "server"    # every route rendered per request
output = "hybrid"    # prerendered by default, opt out per route
```

Per-route overrides are `#[prerender]` and `#[prerender(false)]` in the page frontmatter. Static
and server rendering share the same code path, so a route behaves identically either way.

## Adapters

```rust
// src/site.rs
Site::new().adapter(adapters::Cloudflare::new())
```

| Adapter | Output | Notes |
|---|---|---|
| `Static` | `dist/` | Default. Any static host or CDN. No server. |
| `Standalone` | one binary | `hyper` server, ~8 MB stripped. `FROM scratch` containers. |
| `Wasi` | `wasm32-wasip2` component | Runs on Fastly, Spin, wasmCloud, anything speaking `wasi:http`. |
| `Cloudflare` | Worker (wasm) | KV, R2, D1, and Queues bindings exposed to loaders and endpoints. |
| `Vercel` | Vercel Rust runtime | Fluid Compute; streaming; ISR through revalidation headers. |
| `Lambda` | `lambda_http` | Function URL or API Gateway; static assets to S3/CloudFront. |

### Static

```sh
tri build && ls dist/
```

Deploy `dist/` anywhere. Redirects and headers are emitted in each host's native format where the
adapter knows one (`_redirects`, `_headers`, `vercel.json`).

### Standalone

```dockerfile
FROM rust:1.98 AS build
WORKDIR /app
COPY . .
RUN cargo install triblenka-cli && tri build --adapter standalone

FROM gcr.io/distroless/cc
COPY --from=build /app/target/release/my-site /my-site
COPY --from=build /app/dist /dist
CMD ["/my-site"]
```

One process, no runtime dependencies, no package manager in the image.

### Cloudflare

```sh
tri build --adapter cloudflare
npx wrangler deploy
```

Static assets go to Workers Assets; the wasm Worker handles the rest. Bindings are typed:

```rust
let kv = cx.bindings::<Env>().sessions();
let obj = cx.bindings::<Env>().uploads().get("key").await?;
```

### Vercel

```sh
tri build --adapter vercel && vercel deploy --prebuilt
```

Prerendered routes become static output; server routes become Rust functions on Fluid Compute.
Streaming and out-of-order server islands are supported. `cx.cache().tag(...)` maps to Vercel's
revalidation.

## Feature checking

Adapters declare what they support. Using something the target cannot do is a **build error naming
the route and the feature**, never a silent runtime failure:

```
error: route `/cart` uses `server:defer` with streaming
       adapter `lambda` reports streaming: unsupported
  = help: streaming falls back to one fetch per island; set
          `[adapter] allow_fallbacks = true` to accept the extra round trip
```

## CI

```yaml
- run: tri check --strict          # types, dead links, missing alt text, size budgets
- run: tri build --adapter vercel
```

Cache `.tri/cache/` between runs: the content store, image derivatives, and highlight caches are
digest-keyed, so a PR that touches one post rebuilds one page.
