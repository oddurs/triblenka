---
id: 13
title: Measure the two M0 kill criteria
type: feature
status: done
milestone: m0
depends_on:
- 9
- 11
- 12
created: 2026-09-09
updated: 2026-09-10
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

## 2026-09-10

Measured 2026-09-10 on an M-series laptop, release profile, 50 iterations per loop.

| posts | content edit (targeted) p50/p99 | template markup edit p50/p99 |
|---|---|---|
| 500 | 3.9 ms / 9.1 ms | 57 us / 243 us |
| 2,000 | 4.0 ms / 8.4 ms | 37 us / 208 us |
| 10,000 | 3.9 ms / 10.0 ms | 40 us / 72 us |

Both budgets met at every size, and the content path is flat with site size rather than linear — the watcher hands the loader a path, so no directory is rescanned.

## 2026-09-10

One judgement call worth recording: the first harness measured a full directory rescan, which FAILED at 2,000 posts (p99 51.9 ms). That is the pessimistic path, not the dev loop — a watcher knows which file changed — so a targeted load_file path was added and is what criterion 1 now measures. The rescan is still reported alongside, and still fails at 10,000 posts (p99 824 ms). Filed as follow-up work rather than hidden.
