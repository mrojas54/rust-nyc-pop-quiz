"""AC-100 and D-15: the wall's type model, mirrored for the offline audit.

`bank-audit` fits every question against the configured room with no browser to
measure in, so it reimplements stage 1 of `web/shared/typemodel.js` (floorPx,
sourceMetrics, desiredFontPx) and must agree with it exactly. SPEC 5.2's worked
examples are the fixture both implementations test against; the JS side's are in
`web/test/typemodel.test.js` on PR #9 (branch ai-c11-cc/shared-web-layer,
BUILDPLAN T-02).

What this file proves is the arithmetic and the flag. It does not prove the wall's
measured claim - "fits" after layout - which is `test-full`'s, with a browser.
"""

from __future__ import annotations

import json
import pathlib
import re

import pytest

from popquiz import audit, bank
from popquiz.audit import READING_AREA, TRACE_AREA, Room

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
FIT_FIXTURES = HERE / "fixtures" / "fit"
TYPEMODEL_JS = REPO / "web" / "shared" / "typemodel.js"


def to1(x: float) -> float:
    """Round at the edge, where a number is asserted - never inside the model."""
    return round(x, 1)


def shaped(lines: int, chars: int) -> str:
    return "\n".join("x" * chars for _ in range(lines)) + "\n"


def test_the_constants_are_the_ones_spec_5_2_states() -> None:
    assert audit.CANVAS_H == 630
    assert audit.CAP_RATIO == 0.7
    assert audit.READ_RATIO == 150
    assert audit.MAX_FONT == 46
    assert audit.LINE_HEIGHT == 1.6
    assert audit.GUTTER_CH == 3.5
    assert audit.CHAR_ADVANCE == 0.6
    assert READING_AREA == (994, 177)
    assert TRACE_AREA == (994, 190)
    assert audit.OPTION_MAX_CHARS == 29 == bank.MAX_OPTION_LINE_CHARS
    assert Room() == Room(screen_width_ft=15, screen_height_ft=8.44, back_row_ft=20)


def test_the_default_room_gives_a_14_2_px_floor() -> None:
    assert to1(audit.floor_px()) == 14.2
    assert to1(audit.floor_px(Room())) == 14.2


def test_the_floor_moves_with_the_room() -> None:
    assert audit.floor_px(Room(back_row_ft=40)) > audit.floor_px() * 1.99
    assert audit.floor_px(Room(screen_width_ft=30, screen_height_ft=16.88)) < audit.floor_px()


# SPEC 5.2's worked table at the 15 / 8.44 / 20 guess (floor 14.2 px). `wants` is
# what the source asks for before the floor applies - the number SPEC reports,
# including for the four below the floor.
WORKED = [
    ("q3", 5, 42, 22.1, True),
    ("q4", 6, 56, 18.4, True),
    ("q7", 5, 69, 22.1, True),
    ("q8", 6, 30, 18.4, True),
    ("q5", 9, 50, 12.3, False),
    ("q6", 9, 32, 12.3, False),
    ("q2", 10, 38, 11.1, False),
    ("q1", 16, 36, 6.9, False),
]


@pytest.mark.parametrize(("qid", "lines", "chars", "wants", "fits"), WORKED, ids=[w[0] for w in WORKED])
def test_spec_5_2s_worked_examples_reproduce_to_one_decimal(
    qid: str, lines: int, chars: int, wants: float, fits: bool
) -> None:
    source = shaped(lines, chars)
    assert audit.source_metrics(source) == (lines, chars)
    result = audit.fit(source, READING_AREA)
    assert to1(result.wants) == wants
    assert result.fits is fits
    if fits:
        assert to1(result.font_px) == wants
    else:
        # Legibility wins: the wall renders at the floor, and the question is too
        # long for this room - the room's limit, not a constant in the generator.
        assert result.font_px == result.floor_px
        assert audit.FLAG_PROGRAM in audit.question_flags(_with_source(source))


def test_by_source_length_only_q3_q4_q7_q8_fit() -> None:
    fitting = [qid for qid, lines, chars, _, _ in WORKED if audit.fit(shaped(lines, chars)).fits]
    assert fitting == ["q3", "q4", "q7", "q8"]


