"""The default club and its old names, held equal in both languages (ADR-0005 item 8).

`bank/fixtures/clubs/aliases.json` is the one statement of the default club and
its aliases. This file holds the pipeline's `bank.DEFAULT_CLUB`, `CLUB_ALIASES`
and `club_slug` to it, and the `justfile`'s `demo` ledger check too;
`room/tests/twins.rs` holds the room's `DEFAULT_CLUB`, `CLUB_ALIASES` and
`ClubSlug::parse` to the same file. A table changed on one side only fails here
or there.
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
from pathlib import Path

import pytest

from popquiz import bank

REPO = Path(__file__).resolve().parents[2]
FIXTURE = json.loads((REPO / "bank" / "fixtures" / "clubs" / "aliases.json").read_text(encoding="utf-8"))


def test_the_default_club_and_its_aliases_are_the_fixtures() -> None:
    assert bank.DEFAULT_CLUB == FIXTURE["default"]
    assert bank.CLUB_ALIASES == FIXTURE["aliases"]
    assert set(FIXTURE["aliases"].values()) == {FIXTURE["default"]}


def test_every_fixture_slug_reads_as_its_club() -> None:
    assert len(FIXTURE["slugs"]) >= 10
    for raw, club in FIXTURE["slugs"]:
        assert bank.club_slug(raw) == club, raw


def _demo_filter() -> str:
    """The jq filter `just demo` runs over the room's used ledger."""
    justfile = (REPO / "justfile").read_text(encoding="utf-8")
    recipe = justfile[justfile.index("\ndemo "):justfile.index("\nsync ")]
    found = re.findall(r"jq -e --arg q \"\{\{ question \}\}\" '([^']+)'", recipe)
    assert len(found) == 1, found
    return found[0]


def _demo_counts(filter_: str, club: str) -> bool:
    jq = shutil.which("jq")
    assert jq, "jq runs the demo recipe's ledger check, so this test needs it too"
    line = {"club": club, "question_id": "q3", "used": {}}
    done = subprocess.run([jq, "-e", "--arg", "q", "q3", filter_], input=json.dumps([line]),
                          capture_output=True, text=True, check=False)
    assert done.returncode in (0, 1), done.stderr
    return done.returncode == 0


@pytest.mark.parametrize("club", [FIXTURE["default"], *FIXTURE["aliases"]])
def test_just_demo_counts_the_default_club_under_every_name(club: str) -> None:
    assert _demo_counts(_demo_filter(), club), f"just demo would push q3 though the room ran it as {club}"


def test_just_demo_ignores_another_clubs_use() -> None:
    assert not _demo_counts(_demo_filter(), "la")
