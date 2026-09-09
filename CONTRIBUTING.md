# Contributing

Triblenka is at the design stage: the architecture is settled, the intended API is documented, and
implementation starts at M0. The most valuable contribution right now is an argument — read
[`DESIGN.md`](DESIGN.md) and tell us where it is wrong.

## Setup

```sh
git clone git@github.com:oddurs/triblenka.git
cd triblenka
scripts/setup      # wires git hooks, verifies the toolchain, runs the checks
```

## The workflow

One unit of work, one worktree, one branch, one pull request. `main` only ever advances through a
merged pull request — enforced by branch protection and by a local hook, not by discipline.

```sh
scripts/agent doctor                     # verify the environment
scripts/agent start fix/keep-spans       # branch + worktree, prints the path
cd ../.worktrees/triblenka/fix/keep-spans
# work
scripts/agent commit "fix(compiler): keep spans through codegen"
scripts/agent pr                         # runs checks, pushes, opens the PR
# after it merges
scripts/agent done                       # removes the worktree and the branches
```

Worktrees live at `../.worktrees/<repo>/<branch>/` so that two people — or two agents — never share
a checkout.

## Checks

```sh
scripts/task check     # exactly what CI runs: fmt:check, lint, test, build
```

Every automated caller goes through `scripts/task`, so local hooks and CI cannot drift.

The `check` target also validates the backlog with [cairn](https://github.com/oddurs/cairn) when it
is installed. It is not a prerequisite — the step reports that it was skipped rather than failing —
but if you are changing `cairn/items/`, install it:

```sh
cargo install --git https://github.com/oddurs/cairn --branch main --locked cairn-md
```

## Commits

[Conventional Commits](https://www.conventionalcommits.org/), imperative mood, subject at most 72
characters and no trailing period. The body explains *why*; the diff already says what.

```
fix(compiler): keep spans through codegen

Diagnostics pointed at generated modules whenever an expression spanned more
than one line, because the span map recorded only the start offset.

Refs: 0024
```

The `commit-msg` hook enforces the format and rejects assistant attribution — no co-author
trailers naming a tool, no "generated with" footers. The work is published under the contributors'
own names.

## Issues and the roadmap

Work is tracked with [cairn](https://github.com/oddurs/cairn) as Markdown files under `cairn/items/`,
rendered into [`ROADMAP.md`](ROADMAP.md). `cairn next` shows what is ready to start.

Use GitHub issues for bugs and for proposals that are not on the roadmap yet. Do not add TODO,
PLAN, or NOTES files — file a cairn item so the work appears on the board.

## Review

This is currently a solo repository, so required approvals are set to **0**: the owner would
otherwise be unable to merge their own pull requests. Every other protection still applies —
a pull request is required, CI must be green, conversations must be resolved, and force pushes and
deletions are blocked. If the project gains maintainers, this becomes 1.
