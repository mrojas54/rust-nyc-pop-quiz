"""The bank's record format (SPEC 3.1-3.3): the derivations, the store, AC-17's history.

The theme running through these is that a record may not say more than somebody
observed. That shows up twice mechanically: the correct answer is computed rather
than stored, so there is nothing to hand-edit (AC-7, G-2), and a field nobody
observed is written as an absent key rather than a null, so a legacy record's
missing target triple reads as missing rather than as a value somebody could later
fill in (D-16).
"""

from __future__ import annotations

import dataclasses
import json
import pathlib

import pytest

from popquiz import bank
from popquiz.bank import (
    BankError,
    Explains,
    Flags,
    History,
    Miri,
    Option,
    Question,
    Review,
    Runs,
    Trace,
    TraceStep,
    Used,
    ValueDelta,
    Verified,
    append_question,
    correct_index,
    incorrect_indexes,
    load_bank,
    load_history,
    load_question,
    normalized_output,
    options_needing_reauthoring,
    question_from_dict,
    question_to_dict,
    quoted_outputs,
    receipt_class,
    save_history,
    save_question,
    source_hash,
    validate_question,
)

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
BANK = REPO / "bank"
SCHEMA = BANK / "schema" / "question.schema.json"

RAN = Verified(
    rustc="rustc 1.96.1 (31fca3adb 2026-06-26)",
    edition="2021",
    legacy=True,
    runs=Runs(count=5, byte_identical=True),
    stdout="two\n",
    miri=Miri(clean=True, output_matched=True),
)

REFUSED = Verified(
    rustc="rustc 1.96.1 (31fca3adb 2026-06-26)",
    edition="2021",
    legacy=True,
    compile_error_code=("E0502",),
)


def _options(*texts: str, does_not_compile_at: int = 4) -> tuple[Option, ...]:
    return tuple(
        Option(
            text=t,
            kind="does_not_compile" if i == does_not_compile_at else "output",
            why_tempting=None,
        )
        for i, t in enumerate(texts)
    )


def _question(**overrides: object) -> Question:
    """A minimal valid record. Its 'outputs' are words, not any program's output -
    nothing here claims to have been run."""
    base: dict[str, object] = {
        "id": "qX",
        "source": "fn main() {}\n",
        "topic": "Testing",
        "difficulty_requested": 2,
        "options": _options("one", "two", "three", "four", "does not compile"),
        "hint": "a hint",
        "explains": Explains(what="what happened", takeaway="what to remember"),
        "verified": RAN,
    }
    base.update(overrides)
    return Question(**base)  # type: ignore[arg-type]


# --------------------------------------------------------------------------- #
# The derived answer (SPEC 3.1, AC-7, G-2)
# --------------------------------------------------------------------------- #


def test_there_is_no_correct_field_to_hand_edit() -> None:
    """AC-7, G-2: the correct answer is derived, so the record has nowhere to store one.

    This is the structural half of AC-7. A bank record with a `correct` key would be
    a key a person could edit, and the provenance check would then have to catch the
    edit; with no key there is nothing to catch.
    """
    assert "correct" not in {f.name for f in dataclasses.fields(Question)}
    assert "correct" not in question_to_dict(_question())
    assert "correct" not in json.loads(SCHEMA.read_text(encoding="utf-8"))["properties"]


def test_the_correct_option_is_the_one_matching_the_verified_output() -> None:
    assert correct_index(_question()) == 1  # "two" == normalized "two\n"


def test_the_correct_option_of_a_does_not_compile_question_is_that_option() -> None:
    q = _question(verified=REFUSED)
    assert correct_index(q) == 4
    assert q.options[4].kind == "does_not_compile"


def test_no_option_matching_the_output_is_a_finding_not_a_default() -> None:
    """The drift AC-7 exists to catch: the options and the machine's output have
    parted company, so there is no correct answer rather than a guessed one."""
    q = _question(options=_options("one", "nine", "three", "four", "does not compile"))
    assert correct_index(q) is None


