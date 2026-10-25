"""The organizer's review decisions (AC-19 to AC-22, AC-72, AC-73, AC-88, AC-95, G-12, GAP-24).

Every test runs on a temporary copy of `tests/fixtures/review/`, never on
`bank/questions/`; the module fixture below checks that every file under the real
`bank/` is byte-identical afterwards. Re-verification runs on `StubRunner`'s
recorded outputs, injected through `review.main(runner=...)`; the CLI itself has
no way to ask for them, so outside a test it always reaches the pinned toolchain.

Nothing here starts a process (`test_runner.py` holds `just test` to that); the
TUI's own end-to-end test runs the CLI as a subprocess.
"""

from __future__ import annotations

import ast
import hashlib
import json
import runpy
import shutil
import sys
from pathlib import Path

import pytest

from popquiz import audit, bank, review, schedule
from popquiz.runner import StubRunner

HERE = Path(__file__).parent
FIXTURES = HERE / "fixtures" / "review"
RECORDINGS = HERE / "fixtures" / "verify" / "recordings"
REPO = HERE.parent.parent
NAME = "fixture-organizer"


@pytest.fixture(scope="module", autouse=True)
def the_real_bank_is_untouched():
    def snapshot() -> dict[str, str]:
        return {
            str(p.relative_to(REPO)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted((REPO / "bank").rglob("*"))
            if p.is_file()
        }

    before = snapshot()
    assert before, "the real bank was not found"
    yield
    assert snapshot() == before


@pytest.fixture
def bank_dir(tmp_path: Path) -> Path:
    shutil.copytree(FIXTURES / "questions", tmp_path / "bank" / "questions")
    return tmp_path / "bank"


@pytest.fixture(autouse=True)
def no_ambient_identity(monkeypatch):
    monkeypatch.delenv(review.ORGANIZER_ENV, raising=False)


def run(*args, runner=None) -> int:
    return review.main([str(a) for a in args], runner=runner)


def raw(bank_dir: Path, qid: str) -> bytes:
    return bank.question_path(bank_dir, qid).read_bytes()


def record(bank_dir: Path, qid: str) -> dict:
    return json.loads(raw(bank_dir, qid))


def load(bank_dir: Path, qid: str) -> bank.Question:
    return bank.load_question(bank_dir, qid)


def refused(code: int, capsys, needle: str) -> str:
    err = capsys.readouterr().err
    assert code == 1, err
    assert needle in err
    return err


def accepted_and_affirmed(bank_dir: Path, qid: str = "q4") -> None:
    assert run("accept", qid, "--bank", bank_dir) == 0
    assert run("affirm", qid, "--by", NAME, "--bank", bank_dir) == 0


def stub() -> StubRunner:
    return StubRunner(RECORDINGS)


# --------------------------------------------------------------------------- #
# AC-19: accept, reject with a reason, difficulty, each a round trip
# --------------------------------------------------------------------------- #


def test_ac19_accept_round_trips(bank_dir, capsys):
    assert run("accept", "q4", "--bank", bank_dir) == 0
    assert load(bank_dir, "q4").review.status == "accepted"
    assert "q4 accepted." in capsys.readouterr().out


def test_ac19_reject_needs_a_reason_and_the_generator_can_read_it_back(bank_dir, capsys):
    before = raw(bank_dir, "q4")
    refused(run("reject", "q4", "--reason", "   ", "--bank", bank_dir), capsys, "needs a reason")
    assert raw(bank_dir, "q4") == before
    assert run("reject", "q4", "--reason", "Too close to q3.", "--bank", bank_dir) == 0
    assert load(bank_dir, "q4").review.status == "rejected"
    assert review.rejection_reasons(bank_dir) == [("q4", "Too close to q3.")]
    capsys.readouterr()
    assert run("reasons", "--bank", bank_dir) == 0
    assert json.loads(capsys.readouterr().out) == {"id": "q4", "reason": "Too close to q3."}


def test_ac19_a_rejection_clears_an_affirmation(bank_dir):
    accepted_and_affirmed(bank_dir)
    assert run("reject", "q4", "--reason", "Changed my mind.", "--bank", bank_dir) == 0
    q = load(bank_dir, "q4")
    assert bank.affirmation_state(q) == "none" and q.review.affirmed_digest is None


def test_ac88_difficulty_judged_is_recorded_within_the_generators_range(bank_dir, capsys):
    assert run("difficulty", "q4", "4", "--bank", bank_dir) == 0
    assert load(bank_dir, "q4").review.difficulty_judged == 4
    before = raw(bank_dir, "q4")
    refused(run("difficulty", "q4", "6", "--bank", bank_dir), capsys, "Difficulty is 1 to 5")
    assert raw(bank_dir, "q4") == before


def test_the_bank_has_no_default(bank_dir):
    with pytest.raises(SystemExit):
        review.main(["accept", "q4"])


# --------------------------------------------------------------------------- #
# AC-73: quoted output is checked against the verified output
# --------------------------------------------------------------------------- #


def test_ac73_a_wrong_quote_blocks_acceptance_until_the_organizer_checks_it(bank_dir, capsys):
    before = raw(bank_dir, "wrongquote")
    err = refused(run("accept", "wrongquote", "--bank", bank_dir), capsys, "cannot be accepted")
    assert '"-3 -1 | -4"' in err and "AC-73" in err
    assert raw(bank_dir, "wrongquote") == before
    refused(
        run("accept", "wrongquote", "--checked-quote", "never quoted", "--bank", bank_dir),
        capsys,
        "is not quoted text",
    )
    assert run("accept", "wrongquote", "--checked-quote", "-3 -1 | -4", "--bank", bank_dir) == 0
    assert load(bank_dir, "wrongquote").review.quotes_checked == ("-3 -1 | -4",)


def test_ac73_a_quote_that_is_a_printed_line_needs_no_check(bank_dir):
    path = bank.question_path(bank_dir, "q4")
    data = json.loads(path.read_text())
    data["explains"]["what"] += ' It prints "-3 -1 | -4 1".'
    path.write_text(json.dumps(data))
    assert review.accept_blockers(load(bank_dir, "q4")) == []


# --------------------------------------------------------------------------- #
# AC-72, AC-95, G-12: affirm is blocking and attributable
# --------------------------------------------------------------------------- #


def test_ac72_affirm_refuses_without_an_organizer_identity(bank_dir, capsys):
    assert run("accept", "q4", "--bank", bank_dir) == 0
    capsys.readouterr()
    before = raw(bank_dir, "q4")
    err = refused(run("affirm", "q4", "--bank", bank_dir), capsys, "Affirm needs your name")
    assert err.strip() == "popquiz.review: Affirm needs your name: set POPQUIZ_ORGANIZER or pass --by NAME."
    refused(run("affirm", "q4", "--by", "  ", "--bank", bank_dir), capsys, "Affirm needs your name")
    assert raw(bank_dir, "q4") == before


def test_ac72_affirm_takes_the_name_from_the_environment(bank_dir, monkeypatch):
    assert run("accept", "q4", "--bank", bank_dir) == 0
    monkeypatch.setenv(review.ORGANIZER_ENV, NAME)
    assert run("affirm", "q4", "--bank", bank_dir) == 0
    assert load(bank_dir, "q4").review.affirmed_by == NAME


def test_ac72_affirm_records_who_when_and_what_and_the_question_schedules(bank_dir, capsys):
    accepted_and_affirmed(bank_dir)
    q = load(bank_dir, "q4")
    assert q.review.affirmed_by == NAME
    assert q.review.affirmed_at.endswith("+00:00")
    assert q.review.affirmed_digest == bank.affirmation_digest(q)
    assert bank.affirmation_state(q) == "bound"
    assert schedule.refusal(q) is None and audit.in_reserve(q)


@pytest.mark.parametrize("status", [None, "rejected"])
def test_ac72_affirm_refuses_an_unaccepted_question(bank_dir, capsys, status):
    if status:
        assert run("reject", "q4", "--reason", "no", "--bank", bank_dir) == 0
    capsys.readouterr()
    before = raw(bank_dir, "q4")
    refused(run("affirm", "q4", "--by", NAME, "--bank", bank_dir), capsys, "has not been accepted")
    assert raw(bank_dir, "q4") == before


def test_ac72_a_trace_of_fewer_than_two_steps_cannot_be_affirmed(bank_dir, capsys):
    refused(run("affirm", "q7", "--by", NAME, "--bank", bank_dir), capsys, "at least two steps; it has 0")


def test_ac95_every_incorrect_option_needs_its_why_tempting(bank_dir, capsys):
    assert run("accept", "q4", "--bank", bank_dir) == 0
    path = bank.question_path(bank_dir, "q4")
    data = json.loads(path.read_text())
    wrong = next(i for i, o in enumerate(data["options"]) if o.get("why_tempting"))
    data["options"][wrong]["why_tempting"] = " "
    path.write_text(json.dumps(data))
    capsys.readouterr()
    err = refused(run("affirm", "q4", "--by", NAME, "--bank", bank_dir), capsys, "AC-95")
    assert f"Choice {'ABCDE'[wrong]}" in err


def with_trace(bank_dir: Path, qid: str, *last_values: dict, early: tuple[dict, ...] = ()) -> None:
    """Give `qid` a two-step trace: `early` on step 1, `last_values` on the last.
    Its quoted spans are marked checked, as the organizer checked q8's (AC-73)."""
    path = bank.question_path(bank_dir, qid)
    data = json.loads(path.read_text())
    data["review"]["quotes_checked"] = [q["span"] for q in review.quotes(load(bank_dir, qid))]
    data["trace"]["steps"] = [
        {"lines": [2], "focus": [2, 3], "note": "first", "values": list(early)},
        {"lines": [1, 6], "focus": [1, 6], "note": "last", "values": list(last_values)},
    ]
    path.write_text(json.dumps(data))


def error_row(now: str = "E0502") -> dict:
    return {"name": "error", "was": "—", "now": now}


def test_f15_a_question_that_does_not_compile_affirms_when_its_last_step_names_the_error(bank_dir, capsys):
    """RULING F-15 (client, 2026-10-25): for a does-not-compile question the resolving
    step is the one whose values name `error`, with the compiler's code from the record."""
    with_trace(bank_dir, "q8", error_row())
    q8 = load(bank_dir, "q8")
    report = review.report(q8)
    assert report["blocked_on"] is None
    assert report["affirm_blockers"] == []
    assert run("affirm", "q8", "--by", NAME, "--bank", bank_dir) == 0
    q8 = load(bank_dir, "q8")
    assert bank.affirmation_state(q8) == "bound" and q8.review.affirmed_by == NAME
    assert schedule.refusal(q8) is None and audit.in_reserve(q8)


def test_f15_a_last_step_that_names_no_error_cannot_be_affirmed(bank_dir, capsys):
    with_trace(bank_dir, "q8", {"name": "v", "was": "—", "now": "[1, 2, 3]"})
    before = raw(bank_dir, "q8")
    refused(run("affirm", "q8", "--by", NAME, "--bank", bank_dir), capsys, "last step names no error")
    assert raw(bank_dir, "q8") == before


def test_f15_the_error_row_must_say_what_the_compiler_said(bank_dir, capsys):
    with_trace(bank_dir, "q8", error_row("E0499"))
    err = refused(run("affirm", "q8", "--by", NAME, "--bank", bank_dir), capsys, "E0499")
    assert "E0502" in err


def test_f15_an_error_row_before_the_last_step_cannot_be_affirmed(bank_dir, capsys):
    """`work` shows every step but the last; an earlier `error` row would give the
    answer away before reveal (G-3)."""
    with_trace(bank_dir, "q8", error_row(), early=(error_row(),))
    refused(run("affirm", "q8", "--by", NAME, "--bank", bank_dir), capsys, "step 1 names error")


def test_f15_an_empty_trace_reports_the_two_step_minimum_and_no_error_row(bank_dir):
    blockers = review.affirm_blockers(load(bank_dir, "q8"))
    assert "The trace needs at least two steps; it has 0." in blockers
    assert not any("F-15" in b for b in blockers)


def test_ac73_a_wrong_quote_also_blocks_affirmation(bank_dir, capsys):
    path = bank.question_path(bank_dir, "wrongquote")
    data = json.loads(path.read_text())
    data["review"]["status"] = "accepted"
    path.write_text(json.dumps(data))
    refused(run("affirm", "wrongquote", "--by", NAME, "--bank", bank_dir), capsys, "AC-73")


def test_q3s_hand_affirmation_is_unbound_and_can_be_affirmed_again(bank_dir, capsys):
    """RULING PQ-23 (client, 2026-10-12): q3's hand affirmation (F-52) has no digest,
    so it reads as unbound and does not schedule. The organizer can affirm it again,
    which binds it to its text. This runs on the fixture copy, never the bank's q3."""
    q3 = load(bank_dir, "q3")
    assert review.report(q3)["affirmation"] == "unbound"
    assert schedule.refusal(q3) is not None and not audit.in_reserve(q3)
    assert run("affirm", "q3", "--by", NAME, "--bank", bank_dir) == 0
    q3 = load(bank_dir, "q3")
    assert bank.affirmation_state(q3) == "bound" and q3.review.affirmed_by == NAME
    assert schedule.refusal(q3) is None and audit.in_reserve(q3)


def test_a_bound_affirmation_is_kept_and_not_overwritten(bank_dir, capsys):
    accepted_and_affirmed(bank_dir)
    before = raw(bank_dir, "q4")
    refused(run("affirm", "q4", "--by", "someone-else", "--bank", bank_dir), capsys, "already affirmed")
    assert raw(bank_dir, "q4") == before


# --------------------------------------------------------------------------- #
# Edit and re-verify (SPEC 7.4, GAP-24, AC-7)
# --------------------------------------------------------------------------- #


def edit_file(bank_dir: Path, tmp_path: Path, qid: str, change) -> Path:
    path = tmp_path / f"{qid}-edit.json"
    assert run("export", qid, "--out", path, "--bank", bank_dir) == 0
    data = json.loads(path.read_text())
    change(data)
    path.write_text(json.dumps(data))
    return path


def test_gap24_a_prose_edit_clears_the_affirmation_and_marks_it_edited(bank_dir, tmp_path, capsys):
    accepted_and_affirmed(bank_dir)
    verified_before = record(bank_dir, "q4")["verified"]
    path = edit_file(bank_dir, tmp_path, "q4", lambda d: d["explains"].update(what=d["explains"]["what"] + " More."))
    assert run("edit", "q4", "--from", path, "--bank", bank_dir) == 0
    q = load(bank_dir, "q4")
    assert q.review.status == "edited"
    assert (q.review.affirmed_by, q.review.affirmed_at, q.review.affirmed_digest) == (None, None, None)
    assert q.explains.what.endswith(" More.")
    assert record(bank_dir, "q4")["verified"] == verified_before
    assert "needs affirming again" in capsys.readouterr().out
    assert schedule.refusal(q) is not None and not audit.in_reserve(q)
    # ...and it can be affirmed again, bound to the new text.
    assert run("affirm", "q4", "--by", NAME, "--bank", bank_dir) == 0
    assert bank.affirmation_state(load(bank_dir, "q4")) == "bound"


def test_an_edit_does_not_accept_a_rejected_question(bank_dir, tmp_path, capsys):
    """An edit marks an accepted question `edited`, which still counts as accepted.
    A rejected question stays rejected, reason and all: editing it is not accepting
    it, and only `accept` brings it back."""
    assert run("reject", "q4", "--reason", "Too close to q3", "--bank", bank_dir) == 0
    path = edit_file(bank_dir, tmp_path, "q4", lambda d: d["explains"].update(what=d["explains"]["what"] + " More."))
    assert run("edit", "q4", "--from", path, "--bank", bank_dir) == 0
    q = load(bank_dir, "q4")
    assert q.explains.what.endswith(" More.")
    assert (q.review.status, q.review.reason) == ("rejected", "Too close to q3")
    assert "stays rejected" in capsys.readouterr().out
    assert not audit.is_accepted(q) and not audit.in_reserve(q)
    refused(run("affirm", "q4", "--by", NAME, "--bank", bank_dir), capsys, "accept")


def test_an_edit_does_not_accept_an_unreviewed_question(bank_dir, tmp_path, capsys):
    path = edit_file(bank_dir, tmp_path, "q4", lambda d: d["explains"].update(what=d["explains"]["what"] + " More."))
    assert run("edit", "q4", "--from", path, "--bank", bank_dir) == 0
    q = load(bank_dir, "q4")
    assert q.review.status is None and not audit.is_accepted(q)
    assert "not accepted yet" in capsys.readouterr().out


def test_an_edit_that_changes_nothing_writes_nothing(bank_dir, tmp_path, capsys):
    accepted_and_affirmed(bank_dir)
    before = raw(bank_dir, "q4")
    path = edit_file(bank_dir, tmp_path, "q4", lambda d: None)
    assert run("edit", "q4", "--from", path, "--bank", bank_dir) == 0
    assert raw(bank_dir, "q4") == before
    assert "nothing written" in capsys.readouterr().out


def test_an_option_edit_re_verifies_through_the_verifier(bank_dir, tmp_path, capsys):
    assert run("accept", "q4", "--bank", bank_dir) == 0
    capsys.readouterr()
    q = load(bank_dir, "q4")
    correct = bank.correct_index(q)
    other = next(i for i in range(5) if i != correct and q.options[i].kind == "output")

    def change(d):
        d["options"][other]["text"] = "-9 -9 | -9 9"

    path = edit_file(bank_dir, tmp_path, "q4", change)
    assert run("edit", "q4", "--from", path, "--bank", bank_dir, runner=stub()) == 0
    out = capsys.readouterr().out
    assert "re-verified" in out and "ACCEPTED" in out
    edited = load(bank_dir, "q4")
    assert edited.options[other].text == "-9 -9 | -9 9"
    assert edited.review.status == "edited"
    # Written by the verifier, from the recorded run - not carried over from before.
    assert edited.verified is not None and edited.verified.verified_at != q.verified.verified_at
    assert bank.correct_index(edited) == correct


def test_a_rejected_re_verify_leaves_the_bank_file_byte_identical(bank_dir, tmp_path, capsys):
    accepted_and_affirmed(bank_dir)
    before = raw(bank_dir, "q4")
    path = edit_file(bank_dir, tmp_path, "q4", lambda d: d.update(source=d["source"] + "// edited\n"))
    code = run("edit", "q4", "--from", path, "--expect", "does_not_compile", "--bank", bank_dir, runner=stub())
    err = refused(code, capsys, "did not accept the edited q4, so nothing was written")
    assert str(path) in err
    assert raw(bank_dir, "q4") == before


def test_the_trace_structure_and_its_stdout_row_cannot_be_edited(bank_dir, tmp_path, capsys):
    before = raw(bank_dir, "q4")
    for change, needle in [
        (lambda d: d.update(trace={"steps": []}), "cannot change: trace"),
        (lambda d: d.update(verified={}), "cannot change: verified"),
        (lambda d: d["trace_notes"].pop(), "one per step"),
        (lambda d: d["explains"].update(legacy="new"), "never rewritten"),
    ]:
        path = edit_file(bank_dir, tmp_path, "q4", change)
        refused(run("edit", "q4", "--from", path, "--bank", bank_dir), capsys, needle)
    assert raw(bank_dir, "q4") == before


def test_a_trace_note_edit_keeps_the_stdout_row_the_verifier_wrote(bank_dir, tmp_path):
    rows = record(bank_dir, "q4")["trace"]["steps"][-1]["values"]
    path = edit_file(bank_dir, tmp_path, "q4", lambda d: d["trace_notes"].__setitem__(0, "A new first note."))
    assert run("edit", "q4", "--from", path, "--bank", bank_dir) == 0
    after = record(bank_dir, "q4")["trace"]["steps"]
    assert after[0]["note"] == "A new first note."
    assert after[-1]["values"] == rows


def test_gap24_re_verifying_identical_output_keeps_the_affirmation_bound(bank_dir, capsys):
    accepted_and_affirmed(bank_dir)
    assert run("reverify", "q4", "--bank", bank_dir, runner=stub()) == 0
    q = load(bank_dir, "q4")
    assert bank.affirmation_state(q) == "bound"


# --------------------------------------------------------------------------- #
# Warnings, the report, and what review never writes
# --------------------------------------------------------------------------- #


def test_d23_trope_warnings_are_shown_and_block_nothing(bank_dir):
    report = review.report(load(bank_dir, "tropes"))
    assert {w["field"] for w in report["warnings"]} == {"explains.takeaway"}
    assert {w["group"] for w in report["warnings"]} == {"reassurance", "filler"}
    assert report["accept_blockers"] == []
    assert run("accept", "tropes", "--bank", bank_dir) == 0
    assert run("affirm", "tropes", "--by", NAME, "--bank", bank_dir) == 0


def test_ac20_the_report_carries_one_screen_of_review(bank_dir, capsys):
    assert run("show", "q4", "--bank", bank_dir) == 0
    report = json.loads(capsys.readouterr().out)
    for key in ("source", "options", "explains", "hint", "receipt", "near_duplicate_of", "difficulty_requested"):
        assert key in report
    assert sum(o["correct"] for o in report["options"]) == 1
    assert report["receipt"] and report["affirmation"] == "none"


def test_ac7_no_decision_writes_the_verified_record(bank_dir, tmp_path):
    verified = record(bank_dir, "q4")["verified"]
    rows = record(bank_dir, "q4")["trace"]["steps"][-1]["values"]
    assert run("difficulty", "q4", "3", "--bank", bank_dir) == 0
    accepted_and_affirmed(bank_dir)
    path = edit_file(bank_dir, tmp_path, "q4", lambda d: d.update(hint=d["hint"] + " Look again."))
    assert run("edit", "q4", "--from", path, "--bank", bank_dir) == 0
    assert run("reject", "q4", "--reason", "no", "--bank", bank_dir) == 0
    assert record(bank_dir, "q4")["verified"] == verified
    assert record(bank_dir, "q4")["trace"]["steps"][-1]["values"] == rows


def test_ac7_review_names_verified_only_to_drop_it():
    """The structural half: review.py never assigns `verified` anything but None."""
    tree = ast.parse((REPO / "pipeline" / "src" / "popquiz" / "review.py").read_text())
    for node in ast.walk(tree):
        if isinstance(node, ast.keyword) and node.arg == "verified":
            assert isinstance(node.value, ast.Constant) and node.value.value is None


def test_the_cli_runs_as_the_tui_runs_it(bank_dir, monkeypatch, capsys):
    """The organizer TUI starts `python -m popquiz.review` with an argument list;
    this runs the same `__main__` in-process (the TUI's test spawns it for real)."""
    monkeypatch.setattr(sys, "argv", ["popquiz.review", "affirm", "q4", "--bank", str(bank_dir)])
    monkeypatch.delitem(sys.modules, "popquiz.review")  # run it fresh, as `-m` does
    with pytest.raises(SystemExit) as done:
        runpy.run_module("popquiz.review", run_name="__main__", alter_sys=True)
    assert done.value.code == 1
    assert "Affirm needs your name" in capsys.readouterr().err
