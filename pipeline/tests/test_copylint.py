"""The copy lints over question prose (SPEC 11, 11.1; G-5, AC-98, AC-42, D-23).

Three things, as in `web/test/copylint.test.js`, which runs the JavaScript copy of
the same patterns against the same files:

1. the patterns are SPEC.md's, read out of SPEC.md rather than retyped here;
2. they behave as the hand-written fixtures in `bank/fixtures/copy-lint/` say,
   including the one documented divergence between the two regex engines;
3. `check_prose` finds matches in a question's prose homes and never raises, and
   over every committed question it reports and never fails. A match in question
   prose is a warning for the organizer on the review screen (SPEC 7.4), not a
   build failure; this suite prints them through pytest's warnings summary.
"""

from __future__ import annotations

import copy
import json
import pathlib
import re
import warnings

import pytest

from popquiz import copylint
from popquiz.bank import question_from_dict

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
FIXTURES = REPO / "bank" / "fixtures" / "copy-lint"
QUESTIONS = REPO / "bank" / "questions"
# Text cases: {"text", "expect"}. notes.json holds whole records for check_prose.
TEXT_FIXTURES = [
    "retired.json",
    "forbidden.json",
    "tropes.json",
    "near-misses.json",
    "clean.json",
    "dialect.json",
    "normalization.json",
]
FIXTURE_FILES = [*TEXT_FIXTURES, "notes.json"]


class ProseWarning(UserWarning):
    """A lint match in committed question prose: information for review, never a failure."""


# --- 1. SPEC's patterns, verbatim ------------------------------------------------


def _spec_patterns() -> tuple[list[str], dict[str, list[str]]]:
    spec = (REPO / "SPEC.md").read_text(encoding="utf-8")
    row = next(line for line in spec.splitlines() if line.startswith("| **Forbidden**"))
    start = row.index("cites this row |") + len("cites this row |")
    listed = row[start : row.index("The wall's")]
    forbidden = [m.replace("\\|", "|") for m in re.findall(r"`((?:[^`\\]|\\.)*)`", listed)]
    at = spec.index("### 11.1 The trope check")
    opened = spec.index("```", at)
    closed = spec.index("```", opened + 3)
    tropes: dict[str, list[str]] = {}
    group = None
    for line in spec[opened + 3 : closed].splitlines():
        if not line.strip():
            continue
        m = re.match(r"^(\S+)?\s+(\S.*)$", line)
        assert m, line
        if m.group(1):
            group = m.group(1)
        tropes.setdefault(group, []).append(m.group(2).strip())
    return forbidden, tropes


def test_the_forbidden_patterns_are_spec_s_row_verbatim_and_in_order():
    forbidden, _ = _spec_patterns()
    assert len(forbidden) == 9
    assert list(copylint.FORBIDDEN) == forbidden


def test_the_trope_patterns_are_spec_s_verbatim_grouped_and_in_order():
    _, tropes = _spec_patterns()
    assert list(tropes) == ["contrast", "filler", "signpost", "reassurance", "flattery"]
    assert {g: list(p) for g, p in copylint.TROPES.items()} == tropes


def test_the_compiled_whitespace_class_is_javascript_s():
    # ECMA-262 WhiteSpace and LineTerminator, code point by code point.
    js = {0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x20, 0xA0, 0x1680, *range(0x2000, 0x200B), 0x2028, 0x2029, 0x202F, 0x205F, 0x3000, 0xFEFF}
    rx = re.compile(copylint.JS_WHITESPACE)
    matched = {cp for cp in range(0x10000) if rx.fullmatch(chr(cp))}
    assert matched == js


# --- 2. The shared fixtures ------------------------------------------------------


def _cases(name: str) -> list[dict]:
    cases = json.loads((FIXTURES / name).read_text(encoding="utf-8"))["cases"]
    assert cases, f"{name} has no cases"
    return cases


def test_the_fixture_set_is_the_one_the_javascript_suite_reads():
    assert sorted(p.name for p in FIXTURES.glob("*.json")) == sorted(FIXTURE_FILES)


@pytest.mark.parametrize("name", TEXT_FIXTURES)
def test_every_case_matches_exactly_the_rules_it_names(name):
    for case in _cases(name):
        want = case["expect"] if "expect" in case else case["expect_py"]
        got = [h.id for h in copylint.check(case["text"])]
        assert got == want, f"{case['text']!r} {case.get('note', '')}"


def test_the_engines_alone_behave_as_dialect_json_says():
    # normalize() now collapses the whitespace these cases probe, so the engine
    # dialect is held here, over the text as given.
    for case in _cases("dialect.json"):
        want = case.get("expect_raw", case.get("expect", case.get("expect_py")))
        assert copylint.check_raw(case["text"]) == want, case["note"]


def test_every_normalization_probe_walked_past_the_rules_before_normalize():
    for case in _cases("normalization.json"):
        assert copylint.check_raw(case["text"]) != case["expect"], case["note"]
        assert [h.id for h in copylint.check(case["text"])] == case["expect"], case["note"]


def test_normalize_is_idempotent_and_a_match_is_quoted_from_the_normalized_text():
    for case in _cases("normalization.json") + _cases("dialect.json"):
        once = copylint.normalize(case["text"])
        assert copylint.normalize(once) == once, case["note"]
    [hit] = copylint.check("That\u02bcs\u00a0fine.")
    assert (hit.id, hit.match) == ("reassurance/1", "That's fine")


