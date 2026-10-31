"""The organizer's review decisions: accept, reject, edit, difficulty, affirm (AC-19 to AC-22, SPEC.md 7.4).

`python -m popquiz.review <command> <id> --bank DIR`. The organizer TUI
(`prototypes/organizer-tui`) runs this as a subprocess, the way it runs
`popquiz.schedule`, and shows what it prints. **This is the one writer of a
review decision**: every change goes through `bank.save_question`, which
validates before anything reaches disk, and nothing in Rust writes a record.

Three things this module never does, each because a rule depends on it:

* **It never writes `verified`, or the trace's `stdout` row.** An edit that changes
  the program or an option drops the old record - it describes a program that no
  longer exists - and hands the edited question to `verify.main`, the one writer of
  both (AC-7, G-2). The bank file is replaced only when the verifier accepts; if it
  rejects or refuses, the bank file is untouched.
* **It never affirms without a name.** The organizer's identity comes from
  `--by NAME` or `POPQUIZ_ORGANIZER`; there is no default (AC-72, G-12).
* **It never lets an affirmation outlive its text.** Affirm records a digest of
  what was affirmed (`bank.affirmation_digest`); any edit clears the affirmation and
  marks the record `edited` (GAP-24).

The trope and Forbidden warnings come from `copylint.check_prose` and block
nothing (D-23). Answer position is not a review decision: options stay in bank
order, and the meetup's order is drawn from the date at scheduling (AC-23).

Exit status: 0 done; 1 refused, with one plain sentence on stderr and the record
unchanged; 2 the input could not be read.
"""

from __future__ import annotations

import argparse
import contextlib
import dataclasses
import io
import json
import os
import sys
import tempfile
from collections.abc import Sequence
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, get_args

from popquiz import bank, copylint, receipt, verify
from popquiz.audit import ACCEPTED
from popquiz.bank import BankError, Explains, Option, OptionKind, Question, Review, TraceStep
from popquiz.generate import DIFFICULTIES
from popquiz.runner import Runner

#: Where the organizer's name comes from when `--by` is not given. No default.
ORGANIZER_ENV = "POPQUIZ_ORGANIZER"

#: The name a does-not-compile question's resolving step gives its row (F-15).
ERROR = "error"

#: What `export` writes and `edit` reads. The trace is editable only as its notes:
#: its lines, focus, pivot and values - the `stdout` row among them - are not.
EDITABLE = ("source", "options", "hint", "explains", "trace_notes")

LETTERS = "ABCDE"


class ReviewRefused(Exception):
    """A decision this module will not record, said in one plain sentence."""


def _now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def _review(question: Question) -> Review:
    return question.review or Review()


def _letters(indexes: Sequence[int]) -> str:
    return ", ".join(LETTERS[i] for i in indexes)


# --------------------------------------------------------------------------- #
# The checks (SPEC 7.4)
# --------------------------------------------------------------------------- #


def answer(question: Question) -> tuple[int | None, str | None]:
    """The derived correct option, or why there is none. Never a guess (G-2)."""
    if question.verified is None:
        return None, "it has no verified record"
    try:
        index = bank.correct_index(question)
    except BankError as exc:
        return None, str(exc)
    if index is None:
        return None, "no option matches what the verifier recorded"
    return index, None


def quotes(question: Question) -> list[dict[str, Any]]:
    """Every quoted span, whether it is a printed line, and whether it was checked (AC-73)."""
    stdout = question.verified.stdout if question.verified else None
    checked = set(_review(question).quotes_checked or ())
    return [
        {"span": span, "matches": bank.quote_matches_output(span, stdout), "checked": span in checked}
        for span in bank.quoted_outputs(question)
    ]


def unchecked_quotes(question: Question, also_checked: Sequence[str] = ()) -> list[str]:
    return [
        q["span"]
        for q in quotes(question)
        if not q["matches"] and not q["checked"] and q["span"] not in also_checked
    ]


