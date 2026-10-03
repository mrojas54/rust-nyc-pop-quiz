"""The verifier's decisions, proven on recorded outputs (T-15b; AC-6 to AC-11, AC-13, AC-87, G-2).

Every case replays `tests/fixtures/verify/recordings/`, which `record.py` produced
by running the programs on the T-15a image. The same cases run on the real
toolchain in `tests/sandbox/verify/` (`just test-full`).

No assertion here quotes what a program printed. Where a test needs an output it
reads it from the recording, which a machine wrote (CLAUDE.md, G-2).
"""

from __future__ import annotations

import ast
import dataclasses
import json
import pathlib

import pytest

from fixtures.verify.verify_cases import (
    RECORDINGS,
    Case,
    load_cases,
    programs,
    question_for,
    source_sha256,
)
from popquiz import bank, receipt, sandbox, verify
from popquiz.bank import BORROW_MODELS, Option, Verified
from popquiz.runner import RunStep, StubRunner

HERE = pathlib.Path(__file__).parent
SRC = HERE.parent / "src" / "popquiz"
CASES = load_cases()
FIXED_NOW = "2026-09-26T00:00:00Z"


def _pin() -> sandbox.Pin:
    return sandbox.read_pin()


def _recording(program: str) -> dict:
    return json.loads((RECORDINGS / f"{program}.json").read_text(encoding="utf-8"))


class _Counting:
    """A StubRunner that remembers which kinds it was asked for."""

    def __init__(self, inner=None) -> None:
        self.inner = inner or StubRunner(RECORDINGS)
        self.asked: list[str] = []

    def run(self, kind: str, program_id: str) -> RunStep:
        self.asked.append(kind)
        return self.inner.run(kind, program_id)


def _verify(program: str, expect: str, *, pin=None, runner=None) -> verify.Verdict:
    return verify.verify(
        question_for(program),
        runner or StubRunner(RECORDINGS),
        expect=expect,  # type: ignore[arg-type]
        pin=pin or _pin(),
        now=lambda: FIXED_NOW,
    )


# --------------------------------------------------------------------------- #
# The recordings are what they say they are
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize("program", sorted(programs()))
def test_every_program_has_a_recording_of_its_current_source(program: str) -> None:
    recorded = _recording(program)
    assert recorded["_source_sha256"] == source_sha256(program), (
        f"{program}'s source changed since it was recorded: re-record with "
        "`uv run --no-sync python tests/fixtures/verify/record.py`"
    )


@pytest.mark.parametrize("program", sorted(programs()))
def test_every_recording_was_made_under_the_current_pin(program: str) -> None:
    made = _recording(program)["_recorded"]
    pin = _pin()
    assert (made["rustc_release"], made["rustc_commit_hash"], made["miri_version"]) == (
        pin.release, pin.commit_hash, pin.miri_version
    ), "the pin moved since these were recorded: re-record them"
    assert made["image"] == pin.image


# --------------------------------------------------------------------------- #
# Each decision, on the stub (the same list runs on the real toolchain)
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize("case", CASES, ids=[c.name for c in CASES])
def test_the_verifier_decides_each_case_as_recorded(case: Case) -> None:
    verdict = _verify(case.program, case.expect)
    assert verdict.code == case.verdict, verdict.reason
    assert (verdict.verified is not None) == verdict.accepted
    assert verdict.reason


def test_an_accepted_run_asks_toolchain_compile_n_runs_then_stacked_borrows() -> None:
    runner = _Counting()
    _verify("q3", "ran", runner=runner)
    assert runner.asked == [
        "toolchain", "compile", *(f"run:{i}" for i in range(1, verify.RUNS + 1)),
        "miri:stacked_borrows:0",
    ]


def test_tree_borrows_is_asked_only_for_declared_ub() -> None:
    ran, ub = _Counting(), _Counting()
    _verify("ub-both", "ran", runner=ran)
    _verify("ub-both", "ub", runner=ub)
    assert not any("tree_borrows" in k for k in ran.asked)
    assert ub.asked[-2:] == ["miri:stacked_borrows:0", "miri:tree_borrows:0"]


