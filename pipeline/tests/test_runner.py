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

# `tests/sandbox` is the AC-12 containment suite. It runs real containers, so of
# course it starts processes — and `just test` never collects it: `test-pipeline`
# passes `--ignore=tests/sandbox`, and that directory's conftest refuses to run
# unless asked. Scanning it would make this guard fail for files that are not part
# of what it is a guard over.
#
# The exclusion is only sound while the ignore is real, so it is not taken on
# trust: `test_sandbox.py::test_the_containment_suite_is_not_collected_by_the_inner_loop`
# asserts the flag is in the justfile and the conftest's gate is in place. Remove
# either and that test fails, which is what keeps this line honest.
NOT_PART_OF_JUST_TEST = (HERE / "sandbox",)

# Importing any of these gives a module the ability to start a process.
# `importlib` is on the list because it is how a forbidden import gets made
# without looking like one. That is broader than the threat — `importlib.
# resources` starts nothing — so if a later ticket wants it for something
# innocent, narrow this to the dynamic-import calls rather than dropping the
# entry.
FORBIDDEN_IMPORTS = {"subprocess", "multiprocessing", "pty", "docker", "importlib"}

# The modules that are allowed to start a process, by path relative to
# `pipeline/`. T-15a narrowed this from "nobody" to "exactly one", as the guard's
# own docstring below asks, so that the set stays visible at a glance instead of
# the guard being deleted the first time something legitimately needs to execute.
#
# `popquiz/sandbox.py` owns execution: it is the only caller of `docker run`, and
# `test_sandbox.py` proves separately that importing it starts nothing, which is
# what lets `just test` import it at all.
#
# The check below is two-directional. An offender that is not on this list fails,
# and an entry on this list that no longer offends fails too — a one-way
# allowlist rots into a rubber stamp, and an exemption nobody needs any more is
# exactly the kind of stale permission that lets the next one in unnoticed.
#
# Paths are relative to `pipeline/`, which is the one base that is unambiguous for
# both halves of SCANNED — `src/popquiz` and `tests` sit side by side under it.
MAY_START_A_PROCESS = {"src/popquiz/sandbox.py"}

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

    **Narrowed by T-15a, not deleted.** `popquiz/sandbox.py` runs `docker`, so it
    is on `MAY_START_A_PROCESS`. Two things keep that from being a hole: the check
    is two-directional, so the exemption cannot outlive its reason; and
    `test_sandbox.py` proves the exempt module executes nothing at import time,
    which is the property `just test` actually needs — not "imports nothing that
    could execute" but "executes nothing".
    """
    found: dict[str, list[str]] = {}
    for root in SCANNED:
        for path in sorted(root.rglob("*.py")):
            if any(path.is_relative_to(excluded) for excluded in NOT_PART_OF_JUST_TEST):
                continue
            tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
            where = str(path.relative_to(HERE.parent))
            for lineno, what in _process_starting_nodes(tree):
                found.setdefault(where, []).append(f"{where}:{lineno} {what}")

    unexpected, stale = _classify(found, MAY_START_A_PROCESS)
    assert not unexpected, (
        "something `just test` imports can start a process, and it is not the one "
        "module allowed to: " + "; ".join(unexpected)
    )
    assert not stale, (
        "these modules are exempted from the shell-out guard but no longer start "
        "anything, so the exemption should be removed rather than left standing: "
        + ", ".join(stale)
    )


def _classify(
    found: dict[str, list[str]], allowed: set[str]
) -> tuple[list[str], list[str]]:
    """Split the scan's findings into offenders nobody exempted and exemptions
    nobody needs. Exact path comparison, on purpose: see the test below."""
    unexpected = [line for where, lines in found.items() if where not in allowed for line in lines]
    stale = sorted(allowed - found.keys())
    return unexpected, stale


def test_the_exemption_covers_one_exact_path_and_nothing_near_it() -> None:
    """The exemption is only as narrow as the comparison behind it. A prefix or a
    directory match would let a sibling module — or one whose name merely starts
    the same way — inherit the right to start a process without appearing on the
    list, which is the one thing the list exists to prevent."""
    exempt = "src/popquiz/sandbox.py"
    found = {
        exempt: [f"{exempt}:1 imports subprocess"],
        "src/popquiz/verify.py": ["src/popquiz/verify.py:3 imports subprocess"],
        "src/popquiz/sandbox_helpers.py": ["src/popquiz/sandbox_helpers.py:2 calls os.system"],
        "src/popquiz/sandbox/extra.py": ["src/popquiz/sandbox/extra.py:5 imports pty"],
    }

    unexpected, stale = _classify(found, {exempt})

    assert not stale
    assert not [line for line in unexpected if line.startswith(f"{exempt}:")]
    assert {line.split(":", 1)[0] for line in unexpected} == {
        "src/popquiz/verify.py",
        "src/popquiz/sandbox_helpers.py",
        "src/popquiz/sandbox/extra.py",
    }

    # And the other direction: an exemption with no offence behind it is reported.
    _, stale = _classify({}, {exempt})
    assert stale == [exempt]


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


def test_every_shell_out_exemption_names_a_file_that_exists() -> None:
    """A misspelled exemption would surface as a *stale* one, which is a
    misleading way to find out you typed a path wrong. This says which it is."""
    for where in sorted(MAY_START_A_PROCESS):
        assert (HERE.parent / where).is_file(), (
            f"{where} is exempted from the shell-out guard, but there is no such "
            f"file under {HERE.parent}. Fix the path or drop the entry."
        )
