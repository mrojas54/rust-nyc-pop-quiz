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

FIXTURES = pathlib.Path(__file__).parent / "fixtures"
PACKAGE = pathlib.Path(__file__).parent.parent / "src" / "popquiz"


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


def test_no_module_in_the_package_can_shell_out() -> None:
    """`just test` runs no toolchain, so no module it imports may be able to.

    `EVALUATION.md`'s harness row is explicit: `test` runs neither the pinned
    `rustc` nor Miri nor Docker. That holds today because nothing here can
    execute a subprocess at all, and this test is what keeps it holding.

    T-15b will legitimately need to break this — `RealRunner` executes things.
    When it does, the right move is to narrow this test to exempt the one module
    that owns execution, not to delete it, so that the set of modules able to
    start a process stays something a reader can see at a glance.
    """
    forbidden = {"subprocess", "os.system", "docker", "pty", "multiprocessing"}
    offenders = []

    for path in sorted(PACKAGE.rglob("*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                names = [node.module or ""]
            else:
                continue
            for name in names:
                root = name.split(".")[0]
                if root in forbidden or name in forbidden:
                    offenders.append(f"{path.name}:{node.lineno} imports {name}")

    assert not offenders, (
        "a module in popquiz/ can start a process, which `just test` must never "
        "do: " + "; ".join(offenders)
    )
