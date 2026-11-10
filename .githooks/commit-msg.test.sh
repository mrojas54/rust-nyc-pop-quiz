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

# run_msg <message with \n escapes>: sets RC and OUT.
run_msg() {
  printf '%b' "$1" > "$ROOT/msg"
  OUT="$(sh "$HOOK" "$ROOT/msg" 2>&1)"
  RC=$?
}

accepts() {
  run_msg "$1"
  [ "$RC" = 0 ] && [ -z "$OUT" ] && ok "accepts: ${2:-$1}" || bad "accepts: ${2:-$1}" "rc=$RC out=$OUT"
}

rejects() {
  run_msg "$1"
  [ "$RC" = 1 ] && [ -n "$OUT" ] && ok "rejects: ${2:-$1}" || bad "rejects: ${2:-$1}" "rc=$RC out=$OUT"
}

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

# 8. Through git: the hook fires from core.hooksPath and blocks a bad commit.
if command -v git >/dev/null 2>&1; then
  REPO="$ROOT/repo"
  mkdir -p "$REPO"
  gitq() {
    GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
      git -C "$REPO" -c user.name=Test -c user.email=test@example.com -c core.hooksPath="$HOOK_DIR" "$@"
  }
  if GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git init -q "$REPO" 2>/dev/null; then
    OUT="$(gitq commit -q --allow-empty -m "not conventional" 2>&1)"; RC=$?
    [ "$RC" != 0 ] && echo "$OUT" | grep -q "not a Conventional Commit" && ok "git commit: a bad subject is refused" || bad "git commit: bad subject" "rc=$RC out=$OUT"

    OUT="$(gitq commit -q --allow-empty -m "chore: a good subject" 2>&1)"; RC=$?
    [ "$RC" = 0 ] && ok "git commit: a good subject goes through" || bad "git commit: good subject" "rc=$RC out=$OUT"

    OUT="$(gitq commit -q --allow-empty --no-verify -m "not conventional" 2>&1)"; RC=$?
    [ "$RC" = 0 ] && ok "git commit --no-verify: skips the hook" || bad "git commit --no-verify" "rc=$RC out=$OUT"
  else
    echo "skip - could not create a throwaway repo, so the git-level cases did not run"
  fi
else
  echo "skip - git is not on PATH, so the git-level cases did not run"
fi

echo "$pass passed, $fail failed"
[ "$fail" = 0 ]
