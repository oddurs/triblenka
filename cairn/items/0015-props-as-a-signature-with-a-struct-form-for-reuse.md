---
id: 15
title: Props as a signature, with a struct form for reuse
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: compiler
---

`#[props] fn (title: &str, href: Route, featured: bool = false);` binds parameters as plain template variables. Struct form retained and spreadable (`<Card ..card_props />`) for prop sets shared across components. Reversed from the struct-only first draft in design review.

- [ ] Missing required prop errors at the tag, not the definition
- [ ] Defaults, Option, and borrowed params all work
