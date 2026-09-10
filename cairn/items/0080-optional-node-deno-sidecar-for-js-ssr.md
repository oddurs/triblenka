---
id: 80
title: Optional Node/Deno sidecar for JS SSR
type: feature
status: dropped
milestone: m5
created: 2026-09-09
updated: 2026-09-10
priority: p3
effort: l
area: islands
---

Off by default, absent from the shipped binary, clearly marked the slow path. Embedding V8 or QuickJS in the default build was considered and rejected: it doubles binary size, breaks wasm targets, and re-imports the ecosystem we left.

## 2026-09-10

Dropped 2026-09-10. Server-rendering JavaScript components is an admission that the thesis failed, and a permanent support burden. client:only covers the real need without putting a JS engine in the build. Recorded rather than deleted so the decision is legible.
