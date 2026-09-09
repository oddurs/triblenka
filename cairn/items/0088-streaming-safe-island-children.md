---
id: 88
title: Streaming-safe island children
type: feature
status: backlog
milestone: m2
depends_on:
- 57
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: islands
---

connectedCallback can fire before an island's server-rendered children are parsed when the response streams. Astro emits an await-children attribute plus a trailing marker comment, watches with a MutationObserver, and falls back to DOMContentLoaded in case the marker was stripped. Without it a streamed island hydrates against a partial subtree. Appendix B.1.
