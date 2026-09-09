---
id: 67
title: Server islands with encrypted props
type: feature
status: backlog
milestone: m3
created: 2026-09-09
updated: 2026-09-09
priority: p0
effort: l
area: server
---

server:defer, a fallback slot, and AES-GCM props under a per-build key so a client cannot tamper with a deferred render's inputs. Default delivery is fetch-per-island with a hash-allowlisted shim: CSP-clean and cacheable.

## 2026-09-09

Corrected by reading core/encryption.ts: a per-build key is wrong twice — it breaks horizontally scaled deploys and invalidates the build cache every build. Use an out-of-band TRI_KEY plus a one-way hash of it as a memo-table input. Encrypt the component export alongside the props.
