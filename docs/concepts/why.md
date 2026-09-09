# Why Triblenka exists

> Design-stage docs.

## The eight-minute typo

You misspell a word in a blog post. You fix it. Then:

```
   push  →  CI wakes up  →  install 400 packages  →  rebuild 10,000 pages  →  deploy
                                    8 minutes later
```

Every static site generator works this way, and everyone has agreed to pretend it is fine. It is
not fine — it is the single defining frustration of running a content site at any real size, and it
gets worse linearly with how much content you have and how often you publish.

It happens for one reason: **in a JavaScript framework, content and code are the same artifact.**
Your markdown is imported into the module graph, bundled, and compiled into the output. Touching
content means recompiling, because the compiler cannot tell the difference between the two.

## The thesis

**A content deploy should not be a code deploy.**

Triblenka splits them at the only place the split can be made honestly — the compile boundary:

```
src/**       code     compiled into a binary        changes when engineers change something
content/**   data     loaded into a typed store     changes when writers change something
```

A site is a **program plus a store**, not a pile of HTML. The program is a stable artifact that
content changes never invalidate. Content updates are data updates: a write to the store, and the
affected pages re-render in milliseconds.

That single decision cascades:

| | Everyone else | Triblenka |
|---|---|---|
| Fix a typo | rebuild the world, 2–10 min | re-render the affected pages, 5–20 ms |
| CMS webhook | trigger a full build | write the store, re-render, purge |
| Deploy unit | the whole site | the pages that actually changed |
| Prerendering | an architecture you commit to | a cache policy you set per route |
| Content and code | one artifact, one risk profile | two artifacts, two cadences |

## Prerendering becomes a cache policy

When rendering a page takes ~200µs, you stop prerendering for *speed*. You prerender for CDN
economics — to put bytes near the reader and to keep your origin idle. That is a different kind of
decision, and it is one you can change per route, in one line, without restructuring anything:

```rust
#[prerender]         // bake it: this page is read a million times and changes weekly
#[prerender(false)]  // render on demand: this page is personalized
```

Static and on-demand rendering run **the same code path**, so there is no second implementation to
drift and no class of bug that only appears in one mode. In most frameworks this choice is
architectural and expensive to revisit. Here it is a cache tier.

## What you actually get

- **Publishing latency measured in milliseconds**, not build minutes.
- **Deploys that are the size of the change.** `tri impact` tells you exactly which URLs moved,
  which is also exactly what to purge from your CDN.
- **Content edits that never break the build in a new way**, because the code did not change.
- **A build that can explain itself** — see [Incrementality and provenance](incrementality.md).
- **Byte-reproducible output**, verifiable on any machine — see [Determinism](determinism.md).
- **No Node in your CI, your container, or your supply chain.** Not as a purity argument: as one
  fewer thing that breaks at 3am and one fewer ecosystem to audit.

## What it costs you

Stated plainly, because a framework that hides its costs is lying to you:

- **Template edits invoke `rustc`.** Markup, attributes, and nesting hot-reload in milliseconds
  through the descriptor mechanism, but changing an *expression* or the frontmatter means an
  incremental rebuild — typically 0.4–2 s. Astro is faster at this. We are dramatically faster at
  content edits, which for most sites are 80–90% of all edits.
- **Schema changes are code changes.** Adding a field to a collection means a rebuild and a code
  deploy. This is the correct tradeoff — a schema *is* an interface — but it is a real one.
- **The component ecosystem is small.** You are writing your own UI, using headless CSS patterns,
  or reaching for a `client:only` island. There is no rescue kit of 400 npm components.

## Who this is for

Sites where **content changes far more often than code**: documentation, knowledge bases,
changelogs, editorial and news, large marketing sites, compliance and archival publications. Teams
who have been burned by rebuild-the-world pipelines, and teams who already run Rust and would
rather not maintain a second toolchain to publish a docs site.

## Who this is not for

Applications. If your product is a dashboard, an editor, or anything whose defining characteristic
is client-side state, use [Leptos](https://leptos.dev) or [Dioxus](https://dioxuslabs.com) —
Triblenka embeds them for islands rather than competing with them. If your team is happiest in
JavaScript and your build times do not hurt, Astro is an excellent framework and you should keep
using it.
