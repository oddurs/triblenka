---
id: 62
title: Static children inside an interactive shell
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: islands
---

Children of an island render on the server and are handed over as an opaque fragment hydration must not touch. Most frameworks quietly lose this at the second level of nesting, which is where the islands claim stops being true.
