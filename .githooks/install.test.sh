#!/bin/bash
# Fixture tests for install.sh. Every case runs in its own throwaway repository
# with no user or system git config. Run: bash .githooks/install.test.sh
set -uo pipefail

INSTALL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/install.sh"
ROOT="$(mktemp -d "${TMPDIR:-/tmp}/install-test.XXXXXX")"
if [ -z "$ROOT" ] || [ ! -d "$ROOT" ]; then
  echo "FAIL - could not create a temp directory; no case ran" >&2
  exit 1
fi
trap 'rm -rf "$ROOT"' EXIT
pass=0
fail=0

ok() { pass=$((pass + 1)); echo "ok   - $1"; }
bad() { fail=$((fail + 1)); echo "FAIL - $1"; [ -n "${2:-}" ] && echo "       $2"; }

export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null

# repo <name>: a fresh repository with one commit, so a worktree can be added.
repo() {
  REPO="$ROOT/$1"
  mkdir -p "$REPO"
  git -C "$REPO" init -q
  git -C "$REPO" -c user.name=Test -c user.email=test@example.com commit -q --allow-empty -m "chore: start"
}

# install [dir]: run the installer from a repository; sets RC and OUT.
install() {
  OUT="$(cd "${1:-$REPO}" && sh "$INSTALL" 2>&1)"
  RC=$?
}

path_of() { git -C "${1:-$REPO}" config --get core.hooksPath || true; }

# A default-directory hook that prints a marker and refuses the commit.
hook() {
  printf '#!/bin/sh\necho "%s ran"\nexit 1\n' "$2" > "$1/$2"
  chmod +x "$1/$2"
}

commit() { git -C "${1:-$REPO}" -c user.name=Test -c user.email=test@example.com commit -q --allow-empty -m "chore: try" 2>&1; }

# 1. A clean clone: installs.
repo clean
install
[ "$RC" = 0 ] && [ "$(path_of)" = .githooks ] && echo "$OUT" | grep -q "commit-msg hook on" && ok "clean repo: core.hooksPath set to .githooks" || bad "clean repo" "rc=$RC path=$(path_of) out=$OUT"

# 2. Run again: idempotent.
install
[ "$RC" = 0 ] && [ "$(path_of)" = .githooks ] && echo "$OUT" | grep -q "already on" && ok "second run: already on, exit 0" || bad "second run" "rc=$RC out=$OUT"

# 3. The default hooks directory holds only *.sample files: installs.
repo samples
install
[ "$RC" = 0 ] && [ "$(path_of)" = .githooks ] && ok "only .sample hooks in .git/hooks: installs" || bad "samples only" "rc=$RC out=$OUT"

# 4. A non-executable hook is not run by Git, so it does not block installing.
repo inert
printf '#!/bin/sh\nexit 1\n' > "$REPO/.git/hooks/pre-commit"
install
[ "$RC" = 0 ] && [ "$(path_of)" = .githooks ] && ok "non-executable hook in .git/hooks: installs" || bad "non-executable hook" "rc=$RC out=$OUT"

# 5. An active default-directory hook: refused, still running afterwards.
repo active
hook "$REPO/.git/hooks" pre-commit
hook "$REPO/.git/hooks" pre-push
BEFORE="$(commit)"
install
AFTER="$(commit)"
if [ "$RC" = 1 ] && [ -z "$(path_of)" ] \
  && echo "$OUT" | grep -q "pre-commit" && echo "$OUT" | grep -q "pre-push" \
  && echo "$OUT" | grep -q "commit-msg" \
  && echo "$BEFORE" | grep -q "pre-commit ran" && echo "$AFTER" | grep -q "pre-commit ran"; then
  ok "active hooks in .git/hooks: refused, named, and still blocking commits"
else
  bad "active hooks" "rc=$RC path=$(path_of) before=$BEFORE after=$AFTER out=$OUT"
fi

# 6. One active hook among samples is enough to refuse.
repo mixed
hook "$REPO/.git/hooks" prepare-commit-msg
install
[ "$RC" = 1 ] && [ -z "$(path_of)" ] && echo "$OUT" | grep -q "prepare-commit-msg" && ok "one active hook beside the samples: refused" || bad "mixed" "rc=$RC out=$OUT"

# 7. A linked worktree shares the main repository's hooks directory.
repo shared
hook "$REPO/.git/hooks" pre-commit
git -C "$REPO" worktree add -q "$ROOT/shared-wt" -b wt
install "$ROOT/shared-wt"
[ "$RC" = 1 ] && [ -z "$(path_of "$ROOT/shared-wt")" ] && echo "$OUT" | grep -q "pre-commit" && ok "linked worktree: the shared hooks are found and the install refused" || bad "linked worktree" "rc=$RC out=$OUT"

# 8. Another core.hooksPath: refused and left alone.
repo other
git -C "$REPO" config core.hooksPath tools/hooks
install
[ "$RC" = 1 ] && [ "$(path_of)" = tools/hooks ] && echo "$OUT" | grep -q "already 'tools/hooks'" && ok "another core.hooksPath: refused, unchanged" || bad "other hooksPath" "rc=$RC path=$(path_of) out=$OUT"

# 9. After a refusal, nothing is written to the repository's config.
repo untouched
hook "$REPO/.git/hooks" commit-msg
BEFORE="$(cat "$REPO/.git/config")"
install
[ "$RC" = 1 ] && [ "$BEFORE" = "$(cat "$REPO/.git/config")" ] && ok "refusal leaves .git/config byte for byte as it was" || bad "untouched config" "rc=$RC out=$OUT"

echo "$pass passed, $fail failed"
[ "$fail" = 0 ]
