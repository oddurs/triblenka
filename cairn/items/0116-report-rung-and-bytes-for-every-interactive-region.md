---
id: 116
title: Report rung and bytes for every interactive region
type: feature
status: backlog
milestone: m2
depends_on:
- 115
created: 2026-09-10
updated: 2026-09-10
priority: p0
effort: m
area: build
---

Inference is an ergonomic bet: implicit behaviour delights when right and infuriates when wrong. It survives only if every build makes the implicit visible. tri build --stats prints route, rung, js, wasm and the region name; budgets fail the build rather than warn. Without this item, inference should not ship.
