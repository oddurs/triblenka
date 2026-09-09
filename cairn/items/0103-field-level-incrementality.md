---
id: 103
title: Field-level incrementality
type: feature
status: backlog
milestone: m0
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: l
area: build
---

Key rendered_page on the set of entry fields a render actually read, not the entry digest, so a typo in a post body does not rebuild an index that renders only title and date. Static extraction from compiled templates covers direct access paths; a tracking wrapper records the rest; anything unresolvable degrades soundly to the whole entry.

An M0 experiment, not an M1 commitment: the open question is whether the bookkeeping costs less than the rebuilds it saves on small sites. Documented as an experiment in docs/concepts/incrementality.md — if the answer is no, that page changes and the feature ships above a size threshold or not at all.
