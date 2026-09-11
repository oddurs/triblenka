---
id: 128
title: Cold build is slower than the design target
type: bug
status: backlog
milestone: m1
created: 2026-09-10
updated: 2026-09-11
priority: p1
effort: m
area: build
---

M0 measured a cold build of 500 posts at ~2.2 s, or ~4.3 ms per page, which extrapolates to roughly 43 s for the 10,000-page target in DESIGN §11 — well over the stated 8 s.

Two known causes, neither architectural: every entry is written in its own redb transaction, and rendering is single-threaded. Batch the writes into one transaction per load, and put rayon across routes as §11 already specifies.

- [ ] Cold build of 10,000 pages under 8 s

## 2026-09-11

Measured directly rather than extrapolated: a cold build of 10,000 posts takes 43.96 s against the 8 s target in DESIGN §11.
