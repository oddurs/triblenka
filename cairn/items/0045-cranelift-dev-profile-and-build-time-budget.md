---
id: 45
title: Cranelift dev profile and build-time budget
type: chore
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: s
area: devserver
---

dev-fast profile: opt-level 0, thin debug, incremental, cranelift backend where available, islands in a separate crate so they do not rebuild with the site. Track the expression-edit loop as a regression test — it is the loop the design admits is slow.