def test_a_does_not_compile_verdict_runs_nothing() -> None:
    runner = _Counting()
    _verify("q8", "does_not_compile", runner=runner)
    assert runner.asked == ["toolchain", "compile"]


def test_the_output_that_varied_is_counted_not_flagged() -> None:
    """AC-8: rejected, and the count of distinct outputs is reported."""
    recorded = _recording("hashmap-order")
    distinct = {
        (recorded[f"run:{i}"]["stdout"], recorded[f"run:{i}"]["exit_code"])
        for i in range(1, verify.RUNS + 1)
    }
    verdict = _verify("hashmap-order", "ran")
    assert not verdict.accepted
    assert len(distinct) > 1
    assert verdict.distinct_outputs == len(distinct)


def test_n_is_a_configured_hypothesis() -> None:
    assert verify.RUNS == 5


# --------------------------------------------------------------------------- #
# The record (SPEC 3.2)
# --------------------------------------------------------------------------- #

RAN_KEYS = {
    "rustc", "edition", "target_triple", "flags", "runs", "stdout", "exit_code",
    "miri", "verified_at", "verifier_version",
}
DNC_KEYS = {
    "rustc", "edition", "target_triple", "flags", "compile_error_code",
    "verified_at", "verifier_version",
}


def _accepted(program: str, expect: str) -> Verified:
    verdict = _verify(program, expect)
    assert verdict.accepted, verdict.reason
    assert verdict.verified is not None
    return verdict.verified


def test_a_ran_record_holds_exactly_what_the_machine_observed() -> None:
    """AC-6: `rustc -Vv`, edition, target triple and the flag set, every one read
    off a step or the pin - compared here against the recording, never typed."""
    recorded = _recording("q3")
    v = _accepted("q3", "ran")
    pin = _pin()
    toolchain = verify.parse_toolchain(recorded["toolchain"]["stdout"])

    assert v.rustc == toolchain.rustc
    assert v.rustc.splitlines()[0].startswith("rustc ") and "commit-hash: " in v.rustc
    assert v.target_triple == toolchain.host
    assert v.target_triple in pin.target_triples
    assert v.edition == pin.edition
    assert v.flags == bank.Flags(
        opt_level=str(pin.flags["opt-level"]),
        overflow_checks=pin.flags["overflow-checks"],
        debug_assertions=pin.flags["debug-assertions"],
    )
    assert v.runs == bank.Runs(count=verify.RUNS, byte_identical=True)
    assert (v.stdout, v.exit_code) == (recorded["run:1"]["stdout"], recorded["run:1"]["exit_code"])
    assert v.miri == bank.Miri(
        clean=True, output_matched=True, version=pin.miri_version,
        configs=("stacked_borrows",), seeds=verify.MIRI_SEEDS,
    )
    assert v.verified_at == FIXED_NOW
    assert v.verifier_version == verify.verifier_version()
    assert not v.legacy and v.compile_error_code is None


def test_the_record_writes_exactly_the_spec_fields() -> None:
    """`verified.json` gains no field code did not write, and loses none."""
    assert set(bank._dump(_accepted("q3", "ran"))) == RAN_KEYS
    assert set(bank._dump(_accepted("panics", "ran"))) == RAN_KEYS
    assert set(bank._dump(_accepted("ub-both", "ub"))) == RAN_KEYS
    assert set(bank._dump(_accepted("q8", "does_not_compile"))) == DNC_KEYS


def test_a_does_not_compile_record_records_every_code_the_compiler_gave() -> None:
    """AC-11: the codes come from the recorded diagnostic, and agree with what the
    August batch recorded for the same program."""
    recorded = _recording("q8")
    v = _accepted("q8", "does_not_compile")

    assert v.compile_error_code == verify.error_codes(recorded["compile"]["stderr"])
    assert v.compile_error_code
    assert v.compile_error_code == question_for("q8").verified.compile_error_code
    assert (v.runs, v.stdout, v.exit_code, v.miri) == (None, None, None, None)


