"""The affirmation rule's shared fixtures (G-12, GAP-24, ADR-0005 item 9).

`bank.affirmation_digest` and `bank.affirmation_state` are the rule. The room
refuses a question the rule does not call `bound`, with a Rust twin of the rule
(`room/src/quiz_session/models/affirmation.rs`). One rule, never two: both are held to the
cases in `bank/fixtures/affirmation/`. This file checks that the rule gives each
case's digest and state, and `room/tests/twins.rs` checks that the room gives the
same. A change on either side that moves a digest fails here or there.

The fixtures are written by `test_write_the_fixtures`, which runs only with
`POPQUIZ_REGEN_AFFIRMATION=1`. Rewriting them changes the rule, so the room's
twin must change in the same commit.
"""

from __future__ import annotations

import copy
import json
import os
import re
from pathlib import Path
from typing import Any

import pytest

from popquiz import schedule
from popquiz.bank import (
    BankError,
    affirmation_digest,
    affirmation_state,
    correct_index,
    load_bank,
    question_from_dict,
)

REPO = Path(__file__).resolve().parents[2]
BANK = REPO / "bank"
FIXTURES = BANK / "fixtures" / "affirmation"

#: A wall order that is not the bank's: slot k holds the bank's option `ARRANGED[k]`.
ARRANGED = [3, 0, 4, 1, 2]

#: Every escape `json.dumps(..., ensure_ascii=False)` writes, and characters it
#: writes raw: a quote, a backslash, \b \f \n \r \t, other control characters as
#: \u00xx, DEL, U+2028, non-ASCII and a character outside the BMP.
ESCAPES = 'a "quote", a back\\slash, \b\f\n\r\t, \x01\x1f\x7f,  , é ü 日本, \U0001f980'


def _q3() -> dict[str, Any]:
    return json.loads((BANK / "questions" / "q3.json").read_text(encoding="utf-8"))


def _bound(data: dict[str, Any]) -> dict[str, Any]:
    """`data` affirmed for its own text, as `popquiz.review affirm` would leave it."""
    data = copy.deepcopy(data)
    review = data.setdefault("review", {})
    review.setdefault("affirmed_by", "fixture-organizer")
    review.setdefault("affirmed_at", "2026-10-18T00:00:00+00:00")
    review.pop("affirmed_digest", None)
    review["affirmed_digest"] = affirmation_digest(question_from_dict(data))
    return data


def _edited(edit) -> dict[str, Any]:
    data = _q3()
    edit(data)
    return data


def _cases() -> dict[str, dict[str, Any]]:
    """Each case: `record`, and `arranged` where the room is to be pushed it in that order."""
    correct = correct_index(question_from_dict(_q3()))

    def why_empty(d):
        d["options"][correct]["why_tempting"] = ""

    def pivot_false(d):
        d["trace"]["steps"][0]["pivot"] = False

    def pivot_moved(d):
        for step in d["trace"]["steps"]:
            step.pop("pivot", None)
        d["trace"]["steps"][0]["pivot"] = True

    def values_absent(d):
        for step in d["trace"]["steps"]:
            if step["values"] == []:
                del step["values"]

    def no_legacy(d):
        del d["explains"]["legacy"]

    def escapes(d):
        d["hint"] = ESCAPES
        d["explains"]["what"] = ESCAPES
        d["trace"]["steps"][0]["note"] = ESCAPES

    def hint_edited(d):
        d["hint"] = d["hint"] + " Edited that afternoon."

    def no_digest(d):
        del d["review"]["affirmed_digest"]

    def nobody(d):
        del d["review"]["affirmed_digest"]
        d["review"]["affirmed_by"] = ""

    def no_review(d):
        del d["review"]

    def digest_without_who(d):
        del d["review"]["affirmed_by"]
        del d["review"]["affirmed_at"]

    def malformed(d):
        d["review"]["affirmed_digest"] = "sha256:A888"

    def trace_absent(d):
        del d["trace"]

    def trace_empty(d):
        d["trace"] = {"steps": []}

    return {
        "q3": {"record": _q3()},
        "q3-arranged": {"record": _q3(), "arranged": ARRANGED},
        "escapes": {"record": _bound(_edited(escapes))},
        "why-tempting-empty": {"record": _bound(_edited(why_empty))},
        "pivot-false-written": {"record": _edited(pivot_false)},
        "pivot-moved": {"record": _bound(_edited(pivot_moved))},
        "values-absent": {"record": _edited(values_absent)},
        "legacy-absent": {"record": _bound(_edited(no_legacy))},
        "trace-absent": {"record": _bound(_edited(trace_absent))},
        "trace-empty": {"record": _bound(_edited(trace_empty))},
        "stale-hint-edited": {"record": _edited(hint_edited)},
        "unbound-no-digest": {"record": _edited(no_digest)},
        "none-nobody": {"record": _edited(nobody)},
        "none-no-review": {"record": _edited(no_review)},
        "refused-digest-without-who": {"record": _edited(digest_without_who)},
        "refused-malformed-digest": {"record": _edited(malformed)},
    }


