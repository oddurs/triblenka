---
id: 70
title: Cache tags, revalidation, and secrets
type: feature
status: backlog
milestone: m3
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: server
---

cx.cache().tag(...) maps onto native purging where a host has it and falls back to Cache-Control. secret! fails the build when the name is not declared in tri.toml — a missing production secret should surface before deploy, not at 3am.
