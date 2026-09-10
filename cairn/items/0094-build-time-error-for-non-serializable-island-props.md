---
id: 94
title: Build-time error for non-serializable island props
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-10
priority: p1
effort: s
area: islands
---

Astro needs a tagged-tuple encoding because JSON loses types; serde makes that unnecessary for us. What is worth copying is the error: a cyclic or non-serializable prop must fail at build time naming the component and the field. Appendix B.2.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
