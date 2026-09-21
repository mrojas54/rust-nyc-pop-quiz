"""The MVP migration (BUILDPLAN T-14): AC-73, AC-87, G-2 and the house rule.

The committed records under `bank/questions/` are the deliverable, so most of these
read those files rather than a fresh in-memory build. A test that only exercised the
builder would pass while the files on disk were stale.

The rule these are all circling: nobody writes down what a program prints. For a
migration that is not a slogan - the August batch's outputs are sitting in a JSON
file, and the difference between copying them and retyping them is the difference
between a bank you can trust and a trivia deck.
"""

from __future__ import annotations

import dataclasses
import json
import pathlib

import pytest

from popquiz.bank import (
    Explains,
    correct_index,
    incorrect_indexes,
    load_question,
    normalized_output,
    options_needing_reauthoring,
    question_to_dict,
    quoted_outputs,
    receipt_class,
)
from popquiz import migrate_mvp
from popquiz.migrate_mvp import (
    MIGRATED,
    NOT_MIGRATED,
    MigrationError,
    build_question,
    migrate,
)

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
MVP = REPO / "mvp" / "2026-08-12"
BANK = REPO / "bank"
MIGRATION_SOURCE = HERE.parent / "src" / "popquiz" / "migrate_mvp.py"

BATCH = json.loads((MVP / "verified.json").read_text(encoding="utf-8"))
CONTENT = json.loads((MVP / "content.json").read_text(encoding="utf-8"))
MVP_BY_ID = {q["id"]: q for q in BATCH["questions"]}

# Fields SPEC 3.2 lists for a complete record that a legacy record must NOT have.
# D-16: absent, and never back-filled or inferred. Absence is the proof (G-2).
NEVER_BACK_FILLED = (
    "target_triple",
    "flags",
    "exit_code",
    "verified_at",
    "verifier_version",
)

# Tokens that only appear in a full `rustc -Vv`. The MVP recorded `rustc --version`,
# one line, so a legacy record's compiler string must carry none of them.
VV_ONLY = ("host:", "release:", "commit-hash:", "commit-date:", "LLVM version:")


@pytest.fixture(scope="module")
def committed() -> dict[str, object]:
    return {qid: load_question(BANK, qid) for qid in MIGRATED}


# --------------------------------------------------------------------------- #
# The committed output is what the code produces
# --------------------------------------------------------------------------- #


def test_the_committed_records_are_what_a_fresh_run_writes(
    tmp_path: pathlib.Path,
) -> None:
    """Re-running the migration reproduces the files byte for byte.

    Which makes the committed records reviewable: a reader can regenerate them rather
    than take them on trust, and a later edit to the authored beats cannot land in the
    module without the file beside it changing too.
    """
    migrate(MVP, tmp_path)
    for qid in MIGRATED:
        fresh = (tmp_path / "questions" / f"{qid}.json").read_text(encoding="utf-8")
        landed = (BANK / "questions" / f"{qid}.json").read_text(encoding="utf-8")
        assert fresh == landed, f"{qid}.json on disk is not what the migration writes"
    assert (tmp_path / "history.json").read_text(encoding="utf-8") == (
        BANK / "history.json"
    ).read_text(encoding="utf-8")


def test_only_the_four_questions_that_fit_the_wall_were_migrated() -> None:
    """SPEC 5.2's fit table, D-15. q1, q2, q5 and q6 exceed the reading layout's
    7-line capacity at the guessed room, so the wall cannot show them legibly at
    all; they stay in the MVP bank until re-authored or until the rehearsal's
    measurements raise the floor."""
    assert set(MIGRATED) == {"q3", "q4", "q7", "q8"}
    on_disk = {p.stem for p in (BANK / "questions").glob("*.json")}
    assert on_disk == set(MIGRATED)
    assert set(NOT_MIGRATED) == {"q1", "q2", "q5", "q6"}
    assert set(MIGRATED) & set(NOT_MIGRATED) == set()
    # All eight of the batch are accounted for, so none was silently forgotten.
    assert set(MIGRATED) | set(NOT_MIGRATED) == set(MVP_BY_ID)


