---
id: 39
title: tri build --verify-incremental
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: build
---

The safety net that makes the hand-rolled graph defensible. Incremental build, then a clean build, then byte-compare every output. Runs on every CI build of the framework itself.

- [ ] A deliberately introduced stale-output bug is caught by this check

## 2026-09-09

Byte-comparable output is only meaningful if every ordering in the build is deterministic. Audit for locale- or hash-iteration-dependent ordering — Astro's route tiebreak is a live example of getting this wrong.