def _expected(case: dict[str, Any]) -> dict[str, Any]:
    try:
        question = question_from_dict(case["record"])
    except BankError as e:
        return {"state": "refused", "reason": str(e).split(": ", 1)[1]}
    return {"state": affirmation_state(question), "digest": affirmation_digest(question)}


def _fixtures() -> list[tuple[str, dict[str, Any]]]:
    paths = sorted(FIXTURES.glob("*.json"))
    assert paths, f"no affirmation fixtures found under {FIXTURES}"
    return [(p.stem, json.loads(p.read_text(encoding="utf-8"))) for p in paths]


@pytest.mark.skipif(
    os.environ.get("POPQUIZ_REGEN_AFFIRMATION") != "1",
    reason="writes bank/fixtures/affirmation/; set POPQUIZ_REGEN_AFFIRMATION=1 to rewrite them",
)
def test_write_the_fixtures():
    FIXTURES.mkdir(parents=True, exist_ok=True)
    for name, case in _cases().items():
        fixture = {**case, **_expected(case)}
        text = json.dumps(fixture, ensure_ascii=False, indent=2) + "\n"
        (FIXTURES / f"{name}.json").write_text(text, encoding="utf-8")


def test_the_fixtures_are_the_cases_this_file_builds():
    on_disk = dict(_fixtures())
    assert sorted(on_disk) == sorted(_cases())
    for name, case in _cases().items():
        assert on_disk[name] == {**case, **_expected(case)}, name


@pytest.mark.parametrize("path", sorted(FIXTURES.glob("*.json")), ids=lambda p: p.stem)
def test_the_rule_gives_each_fixture_its_state_and_digest(path):
    fixture = json.loads(path.read_text(encoding="utf-8"))
    if fixture["state"] == "refused":
        with pytest.raises(BankError, match=re.escape(fixture["reason"])):
            question_from_dict(fixture["record"])
        return
    question = question_from_dict(fixture["record"])
    assert affirmation_state(question) == fixture["state"]
    assert affirmation_digest(question) == fixture["digest"]


def test_the_fixtures_cover_every_state():
    states = {f["state"] for _, f in _fixtures()}
    assert states == {"bound", "stale", "unbound", "none", "refused"}


def test_written_defaults_and_absent_keys_hash_the_same():
    """`_dump` drops `None`, `pivot: false` and `legacy: false`, and always writes
    `values` and `steps`, so these pairs are one text and one digest."""
    f = dict(_fixtures())
    assert f["pivot-false-written"]["digest"] == f["q3"]["digest"]
    assert f["values-absent"]["digest"] == f["q3"]["digest"]
    assert f["trace-absent"]["digest"] == f["trace-empty"]["digest"]
    # And these are not: an empty string is a value, and a moved pivot is an edit.
    assert f["why-tempting-empty"]["digest"] != f["q3"]["digest"]
    assert f["pivot-moved"]["digest"] != f["q3"]["digest"]


def test_the_arranged_case_is_the_order_schedule_sends():
    """The room gets the wall's order and `bank_order` with it; `bank_order` is what
    puts the options back, and is the one `schedule.bank_order` computes."""
    fixture = dict(_fixtures())["q3-arranged"]
    question = question_from_dict(fixture["record"])
    arranged = [question.options[i] for i in fixture["arranged"]]
    shown = question_from_dict({**fixture["record"], "options": [vars(o) for o in arranged]})
    assert schedule.bank_order(question, shown) == fixture["arranged"]


def test_q3_is_bound_in_the_bank():
    """Ruling 5: q3 still schedules. Its affirmation is bound to its current text."""
    states = {q.id: affirmation_state(q) for q in load_bank(BANK)}
    assert states["q3"] == "bound"
