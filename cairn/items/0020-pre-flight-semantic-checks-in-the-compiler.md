---
id: 20
title: Pre-flight semantic checks in the compiler
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: compiler
---

Catch the ~80% of template errors that need not reach rustc: unknown component (with spelling suggestion and the list in scope), missing required prop, slot-name typo, unknown directive, void-element misuse, lowercase unknown tags without a hyphen.
