"""`popquiz schedule` and `popquiz sync` (T-20): the gate, the date's arrangement,
the push, the fallback beside it, the used ledger pulled back, and the reserve.

Every test runs on a temporary copy of the committed bank. A test that needs a
question in a given review state puts it there in the copy - the happy path affirms
q3, the one record with a trace, and a refusal resets a record to its draft - so
nothing here depends on which committed records an organizer has affirmed (q3 was,
by F-52), and nothing writes to `bank/questions/`. The room is a fake `urlopen` that records what it
was sent; nothing here reaches a network.
"""

from __future__ import annotations

import ast
import dataclasses
import io
import json
import re
import shutil
import urllib.error
from datetime import date, timedelta
from pathlib import Path

import pytest

from fixtures.verify.verify_cases import RECORDINGS, verify_bank
from popquiz import audit, fallback, sandbox, schedule, verify
from popquiz.bank import affirmation_digest, affirmation_state, correct_index, is_affirmed, load_question, normalized_output, question_from_dict, question_to_dict
from popquiz.runner import StubRunner
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
    # Bound to the text, as `popquiz.review affirm` records it (GAP-24), unless the
    # test says otherwise or the block is only half an affirmation.
    if "affirmed_digest" not in review and {"affirmed_by", "affirmed_at"} <= data["review"].keys():
        data["review"]["affirmed_digest"] = affirmation_digest(question_from_dict(data))
    elif "affirmed_digest" in data["review"] and not {"affirmed_by", "affirmed_at"} <= data["review"].keys():
        del data["review"]["affirmed_digest"]
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def as_drafted(bank_dir: Path, qid: str) -> None:
    """Back to what a draft holds: no judgement, no affirmation, never used."""
    path = bank_dir / "questions" / f"{qid}.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    data.pop("used", None)
    data["review"] = {k: v for k, v in data.get("review", {}).items() if k not in ("status", "affirmed_by", "affirmed_at", "affirmed_digest")}
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


@pytest.fixture
def bank(tmp_path) -> Path:
    """The real questions, every one back to a draft, so a test affirms what it counts
    and an organizer's review in the bank changes no test's arithmetic. Each is then
    re-verified through `verify` over the recorded fixtures, which writes its record
    and its `recordings/` file, so scheduling has a recording to check (GAP-6)."""
    shutil.copytree(BANK / "questions", tmp_path / "bank" / "questions")
    for path in sorted((tmp_path / "bank" / "questions").glob("*.json")):
        as_drafted(tmp_path / "bank", path.stem)
    verify_bank(tmp_path / "bank")
    return tmp_path / "bank"


def recordings(bank_dir: Path) -> Path:
    return bank_dir / verify.RECORDINGS


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


def test_ac23_this_arrangement_passes_its_lint():
    """This file's `arrange`, `_shuffled` and `_rng`, read from disk: an edit to any of
    them that lets something but the date place the answer goes red here, by name."""
    assert audit.arrange_violations(audit.pipeline_sources()["schedule.py"]) == []
    assert audit._lint_check().verdict == "pass"


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
    as_drafted(bank, "q4")
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    code = run("schedule", "q4", "--date", NIGHT.isoformat(), "--room", "https://room.test", "--out", out, "--bank", bank)
    assert_refused(code, capsys, fake, out, "unaffirmed question cannot be scheduled")


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


def test_gap24_a_question_edited_after_affirmation_is_refused(affirmed, room, tmp_path, capsys):
    """GAP-24: the affirmation covers the text it was given to. A hand edit of a beat
    afterwards leaves who and when in place, and the gate refuses it anyway."""
    path = affirmed / "questions" / "q3.json"
    data = json.loads(path.read_text())
    data["explains"]["what"] += " Edited after affirmation."
    path.write_text(json.dumps(data))
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert_refused(schedule_q3(affirmed, out), capsys, fake, out, "its text changed after that")
    assert not audit.in_reserve(load_question(affirmed, "q3"))


def test_gap24_a_hand_typed_affirmation_on_a_verified_record_is_refused(bank, room, tmp_path, capsys):
    """M12b: who and when typed into a record the verifier wrote, with no digest,
    names no text."""
    affirm(bank, "q4", affirmed_digest=None)
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    code = run("schedule", "q4", "--date", NIGHT.isoformat(), "--room", "https://room.test", "--out", out, "--bank", bank)
    assert_refused(code, capsys, fake, out, "not what was affirmed")
    assert not audit.in_reserve(load_question(bank, "q4"))


def test_gap24_q3s_hand_affirmation_is_unbound_and_refused(affirmed, room, tmp_path, capsys):
    """RULING PQ-23 (client, 2026-10-12): q3 was affirmed by hand before the digest
    existed (F-52). That affirmation no longer counts, so q3 leaves the reserve until
    the organizer affirms it again through `popquiz.review`. The organizer has since
    done so, so the copy is put back to the hand affirmation: digest removed."""
    path = affirmed / "questions" / "q3.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    data["review"].pop("affirmed_digest", None)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    q3 = load_question(affirmed, "q3")
    assert q3.review.affirmed_digest is None
    assert not is_affirmed(q3) and not audit.in_reserve(q3)
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert_refused(schedule_q3(affirmed, out), capsys, fake, out, "not what was affirmed")


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


def _edit_record(bank_dir, qid, edit):
    """A hand edit of a bank file. Affirm afterwards, so only the record is wrong."""
    path = bank_dir / "questions" / f"{qid}.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    edit(data)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _unstamped(data):
    del data["verified"]["verified_at"]
    del data["verified"]["verifier_version"]


def _ran_after_all(data):
    data["verified"]["runs"] = {"count": 5, "byte_identical": True}


@pytest.mark.parametrize(
    ("qid", "edit", "needle"),
    [("q4", _unstamped, "not written by the verifier"), ("q8", _ran_after_all, "nothing ran")],
    ids=["M2a-unstamped", "contradicts-itself"],
)
@pytest.mark.parametrize("push", [True, False], ids=["room", "no-push"])
def test_ac7_a_record_the_verifier_did_not_write_is_refused(bank, room, tmp_path, capsys, qid, edit, needle, push):
    """GAP-4, the audit's M2a: an affirmed question in a copied bank, which CI never
    sees, whose record fails the build's provenance check. Nothing is sent or written,
    and the reason is the check's own."""
    _edit_record(bank, qid, edit)
    affirm(bank, qid)
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    where = ("--room", "https://room.test") if push else ("--no-push",)
    code = run("schedule", qid, "--date", NIGHT.isoformat(), *where, "--out", out, "--bank", bank)
    assert_refused(code, capsys, fake, out, needle)


