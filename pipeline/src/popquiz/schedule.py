"""Push one affirmed question to the room and pull the used ledger back (AC-89 to AC-92, SPEC.md 7.7).

Three subcommands, each opening with the reserve (AC-75, AC-76), because the
organizer's first screen in this build is the first lines these print:

* `schedule <id> --date YYYY-MM-DD --room <url> --out <dir>` - the gate, the
  arrangement, the push, and the static fallback with its host sheet beside it.
* `sync --room <url>` - the room's used ledger into the bank's `used` blocks, then
  the organizer's ledger report.
* `reserve` - the reserve lines alone.

**The arrangement happens here and nowhere else.** The bank stores options in the
generator's order, which means nothing (`bank.py`); the room renders the order it
is pushed (`room/src/question.rs`); the fallback renders the order it is given
(`fallback.py`). So this module is the one place the wall's order is made: the
correct option goes to `slot_for_day(meetup_date)`, a uniform draw from the date
and nothing else (AC-23, G-1), and the record the room receives and the record the
fallback is built from are the same object, so the two can never disagree about
where the answer sits. The stored bank order is never rewritten.

**`used` is written by `sync` alone**, and only from the room's ledger (G-10,
AC-92): a question is used when its room is released, not when it is scheduled or
when a file is built. `sync` hands the room's JSON to `bank.py`'s reader rather
than constructing the record here, so the one place a `used` record is made on
the laptop is the reader that parses what the room wrote (`room/tests/used.rs`).

**The admin token** (SPEC 8.3, AC-101) is read from `POPQUIZ_ADMIN_TOKEN` in the
environment and nowhere else - no flag, no file - and appears in no line this
module prints and no exception it raises. Retries are bounded and backed off:
connection failures and 5xx are retried a fixed small number of times with
growing waits, as are 408 and 429, which mean *not now*; any other 4xx is the
room's answer and is never retried.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import http.client
import json
import os
import re
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from collections.abc import Iterable, Sequence
from datetime import date, datetime, timedelta
from pathlib import Path
from typing import Any
from zoneinfo import ZoneInfo

from popquiz import fallback
from popquiz.audit import in_reserve, is_accepted
from popquiz.bank import (
    CLUB_SLUG,
    DEFAULT_CLUB,
    BankError,
    Question,
    correct_index,
    load_bank,
    load_question,
    question_from_dict,
    question_to_dict,
    save_question,
)
from popquiz.slot import slot_for_day

#: The one place the pipeline reads the admin token from (SPEC 8.3, H-11).
TOKEN_ENV = "POPQUIZ_ADMIN_TOKEN"

REPO = Path(__file__).resolve().parents[3]
DEFAULT_BANK = REPO / "bank"

#: Rust NYC meets in New York; the room dates a meetup the same way (room/src/used.rs).
MEETUP_ZONE = ZoneInfo("America/New_York")

#: SPEC 3.3, AC-76: warn when the reserve is below two meetups. One question a
#: meetup (D-7), so the default threshold in questions equals the lead time.
DEFAULT_LEAD_TIME = 2
QUESTIONS_PER_MEETUP = 1

#: How far back the trend counts questions used.
TREND_DAYS = 90

#: SPEC 8.3: bounded and backed off. Four attempts in all; the waits between them.
RETRY_WAITS = (0.5, 1.0, 2.0)
TIMEOUT_SECONDS = 20

LETTERS = fallback.LETTERS


class _NoRedirects(urllib.request.HTTPRedirectHandler):
    """A redirect is refused, not followed: urllib would carry the bearer to
    wherever a 3xx pointed. The admin routes never redirect."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


# Seams the tests replace. Nothing else in this module reaches the network or sleeps.
_urlopen = urllib.request.build_opener(_NoRedirects).open
_sleep = time.sleep

#: Plain http is for a room on this machine only; anything else is Fly's TLS (SPEC 8.3).
LOOPBACK = frozenset({"127.0.0.1", "localhost", "::1"})
_ISO_DATE = re.compile(r"\d{4}-\d{2}-\d{2}")
_TOKEN_CHARS = re.compile(r"[\x21-\x7e]+")

#: 4xx answers that mean "not now" rather than "no": retried like a 5xx.
RETRIED_4XX = frozenset({408, 429})


