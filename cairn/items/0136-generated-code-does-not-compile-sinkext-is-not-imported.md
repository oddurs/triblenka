---
id: 136
title: 'Generated code does not compile: SinkExt is not imported'
type: bug
status: backlog
milestone: m1
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: compiler
part_of:
- 129
---

The emitted module imports tri_core::{Render, Result, Sink} but calls sink.escaped(..), which lives on SinkExt. Building the emitted text gives error[E0599]: no method named 'escaped' found for '&mut dyn tri_core::Sink', plus an unused import warning for Render. The attribute branch masks the asymmetry because it calls the free function tri_core::escaped_attribute.

Directly blocks item 0129.
