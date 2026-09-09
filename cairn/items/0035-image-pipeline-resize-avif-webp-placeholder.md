---
id: 35
title: 'Image pipeline: resize, AVIF, WebP, placeholder'
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: l
area: assets
---

image + ravif + oxipng. Emits <picture> with srcset, intrinsic width and height so nothing shifts, and a tiny DCT blur placeholder. `alt` is required at compile time, not linted.