def accept_blockers(question: Question, also_checked: Sequence[str] = ()) -> list[str]:
    """What stops acceptance: an answer that does not derive, and quoted text that is
    not a printed line and has not been checked (AC-73)."""
    blockers = []
    _, error = answer(question)
    if error:
        blockers.append(f"The answer does not derive: {error}.")
    spans = unchecked_quotes(question, also_checked)
    if spans:
        blockers.append(
            "Quoted text that is not a printed line needs checking: "
            + ", ".join(f'"{s}"' for s in spans)
            + " (AC-73)."
        )
    return blockers


def _is_error(name: str) -> bool:
    return name.strip().lower() == ERROR


def error_row_blockers(question: Question) -> list[str]:
    """What stops a does-not-compile question's trace from resolving (F-15).

    RULING F-15 (client, 2026-10-25): such a question has no `stdout` for a step to
    name (SPEC 3.2), so its resolving step is the last one, and that step's values
    name `error` with the compiler's code from `verified.compile_error_code`,
    spelled as the receipt spells it. Only the last step may name it: `work` shows
    every step but the last, and an earlier `error` row would give the answer away
    (G-3). A question that ran, or has no trace yet, is not checked here.
    """
    verified = question.verified
    if verified is None or bank.receipt_class(verified) != "does_not_compile" or not question.trace.steps:
        return []
    blockers = []
    *before, last = question.trace.steps
    for i, step in enumerate(before, 1):
        if any(_is_error(v.name) for v in step.values):
            blockers.append(
                f"Trace step {i} names error, but only the last step may; the "
                "walk-through would show the answer before reveal (F-15, G-3)."
            )
    codes = ", ".join(verified.compile_error_code or ())
    rows = [v for v in last.values if _is_error(v.name)]
    if not rows:
        blockers.append(
            f"It does not compile, and the trace's last step names no error; it needs "
            f"an error row saying {codes} (F-15)."
        )
    for row in rows:
        if row.now.strip() != codes:
            blockers.append(f"The trace's error row says {row.now}, but the compiler gave {codes} (F-15).")
    return blockers


def affirm_blockers(question: Question) -> list[str]:
    """Everything SPEC 7.4 asks before an affirmation, except the organizer's name."""
    blockers = []
    if _review(question).status not in ACCEPTED:
        blockers.append("It has not been accepted.")
    blockers.extend(error_row_blockers(question))
    blockers.extend(accept_blockers(question))
    steps = len(question.trace.steps)
    if steps < 2:
        blockers.append(f"The trace needs at least two steps; it has {steps}.")
    index, _ = answer(question)
    if index is not None:
        missing = [
            i
            for i, option in enumerate(question.options)
            if i != index and not (option.why_tempting or "").strip()
        ]
        if missing:
            blockers.append(
                f"Choice {_letters(missing)} has no text saying why it is tempting (AC-95)."
            )
    return blockers


def report(question: Question) -> dict[str, Any]:
    """Everything the review screen shows for one question, in one read (AC-20)."""
    index, error = answer(question)
    review = _review(question)
    verified = question.verified
    return {
        "id": question.id,
        "topic": question.topic,
        "source": question.source,
        "options": [
            {"text": o.text, "kind": o.kind, "why_tempting": o.why_tempting, "correct": i == index}
            for i, o in enumerate(question.options)
        ],
        "answer_error": error,
        "hint": question.hint,
        "explains": {"what": question.explains.what, "takeaway": question.explains.takeaway},
        "trace_steps": len(question.trace.steps),
        "receipt": receipt.receipt_lines(verified) or [],
        "legacy_record": bool(verified and verified.legacy),
        "difficulty_requested": question.difficulty_requested,
        "difficulty_judged": review.difficulty_judged,
        "status": review.status,
        "reason": review.reason,
        "affirmed_by": review.affirmed_by,
        "affirmed_at": review.affirmed_at,
        "affirmation": bank.affirmation_state(question),
        "near_duplicate_of": review.near_duplicate_of,
        "quotes": quotes(question),
        "warnings": [dataclasses.asdict(w) for w in copylint.check_prose(question)],
        "accept_blockers": accept_blockers(question),
        "affirm_blockers": affirm_blockers(question),
        # No ruling is pending since F-15 (2026-10-25); the TUI still reads the key.
        "blocked_on": None,
    }


