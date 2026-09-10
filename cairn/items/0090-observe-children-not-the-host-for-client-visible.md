---
id: 90
title: Observe children, not the host, for client:visible
type: bug
status: backlog
milestone: m2
depends_on:
- 57
created: 2026-09-09
updated: 2026-09-10
priority: p1
effort: s
area: islands
---

The island host is display:contents and therefore has no box, so an IntersectionObserver on the host never fires. Observe el.children. Costs a day to rediscover; recorded here so we do not. Appendix B.1.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
