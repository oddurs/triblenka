---
id: 91
title: Slot recovery with a nesting guard
type: feature
status: backlog
milestone: m2
depends_on:
- 57
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: islands
---

Server-rendered slot content is recovered from the DOM and handed to the island. Each candidate must be guarded by closest(tagName).isSameNode(host) or a nested island steals its parent's slots. Appendix B.1.