def test_two_options_matching_the_output_is_an_error() -> None:
    """"The option whose text equals the output" names no single option when two do,
    and a caller silently taking the first would be inventing an answer."""
    # `validate_question` refuses duplicate option text, so a record shaped like this
    # never reaches disk. The guard is here for the caller that builds one in memory,
    # where returning the first match would invent an answer rather than report a
    # record that cannot name one.
    q = _question(
        options=(
            Option(text="two", kind="output"),
            Option(text="two", kind="output"),
            Option(text="three", kind="output"),
            Option(text="four", kind="output"),
            Option(text="does not compile", kind="does_not_compile"),
        )
    )
    with pytest.raises(BankError, match="options match"):
        correct_index(q)


# A panic and UB are answers by option *kind* (ruling on F-19, 2026-09-25). The
# records below are shapes, not observations: no program is behind them.
_KINDED = (
    Option(text="one", kind="output"),
    Option(text="two", kind="output"),
    Option(text="it panics", kind="panic"),
    Option(text="undefined behaviour", kind="ub"),
    Option(text="does not compile", kind="does_not_compile"),
)
_BOTH_MODELS = Miri(clean=False, output_matched=True, configs=bank.BORROW_MODELS)


def _ran(**fields: object) -> Verified:
    return dataclasses.replace(RAN, legacy=False, **fields)  # type: ignore[arg-type]


def test_a_panic_derives_the_panic_option_by_kind() -> None:
    q = _question(options=_KINDED, verified=_ran(exit_code=101))
    assert correct_index(q) == 2


def test_a_panic_is_never_the_output_option_even_when_the_text_matches() -> None:
    """The printed text before a panic can equal an output option; kind decides."""
    q = _question(options=_KINDED, verified=_ran(stdout="two\n", exit_code=101))
    assert correct_index(q) == 2


def test_a_panic_with_no_panic_option_derives_nothing_rather_than_the_output() -> None:
    """A rule that fires is final: it does not fall through to text equality."""
    q = _question(verified=_ran(stdout="two\n", exit_code=101))
    assert correct_index(q) is None


def test_ub_under_both_borrow_models_derives_the_ub_option() -> None:
    q = _question(options=_KINDED, verified=_ran(exit_code=0, miri=_BOTH_MODELS))
    assert correct_index(q) == 3


def test_ub_outranks_a_nonzero_exit() -> None:
    q = _question(options=_KINDED, verified=_ran(exit_code=101, miri=_BOTH_MODELS))
    assert correct_index(q) == 3


def test_ub_under_one_borrow_model_is_not_a_ub_answer() -> None:
    """verify accepts declared UB only when both models agree (AC-9); a record naming
    one model is not that, so it is not read as one."""
    one = Miri(clean=False, output_matched=True, configs=("stacked_borrows",))
    q = _question(options=_KINDED, verified=_ran(stdout="nine\n", exit_code=0, miri=one))
    assert correct_index(q) is None


def test_a_legacy_unclean_miri_record_is_not_read_as_confirmed_ub() -> None:
    """No configs recorded, so which models agreed is unknown and nothing is inferred."""
    legacy = dataclasses.replace(RAN, stdout="nine\n", miri=Miri(clean=False, output_matched=True))
    assert correct_index(_question(options=_KINDED, verified=legacy)) is None


def test_two_options_of_the_derived_kind_is_an_error() -> None:
    options = _KINDED[:1] + (Option(text="panics", kind="panic"),) + _KINDED[2:]
    with pytest.raises(BankError, match="options match"):
        correct_index(_question(options=options, verified=_ran(exit_code=101)))


def test_a_question_with_no_verified_record_has_no_derived_answer() -> None:
    assert correct_index(_question(verified=None)) is None


def test_the_output_comparison_strips_exactly_one_trailing_newline() -> None:
    """SPEC 3.1 says the correct option's text equals the verified output, but every
    recorded `stdout` ends in the newline println! wrote and no option text does
    (mvp/tools/build_deck.py:266). Narrow on purpose: a question whose answer turned
    on trailing whitespace is one a wider normalization would get wrong."""
    assert normalized_output("two\n") == "two"
    assert normalized_output("two\n\n") == "two\n"
    assert normalized_output("two") == "two"
    assert normalized_output("  two  \n") == "  two  "
    assert normalized_output("") == ""


def test_the_incorrect_options_are_every_option_but_the_correct_one() -> None:
    """AC-95's set: every one of these needs a `why_tempting` before affirm."""
    assert incorrect_indexes(_question()) == (0, 2, 3, 4)


