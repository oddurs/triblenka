---
id: 68
title: Opt-in out-of-order streaming
type: feature
status: backlog
milestone: m3
created: 2026-09-09
updated: 2026-09-10
priority: p2
effort: l
area: server
---

One response, templates appended as islands resolve. Opt-in, not default, because of the trilemma in DESIGN §9.6: out-of-order streaming, strict CSP, and full-page caching — pick two.

## 2026-09-10

The WASI component adapter is the forward-looking half of this item: a site as a composable wasm32-wasip2 component rather than a container image. Rust is the best-positioned language for the component model and hosting is still catching up (DESIGN Appendix C.1).
