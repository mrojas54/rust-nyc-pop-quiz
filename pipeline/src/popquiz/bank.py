"""The bank's record format, and the facts derived from it (SPEC.md 3.1-3.3).

One JSON file per question under `bank/questions/<id>.json`, plus `bank/history.json`
for the dedupe stores. No server and no database: a bank record is a file you can
read in a pull request (BUILDPLAN D-C).

Four things about this shape are decisions rather than defaults, each because the
obvious alternative breaks a rule this project does not bend.

**There is no `correct` field.** SPEC 3.1 lists `correct` and marks it *derived*
(G-2), so this module gives you `correct_index()` and no place to write an answer
down. A stored correct-answer field is a field somebody can hand-edit, and that is
the one thing AC-7 exists to catch. The derivation reads `verified`, which only the
verifier writes.

**Bank order is canonical and means nothing.** The wall's option order and its
letters are drawn from the meetup date and nothing else (AC-23, `slot_for_day` in
`mvp/tools/build_deck.py`), so the order options sit in on disk is not the order a
room sees. No later ticket may read a position tell into it (AC-26).

**Absent is absent.** A field the verifier did not observe has no key at all - not a
key holding `null`. `to_dict` drops `None`, and the tests assert the absence,
because for a legacy record (SPEC 3.2, D-16) absence is the entire proof that
nothing was back-filled. `stdout=""` and `exit_code=0` are observations, not
absences, and are kept.

**`why_tempting` rides on the option it describes.** SPEC 3.1 files it under
`explains`; this module carries it on `Option` instead. That is a deliberate
deviation, recorded in `bank/README.md`: a beat keyed by letter dangles the moment
the date draws a different slot, and a beat keyed by position silently attaches to
the wrong option the first time somebody reorders them. Carried on the option there
is no key to go stale, and AC-95's "every incorrect option has a non-empty
`why_tempting`" becomes a structural property rather than a join that can miss.
"""

from __future__ import annotations

import dataclasses
import hashlib
import json
import re
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Literal

# SPEC 3.1: each option's kind. Exactly one option per question is
# `does_not_compile`, so its presence signals nothing (AC-24).
OptionKind = Literal["output", "does_not_compile", "ub", "panic"]

# SPEC 3.1: the review surface's verdict. `None` means nobody has judged it yet,
# which is not the same as rejected.
ReviewStatus = Literal["accepted", "rejected", "edited"]

# SPEC 3.4: the wall's measured verdict, copied into `used` at release (AC-100).
Fit = Literal["fits", "clipped_x", "clipped_y", "clipped_xy"]

OPTION_COUNT = 5

# SPEC 5.2, D-15: one line, at most 29 characters, at the wall's 24 px option size.
# `bank-audit` (T-19) is what fails a bank on this; the constants live here so the
# audit and the migration's review notes cannot disagree about the number.
MAX_OPTION_LINE_CHARS = 29
MAX_OPTION_LINES = 1


class BankError(Exception):
    """A record is malformed, or a derivation was asked for something it cannot give."""


# --------------------------------------------------------------------------- #
# SPEC 3.1 - the question
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class Option:
    """One of the five options (SPEC 3.1).

    `why_tempting` is the middle beat for this option - why a reader would pick it
    (D-9, AC-95). It is `None` on the correct option, which has no tempting to
    explain, and `None` while a question is still awaiting its beats. Affirm is
    blocked until every incorrect option has one (SPEC 7.4).
    """

    text: str
    kind: OptionKind
    why_tempting: str | None = None

    def line_count(self) -> int:
        return self.text.count("\n") + 1

    def widest_line(self) -> int:
        return max(len(line) for line in self.text.split("\n"))

    def fits_the_wall(self) -> bool:
        """Whether this option satisfies D-15's one-line, 29-character rule.

        A convenience for the migration's review notes and for T-19. It is not the
        audit: `bank-audit` fails a *bank*, and the fit of the source program is a
        separate measurement this says nothing about (SPEC 5.2).
        """
        return (
            self.line_count() <= MAX_OPTION_LINES
            and self.widest_line() <= MAX_OPTION_LINE_CHARS
        )