def test_the_incorrect_options_cannot_be_listed_without_a_derived_answer() -> None:
    with pytest.raises(BankError, match="cannot list the incorrect options"):
        incorrect_indexes(_question(verified=None))


def test_receipt_class_puts_does_not_compile_first() -> None:
    """SPEC 7.5's precedence, at the level the classifier owns it."""
    assert receipt_class(RAN) == "ran"
    assert receipt_class(REFUSED) == "does_not_compile"
    assert receipt_class(Verified(rustc="r", edition="2021")) is None
    assert receipt_class(None) is None
    both = dataclasses.replace(RAN, compile_error_code=("E0502",))
    assert receipt_class(both) == "does_not_compile"


# --------------------------------------------------------------------------- #
# The wall's option rule (SPEC 5.2, D-15)
# --------------------------------------------------------------------------- #


def test_an_option_over_one_line_or_29_characters_needs_reauthoring() -> None:
    """D-15. The wall has no design for a wrapped option, so one is a bank-audit
    failure rather than a layout the wall improvises (SPEC 5.2)."""
    options = _options(
        "fits",
        "x" * 29,
        "x" * 30,
        "two\nlines",
        "does not compile",
    )
    assert options_needing_reauthoring(options) == (2, 3)


# --------------------------------------------------------------------------- #
# Validation (SPEC 3.1, AC-24, D-10)
# --------------------------------------------------------------------------- #


def test_a_record_needs_five_options_with_exactly_one_does_not_compile() -> None:
    """AC-24's shape. The bank-wide audit is T-19's; this keeps a malformed record
    off disk in the first place."""
    with pytest.raises(BankError, match="4 options"):
        validate_question(_question(options=_options("a", "b", "c", "does not compile")))

    none_of_them = tuple(
        Option(text=t, kind="output") for t in ("a", "b", "c", "d", "two")
    )
    with pytest.raises(BankError, match="0 options of kind"):
        validate_question(_question(options=none_of_them))

    two_of_them = _options("a", "b", "c", "does not compile", "does not compile")
    with pytest.raises(BankError, match="duplicate option text"):
        validate_question(_question(options=two_of_them))


def test_duplicate_option_text_is_refused() -> None:
    """Two identical options make the derived answer ambiguous and give a room two
    buttons that mean the same thing."""
    with pytest.raises(BankError, match="duplicate option text"):
        validate_question(
            _question(options=_options("one", "one", "three", "four", "does not compile"))
        )


def test_a_question_that_ran_must_have_a_trace_ending_on_stdout() -> None:
    """SPEC 3.1, D-10: the last step is the resolving one."""
    no_stdout = Trace(
        steps=(
            TraceStep(lines=(1,), focus=(1, 1), note="first"),
            TraceStep(
                lines=(2,),
                focus=(1, 2),
                note="last",
                values=(ValueDelta(name="v", was="—", now="1"),),
            ),
        )
    )
    with pytest.raises(BankError, match="does not name 'stdout'"):
        validate_question(_question(trace=no_stdout))


def test_an_empty_trace_is_valid_and_unaffirmable() -> None:
    """A question awaiting its walk-through is storable and cannot be affirmed.

    SPEC 7.4 needs two steps before affirm, so an empty trace is the honest state -
    a stub step would have made it look ready. q4, q7 and q8 migrate this way.
    """
    q = _question(trace=Trace())
    validate_question(q)
    assert q.trace.resolving_step() is None
    assert q.trace.names_stdout_last() is False


def test_a_does_not_compile_question_may_trace_without_naming_stdout() -> None:
    """The narrow reading, and the reason for it.

    SPEC 3.1 and D-10 say the last trace step names `stdout`; SPEC 3.2 says a
    does-not-compile record has none. Requiring it of that class would make a class
    of question unstorable, so the invariant binds only a record that ran.

    This does NOT resolve the affirm gate, which asks two trace steps of a question
    that can never have a resolving one. That is a contract defect reported with this
    ticket and not fixed in it.
    """
    trace = Trace(
        steps=(
            TraceStep(lines=(3,), focus=(1, 5), note="the borrow is taken here"),
            TraceStep(lines=(4,), focus=(1, 5), note="and push wants the vector"),
        )
    )
    validate_question(_question(verified=REFUSED, trace=trace))


