---
id: 107
title: Store schema versioning
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: content
---

Every store records the hash of the collection schemas that wrote it, and a binary refuses to read a store written under a different schema, naming the collection and the change. This is the failure mode the two-artifact model must get right: a code deploy that changes a schema without migrating the store has to fail loudly at startup rather than serve half-broken pages.
