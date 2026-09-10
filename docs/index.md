# Triblenka documentation

> **Design-stage documentation.** These pages describe the v1 API as designed, ahead of
> implementation. See [`DESIGN.md`](../DESIGN.md) for the architecture and the reasoning.

Triblenka builds content-driven websites in Rust. It renders HTML on the server — at build time
or per request — and ships JavaScript only for the components you explicitly mark as interactive.

## Start here

1. [Getting started](getting-started.md) — a working site in five minutes.
2. [Components](guides/components.md) — the `.tri` language.
3. [Content collections](guides/content-collections.md) — where your content lives.
4. [Islands](concepts/islands.md) — how interactivity works.

## The mental model

```
content/**          data      never compiled, reloads in milliseconds
src/pages/**        routes    one file = one URL
src/components/**   code      .tri (server) and #[island] (client)
public/**           verbatim  copied as-is
        │
        ▼
    tri build  ──►  dist/            static files
               ──►  target/…/server  a binary, if output = "server"
```

Two ideas do most of the work:

**Server-first.** A `.tri` component runs on the server and produces HTML. It has no client-side
lifecycle, no state, and no bundle cost. Most of your site is this.

**Islands.** A component marked `#[island]` compiles to WebAssembly and hydrates in the browser on
a trigger you choose. It is an isolated widget in an otherwise static page, and it loads only its
own code.

## Concepts

- [Why Triblenka exists](concepts/why.md) — the thesis, and what it costs
- [Architecture](concepts/architecture.md) — the three artifacts and the render walk
- [Incrementality and provenance](concepts/incrementality.md) — `tri impact`, `tri why`
- [Determinism](concepts/determinism.md) — the reproducibility guarantee
- [Interactivity](concepts/interactivity.md) — the four-rung ladder, and how the compiler picks
- [Islands](concepts/islands.md) — rung 3: partial hydration
- [Contracts](concepts/contracts.md) — guarantees the build enforces, not slogans
- [Errors](concepts/errors.md) — the error-quality contract

## Guides

- [Publishing](guides/publishing.md) — content deploys, code deploys, migrations
- [Components](guides/components.md)
- [Routing](guides/routing.md)
- [Content collections](guides/content-collections.md)
- [Styling](guides/styling.md)
- [Images and assets](guides/assets.md)
- [Middleware and endpoints](guides/server.md)
- [Deployment](guides/deployment.md)
- [Migrating from Astro](guides/from-astro.md)

## Reference

- [CLI](reference/cli.md)
- [Configuration](reference/config.md)
- [Directives](reference/directives.md)
- [API](reference/api.md)