@pytest.mark.parametrize("name", ["stdout", " Stdout "])
@pytest.mark.parametrize("verified", [None, "ran", "refused"])
def test_a_stdout_row_on_any_step_but_the_last_is_refused(tmp_path, name, verified) -> None:
    """PQ-68 F2: only the last step resolves, and only the verifier writes its
    `stdout` row, so a row on an earlier step is an unobserved claim about the
    output. Refused on load and save, whatever the record says; never stripped."""
    record = {None: None, "ran": RAN, "refused": REFUSED}[verified]
    trace = Trace(
        steps=(
            TraceStep(lines=(1,), focus=(1, 1), note="first", values=(ValueDelta(name, "—", "1"),)),
            TraceStep(lines=(2,), focus=(1, 2), note="last", values=(ValueDelta("stdout", "—", "1"),)),
        )
    )
    q = _question(verified=record, trace=trace)
    with pytest.raises(BankError, match="trace step 1 names 'stdout'"):
        validate_question(q)
    with pytest.raises(BankError, match="trace step 1 names 'stdout'"):
        save_question(tmp_path, q)
    assert not (tmp_path / "questions").exists()


def _drafted_trace() -> Trace:
    """A trace as the generator drafts it: steps, and no `stdout` row, which only
    the verifier writes (T-16, option b)."""
    return Trace(
        steps=(
            TraceStep(lines=(1,), focus=(1, 1), note="first"),
            TraceStep(lines=(1,), focus=(1, 1), note="last", values=(ValueDelta("v", "—", "1"),)),
        )
    )


def test_a_drafted_trace_never_passes_as_finished_once_a_record_says_it_ran(tmp_path) -> None:
    """A candidate's drafted trace is storable while it is unverified. Given a
    record that ran by anything but `verify.with_record`, which writes the stdout
    row, it fails load and save - no placeholder value stands in for the output."""
    draft = _question(verified=None, trace=_drafted_trace())
    validate_question(draft)

    ran = dataclasses.replace(draft, verified=RAN)
    with pytest.raises(BankError, match="does not name 'stdout'"):
        question_from_dict(question_to_dict(ran))
    with pytest.raises(BankError, match="does not name 'stdout'"):
        save_question(tmp_path, ran)
    assert not (tmp_path / "questions").exists()


def test_quoted_outputs_reads_trace_notes() -> None:
    """AC-73 covers a drafted trace's notes: a model's note quoting an output is a
    claim the review gate must see (T-16, option b)."""
    steps = _drafted_trace().steps
    noted = Trace(steps=(steps[0], dataclasses.replace(steps[1], note='it prints "zebra" here')))
    assert "zebra" in quoted_outputs(_question(trace=noted))
    assert "zebra" not in quoted_outputs(_question(trace=_drafted_trace()))


def test_affirmation_needs_both_who_and_when() -> None:
    """AC-72: affirmation records who and when, so one without the other is not it."""
    assert Review().affirmed() is False
    assert Review(affirmed_by="michelle").affirmed() is False
    assert Review(affirmed_at="2026-08-11 22:51").affirmed() is False
    assert Review(affirmed_by="michelle", affirmed_at="2026-08-11 22:51").affirmed()


# --------------------------------------------------------------------------- #
# GAP-24: an affirmation is bound to the text it affirmed
# --------------------------------------------------------------------------- #

COMPLETE = dataclasses.replace(
    RAN, legacy=False, verified_at="2026-10-12T00:00:00Z", verifier_version="test"
)


def _affirmed(question: Question, digest: str | None = "current") -> Question:
    review = Review(
        status="accepted",
        affirmed_by="fixture-organizer",
        affirmed_at="2026-10-12T00:00:00+00:00",
        affirmed_digest=bank.affirmation_digest(question) if digest == "current" else digest,
    )
    return dataclasses.replace(question, review=review)


