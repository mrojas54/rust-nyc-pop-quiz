"""Migrate the August MVP batch into the bank as `legacy` records (BUILDPLAN T-14).

Run once; its output is committed. A re-run writes only what the bank does not
already hold, so it can never undo what happened to a record after it landed:
q4, q7 and q8 were re-authored and re-verified since (D-15, D-16).
`test_migration.py` still regenerates all four into an empty directory to show
where each record came from.

    cd pipeline && uv run python -m popquiz.migrate_mvp

**Which questions, and why only four.** SPEC 5.2's fit table, worked at the guessed
15 ft / 20 ft room, is the authority:

* **q3** migrates as authored. Its five options are each one line of at most 29
  characters, which is the only one of the eight that passes D-15 unchanged.
* **q4, q7 and q8** migrate with their programs and their legacy records, and a
  review note saying what each needs. They fit as programs; their options do not.
* **q1, q2, q5 and q6** are not migrated at all. At 16, 10, 9 and 9 source lines
  they exceed the reading layout's 7-line capacity at the floor, so the wall cannot
  show them at a legible size. They stay in the MVP bank until they are re-authored
  or until the rehearsal's measurements (H-5) raise the capacity. `bank/README.md`
  says so where a reader will look for them.

**Nothing a program printed is typed in this file.** Every `stdout` is copied from
`mvp/2026-08-12/verified.json`, which a machine wrote, and the two places that
would otherwise be hand-written outputs are derived instead:

* the **correct option's text**, which is `normalized_output(verified.stdout)` and
  never an authored string, and
* the **trace's resolving step**, whose `stdout` value is built from the record
  (D-10) rather than copied from the prototype, where it was authored by hand
  (`prototypes/_shared/data.js:127`).

`test_migration.py` asserts both, and also that this file's own source text contains
no literal equal to any of the batch's recorded answers - the house rule, mechanized.

**What is authored here, and that is allowed.** The beats and the trace. A human may
write the program, the distractors and the explanation; a human may never write down
what the program prints (PHILOSOPHY 3). Every record this writes leaves
`review.status` unset and no affirmation, so nothing it drafts can reach a room
before an organizer has read it (SPEC 7.4, G-12).
"""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Callable
from pathlib import Path

from popquiz.bank import (
    MAX_OPTION_LINE_CHARS,
    BankError,
    Explains,
    History,
    Miri,
    Option,
    Question,
    Review,
    Runs,
    Trace,
    TraceStep,
    ValueDelta,
    Verified,
    history_path,
    load_history,
    load_question,
    normalized_output,
    options_needing_reauthoring,
    question_path,
    save_history,
    save_question,
)

MIGRATED = ("q3", "q4", "q7", "q8")

# Not migrated, with the measurement that excludes each: source lines against the
# reading layout's 7-line capacity at the floor (SPEC 5.2). Kept as data so
# `bank/README.md` and this script cannot disagree about which questions or why.
NOT_MIGRATED = {
    "q1": "16 source lines",
    "q2": "10 source lines",
    "q5": "9 source lines",
    "q6": "9 source lines",
}

DOES_NOT_COMPILE = "does not compile"

# --------------------------------------------------------------------------- #
# Authored content: the beats and the trace.
#
# q3's `what`, its `[1, 2, 3]` beat and its trace notes are VERBATIM from
# `prototypes/_shared/data.js:62-129`, the shape the client loved on 2026-09-03.
# Two renames happen on the way in, both of which SPEC 3.1 requires: the
# prototype's `argue` becomes `takeaway` (a Stage-2 leak, named as one in the
# spec), and `whyWrong`, which keyed one beat by the letter "A", becomes a beat per
# option keyed by the option's own text - a letter is drawn from the meetup date,
# so a letter-keyed beat points at a different option every month.
#
# q3's other three beats are DRAFTED from the August explanation. q4's, q7's and
# q8's `what` and `takeaway` are drafted from the same source; their
# `why_tempting` beats are deliberately absent, because every incorrect option they
# would attach to is scheduled for replacement.
# --------------------------------------------------------------------------- #

