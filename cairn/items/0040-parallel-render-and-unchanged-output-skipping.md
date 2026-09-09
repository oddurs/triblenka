---
id: 40
title: Parallel render and unchanged-output skipping
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: build
---

rayon across routes — the store is read-only during render, so there is no contention. Compare against HashSink results and skip unchanged writes, which keeps incremental deploys and CDN invalidation surfaces small.
