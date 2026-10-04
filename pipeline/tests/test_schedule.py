"""`popquiz schedule` and `popquiz sync` (T-20): the gate, the date's arrangement,
the push, the fallback beside it, the used ledger pulled back, and the reserve.

Every test runs on a temporary copy of the committed bank. The committed records
are unaffirmed, so the happy path affirms q3 - the one record with a trace - in the
copy, never in `bank/questions/`. The room is a fake `urlopen` that records what it
was sent; nothing here reaches a network.
"""

from __future__ import annotations

import ast
import io
import json
import shutil
import urllib.error
from datetime import date, timedelta
from pathlib import Path

import pytest

from popquiz import audit, fallback, schedule
from popquiz.bank import correct_index, load_question, question_to_dict
from popquiz.slot import slot_for_day

REPO = Path(__file__).resolve().parents[2]
BANK = REPO / "bank"

#: A stand-in token. Plain words, so the repository's secret scan has nothing to find.
PLANT = "plant-for-the-schedule-tests-only"

NIGHT = date(2026, 10, 14)


# --------------------------------------------------------------------------- #
# Fixtures
# --------------------------------------------------------------------------- #


def affirm(bank_dir: Path, qid: str, **review) -> None:
    path = bank_dir / "questions" / f"{qid}.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    fields = {"status": "accepted", "affirmed_by": "fixture-organizer", "affirmed_at": "2026-10-04T15:00:00Z"}
    fields.update(review)
    data["review"] = {k: v for k, v in {**data.get("review", {}), **fields}.items() if v is not None}
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


@pytest.fixture
def bank(tmp_path) -> Path:
    shutil.copytree(BANK / "questions", tmp_path / "bank" / "questions")
    return tmp_path / "bank"


@pytest.fixture
def affirmed(bank) -> Path:
    affirm(bank, "q3")
    return bank


def snapshot(directory: Path) -> dict[str, bytes]:
    return {str(p.relative_to(directory)): p.read_bytes() for p in sorted(directory.rglob("*")) if p.is_file()}


class Response:
    def __init__(self, status: int, body: bytes):
        self.status = status
        self._body = body

    def read(self) -> bytes:
        return self._body

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False


class FakeRoom:
    """Answers each request with the next scripted reply and records the request."""

    def __init__(self, *replies):
        self.replies = list(replies)
        self.requests = []

    def __call__(self, request, timeout=None):
        self.requests.append(request)
        status, body = self.replies.pop(0) if len(self.replies) > 1 else self.replies[0]
        if isinstance(status, Exception):
            raise status
        payload = body if isinstance(body, bytes) else json.dumps(body).encode()
        if status >= 300:
            raise urllib.error.HTTPError(request.full_url, status, "refused", {}, io.BytesIO(payload))  # urllib with no redirect handler raises on 3xx too
        return Response(status, payload)


@pytest.fixture
def room(monkeypatch):
    sleeps = []
    monkeypatch.setattr(schedule, "_sleep", sleeps.append)
    monkeypatch.setenv(schedule.TOKEN_ENV, PLANT)

    def install(*replies) -> FakeRoom:
        fake = FakeRoom(*replies)
        fake.sleeps = sleeps
        monkeypatch.setattr(schedule, "_urlopen", fake)
        return fake

    return install


def run(*argv) -> int:
    return schedule.main([str(a) for a in argv])


def schedule_q3(bank, out, day=NIGHT, *extra):
    return run("schedule", "q3", "--date", day.isoformat(), "--room", "https://room.test", "--out", out, "--bank", bank, *extra)


# --------------------------------------------------------------------------- #
# AC-23, G-1 - the answer sits where the date puts it, and nowhere else decides
# --------------------------------------------------------------------------- #


def _dates_with_slots():
    """Consecutive nights, enough to cover all five slots."""
    found, day = {}, date(2026, 1, 1)
    while len(found) < 5:
        found.setdefault(slot_for_day(day), day)
        day += timedelta(days=1)
    return found


def test_ac23_the_correct_option_sits_at_the_dates_slot(affirmed):
    q3 = load_question(affirmed, "q3")
    seen = set()
    for n in range(120):
        day = date(2026, 1, 1) + timedelta(days=n)
        arranged = schedule.arrange(q3, day)
        assert correct_index(arranged) == slot_for_day(day)
        assert fallback.bake(arranged)["correct"] == "ABCDE"[slot_for_day(day)]
        seen.add(correct_index(arranged))
    assert seen == {0, 1, 2, 3, 4}


