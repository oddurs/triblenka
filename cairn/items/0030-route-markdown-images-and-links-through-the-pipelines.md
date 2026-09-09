---
id: 30
title: Route markdown images and links through the pipelines
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: content
---

Images referenced from a body or from frontmatter get the same AVIF/WebP treatment as <Image>. Internal links are resolved against the route table, and a dead one fails `tri check --strict`.