def test_error_codes_keep_compiler_order_and_drop_repeats() -> None:
    # Diagnostic *shapes* only - no compiler's words.
    assert verify.error_codes("error[E0502]: a\nerror[E0499]: b\nerror[E0502]: c\n") == (
        "E0502", "E0499",
    )
    assert verify.error_codes("warning: x\nerror: y\n") == ()


def test_a_panic_is_recorded_with_its_exit_code_and_derives_the_panic_option() -> None:
    recorded = _recording("panics")
    v = _accepted("panics", "ran")
    assert v.exit_code == recorded["run:1"]["exit_code"] == verify.PANIC_EXIT
    q = dataclasses.replace(
        question_for("panics"),
        options=(Option("a", "output"), Option("b", "panic"), Option("c", "output"),
                 Option("d", "ub"), Option("does not compile", "does_not_compile")),
        verified=v,
    )
    assert bank.correct_index(q) == 1


def test_declared_ub_is_recorded_under_both_models_and_derives_the_ub_option() -> None:
    v = _accepted("ub-both", "ub")
    assert v.miri is not None
    assert (v.miri.clean, v.miri.output_matched, v.miri.configs) == (False, True, BORROW_MODELS)
    q = dataclasses.replace(
        question_for("ub-both"),
        options=(Option("a", "output"), Option("b", "panic"), Option("c", "output"),
                 Option("d", "ub"), Option("does not compile", "does_not_compile")),
        verified=v,
    )
    assert bank.correct_index(q) == 3


def test_an_accepted_record_replaces_a_legacy_one_whole() -> None:
    """D-16: never merged, so no legacy field survives and none is back-filled."""
    before = question_for("q3")
    assert before.verified is not None and before.verified.legacy
    verdict = _verify("q3", "ran")

    after = verify.with_record(before, verdict)

    assert after.verified is verdict.verified
    assert "legacy" not in bank.question_to_dict(after)["verified"]
    assert dataclasses.replace(after, verified=before.verified) == before


def test_a_rejected_verdict_has_no_record_to_write() -> None:
    with pytest.raises(ValueError, match="no record"):
        verify.with_record(question_for("q3"), _verify("q3", "ub"))


@pytest.mark.parametrize(
    ("program", "expect"), [("q3", "ran"), ("q8", "does_not_compile"), ("ub-both", "ub")]
)
def test_the_record_round_trips_the_bank_format(program: str, expect: str) -> None:
    q = verify.with_record(question_for(program), _verify(program, expect))
    assert bank.verified_from_dict(bank.question_to_dict(q)["verified"]) == q.verified


# --------------------------------------------------------------------------- #
# AC-13 / AC-87: the receipt renders from the record with no toolchain present
# --------------------------------------------------------------------------- #


def test_the_receipt_renders_the_ran_list_from_a_record_verify_wrote() -> None:
    lines = receipt.receipt_lines(_accepted("q3", "ran"))
    assert lines == [
        f"{receipt.DONE} Compiled",
        f"{receipt.DONE} Ran {verify.RUNS} times",
        f"{receipt.DONE} Output never varied",
        f"{receipt.DONE} Miri ran clean",
    ]
    detail = receipt.receipt_detail(_accepted("q3", "ran"))
    assert detail is not None and detail.target_triple and not detail.legacy


def test_the_receipt_renders_the_does_not_compile_list_from_a_record_verify_wrote() -> None:
    v = _accepted("q8", "does_not_compile")
    lines = receipt.receipt_lines(v)
    assert lines is not None and len(lines) == 3
    assert f"{receipt.DONE} Error {', '.join(v.compile_error_code or ())}" in lines


def test_the_receipt_says_miri_flagged_ub_for_declared_ub() -> None:
    lines = receipt.receipt_lines(_accepted("ub-both", "ub"))
    assert lines is not None and lines[-1] == f"{receipt.DONE} Miri flagged undefined behavior"


def test_a_legacy_record_still_renders() -> None:
    for qid in ("q3", "q4", "q7", "q8"):
        v = bank.load_question(HERE.parent.parent / "bank", qid).verified
        assert receipt.receipt_lines(v)
        assert receipt.receipt_detail(v) is not None


