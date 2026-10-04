"""bank-audit (SPEC 7.6): the generator, the slot path's purity, and the bank checks.

Every check here has two kinds of test: one that it passes on the real thing, and
one that it FAILS on a planted defect. The second kind is the one that matters.
G-1 has regressed four times and none of the four was caught by reading code; a
check that cannot be shown to fire is indistinguishable from no check.

The synthetic questions and faces below describe no Rust program. They exist to
exercise the audit's arithmetic, which reads only an option's kind, its length,
its bank position and which one the verified record makes correct - never what a
program printed. Their `stdout` values are placeholders in the style
`test_bank.py` uses, and say so.
"""

from __future__ import annotations

import datetime
import json
import math
import pathlib
import random
import shutil

import pytest

from popquiz import audit, bank
from popquiz import slot as slot_module
from popquiz.audit import AuditFailure, Face
from popquiz.bank import Explains, Miri, Option, Question, Review, Runs, Verified

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
BANK = REPO / "bank"

LEGACY_RUSTC = "rustc 1.96.1 (31fca3adb 2026-06-26)"
DNC = "does_not_compile"


# --------------------------------------------------------------------------- #
# AC-23b - the generator audit, both tails
# --------------------------------------------------------------------------- #

# Critical values of the chi-square distribution, df = 4, from the NIST/SEMATECH
# e-Handbook of Statistical Methods, section 1.3.6.7.4, "Critical Values of the
# Chi-Square Distribution" (https://www.itl.nist.gov/div898/handbook/eda/section3/eda3674.htm).
# Keyed (tail, probability of falling beyond the value in that tail).
CHI2_DF4_TABLE = {
    ("upper", 0.001): 18.467,
    ("lower", 0.01): 0.297,
    ("lower", 0.001): 0.091,
}


def chi2_df4_cdf(x: float) -> float:
    """df = 4 has a closed form: F(x) = 1 - e^(-x/2) (1 + x/2)."""
    return 1 - math.exp(-x / 2) * (1 + x / 2)


def test_the_chi_square_table_is_what_it_says() -> None:
    """The table is checked against the closed form, so a mistyped value cannot
    hide - which is exactly how `build_deck.py:180` came to label the 1 % lower
    point "p = 0.999"."""
    for (tail, probability), value in CHI2_DF4_TABLE.items():
        beyond = 1 - chi2_df4_cdf(value) if tail == "upper" else chi2_df4_cdf(value)
        assert round(beyond, 3) == probability, (tail, probability, value, beyond)


def test_the_audit_uses_the_tables_values() -> None:
    assert audit.CHI2_DF4_UPPER == CHI2_DF4_TABLE[("upper", 0.001)]
    # The MVP's lower value, kept: the 1 % point, not the 0.1 % one it was labelled.
    assert audit.CHI2_DF4_LOWER == CHI2_DF4_TABLE[("lower", 0.01)]


def test_the_real_generator_passes_both_tails() -> None:
    result = audit.audit_generator()
    assert result.sample == 20_000
    assert sum(result.counts) == 20_000
    assert audit.CHI2_DF4_LOWER < result.chi2 < audit.CHI2_DF4_UPPER
    assert result.repeats > 0
    assert result.start == "2026-08-12"


def test_the_audit_reexports_the_one_slot_function() -> None:
    assert audit.slot_for_day is slot_module.slot_for_day


def _skewed(share_of_a: float, seed: int):
    rng = random.Random(seed)
    return lambda day: 0 if rng.random() < share_of_a else rng.randrange(1, 5)


def _rebalanced():
    """The fourth regression's repair, as a generator: the least-used letter,
    never last night's. It reads like diligence."""
    counts = [0] * 5
    previous = [None]

    def draw(day: datetime.date) -> int:
        pick = min((k for k in range(5) if k != previous[0]), key=lambda k: (counts[k], k))
        counts[pick] += 1
        previous[0] = pick
        return pick

    return draw


def _never_repeats(seed: int):
    """Uniform on average, but never last night's letter."""
    rng = random.Random(seed)
    previous = [None]

    def draw(day: datetime.date) -> int:
        pick = rng.choice([k for k in range(5) if k != previous[0]])
        previous[0] = pick
        return pick

    return draw


def test_a_skewed_generator_fails_the_upper_tail() -> None:
    with pytest.raises(AuditFailure, match="skewed"):
        audit.audit_generator(_skewed(0.26, seed=11))