@dataclass(frozen=True)
class ValueDelta:
    """One row of a trace step's value table (SPEC 5.3). `was`/`now` are verbatim
    strings; the spec's absent marker is an em dash, written by the author."""

    name: str
    was: str
    now: str


@dataclass(frozen=True)
class TraceStep:
    """One step of the walk-through (SPEC 5.3).

    `lines` are 1-based source lines drawn as executing now; `focus` is the
    inclusive 1-based range kept at full contrast; `note` is what the host reads;
    `values` is the table beneath the source; `pivot` marks the one step to pause
    on. `pivot` defaults to False and serializes only when true, because SPEC 5.3
    makes it optional and an absent marker should stay absent.
    """

    lines: tuple[int, ...]
    focus: tuple[int, int]
    note: str
    values: tuple[ValueDelta, ...] = ()
    pivot: bool = False


@dataclass(frozen=True)
class Trace:
    """The steps, in order (SPEC 3.1, 5.3).

    May be empty. A question with fewer than two steps cannot be affirmed (SPEC
    7.4), so an empty trace is the honest state for a question whose walk-through
    has not been written - not a defect to paper over with a stub step.
    """

    steps: tuple[TraceStep, ...] = ()

    def resolving_step(self) -> TraceStep | None:
        """The last step, which for a question that ran is the one whose `values`
        names `stdout` (D-10). `None` for an empty trace."""
        return self.steps[-1] if self.steps else None

    def names_stdout_last(self) -> bool:
        """Whether the last step names `stdout` in its values.

        SPEC 3.1 and D-10 require this of a question that ran. It is *not* required
        of a does-not-compile question, which has no `stdout` for a step to name
        (SPEC 3.2) - see `validate_question`, and the defect reported with this
        ticket about what that means for the affirm gate.
        """
        last = self.resolving_step()
        return last is not None and any(v.name == "stdout" for v in last.values)


@dataclass(frozen=True)
class Explains:
    """The three beats (SPEC 3.1, D-9).

    `what` is what happened; `takeaway` is what to remember. The middle beat is not
    here - it rides on each `Option` as `why_tempting`, for the reason in this
    module's docstring.

    `legacy` is the MVP's single `explanation` string, kept verbatim so AC-73's
    quoted-output check runs against the text the organizer actually reviewed in
    August (SPEC 3.1). It is never rewritten and never merged into the beats.
    """

    what: str
    takeaway: str
    legacy: str | None = None


@dataclass(frozen=True)
class Review:
    """The review surface's record (SPEC 3.1, 7.4).

    `status` unset means nobody has judged this question yet, which is not
    rejection. `reason` is required when rejecting and is readable by the
    generator's next run; the MVP migration also uses it to say what a question
    needs before it can be affirmed, which is exactly the audience that field has.
    """

    status: ReviewStatus | None = None
    reason: str | None = None
    difficulty_judged: int | None = None
    affirmed_by: str | None = None
    affirmed_at: str | None = None
    near_duplicate_of: str | None = None

    def affirmed(self) -> bool:
        """Whether an organizer has affirmed this question (SPEC 7.4, G-12).

        Both halves are required: affirmation records who and when (AC-72), so a
        record holding one without the other is not affirmed.
        """
        return bool(self.affirmed_by) and bool(self.affirmed_at)


@dataclass(frozen=True)
class Used:
    """Written once, by the release transition and nothing else (SPEC 3.1, G-10).

    Not by a deck build. `build_deck.py` wrote a ledger line at *build* time, which
    retired questions for meetups that then did not happen; that is a defect the
    brownfield table retires (BUILDPLAN 2, G-10).
    """

    meetup_date: str
    room_id: str
    released_at: str
    fit: Fit


# --------------------------------------------------------------------------- #
# SPEC 3.2 - the verified record
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class Runs:
    """The native runs (SPEC 3.2): how many, and whether all were byte-identical."""

    count: int
    byte_identical: bool


