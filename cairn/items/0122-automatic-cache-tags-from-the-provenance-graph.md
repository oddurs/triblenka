---
id: 122
title: Automatic cache tags from the provenance graph
type: feature
status: backlog
milestone: m2
created: 2026-09-10
updated: 2026-09-10
priority: p1
effort: l
area: build
---

Everyone else makes you write revalidateTag("post-123") by hand and debug the one you forgot. We already know which content fields each fragment read, so the cache tags are the dependency edges — derived, not typed. One graph pays for incremental rebuilds, deploy plans and cache invalidation. DESIGN Appendix C.4.