def test_the_real_bank_sources_have_the_shapes_spec_measured() -> None:
    """The worked table was measured from the August sources; the four in the bank
    still have those shapes, and all four fit as programs."""
    expected = {"q3": (5, 42), "q4": (6, 56), "q7": (5, 69), "q8": (6, 30)}
    for q in bank.load_bank(REPO / "bank"):
        assert audit.source_metrics(q.source) == expected[q.id], q.id
        assert audit.fit(q.source, READING_AREA).fits, q.id


def test_nothing_is_rounded_inside_the_model() -> None:
    """q3 wants 177 / 8 = 22.125 exactly. The prototype rounded to one decimal
    inside its refit; two implementations that round at different moments drift
    apart (typemodel.js, "NO ROUNDING IN HERE")."""
    assert audit.fit(shaped(5, 42)).wants == 177 / (5 * 1.6)
    assert audit.floor_px() == (20 * 12 / 150 / 0.7) / (8.44 * 12) * 630


def test_the_trace_layout_is_reported_beside_the_reading_one() -> None:
    for _, lines, chars, _, _ in WORKED:
        source = shaped(lines, chars)
        assert audit.fit(source, TRACE_AREA).wants >= audit.fit(source, READING_AREA).wants
    # The trace layout holds 8 lines at the floor where the reading layout holds 7.
    eight = shaped(8, 20)
    assert audit.fit(eight, TRACE_AREA).fits and not audit.fit(eight, READING_AREA).fits


def test_the_floor_wins_over_the_46_cap() -> None:
    tiny_room = Room(screen_width_ft=4, screen_height_ft=2.25, back_row_ft=30)
    assert audit.floor_px(tiny_room) > audit.MAX_FONT
    result = audit.fit(shaped(2, 10), READING_AREA, tiny_room)
    assert result.wants == audit.MAX_FONT
    assert result.font_px == result.floor_px > audit.MAX_FONT


def test_lines_split_exactly_as_the_well_does() -> None:
    """well.js strips ONE trailing newline, then splits: a trailing blank line after
    that is a line the wall draws."""
    assert audit.source_metrics("a\nbb\n") == (2, 2)
    assert audit.source_metrics("a\nbb") == (2, 2)
    assert audit.source_metrics("a\nbb\n\n") == (3, 2)


def test_width_counts_characters_as_javascript_does() -> None:
    """UTF-16 code units, as JS `.length` counts them, so the audit is never looser
    than the wall. An astral character is two."""
    assert audit.js_length("é") == 1
    assert audit.js_length("🦀") == 2
    line = "let c = '🦀';"
    assert len(line) == 12
    assert audit.source_metrics(line + "\n") == (1, 13)


@pytest.mark.parametrize(
    ("text", "fits"),
    [
        ("x" * 29, True),
        ("x" * 30, False),
        ("does not compile", True),
        ("-3 -1\n-4 1", False),  # two lines: the wall has no design for a wrapped option
        ("🦀" * 15, False),  # 15 characters, 30 UTF-16 units
    ],
)
def test_option_length_is_one_line_of_at_most_29(text: str, fits: bool) -> None:
    assert audit.option_fits(text) is fits


def test_option_length_agrees_with_the_bank_on_every_real_option() -> None:
    for q in bank.load_bank(REPO / "bank"):
        for option in q.options:
            assert audit.option_fits(option.text) == option.fits_the_wall(), (q.id, option.text)


def _with_source(source: str) -> bank.Question:
    passing = json.loads((FIT_FIXTURES / "passing.json").read_text(encoding="utf-8"))["question"]
    return bank.question_from_dict({**passing, "source": source})


def _fixtures() -> list[tuple[str, dict]]:
    return [
        (path.stem, json.loads(path.read_text(encoding="utf-8")))
        for path in sorted(FIT_FIXTURES.glob("*.json"))
    ]


@pytest.mark.parametrize(("name", "fixture"), _fixtures(), ids=[name for name, _ in _fixtures()])
def test_each_fit_fixture_raises_exactly_its_own_flag(name: str, fixture: dict) -> None:
    question = bank.question_from_dict(fixture["question"])
    assert question.verified is None  # no answer is claimed anywhere in the set
    assert list(audit.question_flags(question)) == fixture["expected_flags"], name


def test_the_fixture_set_is_one_of_each() -> None:
    """EVALUATION AC-100: one too-long program, one too-long option, one passing
    question - so the flag is shown to fire on each and only on those."""
    flags = sorted(tuple(f["expected_flags"]) for _, f in _fixtures())
    assert flags == [(), (audit.FLAG_OPTION,), (audit.FLAG_PROGRAM,)]


