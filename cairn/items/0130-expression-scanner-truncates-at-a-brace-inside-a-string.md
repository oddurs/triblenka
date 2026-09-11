---
id: 130
title: Expression scanner truncates at a brace inside a string
type: bug
status: backlog
milestone: m1
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: m
area: compiler
---

## What happens

`{ format!("}") }` parses as the expression `format!("` followed by the literal text `") }`. The scanner counts braces without knowing about string literals, so the first `}` inside a string ends the expression.

## What should happen

The expression should be taken whole. Worse than the truncation is that it is silent — no error, just wrong output.

## Reproduction

crates/tri-compiler/tests/review_probes.rs::known_limitation_a_brace_inside_a_string_literal_truncates_the_expression pins the current behaviour; it fails when this is fixed, which is the signal to delete it.

## Fix

Hand the expression text to `syn` rather than scanning for a balanced brace, or lex string and char literals in the scanner. `syn` is the right answer since expressions must be real Rust anyway.
