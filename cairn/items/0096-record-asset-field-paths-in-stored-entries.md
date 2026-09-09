---
id: 96
title: Record asset field paths in stored entries
type: feature
status: backlog
milestone: m1
depends_on:
- 11
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: content
---

Astro prefixes image fields during schema parsing, strips the prefix when storing, and records the path of each such field so read-time resolution rewrites only those locations instead of traversing or cloning the entry. This is how markdown image references reach the asset pipeline without walking every entry on every read. Appendix B.7.