def test_the_real_bank_is_flagged_where_its_own_records_say() -> None:
    """bank/README.md: q3 passes D-15 as authored; q4, q7 and q8 need their options
    re-authored. All four fit as programs."""
    flags = {q.id: audit.question_flags(q) for q in bank.load_bank(REPO / "bank")}
    for qid, raised in flags.items():
        assert audit.FLAG_PROGRAM not in raised, qid
        assert (audit.FLAG_OPTION in raised) == bool(bank.options_needing_reauthoring(
            bank.load_question(REPO / "bank", qid).options
        )), qid


# --------------------------------------------------------------------------- #
# Parity with web/shared/typemodel.js
# --------------------------------------------------------------------------- #


def typemodel_constants(js: str) -> dict[str, float]:
    """The numbers typemodel.js declares, read from its source."""
    code = re.sub(r"/\*.*?\*/", "", js, flags=re.S)

    def block(name: str) -> str:
        match = re.search(rf"var {name} = \{{(.*?)\}};", code, flags=re.S)
        assert match, f"typemodel.js has no {name} block"
        return match.group(1)

    found: dict[str, float] = {}
    for name in ("TYPE_CONSTANTS", "ROOM_DEFAULTS"):
        for key, value in re.findall(r"(\w+):\s*(-?\d+(?:\.\d+)?)", block(name)):
            found[key] = float(value)
    for area, w, h in re.findall(r"(reading|trace):\s*\{\s*w:\s*(\d+),\s*h:\s*(\d+)", block("CODE_AREA")):
        found[f"{area}_w"], found[f"{area}_h"] = float(w), float(h)
    return found


def python_constants() -> dict[str, float]:
    room = Room()
    return {
        "CANVAS_H": audit.CANVAS_H,
        "CAP_RATIO": audit.CAP_RATIO,
        "READ_RATIO": audit.READ_RATIO,
        "MAX_FONT": audit.MAX_FONT,
        "LINE_HEIGHT": audit.LINE_HEIGHT,
        "GUTTER_CH": audit.GUTTER_CH,
        "CHAR_ADVANCE": audit.CHAR_ADVANCE,
        "OPTION_MAX_CHARS": audit.OPTION_MAX_CHARS,
        "screen_width_ft": room.screen_width_ft,
        "screen_height_ft": room.screen_height_ft,
        "back_row_ft": room.back_row_ft,
        "reading_w": READING_AREA[0],
        "reading_h": READING_AREA[1],
        "trace_w": TRACE_AREA[0],
        "trace_h": TRACE_AREA[1],
    }


def constant_mismatches(js: str) -> dict[str, tuple[float, float | None]]:
    theirs = typemodel_constants(js)
    return {k: (v, theirs.get(k)) for k, v in python_constants().items() if theirs.get(k) != v}


def test_the_typemodel_reader_reads() -> None:
    """So that the parity test below skipping cannot hide a reader that finds nothing."""
    sample = """
      var ROOM_DEFAULTS = { screen_width_ft: 15, /* a note: 99 */ screen_height_ft: 8.44, back_row_ft: 20 };
      var TYPE_CONSTANTS = { CANVAS_H: 630, CAP_RATIO: 0.7, READ_RATIO: 150, MAX_FONT: 46,
        LINE_HEIGHT: 1.6, GUTTER_CH: 3.5, CHAR_ADVANCE: 0.6, OPTION_MAX_CHARS: 29 };
      var CODE_AREA = {
        reading: { w: 994, h: 177 }, /* live */
        trace:   { w: 994, h: 190 }
      };
    """
    assert constant_mismatches(sample) == {}
    assert constant_mismatches(sample.replace("MAX_FONT: 46", "MAX_FONT: 48")) == {"MAX_FONT": (46, 48.0)}


def test_the_constants_match_typemodel_js() -> None:
    if not TYPEMODEL_JS.is_file():
        pytest.skip(
            "web/shared/typemodel.js arrives with PR #9 (branch ai-c11-cc/shared-web-layer, "
            "BUILDPLAN T-02), which is not in this branch's base yet; the constants are "
            "checked against SPEC 5.2 above until it is"
        )
    assert constant_mismatches(TYPEMODEL_JS.read_text(encoding="utf-8")) == {}
