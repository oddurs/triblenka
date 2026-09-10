---
id: 123
title: Remote build cache keyed on input digests
type: chore
status: backlog
milestone: m4
created: 2026-09-10
updated: 2026-09-10
priority: p2
effort: l
area: build
---

Determinism makes a shared content-addressed cache sound: inputs fully determine outputs, so a cache hit is provably the same bytes. Bazel-style, and free in the sense that the property was already committed to for correctness checking.
