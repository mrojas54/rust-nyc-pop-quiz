#!/bin/bash
# Fixture tests for the commit-msg hook. Message cases run the hook directly on a
# temp file; the last cases run it through `git commit` in a throwaway repo, which
# also proves the executable bit. Run: bash .githooks/commit-msg.test.sh
set -uo pipefail

HOOK_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOK="$HOOK_DIR/commit-msg"
ROOT="$(mktemp -d "${TMPDIR:-/tmp}/commit-msg-test.XXXXXX")"
if [ -z "$ROOT" ] || [ ! -d "$ROOT" ]; then
  echo "FAIL - could not create a temp directory; no case ran" >&2
  exit 1
fi
trap 'rm -rf "$ROOT"' EXIT
pass=0
fail=0

ok() { pass=$((pass + 1)); echo "ok   - $1"; }
bad() { fail=$((fail + 1)); echo "FAIL - $1"; [ -n "${2:-}" ] && echo "       $2"; }

# run_msg <message with \n escapes> [VAR=value ...]: sets RC and OUT.
#
# The hook runs from a directory outside any repository, with no user or system
# git config, so a developer's commit.cleanup or core.commentChar cannot change
# the result. GIT_EDITOR is unset, as in a commit that opens an editor; pass
# GIT_EDITOR=: for a commit with none, such as git commit -m. Git sets exactly
# that for the hook, and it is what the hook reads.
run_msg() {
  local message="$1"
  shift
  printf '%b' "$message" > "$ROOT/msg"
  OUT="$(cd "$ROOT" && env -u GIT_EDITOR GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null ${@+"$@"} sh "$HOOK" "$ROOT/msg" 2>&1)"
  RC=$?
}

# check accepts|rejects <message> <label> [VAR=value ...]
check() {
  local want="$1" message="$2" label="$3"
  shift 3
  run_msg "$message" ${@+"$@"}
  if [ "$want" = accepts ]; then
    [ "$RC" = 0 ] && [ -z "$OUT" ] && ok "accepts: $label" || bad "accepts: $label" "rc=$RC out=$OUT"
  else
    [ "$RC" = 1 ] && [ -n "$OUT" ] && ok "rejects: $label" || bad "rejects: $label" "rc=$RC out=$OUT"
  fi
}

accepts() { check accepts "$1" "${2:-$1}"; }
rejects() { check rejects "$1" "${2:-$1}"; }

# 1. Every type, with and without a scope or a breaking-change mark.
for t in feat fix docs test chore refactor ci build perf style revert; do
  accepts "$t: do the thing\n"
done
accepts "fix(organizer-tui): a refused decision no longer reloads the preview\n"
accepts "docs(spec): add the host's signed-out line\n"
accepts "feat(room)!: drop the not-yet-open refusal\n"
accepts "fix(q8.a11y/x): scope characters\n"

# 2. This project's trailing markers are part of the free-text description.
accepts "test(a11y): home's step buttons hold their edges at 3:1 (RED)\n"
accepts "test(fallback): no standalone command builds a fallback (PQ-52 RED)\n"
rejects "fallback: say that --out is a directory (GREEN)\n" "unknown type, even with a marker"

# 3. Subjects that are not Conventional Commits.
rejects "Hold the signed-out line until the line before it has been written\n" "plain imperative sentence"
rejects "feat:missing space\n"
rejects "feat:  \n" "empty description"
rejects "feat: \n" "trailing space only"
rejects "Feat: capitalised type\n"
rejects "wip: unknown type\n"
rejects "feat(): empty scope\n"
rejects "feat(a b): space in scope\n"
rejects "feat (scope): space before scope\n"
rejects "feat(scope) : space before colon\n"
rejects "feat\n" "type only"

# 4. Exempt subjects.
accepts "Merge pull request #144 from mrojas54/codex/reverify-q7-q8\n"
accepts "Merge branch 'main' into feature\n"
accepts 'Revert "feat: add the thing"\n'
accepts "fixup! feat: add the thing\n"
accepts "squash! feat: add the thing\n"
accepts "amend! feat: add the thing\n"