def test_ac23_two_dates_give_two_arrangements_of_one_question(affirmed):
    q3 = load_question(affirmed, "q3")
    slots = _dates_with_slots()
    a, b = schedule.arrange(q3, slots[0]), schedule.arrange(q3, slots[3])
    assert [o.text for o in a.options] != [o.text for o in b.options]
    assert correct_index(a) == 0 and correct_index(b) == 3


def test_ac23_the_arrangement_is_pure_and_keeps_every_option_and_its_beat(affirmed):
    q3 = load_question(affirmed, "q3")
    first, again = schedule.arrange(q3, NIGHT), schedule.arrange(q3, NIGHT)
    assert first == again
    assert sorted(first.options, key=lambda o: o.text) == sorted(q3.options, key=lambda o: o.text)
    # Only the order changes: the rest of the record is the record.
    assert {k: v for k, v in question_to_dict(first).items() if k != "options"} == {
        k: v for k, v in question_to_dict(q3).items() if k != "options"
    }


def test_ac23_the_stored_order_is_never_rewritten(affirmed, room, tmp_path):
    room((201, {"id": "q3", "scheduled": "new"}))
    before = snapshot(affirmed)
    assert schedule_q3(affirmed, tmp_path / "out") == 0
    assert snapshot(affirmed) == before


def test_ac23_the_schedule_module_reads_no_history_and_passes_the_slot_lints():
    source = Path(schedule.__file__).read_text(encoding="utf-8")
    tree = ast.parse(source)
    modules = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            modules.update(a.name for a in node.names)
        elif isinstance(node, ast.ImportFrom):
            modules.add(node.module or "")
    assert not any(m.split(".")[0] in {"mvp", "build_deck"} for m in modules)
    assert "load_history" not in source and "history.json" not in source
    sources = audit.pipeline_sources()
    assert audit.ledger_violations(sources) == []
    assert audit.slot_call_violations(sources) == []


def test_ac23_the_slot_path_is_still_the_slot_path():
    """slot.py is untouched by this ticket (`git diff origin/main -- pipeline/src/popquiz/slot.py`
    is empty - checked at the exit gate, since the test suite starts no process); here, the
    lint that keeps it the date and nothing else still passes."""
    assert audit.slot_path_violations(audit.slot_source()) == []


# --------------------------------------------------------------------------- #
# AC-72, G-12, G-10 - the gate is a hard error
# --------------------------------------------------------------------------- #


def assert_refused(code, capsys, fake, out, needle):
    err = capsys.readouterr().err
    assert code == 1
    assert needle in err
    assert fake.requests == []
    assert not out.exists()


def test_ac72_an_unaffirmed_question_is_refused_and_nothing_is_sent_or_written(bank, room, tmp_path, capsys):
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert_refused(schedule_q3(bank, out), capsys, fake, out, "unaffirmed question cannot be scheduled")


def test_ac72_affirmed_by_without_affirmed_at_is_refused(bank, room, tmp_path, capsys):
    affirm(bank, "q3", affirmed_at=None)
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert_refused(schedule_q3(bank, out), capsys, fake, out, "no affirmed_at")


def test_ac72_affirmed_at_without_affirmed_by_is_refused(bank, room, tmp_path, capsys):
    affirm(bank, "q3", affirmed_by=None)
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert_refused(schedule_q3(bank, out), capsys, fake, out, "no affirmed_by")


def test_ac72_an_affirmed_question_not_accepted_is_refused(bank, room, tmp_path, capsys):
    affirm(bank, "q3", status="rejected")
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert_refused(schedule_q3(bank, out), capsys, fake, out, "not been accepted")


def test_g10_a_used_question_is_refused(affirmed, room, tmp_path, capsys):
    path = affirmed / "questions" / "q3.json"
    data = json.loads(path.read_text())
    data["used"] = {"meetup_date": "2026-09-09", "room_id": "r-1", "released_at": "2026-09-10T01:40:00Z", "fit": "fits"}
    path.write_text(json.dumps(data))
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert_refused(schedule_q3(affirmed, out), capsys, fake, out, "never run twice")


