---
id: 8
title: Generate a Rust module per .tri component
type: feature
status: done
milestone: m0
depends_on:
- 7
created: 2026-09-09
updated: 2026-09-11
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

## 2026-09-10

Partially landed: the emitter (tri_compiler::codegen) produces Rust from a parsed document, with tests covering static-run collapsing, verbatim expression pasting, control flow and frontmatter lifting. Not landed: the build.rs loop that compiles what it emits, and the span map that remaps rustc diagnostics back to .tri. Both are M1 (filed separately), so this item stays open and its third acceptance criterion — generated code compiles for the fixture set — is unverified.

## 2026-09-11

Closing 2026-09-11. The first two acceptance criteria are met and tested in tri_compiler::codegen. The third — generated code compiles for the fixture set — is item 0129 'Compile what codegen emits' in M1, word for word. Keeping two items for one piece of work is rot, so this one closes and 0129 carries the remainder.
