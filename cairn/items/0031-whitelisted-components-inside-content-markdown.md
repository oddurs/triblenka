---
id: 31
title: Whitelisted components inside content markdown
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p2
effort: m
area: content
---

`mdx_component::<Callout>("Callout")` keeps prose as data while allowing a fixed set of components, dispatched at render time. The error for an unregistered component must name both fixes: register it, or move the file to src/pages/**/*.md.tri.
