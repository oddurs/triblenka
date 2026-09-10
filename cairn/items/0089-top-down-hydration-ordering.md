---
id: 89
title: Top-down hydration ordering
type: feature
status: backlog
milestone: m2
depends_on:
- 57
created: 2026-09-09
updated: 2026-09-10
priority: p0
effort: m
area: islands
---

A nested island must wait for its ancestor: check for an un-hydrated parent host and re-run on the parent's hydrate event. Bottom-up hydration lets a parent re-create and discard a child that already mounted. Appendix B.1.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
