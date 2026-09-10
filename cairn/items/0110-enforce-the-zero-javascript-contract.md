---
id: 110
title: Enforce the zero-JavaScript contract
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-10
priority: p1
effort: m
area: islands
---

Let a page or a whole site declare that no JavaScript may reach it, and fail the build naming the component that would have emitted some. Zero JS by default is a slogan until something enforces it; progressive enhancement as a compiler-checked property is the version worth having.

## 2026-09-10

Folded into the contract family (see the contract declarations item). deny(javascript) is one member alongside require(no_js_fallback), deny(external_requests) and the accessibility contracts, sharing one declaration syntax and one enforcement pass.
