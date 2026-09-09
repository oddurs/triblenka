---
id: 60
title: Wasm build, split, and optimize
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: xl
area: islands
---

One island binary per site, then wasm-bindgen, then wasm-split into a shared core plus a lazily fetched chunk per island, then wasm-opt -Oz. One wasm module per island duplicates the runtime and is a non-starter.

- [ ] A page with one visible island fetches core plus that chunk and nothing else