def test_g12_a_question_with_no_verified_record_is_refused(affirmed, room, tmp_path, capsys):
    path = affirmed / "questions" / "q3.json"
    data = json.loads(path.read_text())
    del data["verified"]
    path.write_text(json.dumps(data))
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert_refused(schedule_q3(affirmed, out), capsys, fake, out, "no verified record")


def test_ac72_every_committed_record_is_refused_today(room, tmp_path, capsys):
    """The committed bank holds no affirmed question yet. The day one is affirmed
    this test fails, and that is the news it exists to carry."""
    fake = room((201, {"scheduled": "new"}))
    for qid in ("q3", "q4", "q7", "q8"):
        out = tmp_path / qid
        code = run("schedule", qid, "--date", NIGHT.isoformat(), "--room", "https://room.test", "--out", out, "--bank", BANK)
        assert_refused(code, capsys, fake, out, "unaffirmed question cannot be scheduled")
    assert [q.id for q in schedule.load_bank(BANK) if schedule.refusal(q) is None] == []


def test_g12_the_gate_agrees_with_the_reserve(affirmed):
    for q in schedule.load_bank(affirmed):
        assert (schedule.refusal(q) is None) == audit.in_reserve(q)


# --------------------------------------------------------------------------- #
# AC-101, the laptop's side of the channel
# --------------------------------------------------------------------------- #


def test_ac101_the_push_carries_the_bearer_and_exactly_the_arranged_record(affirmed, room, tmp_path, capsys):
    fake = room((201, {"id": "q3", "scheduled": "new"}))
    assert schedule_q3(affirmed, tmp_path / "out") == 0
    [request] = fake.requests
    assert request.get_method() == "PUT"
    assert request.full_url == "https://room.test/admin/questions/q3"
    assert request.get_header("Authorization") == f"Bearer {PLANT}"
    expected = schedule.arrange(load_question(affirmed, "q3"), NIGHT)
    assert json.loads(request.data) == question_to_dict(expected)
    assert "scheduled: new" in capsys.readouterr().out


def test_ac101_a_re_push_reads_replaced(affirmed, room, tmp_path, capsys):
    room((200, {"id": "q3", "scheduled": "replaced"}))
    assert schedule_q3(affirmed, tmp_path / "out") == 0
    assert "scheduled: replaced" in capsys.readouterr().out


def test_ac101_a_401_is_reported_without_echoing_what_was_sent(affirmed, room, tmp_path, capsys):
    fake = room((401, b""))
    assert schedule_q3(affirmed, tmp_path / "out") == 1
    captured = capsys.readouterr()
    assert "refused the admin token" in captured.err
    assert PLANT not in captured.out + captured.err
    assert "source" not in captured.err and "verified" not in captured.err
    assert len(fake.requests) == 1


@pytest.mark.parametrize("status", [400, 409, 413])
def test_ac101_the_rooms_reason_reaches_stderr_verbatim(affirmed, room, tmp_path, capsys, status):
    reason = "The question q3 has been run; it is never run twice."
    fake = room((status, {"reason": reason}))
    out = tmp_path / "out"
    assert schedule_q3(affirmed, out) == 1
    assert reason in capsys.readouterr().err
    assert len(fake.requests) == 1, "a 4xx is the room's answer and is never retried"
    assert not out.exists(), "a refused push leaves nothing behind"


def test_ac101_5xx_is_retried_a_bounded_number_of_times_with_growing_waits(affirmed, room, tmp_path, capsys):
    fake = room((503, b""))
    assert schedule_q3(affirmed, tmp_path / "out") == 1
    assert len(fake.requests) == len(schedule.RETRY_WAITS) + 1
    assert fake.sleeps == list(schedule.RETRY_WAITS)
    assert fake.sleeps == sorted(fake.sleeps) and len(set(fake.sleeps)) == len(fake.sleeps)
    assert "after 4 attempts" in capsys.readouterr().err


def test_ac101_a_connection_failure_is_retried_then_succeeds(affirmed, room, tmp_path, capsys):
    fake = room((urllib.error.URLError("connection refused"), None), (201, {"scheduled": "new"}))
    assert schedule_q3(affirmed, tmp_path / "out") == 0
    assert len(fake.requests) == 2
    assert "scheduled: new" in capsys.readouterr().out


