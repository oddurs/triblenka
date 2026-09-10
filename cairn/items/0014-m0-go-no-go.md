---
id: 14
title: M0 go/no-go
type: chore
status: done
milestone: m0
depends_on:
- 13
created: 2026-09-09
updated: 2026-09-10
priority: p0
effort: s
area: project
---

Read the harness output against the two kill criteria and write the verdict into this item as a note. If either criterion fails, the honest options are: drop descriptor swapping and accept Zola-with-types, or stop. Do not start M1 before this is closed.

## 2026-09-10

GO, 2026-09-10.

Criterion 1 (content-only rebuild under 50 ms): PASS — p99 10.0 ms at 10,000 posts, flat with site size.
Criterion 2 (structure-only template edit under 100 ms): PASS — p99 72 us at 10,000 posts, roughly 1,400x under budget.

The descriptor mechanism is the load-bearing claim and it holds: a markup-only edit re-parses, lowers, verifies the expression table is unchanged, and re-renders in tens of microseconds, with no rustc involved. Decision B (content is data, never code) also holds — a content edit never touches the compiler.

Two things the spike did not prove, both now filed: the codegen path is not compiled end to end (no build.rs), and the full-rescan path does not scale. Neither is load-bearing for the two criteria, and neither invalidates the design.

M1 may begin.
