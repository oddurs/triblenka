# CLI reference

> Design-stage docs.

```
tri <command> [options]
```

| Command | Purpose |
|---|---|
| `tri new <name>` | Scaffold a site (`--template blog\|docs\|minimal`) |
| `tri dev` | Dev server with hot reload |
| `tri build` | Production build |
| `tri preview` | Serve the build output as an adapter would |
| `tri check` | Type, link, asset, and budget checks |
| `tri add <integration>` | Add and wire an integration |
| `tri info` | Versions, toolchain, resolved config, adapter features |
| `tri explain <route>` | Why a route resolves as it does; what it renders and ships |

## `tri dev`

```
--port <n>            default 4321
--host [addr]         expose on the network
--open                open a browser
--no-islands          skip wasm builds; islands render static (fast iteration on markup)
--profile <name>      cargo profile, default `dev-fast` (cranelift where available)
```

Watches `content/`, `src/`, `public/`, `styles/`, and `tri.toml`, and picks the cheapest response
per change: content → re-render (ms), template → incremental rebuild → DOM morph, island → wasm
rebuild → re-hydrate only that island.

## `tri build`

```
--adapter <name>      override the adapter in src/site.rs
--out <dir>           default dist/
--base <path>         deploy under a subpath
--drafts              include draft entries
--no-cache            ignore .tri/cache
--stats               per-route report: HTML size, CSS, JS, wasm, render time
```

`--stats` output:

```
route                     html     css      js     wasm   render
/                        12.4 KB  3.1 KB  1.1 KB  44 KB    1.8ms
/blog                    31.0 KB  3.4 KB      0       0    3.2ms
/blog/hello-world        22.7 KB  4.0 KB  1.1 KB  44 KB    2.1ms
─────────────────────────────────────────────────────────────────
1,204 routes in 6.1s  ·  cache hit 96%  ·  0 budget violations
```

## `tri check`

```
--strict              warnings become errors
--links               internal link validation (on by default)
--budgets             per-page JS/wasm/CSS budgets from tri.toml
--a11y                alt text, heading order, label association, lang attribute
```

Runs in CI. Fails on dead internal links, missing `alt`, undeclared secrets, undeclared image
domains, adapter feature mismatches, and budget violations.

## `tri explain`

```
$ tri explain /blog/hello-world
route      /blog/:slug          src/pages/blog/[slug].tri
mode       prerendered          via #[static_paths] (1,204 paths)
data       collection `blog`    content/blog/hello-world.md  digest 7b2c…
components Base, Prose, Card    3 components, 4.0 KB scoped CSS
islands    Newsletter           client:visible → isl_a1b2.wasm (3.2 KB) + core (44 KB)
output     dist/blog/hello-world/index.html   22.7 KB
```

## Environment

| Variable | Effect |
|---|---|
| `TRI_LOG` | `error\|warn\|info\|debug\|trace` (uses `tracing`) |
| `TRI_CACHE_DIR` | override `.tri/cache` |
| `TRI_BASE_URL` | override `base_url` for a single build |
