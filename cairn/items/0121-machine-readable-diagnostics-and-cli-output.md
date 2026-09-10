---
id: 121
title: Machine-readable diagnostics and CLI output
type: feature
status: backlog
milestone: m1
created: 2026-09-10
updated: 2026-09-10
priority: p1
effort: m
area: tooling
---

Every diagnostic emittable as JSON; tri explain --json, tri impact --json, tri build --stats --json; a machine-readable project schema. In 2026 a large share of the code in any repository is written by an agent and every framework still has exactly one designed user. The prerequisites — determinism, provenance, typed content — are already in the design for other reasons, so finishing the thought is cheap. See DESIGN Appendix C.5.
