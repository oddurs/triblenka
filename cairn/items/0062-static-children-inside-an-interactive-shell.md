---
id: 62
title: Static children inside an interactive shell
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-10
priority: p1
effort: m
area: islands
---

Children of an island render on the server and are handed over as an opaque fragment hydration must not touch. Most frameworks quietly lose this at the second level of nesting, which is where the islands claim stops being true.

## 2026-09-10

Now rung 3 of the interactivity ladder (DESIGN §9), and sequenced last within M2 rather than first. Frames and resumable handlers should absorb most of what would previously have been an island, so scope this to genuinely continuous local state: canvas, editor, map, data grid.
