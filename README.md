<h1>Triblenka</h1>

[![CI](https://github.com/oddurs/triblenka/actions/workflows/ci.yml/badge.svg)](https://github.com/oddurs/triblenka/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**An Astro-shaped web framework for Rust.** Server-first HTML, zero JavaScript by default,
islands of interactivity, typed content collections, and a single static binary at the end.

> **Status: design stage.** Nothing is implemented yet. [`DESIGN.md`](DESIGN.md) is the
> architecture; [`docs/`](docs/) describes the v1 API we intend to build, written as real
> documentation so the API can be criticised before it is coded. Every snippet here is a
> proposal, not a promise.

```html
---
use triblenka::prelude::*;
let posts = content::blog().published().take(5);
---

<Layout title="Home">
  {#for post in posts}
    <article>
      <h2><a href={route!(blog::post(&post.slug))}>{ post.title }</a></h2>
      <p>{ post.description }</p>
    </article>
  {/for}

  <Newsletter client:visible />
</Layout>
```

That page ships **zero bytes of JavaScript** until the newsletter form scrolls into view.

## Why

- **Content edits do not invoke `rustc`.** `content/` is data, `src/` is code. Editing a blog
  post rebuilds in milliseconds. This constraint shapes the whole framework.
- **Content is typed.** Collections are Rust structs. Bad frontmatter is a build error with a
  line number, not a runtime surprise.
- **Links are typed.** `route!(blog::post(&slug))` won't compile if you delete or rename the page.
- **The output has no runtime dependencies.** A directory of files, one static binary, or one
  `wasm32-wasip2` component. No `node_modules` in your container.
- **Islands are real components** — [Leptos](https://leptos.dev), [Dioxus](https://dioxuslabs.com),
  or plain TypeScript — each hydrated on its own trigger, each loading only its own code.

## Docs

| | |
|---|---|
| [Why it exists](docs/concepts/why.md) | The thesis: a content deploy is not a code deploy |
| [Getting started](docs/getting-started.md) | Install, scaffold, first page |
| [Architecture](docs/concepts/architecture.md) | Three artifacts, one render walk |
| [Incrementality](docs/concepts/incrementality.md) | Field-level rebuilds, `tri impact`, `tri why` |
| [Determinism](docs/concepts/determinism.md) | Byte-reproducible builds |
| [Publishing](docs/guides/publishing.md) | Content deploys, migrations, rollback |
| [Errors](docs/concepts/errors.md) | The error-quality contract |
| [Components (`.tri`)](docs/guides/components.md) | The template language |
| [Routing](docs/guides/routing.md) | Pages, dynamic routes, endpoints |
| [Content collections](docs/guides/content-collections.md) | Typed content and loaders |
| [Islands](docs/concepts/islands.md) | Partial hydration, client directives |
| [Styling](docs/guides/styling.md) | Scoped CSS, global styles |
| [Images & assets](docs/guides/assets.md) | Optimization, fonts, static files |
| [Deployment](docs/guides/deployment.md) | Adapters and output modes |
| [Migrating from Astro](docs/guides/from-astro.md) | Concept-by-concept mapping |
| [CLI reference](docs/reference/cli.md) | `tri dev`, `tri build`, … |
| [Config reference](docs/reference/config.md) | `tri.toml` and `src/site.rs` |
| [Directives reference](docs/reference/directives.md) | `client:*`, `server:*`, `is:*` |
| [API reference](docs/reference/api.md) | `Component`, `Loader`, `Adapter`, `Integration` |
| [Architecture](DESIGN.md) | Why it is built this way |
| [Roadmap](ROADMAP.md) | Generated from `cairn`; `cairn next` for what is ready |

## Non-goals

Triblenka is not an SPA framework, not a React SSR host, and not chasing npm parity. It renders
HTML on the server and hydrates the small parts that need it. For app-shaped software, use
Leptos or Dioxus directly — Triblenka embeds them for islands rather than competing with them.

## Development

The project is at the design stage — there is very little code, and the most useful contribution is
an argument about `DESIGN.md`. If you want to work on it:

```sh
scripts/setup                            # wire hooks, verify the toolchain, run the checks
scripts/agent doctor                     # verify the environment
scripts/agent start feat/my-thing        # branch + worktree; prints the path to cd into
scripts/task check                       # exactly what CI runs
scripts/agent pr                         # check, push, open a pull request
```

`main` only advances through a merged pull request; that is enforced by branch protection and a
local hook rather than by discipline. See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow
and [CLAUDE.md](CLAUDE.md) for the contract agents work under.

Work is tracked with [cairn](https://github.com/oddurs/cairn) as Markdown files under `cairn/items/`
and rendered to [ROADMAP.md](ROADMAP.md). `cairn next` shows what is ready to start.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option — the
standard for the Rust ecosystem.
