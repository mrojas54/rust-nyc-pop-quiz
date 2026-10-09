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

on_path() {
  # `fly` is the name the docs use; `flyctl` is the same binary.
  command -v fly >/dev/null 2>&1 || command -v flyctl >/dev/null 2>&1
}

persist_path() {
  # Make the install visible to the session's later commands.
  if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
    grep -qs "$FLY_BIN" "$CLAUDE_ENV_FILE" 2>/dev/null ||
      echo "export PATH=\"$FLY_BIN:\$PATH\"" >> "$CLAUDE_ENV_FILE"
  fi
}

if on_path; then
  exit 0
fi

if [ -x "$FLY_BIN/flyctl" ] || [ -x "$FLY_BIN/fly" ]; then
  [ -e "$FLY_BIN/fly" ] || ln -s "$FLY_BIN/flyctl" "$FLY_BIN/fly"
  persist_path
  echo "flyctl found in $FLY_BIN; added to PATH for this session"
  exit 0
fi

blocked() {
  echo "flyctl not installed: $1. Allow fly.io and github.com in the environment's network settings (https://code.claude.com/docs/en/cloud-environments#network-access), or put the install in the environment's setup script."
  exit 0
}

command -v curl >/dev/null 2>&1 || blocked "curl is missing"

# Fly's official installer. -f so an HTTP error (a network policy's 403) is a
# failure, not a page of HTML handed to sh.
script="$(curl -fsSL --max-time 30 https://fly.io/install.sh 2>/dev/null)" || blocked "fly.io/install.sh is unreachable"

if ! FLYCTL_INSTALL="$FLY_HOME" sh -c "$script" >/dev/null 2>&1; then
  blocked "the installer ran but could not download flyctl"
fi

if [ ! -x "$FLY_BIN/flyctl" ]; then
  blocked "the installer finished without a flyctl binary"
fi

[ -e "$FLY_BIN/fly" ] || ln -s "$FLY_BIN/flyctl" "$FLY_BIN/fly"
persist_path
echo "flyctl installed in $FLY_BIN and added to PATH for this session"
exit 0
