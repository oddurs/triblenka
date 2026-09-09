# Incrementality and provenance

> Design-stage docs.

Most build tools know *that* something changed. Triblenka is built to know **what changed, what it
affected, and why** — because rendering is a deterministic walk over typed data, the build graph can
be complete rather than approximate.

## Three levels of granularity

```
file level    "content/blog/hello.md changed"        →  rebuild every page that reads that file
entry level   "entry blog/hello changed"             →  rebuild pages that read that entry
field level   "entry blog/hello, field `body`"       →  rebuild pages that read `body`
```

Everyone stops at the first or second. We go to the third, and it matters more than it sounds: an
index page renders `title`, `date`, and `slug`. Fix a typo in a post's *body* and the index does not
change by one byte — but every file-level tool rebuilds it anyway, along with every tag page, every
paginated listing, and the feed.

## How field tracking works

Entries reach templates through a tracking wrapper that records which fields a render actually read:

```rust
// conceptually
let post: Tracked<Post> = store.entry("blog/hello");
sink.escaped(post.title);       // records: blog/hello :: title
```

The memo table then keys `rendered_page` on the **field set** it observed, not on the entry digest.
Re-running a query starts by asking whether any *observed* field moved.

Two sources feed the tracker:

1. **Static extraction.** The compiler already parses every template expression, so direct access
   paths (`post.title`, `post.author.name`) are known without running anything.
2. **Runtime observation.** Anything the compiler cannot see statically — a helper, a method, a
   closure — is recorded as it happens.

When neither can resolve an access precisely, the tracker **degrades soundly to the whole entry**.
Being conservative costs a rebuild; being clever and wrong costs a stale page, so the failure
direction is not negotiable.

> **Status:** field-level tracking is an M0 *experiment*, not an M1 commitment. The open question is
> whether the bookkeeping costs less than the rebuilds it saves on small sites. If it does not, it
> ships only above a size threshold, or not at all. The docs will say which.

## `tri impact` — what did this change touch?

```
$ tri impact content/blog/hello.md
entry  blog/hello        fields changed: body
pages  2
  /blog/hello/           22.7 KB  (was 22.6 KB)
  /sitemap.xml            4.1 KB  (unchanged bytes, timestamp only — skipped)
assets 0
purge  https://example.com/blog/hello/
```

This is also the **deploy plan**. The set of changed outputs is exactly the set of files to upload
and exactly the set of URLs to purge from a CDN. Deploys become the size of the change rather than
the size of the site.

## `tri why` — where did this byte come from?

```
$ tri why dist/blog/hello/index.html:1204
  <h2> at src/components/card.tri:9
    rendering  post.title
    from       content/blog/hello.md:3
    via        src/pages/blog/[slug].tri  →  Base  →  Card
```

Provenance runs in both directions over the same graph. It is the debugging tool that content
systems never have: when a page shows the wrong thing, the question is always "which data, through
which component" and the answer is normally an afternoon of `println!`.

## The memo table

Queries, each keyed on the digests of what it read:

```
file_text(path)              collection_entries(name)
parsed_template(path)        rendered_page(route, params)
component_deps(id)           page_assets(route)
route_table()                emitted_file(path)
```

Inputs are tiered by volatility — `content/**` changes constantly, `src/**` occasionally, config and
vendored crates almost never — so a content edit invalidates a handful of `rendered_page` nodes and
nothing above them. The table persists to `.tri/cache/graph.bin` and is safe to restore in CI.

The graph is hand-rolled rather than built on `salsa`, deliberately and reversibly — see
`DESIGN.md` §11 and Appendix A, motion 5.

## Why you can trust it

Hand-rolled incremental builds are where stale-output bugs live, and a stale output is worse than a
slow build because nobody notices. So the correctness of the graph is not asserted, it is checked:

```sh
tri build --verify-incremental
```

runs the incremental build, then a clean build, and byte-compares every output. It runs on every CI
build of the framework itself and is available in yours. If the two disagree, the incremental build
is wrong — by definition, no argument required.
