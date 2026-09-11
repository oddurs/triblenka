---
id: 132
title: Collection membership changes must invalidate readers
type: feature
status: backlog
milestone: m1
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: build
---

Field-level tracking answers 'did a field this page read change'. It does not answer 'was an entry added or removed', and those are different questions:

- A page that renders an empty list records the sequence expression but none of the body's expressions, because the walk never entered the body.
- Adding a post changes no existing entry's fields, so a field diff finds nothing to invalidate.

The sequence expression is marked on every render, so the mechanism exists — what is missing is a collection-level digest that changes when membership does, fed into the same intersect test. Without it, a new post appears on its own page but not in the index.

Found while running experiment 0103; the experiment measured edits to existing entries only.

- [ ] Adding an entry rebuilds every page that reads the collection
- [ ] Removing an entry does the same, and removes its own page
