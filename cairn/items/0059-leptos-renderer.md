---
id: 59
title: Leptos renderer
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-10
priority: p1
effort: l
area: islands
---

For islands that share logic with the server, are compute-bound, or are large enough that wasm's size curve wins — past roughly 4-6 KB of TypeScript.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