# --------------------------------------------------------------------------- #
# Legacy records: absence is the proof (D-16, G-2, AC-87)
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize("qid", MIGRATED)
def test_no_legacy_record_holds_a_field_the_mvp_never_recorded(
    qid: str, committed: dict
) -> None:
    """D-16, G-2, AC-87. The absent keys are the whole proof that nothing was
    back-filled, which is why this asserts on the written JSON and not on the
    dataclass - a `None` attribute and an absent key look the same in Python and are
    completely different on disk."""
    written = question_to_dict(committed[qid])["verified"]
    assert written["legacy"] is True
    for absent in NEVER_BACK_FILLED:
        assert absent not in written, f"{qid}: {absent} was back-filled"


@pytest.mark.parametrize("qid", MIGRATED)
def test_a_legacy_compiler_string_is_the_version_line_not_a_vv(
    qid: str, committed: dict
) -> None:
    """AC-6, D-16: `verify.py` recorded `rustc --version` and pinned nothing. A `-Vv`
    reconstructed from it would be a hand-written fact about a toolchain nobody
    interrogated."""
    rustc = committed[qid].verified.rustc
    assert rustc == BATCH["rustc"]
    assert "\n" not in rustc
    for token in VV_ONLY:
        assert token not in rustc, f"{qid}: compiler string looks like a -Vv"


def test_q8_carries_its_error_codes_renamed_and_nothing_that_ran() -> None:
    """T-14, D-22: `error_codes` becomes `compile_error_code`, values unchanged.

    The MVP's `miri` field for q8 reads "n/a (does not compile)", which is not a
    result, so no Miri record is written rather than one saying it was fine.
    """
    q8 = load_question(BANK, "q8")
    assert q8.verified.compile_error_code == tuple(MVP_BY_ID["q8"]["error_codes"])
    assert q8.verified.runs is None
    assert q8.verified.stdout is None
    assert q8.verified.miri is None
    assert receipt_class(q8.verified) == "does_not_compile"

    written = question_to_dict(q8)["verified"]
    for absent in ("runs", "stdout", "miri", "byte_identical", "error_codes"):
        assert absent not in written


@pytest.mark.parametrize("qid", [q for q in MIGRATED if q != "q8"])
def test_a_compiling_question_carries_the_runs_and_miri_the_mvp_recorded(
    qid: str, committed: dict
) -> None:
    """D-16: `runs` as a count plus byte-identical, and a Miri result with no
    version, no borrow models and no seeds - the August pass ran outside the repo and
    recorded none of them."""
    mvp = MVP_BY_ID[qid]
    verified = committed[qid].verified

    assert verified.runs is not None
    assert verified.runs.count == mvp["runs"]
    assert verified.runs.byte_identical == mvp["byte_identical"]
    assert verified.stdout == mvp["answer"]
    assert verified.miri is not None
    assert verified.miri.clean is (mvp["miri"] == "clean")
    assert verified.miri.output_matched == mvp["miri_output_matches"]
    assert verified.miri.version is None
    assert verified.miri.configs is None
    assert verified.miri.seeds is None


def test_an_unrecognised_miri_result_is_refused_rather_than_guessed() -> None:
    """Translating free text into a boolean is the migration deciding what a pass
    found. `"clean"` is the one string it knows; anything else stops the run."""
    odd = dict(MVP_BY_ID["q3"], miri="mostly clean?")
    with pytest.raises(MigrationError, match="only knows how to read 'clean'"):
        build_question(BATCH, odd, CONTENT["q3"])


def test_a_non_compiling_question_with_no_error_codes_is_refused() -> None:
    """The does-not-compile receipt needs the codes (SPEC 7.5, D-22), so a record
    that cannot supply them must not be written as if it could."""
    no_codes = {k: v for k, v in MVP_BY_ID["q8"].items() if k != "error_codes"}
    with pytest.raises(MigrationError, match="records no error codes"):
        build_question(BATCH, no_codes, CONTENT["q8"])


