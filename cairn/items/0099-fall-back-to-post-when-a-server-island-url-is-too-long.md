---
id: 99
title: Fall back to POST when a server island URL is too long
type: feature
status: backlog
milestone: m3
depends_on:
- 67
created: 2026-09-09
updated: 2026-09-09
priority: p2
effort: s
area: server
---

Encrypted props, the encrypted component export and slots travel as query parameters; Astro checks the URL stays under 2048 chars. Ours needs the same check and a POST fallback rather than a mysterious truncation. Appendix B.3.
