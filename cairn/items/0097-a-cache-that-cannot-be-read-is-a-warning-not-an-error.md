---
id: 97
title: A cache that cannot be read is a warning, not an error
type: feature
status: backlog
milestone: m1
depends_on:
- 38
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: s
area: build
---

Corrupt, truncated, or version-mismatched .tri/cache must warn loudly and fall back to a cold build. Failing the build on a bad cache turns a local annoyance into a broken CI pipeline. Appendix B.7.
