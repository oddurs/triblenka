# Architecture

> Design-stage docs. See [`DESIGN.md`](../../DESIGN.md) for the reasoning behind each choice.

## Three artifacts

A Triblenka build produces up to three things, and knowing which is which explains most of the
system's behavior:

```
   the binary        compiled from src/**        changes on a code deploy
   the store         loaded from content/**      changes on a content deploy
   dist/             rendered output             derived from the two above
```

The binary is stable across content changes. That is the whole thesis
([why](why.md)) and everything below serves it.

## What `tri build` does

```
  ┌── build.rs ──────────────────────────────────────────────┐
  │  .tri files ─► parse ─► codegen ─► $OUT_DIR/*.rs         │
  │                     └─► template descriptors + span map  │
  └──────────────────────────────────────────────────────────┘
                            │  rustc
                            ▼
  ┌── the site binary ───────────────────────────────────────┐
  │                                                          │
  │  loaders ──► store (redb) ──► in-memory index            │
  │                    │                                     │
  │  route table ──────┼──► render walk ──► Sink ──► bytes   │
  │                    │         │                           │
  │                    │         ├──► collected CSS per page │
  │                    │         └──► island registrations   │
  │                    ▼                                     │
  │              memo table (digest-keyed)                   │
  └──────────────────────────────────────────────────────────┘
                            │
                            ▼
                 dist/ + _tri/ + deploy plan
```

Every arrow is a query in the memo table, keyed on the digests of what it read. Nothing re-runs
unless one of its inputs moved. See [Incrementality](incrementality.md).

## What a request does

In `server` or `hybrid` output the same machinery runs per request:

```
  request ─► adapter shim ─► tower middleware ─► route match ─► page component
                                                       │
                          ┌────────────────────────────┼───────────────────┐
                          ▼                            ▼                   ▼
                     frontmatter                  render walk        island collection
                   (store or live data)        (write into Sink)   (props → hydration)
                          │                            │                   │
                          └──────► response head + streamed body ◄─────────┘
```

Note what is *absent*: no module graph resolution, no JIT warmup, no framework runtime boot. A
cold request on a fresh instance runs the same instructions as a warm one.

## The render walk

Rendering is a depth-first write into a `Sink`. There is no virtual DOM and no intermediate tree —
static markup compiles to `sink.raw(&'static str)` over pre-concatenated literals, so the fast path
is a `memcpy` per run of static HTML.

Three sinks, one walk:

| Sink | Used by | Purpose |
|---|---|---|
| `StringSink` | static builds | one allocation, sized from a cached hint |
| `StreamSink` | server output | writes `Bytes` chunks into an HTTP body |
| `HashSink` | the build graph | renders to a content hash to detect real change |

`HashSink` is why an incremental build can decide *not* to write a file: if the hash matches what is
already on disk, the bytes are identical and the deploy plan stays small.

## Two render modes, one source of truth

The compiler emits both straight-line code and a **template descriptor** — a tree of static chunks,
slots, and expression slots indexed into a table of compiled thunks.

```
release   monomorphized straight-line calls          maximum speed
dev       walk the descriptor, thunks stay compiled  swappable at runtime
```

Expressions are compiled Rust in both modes; the descriptor only *orders* them. That is what lets
the dev server hot-swap markup without `rustc` while keeping exactly one language and zero semantic
drift. Property tests assert both modes produce byte-identical output.

## Crate map

| Crate | Owns |
|---|---|
| `tri-cli` | commands, cargo orchestration, diagnostic remapping |
| `tri-compiler` | `.tri` parser, codegen, descriptors, span maps, pre-flight checks |
| `tri-core` | `Sink`, `Render`, `Component`, routing, the memo table |
| `tri-content` | collections, loaders, the store, markdown |
| `tri-assets` | CSS scoping and bundling, images, fonts, fingerprinting |
| `tri-islands` | registry, hydration protocol, wasm build and split |
| `tri-macros` | `#[props]`, `#[island]`, `route!`, `asset!`, `component!` |
| `tri-devserver` | watcher, change classifier, HMR, error overlay |
| `tri-adapter-*` | one crate per deploy target |

Islands live in their own crate on purpose: editing an island must never rebuild the site, and
editing the site must never rebuild wasm.

## Where the boundaries are

Three rules that the architecture enforces rather than merely encourages:

1. **`content/**` is never compiled.** If a feature requires compiling content, the feature is
   wrong. This is what keeps content edits at millisecond latency.
2. **Rendering never mutates the store.** The store is read-only during a render, which is why
   pages can render in parallel with no locking and why `HashSink` is meaningful.
3. **Static and on-demand share one code path.** Any behavior that exists in only one mode is a
   bug, not a feature.
