"""The one guard that keeps every test away from the real `claude` (PQ-69 ruling 7, PQ-70).

The invariant is *the real `claude` is unreachable from any test*, not *every test
has a scrubbed PATH*: the Docker suites under `tests/sandbox/` need `docker`,
`cargo`, `uv` and the daemon's socket, so nothing here scrubs PATH or refuses
sockets for every test. Instead the guard refuses at the only two places a test can
reach the binary, for every test in every directory below this one:

- building a `ClaudeCliClient` whose resolved binary is the real `claude`, whether
  given by path or found on PATH (`binary=None`);
- starting a process (`subprocess.Popen`, which `subprocess.run` and the generator's
  own child go through) that is the real `claude`, or that is the generator on the
  claude-cli backend with `--claude-bin` (or, without it, `claude` on the child's
  PATH) resolving to it.

Either one fails the test before anything is spawned. "The real `claude`" is
whatever `claude` resolves to on the PATH pytest started with, plus any of the
standard install locations that exists.

`hermetic` is the PATH scrub, billing-variable removal and socket refusal for the
generator's own suites, which take it with `pytest.mark.usefixtures("hermetic")`.
"""

from __future__ import annotations

import os
import shutil
import socket
import stat
import subprocess
from collections.abc import Mapping, Sequence
from pathlib import Path

import pytest

from popquiz import generate_cli

SAFE_PATH = "/usr/bin:/bin"

_INSTALL_DIRS = ("~/.local/bin", "~/.claude/local", "/opt/homebrew/bin", "/usr/local/bin")


def _known_claudes() -> set[str]:
    found = [shutil.which("claude")]
    found += [os.path.join(os.path.expanduser(d), "claude") for d in _INSTALL_DIRS]
    return {os.path.realpath(p) for p in found if p and os.path.isfile(p)}


#: Realpaths a test must never build a client on or start. A test adds a decoy to
#: it (`claude_decoy`) to prove the guard without going near the real one.
REAL_CLAUDE: set[str] = _known_claudes()


def is_real_claude(path: str | os.PathLike[str] | None) -> bool:
    return path is not None and os.path.realpath(path) in REAL_CLAUDE


def _refuse(what: str) -> None:
    pytest.fail(f"the real claude guard (tests/conftest.py) refused {what}", pytrace=False)


def _resolve(name: str, env: Mapping[str, str] | None, cwd: str | os.PathLike[str] | None) -> str | None:
    if os.sep in name:
        return os.path.join(os.fspath(cwd), name) if cwd is not None and not os.path.isabs(name) else name
    return shutil.which(name, path=(os.environ if env is None else env).get("PATH"))


def _check_command(args: object, env: Mapping[str, str] | None, cwd: str | os.PathLike[str] | None) -> None:
    if isinstance(args, (str, bytes, os.PathLike)):
        argv = [os.fsdecode(args)]
    elif isinstance(args, Sequence):
        argv = [os.fsdecode(a) for a in args]
    else:
        return
    if not argv:
        return
    if is_real_claude(_resolve(argv[0], env, cwd)):
        _refuse(f"starting {argv[0]}")
    runs_generator = "popquiz.generate" in argv and "claude-cli" in argv
    if not runs_generator:
        return
    given = None
    for i, arg in enumerate(argv):
        if arg == "--claude-bin" and i + 1 < len(argv):
            given = argv[i + 1]
        elif arg.startswith("--claude-bin="):
            given = arg.split("=", 1)[1]
    binary = _resolve(given if given is not None else "claude", env, cwd)
    if is_real_claude(binary):
        _refuse(f"a claude-cli generator run on {binary}")


class _GuardedPopen(subprocess.Popen):
    def __init__(self, args, *rest, **kwargs):
        _check_command(args, kwargs.get("env"), kwargs.get("cwd"))
        super().__init__(args, *rest, **kwargs)


@pytest.fixture(autouse=True)
def real_claude_unreachable(monkeypatch):
    """Every test: no client on the real `claude`, and no process that is it or runs it."""
    original = generate_cli.ClaudeCliClient.__init__

    def guarded(self, *args, **kwargs):
        original(self, *args, **kwargs)
        if is_real_claude(self.binary):
            _refuse(f"a ClaudeCliClient on {self.binary}")

    monkeypatch.setattr(generate_cli.ClaudeCliClient, "__init__", guarded)
    monkeypatch.setattr(subprocess, "Popen", _GuardedPopen)


@pytest.fixture
def hermetic(monkeypatch):
    """No `claude` on PATH, no billing or provider variable, no network."""
    monkeypatch.setenv("PATH", SAFE_PATH)
    for name in list(os.environ):
        if generate_cli.is_billing_switch(name):
            monkeypatch.delenv(name)

    def refuse(*args, **kwargs):
        raise AssertionError("a hermetic test tried to use the network")

    monkeypatch.setattr(socket.socket, "connect", refuse)
    monkeypatch.setattr(socket.socket, "connect_ex", refuse)
    monkeypatch.setattr(socket, "create_connection", refuse)
    monkeypatch.setattr(socket, "getaddrinfo", refuse)


class Decoy:
    """An executable registered as the real `claude` for one test. Running it leaves
    a marker, so a test can show the guard stopped it before it started."""

    def __init__(self, tmp_path: Path) -> None:
        self.dir = tmp_path / "decoy"
        self.dir.mkdir()
        self.exe = self.dir / "claude"
        self.marker = tmp_path / "decoy-ran"
        self.exe.write_text(f'#!/bin/sh\ntouch "{self.marker}"\nexit 0\n', encoding="utf-8")
        self.exe.chmod(self.exe.stat().st_mode | stat.S_IXUSR)

    def ran(self) -> bool:
        return self.marker.exists()


@pytest.fixture
def claude_decoy(tmp_path):
    decoy = Decoy(tmp_path)
    real = os.path.realpath(decoy.exe)
    REAL_CLAUDE.add(real)
    try:
        yield decoy
    finally:
        REAL_CLAUDE.discard(real)
