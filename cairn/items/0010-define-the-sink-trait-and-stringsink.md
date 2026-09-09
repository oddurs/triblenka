---
id: 10
title: Define the Sink trait and StringSink
type: feature
status: planned
milestone: m0
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: s
area: runtime
---

## Problem

Rendering writes depth-first into a sink; everything else is built on that contract.

## Proposal

`Sink` with `raw`, `escaped`, `defer`. `StringSink` sized from a cached hint so a re-render allocates once. `Render` trait with a Display blanket impl, and the `Html` newtype for raw output.

## Acceptance criteria

- [ ] Escaping is correct for text and attribute contexts, with a fuzz test
- [ ] One allocation per page render on the second and later builds