AUTHORED_BEATS: dict[str, dict[str, str]] = {
    "q3": {
        "what": (
            "dedup only removes duplicates that are sitting next to each other. The "
            "2,2 in the middle and the 1,1 at the end were adjacent, so they "
            "collapsed into one. The other 2 and the other 1 weren't next to their "
            "twins, so they stayed where they were."
        ),
        "takeaway": (
            "To get each value once, either sort first, which puts the duplicates "
            "next to each other, or use a HashSet. Which you pick depends on whether "
            "you care about order."
        ),
    },
    "q4": {
        "what": (
            "Rust's / and % truncate toward zero, so -7 / 2 loses the fraction and "
            "the remainder keeps the sign of the number being divided. div_euclid "
            "and rem_euclid round the other way, down, which is what makes their "
            "remainder come back positive."
        ),
        "takeaway": (
            "Both are real division and both satisfy the same identity: the quotient "
            "times the divisor plus the remainder gets you back where you started. "
            "They only disagree about which way to round. If you have ever indexed "
            "into a ring buffer with a negative offset, you have met this."
        ),
    },
    "q7": {
        "what": (
            "sort_by_key is a stable sort. Two entries whose keys compare equal come "
            "out in the order they went in, so the pairs that share a key keep their "
            "original relative order and only the keys move them."
        ),
        "takeaway": (
            "sort_unstable_by_key makes no such promise and is usually a little "
            "faster. The standard library gives you both on purpose, so the choice "
            "is yours to make rather than something to discover afterwards."
        ),
    },
    "q8": {
        "what": (
            "&v[0] is a shared borrow pointing into the vector's heap buffer. push "
            "needs the vector mutably and may move that buffer somewhere else to "
            "grow it, which would leave the borrow pointing at freed memory. The "
            "borrow checker refuses it before the program ever runs."
        ),
        "takeaway": (
            "The borrow is only still alive because it is used after the push. "
            "Delete the last line and the same program compiles - that is "
            "non-lexical lifetimes, and it is why this error seems to move around "
            "when you edit lines that look unrelated."
        ),
    },
}

# One beat per incorrect option, keyed by that option's text. Every key is checked
# against the options actually built, so a typo fails the migration rather than
# quietly leaving an option without its beat.
AUTHORED_WHY_TEMPTING: dict[str, dict[str, str]] = {
    "q3": {
        # Verbatim from the prototype.
        "[1, 2, 3]": (
            "In almost every other language, a method called dedup means unique: "
            "give me each value once. Rust's doesn't. It removes only consecutive "
            "duplicates. That word appears once in the docs and is easy to read past."
        ),
        # Drafted from the August explanation.
        "[1, 2, 2, 3, 2, 1, 1]": (
            "A reasonable reading is that dedup wants a sorted vector and does "
            "nothing useful without one, so it leaves this vector alone. It does not "
            "check and it does not refuse - it makes its one pass regardless."
        ),
        "[1, 2, 3, 2, 1, 1]": (
            "This is what you get if you catch the pair in the middle and stop "
            "looking. The other pair is the last two elements, which is the easiest "
            "place in a list to skim past."
        ),
        DOES_NOT_COMPILE: (
            "dedup needs the vector mutably and this one is declared mut, so nothing "
            "here is a borrow problem. Reaching for this option is usually a guess "
            "that dedup has a precondition - sorted, or comparable - that the "
            "compiler would enforce. It has neither."
        ),
    },
}

# q3's trace, from the prototype. The resolving step's `stdout` value is NOT here:
# `_trace_for` derives it from the verified record. The intermediate values are
# program state rather than printed output, which is why they are authored - and
# they are exactly what an organizer affirms.
AUTHORED_TRACE: dict[str, tuple[dict[str, object], ...]] = {
    "q3": (
        {
            "lines": (2,),
            "focus": (1, 5),
            "note": (
                "Seven elements go in — and two of the pairs happen to be sitting "
                "next to each other. Hold on to which ones."
            ),
            "values": (("v", "—", "[1, 2, 2, 3, 2, 1, 1]"),),
        },
        {
            "lines": (3,),
            "focus": (2, 4),
            "note": (
                "dedup makes one pass, comparing each element with the one "
                "immediately before it. Only neighbours are ever compared — it never "
                "looks further back."
            ),
            "values": (),
        },
        {
            "lines": (3,),
            "focus": (3, 3),
            "note": "The 2, 2 in the middle collapses. Same value, side by side.",
            "values": (("v", "[1, 2, 2, 3, 2, 1, 1]", "[1, 2, 3, 2, 1, 1]"),),
        },
        {
            "lines": (3,),
            "focus": (3, 3),
            "note": "The 1, 1 on the end collapses for exactly the same reason.",
            "values": (("v", "[1, 2, 3, 2, 1, 1]", "[1, 2, 3, 2, 1]"),),
        },
        {
            "lines": (3,),
            "focus": (3, 3),
            "pivot": True,
            "note": (
                "And the other 2 and the other 1 survive — because they were never "
                "next to their twins. There is already a 2 near the front, and dedup "
                "did not look. This is the step the whole question turns on."
            ),
            "values": (),
        },
        {
            "lines": (4,),
            "focus": (1, 5),
            "note": (
                "Five left. Two were dropped, and both of them only because of who "
                "they happened to be sitting next to."
            ),
            # The resolving step (D-10). Its stdout value is derived, not authored.
            "values": (),
            "resolving": True,
        },
    ),
}