# --------------------------------------------------------------------------- #
# The pin (D-17, AC-6)
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize(
    "change",
    [
        {"release": "0.0.0"},
        {"commit_hash": "0" * 40},
        {"miri_version": "miri 0.0.0 (0000000000 2000-01-01)"},
        {"target_triples": ("riscv64gc-unknown-linux-gnu",)},
    ],
    ids=["release", "commit-hash", "miri", "host triple"],
)
def test_a_toolchain_that_is_not_the_pin_is_refused_before_any_candidate_step(change) -> None:
    runner = _Counting()
    with pytest.raises(verify.PinMismatch, match="does not match the pin"):
        _verify("q3", "ran", pin=dataclasses.replace(_pin(), **change), runner=runner)
    assert runner.asked == ["toolchain"]


@pytest.mark.parametrize("blank", ["release", "commit_hash", "miri_version", "target_triples"])
def test_a_pin_with_no_recorded_value_is_refused_before_anything_runs(blank: str) -> None:
    runner = _Counting()
    empty = () if blank == "target_triples" else ""
    with pytest.raises(verify.PinUnreadable):
        _verify("q3", "ran", pin=dataclasses.replace(_pin(), **{blank: empty}), runner=runner)
    assert runner.asked == []


def test_an_unreadable_pin_file_is_refused(monkeypatch) -> None:
    def broken(*_a, **_k):
        raise sandbox.PinError("no pin at /nowhere.")

    monkeypatch.setattr(sandbox, "read_pin", broken)
    runner = _Counting()
    with pytest.raises(verify.PinUnreadable, match="no pin"):
        verify.verify(question_for("q3"), runner, expect="ran")
    assert runner.asked == []


@pytest.mark.parametrize("dropped", ["release: ", "commit-hash: ", "host: ", "miri "])
def test_a_toolchain_report_missing_a_compared_line_is_refused(dropped: str) -> None:
    """The report is the recorded one with a line taken out, not a typed one."""
    step = StubRunner(RECORDINGS).run("toolchain", "q3")
    cut = "\n".join(l for l in step.stdout.splitlines() if not l.startswith(dropped))

    class Cut:
        def run(self, kind, program_id):
            return dataclasses.replace(step, stdout=cut) if kind == "toolchain" else None

    with pytest.raises(verify.ToolchainUnreadable, match="cannot be compared"):
        verify.verify(question_for("q3"), Cut(), expect="ran", pin=_pin())


def test_is_stale_matches_the_pin_it_was_verified_under() -> None:
    v = _accepted("q3", "ran")
    pin = _pin()
    assert not verify.is_stale(v, pin)
    assert verify.is_stale(v, dataclasses.replace(pin, release="0.0.0"))
    assert verify.is_stale(v, dataclasses.replace(pin, commit_hash="0" * 40))
    assert verify.is_stale(v, dataclasses.replace(pin, edition="2015"))
    assert verify.is_stale(v, dataclasses.replace(pin, miri_version="miri 0.0.0"))
    assert verify.is_stale(
        v, dataclasses.replace(pin, flags={**pin.flags, "overflow-checks": False})
    )


def test_a_does_not_compile_record_is_not_stale_on_miri_alone() -> None:
    v = _accepted("q8", "does_not_compile")
    assert not verify.is_stale(v, dataclasses.replace(_pin(), miri_version="miri 0.0.0"))


def test_a_legacy_record_is_exempt_from_the_stale_check() -> None:
    for qid in ("q3",):
        v = question_for(qid).verified
        assert v is not None and v.legacy
        assert not verify.is_stale(v, dataclasses.replace(_pin(), release="0.0.0"))


def test_a_non_legacy_record_whose_rustc_cannot_be_compared_is_stale() -> None:
    v = dataclasses.replace(_accepted("q3", "ran"), rustc=_accepted("q3", "ran").rustc.splitlines()[0])
    assert verify.is_stale(v, _pin())


