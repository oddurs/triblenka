---
id: 9
title: Emit a template descriptor and a dev-mode walker
type: feature
status: planned
milestone: m0
depends_on:
- 8
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: compiler
---

## Problem

This is the mechanism the whole authoring loop rests on (DESIGN §5.1). If it does not work, kill criterion 2 fails.

## Proposal

Emit, next to the render fn, a `Template` descriptor — static chunks, attributes, slots, expression slots indexed into a table of compiled thunks — plus a walker used in dev. Release monomorphizes; dev walks.

## Acceptance criteria

- [ ] Both modes produce byte-identical HTML across the fixture set (property test)
- [ ] A descriptor can be replaced at runtime without touching the thunk table
- [ ] Dev-mode overhead is under 10% of render time on the 500-post fixture
