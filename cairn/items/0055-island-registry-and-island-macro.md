---
id: 55
title: 'Island registry and #[island] macro'
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: l
area: islands
---

linkme distributed slice keyed by a stable id (crate path + name, hashed). Generates the server render shim, the client hydrate shim, and the props serde glue. Props are one type compiled twice — a shape mismatch is a compile error, not a runtime surprise.
