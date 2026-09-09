---
id: 43
title: HMR protocol and DOM morphing client
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: m
area: devserver
---

Three update kinds — css, html, wasm. HTML updates morph in place and leave hydrated island subtrees untouched when their id and props hash are unchanged, so island state survives a template edit.