# --------------------------------------------------------------------------- #
# The correct answer is derived, never typed (AC-7, G-2, PHILOSOPHY 3)
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize("qid", MIGRATED)
def test_the_correct_option_is_the_machines_output(qid: str, committed: dict) -> None:
    """PHILOSOPHY 3: the machine decides the answer. The correct option's text is
    `normalized_output` of the recorded `stdout` and never an authored string, so the
    derivation and the record cannot disagree."""
    question = committed[qid]
    index = correct_index(question)
    assert index is not None, f"{qid}: no option matches the verified answer"

    if qid == "q8":
        assert question.options[index].kind == "does_not_compile"
    else:
        assert question.options[index].text == normalized_output(
            MVP_BY_ID[qid]["answer"]
        )


def test_the_resolving_trace_step_is_derived_from_the_record() -> None:
    """D-10, G-2. `prototypes/_shared/data.js:127` wrote this value by hand; here it
    comes from the `verified` record, so the step that resolves the question cannot
    say something the program did not print."""
    q3 = load_question(BANK, "q3")
    last = q3.trace.resolving_step()
    assert last is not None
    stdout_values = [v for v in last.values if v.name == "stdout"]
    assert len(stdout_values) == 1
    assert stdout_values[0].now == normalized_output(MVP_BY_ID["q3"]["answer"])
    assert q3.trace.names_stdout_last()


def test_the_last_authored_value_agrees_with_what_the_program_printed() -> None:
    """q3's trace ends by showing `v` and then printing it, so the last authored `v`
    value and the recorded output must be the same string.

    They are equal because this program prints `v` with `{:?}`, which is a fact about
    q3 and not a general property - so it is asserted rather than assumed. If someone
    edits the authored trace and breaks the agreement, this fires and a person looks
    at it, which is the whole point of pinning a coincidence.
    """
    q3 = load_question(BANK, "q3")
    v_values = [v for step in q3.trace.steps for v in step.values if v.name == "v"]
    assert v_values, "q3's trace no longer follows `v`"
    assert v_values[-1].now == normalized_output(MVP_BY_ID["q3"]["answer"])


def test_the_migration_does_not_write_down_what_a_program_printed() -> None:
    """CLAUDE.md's first house rule, mechanized against this module's own source.

    **What this proves.** No recorded `stdout` from the batch appears as a literal in
    `migrate_mvp.py`, so the module cannot be the place an output was typed. Every
    output reaches a record by being read out of the machine-written
    `mvp/2026-08-12/verified.json`.

    **What it does not prove**, and it is worth being exact, because the same care the
    receipt takes applies to a test claiming to enforce a house rule: it checks the
    exact recorded strings, so an output assembled from pieces, or written with
    different whitespace, would pass. It also says nothing about the authored trace's
    intermediate values, which are program *state* a human wrote and an organizer
    affirms - the test above pins the one that coincides with the output, and AC-73's
    quoted-output check at review is what stands behind the rest.

    q8's answer is excluded: it is "does not compile", a verdict rather than anything
    a program printed, and the module names it as a constant on purpose.
    """
    source = MIGRATION_SOURCE.read_text(encoding="utf-8")
    for qid, question in MVP_BY_ID.items():
        if not question["compiled"]:
            continue
        answer = question["answer"]
        assert answer not in source, f"{qid}: its recorded output is typed in the module"
        # Also the JSON-escaped form, which is how it would look if somebody pasted it
        # out of verified.json rather than out of the terminal.
        assert json.dumps(answer)[1:-1] not in source, f"{qid}: escaped output is typed"


def test_that_check_would_catch_a_typed_output() -> None:
    """The guard above is only worth having if it fires."""
    source = MIGRATION_SOURCE.read_text(encoding="utf-8")
    smuggled = source + '\nTYPED = "' + json.dumps(MVP_BY_ID["q3"]["answer"])[1:-1] + '"\n'
    assert json.dumps(MVP_BY_ID["q3"]["answer"])[1:-1] in smuggled