class MigrationError(Exception):
    """The MVP files do not hold what this migration was written against."""


def _verified_for(batch: dict, question: dict) -> Verified:
    """The legacy `verified` record for one MVP question (SPEC 3.2, D-16).

    Only what the MVP wrote. `rustc` is the `--version` string it recorded, never a
    `-Vv`; `target_triple`, `flags`, `exit_code`, `verified_at` and
    `verifier_version` are absent and are not back-filled or inferred (G-2).

    The MVP's `miri` field is free text. `"clean"` becomes `clean=True`; anything
    else on a compiling question raises, because translating an unrecognised string
    into a boolean would be this migration deciding what the Miri pass found.
    """
    qid = question["id"]

    if not question["compiled"]:
        codes = question.get("error_codes")
        if not codes:
            raise MigrationError(
                f"{qid} did not compile but records no error codes; the "
                "does-not-compile receipt needs them (SPEC 7.5, D-22)"
            )
        # No runs, no stdout, no miri: nothing ran. The MVP's miri field reads
        # "n/a (does not compile)", which is not a result, so no record is written.
        return Verified(
            rustc=batch["rustc"],
            edition=batch["edition"],
            legacy=True,
            compile_error_code=tuple(codes),
        )

    miri_text = question.get("miri")
    if miri_text != "clean":
        raise MigrationError(
            f"{qid} records its Miri result as {miri_text!r}, and this migration "
            "only knows how to read 'clean'. Translating anything else would be "
            "inventing a result the pass did not state."
        )
    if "miri_output_matches" not in question:
        raise MigrationError(f"{qid} records no miri_output_matches")

    return Verified(
        rustc=batch["rustc"],
        edition=batch["edition"],
        legacy=True,
        runs=Runs(count=question["runs"], byte_identical=question["byte_identical"]),
        stdout=question["answer"],
        # The August pass ran outside the repo and recorded no version, no borrow
        # models and no seeds, so none are written (D-16).
        miri=Miri(clean=True, output_matched=question["miri_output_matches"]),
    )


def _options_for(
    question: dict, distractors: list[str]
) -> tuple[tuple[Option, ...], int]:
    """The five options in canonical bank order.

    **Bank order carries no meaning.** The wall's order and its letters are drawn
    from the meetup date (AC-23); this order exists only so two runs of the
    migration produce the same file. Authored distractors first, then *does not
    compile* when it is not the answer, then the correct option last.

    The correct option's text is `normalized_output` of the recorded `stdout` - the
    machine's output, stripped of the one newline `println!` added, exactly as
    `mvp/tools/build_deck.py:266` does when it builds a deck. It is never an
    authored string.

    Returns the options and the correct one's index. The index is returned rather
    than re-derived by the caller because `correct_index` needs an assembled
    `Question`, and the review note has to know which option is correct before there
    is one to ask. `test_migration.py` closes the loop: for every written record,
    `correct_index` agrees with the position this put the answer in.
    """
    texts = list(distractors)

    if question["compiled"]:
        correct_text = normalized_output(question["answer"])
        if DOES_NOT_COMPILE not in texts:
            texts.append(DOES_NOT_COMPILE)
    else:
        # The answer is the does-not-compile option itself, so it is not also a
        # distractor. AC-24 is satisfied by the answer, not by an extra option.
        correct_text = DOES_NOT_COMPILE
        if DOES_NOT_COMPILE in texts:
            raise MigrationError(
                f"{question['id']}: {DOES_NOT_COMPILE!r} is both the answer and a "
                "distractor, which would make two options identical"
            )
    texts.append(correct_text)

    beats = AUTHORED_WHY_TEMPTING.get(question["id"], {})
    unmatched = sorted(set(beats) - set(texts))
    if unmatched:
        raise MigrationError(
            f"{question['id']}: why_tempting beats written for options that do not "
            f"exist: {unmatched}"
        )

    options = tuple(
        Option(
            text=text,
            kind="does_not_compile" if text == DOES_NOT_COMPILE else "output",
            # The correct option has no tempting to explain (AC-95 is about the
            # incorrect ones), so it never carries a beat even if one was written.
            why_tempting=None if text == correct_text else beats.get(text),
        )
        for text in texts
    )
    return options, texts.index(correct_text)


