---
id: 56
title: Vanilla TypeScript renderer (the default)
type: feature
status: backlog
milestone: m2
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: l
area: islands
---

Markup from a .tri file, behaviour from a TS class, bundled in-process by oxc + rolldown — no Node toolchain. Default renderer per Appendix A motion 3: the median island is 1-2 KB of TS against a 45 KB wasm core, and a framework built to ship less cannot default to the heavier option.