# --------------------------------------------------------------------------- #
# The beats (AC-73, AC-95, D-9)
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize("qid", MIGRATED)
def test_the_mvp_explanation_is_kept_verbatim(qid: str, committed: dict) -> None:
    """AC-73, SPEC 3.1: `explains.legacy` is the string the organizer reviewed in
    August, byte for byte, because that is the text AC-73's quoted-output check runs
    against on an inherited question. Taken from `content.json`, the MVP's own file -
    `prototypes/_shared/data.js` holds a copy with its punctuation restyled for the
    prototype, and a restyled copy is not verbatim."""
    assert committed[qid].explains.legacy == CONTENT[qid]["explanation"]


@pytest.mark.parametrize("qid", MIGRATED)
def test_the_beats_are_drafted_and_the_legacy_string_is_not_folded_into_them(
    qid: str, committed: dict
) -> None:
    """SPEC 3.1: `explains` replaces the single string; the string is kept beside it."""
    explains = committed[qid].explains
    assert explains.what and explains.takeaway
    assert explains.what != explains.legacy
    assert explains.takeaway != explains.legacy


def test_q3_has_a_middle_beat_for_every_incorrect_option() -> None:
    """AC-95, D-9: a beat per incorrect option, so the reveal has one whichever option
    the room picks. q3 is the only migrated question whose options are final, so it is
    the only one that can have a complete set."""
    q3 = load_question(BANK, "q3")
    correct = correct_index(q3)
    for i, option in enumerate(q3.options):
        if i == correct:
            assert option.why_tempting is None, "the correct option has no tempting"
        else:
            assert option.why_tempting, f"q3 option {i} has no why_tempting"


