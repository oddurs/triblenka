---
id: 135
title: Slug derivation degrades to absolute paths
type: bug
status: backlog
milestone: m1
created: 2026-09-11
updated: 2026-09-11
priority: p1
effort: s
area: content
---

slug_for does path.strip_prefix(root).unwrap_or(path), which silently yields the absolute path when root is not a prefix. On macOS temp_dir() returns /var/folders/... while anything canonicalized reports /private/var/folders/..., so load_file produces a slug of private/var/folders/.../post-0042, writes a second store entry, and emits dist/blog/private/var/... — machine-absolute paths baked into store keys, contradicting the determinism rules the loader cites.

Should be an error rather than an unwrap_or.
