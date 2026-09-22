"""The slot draw (AC-23, SPEC G-1): a pure function of the date, identical to the MVP's.

`slot_for_day` is the only thing in the build that decides where a correct answer
sits, and it has regressed four times (PHILOSOPHY.md section 2). These tests pin
the three things that make it safe: it draws exactly what the MVP drew, it depends
on the date alone, and it cannot be handed anything else. The static half of AC-23
- that no history can reach it - is `popquiz.audit.slot_path_violations`, tested
in `test_audit.py`.
"""

from __future__ import annotations

import builtins
import datetime
import io
import os
import pathlib
import sys

import pytest

from popquiz.slot import slot_for_day

HERE = pathlib.Path(__file__).parent
REPO = HERE.parent.parent
MVP_DECK = REPO / "mvp" / "tools" / "build_deck.py"


@pytest.fixture(scope="module")
def mvp_deck() -> dict:
    """`mvp/tools/build_deck.py`, loaded read-only.

    Not imported: `importlib` is barred from the suite (`test_runner.py`), and a
    plain import would write a `.pyc` into `mvp/`, which this ticket may not touch.
    The source is compiled and executed into a fresh namespace instead. The module
    reads `sys.argv` when it loads and asserts on what it finds
    (`build_deck.py:23-34`), so `pytest -k anything` would crash it; argv is pinned
    to the bare script path for the duration. `__name__` is not `"__main__"`, so
    its `main()` - the part that writes the ledger - never runs.
    """
    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(sys, "argv", [str(MVP_DECK)])
        namespace = {"__file__": str(MVP_DECK), "__name__": "build_deck_read_only"}
        exec(compile(MVP_DECK.read_text(encoding="utf-8"), str(MVP_DECK), "exec"), namespace)
    return namespace


def every_day(year: int) -> list[datetime.date]:
    first = datetime.date(year, 1, 1)
    days = (datetime.date(year + 1, 1, 1) - first).days
    return [first + datetime.timedelta(days=n) for n in range(days)]


def test_slot_for_day_draws_exactly_what_the_mvp_drew(mvp_deck: dict) -> None:
    """Identical behaviour on identical input, over two years of dates.

    The MVP keyed the draw on the day's ISO string; this takes the `date` and uses
    its ISO string, so every night keeps the letter the MVP would have given it.
    """
    days = every_day(2026) + every_day(2027)
    ours = [slot_for_day(day) for day in days]
    theirs = [mvp_deck["slot_for_day"](day.isoformat(), 5) for day in days]

    assert len(days) == 730
    mismatched = [str(d) for d, a, b in zip(days, ours, theirs) if a != b]
    assert not mismatched, f"slot_for_day disagrees with the MVP on {mismatched[:5]}"


def test_slot_for_day_is_a_letter() -> None:
    assert {slot_for_day(day) for day in every_day(2026)} == {0, 1, 2, 3, 4}


def test_slot_for_day_gives_one_night_one_answer() -> None:
    night = datetime.date(2026, 10, 14)
    assert len({slot_for_day(night) for _ in range(10)}) == 1


def test_slot_for_day_reads_no_file(monkeypatch: pytest.MonkeyPatch) -> None:
    """A pure function of the date cannot need the filesystem. If anything on the
    slot path tried to read a ledger, a history or the bank, this fails."""

    def refuse(*args, **kwargs):
        raise AssertionError("slot_for_day touched the filesystem")

    for owner, name in (
        (builtins, "open"),
        (io, "open"),
        (os, "open"),
        (os, "listdir"),
        (pathlib.Path, "open"),
        (pathlib.Path, "read_text"),
        (pathlib.Path, "read_bytes"),
        (pathlib.Path, "write_text"),
        (pathlib.Path, "exists"),
    ):
        monkeypatch.setattr(owner, name, refuse)

    for day in every_day(2026)[:60]:
        slot_for_day(day)


@pytest.mark.parametrize(
    "not_a_date",
    [
        "2026-08-12",
        datetime.datetime(2026, 8, 12, 19, 30),
        20260812,
        None,
    ],
)
def test_slot_for_day_takes_a_date_and_nothing_else(not_a_date: object) -> None:
    """A string is refused so a caller cannot pass something date-shaped that is
    not a date. A `datetime` is refused because it IS a `date` subclass whose ISO
    form carries the time: the same night would draw different letters at 7:30 and
    at 7:31."""
    with pytest.raises(TypeError, match="datetime.date and nothing else"):
        slot_for_day(not_a_date)  # type: ignore[arg-type]


@pytest.mark.parametrize("n_options", [4, 6, 0])
def test_n_options_is_the_constant_five(n_options: int) -> None:
    """SPEC G-1: `n_options` is 5 and never derived from data. A caller computing it
    from a question - `len(question.options)` - gets an error the day it is not 5,
    rather than a draw from a different distribution."""
    with pytest.raises(ValueError, match="constant 5"):
        slot_for_day(datetime.date(2026, 8, 12), n_options)


def test_n_options_five_is_the_same_draw_as_the_default() -> None:
    night = datetime.date(2026, 8, 12)
    assert slot_for_day(night, 5) == slot_for_day(night)
