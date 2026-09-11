---
id: 133
title: Attribute context is lost inside blocks and after a literal gt
type: bug
status: backlog
milestone: m1
created: 2026-09-11
updated: 2026-09-11
priority: p0
effort: m
area: compiler
---

## What happens

The tag-context tracking added in #10 is a local of nodes() and a bare check for < and >, so it fails on two idiomatic shapes:

- `<a title="{#if c}{ x }{/if}">` — every block body starts a fresh nodes() with in_tag = false, so the interpolation is treated as text context.
- `<a data-r="a > b" title="{ x }">` — a literal > inside an earlier attribute value ends tag context early.

Both render `<a title="a" onload="evil()">` from a value of `a" onload="evil()`, which is exactly the injection #10 set out to close.

A third shape is unsafe even with correct context: `<a title={ x }>` is an unquoted attribute, where escaping quotes is not sufficient — whitespace ends the value.

## Fix

Move the scan state onto the parser so it survives recursion, and track attribute quoting rather than guessing from < and >. Reject interpolation into an unquoted attribute at parse time with an actionable error, rather than trying to escape enough characters to make it safe.

## Acceptance criteria

- [ ] Interpolation inside a block inside an attribute keeps attribute escaping
- [ ] A literal > or < inside an attribute value does not change context
- [ ] Interpolation into an unquoted attribute is a parse error naming the attribute
