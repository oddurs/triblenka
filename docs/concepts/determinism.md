# Determinism

> Design-stage docs.

**The guarantee: the same inputs produce byte-identical outputs, on any machine, in any locale, at
any time.**

This is unusual enough to need saying explicitly. It is also fragile enough that it only survives if
it is a stated rule with tests behind it, rather than something that happens to be true today.

## Why it is worth the discipline

- **Incremental correctness is checkable.** `--verify-incremental` compares an incremental build
  against a clean one byte for byte. That check is meaningless if a clean build is itself
  nondeterministic.
- **Deploys are diffable.** If output only changes when input changes, "what did this deploy do" has
  an exact answer, and CDN purges stay minimal.
- **Caches are trustworthy.** A content-addressed artifact is only safe if the address is a function
  of the input alone.
- **Some publications must be reproducible.** Archival, regulatory, and documentation-of-record
  sites need to demonstrate that a given source produced a given output. No npm-based toolchain can
  offer this; we can, and cheaply.

## The rules that keep it true

Contributors: these are enforced in review and, where possible, in CI.

1. **No locale-dependent comparison.** Sort byte-wise. Astro's route comparator tiebreaks with
   `localeCompare`, which means the same source can in principle order two routes differently on two
   machines — the exact bug this rule exists to prevent.
2. **No unordered iteration reaching output.** `HashMap`/`HashSet` iteration order is unspecified
   and randomized. Collect and sort before emitting, or use an ordered map.
3. **No wall-clock time in output.** Build timestamps, "generated at" comments, and cache-busting
   from `now()` are all banned. Anything that needs a time takes it from content (a post's date) or
   from an explicitly passed input.
4. **No absolute paths in output.** Paths are relative to the project root, always.
5. **No randomness.** Ids are derived by hashing stable inputs. Where a unique id per render is
   genuinely required — a server island host id — it is derived from the render path, not
   `random()`.
6. **Pinned toolchain.** `rust-toolchain.toml` pins the exact version; a different `rustc` is a
   different input and may legitimately produce different bytes.
7. **Sorted directory traversal.** Filesystem readdir order is not stable across platforms. Every
   glob and every walk sorts before it emits.

## How it is verified

```sh
tri build --verify-incremental    # incremental vs clean, byte-compared
tri build --verify-reproducible   # two clean builds in different temp dirs, byte-compared
```

The framework's own CI runs both on the fixture site, and additionally runs the reproducible check
under a different `LANG`, a different `TZ`, and a different filesystem path — the three environment
differences that catch rules 1, 3, and 4 respectively.

## What is deliberately excluded

- **Compression output.** If a deploy target compresses at the edge, its output is its business.
- **The store file's internal layout.** The store is a cache, not an artifact; only what it produces
  is guaranteed.
- **Timing information in `--stats`.** Reports are diagnostics, not outputs.