@dataclass(frozen=True)
class Flags:
    """The compiler flags the research found unpinned (SPEC 3.2, AC-6)."""

    opt_level: str
    overflow_checks: bool
    debug_assertions: bool


@dataclass(frozen=True)
class Miri:
    """What Miri reported (SPEC 3.2).

    `clean` false is not a failure: it is how undefined behaviour is recorded when
    UB is the declared answer (AC-9). `output_matched` is whether Miri's output
    matched the native output (AC-10).

    `version`, `configs` and `seeds` are absent on a legacy record, whose Miri pass
    ran outside the verifier and recorded none of them (D-16). Miri proves absence
    of UB *on the paths actually executed* and nothing wider (AC-43, PHILOSOPHY 4).
    """

    clean: bool
    output_matched: bool
    version: str | None = None
    configs: tuple[str, ...] | None = None
    seeds: tuple[int, ...] | None = None


@dataclass(frozen=True)
class Verified:
    """Written by `verify` and never edited (SPEC 3.2).

    Three shapes are legitimate and each renders exactly one receipt (SPEC 7.5):

    * **complete, and it ran** - every field below present.
    * **complete, does not compile** - `compile_error_code` present; no `runs`, no
      `stdout`, no `exit_code`, no `miri`, because nothing ran (AC-11, D-22).
    * **legacy** (`legacy=True`) - what the August MVP batch recorded and no more:
      `rustc` as the `--version` string, `edition`, `runs`, `stdout`, and
      `miri.clean` / `miri.output_matched`. `target_triple`, `flags`, `exit_code`,
      `verified_at`, `verifier_version` and the full `-Vv` are **absent, and never
      back-filled or inferred** (D-16, G-2). A legacy does-not-compile record (q8)
      carries `rustc`, `edition` and `compile_error_code` alone.

    A legacy record renders the same step list as a complete one (D-25), says on
    take-it-home what it lacks (SPEC 13, AC-87), is exempt from the stale-pin check
    (SPEC 7.2), and is replaced whole - never merged - when T-15b re-verifies its
    question, so the bank never holds both.
    """

    rustc: str
    edition: str
    legacy: bool = False
    target_triple: str | None = None
    flags: Flags | None = None
    runs: Runs | None = None
    stdout: str | None = None
    exit_code: int | None = None
    miri: Miri | None = None
    compile_error_code: tuple[str, ...] | None = None
    verified_at: str | None = None
    verifier_version: str | None = None


@dataclass(frozen=True)
class Question:
    """One bank record (SPEC 3.1).

    `verified` is `None` for a candidate that has not been through the verifier.
    There is no `correct` field: see `correct_index`.
    """

    id: str
    source: str
    topic: str
    difficulty_requested: int
    options: tuple[Option, ...]
    hint: str
    explains: Explains
    trace: Trace = field(default_factory=Trace)
    verified: Verified | None = None
    review: Review | None = None
    used: Used | None = None


# --------------------------------------------------------------------------- #
# Derivations - the reads that are computed, never stored
# --------------------------------------------------------------------------- #

ReceiptClass = Literal["ran", "does_not_compile"]


def receipt_class(verified: Verified | None) -> ReceiptClass | None:
    """Which of SPEC 7.5's two lists this record renders, or `None` for neither.

    Precedence is the spec's: does-not-compile first, otherwise the list for a
    record that ran. So every record renders exactly one list and there is no
    third. A record that is none of complete, `legacy` or does-not-compile renders
    no receipt and cannot be scheduled (SPEC 7.5).
    """
    if verified is None:
        return None
    if verified.compile_error_code:
        return "does_not_compile"
    if verified.runs is not None:
        return "ran"
    return None


