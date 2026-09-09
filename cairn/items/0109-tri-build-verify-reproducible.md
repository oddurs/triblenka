---
id: 109
title: tri build --verify-reproducible
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: build
---

Two clean builds in different temp directories, byte-compared, and run in CI under a different LANG, TZ and path — the three environment differences that catch locale collation, wall-clock time and absolute paths. The guarantee and its rules are in docs/concepts/determinism.md.
