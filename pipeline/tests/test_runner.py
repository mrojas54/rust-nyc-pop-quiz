"""The `Runner` seam, and the guarantee that `just test` cannot shell out.

These tests are about the harness, not about verification. The verifier's actual
decisions — determinism at N=5, compile failure with its error code, Miri under
both borrow models — belong to T-15b and arrive with real recorded fixtures.
"""

from __future__ import annotations

import ast
import pathlib

import pytest

from popquiz.runner import FixtureError, RealRunner, RunStep, StubRunner

HERE = pathlib.Path(__file__).parent
FIXTURES = HERE / "fixtures"

# Both halves of what `just test` imports. The tests are scanned too: a test that
# called out to a toolchain directly would make `just test` shell out just as
# surely as a module doing it, and would be an easy way to sidestep the seam.
SCANNED = (HERE.parent / "src" / "popquiz", HERE)

# Importing any of these gives a module the ability to start a process.
# `importlib` is on the list because it is how a forbidden import gets made
# without looking like one. That is broader than the threat — `importlib.
# resources` starts nothing — so if a later ticket wants it for something
# innocent, narrow this to the dynamic-import calls rather than dropping the
# entry.
FORBIDDEN_IMPORTS = {"subprocess", "multiprocessing", "pty", "docker", "importlib"}

# `os` is not forbidden — modules want `os.environ` and `os.fspath`. These are
# the calls on it that start something. Both families are complete: all seven
# `exec*` and all eight `spawn*`, because a guard that covers six of eight while
# saying "the spawn family" is the kind of almost-true this project does not ship.
FORBIDDEN_OS_CALLS = {
    "system", "popen", "fork", "forkpty", "posix_spawn", "posix_spawnp",
    "execl", "execle", "execlp", "execv", "execve", "execvp", "execvpe",
    "spawnl", "spawnle", "spawnlp", "spawnlpe",
    "spawnv", "spawnve", "spawnvp", "spawnvpe",
}


def test_stub_runner_replays_a_recorded_step() -> None:
    step = StubRunner(FIXTURES).run("placeholder-step", "scaffold-placeholder")

    assert isinstance(step, RunStep)
    assert step.kind == "placeholder-step"
    assert step.exit_code == 0
    assert step.recorded_from


def test_stub_runner_refuses_a_step_that_does_not_say_where_it_came_from() -> None:
    """The provenance guard.

    An output with no toolchain behind it is indistinguishable from one somebody
    typed, so it is refused rather than replayed. This is what stops T-15b from
    hand-writing a fixture.
    """
    with pytest.raises(FixtureError, match="recorded from"):
        StubRunner(FIXTURES).run(
            "placeholder-step-without-provenance", "scaffold-placeholder"
        )


def test_stub_runner_reports_a_program_it_has_never_recorded() -> None:
    with pytest.raises(FixtureError, match="no recorded output"):
        StubRunner(FIXTURES).run("placeholder-step", "a-program-nobody-recorded")


def test_stub_runner_reports_a_step_it_has_never_recorded() -> None:
    with pytest.raises(FixtureError, match="no recorded"):
        StubRunner(FIXTURES).run("a-step-nobody-recorded", "scaffold-placeholder")


def test_real_runner_refuses_until_the_sandbox_exists() -> None:
    with pytest.raises(NotImplementedError, match="T-15a/T-15b"):
        RealRunner().run("compile", "anything")


