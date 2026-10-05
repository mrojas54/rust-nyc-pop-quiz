# PQ-25: Schedule and sync

BUILDPLAN.md T-20 (M3).

`popquiz schedule` / `sync`: push the affirmed question to the server over the §8.3 channel and write the **static fallback** file and its **host sheet** with T-26's build; pull `used`; reserve count, trend, and the low-reserve warning; a meetup from reserve with no generation

Criteria: AC-75–77, AC-89, AC-92, AC-101, AC-102, `SPEC.md` §12, G-12
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-11, T-18, T-19, T-25, T-26
BUILDPLAN notes: Human track: H-11, for the first deployed push

Orchestrator notes: Calls T-26's build and host-sheet functions; pushes over T-25's channel. H-11 (admin token) is needed only for the first deployed push, not to build or test.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-10-04)

Branch `ai-c11-cc/schedule-sync` off `origin/main` @ 013076b. Mode inline-full. Resolves PQ-45 (Option A).

## CLI shape

`uv run --offline --no-sync python -m popquiz.schedule <subcommand>`; `main(argv) -> int`, argparse subparsers, errors as one plain sentence on stderr prefixed `popquiz.schedule:` (fallback.py's convention).

- `schedule <id> --date YYYY-MM-DD (--room <base-url> | --no-push) --out <dir> [--bank <dir>] [--home-link URL] [--threshold N] [--lead-time N]`
- `sync --room <base-url> [--bank <dir>] [--threshold N] [--lead-time N]`
- `reserve [--bank <dir>] [--threshold N] [--lead-time N]`

`--date` is required (no "today" default: the organizer names the night; a hidden clock is a second input to reason about). Parsed with `date.fromisoformat` to a `datetime.date`.

## Files

- `pipeline/src/popquiz/schedule.py` (new content): `gate`, `arrange`, `_rng`/`_shuffled` (ported from build_deck.py:75-93, local to this module, never slot.py), HTTP helper with bounded backoff, `push`, `pull_used`, `merge_used`, `reserve_lines`, `ledger_report`, `main`.
- `pipeline/src/popquiz/bank.py`: F-30 only — `Used.fit: Fit | None`; the reader reads `used_raw.get("fit")`. `_dump` already drops `None` (absent is absent). The `else Used(` line stays verbatim (room/tests/used.rs:152) and the class's field lines keep the `name: type` shape (used.rs:180).
- `pipeline/tests/test_schedule.py` (new); `pipeline/tests/test_bank.py` (F-30 round trip).
- `pipeline/README.md`, `room/README.md` (~203-214), `bank/README.md` if it describes used/reserve.

## Gate (AC-72, G-12, G-10)

Hard error, nothing written, nothing sent, exit 1, when: no `verified` block; not accepted (`audit.is_accepted`); `affirmed_by` missing; `affirmed_at` missing (each half named); `used` present (never twice). Reserve membership is `audit.in_reserve` (imported); the gate's job is to name which part fails, and it asserts `in_reserve` agrees. Comment in code: the two-trace-steps rule is affirm's (T-18), not scheduling's; fallback.bake independently refuses a <2-step trace anyway.

## Arrangement (AC-23, G-1)

`arrange(question, day) -> Question`, pure: `slot = slot_for_day(day)` (date only, no n_options argument); `correct = bank.correct_index(question)` (None → BankError); distractors = the four non-correct options shuffled by a local `_rng("distractors", day.isoformat(), question.id)` Fisher-Yates (the MVP's `shuffled`); insert the correct option at `slot`; `dataclasses.replace(question, options=...)`; assert `correct_index(arranged) == slot`. `why_tempting` rides on Option. No history, ledger, bank state, previous slot.

**Distractor rule chosen: the MVP's date+id shuffle, not bank order.** Why: bank order is the generator's order. Preserving it would carry any generator ordering habit (e.g. distractors in ascending value, `does not compile` always last) onto the wall, where the correct option could be spotted as the one that breaks the pattern — a position tell through the back door. A shuffle keyed on (date, question id) destroys that and is still a pure function of the date and the record. The slot itself is untouched: it comes only from slot_for_day.

Audit compatibility: `audit.ledger_violations` refuses any function that both calls slot_for_day and does file I/O (open/write_text/...). `arrange` does no I/O; the CLI calls `arrange`, never slot_for_day. `slot_call_violations`: called with the date only. No string constant naming the MVP ledger.

## Push (AC-101, SPEC §8.3)

`PUT {room}/admin/questions/{id}`, body `json.dumps(question_to_dict(arranged))`, `Content-Type: application/json`, `Authorization: Bearer <token>`; token from `os.environ["POPQUIZ_ADMIN_TOKEN"]` only (missing/blank → refuse before anything, naming the variable, never a value). `urllib.request` only, via a module-level `_urlopen` seam the tests replace. Retries: connection errors (URLError/OSError/timeout) and 5xx retried up to 3 times (4 attempts) with waits 0.5 s, 1 s, 2 s (injectable sleep); 4xx never retried. 201/200 → `scheduled: new` / `scheduled: replaced`. 400/409/413 → the room's `reason` verbatim on stderr (413 has a reason in admin.rs; fall back to a fixed sentence if the body has none), exit 1. 401 → "the room refused the admin token (401)", nothing echoed. Error messages are built from status codes and the room's reason only — never from the Request object, headers or the body.

## Push/write order and --no-push

1. load → gate → arrange → `fallback.bake(arranged)` and `fallback.host_sheet(arranged)` pre-checked in memory (so a record the fallback would refuse is refused before anything is sent);
2. push (unless `--no-push`);
3. only on success, `fallback.write_fallback(arranged, out/<id>.html)` — the same arranged object that was pushed. A refused push leaves nothing behind.
`--no-push` builds the files alone (the night the room is unreachable; AC-77's spirit); same gate, same arrangement, so the file and a later push agree. `--room` and `--no-push` are mutually exclusive and one is required.

Summary printed: id, date, room answer (or "not pushed"), the two paths. Not the letter (it is in the files).

## Sync (AC-92, SPEC §3.1 used row)

`GET {room}/admin/used`, same bearer and retry rule. Group entries by `question_id`. For each id:
- not in the bank → listed as skipped (`smoke-q3`, `burst-q3`), never an error;
- more than one distinct room_id in the ledger for it, or the record already holds `used` with a different room_id → refused loudly for that record, nothing written to it, continue, exit 1 at the end;
- record already holds `used` with the same room_id → left as is (idempotent, no write);
- else merge at dict level: `d = question_to_dict(q); d["used"] = entry["used"]; save_question(bank, question_from_dict(d))` — no `Used(` in schedule.py.
Then the ledger report: every synced record (`<id>  <meetup_date>  room <room_id>`), and one line per room whose fit is not `fits`, `null` reading "the wall never reported a verdict". Empty ledger → says so.

## F-30

`Used.fit: Fit | None`; reader `fit=used_raw.get("fit")`; `None` written as an absent key by `_dump`; missing key reads back `None`. `just test-room` after (used.rs parses the class).

## Reserve (AC-75, AC-76; SPEC §3.3)

First lines of all three subcommands:
`reserve: N question(s) ready (accepted, affirmed, unused) — N meetup(s) of runway at one question a meetup`
`trend: +A accepted awaiting affirmation, +R unreviewed, −U used in the last 90 days`
and when `N < threshold`: `warning: the reserve is below <threshold> — fewer than <lead-time> meetups of runway. Generate and review before the next meetup.`

Trend definition (decided): the bank holds no time series of the reserve, so the trend is built from what it does hold — inflow in the pipe (accepted-not-affirmed, unreviewed records) against outflow (used records whose meetup_date is within 90 days of today, NY). Honest, stateless, no new file.

Threshold/lead time: `--lead-time` meetups (default 2), `--threshold` questions (default = lead time × 1 question per meetup = 2). Warn when reserve < threshold — SPEC §3.3's literal "warn when reserve < 2 meetups". (The boot prompt says "at or below"; SPEC wins on behaviour — deviation 1.) Count uses `audit.in_reserve`, imported.

## Tests (test_schedule.py; F-30 in test_bank.py) and their mutations

Fixture: tmp copy of the committed bank; q3 affirmed by the fixture (status accepted, affirmed_by, affirmed_at). Fake `_urlopen` recording requests; `sleep` stubbed.

- AC-23: different dates → different arrangements of q3 (pick two dates with different slots); correct at slot_for_day(d) for ~60 dates across all 5 letters; bake's correct letter == LETTERS[slot]; schedule.py imports nothing from mvp/history/ledger and audit.ledger_violations/slot_call_violations are clean over pipeline sources; slot.py byte-identical to origin/main (git diff empty; skip if git unavailable). Mutation: arrange keeps stored order (answer stays E) → fails.
- AC-72/G-12/G-10: unaffirmed refused (exit 1, message, no request, no file); affirmed_by without affirmed_at refused naming affirmed_at; not accepted refused; used refused; no verified refused; committed bank: all four refused. Mutation: gate checks only affirmed_by → by-without-at test fails.
- AC-101 client: Authorization header exact; body == question_to_dict(arranged); 401 reported without token/body; 409 reason verbatim on stderr; 5xx retried 3 times then fails, 4xx not retried, connection error retried; planted token absent from stdout/stderr/exception text; missing env var refused. Mutation: retry on 4xx → test fails; include token in error → plant test fails.
- AC-102: one call writes `<id>.html` + `<id>.host-sheet.txt` beside each other; the file's baked correct letter is the slot letter; refused push writes nothing; --no-push writes files and sends nothing. Mutation: write_fallback(question) instead of arranged → letter test fails.
- sync: null fit → absent key, reads None; harness ids skipped and listed; second sync byte-identical; conflicting room_id refused, exit 1, others still written; report names clipped_x and null rooms; empty ledger said. Mutation: drop the room_id comparison → conflict test fails.
- AC-75/76: count == in_reserve count over tmp bank; warning at reserve 1 (below 2), none at 2; default lead time 2; --threshold configurable. Mutation: `<=` → test at 2 fails.
- AC-92: schedule and --no-push leave bank/ byte-identical; only sync writes used.

Planted token: lower-case word string not matching secret-scan's shapes (no `VAR=` literal ≥8 chars on one line, not the CANARY-ADMIN-TOKEN-hex shape); set via `monkeypatch.setenv(TOKEN_ENV, PLANT)`.

## Contract tensions (sides taken)

1. Warning boundary: SPEC §3.3 "< 2" over the brief's "at or below".
2. EVALUATION.md:185 lists "the verified MVP bank, already affirmed by use" as the default for the first affirmations, but the committed records carry no affirmed_by/affirmed_at; per the Orchestrator's ruling the gate reads the record's fields and refuses all four. Flag to Orchestrator; no contract edit.
3. AC-77 (test-full, network blocked) is not built here; `--no-push` + offline fallback is what this ticket contributes.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: general-purpose subagent (sonnet), 2026-10-04. 2 Critical, 5 Major, 7 Minor.

1. **Critical — F-30 vs the schema drift test** (`bank/schema/question.schema.json:239`, `test_bank.py` schema/dataclass drift). Resolution: `Used.fit: Fit | None = None` and `fit` dropped from the schema's `Used.required`, its description saying absent means no verdict (F-30). The schema file is outside the boot prompt's file list: deviation, taken because a published contract that calls `fit` required while the code writes it absent would be false. No validator reads the schema (audit does not); the drift test does.
2. **Critical — redirects carry the bearer.** Accepted. `_urlopen` is an opener with a redirect handler that follows nothing; a 3xx is refused with a plain sentence. `--room` must be https, or http to 127.0.0.1/localhost/::1 (a local room). Tests for both.
3. **Major — pre-check must render the files.** Accepted: `fallback.build_html` and `host_sheet` are rendered before the push; only the disk write follows it.
4. **Major — whole-word `Used(` anywhere in pipeline/src breaks used.rs.** Accepted; schedule.py never spells it, docstrings say "the used record". The `class Used:` body keeps only `name: type` field lines.
5. **Major — token scan constrains fixtures and docs.** Accepted: the name goes through `TOKEN_ENV`; the plant is plain words; README examples use `$(op read …)`. `just secret-scan` at the gate.
6. **Major — warning wording contradicts itself with separate flags.** Accepted: the warning now states the count, the threshold and the lead time without asserting a relation between them: `warning: N ready is below the threshold of T - generate and review now; the lead time is L meetups`. Threshold defaults to lead time × one question a meetup.
7. **Major — the trend reads a hidden clock.** Partly: `reserve_lines(today=…)` is the seam and the tests inject it; the CLI uses today's date in America/New_York (the room's zone). `--date` stays required because it decides the slot; the trend's 90-day window is a display, not a decision.
8. **Minor — date parsing too loose.** Accepted: strict `YYYY-MM-DD` before `fromisoformat`; test.
9. **Minor — git-diff test brittle.** Accepted for another reason too: `test_runner.py` forbids `subprocess` in tests. Replaced by `slot_path_violations(slot_source()) == []`; the byte-identical diff is checked at the exit gate and reported in DONE.
10. **Minor — `--out` inside the repo.** Accepted: refused, since both files hold the answer.
11. **Minor — audit compatibility test.** Accepted: the test runs `ledger_violations` and `slot_call_violations` over every pipeline source; schedule.py imports nothing from mvp and no history.
12. **Minor — duplicate `_rng`.** Kept the port the boot prompt asks for (never import into or edit slot.py's private API), and a test holds it equal to `slot._rng`'s stream.
13. **Minor — a scheduled question still counts in the reserve.** Correct under SPEC 3.3/G-10 (used at release); no extra output line (the brief limits the summary). Said in the README.
14. **Minor — `just` recipes.** The justfile is a shared file not cleared for this ticket: QUESTION comment to the Orchestrator, no edit. `uv run --offline --no-sync python -m popquiz.schedule …` works.