class ScheduleError(Exception):
    """A refusal or a failure, in one plain sentence the organizer can act on.

    Never carries the token: every message is built from a status code, a path or
    the room's own `reason`, and nothing from the request that was sent.
    """


# --------------------------------------------------------------------------- #
# The gate (AC-72, G-12, G-10)
# --------------------------------------------------------------------------- #


def refusal(question: Question, club: str = DEFAULT_CLUB) -> str | None:
    """Why this question cannot be scheduled, or `None` when it can.

    A hard error, never a warning: scheduling takes a question from the reserve
    (SPEC 3.3, `audit.in_reserve`: accepted, affirmed, unused) and a verified
    record. This names *which* part is missing, so the organizer knows what to do.

    The rule that a question needs two trace steps to be affirmed is the affirm
    step's (SPEC 7.4, T-18), not this gate's; `fallback.bake` refuses a trace too
    short to walk on its own account.
    """
    qid = question.id
    if question.verified is None:
        return f"{qid} has no verified record, so it cannot be scheduled"
    used = question.used_by(club)
    if used is not None:
        return (
            f"{qid} was already used by {club} on {used.meetup_date} (room "
            f"{used.room_id}); a question is never run twice for a club"
        )
    review = question.review
    missing = [name for name in ("affirmed_by", "affirmed_at") if not getattr(review, name, None)]
    if missing:
        accepted = "" if is_accepted(question) else ", and it has not been accepted at review"
        return (
            f"{qid} is not affirmed - it has no {' and no '.join(missing)}{accepted} - and an "
            "unaffirmed question cannot be scheduled (AC-72)"
        )
    if not is_accepted(question):
        return f"{qid} is affirmed but has not been accepted at review, so it cannot be scheduled"
    if not in_reserve(question, club):  # the checks above are in_reserve, spelled out
        return f"{qid} is not in the reserve, so it cannot be scheduled"
    return None


# --------------------------------------------------------------------------- #
# The arrangement (AC-23, G-1) - pure
# --------------------------------------------------------------------------- #