def _process_starting_nodes(tree: ast.AST) -> list[tuple[int, str]]:
    """Every place in one module that could start a process. See the test below
    for exactly how much this is and is not worth."""
    found: list[tuple[int, str]] = []

    for node in ast.walk(tree):
        # import subprocess / import docker.client
        if isinstance(node, ast.Import):
            for alias in node.names:
                if alias.name.split(".")[0] in FORBIDDEN_IMPORTS:
                    found.append((node.lineno, f"imports {alias.name}"))

        elif isinstance(node, ast.ImportFrom):
            root = (node.module or "").split(".")[0]
            # from subprocess import run
            if root in FORBIDDEN_IMPORTS:
                found.append((node.lineno, f"imports from {node.module}"))
            # from os import system — `os` itself stays allowed, but pulling one
            # of its process-starting calls out of it and calling it bare would
            # otherwise slip past the os.<attr>(...) check below.
            elif root == "os":
                for alias in node.names:
                    if alias.name in FORBIDDEN_OS_CALLS:
                        found.append((node.lineno, f"imports os.{alias.name}"))

        elif isinstance(node, ast.Call):
            func = node.func
            # __import__("subprocess") — an import that is not an import statement
            if (
                isinstance(func, ast.Name)
                and func.id == "__import__"
                and node.args
                and isinstance(node.args[0], ast.Constant)
                and isinstance(node.args[0].value, str)
                and node.args[0].value.split(".")[0] in FORBIDDEN_IMPORTS
            ):
                found.append((node.lineno, f"__import__({node.args[0].value!r})"))

            # os.system(...), os.execv(...), and the rest of that family
            elif (
                isinstance(func, ast.Attribute)
                and isinstance(func.value, ast.Name)
                and func.value.id == "os"
                and func.attr in FORBIDDEN_OS_CALLS
            ):
                found.append((node.lineno, f"calls os.{func.attr}"))

    return found


def test_nothing_just_test_imports_can_start_a_process() -> None:
    """`just test` runs no toolchain, so nothing it imports should be able to.

    `EVALUATION.md`'s harness row is explicit: `test` runs neither the pinned
    `rustc` nor Miri nor Docker.

    **What this actually proves, and it is worth being exact about it** — the
    same care the receipt takes about verification applies to a test that claims
    to constrain the whole suite. It catches the ordinary ways a module acquires
    the ability to start a process: importing `subprocess`, `multiprocessing`,
    `pty`, `docker` or `importlib` by any import form, `__import__` with a
    literal name, and the `os.system` / `os.exec*` / `os.spawn*` family.

    It is **not** a sandbox and it does not survive an author who is working
    around it: a name computed at runtime, an import hidden behind `getattr`, or
    a helper in a dependency would all pass. It is here so that the first module
    that genuinely needs to execute something has to say so in a way a reader
    sees, not so that the absence of a finding is a guarantee.

    T-15b will legitimately need to break this — `RealRunner` executes things.
    Narrow it to exempt the one module that owns execution rather than deleting
    it, so the set of modules that can start a process stays visible at a glance.
    """
    offenders = []
    for root in SCANNED:
        for path in sorted(root.rglob("*.py")):
            tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
            for lineno, what in _process_starting_nodes(tree):
                offenders.append(f"{path.relative_to(root.parent)}:{lineno} {what}")

    assert not offenders, (
        "something `just test` imports can start a process, which it must never "
        "do: " + "; ".join(offenders)
    )


def test_the_guard_catches_the_ways_it_claims_to() -> None:
    """The guard above is only worth having if it fires. This proves it does.

    Without this, a refactor could quietly turn the scan into one that walks
    every file and finds nothing because it no longer looks for anything — which
    is indistinguishable, from the outside, from a clean tree.
    """
    # These are strings handed to ast.parse, never executed — the point is that
    # the scanner sees them. Nothing in this suite runs a shell.
    caught = [
        "import subprocess",
        "import docker.client",
        "from subprocess import run",
        "import importlib",
        "__import__('subprocess')",
        "import os\nos.system('ls')",
        "import os\nos.execv('/bin/ls', [])",
        "import os\nos.spawnvpe(os.P_WAIT, 'ls', [], {})",
        "from os import system",
    ]
    for source in caught:
        assert _process_starting_nodes(ast.parse(source)), f"missed: {source!r}"

    # And does not fire on the ordinary uses of `os` that modules legitimately want.
    allowed = ["import os\nos.environ.get('HOME')", "import os\nos.fspath('/tmp')"]
    for source in allowed:
        assert not _process_starting_nodes(ast.parse(source)), (
            f"false positive: {source!r}"
        )
