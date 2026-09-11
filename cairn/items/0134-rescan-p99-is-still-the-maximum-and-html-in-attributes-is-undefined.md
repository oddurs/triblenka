---
id: 134
title: Rescan p99 is still the maximum, and Html in attributes is undefined
type: bug
status: backlog
milestone: m1
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: build
---

Three integrity defects found reviewing #10:

- RESCAN_ITERATIONS = 20 makes percentile(0.99) round to index 19, the last one — so the rescan's reported p99 is the maximum, the same overstatement the commit criticised. The 941 ms figure in DESIGN §16 and item 0127 inherits it.
- The known-limitation probe for item 0130 asserts a substring satisfied by both the broken and the fixed parse, so the tripwire can never fire.
- AttributeSink escapes quotes in everything, including Html, whose contract says it is never escaped. Decide whether Html in attribute position is an error or passes through, and test it.
- render_via_display writes through sink.raw with no escaping, so { c } with c == '<' emits a raw <, in both text and attribute context.
