---
id: 101
title: Annotate elements with their source file in dev
type: feature
status: backlog
milestone: m1
depends_on:
- 44
created: 2026-09-09
updated: 2026-09-09
priority: p2
effort: s
area: devserver
---

Astro's compiler adds data-astro-source-file in dev, which is what powers click-to-source in its dev toolbar. Cheap, and it gives the error overlay and any future inspector a way to get from a rendered element back to the .tri file. Appendix B.6.
