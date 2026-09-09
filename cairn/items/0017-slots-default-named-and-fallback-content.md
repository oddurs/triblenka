---
id: 17
title: 'Slots: default, named, and fallback content'
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: compiler
---

Slots compile to `impl Fn(&mut Sink)` — no buffering, no intermediate allocation. `BoxedSlot` is the explicit opt-in for components that store or reorder their slots.

- [ ] Nothing is allocated for a slot that is rendered once in place
- [ ] A slot name that matches nothing is a compile error with a suggestion
