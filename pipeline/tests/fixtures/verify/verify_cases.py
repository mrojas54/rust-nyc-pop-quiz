"""The verifier's cases, read from `cases.toml`, for the three places that run them.

`tests/test_verify.py` replays each case on `StubRunner`; the real-toolchain twin in
`tests/sandbox/verify/` runs it on `RealRunner`; `record.py` records the recordings
the first one replays. One list, so the two suites cannot test different things.
"""

from __future__ import annotations

import dataclasses
import hashlib
import tomllib
from dataclasses import dataclass
from pathlib import Path

from popquiz import bank, sandbox
from popquiz.bank import Explains, Option, Question

HERE = Path(__file__).resolve().parent
CASES_FILE = HERE / "cases.toml"
RECORDINGS = HERE / "recordings"
PIPELINE = HERE.parents[2]
BANK = PIPELINE.parent / "bank"


@dataclass(frozen=True)
class Case:
    name: str
    program: str
    expect: str
    verdict: str
    timeout_seconds: int | None = None

    def pin(self, pin: sandbox.Pin) -> sandbox.Pin:
        """The pin this case runs under: the real one, with a shorter deadline
        where the case asks for it. Never a different toolchain."""
        if self.timeout_seconds is None:
            return pin
        return dataclasses.replace(
            pin, limits=dataclasses.replace(pin.limits, timeout_seconds=self.timeout_seconds)
        )


def _raw() -> dict:
    return tomllib.loads(CASES_FILE.read_text(encoding="utf-8"))


def load_cases() -> tuple[Case, ...]:
    return tuple(Case(**c) for c in _raw()["case"])


def programs() -> dict[str, str]:
    """Program id -> where its source lives, as `cases.toml` says."""
    return dict(_raw()["programs"])


def source_of(program: str) -> str:
    where = programs()[program]
    if where.startswith("bank:"):
        return bank.load_question(BANK, where.removeprefix("bank:")).source
    return (HERE / where).read_text(encoding="utf-8")


def source_sha256(program: str) -> str:
    return hashlib.sha256(source_of(program).encode("utf-8")).hexdigest()


def question_for(program: str) -> Question:
    """The candidate `verify` is handed. A bank program is the bank question,
    legacy record and all, so an accepted verdict can be shown to replace that
    record whole. Any other program gets a placeholder question: `verify` reads
    only its id and source."""
    where = programs()[program]
    if where.startswith("bank:"):
        return bank.load_question(BANK, where.removeprefix("bank:"))
    return Question(
        id=program,
        source=source_of(program),
        topic="verifier fixture",
        difficulty_requested=1,
        options=tuple(Option(text=f"placeholder {n}", kind="output") for n in range(4))
        + (Option(text="does not compile", kind="does_not_compile"),),
        hint="placeholder",
        explains=Explains(what="placeholder", takeaway="placeholder"),
    )