# 5. Git's template: comment lines and leading blank lines are not the subject.
accepts "\n\nfeat: the real subject\n# Please enter the commit message\n# Lines starting with '#' are ignored\n" "comments and leading blank lines skipped"
rejects "# a comment\n\nnot a conventional subject\n" "bad subject below a comment"

# 6. Empty messages are git's to refuse.
accepts "" "empty message"
accepts "# only comments\n" "only comments"

# 7. Body: a blank line must follow the subject.
accepts "fix: x\n\nBody explains why.\n" "subject, blank line, body"
rejects "fix: x\nBody straight after the subject.\n" "no blank line before the body"
accepts "fix: x\n\nBody.\n\nBREAKING CHANGE: y\n" "footer"

# 7b. # lines. Git strips them only when the message is cleaned with `strip`,
# which is the default for a commit that opens an editor. git commit -m uses
# `whitespace` and keeps them, so there a # line is message text and must be
# checked like any other. The hook infers the mode the way Git picks its default
# (GIT_EDITOR=: means no editor) and honors commit.cleanup.
check rejects "# not conventional\n" "no editor: a # line is the subject, not a comment" GIT_EDITOR=:
check rejects "#123 fix key reference\n" "no editor: a hash-prefixed subject" GIT_EDITOR=:
check rejects "# actual subject\nfix: the hook checks the wrong line\n" "no editor: the # line is the subject" GIT_EDITOR=:
check rejects "fix: valid subject\n# body without separator\n" "no editor: a # line straight after the subject" GIT_EDITOR=:
check accepts "fix: valid subject\n\n# kept in the body\n" "no editor: a # line in the body" GIT_EDITOR=:
check accepts "# note\nfix: after a comment\n" "editor: a leading # line is stripped"
check accepts "fix: x\n# comment straight after the subject\n" "editor: a # line after the subject is stripped"
check accepts "# note\nfix: cleaned\n" "commit.cleanup=strip strips # lines with no editor" GIT_EDITOR=: GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=commit.cleanup GIT_CONFIG_VALUE_0=strip
check rejects "# note\nfix: kept\n" "commit.cleanup=whitespace keeps # lines even with an editor" GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=commit.cleanup GIT_CONFIG_VALUE_0=whitespace
check rejects "# note\nfix: kept\n" "commit.cleanup=verbatim keeps # lines even with an editor" GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=commit.cleanup GIT_CONFIG_VALUE_0=verbatim
check accepts "; note\nfix: cleaned\n" "core.commentChar=; strips ; lines" GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.commentChar GIT_CONFIG_VALUE_0=';'
check rejects "# note\nfix: kept\n" "core.commentChar=; leaves # lines alone" GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.commentChar GIT_CONFIG_VALUE_0=';'

