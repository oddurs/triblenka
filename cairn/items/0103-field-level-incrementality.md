---
id: 103
title: Field-level incrementality
type: feature
status: done
milestone: m0
created: 2026-09-09
updated: 2026-09-11
priority: p1
effort: l
area: build
---

Key rendered_page on the set of entry fields a render actually read, not the entry digest, so a typo in a post body does not rebuild an index that renders only title and date. Static extraction from compiled templates covers direct access paths; a tracking wrapper records the rest; anything unresolvable degrades soundly to the whole entry.

An M0 experiment, not an M1 commitment: the open question is whether the bookkeeping costs less than the rebuilds it saves on small sites. Documented as an experiment in docs/concepts/incrementality.md — if the answer is no, that page changes and the feature ships above a size threshold or not at all.

## 2026-09-11

Experiment run 2026-09-11 at 10,000 posts (tri experiment-0103).

| measurement | value |
|---|---|
| index render, 10,000 items | 1.426 ms — what a miss saves |
| page render, untracked | 167 ns |
| page render, tracked | 250 ns (+83 ns, 49.7%) |
| diff + intersect decision | 42 ns |

Break-even is roughly 17,000 tracked renders per avoided index rebuild, so it pays immediately on any site with listing pages. Verdict: ships unconditionally, no size threshold. docs/concepts/incrementality.md updated from 'experiment' to the measured answer.

The experiment earned its keep by failing first: the initial implementation put recording in a Bindings wrapper, and the correctness check caught that a title edit did NOT mark the index for rebuild. A wrapper cannot follow the walk into a loop body — seq_item returns a borrowed child with nowhere to put a wrapper — so every read inside a {#for} went unrecorded. That is silent under-invalidation, which serves stale pages. Recording moved into the walker, where every index at every depth is visible.
