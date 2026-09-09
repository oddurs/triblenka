---
id: 66
title: Endpoints as .rs files
type: feature
status: backlog
milestone: m3
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: server
---

Exported get/post/put/patch/delete/all become methods; anything else returns 405 with a correct Allow header. Prerendered endpoints run at build time and write a file.
