---
id: 118
title: Contract declarations and enforcement
type: feature
status: backlog
milestone: m2
created: 2026-09-10
updated: 2026-09-10
priority: p1
effort: l
area: contracts
---

deny(javascript), require(no_js_fallback), deny(external_requests) declared per page or site-wide, enforced as build errors naming the component that broke them. Rungs 0-2 satisfy no_js_fallback by construction, so it is proved from the rung rather than tested in a browser. A guarantee that is not enforced is a slogan — see docs/concepts/contracts.md.
