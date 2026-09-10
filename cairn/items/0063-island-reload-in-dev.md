---
id: 63
title: Island reload in dev
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-10
priority: p2
effort: m
area: islands
---

Rebuild only the changed chunk and re-hydrate affected hosts. Evaluate Dioxus subsecond hot-patching for the Rust renderers.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