def test_a_rebalanced_generator_fails_the_lower_tail() -> None:
    """Too even is the exploitable failure: enforced balance makes every fifth
    meetup certain. The lower tail is what catches it."""
    with pytest.raises(AuditFailure, match="TOO EVEN"):
        audit.audit_generator(_rebalanced())


def test_a_generator_that_never_repeats_fails() -> None:
    """Its counts are fine in both tails; only the repeat check sees it."""
    with pytest.raises(AuditFailure, match="never repeats"):
        audit.audit_generator(_never_repeats(seed=5))


def test_a_generator_drawing_outside_the_five_fails() -> None:
    with pytest.raises(AuditFailure, match="not a slot"):
        audit.audit_generator(lambda day: 5)


# --------------------------------------------------------------------------- #
# AC-23a - three attendees who remember every night
# --------------------------------------------------------------------------- #


def test_three_attendees_with_perfect_memory_stay_at_chance() -> None:
    """10,000 consecutive nights from 2026-08-12 (the first scheduled meetup). The
    simulation's own randomness - only the not-last-night's attendee has any - is
    seeded with `audit.ATTENDEE_SEED` (20260918)."""
    rates = audit.simulate_attendees()
    assert set(rates) == {"most used so far", "least used so far", "not last night's"}
    assert audit.attendees_outside_band(rates) == {}, rates


def test_the_simulation_catches_the_balancer() -> None:
    """The fourth regression measured 45.7 %; this harness has to see it."""
    rates = audit.simulate_attendees(_rebalanced())
    assert rates["least used so far"] > 0.40, rates
    assert "least used so far" in audit.attendees_outside_band(rates)


def test_the_simulation_catches_never_the_same_letter_twice() -> None:
    rates = audit.simulate_attendees(_never_repeats(seed=5))
    assert "not last night's" in audit.attendees_outside_band(rates), rates


def test_the_simulation_catches_a_skew() -> None:
    rates = audit.simulate_attendees(_skewed(0.30, seed=11))
    assert "most used so far" in audit.attendees_outside_band(rates), rates


def test_the_attendees_guess_before_the_night_is_drawn() -> None:
    """Against a generator that always says D, the most-used attendee has no past on
    night one and guesses blind (A, the lowest letter), then hits every night after.
    99 of 100 - not 100 - is what shows the guess was made before the draw."""
    rates = audit.simulate_attendees(lambda day: 3, nights=100)
    assert rates["most used so far"] == 0.99


# --------------------------------------------------------------------------- #
# AC-23 static, G-1, G-10 - the slot path stays pure; the ledger stays retired
# --------------------------------------------------------------------------- #

SLOT_SOURCE = audit.slot_source()
SIGNATURE = "def slot_for_day(day: date, n_options: int = 5) -> int:"
DRAW = '    return _rng("meetup-slot", day.isoformat())(n_options)'


def with_change(old: str, new: str) -> str:
    assert old in SLOT_SOURCE, f"the test's anchor is gone from slot.py: {old!r}"
    return SLOT_SOURCE.replace(old, new)


def test_the_real_slot_module_is_clean() -> None:
    assert audit.slot_path_violations(SLOT_SOURCE) == []


def test_the_real_pipeline_is_clean() -> None:
    sources = audit.pipeline_sources()
    assert "slot.py" in sources and "audit.py" in sources
    assert audit.slot_call_violations(sources) == []
    assert audit.ledger_violations(sources) == []


