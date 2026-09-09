---
id: 33
title: Scoped CSS via lightningcss
type: feature
status: backlog
milestone: m1
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: l
area: assets
---

Hash the component, tag its elements with data-t-<hash>, rewrite selectors through the lightningcss visitor. `is:global` opts out, `:global(...)` reaches into a child deliberately.

- [ ] A parent's rules never leak into a child component
- [ ] Nesting and custom media compile down to the configured targets

## 2026-09-09

Three specifics verified against the Astro compiler (Appendix B.6): wrap the injected selector in :where() so scoping adds zero specificity; never scope Fragment, base, font, frame, frameset, head, link, meta, noframes, noscript, script, style, slot, title, or the :root selector; scope id is a short base32-truncated hash.