def test_ac101_the_token_appears_in_no_output_and_no_error(affirmed, room, tmp_path, capsys):
    for replies in ([(401, b"")], [(409, {"reason": "held"})], [(500, b"")], [(OSError("down"), None)], [(201, {"scheduled": "new"})]):
        room(*replies)
        schedule_q3(affirmed, tmp_path / "out")
        captured = capsys.readouterr()
        assert PLANT not in captured.out + captured.err
    room((500, b""))
    with pytest.raises(schedule.ScheduleError) as caught:
        schedule.push(schedule.arrange(load_question(affirmed, "q3"), NIGHT), "https://room.test", PLANT)
    assert PLANT not in str(caught.value) and PLANT not in repr(caught.value)


def test_ac101_no_token_in_the_environment_is_refused_before_anything(affirmed, room, monkeypatch, tmp_path, capsys):
    fake = room((201, {"scheduled": "new"}))
    monkeypatch.delenv(schedule.TOKEN_ENV)
    out = tmp_path / "out"
    assert schedule_q3(affirmed, out) == 1
    assert schedule.TOKEN_ENV in capsys.readouterr().err
    assert fake.requests == [] and not out.exists()


# --------------------------------------------------------------------------- #
# AC-102, T-20's half - the fallback and the room get the same arranged record
# --------------------------------------------------------------------------- #


def test_ac102_one_call_writes_the_file_and_its_sheet_from_the_arranged_record(affirmed, room, tmp_path, capsys):
    fake = room((201, {"scheduled": "new"}))
    for day in _dates_with_slots().values():
        out = tmp_path / day.isoformat()
        assert schedule_q3(affirmed, out, day) == 0
        page, sheet = out / "q3.html", out / "q3.host-sheet.txt"
        assert page.is_file() and sheet.is_file()
        arranged = schedule.arrange(load_question(affirmed, "q3"), day)
        assert page.read_text() == fallback.build_html(arranged)
        assert sheet.read_text() == fallback.host_sheet(arranged)
        pushed = json.loads(fake.requests[-1].data)
        assert pushed["options"] == question_to_dict(arranged)["options"]
        assert fallback.bake(arranged)["correct"] == "ABCDE"[slot_for_day(day)]
        printed = capsys.readouterr().out
        assert str(page) in printed and str(sheet) in printed


def test_ac102_no_push_writes_the_files_and_sends_nothing(affirmed, room, monkeypatch, tmp_path):
    fake = room((201, {"scheduled": "new"}))
    monkeypatch.delenv(schedule.TOKEN_ENV)
    out = tmp_path / "out"
    code = run("schedule", "q3", "--date", NIGHT.isoformat(), "--no-push", "--out", out, "--bank", affirmed)
    assert code == 0
    assert fake.requests == []
    arranged = schedule.arrange(load_question(affirmed, "q3"), NIGHT)
    assert (out / "q3.html").read_text() == fallback.build_html(arranged)


# --------------------------------------------------------------------------- #
# AC-92, G-10 - only sync writes `used`, and only from the room's ledger
# --------------------------------------------------------------------------- #


def test_ac92_schedule_writes_nothing_under_the_bank(affirmed, room, tmp_path):
    room((201, {"scheduled": "new"}))
    before = snapshot(affirmed)
    assert schedule_q3(affirmed, tmp_path / "out") == 0
    assert run("schedule", "q3", "--date", NIGHT.isoformat(), "--no-push", "--out", tmp_path / "o2", "--bank", affirmed) == 0
    assert snapshot(affirmed) == before
    assert load_question(affirmed, "q3").used is None


# --------------------------------------------------------------------------- #
# sync
# --------------------------------------------------------------------------- #


def ledger(*entries):
    return [
        {"question_id": qid, "used": {"meetup_date": d, "room_id": r, "released_at": f"{d}T01:40:00Z", "fit": fit}}
        for qid, d, r, fit in entries
    ]


def sync(bank):
    return run("sync", "--room", "https://room.test", "--bank", bank)


