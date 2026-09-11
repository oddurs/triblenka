---
id: 131
title: Deleted source files leave their entries behind
type: bug
status: backlog
milestone: m1
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: content
---

## What happens

load_dir only adds and updates. Delete a markdown file and its store entry survives, so its rendered page is never removed and it keeps appearing in listings.

## What should happen

A source file that is gone should take its entry with it, and its output file with that.

## Reproduction

crates/tri-content/tests/review_probes.rs::known_gap_deleted_files_leave_their_entries_behind

## Fix

Collect the keys seen during a scan and delete the store keys that were not, which also needs the deploy plan to emit removals. Deferred out of M0 because it needs the collection model to say which keys a given scan is authoritative for.
