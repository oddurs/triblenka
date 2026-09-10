---
id: 119
title: Accessibility contracts
type: feature
status: backlog
milestone: m1
depends_on:
- 118
created: 2026-09-10
updated: 2026-09-10
priority: p2
effort: m
area: contracts
---

require(alt_text), require(heading_order), require(labels), require(lang) as build-time checks over rendered HTML. Cheap because the build already produces every page and knows its provenance — a pass over output that exists rather than a browser harness.