SLOT_PLANTS = [
    ("history parameter", with_change(SIGNATURE, "def slot_for_day(day: date, n_options: int = 5, history=None) -> int:"), "exactly (day, n_options=5)"),
    ("n_options default 4", with_change(SIGNATURE, "def slot_for_day(day: date, n_options: int = 4) -> int:"), "literal 5"),
    ("varargs", with_change(SIGNATURE, "def slot_for_day(day: date, *extra, n_options: int = 5) -> int:"), "exactly"),
    ("import json", with_change("import hashlib\n", "import hashlib\nimport json\n"), "imports 'json'"),
    ("import the bank", with_change("import hashlib\n", "import hashlib\nfrom popquiz import bank\n"), "popquiz"),
    ("open the ledger", with_change(DRAW, "    open('answer-history.json')\n" + DRAW), "calls open()"),
    ("json.load", with_change(DRAW, "    json.load(day)\n" + DRAW), "reads 'json'"),
    ("load the bank", with_change(DRAW, "    bank.load_bank(day)\n" + DRAW), "'bank'"),
    ("mutable module value", with_change("\n\ndef _rng", "\n\nPREVIOUS_SLOTS = []\n\n\ndef _rng"), "nowhere for state to live"),
    ("rebound constant", with_change("\n\ndef _rng", "\n\nSEED = 1\nSEED = 2\n\n\ndef _rng"), "bound 2 times"),
    ("global", with_change(DRAW, "    global LAST\n" + DRAW), "global LAST"),
    ("cache decorator", with_change(SIGNATURE, "@functools.cache\n" + SIGNATURE), "decorated"),
    ("with block", with_change(DRAW, "    with day:\n        pass\n" + DRAW), "`with` block"),
    ("nested import", with_change(DRAW, "    import os\n" + DRAW), "inside a function"),
    ("unlisted attribute", with_change(DRAW, "    tally = day.toordinal()\n" + DRAW), "attribute 'toordinal'"),
    ("history in a string", with_change(DRAW, '    note = "never the last letter"\n' + DRAW), "names history"),
    ("history in a name", with_change(DRAW, "    usage_count = 0\n" + DRAW), "reaches for history"),
]


@pytest.mark.parametrize(
    ("planted", "caught_because"),
    [pytest.param(source, reason, id=name) for name, source, reason in SLOT_PLANTS],
)
def test_the_slot_lint_catches_what_it_claims_to(planted: str, caught_because: str) -> None:
    """Each planted change is a way the slot path has acquired, or could acquire, an
    input. The strings are handed to `ast.parse`, never executed."""
    problems = audit.slot_path_violations(planted)
    assert any(caught_because in p for p in problems), problems


def test_the_slot_lint_allows_the_closure_the_generator_needs() -> None:
    """`nonlocal` binds only a name in an enclosing function - it cannot reach
    module state - and the ported `_rng` needs it for its own stream."""
    assert "nonlocal seed" in SLOT_SOURCE
    assert audit.slot_path_violations(SLOT_SOURCE) == []


@pytest.mark.parametrize(
    ("source", "clean"),
    [
        ("from popquiz.slot import slot_for_day\nslot_for_day(d)", True),
        ("from popquiz.slot import slot_for_day\nslot_for_day(d, 5)", True),
        ("from popquiz.slot import slot_for_day\nslot_for_day(day=d, n_options=5)", True),
        ("from popquiz.slot import slot_for_day\nslot_for_day(d, len(q.options))", False),
        ("from popquiz.slot import slot_for_day\nslot_for_day(d, n_options=n)", False),
        ("from popquiz.slot import slot_for_day\nslot_for_day(d, history)", False),
        ("from popquiz.slot import slot_for_day\nslot_for_day(*args)", False),
        ("from popquiz.slot import slot_for_day as pick\npick(d, len(o))", False),
        ("from popquiz import audit\naudit.slot_for_day(d, 4)", False),
    ],
)
def test_the_call_site_lint(source: str, clean: bool) -> None:
    """n_options is the constant 5, never derived from data (SPEC G-1)."""
    assert (audit.slot_call_violations({"x.py": source}) == []) is clean


def test_the_ledger_lint_catches_the_mvp_wrapper_under_any_name() -> None:
    """`build_deck.py:220` drew a slot and wrote the ledger in one function. Renamed,
    it is the same defect."""
    wrapper = (
        "import json\n"
        "from popquiz.slot import slot_for_day\n"
        "def place_answer(day, path):\n"
        "    s = slot_for_day(day)\n"
        "    path.write_text(json.dumps({'slot': s}))\n"
        "    return s\n"
    )
    problems = audit.ledger_violations({"schedule.py": wrapper})
    assert any("place_answer() draws a slot and writes a file" in p for p in problems), problems


@pytest.mark.parametrize(
    "planted",
    [
        "LEDGER = 'mvp/answer-history.json'\n",
        "path = root / 'answer_history.json'\n",
        "import build_deck\n",
        "from mvp.tools import build_deck\n",
        "from mvp.tools.build_deck import slot_for_meetup\n",
    ],
)
def test_the_ledger_lint_catches_the_ledger(planted: str) -> None:
    assert audit.ledger_violations({"x.py": planted}), planted


