"""The default club and its old names, held equal in both languages (ADR-0005 item 8).

`bank/fixtures/clubs/aliases.json` is the one statement of the default club and
its aliases. This file holds the pipeline's `bank.DEFAULT_CLUB`, `CLUB_ALIASES`
and `club_slug` to it, and the `justfile`'s `demo` ledger filter too;
`room/tests/twins.rs` holds the room's `DEFAULT_CLUB`, `CLUB_ALIASES` and
`ClubSlug::parse` to the same file. A table changed on one side only fails here
or there.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

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


def test_just_demo_counts_the_default_club_under_every_name_and_no_other() -> None:
    """`just demo` refuses a question the room's ledger has for the default
    club. A room on the build before the rename names it `nyc`, so the filter
    must count every name the default club has had, and only those: another
    club's use of a question is not this club's (AC-105). Read from the filter
    itself, as `just test` starts no process to run jq."""
    filter_ = _demo_filter()
    assert re.fullmatch(r'any\(\.\[\]; \((\.club == "[a-z0-9-]+")( or \.club == "[a-z0-9-]+")*\) and \.question_id == \$q\)',
                        filter_), filter_
    named = set(re.findall(r'\.club == "([a-z0-9-]+)"', filter_))
    assert named == {FIXTURE["default"], *FIXTURE["aliases"]}, filter_
