---
id: 8
title: Generate a Rust module per .tri component
type: feature
status: planned
milestone: m0
depends_on:
- 7
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: l
area: compiler
---

## Problem

The AST has to become code rustc can check, with static markup collapsed into as few writes as possible.

## Proposal

One generated module per component in OUT_DIR: a `Component` impl whose render fn is straight-line `sink.raw()` over pre-concatenated literals plus `sink.escaped()` for expressions. Async inferred from the frontmatter.

## Acceptance criteria

- [ ] Runs of static markup collapse to one `raw` call
- [ ] Sync components generate no async machinery
- [ ] Generated code compiles for the fixture set