def _trace_for(question: dict, verified: Verified) -> Trace:
    """The trace, with its resolving step derived from the verified record.

    q4, q7 and q8 get an empty trace. The MVP holds no step structure for them, and
    their options are about to be re-authored, so a walk-through drafted now would
    be drafted against text that will not exist. An empty trace also blocks affirm
    (SPEC 7.4), which is the correct state for a question awaiting re-verification -
    a stub step would have made it look ready.
    """
    authored = AUTHORED_TRACE.get(question["id"])
    if not authored:
        return Trace()

    steps = []
    for raw in authored:
        values = [
            ValueDelta(name=n, was=w, now=now)
            for n, w, now in raw["values"]  # type: ignore[misc]
        ]
        if raw.get("resolving"):
            if verified.stdout is None:
                raise MigrationError(
                    f"{question['id']}: its trace has a resolving step, but the "
                    "record holds no stdout for that step to name (D-10)"
                )
            # Derived, never typed. `data.js:127` wrote this value by hand; here it
            # comes from the record the machine produced.
            values.append(
                ValueDelta(name="stdout", was="—", now=normalized_output(verified.stdout))
            )
        steps.append(
            TraceStep(
                lines=tuple(raw["lines"]),  # type: ignore[arg-type]
                focus=raw["focus"],  # type: ignore[arg-type]
                note=raw["note"],  # type: ignore[arg-type]
                values=tuple(values),
                pivot=bool(raw.get("pivot", False)),
            )
        )
    return Trace(steps=tuple(steps))


def _review_for(
    question_id: str, options: tuple[Option, ...], correct: int
) -> Review:
    """The review record: never affirmed, and saying what each question still needs.

    `status` is unset. These questions have not been judged, and unset is not
    rejection - the review surface will show them as waiting (SPEC 7.4).

    The note goes in `reason`, whose reader per SPEC 3.1 is the generator's next
    run, which is exactly who needs to know that the wall's options are one line of
    at most 29 characters. No new field is minted for it.

    The counts are measured, not typed, so this note and `bank-audit` cannot
    disagree about which options fail.
    """
    failing = options_needing_reauthoring(options)

    if not failing:
        return Review(
            reason=(
                "Migrated from the August batch by T-14. Every option is one line of "
                f"at most {MAX_OPTION_LINE_CHARS} characters, so this question runs "
                "on the wall as authored (D-15). The three beats and the trace were "
                "drafted from the August explanation and the loved prototype, and "
                "need an organizer to read them and affirm."
            )
        )

    one = len(failing) == 1
    # The note is read by a person deciding what to re-author, so it reads as
    # English at either count rather than as a template with a number in it.
    breaks = "breaks" if one else "break"
    index_list = (
        f"index {failing[0]}" if one else f"indexes {', '.join(str(i) for i in failing)}"
    )
    if correct in failing:
        consequence = (
            "The correct option is among them, and an option's text has to equal "
            "what the program printed, so shortening it is not an edit to the "
            "option - the program has to print something shorter, and the machine "
            "decides the new answer when it is re-verified (T-15b)."
        )
    else:
        consequence = (
            "The correct option is not among them, so the answer stands and the "
            "program need not change: "
            + ("only that distractor needs" if one else "only those distractors need")
            + " replacing."
        )
    return Review(
        reason=(
            f"Migrated from the August batch by T-14. {len(failing)} of "
            f"{len(options)} options {breaks} the wall's rule that an option is one "
            f"line of at most {MAX_OPTION_LINE_CHARS} characters (D-15, SPEC 5.2): "
            f"{index_list} in bank order. {consequence} No why_tempting beats were "
            "drafted, because every incorrect option they would attach to is "
            "scheduled for replacement. Cannot be affirmed as it stands: the trace "
            "is empty and the middle beats are missing (SPEC 7.4, AC-95)."
        )
    )


