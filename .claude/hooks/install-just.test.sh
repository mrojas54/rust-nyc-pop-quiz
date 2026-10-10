#!/bin/bash
# Fixture tests for install-just.sh. No network: curl is a stub serving a fake
# tarball, and every case runs in its own temp HOME. Run: bash .claude/hooks/install-just.test.sh
set -uo pipefail

HOOK="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/install-just.sh"
ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT
pass=0
fail=0

ok() { pass=$((pass + 1)); echo "ok   - $1"; }
bad() { fail=$((fail + 1)); echo "FAIL - $1"; [ -n "${2:-}" ] && echo "       $2"; }

# Skip honestly if the box already has just on the sanitized PATH.
if PATH="/usr/bin:/bin" command -v just >/dev/null 2>&1; then
  echo "skip - just is already on /usr/bin:/bin, so these fixtures cannot start from an empty PATH"
  exit 0
fi

# A fake release tarball holding a `just` script, and its real sum.
mkdir -p "$ROOT/pkg"
printf '#!/bin/sh\necho just\n' > "$ROOT/pkg/just"; chmod +x "$ROOT/pkg/just"
tar -czf "$ROOT/just.tar.gz" -C "$ROOT/pkg" just
SUM="$(sha256sum "$ROOT/just.tar.gz" | cut -d' ' -f1)"

#   STUB_CURL=ok|fail|stall   what the stub curl does
sandbox() {
  CASE="$ROOT/$1"
  rm -rf "$CASE"
  mkdir -p "$CASE/home" "$CASE/stubs"
  : > "$CASE/env"
  cat > "$CASE/stubs/curl" <<STUB
#!/bin/sh
case "\${STUB_CURL:-ok}" in
  fail)  exit 22 ;;
  stall) sleep 30 ;;
esac
while [ \$# -gt 0 ]; do [ "\$1" = -o ] && out="\$2"; shift; done
cp "$ROOT/just.tar.gz" "\$out"
STUB
  chmod +x "$CASE/stubs/curl"
  # The hook needs real tar, sha256sum, timeout, install, mktemp, uname; stub dir only adds curl.
}

exec_hook() {
  local start end
  start=$(date +%s)
  OUT="$(env -i HOME="$CASE/home" PATH="$CASE/stubs:/usr/bin:/bin" CLAUDE_ENV_FILE="$CASE/env" "$@" bash "$HOOK" 2>&1)"
  RC=$?
  end=$(date +%s)
  SECS=$((end - start))
}

run() { local n="$1"; shift; sandbox "$n"; exec_hook "$@"; }

# 1. Local checkout: does nothing.
run local
[ "$RC" = 0 ] && [ -z "$OUT" ] && ok "local session: silent, exit 0" || bad "local session" "rc=$RC out=$OUT"

# 2. just already on PATH: silent, nothing created.
sandbox present
printf '#!/bin/sh\n' > "$CASE/stubs/just"; chmod +x "$CASE/stubs/just"
exec_hook CLAUDE_CODE_REMOTE=true
[ "$RC" = 0 ] && [ -z "$OUT" ] && [ ! -e "$CASE/home/.just" ] && ok "just on PATH: silent, nothing created" || bad "just on PATH" "rc=$RC out=$OUT"

# 3. Happy path (checksum overridden to the fixture's), then idempotent re-run.
run happy CLAUDE_CODE_REMOTE=true JUST_SHA256="$SUM"
if [ "$RC" = 0 ] && [ -x "$CASE/home/.just/bin/just" ] && echo "$OUT" | grep -q "just 1.58.0 installed" && grep -q "$CASE/home/.just/bin" "$CASE/env"; then
  ok "install: binary placed, PATH written, one line"
else
  bad "install" "rc=$RC out=$OUT"
fi
exec_hook CLAUDE_CODE_REMOTE=true JUST_SHA256="$SUM"
LINES="$(grep -c "$CASE/home/.just/bin" "$CASE/env")"
[ "$RC" = 0 ] && [ "$LINES" = 1 ] && echo "$OUT" | grep -q "just found" && ok "re-run: found, PATH written once" || bad "re-run" "rc=$RC lines=$LINES out=$OUT"

# 4. Wrong checksum: refused, nothing installed.
run badsum CLAUDE_CODE_REMOTE=true
[ "$RC" = 0 ] && [ ! -e "$CASE/home/.just/bin/just" ] && echo "$OUT" | grep -q "pinned checksum" && ok "checksum mismatch: nothing installed, guidance, exit 0" || bad "checksum mismatch" "rc=$RC out=$OUT"

# 5. Network blocked: one guidance line, exit 0.
run blocked CLAUDE_CODE_REMOTE=true STUB_CURL=fail
[ "$RC" = 0 ] && [ "$(echo "$OUT" | wc -l)" = 1 ] && echo "$OUT" | grep -q "just not installed" && ok "network blocked: one guidance line, exit 0" || bad "network blocked" "rc=$RC out=$OUT"

# 6. Stalled download: bounded, graceful.
run stall CLAUDE_CODE_REMOTE=true STUB_CURL=stall JUST_INSTALL_TIMEOUT=1
if [ "$RC" = 0 ] && [ "$SECS" -lt 15 ] && echo "$OUT" | grep -qi "timed out"; then
  ok "stalled download: stopped after the limit (${SECS}s), guidance line, exit 0"
else
  bad "stalled download" "rc=$RC secs=$SECS out=$OUT"
fi

# 7. Unpinned version without a checksum: refused.
run unpinned CLAUDE_CODE_REMOTE=true JUST_VERSION=9.9.9
[ "$RC" = 0 ] && echo "$OUT" | grep -q "no pinned checksum" && ok "unpinned version: refused, exit 0" || bad "unpinned version" "rc=$RC out=$OUT"

echo "$pass passed, $fail failed"
[ "$fail" = 0 ]
