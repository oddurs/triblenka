---
id: 115
title: Rung inference in the compiler
type: feature
status: backlog
milestone: m2
created: 2026-09-10
updated: 2026-09-10
priority: p0
effort: l
area: compiler
---

## Problem

A directive is a workaround for a compiler that cannot infer. Marko has done automatic partial hydration for years while everyone else asks you to type client:visible. Ours can infer.

## Proposal

No handlers and no signals is static; a submit or a link is a frame; handlers closing over serializable state are resumable; continuous local state is an island. rung:static, rung:frame, rung:resumable and rung:island override when the author disagrees, and rung:static is an assertion that fails to compile if wrong.

## Acceptance criteria

- [ ] Inference is deterministic and explained by tri explain for any component
- [ ] An override that contradicts what the component actually does is a compile error, not a silent downgrade