def normalized_output(stdout: str) -> str:
    """`stdout` as an option's text spells it.

    SPEC 3.1 says the correct option is the one whose text "equals the verified
    output", but every recorded `stdout` ends in the newline `println!` wrote and no
    option text carries one - `mvp/tools/build_deck.py:266` strips it when building
    a deck, and the loved prototype's options show it stripped. Comparing raw would
    make `correct_index` return `None` for every output question in the bank.

    So the comparison normalizes exactly one trailing newline and nothing else. Not
    whitespace, not case, not interior newlines: a question whose answer turns on
    trailing whitespace is a question this normalization would get wrong, and
    narrowing it to the one character `println!` adds is what keeps that honest.
    """
    return stdout[:-1] if stdout.endswith("\n") else stdout


def correct_index(question: Question) -> int | None:
    """Which option is correct, derived from `verified` (SPEC 3.1, G-2).

    There is no stored answer to disagree with. For a record that ran, the correct
    option is the one whose text equals the verified output; for a does-not-compile
    record it is the single `does_not_compile` option. `None` when the question has
    no verified record yet, or when nothing matches - and *nothing matching is a
    finding, not a default*: it means the options and the machine's output have
    drifted apart, which is what AC-7's provenance check is looking for.

    Raises `BankError` when two options match, because then "the option whose text
    equals the output" names no single option and a caller silently taking the
    first would be inventing an answer.
    """
    if question.verified is None:
        return None

    kind = receipt_class(question.verified)
    if kind == "does_not_compile":
        matches = [
            i for i, o in enumerate(question.options) if o.kind == "does_not_compile"
        ]
    elif kind == "ran":
        if question.verified.stdout is None:
            return None
        wanted = normalized_output(question.verified.stdout)
        matches = [i for i, o in enumerate(question.options) if o.text == wanted]
    else:
        return None

    if len(matches) > 1:
        raise BankError(
            f"{question.id}: {len(matches)} options match the verified answer "
            f"(indexes {matches}), so no single option is the correct one"
        )
    return matches[0] if matches else None


def incorrect_indexes(question: Question) -> tuple[int, ...]:
    """Every option that is not the correct one (SPEC 4.5, AC-95).

    Raises `BankError` when the correct option cannot be derived, because "the
    incorrect options" is not a meaningful set until it is known which one is not.
    """
    correct = correct_index(question)
    if correct is None:
        raise BankError(
            f"{question.id}: cannot list the incorrect options - the correct one "
            "does not derive from the verified record"
        )
    return tuple(i for i in range(len(question.options)) if i != correct)


# The three ways a beat delimits a span: a double-quoted string, a backticked code
# span, and a bracketed debug print. Kept as separate patterns rather than one
# alternation so that a later reader can see which delimiters are covered, and so a
# fourth can be added without rewriting a regex nobody wants to read.
_DOUBLE_QUOTED = re.compile(r'"([^"\n]*)"')
_BACKTICKED = re.compile(r"`([^`\n]*)`")
_DEBUG_PRINTED = re.compile(r"(\[[^\[\]\n]*\])")


