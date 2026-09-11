---
id: 127
title: Full content rescan does not scale
type: bug
status: backlog
milestone: m1
created: 2026-09-10
updated: 2026-09-11
priority: p1
effort: m
area: content
---

Measured at M0: rescanning every file costs p50 160 ms and p99 824 ms at 10,000 posts, because every file is read and hashed. The dev loop never takes this path — a watcher names the changed file, and that path is flat at ~4 ms — but a cold start and a CI build both do.

Fixes, in order of expected value: an mtime and size pre-filter before hashing, hashing in parallel with rayon, and persisting the previous scan's stat data alongside the digest.

- [ ] p99 under 50 ms for a no-op rescan of 10,000 entries

## 2026-09-11

Confirmed at 10,000 posts after the review: p50 197 ms, p99 941 ms for a no-op rescan.