def test_the_declared_answer_is_recovered_from_a_record() -> None:
    assert verify.expect_from_record(_accepted("q3", "ran")) == "ran"
    assert verify.expect_from_record(_accepted("q8", "does_not_compile")) == "does_not_compile"
    assert verify.expect_from_record(_accepted("ub-both", "ub")) == "ub"


# --------------------------------------------------------------------------- #
# AC-7 / G-2: the build's provenance check
# --------------------------------------------------------------------------- #

KINDED = (
    Option("a", "output"), Option("b", "panic"), Option("c", "output"),
    Option("d", "ub"), Option("does not compile", "does_not_compile"),
)


def _file(program: str, expect: str, **question_changes) -> dict:
    """A candidate file as `verify` leaves it."""
    q = verify.with_record(question_for(program), _verify(program, expect))
    return bank.question_to_dict(dataclasses.replace(q, **question_changes))


def test_a_candidate_verify_wrote_passes_the_provenance_check() -> None:
    verify.check_provenance(_file("q3", "ran"))
    verify.check_provenance(_file("q8", "does_not_compile"))
    verify.check_provenance(_file("panics", "ran", options=KINDED))
    verify.check_provenance(_file("ub-both", "ub", options=KINDED))


def test_every_bank_question_passes_the_provenance_check() -> None:
    bank_dir = HERE.parent.parent / "bank" / "questions"
    for path in sorted(bank_dir.glob("*.json")):
        verify.check_provenance(json.loads(path.read_text(encoding="utf-8")))


def test_a_hand_edited_answer_is_refused() -> None:
    """AC-7: the correct option's text edited away from what the machine printed."""
    data = _file("q3", "ran")
    index = bank.correct_index(bank.question_from_dict(data))
    assert index is not None
    data["options"][index]["text"] += " (edited)"
    with pytest.raises(verify.ProvenanceError, match="drifted apart"):
        verify.check_provenance(data)


def test_a_stored_correct_field_is_refused() -> None:
    data = _file("q3", "ran")
    data["correct"] = 0
    with pytest.raises(verify.ProvenanceError, match="never stored"):
        verify.check_provenance(data)


def test_a_hand_edited_option_kind_is_refused() -> None:
    data = _file("panics", "ran", options=KINDED)
    data["options"][1]["kind"] = "output"
    with pytest.raises(verify.ProvenanceError):
        verify.check_provenance(data)


@pytest.mark.parametrize("stripped", ["verified_at", "verifier_version"])
def test_a_record_the_verifier_did_not_sign_off_is_refused(stripped: str) -> None:
    data = _file("q3", "ran")
    del data["verified"][stripped]
    with pytest.raises(verify.ProvenanceError, match="not written by the verifier"):
        verify.check_provenance(data)


def test_ub_under_one_borrow_model_is_refused() -> None:
    data = _file("ub-both", "ub", options=KINDED)
    data["verified"]["miri"]["configs"] = ["stacked_borrows"]
    with pytest.raises(verify.ProvenanceError, match="one borrow model"):
        verify.check_provenance(data)


def test_a_miri_config_that_is_not_a_borrow_model_is_refused() -> None:
    data = _file("q3", "ran")
    data["verified"]["miri"]["configs"] = ["weak_borrows"]
    with pytest.raises(verify.ProvenanceError, match="not borrow models"):
        verify.check_provenance(data)


def test_a_second_ub_option_is_refused() -> None:
    data = _file("ub-both", "ub", options=KINDED)
    data["options"][0]["kind"] = "ub"
    with pytest.raises(verify.ProvenanceError, match="options match"):
        verify.check_provenance(data)


def test_an_exit_code_with_no_runs_is_refused() -> None:
    data = _file("q3", "ran")
    del data["verified"]["runs"]
    with pytest.raises(verify.ProvenanceError, match="neither runs nor a compile error"):
        verify.check_provenance(data)


def test_a_does_not_compile_record_that_ran_is_refused() -> None:
    data = _file("q8", "does_not_compile")
    data["verified"]["runs"] = {"count": 5, "byte_identical": True}
    with pytest.raises(verify.ProvenanceError, match="nothing ran"):
        verify.check_provenance(data)