def quoted_outputs(question: Question) -> tuple[str, ...]:
    """Every quoted span in a question's beats, for T-18's AC-73 check.

    T-18 compares these against `verified.stdout` and blocks acceptance on a
    mismatch (AC-73). **This function does not do that comparison and does not
    decide which spans are claims about output** - it lists candidates.

    Searched: `explains.what`, `explains.takeaway`, `explains.legacy`, and every
    option's `why_tempting`. The hint is **not** searched: it is written to be read
    before anyone has seen an answer, so an output quoted in it is a leak for the
    canary suite (G-3) and not a mismatch for AC-73. Reporting it here would send a
    real finding to the wrong gate.

    **No filter on the shape of a span is applied, and that is a finding rather
    than a shortcut.** Reading the eight August explanations for what they actually
    quote, the two obvious rules each fail in a different direction:

    * *Drop identifier-shaped spans.* q2's explanation quotes `"created"`, which the
      program printed and which is also a perfectly good Rust identifier. `"false"`
      and `"true"` would go the same way. The rule drops real output.
    * *Keep bracketed or digit-bearing spans.* q8's explanation contains ``[0]``,
      from the expression `&v[0]` - bracketed, but an index and not output. And q1's
      `"end of main"` is printed output with no bracket and no digit in it, so the
      rule misses it while admitting the false positive.

    There is no syntactic property that separates a quoted output from a quoted
    identifier in this corpus, because the same text is legitimately both. Choosing
    would mean either blocking good questions - AC-73 is a *blocking* gate, so a
    false positive costs an organizer a question - or letting a wrong quote past the
    one check built to catch it. Deciding needs the verified output and a person, and
    T-18 has both: the review surface shows one question per screen to a reviewer who
    has already seen the answer (AC-22). So this hands over everything it found and
    decides nothing.

    Returns each distinct span once, in the order first seen, so a caller reporting a
    mismatch can quote it back to the author exactly as they wrote it. Empty spans are
    dropped, being nothing at all.
    """
    texts: list[str] = [question.explains.what, question.explains.takeaway]
    if question.explains.legacy is not None:
        texts.append(question.explains.legacy)
    texts.extend(o.why_tempting for o in question.options if o.why_tempting)

    seen: dict[str, None] = {}
    for text in texts:
        for pattern in (_DOUBLE_QUOTED, _BACKTICKED, _DEBUG_PRINTED):
            for match in pattern.finditer(text):
                span = match.group(1)
                if span:
                    seen.setdefault(span, None)
    return tuple(seen)


def options_needing_reauthoring(options: Sequence[Option]) -> tuple[int, ...]:
    """Indexes of options that break D-15's one-line, 29-character rule (SPEC 5.2).

    Takes the options rather than the question so it can be asked before a record is
    assembled - the MVP migration needs the answer while it is still building one.
    Not the audit: `bank-audit` fails a bank, and it also measures the source
    program's fit, which this says nothing about.
    """
    return tuple(i for i, o in enumerate(options) if not o.fits_the_wall())


def validate_question(question: Question) -> None:
    """Structural checks a record must pass to be stored. Raises `BankError`.

    What is here: five options with exactly one `does_not_compile` (AC-24's shape,
    though the bank-wide audit is T-19's); no duplicate option text, because two
    identical options make `correct_index` ambiguous; a trace whose last step names
    `stdout` **when the record ran and the trace is non-empty**.

    What is deliberately not here: the affirm gate (SPEC 7.4 - T-18's), the
    enumerated tells and the fit measurements (SPEC 7.6 - T-19's), and any check
    that a legacy record's absent fields are absent, which is a property of how
    `to_dict` serializes and is proven there.

    The trace condition is narrow on purpose. SPEC 3.1 and D-10 say the last step
    is the one whose `values` names `stdout`, but SPEC 3.2 says a does-not-compile
    record has no `stdout`, so requiring it of that class would make a class of
    question unstorable. Requiring it only of a record that ran is the narrowest
    reading that keeps both sentences true. It does not resolve the affirm gate,
    which asks for two trace steps from a question that can never have a resolving
    one - a contract defect reported with this ticket, not fixed in it.
    """
    if len(question.options) != OPTION_COUNT:
        raise BankError(
            f"{question.id}: {len(question.options)} options, expected {OPTION_COUNT}"
        )

    does_not_compile = [o for o in question.options if o.kind == "does_not_compile"]
    if len(does_not_compile) != 1:
        raise BankError(
            f"{question.id}: {len(does_not_compile)} options of kind "
            "'does_not_compile', expected exactly 1 (AC-24)"
        )

    texts = [o.text for o in question.options]
    if len(set(texts)) != len(texts):
        duplicated = sorted({t for t in texts if texts.count(t) > 1})
        raise BankError(f"{question.id}: duplicate option text: {duplicated}")

    if (
        receipt_class(question.verified) == "ran"
        and question.trace.steps
        and not question.trace.names_stdout_last()
    ):
        raise BankError(
            f"{question.id}: the trace's last step does not name 'stdout' in its "
            "values, and this question ran (SPEC 3.1, D-10)"
        )


# --------------------------------------------------------------------------- #
# Serialization - absent is absent
# --------------------------------------------------------------------------- #


