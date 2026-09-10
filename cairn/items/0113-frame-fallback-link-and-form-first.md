---
id: 113
title: 'Frame fallback: link and form first'
type: feature
status: backlog
milestone: m2
depends_on:
- 112
created: 2026-09-10
updated: 2026-09-10
priority: p0
effort: m
area: interactivity
---

A frame must be an ordinary link or form that happens to be enhanced, never a div with a click handler. The no-JS path is then the primary path and cannot rot, which is what lets require(no_js_fallback) be proved from the rung rather than tested in a browser.

- [ ] Every frame interaction works with JavaScript disabled, verified in CI with a JS-off pass
- [ ] The enhanced and unenhanced paths render byte-identical fragment HTML
