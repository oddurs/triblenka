---
id: 100
title: waitUntil and prerendered error pages in the adapter contract
type: feature
status: backlog
milestone: m3
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: adapters
---

Two fields missing from the Features matrix. waitUntil keeps background work alive after the response is sent and is required for write-behind cache updates on edge platforms — an adapter without it must not be able to claim that capability. prerenderedErrorPageFetch lets a dynamic route that 404s in a hybrid build still serve the prerendered 404, which the server cannot always read from disk. Appendix B.8.
