---
id: 27
title: Ref<T> with referential integrity
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p1
effort: m
area: content
---

`author: Ref<Author>` fails the build when it points at nothing, and `post.author.resolve()` returns `&Author` without lookup boilerplate. The class of bug that ships as undefined in every JS content framework.