def _dump(value: Any) -> Any:
    """Dataclasses to plain JSON types, dropping exactly the fields that are `None`.

    Dropping `None` and nothing else is the point. `stdout=""` is an observation - a
    program that printed nothing - and `exit_code=0` is an observation too, so a
    filter on falsiness would erase two facts and turn a complete record into one
    that renders a shorter receipt. Only `None`, which means nobody observed it,
    becomes an absent key (G-2, D-16).
    """
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        out: dict[str, Any] = {}
        for f in dataclasses.fields(value):
            attr = getattr(value, f.name)
            if attr is None:
                continue
            # `pivot` is optional in SPEC 5.3; false is the absence of a pivot, so
            # it stays out of the file rather than sitting in every step as noise.
            if f.name == "pivot" and attr is False:
                continue
            # An empty `legacy` flag is the ordinary case; writing `false` on every
            # complete record would make the exceptional case harder to spot.
            if f.name == "legacy" and attr is False:
                continue
            out[f.name] = _dump(attr)
        return out
    if isinstance(value, tuple):
        return [_dump(v) for v in value]
    return value


def question_to_dict(question: Question) -> dict[str, Any]:
    """The record as it is written to disk. No `correct` key: it is derived."""
    return _dump(question)


def _require_keys(data: dict[str, Any], known: set[str], where: str) -> None:
    unknown = sorted(set(data) - known)
    if unknown:
        raise BankError(f"{where}: unknown field(s) {unknown}")


def _field_names(cls: type) -> set[str]:
    return {f.name for f in dataclasses.fields(cls)}


def _option_from_dict(data: dict[str, Any], where: str) -> Option:
    _require_keys(data, _field_names(Option), where)
    return Option(
        text=data["text"], kind=data["kind"], why_tempting=data.get("why_tempting")
    )


def _trace_from_dict(data: dict[str, Any], where: str) -> Trace:
    _require_keys(data, _field_names(Trace), where)
    steps = []
    for n, raw in enumerate(data.get("steps", [])):
        _require_keys(raw, _field_names(TraceStep), f"{where}.steps[{n}]")
        values = []
        for m, v in enumerate(raw.get("values", [])):
            _require_keys(v, _field_names(ValueDelta), f"{where}.steps[{n}].values[{m}]")
            values.append(ValueDelta(name=v["name"], was=v["was"], now=v["now"]))
        steps.append(
            TraceStep(
                lines=tuple(raw["lines"]),
                focus=(raw["focus"][0], raw["focus"][1]),
                note=raw["note"],
                values=tuple(values),
                pivot=bool(raw.get("pivot", False)),
            )
        )
    return Trace(steps=tuple(steps))


def verified_from_dict(data: dict[str, Any], where: str = "verified") -> Verified:
    """A `verified` record from its JSON form. Used by the receipt fixtures too."""
    _require_keys(data, _field_names(Verified), where)

    flags = data.get("flags")
    runs = data.get("runs")
    miri = data.get("miri")
    codes = data.get("compile_error_code")

    if flags is not None:
        _require_keys(flags, _field_names(Flags), f"{where}.flags")
    if runs is not None:
        _require_keys(runs, _field_names(Runs), f"{where}.runs")
    if miri is not None:
        _require_keys(miri, _field_names(Miri), f"{where}.miri")

    return Verified(
        rustc=data["rustc"],
        edition=data["edition"],
        legacy=bool(data.get("legacy", False)),
        target_triple=data.get("target_triple"),
        flags=None
        if flags is None
        else Flags(
            opt_level=flags["opt_level"],
            overflow_checks=flags["overflow_checks"],
            debug_assertions=flags["debug_assertions"],
        ),
        runs=None
        if runs is None
        else Runs(count=runs["count"], byte_identical=runs["byte_identical"]),
        stdout=data.get("stdout"),
        exit_code=data.get("exit_code"),
        miri=None
        if miri is None
        else Miri(
            clean=miri["clean"],
            output_matched=miri["output_matched"],
            version=miri.get("version"),
            configs=None if miri.get("configs") is None else tuple(miri["configs"]),
            seeds=None if miri.get("seeds") is None else tuple(miri["seeds"]),
        ),
        compile_error_code=None if codes is None else tuple(codes),
        verified_at=data.get("verified_at"),
        verifier_version=data.get("verifier_version"),
    )


