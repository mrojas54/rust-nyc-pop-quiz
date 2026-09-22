"""The receipt (SPEC 7.5): AC-13, AC-43 and AC-87.

Every case is driven from `bank/fixtures/receipts/`, not from records written inline
here. That is deliberate: T-05 needs the same function in JavaScript for the wall and
take-it-home, the contract asks for *one* function, and two languages cannot share
one. Testing both against the same files is how the build keeps them one function
rather than two that agree today.

It also means the expectations were written from SPEC 7.5 by hand, in the fixture
files, rather than recorded from this implementation. A fixture whose
`expected_lines` came out of `receipt_lines` would prove only that the code equals
itself.
"""

from __future__ import annotations

import ast
import json
import pathlib

import pytest

from popquiz.bank import verified_from_dict
from popquiz.receipt import (
    FORBIDDEN_IN_A_LINE,
    RECEIPT_HEADING,
    TERMINAL_PUNCTUATION,
    line_is_well_formed,
    receipt_detail,
    receipt_lines,
)

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
FIXTURES = REPO / "bank" / "fixtures" / "receipts"


def _cases() -> list[tuple[str, dict]]:
    """Every published fixture, by name. Fails if the directory is empty rather than
    passing vacuously - a suite that finds no cases is indistinguishable from one
    where every case passed."""
    paths = sorted(FIXTURES.glob("*.json"))
    assert paths, f"no receipt fixtures found under {FIXTURES}"
    return [(p.stem, json.loads(p.read_text(encoding="utf-8"))) for p in paths]


CASES = _cases()
CASE_IDS = [name for name, _ in CASES]


def _lines_for(case: dict) -> list[str] | None:
    return receipt_lines(verified_from_dict(case["verified"]))


@pytest.mark.parametrize(("name", "case"), CASES, ids=CASE_IDS)
def test_each_fixture_renders_the_lines_the_spec_gives_it(name: str, case: dict) -> None:
    """AC-43: the receipt is exactly SPEC 7.5's lines for the record, in its order."""
    assert _lines_for(case) == case["expected_lines"], name


@pytest.mark.parametrize(("name", "case"), CASES, ids=CASE_IDS)
def test_no_rendered_line_overstates(name: str, case: dict) -> None:
    """AC-43: no line ends in terminal punctuation, and none carries a claim word.

    The list shows steps that were taken. A line that ended in a full stop would be
    a sentence, and a line containing *verified*, *established*, *proves*, *always*
    or *guaranteed* would assert a result the steps do not support - which
    PHILOSOPHY 4 calls the one unforgivable bug in a project about correctness.
    """
    for line in _lines_for(case) or []:
        assert not line.endswith(TERMINAL_PUNCTUATION), f"{name}: {line!r} ends a sentence"
        lowered = line.lower()
        for word in FORBIDDEN_IN_A_LINE:
            assert word not in lowered, f"{name}: {line!r} contains {word!r}"
        assert line_is_well_formed(line), f"{name}: {line!r}"


def test_the_overstatement_check_catches_what_it_claims_to() -> None:
    """The check above is only worth having if it fires.

    Without this, a refactor could turn `line_is_well_formed` into something that
    accepts everything, and the suite would go green for the wrong reason - the same
    failure mode the scaffold's process-guard test guards against.
    """
    for bad in (
        "✓ Compiled.",
        "✓ Answer verified",
        "✓ This proves it",
        "✓ Output always identical",
        "✓ Determinism guaranteed",
        "✓ Established by Miri",
    ):
        assert not line_is_well_formed(bad), f"missed: {bad!r}"
    for good in ("✓ Compiled", "✓ Ran 5 times", "✓ Output never varied", "✓ Nothing ran"):
        assert line_is_well_formed(good), f"false positive: {good!r}"


def _case(name: str) -> dict:
    return dict(CASES)[name]


def test_a_legacy_record_renders_the_same_list_as_a_complete_one() -> None:
    """AC-87, D-25: both render the same four lines, and the wall names no machine.

    This is the whole reason the receipt became a list of steps. The August batch
    recorded no target triple, so any receipt that named one would have overstated on
    a migrated question; a list of steps has no machine to name.
    """
    legacy = _lines_for(_case("legacy-ran"))
    complete = _lines_for(_case("complete-ran"))

    assert legacy == complete
    assert legacy is not None
    joined = " ".join(legacy).lower()
    for machine_word in ("aarch64", "darwin", "triple", "machine", "computer"):
        assert machine_word not in joined


def test_a_does_not_compile_record_claims_no_run_and_no_miri() -> None:
    """AC-87, D-22: its own three lines, and nothing about an execution."""
    for name in ("q8-does-not-compile-legacy", "does-not-compile-two-codes"):
        lines = _lines_for(_case(name))
        assert lines is not None and len(lines) == 3, name
        joined = " ".join(lines).lower()
        assert "ran " not in joined and "times" not in joined, name
        assert "miri" not in joined, name
        assert "nothing ran" in joined, name


def test_the_two_lists_are_the_only_two() -> None:
    """SPEC 7.5: does-not-compile first, otherwise the four-line list, and no third.

    Checked across every published fixture, so a later fixture cannot introduce a
    shape the precedence rule does not cover.
    """
    for name, case in CASES:
        lines = _lines_for(case)
        if lines is None:
            continue
        first = lines[0]
        assert first in ("✓ Compiler refused it", "✓ Compiled"), f"{name}: {first!r}"


def test_a_record_that_is_none_of_the_three_renders_no_receipt() -> None:
    """AC-87: no receipt, and `None` rather than an empty list.

    An empty list would render a *How we know* heading with nothing under it, which
    claims a verification happened and shows none.
    """
    assert _lines_for(_case("no-receipt")) is None
    assert receipt_lines(None) is None


