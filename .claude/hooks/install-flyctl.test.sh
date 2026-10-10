#!/bin/bash
# Fixture tests for install-flyctl.sh. No network: curl and the installer are
# stubs, and every case runs in its own temp HOME. Run: bash .claude/hooks/install-flyctl.test.sh
set -uo pipefail

HOOK="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/install-flyctl.sh"
ROOT="$(mktemp -d)"
trap 'rm -rf "$ROOT"' EXIT
pass=0
fail=0

ok() { pass=$((pass + 1)); echo "ok   - $1"; }
bad() { fail=$((fail + 1)); echo "FAIL - $1"; [ -n "${2:-}" ] && echo "       $2"; }

# Run the hook in a fresh sandbox. Output goes to $OUT, exit status to $RC.
# Args: name, then VAR=value pairs. STUBS names what the sandbox PATH holds.
#   STUB_CURL=ok|fail   what the stub curl does
#   STUB_INSTALL=binary|nobinary|stall   what the fetched installer does
sandbox() {
  local name="$1"
  CASE="$ROOT/$name"
  rm -rf "$CASE"
  mkdir -p "$CASE/home" "$CASE/stubs"
  : > "$CASE/env"

  cat > "$CASE/stubs/curl" <<'STUB'
#!/bin/sh
[ "${STUB_CURL:-ok}" = fail ] && exit 22
case "${STUB_INSTALL:-binary}" in
  binary)   printf 'mkdir -p "$FLYCTL_INSTALL/bin"; printf "#!/bin/sh\\necho fly\\n" > "$FLYCTL_INSTALL/bin/flyctl"; chmod +x "$FLYCTL_INSTALL/bin/flyctl"\n' ;;
  nobinary) printf 'exit 0\n' ;;
  stall)    printf 'sleep 30\n' ;;
esac
STUB
  chmod +x "$CASE/stubs/curl"
}

# Run the hook in the current sandbox with the given VAR=value pairs.
exec_hook() {
  local start end
  start=$(date +%s)
  OUT="$(env -i HOME="$CASE/home" PATH="$CASE/stubs:/usr/bin:/bin" CLAUDE_ENV_FILE="$CASE/env" "$@" bash "$HOOK" 2>&1)"
  RC=$?
  end=$(date +%s)
  SECS=$((end - start))
}

run() {
  local name="$1"; shift
  sandbox "$name"
  exec_hook "$@"
}

# Skip honestly if the box already has fly on the sanitized PATH.
if PATH="/usr/bin:/bin" command -v fly >/dev/null 2>&1 || PATH="/usr/bin:/bin" command -v flyctl >/dev/null 2>&1; then
  echo "skip - fly is already on /usr/bin:/bin, so these fixtures cannot start from an empty PATH"
  exit 0
fi

# 1. Local checkout: does nothing.
run local
[ "$RC" = 0 ] && [ -z "$OUT" ] && ok "local session: silent, exit 0" || bad "local session" "rc=$RC out=$OUT"

# 2. fly already on PATH: silent, nothing created.
sandbox fly_present
printf '#!/bin/sh\n' > "$CASE/stubs/fly"; chmod +x "$CASE/stubs/fly"
exec_hook CLAUDE_CODE_REMOTE=true
[ "$RC" = 0 ] && [ -z "$OUT" ] && [ ! -e "$CASE/home/.fly" ] && ok "fly on PATH: silent, nothing created" || bad "fly on PATH" "rc=$RC out=$OUT"

# 3. Only flyctl on PATH (review finding): the hook must give the session a `fly`.
sandbox flyctl_only
printf '#!/bin/sh\necho flyctl\n' > "$CASE/stubs/flyctl"; chmod +x "$CASE/stubs/flyctl"
exec_hook CLAUDE_CODE_REMOTE=true
exec_hook CLAUDE_CODE_REMOTE=true
PATH_LINE="$(grep -c "$CASE/home/.fly/bin" "$CASE/env")"
if [ "$RC" = 0 ] && [ -x "$CASE/home/.fly/bin/fly" ] && [ "$PATH_LINE" = 1 ] \
   && [ "$(env -i PATH="$CASE/home/.fly/bin:/usr/bin:/bin" sh -c 'fly')" = flyctl ]; then
  ok "only flyctl on PATH: fly is created, PATH written once across two runs"
else
  bad "only flyctl on PATH" "rc=$RC path_lines=$PATH_LINE out=$OUT"
fi

# 4. Binary already under FLYCTL_INSTALL but not on PATH.
sandbox cached
mkdir -p "$CASE/fly/bin"; printf '#!/bin/sh\n' > "$CASE/fly/bin/flyctl"; chmod +x "$CASE/fly/bin/flyctl"
exec_hook CLAUDE_CODE_REMOTE=true FLYCTL_INSTALL="$CASE/fly"
[ "$RC" = 0 ] && [ -e "$CASE/fly/bin/fly" ] && grep -q "$CASE/fly/bin" "$CASE/env" && ok "cached install: found, fly linked, PATH written" || bad "cached install" "rc=$RC out=$OUT"

# 5. Network blocked: one guidance line, exit 0.
run blocked CLAUDE_CODE_REMOTE=true STUB_CURL=fail
[ "$RC" = 0 ] && [ "$(echo "$OUT" | wc -l)" = 1 ] && echo "$OUT" | grep -q "flyctl not installed" && ok "network blocked: one guidance line, exit 0" || bad "network blocked" "rc=$RC out=$OUT"

# 6. Installer finishes but leaves no binary.
run nobinary CLAUDE_CODE_REMOTE=true STUB_INSTALL=nobinary
[ "$RC" = 0 ] && echo "$OUT" | grep -q "flyctl not installed" && ok "installer left no binary: guidance, exit 0" || bad "installer left no binary" "rc=$RC out=$OUT"

# 7. Happy path.
run happy CLAUDE_CODE_REMOTE=true
[ "$RC" = 0 ] && [ -x "$CASE/home/.fly/bin/fly" ] && echo "$OUT" | grep -q "flyctl installed" && ok "install: binary placed, fly linked, one line" || bad "install" "rc=$RC out=$OUT"

# 8. A stalled installer (review finding): bounded, graceful.
run stall CLAUDE_CODE_REMOTE=true STUB_INSTALL=stall FLYCTL_INSTALL_TIMEOUT=1
if [ "$RC" = 0 ] && [ "$SECS" -lt 15 ] && echo "$OUT" | grep -q "flyctl not installed" && echo "$OUT" | grep -qi "timed out"; then
  ok "stalled installer: stopped after the limit (${SECS}s), guidance line, exit 0"
else
  bad "stalled installer" "rc=$RC secs=$SECS out=$OUT"
fi

echo "$pass passed, $fail failed"
[ "$fail" = 0 ]
