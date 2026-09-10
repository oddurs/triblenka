---
id: 3
key: m2
title: M2 — The interactivity ladder
type: milestone
status: backlog
created: 2026-09-09
updated: 2026-09-10
priority: p2
due: 2027-04-30
---

Partial hydration: registry, TypeScript renderer (default), Leptos renderer, wasm split, client directives, size budgets. A bet placed only if M1 finds users.

## 2026-09-10

Reshaped 2026-09-10. M2 is no longer 'Islands' — it is the interactivity ladder (DESIGN §9, Appendix C). Sequencing is the opinion: server frames (rung 1) first, then rung inference and accountability, then the resumable-handler spike (rung 2), and rung-3 islands last and smallest. Building islands first would produce Astro; building frames first produces a framework whose interactivity story is its own.
