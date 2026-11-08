"""The verifier's cases, read from `cases.toml`, for the three places that run them.

`tests/test_verify.py` replays each case on `StubRunner`; the real-toolchain twin in
`tests/sandbox/verify/` runs it on `RealRunner`; `record.py` records the recordings
the first one replays. One list, so the two suites cannot test different things.
"""

from __future__ import annotations

import dataclasses
import hashlib
import json
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


def legacy_question(qid: str) -> Question:
    """An actual migration result, independent of later bank re-verification."""
    from popquiz.migrate_mvp import build_question

    mvp = BANK.parent / "mvp" / "2026-08-12"
    batch = json.loads((mvp / "verified.json").read_text(encoding="utf-8"))
    content = json.loads((mvp / "content.json").read_text(encoding="utf-8"))
    question = next(q for q in batch["questions"] if q["id"] == qid)
    return build_question(batch, question, content[qid])


def question_for(program: str) -> Question:
    """The candidate `verify` is handed. A bank program is the bank question,
    with its current record. Legacy-specific cases use `legacy_question`, so
    refreshing the bank does not remove their legacy coverage. Any other program gets a placeholder question: `verify` reads
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


def verify_bank(bank_dir: Path) -> None:
    """Re-verify every question in a copied bank through the verifier, over these
    recordings, so each record and its `<bank>/recordings/` file are the verifier's
    and never typed (GAP-6). It does what `verify.main` does - `verify` through a
    `RecordingRunner`, `with_record`, `write_recording` - with one fixed, earlier
    `verified_at`, so a test that re-verifies later sees the time move. A test that
    schedules needs both files: scheduling refuses a record with no recording, or
    one that does not match it. A question with no fixture recording is left as it
    is, so it has no recording either."""
    from popquiz import verify
    from popquiz.runner import RecordingRunner, StubRunner

    pin = verify._load_pin(None)
    for path in sorted((bank_dir / "questions").glob("*.json")):
        if not (RECORDINGS / path.name).is_file():
            continue
        question = bank.load_question(bank_dir, path.stem)
        expect = verify.expect_from_record(question.verified)
        recorder = RecordingRunner(StubRunner(RECORDINGS))
        verdict = verify.verify(question, recorder, expect=expect, pin=pin, now=lambda: "2026-10-01T00:00:00Z")
        assert verdict.accepted, f"{path.name}: {verdict.reason}"
        verified = verify.with_record(question, verdict)
        bank.save_question(bank_dir, verified)
        verify.write_recording(bank_dir / verify.RECORDINGS, verified, recorder.steps, pin)