# 8. Through git: the hook fires from core.hooksPath and blocks a bad commit.
if command -v git >/dev/null 2>&1; then
  REPO="$ROOT/repo"
  mkdir -p "$REPO"
  gitq() {
    GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
      git -C "$REPO" -c user.name=Test -c user.email=test@example.com -c core.hooksPath="$HOOK_DIR" "$@"
  }

  # A commit git must refuse: the hook says so, and the repository gains no commit.
  git_refuses() {
    local label="$1" before after
    shift
    before="$(gitq rev-list --count --all 2>/dev/null)"
    OUT="$(gitq commit -q --allow-empty "$@" 2>&1)"; RC=$?
    after="$(gitq rev-list --count --all 2>/dev/null)"
    if [ "$RC" != 0 ] && [ "$before" = "$after" ] && echo "$OUT" | grep -q "commit-msg:"; then
      ok "git commit: $label is refused by the hook"
    else
      bad "git commit: $label" "rc=$RC commits $before -> $after out=$OUT"
    fi
  }

  # A commit git must take, and the subject it stored.
  git_takes() {
    local label="$1" want="$2" got
    shift 2
    OUT="$(gitq commit -q --allow-empty "$@" 2>&1)"; RC=$?
    got="$(gitq log -1 --format=%s 2>/dev/null)"
    if [ "$RC" = 0 ] && [ "$got" = "$want" ]; then
      ok "git commit: $label"
    else
      bad "git commit: $label" "rc=$RC subject=$got out=$OUT"
    fi
  }
  if GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git init -q "$REPO" 2>/dev/null; then
    OUT="$(gitq commit -q --allow-empty -m "not conventional" 2>&1)"; RC=$?
    [ "$RC" != 0 ] && echo "$OUT" | grep -q "not a Conventional Commit" && ok "git commit: a bad subject is refused" || bad "git commit: bad subject" "rc=$RC out=$OUT"

    OUT="$(gitq commit -q --allow-empty -m "chore: a good subject" 2>&1)"; RC=$?
    [ "$RC" = 0 ] && ok "git commit: a good subject goes through" || bad "git commit: good subject" "rc=$RC out=$OUT"

    OUT="$(gitq commit -q --allow-empty --no-verify -m "not conventional" 2>&1)"; RC=$?
    [ "$RC" = 0 ] && ok "git commit --no-verify: skips the hook" || bad "git commit --no-verify" "rc=$RC out=$OUT"

    # git commit -m keeps # lines, so they are message text the hook must check.
    git_refuses "-m '# not conventional'" -m "# not conventional"
    git_refuses "-m '#123 fix key reference'" -m "#123 fix key reference"
    git_refuses "-m with a # subject over a valid line" -m "$(printf '# actual subject\nfix: the hook checks the wrong line')"
    git_refuses "-m with a # line straight after the subject" -m "$(printf 'fix: valid subject\n# body without separator')"
    git_takes "-m keeps a # line in the body" "fix: a body with a hash line" -m "$(printf 'fix: a body with a hash line\n\n# why: kept in the body')"

    # A commit that opens an editor gets git's template, and git strips its # lines.
    EDITOR_SH="$ROOT/editor.sh"
    printf '%s\n' '#!/bin/sh' '{ printf "%s\n" "$TYPED_SUBJECT"; cat "$1"; } > "$1.new" && mv "$1.new" "$1"' > "$EDITOR_SH"
    chmod +x "$EDITOR_SH"

    OUT="$(export TYPED_SUBJECT="fix: typed in an editor" GIT_EDITOR="$EDITOR_SH"; gitq commit -q --allow-empty 2>&1)"; RC=$?
    got="$(gitq log -1 --format=%s 2>/dev/null)"
    [ "$RC" = 0 ] && [ "$got" = "fix: typed in an editor" ] && ok "git commit with an editor: the template's # lines are stripped" || bad "git commit with an editor" "rc=$RC subject=$got out=$OUT"

    before="$(gitq rev-list --count --all 2>/dev/null)"
    OUT="$(export TYPED_SUBJECT="typed in an editor, not conventional" GIT_EDITOR="$EDITOR_SH"; gitq commit -q --allow-empty 2>&1)"; RC=$?
    after="$(gitq rev-list --count --all 2>/dev/null)"
    [ "$RC" != 0 ] && [ "$before" = "$after" ] && echo "$OUT" | grep -q "commit-msg:" && ok "git commit with an editor: a bad subject is refused" || bad "git commit with an editor, bad subject" "rc=$RC commits $before -> $after out=$OUT"

    # commit.cleanup=strip makes -m lose its # lines too.
    gitq config commit.cleanup strip
    git_takes "commit.cleanup=strip: -m loses its # lines" "fix: cleaned up" -m "$(printf '# note\nfix: cleaned up')"
    gitq config --unset commit.cleanup
  else
    echo "skip - could not create a throwaway repo, so the git-level cases did not run"
  fi
else
  echo "skip - git is not on PATH, so the git-level cases did not run"
fi

echo "$pass passed, $fail failed"
[ "$fail" = 0 ]