def rejection_reasons(bank_dir: Path) -> list[tuple[str, str]]:
    """Every rejected question and its reason: the signal AC-19 hands the generator."""
    return [
        (q.id, q.review.reason or "")
        for q in bank.load_bank(bank_dir)
        if q.review is not None and q.review.status == "rejected"
    ]


# --------------------------------------------------------------------------- #
# Decisions
# --------------------------------------------------------------------------- #


def _save(bank_dir: Path, question: Question, **review: Any) -> Path:
    return bank.save_question(
        bank_dir, dataclasses.replace(question, review=dataclasses.replace(_review(question), **review))
    )


def _cleared() -> dict[str, None]:
    return {"affirmed_by": None, "affirmed_at": None, "affirmed_digest": None}


def accept(bank_dir: Path, qid: str, checked: Sequence[str] = ()) -> str:
    question = bank.load_question(bank_dir, qid)
    unmatched = {q["span"] for q in quotes(question) if not q["matches"]}
    stray = [s for s in checked if s not in unmatched]
    if stray:
        raise ReviewRefused(f'"{stray[0]}" is not quoted text in {qid} that needs checking.')
    blockers = accept_blockers(question, checked)
    if blockers:
        raise ReviewRefused(f"{qid} cannot be accepted. " + " ".join(blockers))
    kept = list(_review(question).quotes_checked or ())
    kept += [s for s in checked if s not in kept]
    _save(bank_dir, question, status="accepted", quotes_checked=tuple(kept) or None)
    return f"{qid} accepted."


def reject(bank_dir: Path, qid: str, reason: str) -> str:
    reason = reason.strip()
    if not reason:
        raise ReviewRefused("A rejection needs a reason, so the next generator run can learn from it.")
    question = bank.load_question(bank_dir, qid)
    _save(bank_dir, question, status="rejected", reason=reason, **_cleared())
    return f"{qid} rejected: {reason}"


def judge_difficulty(bank_dir: Path, qid: str, level: int) -> str:
    if level not in DIFFICULTIES:
        raise ReviewRefused(f"Difficulty is {DIFFICULTIES.start} to {DIFFICULTIES.stop - 1}, not {level}.")
    question = bank.load_question(bank_dir, qid)
    _save(bank_dir, question, difficulty_judged=level)
    return f"{qid} judged difficulty {level} (requested {question.difficulty_requested})."


def affirm(bank_dir: Path, qid: str, by: str | None, environ: dict[str, str] | None = None) -> str:
    environ = os.environ if environ is None else environ
    name = (by if by is not None else environ.get(ORGANIZER_ENV, "")).strip()
    if not name:
        raise ReviewRefused(f"Affirm needs your name: set {ORGANIZER_ENV} or pass --by NAME.")
    question = bank.load_question(bank_dir, qid)
    if bank.is_affirmed(question):
        review = _review(question)
        raise ReviewRefused(f"{qid} is already affirmed by {review.affirmed_by} at {review.affirmed_at}.")
    blockers = affirm_blockers(question)
    if blockers:
        raise ReviewRefused(f"{qid} cannot be affirmed. " + " ".join(blockers))
    at = _now()
    _save(
        bank_dir,
        question,
        affirmed_by=name,
        affirmed_at=at,
        affirmed_digest=bank.affirmation_digest(question),
    )
    return f"{qid} affirmed by {name} at {at}."


# --------------------------------------------------------------------------- #
# Edit and re-verify
# --------------------------------------------------------------------------- #