def test_a_candidate_with_no_record_is_refused() -> None:
    data = bank.question_to_dict(question_for("panics"))
    with pytest.raises(verify.ProvenanceError, match="no verified record"):
        verify.check_provenance(data)


# --------------------------------------------------------------------------- #
# `verify <program>`
# --------------------------------------------------------------------------- #


def _candidate(tmp_path: pathlib.Path, program: str) -> pathlib.Path:
    path = tmp_path / f"{program}.json"
    path.write_text(json.dumps(bank.question_to_dict(question_for(program))), encoding="utf-8")
    return path


def test_the_cli_writes_the_record_into_the_candidate(tmp_path, capsys) -> None:
    path = _candidate(tmp_path, "q3")
    assert verify.main([str(path), "--expect", "ran"], runner=StubRunner(RECORDINGS)) == 0
    written = json.loads(path.read_text(encoding="utf-8"))
    assert "legacy" not in written["verified"]
    assert written["verified"]["verifier_version"] == verify.verifier_version()
    verify.check_provenance(written)
    assert "ACCEPTED" in capsys.readouterr().out


def test_the_cli_recovers_the_declared_answer_from_an_existing_record(tmp_path) -> None:
    path = _candidate(tmp_path, "q8")
    assert verify.main([str(path)], runner=StubRunner(RECORDINGS)) == 0


def test_the_cli_leaves_a_rejected_candidate_untouched(tmp_path, capsys) -> None:
    path = _candidate(tmp_path, "hashmap-order")
    before = path.read_bytes()
    assert verify.main([str(path), "--expect", "ran"], runner=StubRunner(RECORDINGS)) == 1
    assert path.read_bytes() == before
    assert verify.NONDETERMINISTIC in capsys.readouterr().out


def test_the_cli_needs_a_declared_answer_when_there_is_no_record(tmp_path) -> None:
    assert verify.main([str(_candidate(tmp_path, "panics"))], runner=StubRunner(RECORDINGS)) == 2


# --------------------------------------------------------------------------- #
# Structure: who writes a record, and what is never typed
# --------------------------------------------------------------------------- #


def test_only_the_verifier_the_migration_and_the_reader_construct_a_record() -> None:
    """G-2: a `Verified` is built by `verify` (new records), `migrate_mvp` (the
    August legacy records, D-16) and `bank`'s JSON reader - nowhere else."""
    builders = set()
    for path in sorted(SRC.glob("*.py")):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        if any(
            isinstance(n, ast.Call) and isinstance(n.func, ast.Name) and n.func.id == "Verified"
            for n in ast.walk(tree)
        ):
            builders.add(path.name)
    assert builders == {"verify.py", "migrate_mvp.py", "bank.py"}


def test_no_pin_value_and_no_recorded_output_is_typed_into_the_code() -> None:
    """D-17: the pin is read from pin.toml, never re-declared. G-2: an output a
    program printed appears only in a recording.

    An output that is only words is not scanned: a word a program prints is also
    a word this code uses. Structured output - a vector, a debug print - is what a
    person would transcribe, and that is scanned."""
    pin = _pin()
    typed = [pin.release, pin.commit_hash, pin.miri_version, pin.nightly]

    def structured(text: str) -> bool:
        return bool(text) and not text.replace(" ", "").isalpha()

    outputs = {
        step["stdout"].strip()
        for program in programs()
        for kind, step in _recording(program).items()
        if not kind.startswith("_") and kind != "toolchain" and structured(step["stdout"].strip())
    }
    assert outputs, "the scan must have something to look for"
    scanned = [
        SRC / "verify.py",
        SRC / "runner.py",
        HERE / "test_verify.py",
        HERE / "fixtures" / "verify" / "verify_cases.py",
        HERE / "fixtures" / "verify" / "record.py",
        HERE / "fixtures" / "verify" / "cases.toml",
    ]
    for path in scanned:
        text = path.read_text(encoding="utf-8")
        for value in typed:
            assert value not in text, f"{path.name} re-declares the pin value {value!r}"
        for output in outputs:
            assert output not in text, f"{path.name} holds a recorded output: {output!r}"