@pytest.mark.parametrize(
    "change",
    [
        lambda q: dataclasses.replace(q, explains=Explains(what="changed", takeaway=q.explains.takeaway)),
        lambda q: dataclasses.replace(q, explains=Explains(what=q.explains.what, takeaway="changed")),
        lambda q: dataclasses.replace(
            q, options=(dataclasses.replace(q.options[0], why_tempting="changed"), *q.options[1:])
        ),
        lambda q: dataclasses.replace(q, trace=_drafted_trace()),
        lambda q: dataclasses.replace(q, source="fn main() { }\n"),
        lambda q: dataclasses.replace(q, options=(dataclasses.replace(q.options[0], text="uno"), *q.options[1:])),
        lambda q: dataclasses.replace(q, hint="changed"),
    ],
    ids=["what", "takeaway", "why_tempting", "trace", "source", "option text", "hint"],
)
def test_gap24_any_change_to_the_affirmed_text_unbinds_it(change) -> None:
    affirmed = _affirmed(_question(verified=COMPLETE))
    assert bank.affirmation_state(affirmed) == "bound"
    assert bank.is_affirmed(affirmed)
    edited = change(affirmed)
    assert bank.affirmation_digest(edited) != affirmed.review.affirmed_digest
    assert bank.affirmation_state(edited) == "stale"
    assert not bank.is_affirmed(edited)


def test_gap24_the_digest_ignores_the_review_block_and_the_verified_record() -> None:
    q = _question(verified=COMPLETE)
    later = dataclasses.replace(q, verified=dataclasses.replace(COMPLETE, verified_at="2026-11-01T00:00:00Z"))
    assert bank.affirmation_digest(q) == bank.affirmation_digest(later)
    assert bank.affirmation_digest(q) == bank.affirmation_digest(_affirmed(q))
    assert bank.DIGEST_PATTERN.fullmatch(bank.affirmation_digest(q))


def test_gap24_a_digest_less_affirmation_is_never_affirmed() -> None:
    """RULING PQ-23 (client, 2026-10-12): an affirmation with no digest names who
    and when but not what, on any record. q3's hand affirmation (F-52), on a legacy
    record, is read the same as GAP-24's M12b on a record the verifier wrote."""
    for verified in (RAN, COMPLETE):
        unbound = _affirmed(_question(verified=verified), digest=None)
        assert bank.affirmation_state(unbound) == "unbound"
        assert not bank.is_affirmed(unbound)
    assert bank.affirmation_state(_question()) == "none"


def test_gap24_a_stale_record_still_loads_but_a_malformed_digest_does_not() -> None:
    stale = _affirmed(_question(verified=COMPLETE), digest="sha256:" + "0" * 64)
    assert bank.affirmation_state(question_from_dict(question_to_dict(stale))) == "stale"
    for bad in ("sha256:xyz", "md5:" + "0" * 32):
        with pytest.raises(BankError, match="affirmed_digest"):
            validate_question(_affirmed(_question(), digest=bad))
    orphan = _question(review=Review(status="accepted", affirmed_digest="sha256:" + "0" * 64))
    with pytest.raises(BankError, match="without affirmed_by"):
        validate_question(orphan)
    data = question_to_dict(_question(review=Review(status="accepted")))
    data["review"]["quotes_checked"] = "not a list"
    with pytest.raises(BankError, match="quotes_checked"):
        question_from_dict(data)


def test_ac73_a_quote_matches_only_a_whole_printed_line() -> None:
    assert bank.quote_matches_output("two", "two\n")
    assert bank.quote_matches_output("b", "a\nb\n")
    assert not bank.quote_matches_output("tw", "two\n")
    assert not bank.quote_matches_output("[1, 2]", "[1, 2, 3]\n")
    assert not bank.quote_matches_output("anything", None)


# --------------------------------------------------------------------------- #
# Serialization: absent is absent (G-2, D-16)
# --------------------------------------------------------------------------- #


def test_a_field_nobody_observed_has_no_key_at_all() -> None:
    """D-16, G-2. This is the whole proof that a legacy record back-fills nothing.

    A key holding `null` would be a slot: something a later writer could fill without
    anyone noticing the record had grown a fact nobody observed. An absent key has to
    be added deliberately.
    """
    written = question_to_dict(_question())["verified"]
    for absent in (
        "target_triple",
        "flags",
        "exit_code",
        "verified_at",
        "verifier_version",
    ):
        assert absent not in written, f"{absent} should be absent, not null"


