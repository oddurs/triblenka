---
id: 126
title: Check documentation links in CI
type: chore
status: backlog
milestone: m1
created: 2026-09-10
updated: 2026-09-10
priority: p2
effort: s
area: docs
---

Every internal Markdown link in the repository resolves, and every page under docs/ is reachable from docs/index.md or README.md. Found by hand after the ladder change, which left the reference pages describing a surface that no longer existed — exactly the drift class a check should catch rather than a reviewer.

Note the one legitimate exception: docs/concepts/errors.md contains a deliberately broken link inside a sample error message demonstrating dead-link detection. The checker needs to ignore fenced code blocks.

- [ ] Wired into scripts/task check
- [ ] Fenced code blocks excluded