def export(bank_dir: Path, qid: str, out: Path) -> str:
    """The editable fields, for the organizer's editor. Never written into the bank."""
    question = bank.load_question(bank_dir, qid)
    payload = {
        "_note": (
            "Edit the values and save. Changing the source or a choice re-verifies with "
            "the pinned rustc and Miri. The trace's lines and values, its stdout row and "
            "explains.legacy are not editable here."
        ),
        "id": question.id,
        "source": question.source,
        "options": [
            {"text": o.text, "kind": o.kind, "why_tempting": o.why_tempting} for o in question.options
        ],
        "hint": question.hint,
        "explains": {"what": question.explains.what, "takeaway": question.explains.takeaway},
        "trace_notes": [step.note for step in question.trace.steps],
    }
    Path(out).write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return f"{qid}'s editable fields written to {out}."


def _string(data: dict[str, Any], key: str, where: str) -> str:
    value = data[key]
    if not isinstance(value, str):
        raise ReviewRefused(f"{where} must be text.")
    return value


def _edited(question: Question, data: Any) -> Question:
    """The question with the edit file's fields in place. Refuses anything else."""
    if not isinstance(data, dict):
        raise ReviewRefused("The edit file must hold one JSON object.")
    unknown = sorted(k for k in data if not k.startswith("_") and k not in (*EDITABLE, "id"))
    if unknown:
        raise ReviewRefused(f"The edit file has fields review cannot change: {', '.join(unknown)}.")
    if data.get("id", question.id) != question.id:
        raise ReviewRefused(f"The edit file is for {data['id']}, not {question.id}.")

    source = _string(data, "source", "source") if "source" in data else question.source
    hint = _string(data, "hint", "hint") if "hint" in data else question.hint

    options = question.options
    if "options" in data:
        raw = data["options"]
        if not isinstance(raw, list) or len(raw) != bank.OPTION_COUNT:
            raise ReviewRefused(f"options must be a list of {bank.OPTION_COUNT} choices.")
        built = []
        for i, o in enumerate(raw):
            if not isinstance(o, dict) or set(o) - {"text", "kind", "why_tempting"}:
                raise ReviewRefused(f"Choice {LETTERS[i]} may hold only text, kind and why_tempting.")
            kind = o.get("kind", question.options[i].kind)
            if kind not in get_args(OptionKind):
                raise ReviewRefused(f"Choice {LETTERS[i]}'s kind must be one of {', '.join(get_args(OptionKind))}.")
            why = o.get("why_tempting")
            if why is not None and not isinstance(why, str):
                raise ReviewRefused(f"Choice {LETTERS[i]}'s why_tempting must be text.")
            built.append(Option(text=_string(o, "text", f"Choice {LETTERS[i]}'s text"), kind=kind, why_tempting=why or None))
        options = tuple(built)

    explains = question.explains
    if "explains" in data:
        raw = data["explains"]
        if not isinstance(raw, dict) or set(raw) - {"what", "takeaway"}:
            raise ReviewRefused("explains may hold only what and takeaway; explains.legacy is never rewritten.")
        explains = Explains(
            what=_string(raw, "what", "explains.what") if "what" in raw else explains.what,
            takeaway=_string(raw, "takeaway", "explains.takeaway") if "takeaway" in raw else explains.takeaway,
            legacy=question.explains.legacy,
        )

    trace = question.trace
    if "trace_notes" in data:
        notes = data["trace_notes"]
        steps = trace.steps
        if not isinstance(notes, list) or len(notes) != len(steps) or not all(isinstance(n, str) for n in notes):
            raise ReviewRefused(
                f"trace_notes must be {len(steps)} pieces of text, one per step; steps are not added or removed here."
            )
        trace = bank.Trace(steps=tuple(dataclasses.replace(s, note=n) for s, n in zip(steps, notes, strict=True)))

    return dataclasses.replace(question, source=source, hint=hint, options=options, explains=explains, trace=trace)


def _without_stdout_row(trace: bank.Trace) -> bank.Trace:
    """The trace with the last step's `stdout` row removed: nothing has been observed
    for an edited program yet, and only the verifier writes that row back."""
    if not trace.steps:
        return trace
    *before, last = trace.steps
    values = tuple(v for v in last.values if v.name.strip().lower() != "stdout")
    return bank.Trace(steps=(*before, TraceStep(last.lines, last.focus, last.note, values, last.pivot)))