def test_an_empty_output_and_a_zero_exit_code_are_kept() -> None:
    """The counterpart. A program that printed nothing is an observation, and so is
    exiting zero, so a filter on falsiness would erase two facts and shorten the
    receipt of a question whose answer is a panic."""
    panic = dataclasses.replace(RAN, stdout="", exit_code=0, legacy=False)
    written = question_to_dict(_question(verified=panic, options=_options(
        "one", "", "three", "four", "does not compile"
    )))["verified"]
    assert written["stdout"] == ""
    assert written["exit_code"] == 0


def test_false_flags_that_mean_absence_are_written_as_absence() -> None:
    """`pivot` is optional in SPEC 5.3 and `legacy` marks the exceptional record, so
    writing either as `false` on every step and every record would bury the case a
    reader is scanning for."""
    complete = dataclasses.replace(RAN, legacy=False)
    trace = Trace(
        steps=(
            TraceStep(lines=(1,), focus=(1, 1), note="ordinary"),
            TraceStep(
                lines=(2,),
                focus=(1, 2),
                note="resolving",
                values=(ValueDelta(name="stdout", was="—", now="two"),),
                pivot=True,
            ),
        )
    )
    written = question_to_dict(_question(verified=complete, trace=trace))
    assert "legacy" not in written["verified"]
    assert "pivot" not in written["trace"]["steps"][0]
    assert written["trace"]["steps"][1]["pivot"] is True


def test_a_record_round_trips_through_json() -> None:
    q = _question(
        trace=Trace(
            steps=(
                TraceStep(lines=(1,), focus=(1, 1), note="one"),
                TraceStep(
                    lines=(2,),
                    focus=(1, 2),
                    note="two",
                    values=(ValueDelta(name="stdout", was="—", now="two"),),
                    pivot=True,
                ),
            )
        ),
        review=Review(status="accepted", difficulty_judged=2, affirmed_by="m", affirmed_at="t"),
        used=(Used(meetup_date="2026-08-12", room_id="r", released_at="t", fit="fits"),),
    )
    assert question_from_dict(json.loads(json.dumps(question_to_dict(q)))) == q


def test_a_null_fit_from_the_room_is_an_absent_key_and_reads_back_as_none() -> None:
    """F-30: the room sends `fit: null` when no wall reported a verdict. Nothing
    observed it, so the record holds no `fit` key - absent is absent - and a missing
    key reads back as `None`, which is not *fits*."""
    data = question_to_dict(_question())
    data["used"] = {"meetup_date": "2026-10-14", "room_id": "r", "released_at": "t", "fit": None}
    q = question_from_dict(data)
    assert q.used_by().fit is None
    written = question_to_dict(q)
    # Written as a list, one record per club; the bare object read above is the
    # form the bank wrote before clubs, and reads as the default club's.
    assert written["used"] == [{"meetup_date": "2026-10-14", "room_id": "r", "released_at": "t", "club": "rustnyc"}]
    assert question_from_dict(json.loads(json.dumps(written))) == q
    # The published contract agrees: what was written has every required key and
    # nothing the schema does not name.
    schema = _schema()["$defs"]["Used"]
    assert set(schema["required"]) <= set(written["used"][0]) <= set(schema["properties"])
    assert "fit" not in schema["required"]


def test_an_unknown_field_is_an_error_rather_than_ignored() -> None:
    """A typo'd key would otherwise read as a field somebody forgot to fill, and a
    record silently missing its beats is the failure this project cannot have."""
    data = question_to_dict(_question())
    data["explanations"] = "a typo for explains"
    with pytest.raises(BankError, match="unknown field"):
        question_from_dict(data)

    nested = question_to_dict(_question())
    nested["verified"]["target"] = "aarch64-apple-darwin"
    with pytest.raises(BankError, match="unknown field"):
        question_from_dict(nested)


# --------------------------------------------------------------------------- #
# The store
# --------------------------------------------------------------------------- #


def test_a_record_is_validated_before_it_reaches_disk(tmp_path: pathlib.Path) -> None:
    with pytest.raises(BankError):
        save_question(tmp_path, _question(options=_options("a", "b", "c", "d")))
    assert not (tmp_path / "questions").exists()


