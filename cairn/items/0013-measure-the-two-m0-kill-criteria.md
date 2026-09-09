---
id: 13
title: Measure the two M0 kill criteria
type: feature
status: planned
milestone: m0
depends_on:
- 9
- 11
- 12
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: build
---

## Problem

The design's central claims are latency claims. They need a harness before anyone argues about them.

## Proposal

A criterion-driven harness measuring, on the fixture: (a) edit one post body to rendered HTML on disk; (b) edit markup in one template to a swapped descriptor and re-rendered page. Report p50 and p99, record the machine.

## Acceptance criteria

- [ ] Content-only rebuild p99 under 50 ms
- [ ] Structure-only template edit p99 under 100 ms
- [ ] Numbers reproduced on a second machine before the go/no-go