def test_a_beat_written_for_an_option_that_does_not_exist_stops_the_migration(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The negative half of carrying `why_tempting` on the option.

    The whole case for that shape over a letter or an index is that a beat cannot end
    up attached to the wrong option - so the migration must refuse a beat whose key
    matches no option at all, rather than dropping it and leaving an option silently
    without its middle beat. A dropped beat would pass every positive test above: each
    of them checks that the beats present are attached correctly, and none would
    notice one that quietly went missing.

    Exercised directly rather than only through the real data, where a typo would fail
    the reproducibility test without saying why.
    """
    beats = dict(migrate_mvp.AUTHORED_WHY_TEMPTING)
    beats["q3"] = dict(beats["q3"], **{"[1, 2, 3, 4, 5]": "an option q3 does not have"})
    monkeypatch.setattr(migrate_mvp, "AUTHORED_WHY_TEMPTING", beats)

    with pytest.raises(MigrationError, match="options that do not exist"):
        build_question(BATCH, MVP_BY_ID["q3"], CONTENT["q3"])


def test_every_beat_resolves_to_exactly_one_option(committed: dict) -> None:
    """The positive half, stated as the property rather than per question.

    A beat belongs to one option and the correct option never carries one, so across
    the bank the beats and the incorrect options line up one to one - which is what
    makes AC-95's "every incorrect option has a non-empty why_tempting" a structural
    check at affirm rather than a join T-18 has to get right.
    """
    for qid, question in committed.items():
        carried = [i for i, o in enumerate(question.options) if o.why_tempting]
        assert len(carried) == len(set(carried)), f"{qid}: a beat counted twice"
        assert correct_index(question) not in carried, (
            f"{qid}: the correct option carries a why_tempting"
        )
        for i in carried:
            assert i in incorrect_indexes(question), (
                f"{qid}: option {i} carries a beat but is not an incorrect option"
            )


@pytest.mark.parametrize("qid", ["q4", "q7", "q8"])
def test_the_questions_awaiting_reauthoring_have_no_middle_beats(
    qid: str, committed: dict
) -> None:
    """Every incorrect option they would attach to is scheduled for replacement, so a
    beat drafted now would be drafted against text that will not exist. The absence
    blocks affirm (SPEC 7.4, AC-95), which is the correct state."""
    assert all(o.why_tempting is None for o in committed[qid].options)


def test_no_middle_beat_is_keyed_by_a_letter() -> None:
    """The prototype keyed one beat to the letter "A" (`data.js:65`). Letters are drawn
    from the meetup date (AC-23), so a letter-keyed beat points at a different option
    every month. Carried on the option, there is no key to go stale."""
    written = json.loads((BANK / "questions" / "q3.json").read_text(encoding="utf-8"))
    assert "why_tempting" not in written["explains"]
    assert "option" not in written["explains"]
    assert any("why_tempting" in o for o in written["options"])


def test_the_prototypes_stage_two_field_names_did_not_survive() -> None:
    """SPEC 3.1: field names are English words, not the prototype's `argue`."""
    for qid in MIGRATED:
        written = json.loads(
            (BANK / "questions" / f"{qid}.json").read_text(encoding="utf-8")
        )
        assert "argue" not in written["explains"]
        assert "whyWrong" not in written["explains"]
        assert "takeaway" in written["explains"]


# --------------------------------------------------------------------------- #
# Review state: nothing migrated is ready for a room (SPEC 7.4, G-12)
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize("qid", MIGRATED)
def test_nothing_migrated_is_affirmed(qid: str, committed: dict) -> None:
    """G-12, AC-72: affirmation is a human act. A migration drafts; it never affirms,
    so none of these can reach a deck until an organizer has read it."""
    review = committed[qid].review
    assert review is not None
    assert review.status is None
    assert review.affirmed() is False
    assert review.affirmed_by is None
    assert review.affirmed_at is None


def test_q3_runs_on_the_wall_as_authored() -> None:
    """D-15, SPEC 5.2: every option one line of at most 29 characters. The one
    question of the eight that passes unchanged."""
    q3 = load_question(BANK, "q3")
    assert options_needing_reauthoring(q3.options) == ()


@pytest.mark.parametrize(
    ("qid", "expected_failures"), [("q4", 4), ("q7", 4), ("q8", 1)]
)
def test_the_review_note_names_the_options_that_break_the_wall_rule(
    qid: str, expected_failures: int, committed: dict
) -> None:
    """The note is generated from the measurement, not typed, so it and `bank-audit`
    cannot disagree about which options fail (D-15)."""
    question = committed[qid]
    failing = options_needing_reauthoring(question.options)
    assert len(failing) == expected_failures

    reason = question.review.reason
    assert reason is not None
    assert f"{expected_failures} of 5" in reason
    assert "29 characters" in reason
    for index in failing:
        assert str(index) in reason


def test_q8_needs_only_a_distractor_replaced_and_keeps_its_answer() -> None:
    """The distinction SPEC 5.2's table compresses. q4's and q7's failing options
    include the correct one, so their programs have to print something shorter and the
    machine decides a new answer. q8's failing option is a distractor, so its answer
    stands and its program need not change."""
    q8 = load_question(BANK, "q8")
    failing = options_needing_reauthoring(q8.options)
    assert correct_index(q8) not in failing
    assert "The correct option is not among them" in q8.review.reason

    for qid in ("q4", "q7"):
        question = load_question(BANK, qid)
        assert correct_index(question) in options_needing_reauthoring(question.options)
        assert "The correct option is among them" in question.review.reason


# --------------------------------------------------------------------------- #
# AC-73's quoted-output helper
# --------------------------------------------------------------------------- #


def _with_beats(what: str = "", takeaway: str = "", legacy: str | None = None) -> object:
    """q3 with its beats replaced, so a candidate span can be put where AC-73 looks."""
    q3 = load_question(BANK, "q3")
    return dataclasses.replace(
        q3, explains=Explains(what=what, takeaway=takeaway, legacy=legacy)
    )


def test_a_beat_quoting_the_verified_output_is_offered_to_the_check() -> None:
    """AC-73. The helper lists candidates; T-18 compares them to `verified.stdout` and
    blocks acceptance on a mismatch. If the span a beat quotes is not in the list, the
    gate has nothing to check and the criterion is unproven."""
    answer = normalized_output(MVP_BY_ID["q3"]["answer"])
    question = _with_beats(what=f"It prints {answer} and stops there.")
    assert answer in quoted_outputs(question)


def test_a_beat_quoting_the_WRONG_output_is_offered_too() -> None:
    """AC-73's actual fixture: a wrong quote must reach the gate to be blocked.

    The failure mode this guards is a helper that only recognises spans equal to the
    verified output - it would find nothing to report on exactly the question AC-73
    exists to reject.
    """
    wrong = "[1, 2, 3]"
    question = _with_beats(what=f"It prints {wrong}, which is the trap.")
    candidates = quoted_outputs(question)
    assert wrong in candidates
    assert wrong != normalized_output(MVP_BY_ID["q3"]["answer"])


def test_the_legacy_string_is_searched_too() -> None:
    """AC-73 on an inherited question runs against `explains.legacy` - the text the
    organizer actually reviewed (SPEC 3.1), so a helper that skipped it would leave
    every migrated question unchecked."""
    answer = normalized_output(MVP_BY_ID["q3"]["answer"])
    question = _with_beats(legacy=f"The August note said it prints {answer}.")
    assert answer in quoted_outputs(question)


def test_a_middle_beat_is_searched_too() -> None:
    """A `why_tempting` beat is the one most likely to quote an output - it exists to
    say why a reader would have expected a different one (D-9)."""
    q3 = load_question(BANK, "q3")
    incorrect = next(i for i, o in enumerate(q3.options) if o.why_tempting)
    options = list(q3.options)
    options[incorrect] = dataclasses.replace(
        options[incorrect], why_tempting="You would expect [9, 9, 9] here."
    )
    assert "[9, 9, 9]" in quoted_outputs(dataclasses.replace(q3, options=tuple(options)))


def test_the_hint_is_not_searched() -> None:
    """The hint is written to be read before anyone has seen an answer, so an output
    quoted in it is a leak for the canary suite (G-3), not a mismatch for AC-73.
    Searching it would report a finding to the wrong gate."""
    answer = normalized_output(MVP_BY_ID["q3"]["answer"])
    q3 = load_question(BANK, "q3")
    leaky = dataclasses.replace(q3, hint=f"It prints {answer}.")
    assert answer not in quoted_outputs(leaky)


def test_q3_as_migrated_quotes_no_output_at_all() -> None:
    """Worth pinning, because it is easy to assume otherwise.

    q3's three beats and its August explanation discuss `dedup` in prose and never
    quote what the program printed - the answer appears in q3's *options* and in its
    trace, which are checked by the derivation tests above, not by AC-73. So AC-73 has
    nothing to compare on this question, and that is a pass rather than a gap.

    If a later edit to the beats introduces a quoted span, this fires. That is the
    intent: the author should then see that AC-73 has started applying to q3.
    """
    assert quoted_outputs(load_question(BANK, "q3")) == ()


def test_no_span_is_filtered_out_by_its_shape() -> None:
    """The deliberate absence of a shape filter, pinned with the cases that force it.

    Read against the eight August explanations, the two obvious filters each fail in a
    different direction, so neither is applied:

    * `created` is what q2's program printed, and is also a valid Rust identifier -
      dropping identifier-shaped spans would drop real output.
    * `[0]` comes from `&v[0]` in q8's explanation - it is bracketed and is not
      output, so keeping bracketed spans admits a false positive.
    * `end of main` is what q1's program printed, with no bracket and no digit.

    AC-73 is a blocking gate, so a false positive costs an organizer a question and a
    false negative defeats the check. Both go to T-18, which has the verified output
    and a reviewer looking at it.
    """
    question = _with_beats(
        what='It printed "created" and then "end of main".',
        takeaway="The borrow was `&v[0]`, and `sort_unstable_by_key` is the other one.",
    )
    candidates = quoted_outputs(question)
    for span in ("created", "end of main", "[0]", "sort_unstable_by_key"):
        assert span in candidates, f"{span!r} was filtered out by its shape"


def test_a_span_is_reported_once_and_an_empty_one_not_at_all() -> None:
    """So a reviewer reads a list of distinct spans rather than a concordance."""
    question = _with_beats(
        what='It printed "created", and "created" again, and "".',
        takeaway='Then "created".',
    )
    assert quoted_outputs(question) == ("created",)
