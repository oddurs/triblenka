# CLAUDE.md

Instructions for agents working in this repository. Read this before touching anything; it is
short on purpose.

## What this is

Triblenka is a server-first web framework for content-driven sites, written in Rust. It is at the
**design stage**: the architecture is settled and recorded in `DESIGN.md`, the intended v1 API is
documented under `docs/`, and implementation begins at M0. Almost everything in `docs/` describes
software that does not exist yet, and every page says so at the top.

One idea shapes every decision here:

> **A content deploy is not a code deploy.** `src/**` is code, compiled into a binary. `content/**`
> is data, loaded into a typed store. A site is a program plus a store, so a content change is a
> data change — a store write and a millisecond re-render — not a rebuild.

If a proposal requires compiling something in `content/**`, the proposal is wrong. That is not a
preference; it is the property the whole design is built to protect.

## Hard rules

1. **Never commit to `main`.** Not for a typo, not "just for setup". `main` advances only through a
   merged pull request. A local hook and branch protection both enforce this; do not look for a way
   around either.
2. **No assistant attribution anywhere.** No co-author trailers naming a tool, no "generated with"
   footers, no robot emoji, in commits, PR bodies, issues, code comments, docs, or release notes.
   The `commit-msg` hook rejects these patterns. Everything here is published under the owner's
   name.
3. **One unit of work, one worktree, one branch, one PR.** Two agents never share a checkout.
4. **A branch is green before it becomes a PR.** `scripts/task check` — no exceptions, no
   `--no-verify`, no `|| true`, no `continue-on-error`.
5. **Never invent a TODO, PLAN, or NOTES file.** Work is tracked in cairn (below). A parallel list
   competes with the real one and rots.

## The loop

```sh
scripts/agent doctor                    # verify the environment first
scripts/agent start fix/keep-spans      # branch + worktree; prints the path to cd into
scripts/agent commit "fix(compiler): keep spans through codegen"
scripts/agent pr                        # check, push, open the PR
scripts/agent done                      # after merge: remove worktree and branches
```

Worktrees live at `../.worktrees/triblenka/<branch>/`. `scripts/agent` never `cd`s for you — it
prints the path.

<!-- cairn:begin -->
## Roadmap and issues

This project tracks its roadmap and issues with `cairn`. Every item is a Markdown file under `cairn/items`, described by the schema in `cairn.toml`.

**Do not create ad-hoc TODO, PLAN or NOTES files.** Create a cairn item instead, so the work appears on the board and in the generated roadmap.

### The loop

1. `cairn next` — what is ready to start. It excludes anything blocked by unfinished dependencies and puts work already in progress first.
2. `cairn claim <ID>` — take it before you start, so no one duplicates the work. `cairn claim --next` picks and claims the top-ranked unclaimed item in one step, and prints its body so you can begin immediately.
3. Do the work. Record what you learn: `cairn set <ID> <field>=<value>` for fields, `cairn note <ID> "<TEXT>"` for anything that needs a sentence — why you chose something, what you tried, what to watch for.
4. `cairn close <ID>` when it is done, or `cairn release <ID>` to hand it back.
5. `cairn check` before you report finished. It must pass.

### Commands

```sh
cairn next --json                 # ready work, ranked
cairn claim --next                # take the next ready item
cairn search <TEXT> --json        # titles, bodies and labels
cairn list --json                 # all open items
cairn list --filter 'blocked=false,priority=p0'
cairn show <ID> --json            # one item, including its body
cairn new "<TITLE>" --type <TYPE> --milestone <MILESTONE>
cairn set <ID> status=<STATUS>    # also labels+=x, or any field below
cairn note <ID> "<TEXT>"          # append reasoning; never replaces
cairn close <ID>
cairn check                       # validate; run before finishing
cairn render                      # regenerate ROADMAP.md
```

### Schema

- **Types**: `feature`, `bug`, `chore`, `docs`, `milestone`
- **Statuses**: `backlog` (open), `planned` (open), `doing` (active), `blocked` (active), `done` (done), `dropped` (dropped)
- **`milestone`**: names a `milestone` item, by key — what this ships in
- **`due`**: date, YYYY-MM-DD — when a milestone is meant to land
- **`part_of`**: names any items, by id, several allowed — a larger piece of work this belongs to
- **`priority`**: one of p0, p1, p2, p3 — p0 is a release blocker
- **`effort`**: one of s, m, l, xl — Rough size, not an estimate
- **`area`**: free text — Subsystem this touches
- **Milestones**: `m0` (due 2026-09-30), `m1` (due 2027-02-05), `m2` (due 2027-04-30), `m3` (due 2027-06-30), `m4` (due 2027-09-30), `m5` (due 2027-12-31)
- **Saved views** (`cairn list --view NAME`): `now`, `next`, `triage`

### Rules

1. Before starting work, find or create the item and set it to an active status.
2. Use the fields above rather than inventing new ones; add new fields to `cairn.toml` first.
3. Never hand-edit the generated roadmap file — change items and run `cairn render`.
4. `cairn check` must pass before the work is considered done.

<!-- cairn:end -->
## The seam

All automation goes through one interface, so CI, hooks and scripts never learn anything
stack-specific:

```sh
scripts/task fmt        fmt:check      lint      test      build      check
```

`check` is the gate: `fmt:check && lint && test && build`, plus `cairn check` where cairn is
installed. If you need a new kind of check, add it to `scripts/task` — never directly to a workflow
or a hook, or local and CI will drift.

