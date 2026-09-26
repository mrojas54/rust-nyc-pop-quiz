"""The `Runner` seam (SPEC.md 7.2, BUILDPLAN D-18).

The verifier never touches `rustc`, a compiled binary or Miri directly. It goes
through a `Runner`, and there are two of them:

* `StubRunner` replays outputs that were recorded once from the real toolchain.
  This is what `just test` uses, which is how the verifier's decision logic gets
  proven in under a minute with no toolchain, no Docker and no network.
* `RealRunner` executes inside the sandbox image, through
  `popquiz.sandbox.run_in_sandbox` - the one module allowed to start a process.
  This module starts nothing itself, which is why the shell-out guard in
  `test_runner.py` still exempts exactly one file.

The split matters beyond speed. `EVALUATION.md` requires the *same cases* to run
against the stub in `test` and against the real toolchain in `test-full`, so a
recorded output that drifts from what the toolchain now does shows up as a
disagreement between the two hooks rather than as a silent pass.

**The step vocabulary** (`kind`), fixed here and nowhere else:

* `toolchain` - the image's `rustc -Vv` then `cargo miri --version`; no candidate.
* `compile` - compile under the pinned edition and flag set; diagnostics only.
* `run:<i>` - the i-th native run, `i` from 1. Each is its own container.
* `miri:<model>:<seed>` - Miri under one borrow model (`bank.BORROW_MODELS`) and
  one seed.
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from popquiz import sandbox

TOOLCHAIN = "toolchain"
COMPILE = "compile"


def run_kind(i: int) -> str:
    """The kind of the i-th native run, counting from 1."""
    return f"run:{i}"


def miri_kind(model: str, seed: int) -> str:
    return f"miri:{model}:{seed}"


class FixtureError(Exception):
    """A recorded output is missing, malformed, or does not say where it came from."""


@dataclass(frozen=True)
class RunStep:
    """One execution the verifier asked for, and what came back.

    `recorded_from` says which toolchain produced this output. It is required,
    and it is the reason this dataclass exists rather than a bare tuple: the
    house rule is that nobody writes down what a program prints, so a replayed
    output has to carry the provenance of the run that produced it. A fixture
    that cannot name its toolchain is refused rather than replayed.

    `exit_code` is `None` when the step never ended on its own (the sandbox
    killed it at the deadline). `stopped_by` names the limits that fired
    (`sandbox.TIMEOUT`, `sandbox.MEMORY_LIMIT`); empty for an ordinary ending.
    `wall_clock_s` is how long the step took, when it was measured - the Miri
    wall-clock is a number Q-E4 asks for.
    """

    kind: str
    exit_code: int | None
    stdout: str
    stderr: str
    recorded_from: str
    stopped_by: tuple[str, ...] = ()
    wall_clock_s: float | None = None

    def to_dict(self) -> dict[str, object]:
        """The step as a fixture stores it, keyed by `kind` one level up."""
        out: dict[str, object] = {
            "exit_code": self.exit_code,
            "stdout": self.stdout,
            "stderr": self.stderr,
            "recorded_from": self.recorded_from,
        }
        if self.stopped_by:
            out["stopped_by"] = list(self.stopped_by)
        if self.wall_clock_s is not None:
            out["wall_clock_s"] = round(self.wall_clock_s, 3)
        return out


class Runner(Protocol):
    """How the verifier reaches a toolchain."""

    def run(self, kind: str, program_id: str) -> RunStep: ...


def is_bookkeeping(key: str) -> bool:
    """A fixture key that is not a step: provenance of the file as a whole
    (`_recorded`, `_source_sha256`). Never replayed, never listed as a step."""
    return key.startswith("_")


class StubRunner:
    """Replays outputs recorded from the real toolchain.

    Fixtures live one JSON file per program under `directory`, each a mapping of
    `kind` to a recorded step, beside `_`-prefixed bookkeeping keys that say how
    the file was recorded. See `tests/fixtures/README.md` for how a real one is
    produced - by running it on T-15a's image, never by hand.
    """

    def __init__(self, directory: Path) -> None:
        self._directory = Path(directory)

    def run(self, kind: str, program_id: str) -> RunStep:
        path = self._directory / f"{program_id}.json"
        if not path.is_file():
            raise FixtureError(
                f"no recorded output for program {program_id!r} at {path}. "
                "Record it on the T-15a image; do not write one by hand."
            )

        recorded = json.loads(path.read_text(encoding="utf-8"))
        steps = {k: v for k, v in recorded.items() if not is_bookkeeping(k)}
        if is_bookkeeping(kind) or kind not in steps:
            raise FixtureError(
                f"program {program_id!r} has no recorded {kind!r} step "
                f"(it has: {', '.join(sorted(steps)) or 'nothing'})"
            )

        step = steps[kind]
        # Provenance is not optional. A step that cannot say which toolchain
        # produced it is indistinguishable from someone having typed the output,
        # and that is the one thing this project does not allow.
        if not step.get("recorded_from"):
            raise FixtureError(
                f"the {kind!r} step of {program_id!r} does not say what it was "
                "recorded from. Every replayed output names its toolchain."
            )

        return RunStep(
            kind=kind,
            exit_code=step["exit_code"],
            stdout=step["stdout"],
            stderr=step["stderr"],
            recorded_from=step["recorded_from"],
            stopped_by=tuple(step.get("stopped_by", ())),
            wall_clock_s=step.get("wall_clock_s"),
        )


class RealRunner:
    """Executes against the pinned toolchain inside the sandbox image (AC-12).

    `sources` maps a program id to its source. Every step is one
    `sandbox.run_in_sandbox` call - no network, no environment, no host
    filesystem, the pin's limits - and `process_runner` is passed straight
    through to it, so a test can drive this class without Docker exactly the way
    `test_sandbox.py` drives the sandbox.

    The `toolchain` step does not depend on the program and is asked once per
    runner: the image cannot change under a running verification.
    """

    def __init__(
        self,
        sources: Mapping[str, str],
        *,
        pin: sandbox.Pin | None = None,
        process_runner: sandbox.Runner | None = None,
    ) -> None:
        self._sources = dict(sources)
        self._pin = pin
        self._process_runner = process_runner
        self._toolchain: RunStep | None = None

    @property
    def pin(self) -> sandbox.Pin:
        if self._pin is None:
            self._pin = sandbox.read_pin()
        return self._pin

    def run(self, kind: str, program_id: str) -> RunStep:
        if kind == TOOLCHAIN:
            if self._toolchain is None:
                self._toolchain = self._execute(kind, "", "toolchain")
            return self._toolchain

        if program_id not in self._sources:
            raise KeyError(f"RealRunner was given no source for program {program_id!r}")
        source = self._sources[program_id]

        if kind == COMPILE:
            return self._execute(kind, source, "compile")
        head, _, rest = kind.partition(":")
        if head == "run" and rest.isdigit() and int(rest) >= 1:
            return self._execute(kind, source, "run")
        if head == "miri":
            model, _, seed = rest.partition(":")
            if model in sandbox.MIRI_CONFIGS and seed.isdigit():
                return self._execute(
                    kind, source, "miri", miri_config=model, miri_seed=int(seed)
                )
        raise ValueError(f"unknown step kind {kind!r}")

    def _execute(self, kind: str, source: str, mode: str, **miri: object) -> RunStep:
        pin = self.pin
        result = sandbox.run_in_sandbox(
            source, mode, pin=pin, runner=self._process_runner, **miri  # type: ignore[arg-type]
        )
        stopped = tuple(
            v for v in (sandbox.TIMEOUT, sandbox.MEMORY_LIMIT) if v in result.verdicts
        )
        return RunStep(
            kind=kind,
            exit_code=result.exit_code,
            stdout=result.stdout,
            stderr=result.stderr,
            recorded_from=f"{pin.image}{f' ({pin.platform})' if pin.platform else ''}",
            stopped_by=stopped,
            wall_clock_s=result.wall_clock_s,
        )