def build_question(batch: dict, question: dict, content: dict) -> Question:
    """One bank record from the MVP's two files."""
    qid = question["id"]
    beats = AUTHORED_BEATS.get(qid)
    if beats is None:
        raise MigrationError(f"{qid}: no beats written for it in this migration")
    if "explanation" not in content:
        raise MigrationError(f"{qid}: content.json holds no explanation")

    verified = _verified_for(batch, question)
    options, correct = _options_for(question, list(content["distractors"]))

    return Question(
        id=qid,
        source=question["source"],
        topic=question["topic"],
        difficulty_requested=question["difficulty"],
        options=options,
        hint=question["hint"],
        explains=Explains(
            what=beats["what"],
            takeaway=beats["takeaway"],
            # Verbatim, for AC-73's check against the text the organizer reviewed in
            # August. Never rewritten, never folded into the beats.
            legacy=content["explanation"],
        ),
        trace=_trace_for(question, verified),
        verified=verified,
        review=_review_for(qid, options, correct),
    )


def migrate(mvp_dir: Path, bank_dir: Path) -> list[Path]:
    """Read the MVP batch, write the bank records and the history shape.

    A file already in the bank is kept, never overwritten: the bank is
    append-only (SPEC 3.3), and a record that was re-authored or re-verified
    after it was migrated is no longer the migration's to write. A file that is
    there but cannot be read stops the run with `MigrationError`, neither kept
    nor overwritten. Returns the paths it wrote.
    """
    batch = json.loads((mvp_dir / "verified.json").read_text(encoding="utf-8"))
    content = json.loads((mvp_dir / "content.json").read_text(encoding="utf-8"))
    by_id = {q["id"]: q for q in batch["questions"]}

    missing = [qid for qid in MIGRATED if qid not in by_id]
    if missing:
        raise MigrationError(f"the batch does not hold {missing}")

    written = []
    for qid in MIGRATED:
        question = build_question(batch, by_id[qid], content[qid])
        if _already_held(question_path(bank_dir, qid), lambda: load_question(bank_dir, qid)):
            continue
        written.append(save_question(bank_dir, question))

    # The history store's shape, with its three stores empty. T-17 fills and reads
    # them and proves AC-17 on this shape; writing it here means T-17 arrives to a
    # file that exists rather than one it has to invent (SPEC 3.3).
    if not _already_held(history_path(bank_dir), lambda: load_history(bank_dir)):
        written.append(save_history(bank_dir, History()))
    return written


def _already_held(path: Path, load: Callable[[], object]) -> bool:
    """Whether the bank already holds a readable file at `path`.

    A file that is there but does not load - truncated, say - stops the run. Keeping
    it would leave a damaged record looking migrated, and overwriting it would put
    the August record back over whatever it had become (SPEC 3.3).
    """
    if not path.exists():
        return False
    try:
        load()
    except (OSError, ValueError, KeyError, TypeError, BankError) as exc:
        raise MigrationError(
            f"{path} is in the bank but cannot be read ({exc}); it is neither kept nor "
            "overwritten. Restore it from git, then re-run."
        ) from exc
    return True


def main(argv: list[str] | None = None) -> int:
    """Exit 0: migrated, or kept what the bank already held. Exit 1: refused, a
    `MigrationError` such as a bank file that cannot be read, said on stderr."""
    repo =Path(__file__).resolve().parents[3]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mvp-dir", type=Path, default=repo / "mvp" / "2026-08-12")
    parser.add_argument("--bank-dir", type=Path, default=repo / "bank")
    args = parser.parse_args(argv)

    try:
        written = migrate(args.mvp_dir, args.bank_dir)
    except MigrationError as e:
        print(f"popquiz.migrate_mvp: {e}", file=sys.stderr)
        return 1
    for path in written:
        print(f"wrote {path}")
    kept = [qid for qid in MIGRATED if question_path(args.bank_dir, qid) not in written]
    if history_path(args.bank_dir) not in written:
        kept.append("history.json")
    if kept:
        print("kept, already in the bank: " + ", ".join(kept))
    print(
        "not migrated: "
        + ", ".join(f"{q} ({why})" for q, why in sorted(NOT_MIGRATED.items()))
        + " — over the wall's 7-line capacity at the guessed room (SPEC 5.2)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