def test_ac7_the_reason_is_the_checks_own_words(bank):
    _edit_record(bank, "q4", _unstamped)
    affirm(bank, "q4")
    q4 = load_question(bank, "q4")
    with pytest.raises(verify.ProvenanceError) as caught:
        verify.check_question(q4)
    assert schedule.refusal(q4) == str(caught.value)


def test_ac7_the_check_comes_after_the_refusals_that_were_already_there(bank):
    """A question already refused for being unaffirmed or used reads as it did, even
    with a record the check would refuse."""
    _edit_record(bank, "q4", _unstamped)
    q4 = load_question(bank, "q4")
    assert "unaffirmed question cannot be scheduled" in schedule.refusal(q4)

    def used(data):
        data["used"] = {"meetup_date": "2026-09-09", "room_id": "r-1", "released_at": "2026-09-10T01:40:00Z", "fit": "fits"}

    _edit_record(bank, "q4", used)
    affirm(bank, "q4")
    assert "never run twice" in schedule.refusal(load_question(bank, "q4"))


def test_ac7_just_demo_checks_the_file_before_any_network_call():
    """`just demo` pushes the stored file, so neither `refusal` nor `bake` sees it, and
    it runs the check itself, first, unguarded, so a refusal stops the recipe (its
    `set -e`) with nothing sent. Read from the recipe, as `just test` starts no process."""
    justfile = (BANK.parent / "justfile").read_text(encoding="utf-8")
    lines = [line.strip() for line in justfile[justfile.index("\ndemo "):justfile.index("\nsync ")].splitlines()]
    assert "set -euo pipefail" in lines
    check = "uv run --offline --no-sync --project pipeline python -m popquiz.verify --check \"$record\""
    assert lines.count(check) == 1
    network = [i for i, line in enumerate(lines) if re.search(r"\b(op read|auth |curl)\b", line)]
    assert network and lines.index(check) < min(network)


def test_ac72_every_committed_record_is_refused_iff_unaffirmed(bank, room, tmp_path, capsys):
    """Read from the committed records, not typed: each one an organizer has not
    affirmed is refused with nothing sent or written, and each one affirmed and
    bound to its text schedules. q3's digest-less hand affirmation (F-52) is
    refused as unbound until it is affirmed again. The next affirmation moves a
    record from one side to the other without touching this test."""
    for q in schedule.load_bank(bank):
        out = tmp_path / q.id
        fake = room((201, {"id": q.id, "scheduled": "new"}))
        code = run("schedule", q.id, "--date", NIGHT.isoformat(), "--room", "https://room.test", "--out", out, "--bank", bank)
        state = affirmation_state(q)
        if state == "unbound":
            assert_refused(code, capsys, fake, out, "not what was affirmed")
        elif not is_affirmed(q):
            assert_refused(code, capsys, fake, out, "unaffirmed question cannot be scheduled")
        else:
            capsys.readouterr()
            assert code == 0, q.id
            assert len(fake.requests) == 1 and out.exists(), q.id


def test_g12_the_gate_agrees_with_the_reserve(affirmed):
    for q in schedule.load_bank(affirmed):
        assert (schedule.refusal(q, recordings=recordings(affirmed)) is None) == audit.in_reserve(q)


# --------------------------------------------------------------------------- #
# AC-6, the stale pin (GAP-5)
# --------------------------------------------------------------------------- #


def _pin_matching(question) -> sandbox.Pin:
    """A pin built here, from the record it is compared with, never from this
    machine's `pin.toml`: what the record says it ran under, and nothing more."""
    v = question.verified
    vv = dict(line.split(": ", 1) for line in v.rustc.splitlines() if ": " in line)
    return sandbox.Pin(
        rust_version=vv["release"],
        release=vv["release"],
        commit_hash=vv["commit-hash"],
        edition=v.edition,
        nightly="2000-01-01",
        miri_version=v.miri.version,
        target_triples=(v.target_triple,),
        flags={
            "opt-level": v.flags.opt_level,
            "overflow-checks": v.flags.overflow_checks,
            "debug-assertions": v.flags.debug_assertions,
        },
        image_tag="fixture",
        platform="linux/amd64",
        limits=sandbox.Limits(memory="1g", memory_swap="1g", cpus="1", pids=64, timeout_seconds=10, tmpfs_size="64m"),
    )


def test_ac6_a_record_verified_under_another_rustc_release_is_refused(bank):
    affirm(bank, "q4")
    q4 = load_question(bank, "q4")
    pin = _pin_matching(q4)
    recorded = pin.release
    why = schedule.refusal(q4, pin=dataclasses.replace(pin, release="0.0.0"))
    assert why == (
        f"q4 was verified under a toolchain that no longer matches the pin (rustc release "
        f"{recorded}, the pin says 0.0.0) - re-verify it with `just verify` before it is "
        "scheduled (AC-6)"
    )


def legacy_bank(tmp_path) -> Path:
    """The committed q3 as it is, legacy record and all, never re-verified, affirmed."""
    (tmp_path / "legacy" / "questions").mkdir(parents=True)
    shutil.copy(BANK / "questions" / "q3.json", tmp_path / "legacy" / "questions" / "q3.json")
    as_drafted(tmp_path / "legacy", "q3")
    affirm(tmp_path / "legacy", "q3")
    return tmp_path / "legacy"


def test_ac6_a_legacy_record_is_not_refused_for_its_pin(bank, tmp_path):
    """Exempt from the stale-pin check, it is still refused for having no recording."""
    legacy = legacy_bank(tmp_path)
    q3 = load_question(legacy, "q3")
    assert q3.verified.legacy
    stale = dataclasses.replace(_pin_matching(load_question(bank, "q4")), release="0.0.0")
    why = schedule.refusal(q3, pin=stale, recordings=recordings(legacy))
    assert "no longer matches the pin" not in why
    assert why == NO_RECORDING.format(qid="q3")


def test_ac6_a_record_that_matches_the_pin_is_scheduled(bank):
    affirm(bank, "q4")
    q4 = load_question(bank, "q4")
    assert schedule.refusal(q4, pin=_pin_matching(q4), recordings=recordings(bank)) is None


