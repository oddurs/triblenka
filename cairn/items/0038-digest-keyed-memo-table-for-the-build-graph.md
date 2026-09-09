---
id: 38
title: Digest-keyed memo table for the build graph
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: l
area: build
---

Hand-rolled, not salsa — see DESIGN §11 and Appendix A motion 5. Queries record the input digests they read; the table persists to .tri/cache/graph.bin. Query boundaries stay salsa-shaped so adoption remains a drop-in at M4.