def test_the_only_divergence_is_the_one_dialect_json_names():
    split = [c for c in _cases("dialect.json") if "expect" not in c]
    assert len(split) == 1
    assert split[0]["expect_js"] != split[0]["expect_py"]


def test_every_pattern_has_a_positive_case_of_its_own():
    fired = {i for c in _cases("forbidden.json") + _cases("tropes.json") for i in c["expect"]}
    assert {r.id for r in copylint.RULES} <= fired


def test_every_trope_group_fires_on_the_retired_strings():
    retired = _cases("retired.json")
    for group in copylint.TROPES:
        assert any(h.group == group for c in retired for h in copylint.trope_matches(c["text"])), group
    for c in retired:
        assert copylint.trope_matches(c["text"]), c["text"]


def test_the_allowed_near_misses_stay_allowed():
    for c in _cases("near-misses.json"):
        assert copylint.forbidden_matches(c["text"]) == []


# --- 3. check_prose --------------------------------------------------------------


def _q3() -> dict:
    return json.loads((QUESTIONS / "q3.json").read_text(encoding="utf-8"))


def test_check_prose_reads_its_homes_and_only_its_homes():
    record = _q3()
    clean = "It prints 3."
    record["hint"] = clean
    record["explains"]["what"] = clean
    record["explains"]["takeaway"] = clean
    record["explains"]["legacy"] = "That's fine, it is wrong."  # never rendered: never read
    record["source"] = 'fn main() { println!("that\'s fine, wrong"); }'  # program text
    for i, option in enumerate(record["options"]):
        option["text"] = "error: wrong number of arguments"  # program text
        if "why_tempting" in option:
            option["why_tempting"] = clean
    assert copylint.check_prose(record) == []

    record["hint"] = "Don't worry, it is not just a copy."
    record["explains"]["takeaway"] = "The key part is the drop."
    tempting = next(i for i, o in enumerate(record["options"]) if "why_tempting" in o)
    record["options"][tempting]["why_tempting"] = "That answer is wrong."
    got = {(w.field, w.group, w.matched) for w in copylint.check_prose(record)}
    assert got == {
        ("hint", "contrast", "not just"),
        ("hint", "reassurance", "Don't worry"),
        ("explains.takeaway", "signpost", "The key part"),
        (f"options[{tempting}].why_tempting", "forbidden", "wrong"),
    }


def _records() -> list[dict]:
    records = json.loads((FIXTURES / "notes.json").read_text(encoding="utf-8"))["records"]
    assert records, "notes.json has no records"
    return records


def test_trace_notes_are_read_and_warned_on_as_hint_is():
    # F-47: AC-98 covers every participant-facing string, the trace's notes included.
    for r in _records():
        got = [[w.field, w.id] for w in copylint.check_prose(r["record"])]
        assert got == r["expect"], r["note"]


def test_the_notes_fixture_trips_every_rule_family_in_a_note():
    fired = {i.split("/")[0] for r in _records() for f, i in r["expect"] if f.startswith("trace.steps[")}
    assert fired == {"forbidden", *copylint.TROPES}


def test_a_note_warning_is_the_same_warning_hint_gets():
    note = "Don't worry, it is not just a copy."
    as_hint = copylint.check_prose({"hint": note})
    as_note = copylint.check_prose({"trace": {"steps": [{"note": note}]}})
    assert [(w.group, w.pattern, w.matched, w.id) for w in as_note] == [(w.group, w.pattern, w.matched, w.id) for w in as_hint]
    assert {w.field for w in as_note} == {"trace.steps[0].note"}


def test_check_prose_takes_a_question_as_well_as_its_json():
    record = _q3()
    as_json = copylint.check_prose(record)
    assert copylint.check_prose(question_from_dict(copy.deepcopy(record))) == as_json


@pytest.mark.parametrize(
    "record",
    [
        None,
        {},
        [],
        "a string",
        42,
        {"explains": None, "options": None, "hint": None},
        {"explains": "not a mapping", "options": "not a list", "hint": 7},
        {"explains": {"what": 3}, "options": [None, 5, {"why_tempting": ["x"]}], "hint": b"bytes"},
    ],
)
def test_check_prose_never_raises(record):
    assert copylint.check_prose(record) == []


def test_check_prose_reads_what_it_can_of_a_partial_record():
    got = copylint.check_prose({"options": [{"text": "A"}, {"why_tempting": "Don't worry."}]})
    assert [(w.field, w.group) for w in got] == [("options[1].why_tempting", "reassurance")]


def test_every_committed_question_is_checked_and_its_warnings_reported():
    # Information, not a failure (SPEC 11.1): the review screen shows these beside the
    # text and the organizer decides. pytest prints each in its warnings summary.
    paths = sorted(QUESTIONS.glob("*.json"))
    assert paths, f"no questions under {QUESTIONS}"
    for path in paths:
        record = json.loads(path.read_text(encoding="utf-8"))
        for w in copylint.check_prose(record):
            warnings.warn(ProseWarning(f"{path.stem} {w.field}: {w.group} {w.pattern!r} matched {w.matched!r}"), stacklevel=1)
