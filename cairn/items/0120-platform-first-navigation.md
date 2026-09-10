---
id: 120
title: Platform-first navigation
type: feature
status: backlog
milestone: m1
created: 2026-09-10
updated: 2026-09-10
priority: p2
effort: m
area: routing
---

Cross-document view transitions plus speculation rules give SPA feel with zero framework JavaScript. We do not ship a client router — the platform has one, and most frameworks cannot take this position because their state lives in the JS heap. Ours does not. Emitted from route metadata, so authors declare intent rather than wiring a router.
