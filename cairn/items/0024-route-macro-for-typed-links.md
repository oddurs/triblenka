---
id: 24
title: route! macro for typed links
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: routing
---

Generated from the page tree. Renaming or deleting a page breaks every call site at compile time. `tri check --strict` reports internal links written as bare strings, since those bypass the guarantee.