def _verify_into_bank(bank_dir: Path, candidate: Question, expect: str, runner: Runner | None) -> tuple[bool, str]:
    """Verify `candidate` in a scratch bank; copy it into the real one only if accepted.

    `verify.main` is the writer of `verified` and of the trace's `stdout` row. It
    writes into the scratch file; this function never touches either field.
    """
    with tempfile.TemporaryDirectory(prefix="popquiz-review-") as scratch:
        path = bank.save_question(Path(scratch), candidate)
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = verify.main([str(path), "--expect", expect], runner=runner)
        said = (out.getvalue() + err.getvalue()).replace(str(path), str(bank.question_path(bank_dir, candidate.id)))
        if code != 0:
            return False, said.strip()
        verified = bank.question_from_dict(json.loads(path.read_text(encoding="utf-8")))
    bank.save_question(bank_dir, verified)
    return True, said.strip()


def _expect(question: Question, expect: str | None) -> str:
    found = expect or (verify.expect_from_record(question.verified) if question.verified else None)
    if found is None:
        raise ReviewRefused(
            f"{question.id} has no verified record to say what it does; pass --expect "
            f"({', '.join(verify.EXPECTS)})."
        )
    return found


def edit(bank_dir: Path, qid: str, source_file: Path, expect: str | None = None, runner: Runner | None = None) -> str:
    question = bank.load_question(bank_dir, qid)
    try:
        data = json.loads(Path(source_file).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ReviewRefused(f"Cannot read the edit file {source_file}: {exc}.") from None
    edited = _edited(question, data)
    if edited == question:
        return f"No change to {qid}; nothing written."

    # Any edit clears the affirmation (GAP-24). An accepted record becomes `edited`,
    # which still counts as accepted; a rejected or unreviewed one keeps its status,
    # because editing a question is not accepting it.
    #
    # Any edit also clears `quotes_checked` (AC-73; client, 2026-10-31). A check says a
    # span is not a claim about the output, and that depends on the sentence around it
    # and on the program, so a span that survives an edit is checked again before the
    # question can be affirmed. A span removed and later restored comes back unchecked.
    # Both edit paths below write this record.
    prior = _review(question)
    status = "edited" if prior.status in ACCEPTED else prior.status
    edited = dataclasses.replace(
        edited, review=dataclasses.replace(prior, status=status, quotes_checked=None, **_cleared())
    )
    if status == "rejected":
        after = "it stays rejected until you accept it"
    elif status is None:
        after = "it is not accepted yet"
    else:
        after = "its affirmation is cleared and it needs affirming again"
    program_changed = edited.source != question.source or [(o.text, o.kind) for o in edited.options] != [
        (o.text, o.kind) for o in question.options
    ]
    if not program_changed:
        try:
            bank.save_question(bank_dir, edited)
        except BankError as exc:
            raise ReviewRefused(f"The edit makes {qid} invalid: {exc}.") from None
        return f"{qid} edited; {after}."

    kind = _expect(question, expect)
    candidate = dataclasses.replace(edited, verified=None, trace=_without_stdout_row(edited.trace))
    try:
        bank.validate_question(candidate)
    except BankError as exc:
        raise ReviewRefused(f"The edit makes {qid} invalid: {exc}.") from None
    ok, said = _verify_into_bank(bank_dir, candidate, kind, runner)
    if not ok:
        raise ReviewRefused(
            f"The verifier did not accept the edited {qid}, so nothing was written and your edit "
            f"is still in {source_file}. {said}"
        )
    return (
        f"{qid} edited and re-verified; {after}.\n{said}"
    )


def _what_it_says(question: Question) -> tuple[Any, ...]:
    """What re-verifying can change in a question, leaving out when it ran: the text
    the organizer read, the trace's `stdout` row with it, and the recorded result."""
    verified = question.verified
    return (
        bank.affirmation_digest(question),
        verified.stdout if verified else None,
        verified.compile_error_code if verified else None,
    )


def reverify(bank_dir: Path, qid: str, expect: str | None = None, runner: Runner | None = None) -> str:
    question = bank.load_question(bank_dir, qid)
    ok, said = _verify_into_bank(bank_dir, question, _expect(question, expect), runner)
    if not ok:
        raise ReviewRefused(f"The verifier did not accept {qid}, so its record is unchanged. {said}")
    # A re-verify that changed what the question says counts as an edit for its checked
    # quotes (AC-73): each one was checked against an output that is no longer recorded.
    after = bank.load_question(bank_dir, qid)
    if _what_it_says(after) != _what_it_says(question):
        _save(bank_dir, after, quotes_checked=None)
    return f"{qid} re-verified.\n{said}"


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #


def main(argv: Sequence[str] | None = None, *, runner: Runner | None = None) -> int:
    """The CLI. `runner` reaches `verify.main` and exists for tests only: without it,
    re-verification runs on the pinned toolchain."""
    parser = argparse.ArgumentParser(
        prog="popquiz.review",
        description="Record the organizer's review decisions on one question (SPEC 7.4).",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    def command(name: str, help: str) -> argparse.ArgumentParser:
        p = sub.add_parser(name, help=help)
        if name != "reasons":
            p.add_argument("question_id")
        p.add_argument("--bank", type=Path, required=True, help="the bank directory")
        return p

    command("show", help="print the review report as JSON")
    a = command("accept", help="accept the question")
    a.add_argument("--checked-quote", action="append", default=[], metavar="SPAN",
                   help="quoted text you checked is not a claim about the output (AC-73); repeatable")
    r = command("reject", help="reject the question, with a reason")
    r.add_argument("--reason", required=True)
    d = command("difficulty", help="record the difficulty you judged")
    d.add_argument("level", type=int)
    x = command("export", help="write the editable fields to a file for your editor")
    x.add_argument("--out", type=Path, required=True)
    e = command("edit", help="save an edit; a source or choice change re-verifies")
    e.add_argument("--from", dest="source_file", type=Path, required=True)
    e.add_argument("--expect", choices=verify.EXPECTS)
    v = command("reverify", help="re-run the verifier on the question as it stands")
    v.add_argument("--expect", choices=verify.EXPECTS)
    f = command("affirm", help="affirm the question, recording who and when")
    f.add_argument("--by", help=f"your name (default: ${ORGANIZER_ENV}; there is no other default)")
    command("reasons", help="list rejected questions and their reasons")

    args = parser.parse_args(argv)
    try:
        if args.command == "show":
            print(json.dumps(report(bank.load_question(args.bank, args.question_id)), ensure_ascii=False))
            return 0
        if args.command == "reasons":
            for qid, reason in rejection_reasons(args.bank):
                print(json.dumps({"id": qid, "reason": reason}, ensure_ascii=False))
            return 0
        if args.command == "accept":
            said = accept(args.bank, args.question_id, args.checked_quote)
        elif args.command == "reject":
            said = reject(args.bank, args.question_id, args.reason)
        elif args.command == "difficulty":
            said = judge_difficulty(args.bank, args.question_id, args.level)
        elif args.command == "export":
            said = export(args.bank, args.question_id, args.out)
        elif args.command == "edit":
            said = edit(args.bank, args.question_id, args.source_file, args.expect, runner)
        elif args.command == "reverify":
            said = reverify(args.bank, args.question_id, args.expect, runner)
        else:
            said = affirm(args.bank, args.question_id, args.by)
    except ReviewRefused as exc:
        print(f"popquiz.review: {exc}", file=sys.stderr)
        return 1
    except (BankError, OSError, json.JSONDecodeError) as exc:
        print(f"popquiz.review: cannot read: {exc}", file=sys.stderr)
        return 2
    print(said)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