def question_from_dict(data: dict[str, Any]) -> Question:
    """A record from its JSON form, validated. Raises `BankError` on anything odd.

    An unknown field is an error rather than something to ignore: a typo in a key
    would otherwise read as a field somebody forgot to fill, and a record silently
    missing its beats is exactly the failure this project cannot have.
    """
    where = data.get("id", "<no id>")
    _require_keys(data, _field_names(Question), where)

    explains_raw = data["explains"]
    _require_keys(explains_raw, _field_names(Explains), f"{where}.explains")
    review_raw = data.get("review")
    if review_raw is not None:
        _require_keys(review_raw, _field_names(Review), f"{where}.review")
    used_raw = data.get("used")
    if used_raw is not None:
        _require_keys(used_raw, _field_names(Used), f"{where}.used")
    verified_raw = data.get("verified")

    question = Question(
        id=data["id"],
        source=data["source"],
        topic=data["topic"],
        difficulty_requested=data["difficulty_requested"],
        options=tuple(
            _option_from_dict(o, f"{where}.options[{n}]")
            for n, o in enumerate(data["options"])
        ),
        hint=data["hint"],
        explains=Explains(
            what=explains_raw["what"],
            takeaway=explains_raw["takeaway"],
            legacy=explains_raw.get("legacy"),
        ),
        trace=_trace_from_dict(data.get("trace", {}), f"{where}.trace"),
        verified=None
        if verified_raw is None
        else verified_from_dict(verified_raw, f"{where}.verified"),
        review=None
        if review_raw is None
        else Review(
            status=review_raw.get("status"),
            reason=review_raw.get("reason"),
            difficulty_judged=review_raw.get("difficulty_judged"),
            affirmed_by=review_raw.get("affirmed_by"),
            affirmed_at=review_raw.get("affirmed_at"),
            near_duplicate_of=review_raw.get("near_duplicate_of"),
        ),
        used=None
        if used_raw is None
        else Used(
            meetup_date=used_raw["meetup_date"],
            room_id=used_raw["room_id"],
            released_at=used_raw["released_at"],
            fit=used_raw["fit"],
        ),
    )
    validate_question(question)
    return question


# --------------------------------------------------------------------------- #
# The store on disk
# --------------------------------------------------------------------------- #

QUESTIONS_DIR = "questions"
HISTORY_FILE = "history.json"