def test_the_bank_is_append_only(tmp_path: pathlib.Path) -> None:
    """SPEC 3.3. Re-verification replaces a record whole, deliberately and with its
    own ticket (D-16); an append must not do it by accident."""
    append_question(tmp_path, _question())
    with pytest.raises(BankError, match="append-only"):
        append_question(tmp_path, _question())


def test_a_saved_record_loads_back_identically(tmp_path: pathlib.Path) -> None:
    q = _question()
    save_question(tmp_path, q)
    assert load_question(tmp_path, "qX") == q
    assert load_bank(tmp_path) == (q,)


def test_loading_a_bank_that_is_not_there_is_empty_not_an_error(
    tmp_path: pathlib.Path,
) -> None:
    assert load_bank(tmp_path / "nothing-here") == ()


def test_a_missing_question_says_which_one(tmp_path: pathlib.Path) -> None:
    with pytest.raises(BankError, match="q9"):
        load_question(tmp_path, "q9")


# --------------------------------------------------------------------------- #
# The history store (SPEC 3.3, AC-17)
# --------------------------------------------------------------------------- #


def test_the_history_persists_across_two_separate_working_directories(
    tmp_path: pathlib.Path,
) -> None:
    """AC-17's persistence half. T-17 fills these stores and proves the rest on this
    shape; what is fixed here is that the file is the whole state, so a second run
    somewhere else reads exactly what the first wrote."""
    first, second = tmp_path / "run-one", tmp_path / "run-two"
    history = History(
        exact_hashes={"q3": source_hash("fn main() {}\n")},
        ast_fingerprints={"q3": "fingerprint-written-by-T-17"},
        token_bigrams={"q3": ["fn main", "main ("]},
    )
    save_history(first, history)

    (second).mkdir(parents=True)
    (second / "history.json").write_bytes((first / "history.json").read_bytes())
    assert load_history(second) == history


def test_the_history_reports_its_size(tmp_path: pathlib.Path) -> None:
    """AC-17: size and growth are visible. This is size; the run report and the
    review surface subtract two readings for growth, and T-17 owns both."""
    empty = History()
    assert empty.sizes() == {
        "exact_hashes": 0,
        "ast_fingerprints": 0,
        "token_bigrams": 0,
    }
    assert empty.total() == 0

    filled = History(
        exact_hashes={"q3": "h", "q4": "h"},
        ast_fingerprints={"q3": "f"},
        token_bigrams={},
    )
    assert filled.sizes() == {
        "exact_hashes": 2,
        "ast_fingerprints": 1,
        "token_bigrams": 0,
    }
    assert filled.total() == 3


def test_a_missing_history_reads_as_empty(tmp_path: pathlib.Path) -> None:
    assert load_history(tmp_path) == History()


def test_a_history_from_a_future_version_is_refused(tmp_path: pathlib.Path) -> None:
    """A migration is a deliberate act. Reading an unknown layout as if it were this
    one would silently drop whatever the newer writer added."""
    (tmp_path / "history.json").write_text(json.dumps({"version": 99}), encoding="utf-8")
    with pytest.raises(BankError, match="version 99"):
        load_history(tmp_path)


def test_the_exact_hash_is_exact(tmp_path: pathlib.Path) -> None:
    """AC-14 rejects a byte-identical resubmission; AC-15 rejects a reformatted one.
    Those are different rejections, so this hash normalizes nothing - folding them
    together here would lose the distinction T-17 needs."""
    assert source_hash("fn main() {}\n") == source_hash("fn main() {}\n")
    assert source_hash("fn main() {}\n") != source_hash("fn main() { }\n")
    assert source_hash("fn main() {}\n") != source_hash("fn main() {}")


# --------------------------------------------------------------------------- #
# The published schema, against the types (SPEC 3.1-3.2)
# --------------------------------------------------------------------------- #

# Each dataclass and where it lives in the schema. `Question` is the root; the rest
# are $defs named after the class, which is what lets this walk be mechanical.
SCHEMA_TYPES: tuple[type, ...] = (
    Question,
    Option,
    Explains,
    Trace,
    TraceStep,
    ValueDelta,
    Verified,
    Runs,
    Flags,
    Miri,
    Review,
    Used,
)


def _schema() -> dict:
    return json.loads(SCHEMA.read_text(encoding="utf-8"))


def _subschema(schema: dict, cls: type) -> dict:
    return schema if cls is Question else schema["$defs"][cls.__name__]


