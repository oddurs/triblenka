---
id: 91
title: Slot recovery with a nesting guard
type: feature
status: backlog
milestone: m2
depends_on:
- 57
created: 2026-09-09
updated: 2026-09-10
priority: p1
effort: m
area: islands
---

Server-rendered slot content is recovered from the DOM and handed to the island. Each candidate must be guarded by closest(tagName).isSameNode(host) or a nested island steals its parent's slots. Appendix B.1.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