def _write_json(path: Path, payload: Any) -> None:
    """Two spaces, sorted nowhere, and a trailing newline.

    Key order follows the dataclasses rather than the alphabet, so a record reads
    in the order SPEC 3.1 lists its fields and a diff in a pull request shows a
    changed beat next to the beat it replaced.
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def question_path(bank_dir: Path, question_id: str) -> Path:
    return Path(bank_dir) / QUESTIONS_DIR / f"{question_id}.json"


def save_question(bank_dir: Path, question: Question) -> Path:
    """Write one record. Validates first, so a malformed record never reaches disk."""
    validate_question(question)
    path = question_path(bank_dir, question.id)
    _write_json(path, question_to_dict(question))
    return path


def append_question(bank_dir: Path, question: Question) -> Path:
    """Write a record that is not there yet (SPEC 3.3: the bank is append-only).

    Refuses to overwrite. A question is replaced by re-verification writing a whole
    new record (D-16), which is a deliberate act with its own ticket, not something
    an append should do by accident.
    """
    path = question_path(bank_dir, question.id)
    if path.exists():
        raise BankError(
            f"{question.id}: {path} already exists, and the bank is append-only. "
            "Re-verification replaces a record whole; it does not append over one."
        )
    return save_question(bank_dir, question)


def load_question(bank_dir: Path, question_id: str) -> Question:
    path = question_path(bank_dir, question_id)
    if not path.is_file():
        raise BankError(f"no question {question_id!r} in the bank at {path}")
    return question_from_dict(json.loads(path.read_text(encoding="utf-8")))


def load_bank(bank_dir: Path) -> tuple[Question, ...]:
    """Every record, ordered by id so two runs on one bank agree."""
    directory = Path(bank_dir) / QUESTIONS_DIR
    if not directory.is_dir():
        return ()
    return tuple(
        question_from_dict(json.loads(p.read_text(encoding="utf-8")))
        for p in sorted(directory.glob("*.json"))
    )


# --------------------------------------------------------------------------- #
# SPEC 3.3 - the history store
# --------------------------------------------------------------------------- #

HISTORY_VERSION = 1

# The three stores SPEC 3.3 names, for AC-14 to AC-17. T-17 fills and reads them;
# this ticket fixes the shape and proves it persists.
HISTORY_STORES = ("exact_hashes", "ast_fingerprints", "token_bigrams")


@dataclass(frozen=True)
class History:
    """Dedupe's memory across runs and machines (SPEC 3.3, AC-14 to AC-17).

    Three stores, each mapping a question id to what was recorded for it: the exact
    hash of its source, its normalized-AST fingerprint, and its token-bigram set
    (D-13 - token-bigram Jaccard, no embedding model). All three are empty here.
    T-17 fills them, reads them, and proves AC-17 on this shape.

    `source_hash` is the one store this module can honestly populate, because
    hashing a string is not a judgment: it is here so the shape has a worked
    example rather than three empty dicts and a promise.
    """

    exact_hashes: dict[str, str] = field(default_factory=dict)
    ast_fingerprints: dict[str, str] = field(default_factory=dict)
    token_bigrams: dict[str, list[str]] = field(default_factory=dict)
    version: int = HISTORY_VERSION

    def sizes(self) -> dict[str, int]:
        """How many entries each store holds.

        AC-17 asks that the history's size and growth be visible; this is the size
        half, and the run report and review surface subtract two of these to get
        growth. T-17 owns both surfaces.
        """
        return {name: len(getattr(self, name)) for name in HISTORY_STORES}

    def total(self) -> int:
        return sum(self.sizes().values())


def source_hash(source: str) -> str:
    """The exact hash of a question's source, for AC-14's byte-identical check.

    SHA-256 of the UTF-8 bytes, hex. Exact means exact: no normalization, no
    stripping, no formatting pass. Normalizing is the *next* check (AC-15), and
    T-17 writes it; folding the two together here would lose the distinction
    between a resubmission and a reformatting, which are different rejections.
    """
    return hashlib.sha256(source.encode("utf-8")).hexdigest()


def history_path(bank_dir: Path) -> Path:
    return Path(bank_dir) / HISTORY_FILE


def load_history(bank_dir: Path) -> History:
    """The history, or an empty one if the file is not there yet."""
    path = history_path(bank_dir)
    if not path.is_file():
        return History()
    data = json.loads(path.read_text(encoding="utf-8"))
    _require_keys(data, _field_names(History), "history")
    version = data.get("version", HISTORY_VERSION)
    if version != HISTORY_VERSION:
        raise BankError(
            f"history at {path} is version {version}, and this code reads version "
            f"{HISTORY_VERSION}. A migration is a deliberate act, not a fallback."
        )
    return History(
        exact_hashes=dict(data.get("exact_hashes", {})),
        ast_fingerprints=dict(data.get("ast_fingerprints", {})),
        token_bigrams={k: list(v) for k, v in data.get("token_bigrams", {}).items()},
        version=version,
    )


def save_history(bank_dir: Path, history: History) -> Path:
    path = history_path(bank_dir)
    _write_json(
        path,
        {
            "version": history.version,
            "exact_hashes": history.exact_hashes,
            "ast_fingerprints": history.ast_fingerprints,
            "token_bigrams": history.token_bigrams,
        },
    )
    return path
