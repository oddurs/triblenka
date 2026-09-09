---
id: 11
title: Load markdown into a digest-keyed redb store
type: feature
status: planned
milestone: m0
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: content
---

## Problem

Kill criterion 1 depends on content never reaching rustc: load, digest, store, query.

## Proposal

`Glob` loader over markdown with YAML frontmatter, comrak for bodies, serde into a collection type, blake3 digests, redb tables for entries and meta. `Changed::No` on an unchanged digest.

## Acceptance criteria

- [ ] Re-loading unchanged content dirties nothing
- [ ] A malformed frontmatter field reports file, line, field, and expected type
- [ ] 500 posts load cold in under 200 ms