def test_the_ledger_lint_reads_prose_as_prose() -> None:
    """A docstring may explain why the ledger is not read; that is not reading it."""
    documented = '"""Never reads mvp/answer-history.json."""\n\ndef f():\n    """Nor here: answer-history."""\n'
    assert audit.ledger_violations({"x.py": documented}) == []


def test_a_function_that_writes_without_drawing_is_fine() -> None:
    writer = "def save(path, text):\n    path.write_text(text)\n"
    assert audit.ledger_violations({"x.py": writer}) == []


def test_the_record_has_nowhere_to_keep_a_position() -> None:
    """A ledger needs a field to live in. No dataclass in the bank's record format has
    one named for a slot, a position or a letter - `Used` records when a question
    ran and what the wall measured, and nothing about where its answer sat."""
    import dataclasses

    offenders = [
        f"{name}.{f.name}"
        for name, cls in vars(bank).items()
        if isinstance(cls, type) and dataclasses.is_dataclass(cls)
        for f in dataclasses.fields(cls)
        if any(word in f.name.lower() for word in ("slot", "position", "letter"))
    ]
    assert offenders == []


# --------------------------------------------------------------------------- #
# AC-24 - five options, exactly one does not compile
# --------------------------------------------------------------------------- #


def test_every_real_question_has_five_options_and_one_does_not_compile() -> None:
    paths = sorted((BANK / "questions").glob("*.json"))
    assert paths
    for path in paths:
        assert audit.check_five_options(json.loads(path.read_text(encoding="utf-8")), path.name) == []


def _record(kinds: list[str]) -> dict:
    return {"options": [{"text": f"option {n}", "kind": k} for n, k in enumerate(kinds)]}


@pytest.mark.parametrize(
    ("kinds", "complaint"),
    [
        (["output", "output", "output", DNC], "4 options"),
        (["output", "output", DNC, DNC, "output"], "2 options of kind does_not_compile"),
        (["output"] * 5, "0 options of kind does_not_compile"),
    ],
)
def test_five_options_fails_on_a_malformed_record(kinds: list[str], complaint: str) -> None:
    problems = audit.check_five_options(_record(kinds), "q99.json")
    assert any(complaint in p for p in problems), problems


# --------------------------------------------------------------------------- #
# AC-25, G-11 - no published distribution
# --------------------------------------------------------------------------- #


def test_the_real_participant_facing_files_are_clean() -> None:
    hits, scanned, _absent = audit.distribution_lint(REPO)
    assert hits == [], [str(h) for h in hits]
    assert "README.md" in scanned and "mvp/README.md" in scanned


def test_the_copy_table_and_take_it_home_are_in_scope() -> None:
    """`web/shared/copy.js` arrives with PR #9 (branch ai-c11-cc/shared-web-layer,
    BUILDPLAN T-02) and take-it-home with T-12. Neither is on this branch yet;
    both are linted the moment they are in the tree."""
    assert "web/shared/copy.js" in audit.PARTICIPANT_FACING
    assert "web/home" in audit.PARTICIPANT_FACING_DIRS


@pytest.mark.parametrize(
    "planted",
    [
        "About 20% of answers are *does not compile*.",
        "Roughly one in five questions does not compile.",
        "3/10 of the answers are a panic.",
        "Half the answers are UB, so guess it.",
        "Only 2 of 8 printed output worth reading.",
        "The answer is undefined behaviour 40 percent of the time.",
    ],
)
def test_a_published_distribution_is_caught(planted: str) -> None:
    assert audit.distribution_hits(planted + "\n", "README.md"), planted


def test_a_ratio_one_line_from_a_kind_word_is_caught() -> None:
    text = "Last season:\n20% of the time\nthe answer did not compile.\n"
    assert [h.line for h in audit.distribution_hits(text, "x.md")] == [2]


@pytest.mark.parametrize(
    "innocent",
    [
        "20% of the room\n\n\nsaw the output",  # the kind word is three lines away
        "Doors at 18:30; the screen is 16:9.\nIt will not compile your code.",  # times and aspect ratios
        "20% of pubs close early.",  # "ub" inside a word is not UB
        "The room holds 200 people.\nAnswers are closed.",
    ],
)
def test_ordinary_numbers_are_not_a_distribution(innocent: str) -> None:
    assert audit.distribution_hits(innocent, "x.md") == [], innocent


