---
id: 111
title: Writer-grade error messages
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: l
area: diagnostics
---

Most people who hit an error here are not Rust programmers. Every message names where, what, what was expected and where that expectation is declared, and what to do about it. No serde jargon, no backtraces, no generated code. Every shipped message gets a test asserting its text — messages are an interface. Contract in docs/concepts/errors.md.
