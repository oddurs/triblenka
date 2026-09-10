---
id: 117
title: 'Spike: resumable handlers'
type: feature
status: backlog
milestone: m2
created: 2026-09-10
updated: 2026-09-10
priority: p1
effort: xl
area: interactivity
---

## Problem

Our worst structural weakness is the wasm constant factor: 45 KB before a counter does anything. Resumability removes the boot entirely — serialize the captured state, attach one delegated listener, fetch the closure on demand.

## Proposal

Qwik's idea, more natural in Rust: a JavaScript compiler must discover what a closure captured, while rustc already knows it as a typed struct, so the serialization is derived rather than inferred. #[handler] becomes a wasm-split point.

## Kill criterion

A spike, not a commitment. If serializing captured state costs more than the boot it saves — measured on a real widget, not a counter — it does not ship, and docs/concepts/interactivity.md loses the rung. Decide before rung 3 work starts, because the answer changes how much of rung 3 we need.
