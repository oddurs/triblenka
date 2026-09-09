---
id: 57
title: Hydration shim and tri-island custom element
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: islands
---

~1.1 KB inline: arm the trigger, fetch the chunks, deserialize props, hydrate in place. A failed island must leave the server-rendered markup intact — degradation to static content, never a blank box.

## 2026-09-09

Six correctness mechanisms lifted from astro-island.ts (Appendix B.1) now have their own items: streaming-safe children, top-down hydration, observer-on-children, slot recovery guard, hydration-error event, re-hydrate on props change. None were in the first draft.