## Where things are

| Path | What |
|---|---|
| `DESIGN.md` | the architecture, the design review (Appendix A), the Astro source audit (Appendix B) |
| `docs/concepts/` | why the project exists, architecture, incrementality, determinism, errors |
| `docs/guides/`, `docs/reference/` | the intended v1 API |
| `cairn/items/`, `ROADMAP.md` | the backlog and its rendered view — never hand-edit `ROADMAP.md` |
| `crates/triblenka` | the library crate |
| `crates/tri-cli` | the `tri` binary |
| `scripts/`, `.githooks/` | the workflow; POSIX `sh`, dependency-free |

Planned crate boundaries are listed in `docs/concepts/architecture.md`. Islands get their own crate
on purpose: editing an island must never rebuild the site, and editing the site must never rebuild
wasm.

## Before proposing an architectural change

Read `DESIGN.md` Appendix A first. Eleven decisions were argued and recorded there, two of them
reversed — if you are about to suggest salsa for the build graph, Leptos as the default island
renderer, or making route ambiguity a hard error, the argument has already happened and the answer
is written down. Bring new evidence, not the original intuition.

If a decision does change, update `DESIGN.md` in the same PR, with the reasoning. A design document
that disagrees with the code is worse than none.

## Rust conventions

- **Errors are `miette` diagnostics with spans.** An error without a span needs a good reason. See
  `docs/concepts/errors.md` for the contract every message must meet — where, what, what was
  expected, and what to do.
- **Never show generated code to a user.** `.tri` compiles to Rust; diagnostics are remapped back to
  the `.tri` source. If a path containing `OUT_DIR` or `tri_generated` reaches a user, that is a bug.
- **No `unwrap`/`expect`/`panic!` on any path user input can reach.** A malformed content file is an
  error value, not a crash. `clippy::unwrap_used` is on.
- **Every shipped error message gets a test asserting its text.** Messages are an interface.
- **Determinism is not optional** — see `docs/concepts/determinism.md`. Sort byte-wise, never with
  `localeCompare`-style locale collation. Never let `HashMap`/`HashSet` iteration order reach
  output. No wall-clock time, no absolute paths, no randomness in emitted bytes.
- **Adding a dependency requires a sentence in the PR body** saying what it replaces and why writing
  it ourselves is worse. The dependency tree is a feature of this project.

## Budgets that gate a merge

These are the design's load-bearing claims. If a change moves one of them, that is the headline of
the PR, not a footnote.

| Budget | Limit |
|---|---|
| Content-only rebuild | < 50 ms (p99, fixture site) |
| Structure-only template edit | < 100 ms end to end |
| A page with no islands | **0 bytes** of JavaScript |
| A page with one island | ≤ ~1.1 KB JS + one cached wasm core |
| Incremental vs clean build | byte-identical (`--verify-incremental`) |

## Do not

- Add a JavaScript engine to the default build. Considered and rejected; `client:only` covers the
  real need.
- Add a second Rust island renderer. One is enough.
- Add a configuration knob to avoid making a decision.
- Compile anything in `content/**`.
- Ship a page with JavaScript it did not ask for.
- Remove a "design-stage" banner from a docs page until that page's behaviour actually exists —
  and when it does, say so in `CHANGELOG.md`.
- Silently narrow scope. If you cut something, file the cut piece as a cairn item so it is recorded
  rather than lost.

## Agents

Six subagents live in `.claude/agents/`. Delegate to them rather than re-deriving what they already
know — each one carries this project's standards for its slice of the work.

| Agent | Use it when |
|---|---|
| `design-critic` | Someone proposes an architectural change, including you. It knows Appendix A and will tell you which motion you are re-opening |
| `astro-parity` | You are about to implement a subsystem Astro also has. It reads their source, not their docs, and reports adopt / adapt / diverge |
| `determinism-auditor` | You added code that emits bytes or orders a collection. It hunts the seven rules in `docs/concepts/determinism.md` |
| `error-smith` | You are adding an error path, or a message reads like it was written for a compiler engineer |
| `docs-keeper` | You implemented something the docs describe, or changed an API. It owns the design-stage banners |
| `backlog-groomer` | A session discovered or dropped work. It files it in cairn with the reasoning |

There is deliberately no agent for island payloads, the compiler, or the build graph yet: there is
no code for them to act on, and this file forbids dead config. Add them when M0 produces something
to measure.

## Settings and hooks

`.claude/settings.json` is committed, so these apply to everyone working in the repository.

- **Denied outright**: writes to `ROADMAP.md` (generated — change `cairn/items/` and run
  `cairn render`), and `git commit --no-verify` / `git push --no-verify` (hard rule 4).
- **One hook**: `PreToolUse` on `git commit*` refuses to commit while the default branch is checked
  out, and tells you to run `scripts/agent start` instead. The repository's own hooks refuse the
  *push*; this closes the gap where a commit lands on `main` locally and has to be moved.
- **Allowlisted**: read-only `cargo`, `git`, `gh`, `cairn` and `scripts/task` invocations, so the
  ordinary loop does not generate permission prompts.

Personal overrides go in `.claude/settings.local.json`, which is gitignored.

## When you are unsure

Ask, or write the argument down in the item and proceed under a stated assumption. The one thing
not to do is guess silently — this project's whole value is that its decisions are legible a year
later.