def test_sync_writes_used_from_the_ledger_and_a_null_fit_is_an_absent_key(bank, room, capsys):
    room((200, ledger(("q3", "2026-10-14", "room-a", None), ("q4", "2026-10-15", "room-b", "clipped_x"))))
    assert sync(bank) == 0
    q3 = json.loads((bank / "questions" / "q3.json").read_text())
    assert q3["used"] == {"meetup_date": "2026-10-14", "room_id": "room-a", "released_at": "2026-10-14T01:40:00Z"}
    assert load_question(bank, "q3").used.fit is None
    assert load_question(bank, "q4").used.fit == "clipped_x"
    out = capsys.readouterr().out
    assert "used: q3 on 2026-10-14, room room-a" in out
    assert "fit: room room-a (q3) - the wall never reported a verdict" in out
    assert "fit: room room-b (q4) - clipped_x" in out


def test_sync_lists_and_skips_harness_ids(bank, room, capsys):
    room((200, ledger(("smoke-q3", "2026-10-14", "room-s", "fits"), ("burst-q3", "2026-10-14", "room-t", None))))
    before = snapshot(bank)
    assert sync(bank) == 0
    out = capsys.readouterr().out
    assert "skipped: smoke-q3" in out and "skipped: burst-q3" in out
    assert snapshot(bank) == before


def test_sync_twice_changes_no_file(bank, room):
    room((200, ledger(("q3", "2026-10-14", "room-a", "fits"))))
    assert sync(bank) == 0
    after_first = snapshot(bank)
    assert sync(bank) == 0
    assert snapshot(bank) == after_first


def test_sync_refuses_a_second_room_for_one_question_and_carries_on(bank, room, capsys):
    room((200, ledger(("q3", "2026-10-14", "room-a", "fits"))))
    assert sync(bank) == 0
    q3_before = (bank / "questions" / "q3.json").read_bytes()
    room((200, ledger(("q3", "2026-11-11", "room-z", "fits"), ("q4", "2026-11-11", "room-y", "fits"))))
    assert sync(bank) == 1
    assert "never run twice" in capsys.readouterr().err
    assert (bank / "questions" / "q3.json").read_bytes() == q3_before
    assert load_question(bank, "q4").used.room_id == "room-y"


def test_sync_refuses_one_question_released_by_two_rooms_in_one_ledger(bank, room, capsys):
    room((200, ledger(("q3", "2026-10-14", "room-a", "fits"), ("q3", "2026-10-15", "room-b", "fits"))))
    before = snapshot(bank)
    assert sync(bank) == 1
    assert "2 rooms" in capsys.readouterr().err
    assert snapshot(bank) == before


def test_sync_says_so_when_the_ledger_is_empty(bank, room, capsys):
    room((200, []))
    assert sync(bank) == 0
    assert "released no bank question" in capsys.readouterr().out


def test_sync_a_401_is_reported_without_the_token(bank, room, capsys):
    room((401, b""))
    assert sync(bank) == 1
    err = capsys.readouterr().err
    assert "refused the admin token" in err and PLANT not in err


# --------------------------------------------------------------------------- #
# AC-75, AC-76 - the reserve, first
# --------------------------------------------------------------------------- #


def test_ac75_the_count_is_what_in_reserve_admits(bank):
    for qid in ("q3", "q4"):
        affirm(bank, qid)
    questions = schedule.load_bank(bank)
    expected = sum(1 for q in questions if audit.in_reserve(q))
    assert expected == 2
    assert schedule.reserve_lines(questions, today=NIGHT)[0].startswith(f"reserve: {expected} ready")


def test_ac75_the_reserve_and_trend_are_the_first_lines_of_every_command(affirmed, room, tmp_path, capsys):
    room((201, {"scheduled": "new"}))
    expected = schedule.reserve_lines(schedule.load_bank(affirmed))
    schedule_q3(affirmed, tmp_path / "out")
    assert capsys.readouterr().out.splitlines()[: len(expected)] == expected
    room((200, []))
    sync(affirmed)
    assert capsys.readouterr().out.splitlines()[: len(expected)] == expected
    run("reserve", "--bank", affirmed)
    assert capsys.readouterr().out.splitlines() == expected
    assert expected[1].startswith("trend: ")


def test_ac75_the_trend_counts_what_comes_in_and_what_went_out(bank):
    affirm(bank, "q4", affirmed_by=None, affirmed_at=None)  # accepted, awaiting affirmation
    path = bank / "questions" / "q7.json"
    data = json.loads(path.read_text())
    data["used"] = {"meetup_date": "2026-09-09", "room_id": "r", "released_at": "2026-09-10T01:00:00Z"}
    path.write_text(json.dumps(data))
    trend = schedule.reserve_lines(schedule.load_bank(bank), today=NIGHT)[1]
    assert trend == "trend: +1 accepted awaiting affirmation, +2 not yet reviewed, -1 used in the last 90 days"