def _required_of(cls: type) -> set[str]:
    """A field with no default is required; one with a default or a factory is not."""
    return {
        f.name
        for f in dataclasses.fields(cls)
        if f.default is dataclasses.MISSING
        and f.default_factory is dataclasses.MISSING  # type: ignore[misc]
    }


@pytest.mark.parametrize("cls", SCHEMA_TYPES, ids=[c.__name__ for c in SCHEMA_TYPES])
def test_the_schema_and_the_dataclasses_do_not_drift(cls: type) -> None:
    """The schema is the published contract; this walk is its enforcement.

    Both directions, because one direction catches only half a drift: a field added
    to a dataclass and left out of the schema publishes an incomplete contract, and a
    property left in the schema after its field is gone documents something the code
    will now refuse. There is no JSON Schema validator in the standard library and
    `pyproject.toml` belongs to another ticket, so this test is what keeps the two
    honest - and `bank/README.md` says that plainly rather than implying the schema
    validates records.
    """
    sub = _subschema(_schema(), cls)
    fields = {f.name for f in dataclasses.fields(cls)}
    properties = set(sub["properties"])

    assert fields - properties == set(), (
        f"{cls.__name__}: fields missing from the schema: {sorted(fields - properties)}"
    )
    assert properties - fields == set(), (
        f"{cls.__name__}: schema properties with no field: "
        f"{sorted(properties - fields)}"
    )
    assert set(sub.get("required", [])) == _required_of(cls), (
        f"{cls.__name__}: the schema and the dataclass disagree about which fields "
        "are required"
    )
    assert sub.get("additionalProperties") is False, (
        f"{cls.__name__}: the schema should refuse unknown properties, as "
        "question_from_dict does"
    )


def test_every_option_kind_the_types_allow_is_in_the_schema() -> None:
    """The one field whose values, not just its name, are part of the contract."""
    kinds = set(_schema()["$defs"]["Option"]["properties"]["kind"]["enum"])
    assert kinds == {"output", "does_not_compile", "ub", "panic"}
    assert set(_schema()["$defs"]["Used"]["properties"]["fit"]["enum"]) == {
        "fits",
        "clipped_x",
        "clipped_y",
        "clipped_xy",
    }
    assert set(_schema()["$defs"]["Review"]["properties"]["status"]["enum"]) == {
        "accepted",
        "rejected",
        "edited",
    }


def test_the_schema_covers_every_dataclass_the_record_can_hold() -> None:
    """So a type added later cannot quietly go undocumented: every frozen dataclass
    in `bank` that is part of a question is in the walk above."""
    in_module = {
        obj.__name__
        for obj in vars(bank).values()
        if dataclasses.is_dataclass(obj)
        and isinstance(obj, type)
        and obj.__module__ == bank.__name__
    }
    # History is the other store, with its own file and no place in a question.
    assert in_module - {"History"} == {c.__name__ for c in SCHEMA_TYPES}


def test_a_used_record_under_nyc_is_the_default_clubs() -> None:
    """ADR-0005 item 8: `nyc` is the default club's old name. A record written
    under it reads, and is written back, as `rustnyc`: a question used under
    `nyc` is never unused under `rustnyc`."""
    data = question_to_dict(_question())
    data["used"] = [{"meetup_date": "2026-10-14", "room_id": "r", "released_at": "t", "club": "nyc"}]
    q = question_from_dict(data)
    assert q.used_by("rustnyc").room_id == "r"
    assert q.used_by().room_id == "r" and q.used_by("nyc").room_id == "r"
    assert q.used_by("la") is None
    assert question_to_dict(q)["used"][0]["club"] == "rustnyc"
    assert Used(meetup_date="d", room_id="r", released_at="t", club="nyc").club == "rustnyc"


def test_nyc_and_rustnyc_on_one_record_are_one_club_twice() -> None:
    data = question_to_dict(_question())
    data["used"] = [
        {"meetup_date": "2026-10-14", "room_id": "a", "released_at": "t", "club": "nyc"},
        {"meetup_date": "2026-11-11", "room_id": "b", "released_at": "t", "club": "rustnyc"},
    ]
    with pytest.raises(BankError, match="one club appears twice"):
        question_from_dict(data)
