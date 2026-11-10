#!/bin/sh
# Turn on the commit-msg hook: point core.hooksPath at .githooks. `just hooks`.
#
# core.hooksPath replaces Git's hook directory, it does not add to it. So this
# refuses when it would silently switch off hooks that are running today:
#   - a core.hooksPath that is already set to something else, and
#   - executable hooks in the default hooks directory (.git/hooks, or the shared
#     one in a linked worktree), which Git would stop running.
# In both cases nothing is changed and the message says how to keep them.
#
# Opt-in and local to this clone. Skip a commit once with --no-verify; turn it
# off with `git config --unset core.hooksPath`. Tests: bash .githooks/install.test.sh

set -eu

want=.githooks
current=$(git config --get core.hooksPath || true)

if [ "$current" = "$want" ]; then
  echo "commit-msg hook already on (core.hooksPath=$want)"
  exit 0
fi

top=$(git rev-parse --show-toplevel)

if [ -n "$current" ]; then
  {
    echo "core.hooksPath is already '$current'; not replacing it."
    echo "Call the check from a hook there instead of changing the path:"
    echo "  '$top/.githooks/commit-msg' \"\$1\" || exit 1"
  } >&2
  exit 1
fi

dir=$(git rev-parse --git-path hooks)
active=
# Git runs an executable regular file named for a hook; *.sample files are inert.
for file in "$dir"/*; do
  [ -f "$file" ] && [ -x "$file" ] || continue
  case ${file##*/} in *.sample) continue ;; esac
  active="$active  ${file##*/}
"
done

if [ -n "$active" ]; then
  {
    echo "Not turning the hook on: these hooks in $dir would stop running,"
    echo "because core.hooksPath replaces that directory:"
    printf '%s' "$active"
    echo "Leave core.hooksPath alone and call the check from your own commit-msg"
    echo "hook ($dir/commit-msg), keeping its exit status:"
    echo "  '$top/.githooks/commit-msg' \"\$1\" || exit 1"
  } >&2
  exit 1
fi

git config core.hooksPath "$want"
echo "commit-msg hook on: subjects must be Conventional Commits (.agents/skills/conventional-commits/SKILL.md)"
