---
id: 55
title: 'Island registry and #[island] macro'
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-10
priority: p0
effort: l
area: islands
---

linkme distributed slice keyed by a stable id (crate path + name, hashed). Generates the server render shim, the client hydrate shim, and the props serde glue. Props are one type compiled twice — a shape mismatch is a compile error, not a runtime surprise.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
