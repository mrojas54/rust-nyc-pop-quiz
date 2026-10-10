#!/bin/bash
# Installs flyctl in Claude Code cloud sessions, so `fly deploy` is available
# (room/README.md, Deploying). Remote sessions only; idempotent; and it never
# fails session start: a blocked network gets one line of guidance and exit 0.
#
# It installs the binary and nothing else. FLY_API_TOKEN is a separate
# environment secret, never written, read or printed here.
set -uo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

FLY_HOME="${FLYCTL_INSTALL:-$HOME/.fly}"
FLY_BIN="$FLY_HOME/bin"

# The whole install, fetch and download, may take this long (seconds). Past it
# the hook gives up with the guidance line instead of holding the session start.
INSTALL_LIMIT="${FLYCTL_INSTALL_TIMEOUT:-120}"

persist_path() {
  # Make the install visible to the session's later commands.
  if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
    grep -qs "$FLY_BIN" "$CLAUDE_ENV_FILE" 2>/dev/null ||
      echo "export PATH=\"$FLY_BIN:\$PATH\"" >> "$CLAUDE_ENV_FILE"
  fi
}

blocked() {
  echo "flyctl not installed: $1. Allow fly.io and github.com in the environment's network settings (https://code.claude.com/docs/en/cloud-environments#network-access), or put the install in the environment's setup script."
  exit 0
}

# `fly` is the name the docs use (`fly deploy ...`), so it is `fly` that must
# exist, not just `flyctl`.
if command -v fly >/dev/null 2>&1; then
  exit 0
fi

# Only `flyctl` is on PATH: give the session a `fly` beside it, in a directory
# this hook owns, never in a system one.
if existing="$(command -v flyctl 2>/dev/null)"; then
  mkdir -p "$FLY_BIN" && ln -sf "$existing" "$FLY_BIN/fly" || blocked "flyctl is installed but a fly link could not be made in $FLY_BIN"
  persist_path
  echo "flyctl found at $existing; linked as fly in $FLY_BIN and added to PATH for this session"
  exit 0
fi

if [ -x "$FLY_BIN/flyctl" ] || [ -x "$FLY_BIN/fly" ]; then
  [ -e "$FLY_BIN/fly" ] || ln -s "$FLY_BIN/flyctl" "$FLY_BIN/fly"
  persist_path
  echo "flyctl found in $FLY_BIN; added to PATH for this session"
  exit 0
fi

command -v curl >/dev/null 2>&1 || blocked "curl is missing"
command -v timeout >/dev/null 2>&1 || blocked "timeout is missing, so the install could not be bounded"

# Fly's official installer. -f so an HTTP error (a network policy's 403) is a
# failure, not a page of HTML handed to sh.
script="$(curl -fsSL --connect-timeout 10 --max-time 30 https://fly.io/install.sh 2>/dev/null)" || blocked "fly.io/install.sh is unreachable"

# The installer makes its own release lookup and download, with no limit of its
# own. `timeout` bounds the whole run and signals its process group, so the
# installer's curl calls stop too; expiry takes the same graceful path.
FLYCTL_INSTALL="$FLY_HOME" timeout --kill-after=5 "$INSTALL_LIMIT" sh -c "$script" >/dev/null 2>&1
status=$?
case "$status" in
  0) ;;
  124 | 137) blocked "the installer timed out after ${INSTALL_LIMIT}s" ;;
  *) blocked "the installer ran but could not download flyctl" ;;
esac

if [ ! -x "$FLY_BIN/flyctl" ]; then
  blocked "the installer finished without a flyctl binary"
fi

[ -e "$FLY_BIN/fly" ] || ln -s "$FLY_BIN/flyctl" "$FLY_BIN/fly"
persist_path
echo "flyctl installed in $FLY_BIN and added to PATH for this session"
exit 0
