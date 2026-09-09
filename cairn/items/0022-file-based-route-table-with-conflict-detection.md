---
id: 22
title: File-based route table with conflict detection
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: routing
---

src/pages/** to a generated route table. Static beats dynamic beats catch-all, longest path first. Two routes that can match one URL is a build error naming both files — not a race.

## 2026-09-09

Astro's comparator (core/routing/priority.ts) is segment-by-segment, not whole-route, and handles partially dynamic segments (game-[title] outranks [title]) and an index-counts-as-one-deeper rule. Two divergences decided in DESIGN Appendix B.5: tiebreak byte-wise rather than localeCompare (locale-dependent output breaks byte-comparable builds), and only identical patterns are an error — ambiguity is ordered and explained by tri explain, not rejected.