def test_a_line_renders_only_when_the_record_holds_its_step() -> None:
    """G-2, AC-43: the gates fire, so the list cannot claim more than was done."""
    varied = _lines_for(_case("output-varied"))
    assert varied is not None
    assert "✓ Output never varied" not in varied
    assert "✓ Compiled" in varied  # the other steps still happened

    mismatched = _lines_for(_case("miri-output-mismatched"))
    assert mismatched is not None
    assert not any("Miri" in line for line in mismatched)


def test_undefined_behavior_as_the_answer_reads_as_a_flag_not_a_pass() -> None:
    """AC-9, AC-43: `miri.clean` false renders the flagged wording, still one line."""
    lines = _lines_for(_case("ub-declared"))
    assert lines is not None
    miri_lines = [line for line in lines if "Miri" in line]
    assert miri_lines == ["✓ Miri flagged undefined behavior"]


def test_a_panic_answer_renders_the_full_list() -> None:
    """AC-43: empty `stdout` and a non-zero exit code still compiled and still ran.

    'Compiled' gates on the runs being present, not on the program having printed
    anything, so a question whose answer is a panic is not silently short a line.
    """
    lines = _lines_for(_case("panic-answer"))
    assert lines == [
        "✓ Compiled",
        "✓ Ran 5 times",
        "✓ Output never varied",
        "✓ Miri ran clean",
    ]


def test_the_heading_is_the_spec_s_words_and_is_not_a_step() -> None:
    """SPEC 7.5. The heading carries AC-74's provenance marker, which is the
    renderer's business, so it is a constant rather than a returned line."""
    assert RECEIPT_HEADING == "How we know"
    for _, case in CASES:
        assert RECEIPT_HEADING not in (_lines_for(case) or [])


# --------------------------------------------------------------------------- #
# AC-87 on take-it-home, and AC-13 structurally
# --------------------------------------------------------------------------- #


def test_detail_gives_the_machine_facts_for_a_complete_record() -> None:
    """AC-87, SPEC 13: the full -Vv, the edition, the triple, the flags, Miri's config."""
    detail = receipt_detail(verified_from_dict(_case("complete-ran")["verified"]))
    assert detail is not None
    assert detail.legacy is False
    assert detail.ran is True
    assert detail.target_triple == "aarch64-apple-darwin"
    assert detail.edition == "2021"
    assert detail.flags is not None and detail.flags.opt_level == "0"
    assert detail.miri is not None and detail.miri.configs == (
        "stacked_borrows",
        "tree_borrows",
    )
    # The full -Vv, not the one-line --version string.
    assert "host:" in detail.compiler
    assert detail.miri_ran_outside_verifier is False


def test_detail_back_fills_nothing_for_a_legacy_record() -> None:
    """AC-87, D-16, G-2: the facts take-it-home needs are derivable, and absence stays.

    The page renders `target_triple is None` as *not recorded* and the flag as the
    Miri row saying the check was run separately. Those strings are participant-facing
    copy on a surface T-12 renders and T-22 freezes; this ticket derives the facts and
    does not author the words.
    """
    detail = receipt_detail(verified_from_dict(_case("legacy-ran")["verified"]))
    assert detail is not None
    assert detail.legacy is True
    assert detail.ran is True
    assert detail.target_triple is None
    assert detail.flags is None
    assert detail.miri_ran_outside_verifier is True
    # Present but bare: the August pass recorded a result and nothing about itself.
    assert detail.miri is not None
    assert detail.miri.version is None
    assert detail.miri.configs is None
    assert detail.miri.seeds is None
    # The --version string, which is all the MVP wrote. Never a -Vv built from it.
    assert "\n" not in detail.compiler
    assert "host:" not in detail.compiler


def test_detail_says_nothing_about_determinism_for_a_does_not_compile_record() -> None:
    """D-22: nothing ran, so there is no run and no Miri result to show."""
    detail = receipt_detail(
        verified_from_dict(_case("q8-does-not-compile-legacy")["verified"])
    )
    assert detail is not None
    assert detail.ran is False
    assert detail.miri is None
    assert detail.compile_error_code == ("E0502",)
    assert detail.miri_ran_outside_verifier is False


def test_detail_and_lines_agree_about_which_records_have_a_receipt() -> None:
    """The two functions must not disagree, or take-it-home renders a *How we know*
    section for a record the wall shows nothing for."""
    for name, case in CASES:
        verified = verified_from_dict(case["verified"])
        assert (receipt_lines(verified) is None) == (
            receipt_detail(verified) is None
        ), name
    assert receipt_detail(None) is None


def test_the_receipt_module_needs_no_toolchain_and_no_world() -> None:
    """AC-13: the receipt renders from the `verified` record alone.

    Proven structurally rather than by observing one call succeed: `receipt.py`
    imports only the standard library and this package, so there is no `rustc`, no
    filesystem, no clock and no network anywhere in its reach. The scaffold's
    `test_nothing_just_test_imports_can_start_a_process` covers the suite's ability
    to start a process; this covers this module's ability to depend on a world.

    `dataclasses` and `__future__` are the standard library. `popquiz.bank` is types
    and pure functions, and is scanned by the same guard.
    """
    allowed = {"__future__", "dataclasses", "popquiz.bank"}
    source = (HERE.parent / "src" / "popquiz" / "receipt.py").read_text(encoding="utf-8")
    imported = set()
    for node in ast.walk(ast.parse(source)):
        if isinstance(node, ast.Import):
            imported.update(a.name for a in node.names)
        elif isinstance(node, ast.ImportFrom):
            imported.add(node.module or "")
    assert imported <= allowed, f"receipt.py imports {sorted(imported - allowed)}"
