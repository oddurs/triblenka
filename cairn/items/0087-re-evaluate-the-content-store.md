---
id: 87
title: Re-evaluate the content store
type: chore
status: backlog
milestone: m4
created: 2026-09-09
updated: 2026-09-09
priority: p3
effort: s
area: content
---

redb was chosen for v1 because Turso — the Rust SQLite rewrite — is beta and explicitly not at SQLite-level reliability. Behind the Store trait. Revisit when live collections and full-text search want real SQL, or when Turso stabilizes.