def _rng(*parts):
    """Deterministic, well-mixed stream. Ported from `build_deck.py:75-84`
    verbatim, kept here and not in `slot.py`, which holds the slot path alone."""
    seed = int(hashlib.blake2b("|".join(map(str, parts)).encode(), digest_size=8).hexdigest(), 16)

    def nxt(bound):
        nonlocal seed
        seed = (seed * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        return (seed >> 33) % bound

    return nxt


def _shuffled(items: Sequence[Any], *seed_parts: str) -> list[Any]:
    """Fisher-Yates on `_rng`, as `build_deck.py`'s `shuffled`."""
    out = list(items)
    nxt = _rng(*seed_parts)
    for i in range(len(out) - 1, 0, -1):
        j = nxt(i + 1)
        out[i], out[j] = out[j], out[i]
    return out


def arrange(question: Question, day: date) -> Question:
    """The question in the wall's order for the meetup on `day`.

    The correct option sits at `slot_for_day(day)` - the date and nothing else.
    The four others are shuffled on (date, question id), so whatever order the
    generator wrote them in cannot become a tell beside the answer. Every input is
    the date and the record: no history, no ledger, no bank state, no previous
    slot. `why_tempting` rides on each option and moves with it.
    """
    slot = slot_for_day(day)
    correct = correct_index(question)
    if correct is None:
        raise BankError(f"{question.id}: no option matches the verified answer, so it cannot be arranged")
    others = [o for i, o in enumerate(question.options) if i != correct]
    options = _shuffled(others, "distractors", day.isoformat(), question.id)
    options.insert(slot, question.options[correct])
    arranged = dataclasses.replace(question, options=tuple(options))
    if correct_index(arranged) != slot:
        raise BankError(f"{question.id}: the arranged answer is not at the slot the date draws")
    return arranged


# --------------------------------------------------------------------------- #
# The channel (SPEC 8.3, AC-101)
# --------------------------------------------------------------------------- #


def admin_token(environ: dict[str, str] | None = None) -> str:
    """The token, stripped. A value that cannot travel in an HTTP header (a stray
    newline inside it, a non-ASCII character) is refused here, without echoing it,
    because `http.client` would otherwise raise with the header in its message."""
    value = (os.environ if environ is None else environ).get(TOKEN_ENV, "").strip()
    if not value:
        raise ScheduleError(
            f"{TOKEN_ENV} is not set in the environment; the room's admin channel needs it "
            "(see pipeline/README.md, Scheduling a meetup)"
        )
    if not _TOKEN_CHARS.fullmatch(value):
        raise ScheduleError(
            f"{TOKEN_ENV} holds characters a token cannot have (spaces, control characters "
            "or non-ASCII); check the value you exported"
        )
    return value


def room_url(room: str, path: str) -> str:
    """`{room}{path}`, refusing a room the token must not be sent to: https, or
    plain http to this machine for a local room."""
    parts = urllib.parse.urlsplit(room)
    if parts.scheme == "https" or (parts.scheme == "http" and parts.hostname in LOOPBACK):
        return f"{room.rstrip('/')}{path}"
    raise ScheduleError(
        f"{room!r} is not an https URL (plain http is for a room on this machine), "
        "and the admin token is sent over TLS only"
    )


def _call(method: str, url: str, token: str, body: bytes | None = None) -> tuple[int, bytes]:
    """One admin request, retried on connection failures and 5xx only.

    Returns the room's status and body for anything that is the room's answer
    (2xx, 4xx). A 3xx is not followed (`_NoRedirects`) and is refused here.
    Raises `ScheduleError` once the retries are spent.
    """
    headers = {"Authorization": f"Bearer {token}", "Accept": "application/json"}
    if body is not None:
        headers["Content-Type"] = "application/json"
    attempts = len(RETRY_WAITS) + 1
    last = ""
    for attempt in range(attempts):
        if attempt:
            _sleep(RETRY_WAITS[attempt - 1])
        request = urllib.request.Request(url, data=body, method=method, headers=headers)
        try:
            with _urlopen(request, timeout=TIMEOUT_SECONDS) as response:
                return response.status, response.read()
        except urllib.error.HTTPError as e:
            status, payload = e.code, e.read() if e.fp is not None else b""
            e.close()
            if 300 <= status < 400:
                raise ScheduleError(
                    f"{method} {url}: the room answered with a redirect ({status}), which is "
                    "not followed with the admin token; give the room's own URL"
                ) from None
            if status < 500 and status not in RETRIED_4XX:
                return status, payload
            last = f"the room answered {status}"
        except (urllib.error.URLError, OSError, http.client.HTTPException) as e:
            # The class name only: a message here may quote what was sent.
            reason = getattr(e, "reason", None)
            last = f"the room could not be reached ({reason if isinstance(reason, str) else type(e).__name__})"
        except (ValueError, UnicodeError) as e:
            # http.client refuses a header it cannot send with the header in its
            # message; the token is in that header, so the message is dropped.
            raise ScheduleError(
                f"{method} {url}: the request could not be sent ({type(e).__name__})"
            ) from None
    raise ScheduleError(f"{method} {url}: {last}, after {attempts} attempts")


def _reason(payload: bytes, fallback_text: str) -> str:
    try:
        reason = json.loads(payload.decode("utf-8")).get("reason")
    except (ValueError, AttributeError, UnicodeDecodeError):
        reason = None
    return reason if isinstance(reason, str) and reason else fallback_text


def _refused_token(status: int) -> ScheduleError:
    return ScheduleError(
        f"the room refused the admin token ({status}); the value in {TOKEN_ENV} is not the room's"
    )


def push(arranged: Question, room: str, token: str, club: str = DEFAULT_CLUB) -> str:
    """`PUT` the arranged record to the room, for `club`: `/admin/questions/{id}`
    for the default club, `/admin/clubs/{club}/questions/{id}` for another (D-26).
    The record is the question arranged for that club's meetup date, so it is kept
    per club and one club's push never replaces another's. Returns `new` or
    `replaced`; raises `ScheduleError` with the room's reason on a refusal."""
    path = f"/admin/questions/{arranged.id}" if club == DEFAULT_CLUB else f"/admin/clubs/{club}/questions/{arranged.id}"
    url = room_url(room, path)
    body = json.dumps(question_to_dict(arranged), ensure_ascii=False).encode("utf-8")
    status, payload = _call("PUT", url, token, body)
    if status in (200, 201):
        try:
            answer = json.loads(payload.decode("utf-8")).get("scheduled")
        except (ValueError, AttributeError, UnicodeDecodeError):
            answer = None
        return answer if answer in ("new", "replaced") else ("new" if status == 201 else "replaced")
    if status == 401:
        raise _refused_token(status)
    if status in (400, 409, 413):
        raise ScheduleError(_reason(payload, f"the room refused the record ({status})").replace(token, "[redacted]"))
    raise ScheduleError(f"the room answered {status} to the push")


def pull_used(room: str, token: str) -> list[dict[str, Any]]:
    """`GET {room}/admin/used`: `[{club, question_id, used: {...}}]`."""
    url = room_url(room, "/admin/used")
    status, payload = _call("GET", url, token)
    if status == 401:
        raise _refused_token(status)
    if status != 200:
        raise ScheduleError(f"the room answered {status} to the ledger request")
    try:
        entries = json.loads(payload.decode("utf-8"))
    except (ValueError, UnicodeDecodeError):
        raise ScheduleError("the room's ledger is not JSON") from None
    if not isinstance(entries, list) or not all(
        isinstance(e, dict) and isinstance(e.get("question_id"), str) and isinstance(e.get("used"), dict)
        for e in entries
    ):
        raise ScheduleError("the room's ledger is not a list of {club, question_id, used} entries")
    return entries


# --------------------------------------------------------------------------- #
# The reserve (AC-75, AC-76, SPEC 3.3)
# --------------------------------------------------------------------------- #


def today_in_new_york() -> date:
    return datetime.now(MEETUP_ZONE).date()


def reserve_lines(
    questions: Iterable[Question],
    *,
    lead_time: int = DEFAULT_LEAD_TIME,
    threshold: int | None = None,
    today: date | None = None,
    club: str = DEFAULT_CLUB,
) -> list[str]:
    """The reserve count, its trend, and the warning when it is low.

    The count is `audit.in_reserve`: accepted, affirmed, unused. The bank keeps no
    series of past counts, so the trend is read from what it does hold: what is
    coming in (accepted and awaiting affirmation, not yet reviewed) against what
    went out (used in the last 90 days). The warning fires when the reserve is
    below the threshold - by default the lead time's worth of meetups at one
    question a meetup - so it comes while there is still a meetup to act in.
    """
    questions = list(questions)
    today = today or today_in_new_york()
    threshold = lead_time * QUESTIONS_PER_MEETUP if threshold is None else threshold
    ready = sum(1 for q in questions if in_reserve(q, club))
    awaiting = sum(1 for q in questions if q.used_by(club) is None and is_accepted(q) and not in_reserve(q, club))
    unreviewed = sum(1 for q in questions if q.used_by(club) is None and (q.review is None or q.review.status is None))
    since = today - timedelta(days=TREND_DAYS)
    used_recently = 0
    for q in questions:
        used = q.used_by(club)
        if used is not None:
            try:
                if date.fromisoformat(used.meetup_date) >= since:
                    used_recently += 1
            except ValueError:
                pass
    runway = ready // QUESTIONS_PER_MEETUP
    lines = [
        f"reserve ({club}): {ready} ready (accepted, affirmed, unused) - "
        f"{runway} meetup{'s' if runway != 1 else ''} of runway at one question a meetup",
        f"trend: +{awaiting} accepted awaiting affirmation, +{unreviewed} not yet reviewed, "
        f"-{used_recently} used in the last {TREND_DAYS} days",
    ]
    if ready < threshold:
        lines.append(
            f"warning: {ready} ready is below the threshold of {threshold} - generate and "
            f"review now; the lead time is {lead_time} meetup{'s' if lead_time != 1 else ''}"
        )
    return lines


# --------------------------------------------------------------------------- #
# Sync (AC-92, SPEC 3.1's used row)
# --------------------------------------------------------------------------- #


@dataclasses.dataclass
class SyncResult:
    written: list[str] = dataclasses.field(default_factory=list)
    unchanged: list[str] = dataclasses.field(default_factory=list)
    skipped: list[str] = dataclasses.field(default_factory=list)
    refused: list[str] = dataclasses.field(default_factory=list)
    report: list[dict[str, Any]] = dataclasses.field(default_factory=list)


def merge_used(bank_dir: Path, entries: Sequence[dict[str, Any]]) -> SyncResult:
    """Write each ledger entry's `used` onto its bank record, under its club.

    A question is used *per club* (D-26, AC-105): the key is `(club, question)`.
    Idempotent: a record already holding `used` for that club from the same room
    is left as it is. A club's record the ledger names with a different room - or
    the ledger naming one question under two rooms for one club - is *never twice*
    broken: it is refused, nothing is written to it, and the rest carry on. Another
    club's record is never touched. Ids not in the bank (the CI harness's
    `smoke-q3`, `burst-q3`) are listed and skipped. An entry from a room that
    predates clubs names none, and is the default club's.

    The record is merged at the dict level and read back through
    `bank.question_from_dict`, so the room's JSON is parsed by the bank's one
    reader and this module builds no `used` record of its own.
    """
    result = SyncResult()
    by_key: dict[tuple[str, str], list[dict[str, Any]]] = {}
    for entry in entries:
        club = entry.get("club", DEFAULT_CLUB)
        by_key.setdefault((club, entry["question_id"]), []).append(entry["used"])

    known = {q.id: q for q in load_bank(bank_dir)}
    for (club, qid), records in by_key.items():
        question = known.get(qid)
        if question is None:
            if qid not in result.skipped:
                result.skipped.append(qid)
            continue
        rooms = sorted({str(r.get("room_id")) for r in records})
        if len(rooms) > 1:
            result.refused.append(
                f"{qid}: the room's ledger has it released by {len(rooms)} rooms for {club} "
                f"({', '.join(rooms)}); a question is never run twice for a club, so nothing was written"
            )
            continue
        incoming = {**records[0], "club": club}
        held = question.used_by(club)
        if held is not None:
            if held.room_id == incoming.get("room_id"):
                stored = next(u for u in question_to_dict(question)["used"] if u["club"] == club)
                if stored != {k: v for k, v in incoming.items() if v is not None}:
                    result.refused.append(
                        f"{qid}: the bank's used record for {club}, room {held.room_id}, differs "
                        "from the room's ledger for the same room; the record is written once, "
                        "so nothing was changed"
                    )
                    continue
                result.unchanged.append(qid)
                result.report.append(_report_row(question, club))
                continue
            result.refused.append(
                f"{qid}: the bank already records it used by {club} in room {held.room_id} on "
                f"{held.meetup_date}, and the room's ledger says room "
                f"{incoming.get('room_id')}; a question is never run twice for a club, so nothing was written"
            )
            continue
        data = question_to_dict(question)
        data["used"] = [*data.get("used", []), incoming]
        try:
            merged = question_from_dict(data)
        except (BankError, KeyError, TypeError) as e:
            result.refused.append(f"{qid}: the room's used record does not read as one ({e})")
            continue
        save_question(bank_dir, merged)
        known[qid] = merged
        result.written.append(qid)
        result.report.append(_report_row(merged, club))
    return result


def _report_row(question: Question, club: str) -> dict[str, Any]:
    used = question.used_by(club)
    return {
        "club": club,
        "question_id": question.id,
        "meetup_date": used.meetup_date,
        "room_id": used.room_id,
        "fit": used.fit,
    }


def ledger_report(result: SyncResult) -> list[str]:
    """SPEC 3.1: every synced record, and any room whose fit was not *fits*."""
    lines: list[str] = []
    if not result.report:
        lines.append("ledger: the room has released no bank question since it last started")
    for r in result.report:
        lines.append(f"used: {r['question_id']} by {r['club']} on {r['meetup_date']}, room {r['room_id']}")
    for r in result.report:
        if r["fit"] is None:
            lines.append(
                f"fit: room {r['room_id']} ({r['question_id']}) - the wall never reported a verdict"
            )
        elif r["fit"] != "fits":
            lines.append(f"fit: room {r['room_id']} ({r['question_id']}) - {r['fit']}")
    for qid in result.skipped:
        lines.append(f"skipped: {qid} is not a bank question (not in the bank)")
    return lines


# --------------------------------------------------------------------------- #
# The command line
# --------------------------------------------------------------------------- #


def _print_reserve(bank_dir: Path, args: argparse.Namespace) -> None:
    for line in reserve_lines(load_bank(bank_dir), lead_time=args.lead_time, threshold=args.threshold, club=args.club):
        print(line)


def home_link_for(club: str, explicit: str | None = None) -> str:
    """The take-it-home link a club's fallback shows: its own page, `/last/{club}`;
    the default club's is `/last`. An explicit `--home-link` wins (D-26)."""
    if explicit:
        return explicit
    return fallback.DEFAULT_HOME_LINK if club == DEFAULT_CLUB else f"{fallback.DEFAULT_HOME_LINK}/{club}"


def _schedule(args: argparse.Namespace) -> int:
    if not CLUB_SLUG.fullmatch(args.club):
        raise ScheduleError(f"{args.club!r} is not a club name (a-z, 0-9 and -, 1 to 24)")
    home_link = home_link_for(args.club, args.home_link)
    question = load_question(args.bank, args.question_id)
    why = refusal(question, args.club)
    if why:
        raise ScheduleError(why)
    out = Path(args.out).resolve()
    if out == REPO or REPO in out.parents:
        raise ScheduleError(
            f"{args.out} is inside the repository; the fallback and its sheet hold the answer, "
            "so they are written outside it"
        )
    token = None
    if not args.no_push:
        room_url(args.room, "")
        token = admin_token()
    arranged = arrange(question, args.date)
    # Render both files before anything is sent, so a record the fallback refuses
    # is refused before the room ever sees it.
    fallback.build_html(arranged, home_link=home_link)
    fallback.host_sheet(arranged)

    answer = "not pushed (--no-push)"
    if token is not None:
        answer = f"scheduled: {push(arranged, args.room, token, args.club)}"
    try:
        page, sheet = fallback.write_fallback(
            arranged, Path(args.out) / f"{arranged.id}.html", home_link=home_link
        )
    except OSError as e:
        held = "the room holds the record, but " if token is not None else ""
        raise ScheduleError(
            f"{held}the fallback and host sheet were not written to {args.out} "
            f"({type(e).__name__}: {e.strerror or 'write failed'})"
        ) from None
    print(f"{arranged.id} for {args.date.isoformat()}")
    print(answer)
    print(page)
    print(sheet)
    return 0


def _sync(args: argparse.Namespace) -> int:
    entries = pull_used(args.room, admin_token())
    result = merge_used(args.bank, entries)
    for line in ledger_report(result):
        print(line)
    for line in result.refused:
        print(f"popquiz.schedule: refused {line}", file=sys.stderr)
    return 1 if result.refused else 0


def _meetup_date(text: str) -> date:
    if not _ISO_DATE.fullmatch(text):
        raise argparse.ArgumentTypeError(f"{text!r} is not a date in YYYY-MM-DD form")
    try:
        return date.fromisoformat(text)
    except ValueError:
        raise argparse.ArgumentTypeError(f"{text!r} is not a date in YYYY-MM-DD form") from None


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="popquiz.schedule",
        description="Schedule one question for a meetup, and sync the used ledger back (SPEC 7.7, 8.3).",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    def common(p: argparse.ArgumentParser) -> None:
        p.add_argument("--bank", type=Path, default=DEFAULT_BANK, help="the bank directory")
        p.add_argument("--club", default=DEFAULT_CLUB,
                       help="the club the question is for (default nyc): never twice is per club (D-26)")
        p.add_argument("--lead-time", type=int, default=DEFAULT_LEAD_TIME,
                       help="meetups of warning before the reserve runs out (default 2)")
        p.add_argument("--threshold", type=int, default=None,
                       help="warn below this many ready questions (default: the lead time)")

    s = sub.add_parser("schedule", help="push one affirmed question and write its fallback")
    s.add_argument("question_id")
    s.add_argument("--date", type=_meetup_date, required=True, help="the meetup's date, YYYY-MM-DD")
    where = s.add_mutually_exclusive_group(required=True)
    where.add_argument("--room", help="the room's base URL")
    where.add_argument("--no-push", action="store_true",
                       help="write the fallback and host sheet only, for a night the room cannot be reached")
    s.add_argument("--out", type=Path, required=True, help="the directory the fallback and sheet go in")
    s.add_argument("--home-link", default=None,
                   help="the take-it-home link (default: the club's own page, /last for nyc, /last/<club> otherwise)")
    common(s)

    y = sub.add_parser("sync", help="pull the room's used ledger into the bank")
    y.add_argument("--room", required=True, help="the room's base URL")
    common(y)

    r = sub.add_parser("reserve", help="print the reserve count, trend and warning")
    common(r)

    args = parser.parse_args(argv)
    try:
        _print_reserve(args.bank, args)
        if args.command == "schedule":
            return _schedule(args)
        if args.command == "sync":
            return _sync(args)
        return 0
    except (ScheduleError, BankError) as e:
        print(f"popquiz.schedule: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
