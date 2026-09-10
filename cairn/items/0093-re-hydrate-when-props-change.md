---
id: 93
title: Re-hydrate when props change
type: feature
status: backlog
milestone: m2
depends_on:
- 57
created: 2026-09-09
updated: 2026-09-10
priority: p2
effort: s
area: islands
---

observedAttributes on the props attribute, re-running hydration on change. This is the seam view transitions and server islands both use to update an island in place. Appendix B.1.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
