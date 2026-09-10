---
id: 114
title: Warn when frames are far from readers
type: feature
status: backlog
milestone: m3
depends_on:
- 112
created: 2026-09-10
updated: 2026-09-10
priority: p2
effort: s
area: adapters
---

Rung 1 assumes the origin is near the reader; at the edge the swap is invisible, from a single region to another continent it is not. tri check should say so based on the adapter's deployment shape rather than letting the assumption fail silently. The honest caveat is documented in docs/concepts/interactivity.md.