def test_the_one_exemption_covers_one_exact_line() -> None:
    """`mvp/README.md:149` states the odds of guessing a position, beside the
    does-not-compile bullet. Exempt as written; any edit re-arms the lint, and the
    same words anywhere else are not exempt."""
    exempt = "with perfect memory of every previous answer is still guessing 1 in 5."
    context = "  An attendee\n  {line}\n- **\"does not compile\" appears on every question**\n"
    assert audit.distribution_hits(context.format(line=exempt), "mvp/README.md") == []
    edited = exempt.replace("1 in 5", "1 in 4")
    assert audit.distribution_hits(context.format(line=edited), "mvp/README.md")
    assert audit.distribution_hits(context.format(line=exempt), "README.md")


def test_the_exempt_line_is_still_what_was_reviewed() -> None:
    """If the MVP README is rewritten, the exemption must be re-reviewed, not left
    silently covering nothing."""
    readme = (REPO / "mvp" / "README.md").read_text(encoding="utf-8")
    for (path, line), _reason in audit.LINT_EXEMPTIONS.items():
        assert line in (REPO / path).read_text(encoding="utf-8"), (path, line)
    assert "guessing 1 in 5." in readme


def test_nothing_participant_facing_reads_the_tell_report() -> None:
    assert audit.bank_audit_references(REPO) == []


def test_a_page_that_reads_the_tell_report_is_caught(tmp_path: pathlib.Path) -> None:
    page = tmp_path / "web" / "home" / "index.js"
    page.parent.mkdir(parents=True)
    page.write_text("fetch('/bank/audit/2026-10-14.json')\n", encoding="utf-8")
    assert audit.bank_audit_references(tmp_path) == ["web/home/index.js"]


def test_the_report_is_written_under_bank_audit_and_nowhere_else(tmp_path: pathlib.Path) -> None:
    path = audit.write_report(tmp_path, {"checks": []}, "2026-09-21")
    assert path == (tmp_path / "bank" / "audit" / "2026-09-21.json").resolve()
    assert json.loads(path.read_text(encoding="utf-8"))["date"] == "2026-09-21"
    for escape in ("../../web/leak", "2026-09-21/../../x", "/tmp/x"):
        with pytest.raises(ValueError):
            audit.write_report(tmp_path, {"checks": []}, escape)


# --------------------------------------------------------------------------- #
# AC-26 - the enumerated tells
# --------------------------------------------------------------------------- #


def face(
    n: int,
    correct: int,
    *,
    kinds: tuple[str, ...] = ("output", "output", "output", DNC, "output"),
    lengths: tuple[int, ...] = (10, 11, 12, 16, 13),
    unsafe: bool = False,
    lines: int = 5,
    topic: str | None = None,
) -> Face:
    return Face(f"f{n}", unsafe, lines, topic or f"topic {n}", kinds, lengths, correct)


def with_dnc_at(position: int) -> tuple[str, ...]:
    kinds = ["output"] * 5
    kinds[position] = DNC
    return tuple(kinds)


