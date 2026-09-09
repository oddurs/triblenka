#!/bin/sh
# PreToolUse guard: refuse `git commit` while the default branch is checked out.
#
# The repository's pre-push hook already refuses to push the default branch, and
# branch protection refuses on the server. Neither stops a commit being made on
# the default branch in the first place — which leaves a commit that has to be
# moved before any work can be pushed. This closes that gap.
#
# Filtered by `if: Bash(git commit*)` in settings.json, so it does not run for
# unrelated shell commands and does not need to read its stdin payload.
set -eu

default=$(git symbolic-ref --quiet --short refs/remotes/origin/HEAD 2>/dev/null | sed 's|^origin/||' || true)
[ -n "$default" ] || default=main
current=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)

if [ "$current" = "$default" ]; then
	cat <<JSON
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "Refusing to commit on '$default'. This repository's default branch only ever advances through a merged pull request (CLAUDE.md, hard rule 1). Run 'scripts/agent start <type>/<slug>' to get a branch and a worktree, cd into the path it prints, and commit there."
  }
}
JSON
fi
