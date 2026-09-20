"""The `Runner` seam (SPEC.md 7.2, BUILDPLAN D-18).

The verifier never touches `rustc`, a compiled binary or Miri directly. It goes
through a `Runner`, and there are two of them:

* `StubRunner` replays outputs that were recorded once from the real toolchain.
  This is what `just test` uses, which is how the verifier's decision logic gets
  proven in under a minute with no toolchain, no Docker and no network.
* `RealRunner` executes inside the sandbox image. It does not exist yet — T-15a
  builds the image, T-15b writes the runner — so every call raises.

The split matters beyond speed. `EVALUATION.md` requires the *same cases* to run
against the stub in `test` and against the real toolchain in `test-full`, so a
recorded output that drifts from what the toolchain now does shows up as a
disagreement between the two hooks rather than as a silent pass.

Nothing in this package shells out today, and
`test_nothing_just_test_imports_can_start_a_process` is what makes the first
module that needs to say so out loud. That test catches the ordinary ways, not
every conceivable one; its own docstring is exact about which.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Protocol


class FixtureError(Exception):
    """A recorded output is missing, malformed, or does not say where it came from."""


@dataclass(frozen=True)
class RunStep:
    """One execution the verifier asked for, and what came back.

    `kind` is which step it was — compiling, running the binary, running Miri.
    T-15b fixes the vocabulary; the scaffold does not guess at it.

    `recorded_from` says which toolchain produced this output. It is required,
    and it is the reason this dataclass exists rather than a bare tuple: the
    house rule is that nobody writes down what a program prints, so a replayed
    output has to carry the provenance of the run that produced it. A fixture
    that cannot name its toolchain is refused rather than replayed.
    """

    kind: str
    exit_code: int
    stdout: str
    stderr: str
    recorded_from: str


class Runner(Protocol):
    """How the verifier reaches a toolchain. T-15b fixes the final signature."""

    def run(self, kind: str, program_id: str) -> RunStep: ...


class StubRunner:
    """Replays outputs recorded from the real toolchain.

    Fixtures live one JSON file per program under `directory`, each a mapping of
    `kind` to a recorded step. See `tests/fixtures/README.md` for how a real one
    is produced — by running it on T-15a's image, never by hand.
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
        if kind not in recorded:
            raise FixtureError(
                f"program {program_id!r} has no recorded {kind!r} step "
                f"(it has: {', '.join(sorted(recorded)) or 'nothing'})"
            )

        step = recorded[kind]
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
        )


class RealRunner:
    """Executes against the pinned toolchain inside the sandbox. Not yet built.

    T-15a builds the image and defines the pin — the full `rustc -Vv` release and
    commit-hash, plus the nightly's date for Miri — once, in one place. T-15b
    writes what goes in here: no network, no secrets, no host filesystem, and
    CPU, memory and time limits (AC-12).
    """

    def run(self, kind: str, program_id: str) -> RunStep:
        raise NotImplementedError(
            "not until T-15a/T-15b: T-15a builds the sandbox image and defines "
            "the toolchain pin, T-15b writes the runner that executes in it."
        )
