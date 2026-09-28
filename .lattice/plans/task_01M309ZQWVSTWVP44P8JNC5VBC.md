# PQ-14: Room lifecycle and the used ledger

BUILDPLAN.md T-11 (M2).

Lifecycle: 4 h expiry, room deleted at release, totals with expiry, **`used` written at release**, take-it-home rebuild at release, *Run it again* refuses a used question

Criteria: AC-56, AC-57, AC-69, AC-92, G-4, G-10
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04b
BUILDPLAN notes: **The only writer of the used-question ledger.**

Orchestrator notes: The only writer of the used-question ledger (G-10). Touches `room/src/phase.rs` (F-10): serialized.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-27)

Base: `origin/main` @ 934e6c8. Branch `ai-c11-cc/room-lifecycle`. `phase.rs` is **not** changed: no transition changes; the release side effects hang on the `released` transition inside `Room::act` / `AppState::act` in `rooms.rs`.

## Shape in one paragraph

A new `room/src/lifecycle.rs` holds the clock seam (`Clock` trait, `SystemClock`, the one `SystemTime::now()` call in the crate's lifecycle path), the bounds (`ROOM_LIFETIME` stays in rooms.rs; `IDLE_INACTIVITY = 30 min`, `LATER_INACTIVITY = 20 min`, §4.6), the `Ended` verdict (`Expired | Inactive`) that a room has at a given instant, and the reaper (a tokio task that calls `AppState::sweep(now)` every 30 s and pokes the transport for each room it deleted). A new `room/src/used.rs` holds the used ledger (`UsedLedger`, append-only, `pub(crate) fn append` called from exactly one place), the `UsedRecord {meetup_date, room_id, released_at, fit}` in the pipeline's JSON shape, `UsedEntry {question_id, used}`, the meetup-date rule (America/New_York, as a constant in `used.rs`), RFC 3339 formatting, and `TakeHome` (the take-it-home snapshot). `rooms.rs` wires them: the release transition builds the snapshot while the machine still holds the reveal witness, writes the used record, hollows the room and drops every session; the sweep deletes expired/inactive rooms whole and leaves a tombstone (id, code, why it ended — nothing else) so joins and host commands get the right refusal.

## Files

| File | Change |
|---|---|
| `room/src/lifecycle.rs` (new) | `Clock`, `SystemClock`, `ManualClock` (pub, for tests and the `test-full` shape), `Ended`, `ended(room, now)`, bounds, `spawn_reaper(state, transport)` |
| `room/src/used.rs` (new) | `UsedLedger { entries: Mutex<Vec<UsedEntry>> }` with `pub(crate) fn append`, `pub fn all() -> Vec<UsedEntry>`, `pub fn contains(question_id)`; `UsedRecord`/`UsedEntry` (`Serialize`, field names = `bank.py`'s `Used`); `meetup_date(created_at)`, `rfc3339(t)`; `TakeHome` + `TakeHomeOption` (`Serialize`) |
| `room/src/rooms.rs` | `Room`: new `last_host_action` (writer: `create`, `act`); release arm builds `Parting {take_home, fit}` from `self.open()` **before** `self.machine = next` (the witness is the reveal machine's), then hollows (`frozen = None`, counts 0, `fit = None`); `record_fit` is a no-op once released. `AppState`: `released_questions` → `used: UsedLedger`; `take_home: Mutex<Option<TakeHome>>`; `ended: Mutex<HashMap<id, Tombstone>>`; `clock: Arc<dyn Clock>` + `with_clock`; `act` on release: append used, store snapshot, `sessions.release()`; `sweep(now) -> Vec<String>`; lazy `ended` check in `join`, `answer`, `act`, `run_again`, `buzzer_for`; tombstone lookup gives `AlreadyEnded` / `ClosedForInactivity`; `create_for` refuses a used question (same string as today) and never reissues a tombstoned code; accessors `used()`, `take_home()`, `room_count()`, `tombstone_count()` for inspection |
| `room/src/sessions.rs` | none planned (`release` already drops the map; kept) |
| `room/src/lib.rs` | `pub mod lifecycle; pub mod used;` + doc lines |
| `room/src/routes.rs` | under `// T-11`: spawn the reaper once when the router is built inside a tokio runtime (`Handle::try_current`), poking the default transport; map an ended-room refusal on host routes to `409 {refusal, reason}` (reusing `JoinRefusal` slugs and copy keys) |
| `room/src/view.rs` | `TakeHome` lives in `used.rs`; view.rs gets nothing unless the reviewer asks (decision: the snapshot is not a projection of a live room) |
| `room/tests/lifecycle.rs` (new) | AC-69, AC-56, AC-57, inactivity, sweep, snapshot, `#[ignore]` test-full shape |
| `room/tests/used.rs` (new) | AC-92, G-10 never-twice, single-writer scan, bank.py shape |
| `room/tests/common/mod.rs` | under `// T-11`: `clocked_state()` helper (TestAuth, q3, a `ManualClock`) and `run_to(state, room, phase)` |
| `room/README.md` | *What is here* + seams: `UsedLedger::all()` (T-25 `GET /admin/used`, T-20 sync), `AppState::take_home()` (T-12 `/last`), `Clock` / reaper |

## Decisions (open choices, each with why)

1. **Sweep = a reaper task + a lazy check on touch.** Every entry point asks `lifecycle::ended(room, clock.now())` and refuses, so there is never a window where an expired room admits a join. **Only the sweep deletes**, so there is one deletion path, and it returns the deleted ids so the reaper can poke the transport (sockets close `4404`, the wall keeps its last frame, which is what wall.js already does). A lazy delete inside `join` could not poke the transport, and sockets would hang open. The clock is injected (`AppState::with_clock`), and tests call `sweep(now)` directly with a `ManualClock`. The task spawns from `routes::routes` only when a tokio runtime is present, so sync tests and `router()` are unaffected.
2. **Released room is hollowed, not removed, until its 4 h expiry.** SPEC §4.6 says the room is deleted with its sessions at release. But the `released` row of §4 (G-6's seventh phase) still has to render: *Let's go to the bar.* with the link and QR on the wall, the room code on the buzzer, and *Run it again* on the host phone, whose route needs the room's organizer. `ws.rs` and `view.rs` render only from a `Room`, and `ws.rs` is not mine. So at release everything the room held about people and answers is deleted: every session, `totals`, `answered`, the §4.5 verdict, `present`/`answered_live`, and `fit` (moved into `used`). What stays is the phase shell: id, code, join url, organizer, host session, phase `released`, and the `Arc` to the scheduled question, which every other room on that question shares anyway. The shell is deleted whole at `expires_at` by the sweep. The AC-56 test inspects `AppState` directly and asserts: no session, no totals/answered/verdict, counts 0, and the only survivors carrying question data are `used` and `take_home`, neither holding a count. **Deviation, flagged.**
3. **Tombstones.** An expired or inactive room is deleted whole and leaves `{id, code, ended: Expired|Inactive, at}` for `ENDED_MEMORY = 4 h`, so a join says *already ended* / *closed for inactivity* (AC-29) and not *unknown*, and a new room never reuses a remembered code. A tombstone holds nothing per-person and no count. Released shells answer *already ended* through their phase, as today.
4. **Inactivity** per §4.6: `idle` with no host action for 30 min, or `live`…`reveal` with no host action for 20 min. `released` shells are not closed for inactivity (already ended) and go at 4 h. `last_host_action` is written by `create` and every successful `act`; a refused command is not activity.
5. **Totals with expiry.** Totals exist only inside the room (`Frozen`), so they expire with it: deleted at release (hollow) and at expiry/inactivity (whole-room delete). Nothing that survives release carries a count (AC-56, D-12). The boot prompt's "totals that survive release" contradicts SPEC §4.6, AC-56 and D-12. The criteria win: **no totals survive release.** Deviation, flagged.
6. **Snapshot carries no counts.** The boot prompt asks for "the counts at release (AC-95's count from release)". AC-95 as amended, AC-56, EVALUATION AC-56/AC-95 and SPEC §13 all say take-it-home carries **no count** and no most-chosen option (D-12). The criteria win. Deviation, flagged.
7. **Snapshot contents** are what the witnessed reads give, copied out while the reveal witness exists: `meetup_date`, `question_id`, `source`, `colour: true`, the five options `{letter, text, correct: bool}` plus `correct` and `mark: "✓"`, the whole trace `0..=M-1`, `explains.what`, `explains.takeaway`, and `receipt {heading, lines}`. **Not reachable without touching `answers.rs` internals:** the `why_tempting` text for *every* incorrect option (the vault lends only the room's middle one via `Verdict`) and the verified record's detail rows for *How we know* (full `-Vv`, target, flags, Miri configs). T-12 needs both. I will not reach inside the sealed module. The snapshot's shape leaves `why_tempting: []` out rather than fake it, and a Lattice comment asks the Orchestrator for a witnessed read (an additive `Revealed` field) under T-12 or a follow-up.
8. **Typed accessor, no route.** `AppState::take_home() -> Option<TakeHome>` (`TakeHome: Serialize`) plus a JSON fixture of its shape in `room/tests`. No `/last.json` now, for two reasons. The snapshot then cannot leak before release, because nothing serves it (the canary stays trivially green, and a test asserts it is `None` in every phase before the first release). And T-12 owns `/last` and decides the page's transport. It is rebuilt (replaced whole) at every release.
9. **Used record.** `{meetup_date: "YYYY-MM-DD", room_id, released_at: RFC 3339 UTC "…Z", fit}`. `meetup_date` is the **civil date of `created_at` in America/New_York** (`used::MEETUP_ZONE`, a US-Eastern DST rule implemented in `used.rs`, no new crate). Why `created_at`: a room made at 23:50 and released after midnight belongs to that meetup. It is a constant in `used.rs` because `config.rs` is not mine. T-09/T-20 can move it into config. `fit` is the room's `fit` at release, which is the last verdict the wall reported, during `reveal`. If the wall never reported one, `fit` is JSON `null`: nothing observed it, so nothing is written (G-2's spirit). The pipeline's `Fit` literal does not admit null. Contract tension, flagged to the Orchestrator. The ledger entry is `{question_id, used: UsedRecord}`, because the pipeline stores `used` on the question.
10. **Never twice.** `create_for` (used by both `POST /rooms` and `run-it-again`) refuses a question present in the ledger with the existing string *That question has already been run. Pick another.* (already asserted by `canary.rs`). No new copy, and `copy.rs`/`copy.js` stay untouched.
11. **Ended-room host commands** answer `409 {refusal: "already_ended"|"closed_for_inactivity", reason: <JoinRefusal message>}`, reusing PQ-8's copy keys through `JoinRefusal::message()`. After the tombstone is forgotten they answer 404, like any unknown room.

## Tests by criterion (all in `test`)

- **AC-69** `lifecycle.rs`: `a_room_is_gone_at_four_hours` (ManualClock: at 4h−1s joins work; at 4h join → `already_ended`, host command → 409 already_ended; `sweep` deletes room + sessions, `room_count()==0`, tombstone present); `expiry_refuses_before_the_sweep_runs`; `the_reaper_closes_sockets_of_a_swept_room` (Live harness, socket closes 4404); `#[ignore] test_full_open_room_runs_to_release_with_auth_down` (shape for T-10).
- **§4.6 inactivity**: `idle_room_closes_after_thirty_quiet_minutes`, `later_phase_closes_after_twenty_quiet_minutes`, `a_host_action_resets_the_quiet_clock`, `a_refused_command_is_not_activity`; join → `closed_for_inactivity`.
- **AC-56 / G-4**: `nothing_per_person_survives_release` (inspect `AppState`: session_count 0, no totals, host `answered` absent, counts 0, tokens resolve to nothing); `totals_die_with_an_expired_room`; `the_survivors_carry_no_count` (serialize `used.all()` and `take_home()`, and assert no key `totals|count|answered|present|middle` and no number but in the trace).
- **AC-57** static: `no_participant_identity_anywhere` (scan `room/src/*.rs` struct fields for identity words, and pin `UsedRecord`, `UsedEntry`, `TakeHome` fields with no-`..` patterns like `sessions.rs`'s).
- **AC-92 / G-10** `used.rs`: `used_is_written_at_release`, `nothing_is_written_at_reveal_or_before`, `an_expired_room_records_nothing`, `an_inactive_room_records_nothing`, `scheduling_a_question_writes_nothing` (AppState::new with questions → ledger empty), `the_pipeline_constructs_used_only_when_reading_the_bank` (scan `pipeline/src` for `Used(`: only `bank.py`'s reader), `the_release_transition_is_the_only_writer` (scan `room/src`: `.append(` on the ledger called once, inside the release branch of `AppState::act`), `the_record_has_the_pipelines_shape` (keys == `bank.py` `class Used` field names, parsed from the file; `meetup_date` NY date incl. the DST edges and the after-midnight case).
- **Never twice**: `run_it_again_refuses_the_used_question`, `create_refuses_the_used_question`, `run_it_again_on_a_fresh_question_makes_a_new_room`.
- **Snapshot**: `take_home_is_absent_until_the_first_release`, `take_home_is_rebuilt_at_each_release`, `take_home_shape_matches_the_fixture` (`room/tests/fixtures/take_home.shape.json`, keys and types only, so no program output is written down).

Exit: `just test-room` time, `just canary`, `just test-web` untouched and green.

## Contract tensions (sides taken)

- T1 boot "room deleted at release" vs. the released phase needing to render → hollow shell until 4 h (Decision 2).
- T2 boot "totals that survive release" vs. SPEC §4.6/AC-56/D-12 → none survive (5).
- T3 boot "snapshot with counts at release" vs. AC-95/AC-56/§13 → no counts (6).
- T4 §13 needs every `why_tempting` and the verified detail, which are unreachable through the sealed module's public surface → gap named for T-12 (7).
- T5 `bank.py` `Fit` has no null, but a room can release unmeasured → `null`, flagged (9).
- T6 `mvp/tools/build_deck.py` still writes `mvp/answer-history.json` at build (line 233). `mvp/**` is contract-protected. The built system's builds (T-20/T-26) do not exist yet. The room crate writes nothing at build, and a test asserts it. Flagged to the Orchestrator.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

1. **Major, release data crossing from `Room::act` to `AppState`.** Concern: nothing carries the snapshot out of `Room::act`. Resolution: `Room::act`'s `Released` arm runs **before** `self.machine = next`. It calls `self.open()`, which works because the machine is still `reveal` and the witness is the machine's own, and builds a private `Parting { take_home, fit }` stored in a private `parting: Option<Parting>` field. `AppState::act` takes it with `room.take_parting()` right after `room.act` returns, under the same lock, then appends `used`, stores `take_home` and drops the sessions. A test asserts the snapshot is non-empty and holds the trace's final step.
2. **Major, two sources of "now".** Resolution: one clock. `AppState` holds `clock: Arc<dyn Clock>` and exposes `now()`. The three route handlers pass `state.now()` instead of `SystemTime::now()` (routes.rs, a one-token change at each call site). `create_room`, `act` and `run_again` keep their explicit `now` parameter and use it for **both** the timestamps and the ended check. `join`, `answer` and `buzzer_for` use `self.now()`. Tests set the `ManualClock` and pass `clock.now()`, so every path sees the same instant.
3. **Minor, host refusal register.** Resolution: accepted. Host commands on an ended room answer `409 {refusal: "already_ended"|"closed_for_inactivity", reason}`, where `reason` is a plain host diagnostic ("This room has ended." / "This room closed for inactivity."), the same convention as `phase::Refused` (an API diagnostic, not §11 copy). Participant copy is not borrowed.
4. **Minor, released shell and question content.** Resolution: accepted. Added test `a_released_room_serves_no_question_content`: the wall, buzzer and host payloads of a released room carry no `source`, `options`, `hint`, `trace`, `split`, `counts` or `read_aloud`.
5. **Minor, phase.rs reservation.** Resolution: `phase.rs` is not touched. The completion comment says so, so its serialization can be released.

## Orchestrator ruling applied (2026-09-27, ev_01M3JJT6M70ZSF2H3TFWJM70EM) — overrides Decisions 2 and 5

Release drops the sessions and the live counts only. The anonymous per-option `totals`, the frozen `answered` and the §4.5 verdict stay on the released shell and expire with it at 4 h (AC-56 read literally). The snapshot and the used record carry no count (§13 names none). T-4's witnessed read is PQ-15's; `fit: null` stands (F-30); T-6 stays out of scope. Commit `Keep the anonymous totals on a released room until it expires`.
