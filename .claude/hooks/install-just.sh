#!/bin/bash
# Installs `just` in Claude Code cloud sessions, so `just test` and
# `just test-full` (the justfile) run there. Remote sessions only; idempotent;
# and it never fails session start: a blocked network gets one line of guidance
# and exit 0.
#
# It downloads one prebuilt release tarball from GitHub, checks it against a
# pinned SHA-256, and unpacks the `just` binary into a directory this hook owns.
# No `cargo install`, no installer script. Bump JUST_VERSION and both sums
# together (the release's SHA256SUMS file lists them).
set -uo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

# The justfile uses `[parallel]`, which needs just 1.42 or newer.
JUST_VERSION="${JUST_VERSION:-1.58.0}"
JUST_HOME="${JUST_INSTALL:-$HOME/.just}"
JUST_BIN="$JUST_HOME/bin"

# The whole download may take this long (seconds). Past it the hook gives up with
# the guidance line instead of holding the session start.
INSTALL_LIMIT="${JUST_INSTALL_TIMEOUT:-60}"

persist_path() {
  # Make the install visible to the session's later commands.
  if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
    grep -qs "$JUST_BIN" "$CLAUDE_ENV_FILE" 2>/dev/null ||
      echo "export PATH=\"$JUST_BIN:\$PATH\"" >> "$CLAUDE_ENV_FILE"
  fi
}

blocked() {
  echo "just not installed: $1. Allow github.com in the environment's network settings (https://code.claude.com/docs/en/cloud-environments#network-access), or put the install in the environment's setup script."
  exit 0
}

if command -v just >/dev/null 2>&1; then
  exit 0
fi

if [ -x "$JUST_BIN/just" ]; then
  persist_path
  echo "just found in $JUST_BIN; added to PATH for this session"
  exit 0
fi

case "$(uname -m)" in
  x86_64 | amd64)
    target="x86_64-unknown-linux-musl"
    sha256="4a5cc2f53e6f0f8c59092a6cc38291eb729d46a7dd95d3ae582008881b84931d" ;;
  aarch64 | arm64)
    target="aarch64-unknown-linux-musl"
    sha256="748237128c4c40cbdabc65e841d05ceba13cc23a91eaba395495894c1d9764df" ;;
  *) blocked "no prebuilt binary is pinned for $(uname -m)" ;;
esac

# The sums above belong to 1.58.0; another version needs its own.
if [ "$JUST_VERSION" != "1.58.0" ] && [ -z "${JUST_SHA256:-}" ]; then
  blocked "JUST_VERSION=$JUST_VERSION has no pinned checksum (set JUST_SHA256 with it)"
fi
sha256="${JUST_SHA256:-$sha256}"

command -v curl >/dev/null 2>&1 || blocked "curl is missing"
command -v tar >/dev/null 2>&1 || blocked "tar is missing"
command -v sha256sum >/dev/null 2>&1 || blocked "sha256sum is missing, so the download could not be verified"
command -v timeout >/dev/null 2>&1 || blocked "timeout is missing, so the install could not be bounded"

work="$(mktemp -d)" || blocked "no temporary directory"
trap 'rm -rf "$work"' EXIT

url="https://github.com/casey/just/releases/download/$JUST_VERSION/just-$JUST_VERSION-$target.tar.gz"
# -f so an HTTP error (a network policy's 403) is a failure, not a page of HTML
# handed to tar. `timeout` bounds the whole transfer.
timeout --kill-after=5 "$INSTALL_LIMIT" curl -fsSL --connect-timeout 10 -o "$work/just.tar.gz" "$url" >/dev/null 2>&1
status=$?
case "$status" in
  0) ;;
  124 | 137) blocked "the download timed out after ${INSTALL_LIMIT}s" ;;
  *) blocked "$url is unreachable" ;;
esac

echo "$sha256  $work/just.tar.gz" | sha256sum -c --status - || blocked "the download did not match its pinned checksum"

mkdir -p "$JUST_BIN" || blocked "$JUST_BIN could not be created"
tar -xzf "$work/just.tar.gz" -C "$work" just 2>/dev/null || blocked "the tarball had no just binary"
install -m 0755 "$work/just" "$JUST_BIN/just" 2>/dev/null || blocked "just could not be placed in $JUST_BIN"

persist_path
echo "just $JUST_VERSION installed in $JUST_BIN and added to PATH for this session"
exit 0