def test_ac76_the_warning_fires_below_the_threshold_and_not_at_it(bank):
    affirm(bank, "q3")
    one = schedule.reserve_lines(schedule.load_bank(bank), today=NIGHT)
    assert any(line.startswith("warning:") for line in one)
    affirm(bank, "q4")
    two = schedule.reserve_lines(schedule.load_bank(bank), today=NIGHT)
    assert not any(line.startswith("warning:") for line in two)


def test_ac76_the_default_lead_time_is_two_meetups_and_both_are_configurable(bank):
    assert schedule.DEFAULT_LEAD_TIME == 2
    affirm(bank, "q3")
    affirm(bank, "q4")
    questions = schedule.load_bank(bank)
    assert any(line.startswith("warning:") for line in schedule.reserve_lines(questions, lead_time=3, today=NIGHT))
    assert any(line.startswith("warning:") for line in schedule.reserve_lines(questions, threshold=5, today=NIGHT))
    assert not any(line.startswith("warning:") for line in schedule.reserve_lines(questions, threshold=1, today=NIGHT))


# --------------------------------------------------------------------------- #
# Where the token may go, and where the files may not
# --------------------------------------------------------------------------- #


def test_ac101_a_redirect_is_refused_and_not_followed(affirmed, room, tmp_path, capsys):
    fake = room((307, b""))
    out = tmp_path / "out"
    assert schedule_q3(affirmed, out) == 1
    assert "redirect" in capsys.readouterr().err
    assert len(fake.requests) == 1 and not out.exists()


def test_ac101_the_real_opener_follows_no_redirect():
    opener = getattr(schedule._urlopen, "__self__", None)
    assert opener is not None and any(isinstance(h, schedule._NoRedirects) for h in opener.handlers), (
        "the module's opener must be the one that follows no redirect"
    )
    handler = schedule._NoRedirects()
    assert handler.redirect_request(None, None, 302, "Found", {}, "https://elsewhere.test/") is None


@pytest.mark.parametrize("url", ["http://room.example", "ftp://room.test", "room.test"])
def test_ac101_the_token_goes_to_https_or_to_this_machine_only(affirmed, room, tmp_path, capsys, url):
    fake = room((201, {"scheduled": "new"}))
    code = run("schedule", "q3", "--date", NIGHT.isoformat(), "--room", url, "--out", tmp_path / "o", "--bank", affirmed)
    assert code == 1
    assert "https" in capsys.readouterr().err
    assert fake.requests == []


def test_ac101_plain_http_to_a_local_room_is_allowed():
    assert schedule.room_url("http://127.0.0.1:8080/", "/admin/used") == "http://127.0.0.1:8080/admin/used"
    assert schedule.room_url("https://rustnyc-popquiz.fly.dev", "/admin/used") == "https://rustnyc-popquiz.fly.dev/admin/used"


def test_ac102_the_fallback_is_never_written_inside_the_repository(affirmed, room, capsys):
    fake = room((201, {"scheduled": "new"}))
    out = REPO / "pipeline" / "never-written-here"
    code = run("schedule", "q3", "--date", NIGHT.isoformat(), "--room", "https://room.test", "--out", out, "--bank", affirmed)
    assert code == 1
    assert "inside the repository" in capsys.readouterr().err
    assert fake.requests == [] and not out.exists()


@pytest.mark.parametrize("text", ["20261014", "2026-W42-3", "2026-10-14T19:00", "14/10/2026"])
def test_the_date_is_yyyy_mm_dd_and_nothing_looser(text, capsys):
    with pytest.raises(SystemExit):
        run("schedule", "q3", "--date", text, "--no-push", "--out", "/tmp/x")
    assert "YYYY-MM-DD" in capsys.readouterr().err


def test_the_ported_rng_is_the_slot_modules_stream():
    from popquiz import slot

    for parts in (("a",), ("distractors", "2026-10-14", "q3"), ("meetup-slot", "2026-10-14")):
        mine, theirs = schedule._rng(*parts), slot._rng(*parts)
        assert [mine(5) for _ in range(20)] == [theirs(5) for _ in range(20)]