def test_ac6_a_record_whose_miri_version_differs_is_refused(bank):
    affirm(bank, "q4")
    q4 = load_question(bank, "q4")
    assert q4.verified.miri is not None  # Miri ran, so its version is compared
    pin = dataclasses.replace(_pin_matching(q4), miri_version="miri 0.0.0")
    why = schedule.refusal(q4, pin=pin)
    assert why is not None
    assert f"Miri {q4.verified.miri.version}, the pin says miri 0.0.0" in why
    assert "re-verify it with `just verify`" in why


# --------------------------------------------------------------------------- #
# AC-7, the record and its recording (GAP-6)
# --------------------------------------------------------------------------- #

NO_RECORDING = "{qid} has no recording - re-verify it with `just verify` before it is scheduled (AC-7)"


def _json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def _write(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def _hand_edit_answer(bank_dir: Path, qid: str) -> None:
    """`stdout` and the option that names it, edited together, and the affirmation
    bound again: a record `check_question` cannot tell from a genuine one."""
    path = bank_dir / "questions" / f"{qid}.json"
    data = _json(path)
    old = normalized_output(data["verified"]["stdout"])
    data["verified"]["stdout"] = "hand edited\n"
    named = [o for o in data["options"] if o["text"] == old]
    assert len(named) == 1
    named[0]["text"] = "hand edited"
    _write(path, data)
    affirm(bank_dir, qid)
    verify.check_question(load_question(bank_dir, qid))  # self-consistent: the old check passes it


def _gate(bank_dir: Path, qid: str = "q3") -> str | None:
    q = load_question(bank_dir, qid)
    return schedule.refusal(q, pin=_pin_matching(q), recordings=recordings(bank_dir))


def test_gap6_verify_writes_the_recording_beside_the_record(bank):
    rec = _json(recordings(bank) / "q3.json")
    q3 = load_question(bank, "q3")
    assert rec["_record_sha256"] == verify.record_sha256(q3)
    assert rec["_source_sha256"] == verify.source_sha256(q3.source)
    assert {"toolchain", "compile", "run:1"} <= rec.keys()


def test_gap6_just_verify_writes_the_recording_after_the_record(tmp_path, capsys):
    (tmp_path / "b" / "questions").mkdir(parents=True)
    shutil.copy(BANK / "questions" / "q4.json", tmp_path / "b" / "questions" / "q4.json")
    assert verify.main([str(tmp_path / "b" / "questions" / "q4.json")], runner=StubRunner(RECORDINGS)) == 0
    assert "q4: recording written to" in capsys.readouterr().out
    q4 = load_question(tmp_path / "b", "q4")
    rec = _json(tmp_path / "b" / "recordings" / "q4.json")
    assert rec["_record_sha256"] == verify.record_sha256(q4)
    assert rec["_source_sha256"] == verify.source_sha256(q4.source)
    assert rec["_recorded"]["verified_at"] == q4.verified.verified_at
    q4 = dataclasses.replace(q4, review=dataclasses.replace(q4.review, status="accepted"))
    assert schedule._recording_refusal(q4, tmp_path / "b" / "recordings", None) is None


def test_gap6_a_record_that_agrees_with_its_recording_is_scheduled(bank):
    affirm(bank, "q3")
    assert _gate(bank) is None


def test_gap6_a_record_with_no_recording_is_refused(bank):
    affirm(bank, "q3")
    (recordings(bank) / "q3.json").unlink()
    assert _gate(bank) == NO_RECORDING.format(qid="q3")


def test_gap6_a_legacy_record_with_no_recording_is_refused(tmp_path):
    legacy = legacy_bank(tmp_path)
    q3 = load_question(legacy, "q3")
    assert q3.verified.legacy
    assert schedule.refusal(q3, recordings=recordings(legacy)) == NO_RECORDING.format(qid="q3")


def test_gap6_an_answer_edited_with_its_option_is_refused_when_it_has_a_recording(bank):
    _hand_edit_answer(bank, "q3")
    why = _gate(bank)
    assert why == (
        "q3 does not match its recording (its verified record changed after `just verify` "
        "wrote it) - re-verify it with `just verify` before it is scheduled (AC-7)"
    )


def test_gap6_an_edit_that_also_recomputes_the_record_digest_is_caught_by_the_replay(bank):
    _hand_edit_answer(bank, "q3")
    path = recordings(bank) / "q3.json"
    rec = _json(path)
    rec["_record_sha256"] = verify.record_sha256(load_question(bank, "q3"))
    _write(path, rec)
    why = _gate(bank)
    assert why.startswith("q3 does not match its recording (replayed through the stub runner, it gives a different stdout)")


@pytest.mark.parametrize("field, words", [("_source_sha256", "its source"), ("_record_sha256", "its verified record")])
def test_gap6_a_recording_whose_digest_does_not_match_is_refused(bank, field, words):
    affirm(bank, "q3")
    path = recordings(bank) / "q3.json"
    rec = _json(path)
    rec[field] = "0" * 64
    _write(path, rec)
    assert f"q3 does not match its recording ({words} changed after `just verify` wrote it)" in _gate(bank)


def test_gap6_a_recording_whose_miri_replays_differently_is_refused(bank):
    affirm(bank, "q3")
    path = recordings(bank) / "q3.json"
    rec = _json(path)
    rec["miri:stacked_borrows:0"]["stderr"] += "\nerror: Undefined Behavior: planted\n"
    _write(path, rec)
    why = _gate(bank)
    assert why.startswith("q3 does not match its recording (replayed through the stub runner, it is rejected:")
    assert "undefined behavior" in why


def test_gap6_an_unreadable_recording_is_refused(bank):
    affirm(bank, "q3")
    (recordings(bank) / "q3.json").write_text("not json", encoding="utf-8")
    assert _gate(bank).startswith("q3's recording cannot be read")


def test_gap6_a_recording_with_a_malformed_step_is_refused_not_raised(bank):
    """A step missing a field the stub runner reads is a refusal, never a traceback."""
    affirm(bank, "q3")
    path = recordings(bank) / "q3.json"
    rec = _json(path)
    del rec["compile"]["exit_code"]
    _write(path, rec)
    why = _gate(bank)
    assert why.startswith("q3's recording does not replay (")
    assert why.endswith(" - re-verify it with `just verify` before it is scheduled (AC-7)")


def test_gap6_a_record_miri_never_ran_for_survives_a_miri_only_pin_change(bank):
    """Miri's version is part of a record only when Miri ran (`verify.is_stale`), so
    the replay must not hold a does-not-compile record to the pin's new Miri."""
    affirm(bank, "q8")
    q8 = load_question(bank, "q8")
    assert q8.verified.miri is None  # it never compiled, so Miri never ran
    pin = dataclasses.replace(_pin_matching(load_question(bank, "q4")), miri_version="miri 0.0.0")
    assert not verify.is_stale(q8.verified, pin)
    assert schedule.refusal(q8, pin=pin, recordings=recordings(bank)) is None


def test_gap6_a_record_miri_ran_for_is_still_refused_after_a_miri_only_pin_change(bank):
    affirm(bank, "q4")
    q4 = load_question(bank, "q4")
    pin = dataclasses.replace(_pin_matching(q4), miri_version="miri 0.0.0")
    why = schedule.refusal(q4, pin=pin, recordings=recordings(bank))
    assert why is not None and "no longer matches the pin" in why


def _sandbox(monkeypatch, runner_for):
    calls = []

    def fake(question, pin):
        calls.append(question.id)
        return runner_for(question, pin)

    monkeypatch.setattr(schedule, "_sandbox_runner", fake)
    return calls


def test_gap6_replay_that_agrees_schedules(bank, room, tmp_path, monkeypatch, capsys):
    affirm(bank, "q3")
    calls = _sandbox(monkeypatch, lambda q, pin: StubRunner(RECORDINGS))
    fake = room((201, {"id": "q3", "scheduled": "new"}))
    assert schedule_q3(bank, tmp_path / "out", NIGHT, "--replay") == 0, capsys.readouterr().err
    assert calls == ["q3"] and len(fake.requests) == 1


def test_gap6_replay_refuses_a_record_a_new_run_disagrees_with(bank, room, tmp_path, monkeypatch, capsys):
    """Record, recording and both digests edited together pass the recording check;
    only a new run catches them. Here the new run is what the toolchain recorded."""
    affirm(bank, "q3")
    _hand_edit_answer(bank, "q3")
    path = recordings(bank) / "q3.json"
    rec = _json(path)
    for kind in [k for k in rec if k.startswith("run:")]:
        rec[kind]["stdout"] = "hand edited\n"
    for kind in [k for k in rec if k.startswith("miri:")]:
        rec[kind]["stdout"] = "hand edited\n"
    rec["_record_sha256"] = verify.record_sha256(load_question(bank, "q3"))
    _write(path, rec)
    assert _gate(bank) is None  # the limit, stated plainly: the recording check passes it
    _sandbox(monkeypatch, lambda q, pin: StubRunner(RECORDINGS))
    fake = room((201, {"id": "q3", "scheduled": "new"}))
    code = schedule_q3(bank, tmp_path / "out", NIGHT, "--replay")
    assert_refused(code, capsys, fake, tmp_path / "out", "q3 does not match a new run in the sandbox (a different stdout)")


def test_gap6_replay_on_the_other_supported_host_agrees(bank, tmp_path, monkeypatch):
    """The platform is not pinned: a new run on the pin's other triple is the same run."""
    affirm(bank, "q3")
    q3 = load_question(bank, "q3")
    other = next(t for t in ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu") if t != q3.verified.target_triple)
    rec = _json(RECORDINGS / "q3.json")
    rec["toolchain"]["stdout"] = rec["toolchain"]["stdout"].replace(f"host: {q3.verified.target_triple}", f"host: {other}")
    (tmp_path / "elsewhere").mkdir()
    _write(tmp_path / "elsewhere" / "q3.json", rec)
    pin = dataclasses.replace(_pin_matching(q3), target_triples=(q3.verified.target_triple, other))
    _sandbox(monkeypatch, lambda q, p: StubRunner(tmp_path / "elsewhere"))
    assert schedule.replay(q3, pin=pin) is None


class NoDocker:
    def run(self, kind, program_id):
        raise FileNotFoundError(2, "No such file or directory", "docker")


class NoImage:
    def run(self, kind, program_id):
        return verify.RunStep(kind=kind, exit_code=125, stdout="", stderr="Unable to find image", recorded_from="none")


@pytest.mark.parametrize("runner", [NoDocker(), NoImage()], ids=["no-docker", "no-image"])
def test_gap6_replay_with_no_sandbox_refuses_and_sends_nothing(bank, room, tmp_path, monkeypatch, capsys, runner):
    affirm(bank, "q3")
    _sandbox(monkeypatch, lambda q, pin: runner)
    fake = room((201, {"id": "q3", "scheduled": "new"}))
    code = schedule_q3(bank, tmp_path / "out", NIGHT, "--replay")
    assert_refused(code, capsys, fake, tmp_path / "out", "`--replay` needs the sandbox")
    err_again = schedule.replay(load_question(bank, "q3"), pin=_pin_matching(load_question(bank, "q3")))
    assert err_again.endswith("; run `just sandbox-build`, or schedule without `--replay`")


def test_gap6_without_the_flag_nothing_new_runs(bank, room, tmp_path, monkeypatch, capsys):
    affirm(bank, "q3")
    calls = _sandbox(monkeypatch, lambda q, pin: NoDocker())
    fake = room((201, {"id": "q3", "scheduled": "new"}))
    assert schedule_q3(bank, tmp_path / "out", NIGHT) == 0, capsys.readouterr().err
    assert calls == [] and len(fake.requests) == 1


def _rustc_release_zero(data):
    data["verified"]["rustc"] = re.sub(r"(?m)^release: .*$", "release: 0.0.0", data["verified"]["rustc"])


def test_ac6_schedule_refuses_a_stale_record_and_sends_nothing(bank, room, tmp_path, capsys):
    """Through the command, with the pin it reads itself: no pin's release is 0.0.0."""
    _edit_record(bank, "q4", _rustc_release_zero)
    affirm(bank, "q4")
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    code = run("schedule", "q4", "--date", NIGHT.isoformat(), "--room", "https://room.test", "--out", out, "--bank", bank)
    assert_refused(code, capsys, fake, out, "rustc release 0.0.0, the pin says")


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
    question = load_question(affirmed, "q3")
    expected = schedule.arrange(question, NIGHT)
    body = json.loads(request.data)
    # ADR-0005 item 9: beside the record, the bank order the room checks the
    # affirmation's digest in; slot k holds the bank's option bank_order[k].
    order = body.pop("bank_order")
    # SPEC §8.5 (R1-a): and the meetup date it was arranged for.
    assert body.pop("meetup_date") == NIGHT.isoformat()
    assert body == question_to_dict(expected)
    assert [question.options[i] for i in order] == list(expected.options)
    assert "scheduled: new" in capsys.readouterr().out


@pytest.mark.parametrize("club", ["rustnyc", "la"])
def test_r1a_the_push_carries_the_date_it_was_arranged_for(affirmed, room, tmp_path, club):
    """SPEC §8.5: the room offers a pushed record only on the date it was
    arranged for, so the push names that date, the `--date`, not today."""
    fake = room((201, {"scheduled": "new"}), (201, {"scheduled": "new"}))
    question = load_question(affirmed, "q3")
    for day in (NIGHT, NIGHT + timedelta(days=28)):
        assert schedule_q3(affirmed, tmp_path / f"out-{day}", day, "--club", club) == 0
        body = json.loads(fake.requests[-1].data)
        assert body["meetup_date"] == day.isoformat()
        body.pop("meetup_date"), body.pop("bank_order")
        assert body == question_to_dict(schedule.arrange(question, day))


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
        schedule.push(schedule.arrange(load_question(affirmed, "q3"), NIGHT), "https://room.test", PLANT, NIGHT)
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
    assert load_question(affirmed, "q3").used == ()


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


def club_ledger(*entries):
    """Ledger lines that name their club, as the room writes them (D-26)."""
    return [{"club": club, **line} for club, line in zip((e[0] for e in entries), ledger(*(e[1:] for e in entries)))]


# --------------------------------------------------------------------------- #
# D-26, AC-105: never twice is per club
# --------------------------------------------------------------------------- #


def test_ac105_two_clubs_may_each_run_a_question_once(bank, room, capsys):
    room((200, club_ledger(("nyc", "q3", "2026-10-14", "room-a", "fits"), ("la", "q3", "2026-10-15", "room-l", None))))
    assert sync(bank) == 0
    q3 = load_question(bank, "q3")
    assert (q3.used_by("nyc").room_id, q3.used_by("la").room_id) == ("room-a", "room-l")
    assert q3.used_by("sf") is None
    out = capsys.readouterr().out
    assert "used: q3 by rustnyc on 2026-10-14, room room-a" in out
    assert "used: q3 by la on 2026-10-15, room room-l" in out


def test_ac105_a_second_room_for_one_club_is_refused_and_the_other_club_is_untouched(bank, room, capsys):
    room((200, club_ledger(("nyc", "q3", "2026-10-14", "room-a", "fits"), ("la", "q3", "2026-10-15", "room-l", "fits"))))
    assert sync(bank) == 0
    before = (bank / "questions" / "q3.json").read_bytes()
    room((200, club_ledger(("la", "q3", "2026-11-12", "room-m", "fits"))))
    assert sync(bank) == 1
    err = capsys.readouterr().err
    assert "used by la" in err and "never run twice for a club" in err
    assert (bank / "questions" / "q3.json").read_bytes() == before


def test_ac105_sync_twice_with_two_clubs_changes_no_file(bank, room):
    room((200, club_ledger(("nyc", "q3", "2026-10-14", "room-a", "fits"), ("la", "q3", "2026-10-15", "room-l", "fits"))))
    assert sync(bank) == 0
    after_first = snapshot(bank)
    assert sync(bank) == 0
    assert snapshot(bank) == after_first


def test_ac105_the_gate_and_the_reserve_are_per_club(affirmed, room, tmp_path, capsys):
    room((200, club_ledger(("nyc", "q3", "2026-10-14", "room-a", "fits"))))
    assert sync(affirmed) == 0
    question = load_question(affirmed, "q3")
    assert "never run twice for a club" in schedule.refusal(question, "nyc")
    assert schedule.refusal(question, "la", recordings=recordings(affirmed)) is None
    assert audit.in_reserve(question, "la") and not audit.in_reserve(question, "nyc")
    lines = schedule.reserve_lines(schedule.load_bank(affirmed), today=NIGHT, club="la")
    assert lines[0].startswith("reserve (la):")
    # And the command takes the club: la may schedule what nyc has run.
    room((201, {"scheduled": "new"}))
    assert run("schedule", "q3", "--date", NIGHT.isoformat(), "--no-push", "--out", tmp_path / "o", "--bank", affirmed, "--club", "la") == 0
    assert run("schedule", "q3", "--date", NIGHT.isoformat(), "--no-push", "--out", tmp_path / "o3", "--bank", affirmed) == 1


def test_a_legacy_bare_used_object_reads_as_the_default_clubs(affirmed):
    path = affirmed / "questions" / "q3.json"
    data = json.loads(path.read_text())
    data["used"] = {"meetup_date": "2026-09-09", "room_id": "r-1", "released_at": "2026-09-10T01:40:00Z"}
    path.write_text(json.dumps(data))
    q3 = load_question(affirmed, "q3")
    assert q3.used_by("nyc").room_id == "r-1" and q3.used_by("la") is None


def test_sync_writes_used_from_the_ledger_and_a_null_fit_is_an_absent_key(bank, room, capsys):
    room((200, ledger(("q3", "2026-10-14", "room-a", None), ("q4", "2026-10-15", "room-b", "clipped_x"))))
    assert sync(bank) == 0
    q3 = json.loads((bank / "questions" / "q3.json").read_text())
    assert q3["used"] == [
        {"meetup_date": "2026-10-14", "room_id": "room-a", "released_at": "2026-10-14T01:40:00Z", "club": "rustnyc"}
    ]
    assert load_question(bank, "q3").used_by().fit is None
    assert load_question(bank, "q4").used_by().fit == "clipped_x"
    out = capsys.readouterr().out
    assert "used: q3 by rustnyc on 2026-10-14, room room-a" in out
    assert "fit: room room-a (q3) - the wall never reported a verdict" in out
    assert "fit: room room-b (q4) - clipped_x" in out


def test_sync_writes_who_affirmed_the_question_the_room_ran(bank, room, capsys):
    """ADR-0005 item 9: the room's used record names the affirming organizer, and
    sync carries it into the bank and the ledger report. An older record without
    it reads and prints as before (absent is absent)."""
    entries = ledger(("q3", "2026-10-14", "room-a", "fits"), ("q4", "2026-10-15", "room-b", "fits"))
    entries[0]["used"]["affirmed_by"] = "queenrose54"
    room((200, entries))
    assert sync(bank) == 0
    q3 = json.loads((bank / "questions" / "q3.json").read_text())
    assert q3["used"][0]["affirmed_by"] == "queenrose54"
    assert load_question(bank, "q3").used_by().affirmed_by == "queenrose54"
    assert "affirmed_by" not in json.loads((bank / "questions" / "q4.json").read_text())["used"][0]
    out = capsys.readouterr().out
    assert "used: q3 by rustnyc on 2026-10-14, room room-a, affirmed by queenrose54" in out
    assert "used: q4 by rustnyc on 2026-10-15, room room-b\n" in out
    # The same ledger again changes nothing.
    after_first = snapshot(bank)
    assert sync(bank) == 0
    assert snapshot(bank) == after_first


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
    assert load_question(bank, "q4").used_by().room_id == "room-y"


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
    assert schedule.reserve_lines(questions, today=NIGHT)[0].startswith(f"reserve (rustnyc): {expected} ready")


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
    """Every record is put back to its draft first, so the counts follow from the
    states set here and the size of the bank, not from which records an organizer
    has affirmed since."""
    drafts = [q.id for q in schedule.load_bank(bank)]
    for qid in drafts:
        as_drafted(bank, qid)
    affirm(bank, "q3")  # in the reserve: neither coming in nor gone out
    affirm(bank, "q4", affirmed_by=None, affirmed_at=None)  # accepted, awaiting affirmation
    path = bank / "questions" / "q7.json"
    data = json.loads(path.read_text())
    data["used"] = {"meetup_date": "2026-09-09", "room_id": "r", "released_at": "2026-09-10T01:00:00Z"}
    path.write_text(json.dumps(data))
    reserve, trend = schedule.reserve_lines(schedule.load_bank(bank), today=NIGHT)[:2]
    assert reserve.startswith("reserve (rustnyc): 1 ready")
    unreviewed = len(drafts) - 3
    assert trend == f"trend: +1 accepted awaiting affirmation, +{unreviewed} not yet reviewed, -1 used in the last 90 days"


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


# --------------------------------------------------------------------------- #
# Code-review follow-ups
# --------------------------------------------------------------------------- #


@pytest.mark.parametrize("bad", [PLANT + "\n" + "x", "token with space", PLANT + "é"])
def test_ac101_a_token_that_cannot_be_a_header_is_refused_without_echoing_it(affirmed, room, monkeypatch, tmp_path, capsys, bad):
    fake = room((201, {"scheduled": "new"}))
    monkeypatch.setenv(schedule.TOKEN_ENV, bad)
    assert schedule_q3(affirmed, tmp_path / "out") == 1
    captured = capsys.readouterr()
    assert "characters a token cannot have" in captured.err
    assert bad not in captured.out + captured.err and PLANT not in captured.err
    assert fake.requests == []


def test_ac101_a_trailing_newline_is_stripped_not_sent(affirmed, room, monkeypatch, tmp_path):
    fake = room((201, {"scheduled": "new"}))
    monkeypatch.setenv(schedule.TOKEN_ENV, PLANT + "\n")
    assert schedule_q3(affirmed, tmp_path / "out") == 0
    assert fake.requests[0].get_header("Authorization") == f"Bearer {PLANT}"


def test_ac101_a_header_the_transport_refuses_never_carries_its_message(affirmed, room, tmp_path, capsys):
    room((ValueError(f"Invalid header value b'Bearer {PLANT}'"), None))
    assert schedule_q3(affirmed, tmp_path / "out") == 1
    err = capsys.readouterr().err
    assert "could not be sent (ValueError)" in err and PLANT not in err


def test_ac101_a_reason_that_echoes_the_token_is_redacted(affirmed, room, tmp_path, capsys):
    room((400, {"reason": f"bad request with Bearer {PLANT}"}))
    assert schedule_q3(affirmed, tmp_path / "out") == 1
    err = capsys.readouterr().err
    assert "[redacted]" in err and PLANT not in err


@pytest.mark.parametrize("status", [408, 429])
def test_ac101_not_now_answers_are_retried(affirmed, room, tmp_path, status):
    fake = room((status, b""), (201, {"scheduled": "new"}))
    assert schedule_q3(affirmed, tmp_path / "out") == 0
    assert len(fake.requests) == 2


def test_sync_refuses_a_same_room_record_that_differs(bank, room, capsys):
    room((200, ledger(("q3", "2026-10-14", "room-a", "fits"))))
    assert sync(bank) == 0
    before = snapshot(bank)
    room((200, ledger(("q3", "2026-10-14", "room-a", "clipped_y"))))
    assert sync(bank) == 1
    assert "differs" in capsys.readouterr().err
    assert snapshot(bank) == before


# --------------------------------------------------------------------------- #
# Fix-back round (exact-head review of 19ca530)
# --------------------------------------------------------------------------- #


def test_ac102_a_record_the_fallback_refuses_is_refused_before_any_request(affirmed, room, tmp_path, capsys):
    """The files are rendered before the push, so a record the fallback cannot
    build (here, a trace too short to walk) never reaches the room."""
    path = affirmed / "questions" / "q3.json"
    data = json.loads(path.read_text())
    data["trace"]["steps"] = data["trace"]["steps"][-1:]
    path.write_text(json.dumps(data))
    affirm(affirmed, "q3")  # bound to the shortened trace, so the fallback is what refuses
    fake = room((201, {"scheduled": "new"}))
    out = tmp_path / "out"
    assert schedule_q3(affirmed, out) == 1
    assert "fewer than two steps" in capsys.readouterr().err
    assert fake.requests == [] and not out.exists()


@pytest.mark.parametrize("status", [403, 404])
def test_ac101_an_unexpected_answer_is_reported_without_the_token(affirmed, bank, room, tmp_path, capsys, status):
    room((status, b"not for you"))
    assert schedule_q3(affirmed, tmp_path / "out") == 1
    pushed = capsys.readouterr()
    assert f"the room answered {status} to the push" in pushed.err
    assert sync(bank) == 1
    pulled = capsys.readouterr()
    assert f"the room answered {status} to the ledger request" in pulled.err
    assert PLANT not in pushed.out + pushed.err + pulled.out + pulled.err


def test_ac23_the_distractor_shuffle_is_keyed_on_the_date(affirmed):
    """Nights that share a slot still order the other four differently: the
    shuffle reads the date, so one night's order says nothing about another's."""
    q3 = load_question(affirmed, "q3")
    same_slot, day = [], date(2026, 1, 1)
    while len(same_slot) < 6:
        if slot_for_day(day) == 2:
            same_slot.append(day)
        day += timedelta(days=1)
    arranged = [schedule.arrange(q3, d) for d in same_slot]
    assert {correct_index(a) for a in arranged} == {2}
    assert len({tuple(o.text for o in a.options) for a in arranged}) > 1


def test_ac102_a_write_failure_after_a_push_says_the_room_holds_the_record(affirmed, room, monkeypatch, tmp_path, capsys):
    room((201, {"scheduled": "new"}))

    def full_disk(*args, **kwargs):
        raise OSError(28, "No space left on device")

    monkeypatch.setattr(fallback, "write_fallback", full_disk)
    assert schedule_q3(affirmed, tmp_path / "out") == 1
    err = capsys.readouterr().err
    assert "the room holds the record" in err and "may not both be in" in err
    assert "Traceback" not in err and PLANT not in err


def test_ac23_the_postcondition_is_a_raise_not_an_assert():
    tree = ast.parse(Path(schedule.__file__).read_text(encoding="utf-8"))
    arrange = next(n for n in ast.walk(tree) if isinstance(n, ast.FunctionDef) and n.name == "arrange")
    assert not any(isinstance(n, ast.Assert) for n in ast.walk(arrange)), "-O strips an assert"
    assert any(isinstance(n, ast.Raise) for n in ast.walk(arrange))


# --------------------------------------------------------------------------- #
# Review of PR #52: P1 (the push is per club) and P2 (the fallback link is too)
# --------------------------------------------------------------------------- #


def test_p1_a_push_for_another_club_goes_to_that_clubs_path(affirmed, room, tmp_path):
    fake = room((201, {"scheduled": "new"}))
    args = ("schedule", "q3", "--date", NIGHT.isoformat(), "--room", "https://room.test", "--out", tmp_path / "o", "--bank", affirmed)
    assert run(*args, "--club", "la") == 0
    assert fake.requests[-1].full_url.endswith("/admin/clubs/la/questions/q3")
    assert run(*args) == 0
    assert fake.requests[-1].full_url.endswith("/admin/questions/q3"), "the default club keeps its old path"


def test_p1_a_club_that_is_not_a_slug_is_refused_before_anything_is_sent(affirmed, room, tmp_path, capsys):
    fake = room((201, {"scheduled": "new"}))
    for bad in ("LA", "la/../nyc", "", "a b"):
        out = tmp_path / ("o" + str(abs(hash(bad))))
        code = run("schedule", "q3", "--date", NIGHT.isoformat(), "--room", "https://room.test",
                   "--out", out, "--bank", affirmed, "--club", bad)
        assert code == 1, bad
        assert "club" in capsys.readouterr().err
    assert fake.requests == []


def _baked_home_link(out):
    import re
    html = (out / "q3.html").read_text()
    return re.search(r'"home_link":\s*"([^"]+)"', html).group(1)


def test_p2_the_fallback_links_to_the_selected_clubs_page(affirmed, tmp_path):
    base = "https://popquiz.rustnyc.org/last"
    for extra, want in ((["--club", "la"], base + "/la"), (["--club", "nyc"], base), ([], base)):
        out = tmp_path / ("o" + "".join(extra).replace("-", ""))
        assert run("schedule", "q3", "--date", NIGHT.isoformat(), "--no-push", "--out", out, "--bank", affirmed, *extra) == 0
        assert _baked_home_link(out) == want, extra


def test_p2_an_explicit_home_link_wins(affirmed, tmp_path):
    out = tmp_path / "o"
    assert run("schedule", "q3", "--date", NIGHT.isoformat(), "--no-push", "--out", out, "--bank", affirmed,
               "--club", "la", "--home-link", "https://example.test/mine") == 0
    assert _baked_home_link(out) == "https://example.test/mine"


# --------------------------------------------------------------------------- #
# ADR-0005 item 8: the default club is rustnyc, and nyc is its old name
# --------------------------------------------------------------------------- #


def _used_under(bank_dir: Path, qid: str, club: str | None) -> Path:
    """Give `qid` a used record as a bank written before the rename holds it."""
    path = bank_dir / "questions" / f"{qid}.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    record = {"meetup_date": "2026-10-14", "room_id": "room-a", "released_at": "2026-10-14T01:40:00Z"}
    if club is not None:
        record["club"] = club
    data["used"] = [record]
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return path


def test_sync_from_a_room_on_the_old_build_writes_rustnyc(bank, room):
    room((200, club_ledger(("nyc", "q3", "2026-10-14", "room-a", "fits"))))
    assert sync(bank) == 0
    text = (bank / "questions" / "q3.json").read_text(encoding="utf-8")
    assert json.loads(text)["used"][0]["club"] == "rustnyc"
    assert '"nyc"' not in text
    # Syncing the old room again (the release order's "sync again right before
    # merging") changes nothing, and nor does the new room's spelling of it.
    after_first = snapshot(bank)
    for club in ("nyc", "rustnyc"):
        room((200, club_ledger((club, "q3", "2026-10-14", "room-a", "fits"))))
        assert sync(bank) == 0, club
        assert snapshot(bank) == after_first, club


def test_sync_refuses_a_ledger_club_that_is_not_a_slug(bank, room, capsys):
    before = snapshot(bank)
    room((200, club_ledger(("NYC", "q3", "2026-10-14", "room-a", "fits"))))
    assert sync(bank) == 1
    assert "not a club slug" in capsys.readouterr().err
    assert snapshot(bank) == before


@pytest.mark.parametrize("stored", ["nyc", "rustnyc", None])
def test_a_question_used_under_nyc_is_used_under_rustnyc(affirmed, tmp_path, stored, capsys):
    _used_under(affirmed, "q3", stored)
    question = load_question(affirmed, "q3")
    for club in ("rustnyc", "nyc"):
        assert "never run twice for a club" in schedule.refusal(question, club), (stored, club)
        assert not audit.in_reserve(question, club)
    assert schedule.refusal(question, "la", recordings=recordings(affirmed)) is None
    lines = schedule.reserve_lines(schedule.load_bank(affirmed), today=NIGHT)
    assert lines[0].startswith("reserve (rustnyc): 0 ready"), lines[0]
    for extra in ([], ["--club", "nyc"], ["--club", "rustnyc"]):
        out = tmp_path / f"o-{len(extra)}-{extra[-1] if extra else 'default'}"
        assert run("schedule", "q3", "--date", NIGHT.isoformat(), "--no-push", "--out", out, "--bank", affirmed, *extra) == 1
        assert "was already used by rustnyc" in capsys.readouterr().err


def test_the_club_flag_reads_nyc_as_rustnyc(affirmed, room, capsys):
    assert run("reserve", "--bank", affirmed, "--club", "nyc") == 0
    assert capsys.readouterr().out.startswith("reserve (rustnyc): 1 ready")
    assert schedule.home_link_for("nyc") == schedule.home_link_for("rustnyc") == schedule.fallback.DEFAULT_HOME_LINK


def migrate(bank_dir: Path, *extra) -> int:
    return run("migrate-club", "--bank", bank_dir, *extra)


def test_migrate_club_is_a_dry_run_by_default(bank, capsys):
    _used_under(bank, "q3", "nyc")
    before = snapshot(bank)
    assert migrate(bank) == 0
    assert "would rewrite q3: used[0].club nyc -> rustnyc" in capsys.readouterr().out
    assert snapshot(bank) == before, "a dry run writes nothing, not even a backup"


def test_migrate_club_backs_up_then_rewrites_and_a_second_run_does_nothing(bank, tmp_path, capsys):
    path = _used_under(bank, "q3", "nyc")
    _used_under(bank, "q4", None)
    original_q3 = path.read_bytes()
    original_q4 = (bank / "questions" / "q4.json").read_bytes()
    backups = tmp_path / "backups"
    assert migrate(bank, "--write", "--backup-dir", backups) == 0
    out = capsys.readouterr().out
    assert "rewrote q3: used[0].club nyc -> rustnyc" in out and f"backup: {backups / 'q3.json'}" in out
    assert (backups / "q3.json").read_bytes() == original_q3
    assert sorted(p.name for p in backups.iterdir()) == ["q3.json"], "only a file it changes is backed up"
    assert (bank / "questions" / "q4.json").read_bytes() == original_q4, "an absent club already reads as rustnyc"
    rewritten = json.loads(path.read_text(encoding="utf-8"))
    assert rewritten["used"][0]["club"] == "rustnyc"
    assert rewritten == {**json.loads(original_q3), "used": [{**json.loads(original_q3)["used"][0], "club": "rustnyc"}]}
    assert load_question(bank, "q3").used_by("rustnyc").room_id == "room-a"
    # Idempotent: the second run finds nothing, writes nothing, backs up nothing.
    after_first = snapshot(bank)
    again = tmp_path / "again"
    assert migrate(bank, "--write", "--backup-dir", again) == 0
    assert "nothing to migrate" in capsys.readouterr().out
    assert snapshot(bank) == after_first and not again.exists()


def test_migrate_club_backs_up_inside_the_bank_by_default_and_never_overwrites_a_backup(bank, tmp_path, capsys):
    _used_under(bank, "q3", "nyc")
    assert migrate(bank, "--write") == 0
    made = list((bank / "backups").glob("club-rename-*/q3.json"))
    assert len(made) == 1
    assert [q.id for q in schedule.load_bank(bank)] == sorted(p.stem for p in (bank / "questions").glob("*.json"))
    capsys.readouterr()
    # A backup already there is never replaced, and then nothing is rewritten.
    _used_under(bank, "q3", "nyc")
    before = snapshot(bank)
    assert migrate(bank, "--write", "--backup-dir", made[0].parent) == 1
    assert "never overwritten" in capsys.readouterr().err
    assert snapshot(bank) == before


def test_migrate_club_default_backups_are_ignored_by_git(bank):
    # A real migration writes into the repository's bank; `git add bank` must not sweep its backups in.
    # Read as a .gitignore line rather than by running git: `just test` starts no process.
    _used_under(bank, "q3", "nyc")
    assert migrate(bank, "--write") == 0
    (made,) = bank.glob("*/club-rename-*/q3.json")
    rule = f"/{BANK.relative_to(REPO)}/{made.relative_to(bank).parts[0]}/"
    assert rule == "/bank/backups/"
    assert rule in (REPO / ".gitignore").read_text().splitlines(), f"{rule} is not in .gitignore"


def test_migrate_club_refuses_a_record_it_does_not_understand_and_writes_nothing(bank, capsys):
    _used_under(bank, "q3", "nyc")
    broken = bank / "questions" / "q4.json"
    data = json.loads(broken.read_text(encoding="utf-8"))
    data["used"] = "yesterday"
    broken.write_text(json.dumps(data), encoding="utf-8")
    before = snapshot(bank)
    assert migrate(bank, "--write") == 1
    assert "refused q4.json: does not read as a bank record" in capsys.readouterr().err
    assert snapshot(bank) == before


def test_migrate_club_leaves_every_record_whole_when_a_rewrite_cannot_be_written(bank, tmp_path, monkeypatch, capsys):
    _used_under(bank, "q3", "nyc")
    _used_under(bank, "q4", "nyc")
    before = snapshot(bank)
    real = Path.write_text

    def full_disk(self, *args, **kwargs):
        if self.name == "q4.json.club-rename":
            real(self, "{", encoding="utf-8")
            raise OSError(28, "No space left on device")
        return real(self, *args, **kwargs)

    monkeypatch.setattr(Path, "write_text", full_disk)
    assert migrate(bank, "--write", "--backup-dir", tmp_path / "b") == 1
    assert "nothing was rewritten" in capsys.readouterr().err
    assert snapshot(bank) == before, "no original touched, no staged file left behind"


def test_migrate_club_replaces_whole_files_and_a_rerun_finishes(bank, tmp_path, monkeypatch, capsys):
    _used_under(bank, "q3", "nyc")
    _used_under(bank, "q4", "nyc")
    real = schedule.os.replace

    def fail_on_q4(src, dst):
        if Path(dst).name == "q4.json":
            raise OSError(5, "Input/output error")
        return real(src, dst)

    monkeypatch.setattr(schedule.os, "replace", fail_on_q4)
    assert migrate(bank, "--write", "--backup-dir", tmp_path / "b1") == 1
    assert "already rewritten: q3.json" in capsys.readouterr().err
    # Each file is whole and reads the same either way; nothing staged is left.
    assert [q.used_by("rustnyc").room_id for q in schedule.load_bank(bank) if q.id in ("q3", "q4")] == ["room-a"] * 2
    assert not list((bank / "questions").glob("*.club-rename"))
    monkeypatch.setattr(schedule.os, "replace", real)
    assert migrate(bank, "--write", "--backup-dir", tmp_path / "b2") == 0
    assert "rewrote q4: used[0].club nyc -> rustnyc" in capsys.readouterr().out
    assert migrate(bank) == 0 and "nothing to migrate" in capsys.readouterr().out
