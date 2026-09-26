"""Every case in `tests/fixtures/verify/cases.toml`, on the T-15a image (D-18).

The stub suite (`tests/test_verify.py`) proves the verifier's decisions against
recordings; this proves the recordings still describe what the toolchain does. A
case whose real verdict or record differs from the replayed one fails here, which
is how a drifted recording surfaces instead of passing quietly.
"""

from __future__ import annotations

import dataclasses

import pytest

from fixtures.verify.verify_cases import RECORDINGS, load_cases, question_for, source_of
from popquiz import sandbox, verify
from popquiz.runner import RealRunner, StubRunner

CASES = load_cases()
FIXED_NOW = "2026-09-26T00:00:00Z"


@pytest.fixture(scope="module")
def pin() -> sandbox.Pin:
    return sandbox.read_pin()


def _without_host(v):
    """The record less the two fields that name the machine's architecture. The
    pin supports more than one triple, so a recording made on one is still a
    recording of the same toolchain on the other."""
    rustc = "\n".join(l for l in v.rustc.splitlines() if not l.startswith("host: "))
    return dataclasses.replace(v, rustc=rustc, target_triple=None)


@pytest.mark.parametrize("case", CASES, ids=[c.name for c in CASES])
def test_the_real_toolchain_decides_each_case_as_the_recording_does(case, pin) -> None:
    question = question_for(case.program)
    real = verify.verify(
        question,
        RealRunner({case.program: source_of(case.program)}, pin=case.pin(pin)),
        expect=case.expect,
        pin=pin,
        now=lambda: FIXED_NOW,
    )
    stub = verify.verify(
        question, StubRunner(RECORDINGS), expect=case.expect, pin=pin, now=lambda: FIXED_NOW
    )

    assert real.code == case.verdict, real.reason
    assert real.code == stub.code
    assert real.steps == stub.steps
    if real.accepted:
        assert real.verified is not None and stub.verified is not None
        assert real.verified.target_triple in pin.target_triples
        assert _without_host(real.verified) == _without_host(stub.verified)
    if case.verdict == verify.NONDETERMINISTIC:
        assert real.distinct_outputs is not None and real.distinct_outputs > 1


def test_the_real_toolchain_is_refused_under_a_pin_it_does_not_match(pin) -> None:
    runner = RealRunner({"q3": source_of("q3")}, pin=pin)
    with pytest.raises(verify.PinMismatch):
        verify.verify(
            question_for("q3"),
            runner,
            expect="ran",
            pin=dataclasses.replace(pin, commit_hash="0" * 40),
        )


def test_a_record_the_real_toolchain_writes_is_not_stale(pin) -> None:
    verdict = verify.verify(
        question_for("q3"), RealRunner({"q3": source_of("q3")}, pin=pin), expect="ran", pin=pin
    )
    assert verdict.verified is not None
    assert not verify.is_stale(verdict.verified, pin)
