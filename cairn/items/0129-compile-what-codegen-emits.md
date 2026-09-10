---
id: 129
title: Compile what codegen emits
type: feature
status: backlog
milestone: m1
created: 2026-09-10
updated: 2026-09-10
priority: p0
effort: l
area: compiler
---

The M0 emitter produces Rust from a parsed document but nothing compiles it. Wire build.rs to walk src/, emit into OUT_DIR, and have rustc build the result, so the release path is exercised rather than asserted. This is the remaining half of item 0008 and a prerequisite for the span map.

- [ ] The fixture's templates compile through build.rs
- [ ] Descriptor output and compiled output are byte-identical (property test)