def no_tell_bank() -> list[Face]:
    """Twenty faces built so that no rule has anything to learn: the correct
    option's bank position, its length rank and does-not-compile's position all
    cycle, and a fifth of the answers are *does not compile*, spread across its
    positions."""
    faces = []
    for n in range(20):
        dnc = n % 5
        if n % 5 == n // 5:
            correct = dnc
        else:
            outputs = [i for i in range(5) if i != dnc]
            correct = outputs[(n // 5 + n) % 4]
        base = [8, 12, 16, 20, 24]
        shift = (n * 3) % 5
        faces.append(
            face(
                n,
                correct,
                kinds=with_dnc_at(dnc),
                lengths=tuple(base[shift:] + base[:shift]),
                lines=3 + n % 5,
                topic=f"topic {n % 5}",
            )
        )
    return faces


def verdicts(faces: list[Face]) -> dict[str, str]:
    return {t.tell: t.verdict for t in audit.measure_tells(faces)}


def test_six_lines_are_measured_and_five_are_ac26s() -> None:
    names = [t.tell for t in audit.measure_tells(no_tell_bank())]
    assert names == ["unsafe", "source length", "option text length", "option position", "topic", "answer category"]


def test_a_bank_with_no_tell_passes_every_line() -> None:
    assert set(verdicts(no_tell_bank()).values()) <= {"pass", "n/a"}, verdicts(no_tell_bank())


def test_unsafe_meaning_ub_is_a_tell() -> None:
    faces = [
        face(n, 1, kinds=("output", "ub", "output", DNC, "output"), unsafe=True) for n in range(10)
    ] + [face(n, 0, kinds=("output", "ub", "output", DNC, "output")) for n in range(10, 20)]
    assert verdicts(faces)["unsafe"] == "fail"


def test_long_programs_not_compiling_is_a_tell() -> None:
    faces = [face(n, 3, lines=9) for n in range(10)] + [face(n, n % 3, lines=4) for n in range(10, 20)]
    assert verdicts(faces)["source length"] == "fail"


def test_one_topic_always_not_compiling_is_a_tell() -> None:
    faces = [face(n, 3, topic="Borrow checking") for n in range(10)] + [
        face(n, n % 3, topic=f"topic {n % 4}") for n in range(10, 20)
    ]
    assert verdicts(faces)["topic"] == "fail"


def test_the_longest_option_being_correct_is_a_tell() -> None:
    faces = []
    for n in range(20):
        correct = n % 5
        lengths = [8, 9, 10, 11, 12]
        lengths[correct] = 25
        dnc = (correct + 1) % 5
        faces.append(face(n, correct, kinds=with_dnc_at(dnc), lengths=tuple(lengths)))
    assert verdicts(faces)["option text length"] == "fail"


def test_the_answer_always_in_one_place_is_a_tell() -> None:
    faces = [face(n, 2, kinds=with_dnc_at(n % 2 * 4)) for n in range(20)]
    assert verdicts(faces)["option position"] == "fail"


def test_does_not_compile_being_the_answer_too_often_is_a_tell() -> None:
    """PHILOSOPHY section 2's first example: "never pick does not compile" works
    when it is rarely the answer; "always pick it" works when it is often."""
    faces = [face(n, 3) for n in range(12)] + [face(n, n % 3) for n in range(12, 20)]
    assert verdicts(faces)["answer category"] == "fail"


def test_the_answer_written_last_warns_at_four_and_fails_at_five() -> None:
    """Every migrated record puts its correct option at bank index 4 (C1 in the plan):
    a WARN at the bank's present size, a FAIL the day a fifth arrives that way."""
    assert verdicts([face(n, 4) for n in range(4)])["option position"] == "warn"
    assert verdicts([face(n, 4) for n in range(5)])["option position"] == "fail"


def test_a_small_bank_does_not_fail_on_luck() -> None:
    """Two hits in four is 2.5x chance and happens by luck a fifth of the time; the
    bare margin would fail it. It warns, it does not fail."""
    faces = [face(0, 4), face(1, 4), face(2, 0), face(3, 1)]
    assert verdicts(faces)["option position"] == "warn"


def test_the_real_bank_has_no_failing_tell() -> None:
    questions = bank.load_bank(BANK)
    faces, left_out = audit.tell_pool(questions)
    assert len(faces) == len(questions) and left_out == []
    failing = {t.tell: t.worst for t in audit.measure_tells(faces) if t.verdict == "fail"}
    assert failing == {}


def test_the_tail_is_exact() -> None:
    assert audit.poisson_binomial_tail([0.2] * 4, 4) == pytest.approx(0.2**4)
    assert audit.poisson_binomial_tail([0.2] * 4, 0) == 1.0
    for hits in range(11):
        binomial = sum(math.comb(10, k) * 0.3**k * 0.7 ** (10 - k) for k in range(hits, 11))
        assert audit.poisson_binomial_tail([0.3] * 10, hits) == pytest.approx(binomial)


# --------------------------------------------------------------------------- #
# Synthetic questions, for AC-27 and AC-88
# --------------------------------------------------------------------------- #


def question(
    qid: str,
    *,
    answer: str = "output",
    status: str | None = "accepted",
    unsafe: bool = False,
    requested: int = 2,
    judged: int | None = None,
) -> Question:
    """A synthetic record. Its source is a comment and its `stdout` a placeholder:
    it describes no program and nobody ran it."""
    texts = [f"{qid} placeholder {n}" for n in range(4)]
    kinds = ["output"] * 4
    if answer == "ub":
        texts[1], kinds[1] = "undefined behavior", "ub"
    options = tuple(Option(t, k) for t, k in zip(texts, kinds)) + (Option("does not compile", DNC),)
    if answer == "ub":
        verified = Verified(
            rustc=LEGACY_RUSTC, edition="2021", legacy=True, runs=Runs(5, True), stdout="",
            miri=Miri(clean=False, output_matched=True),
        )
    else:
        verified = Verified(
            rustc=LEGACY_RUSTC, edition="2021", legacy=True, runs=Runs(5, True),
            stdout=texts[0] + "\n", miri=Miri(clean=True, output_matched=True),
        )
    source = "// synthetic: describes no program\n" + ("unsafe {}\n" if unsafe else "")
    return Question(
        id=qid,
        source=source,
        topic="Synthetic",
        difficulty_requested=requested,
        options=options,
        hint="placeholder",
        explains=Explains(what="placeholder", takeaway="placeholder"),
        verified=verified,
        review=None if status is None else Review(status=status, difficulty_judged=judged),
    )


def test_a_ub_answer_derives_from_the_miri_record() -> None:
    q = question("u1", answer="ub")
    assert bank.correct_index(q) is None  # C2: bank.correct_index does not cover UB
    assert audit.answer_index(q) == 1
    assert audit.answer_is_ub(q)


def test_a_question_whose_answer_does_not_derive_is_left_out_by_name() -> None:
    """Options and the machine's output drifted apart: nothing matches, so there is no
    answer to score and none is guessed. Rejected questions are not in the pool."""
    import dataclasses

    derives = question("x1")
    drifted = dataclasses.replace(
        question("x2"),
        verified=dataclasses.replace(derives.verified, stdout="no option says this\n"),
    )
    faces, left_out = audit.tell_pool([derives, drifted, question("r1", status="rejected")])
    assert [f.id for f in faces] == ["x1"]
    assert left_out == ["x2"]


# --------------------------------------------------------------------------- #
# AC-27 - unsafe parity
# --------------------------------------------------------------------------- #


def test_no_ub_answers_needs_no_decoy() -> None:
    assert audit.check_unsafe_parity([question("a"), question("b")]).verdict == "pass"


def test_a_ub_answer_with_unsafe_nowhere_else_fails() -> None:
    result = audit.check_unsafe_parity([question("u1", answer="ub", unsafe=True), question("a")])
    assert result.verdict == "fail"
    assert "u1" in result.summary


def test_a_non_ub_question_containing_unsafe_restores_parity() -> None:
    questions = [question("u1", answer="ub", unsafe=True), question("d1", unsafe=True)]
    assert audit.check_unsafe_parity(questions).verdict == "pass"


def test_only_accepted_questions_count_toward_parity() -> None:
    rejected_decoy = question("d1", unsafe=True, status="rejected")
    unreviewed_decoy = question("d2", unsafe=True, status=None)
    questions = [question("u1", answer="ub", unsafe=True), rejected_decoy, unreviewed_decoy]
    assert audit.check_unsafe_parity(questions).verdict == "fail"


def test_an_edited_question_counts_as_accepted() -> None:
    """D7: edit re-verifies and keeps a question; leaving it out would let an edited
    UB answer slip past the check."""
    assert audit.check_unsafe_parity([question("u1", answer="ub", status="edited")]).verdict == "fail"
    assert audit.check_unsafe_parity(
        [question("u1", answer="ub", status="edited"), question("d1", unsafe=True, status="edited")]
    ).verdict == "pass"


# --------------------------------------------------------------------------- #
# AC-88 - difficulty drift fails the run
# --------------------------------------------------------------------------- #


def test_drift_above_one_level_fails_the_run_and_names_no_question() -> None:
    run = [
        question("drift-q1", requested=1, judged=3),
        question("drift-q2", requested=2, judged=4),
        question("drift-q3", judged=2),
    ]
    result = audit.difficulty_drift(run)
    assert result.verdict == "fail"
    assert result.detail["mean_drift"] == pytest.approx(4 / 3)
    for q in run:
        assert q.id not in result.summary


def test_drift_of_exactly_one_level_passes() -> None:
    run = [question(q, requested=2, judged=3) for q in "abc"]
    assert audit.difficulty_drift(run).verdict == "pass"


def test_drift_ignores_unjudged_and_unaccepted_questions() -> None:
    run = [question("a", judged=2), question("b"), question("c", status=None, judged=5)]
    result = audit.difficulty_drift(run)
    assert (result.verdict, result.detail["judged"]) == ("pass", 1)


def test_no_judged_sample_is_said_rather_than_passed() -> None:
    assert audit.difficulty_drift([question("a")]).verdict == "n/a"


# --------------------------------------------------------------------------- #
# The whole run
# --------------------------------------------------------------------------- #


def test_the_real_bank_passes_every_bank_level_check() -> None:
    report = audit.run_audit(REPO)
    assert report["failed"] == [], [c for c in report["checks"] if c["verdict"] == "fail"]
    assert audit.exit_status(report) == 0
    assert set(report["bank"]) == {p.name for p in (BANK / "questions").glob("*.json")}


def copy_bank(tmp_path: pathlib.Path) -> pathlib.Path:
    shutil.copytree(BANK / "questions", tmp_path / "bank" / "questions")
    return tmp_path


def test_the_command_line_writes_its_report_under_bank_audit(
    tmp_path: pathlib.Path, capsys: pytest.CaptureFixture[str]
) -> None:
    repo = copy_bank(tmp_path)
    assert audit.main(["--repo", str(repo), "--date", "2026-09-21"]) == 0
    report = json.loads((repo / "bank" / "audit" / "2026-09-21.json").read_text(encoding="utf-8"))
    assert report["date"] == "2026-09-21"
    assert report["room"]["status"] == "hypothesis until HC-1"
    assert "bank-audit: passed" in capsys.readouterr().out


def test_a_flagged_question_in_the_reserve_fails_the_run(
    tmp_path: pathlib.Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """An option too long for the wall is a flag while its question waits, and a
    failure the moment the question is accepted and affirmed, because then tonight's
    schedule could pick it (AC-100: flagged before it can be scheduled).

    The long option is made here, in the copy: every option in the real bank fits
    since D-15's re-authoring, so the bank no longer holds one to lean on."""
    repo = copy_bank(tmp_path)
    path = repo / "bank" / "questions" / "q7.json"
    record = json.loads(path.read_text(encoding="utf-8"))
    wrong = next(o for o in record["options"] if o["kind"] == "output" and "why_tempting" in o)
    wrong["text"] = "x" * 48
    record["review"].update(status="accepted", affirmed_by="organizer", affirmed_at="2026-09-21T20:00:00Z")
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    assert audit.main(["--repo", str(repo), "--date", "2026-09-21"]) == 1
    out = capsys.readouterr().out
    assert "IN THE RESERVE, schedulable tonight: q7" in out
    assert "bank-audit: FAILED" in out


def test_strict_fails_on_any_flag(tmp_path: pathlib.Path) -> None:
    """A flag on a question still waiting for review is reported, and fails the run
    only under --strict.

    No question in the real bank carries a flag since D-15's re-authoring, and the
    option-position tell warns, which fails --strict on its own - so a copy of the
    bank as it is would pass this through the warning and prove nothing about
    flags. The flag is made here, in the copy, as the reserve test above makes
    one, and --strict is asked of the report with its warnings set aside.

    q7 is put back to waiting for review in the copy, too, rather than trusted to
    be waiting in the real bank: an organizer affirming it would otherwise put the
    flagged copy in the reserve and fail this test on a legitimate change."""
    repo = copy_bank(tmp_path)
    path = repo / "bank" / "questions" / "q7.json"
    record = json.loads(path.read_text(encoding="utf-8"))
    for field in ("status", "affirmed_by", "affirmed_at"):
        record["review"].pop(field, None)
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    clean = audit.run_audit(repo)
    assert not any(q["flags"] for q in clean["questions"].values())
    assert audit.exit_status({**clean, "warned": []}, strict=True) == 0

    wrong = next(o for o in record["options"] if o["kind"] == "output" and "why_tempting" in o)
    wrong["text"] = "x" * 48
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    flagged = audit.run_audit(repo)
    assert flagged["questions"]["q7"]["flags"]
    assert not flagged["questions"]["q7"]["in_reserve"]
    assert flagged["failed"] == []
    assert "fits the configured room" in flagged["warned"]
    assert audit.exit_status({**flagged, "warned": []}, strict=True) == 1
    assert audit.main(["--repo", str(repo), "--date", "2026-09-21"]) == 0
    assert audit.main(["--repo", str(repo), "--date", "2026-09-21", "--strict"]) == 1


def test_a_malformed_record_is_reported_not_crashed_on(tmp_path: pathlib.Path) -> None:
    repo = copy_bank(tmp_path)
    path = repo / "bank" / "questions" / "q3.json"
    record = json.loads(path.read_text(encoding="utf-8"))
    record["options"] = record["options"][:4]
    path.write_text(json.dumps(record), encoding="utf-8")
    report = audit.run_audit(repo)
    assert "five options, one does not compile" in report["failed"]
    assert audit.exit_status(report) == 1
