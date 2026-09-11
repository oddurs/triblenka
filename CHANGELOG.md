# Changelog

All notable changes to this project are recorded here, in the format of
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). This project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html); until 0.1.0 it has no public API and
anything may change.

## [Unreleased]

### Added

- The architecture, recorded in `DESIGN.md`, with the design review and the source-verified Astro
  parity audit as appendices.
- Documentation of the intended v1 API under `docs/`, written before implementation so the design
  can be criticised while it is still cheap to change.
- The roadmap, tracked as [cairn](https://github.com/oddurs/cairn) items and rendered to
  `ROADMAP.md`.
- A cargo workspace, the `tri` binary, and the branch/worktree/pull-request workflow in `scripts/`.
- The interactivity ladder: four rungs — platform, server frame, resumable handler, island — with
  the rung inferred by the compiler. Documented in `docs/concepts/interactivity.md` and `DESIGN.md`
  §9; the wider survey it came from is `DESIGN.md` Appendix C.
- Contracts: `deny(javascript)`, `require(no_js_fallback)`, `deny(external_requests)` and the
  accessibility checks, enforced as build errors rather than lints
  (`docs/concepts/contracts.md`).

### Added — M0

- `tri-core`: the `Sink` trait, `StringSink`, HTML escaping, the `Render`/`Html` types, and the
  template descriptor with its walker and the `is_hot_swappable_with` check that decides whether a
  template edit needs `rustc`.
- `tri-compiler`: a span-carrying `.tri` parser (text, interpolation, `{#if}`/`{:else}`, `{#for}`,
  frontmatter) with actionable errors, descriptor lowering with expression interning, and a Rust
  emitter.
- `tri-content`: blake3 digests, a redb-backed store whose `set` short-circuits on an unchanged
  digest, a markdown loader with frontmatter validation, and a targeted single-file load path.
- `tri bench-m0`: the kill-criteria harness, and a deterministic fixture generator.

### Changed

- M2 is no longer "Islands". It is the ladder, sequenced frames-first; islands become rung 3 and
  land last. Building islands first would have produced an Astro port.

### Fixed

- Interpolation inside a tag is escaped for attribute context, so a value containing a quote can no
  longer close the attribute it sits in. Text context is unchanged, and ampersands are not
  double-escaped.
- Content keys are derived from a document's path rather than its file stem; two files with the
  same name in different directories no longer collapse into one entry, silently losing one.
- The M0 harness asserts that each iteration did real work, and takes enough samples for its p99 to
  be a percentile rather than the largest of fifty.

- Documentation drift from the ladder change: the reference pages had no `#[frame]`, `frame!()`,
  `#[handler]`, `rung:*` or contract entries, three guides still taught islands as the only
  interactivity story, and two risk rows plus the M1 roadmap prose in `DESIGN.md` were stale.

### Removed

- The client-side router, the Node/Deno sidecar for JavaScript SSR, and the second Rust island
  renderer. Each is recorded as a dropped cairn item with the reasoning rather than deleted.
