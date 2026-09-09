---
id: 92
title: Hydration error event and graceful degradation
type: feature
status: backlog
milestone: m2
depends_on:
- 57
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: s
area: islands
---

Dispatch a cancelable tri:hydration-error before logging so an app can suppress or report it, and catch chunk-load failures inside the loader so directives never leak rejections. A failed island leaves the server-rendered markup in place. Appendix B.1.
