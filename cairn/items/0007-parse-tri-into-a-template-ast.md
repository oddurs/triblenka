---
id: 7
title: Parse .tri into a template AST
type: feature
status: done
milestone: m0
created: 2026-09-09
updated: 2026-09-10
priority: p0
effort: l
area: compiler
---

## Problem

Everything downstream needs a faithful, span-carrying AST: frontmatter Rust, markup, interpolations, block forms, component tags, directives.

## Proposal

Tolerant HTML-shaped parser for the markup; `syn` for the frontmatter block and for every embedded expression and pattern. Every node carries a byte span into the source. Block forms are the closed set: if / for / match / let.

## Acceptance criteria

- [ ] Round-trips the fixture set without loss of spans
- [ ] Unclosed block, unknown directive, and void-element misuse are parse errors with exact spans
- [ ] Frontmatter and template are separable without evaluating either
