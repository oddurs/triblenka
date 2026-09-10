---
id: 112
title: Server frames and the swap protocol
type: feature
status: backlog
milestone: m2
created: 2026-09-10
updated: 2026-09-10
priority: p0
effort: l
area: interactivity
---

## Problem

Filters, pagination, sorting, search, forms, cart badges — the overwhelming majority of what content sites call interactivity. Every framework answers with client-side state because their server round trip is too slow to use. Ours is ~200µs.

## Proposal

A frame is a declared region re-rendered on the server and swapped into the DOM. #[frame(id)] on a component, frame!() for its URL, one ~2 KB script for the whole page regardless of frame count. htmx's analysis with a server fast enough to make it invisible.

## Acceptance criteria

- [ ] A page with any number of frames ships one ~2 KB script and no wasm
- [ ] A frame render is served in under 1 ms server-side on the fixture site
- [ ] Swapping preserves focus and scroll position, and announces to assistive technology
