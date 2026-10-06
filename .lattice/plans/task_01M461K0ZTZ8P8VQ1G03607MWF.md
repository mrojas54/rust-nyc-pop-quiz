# PQ-54: Two live rooms on one question: releasing one publishes the answer at /last while the other is pre-reveal

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-8** (G-3, Major, new): Two live rooms on one question: releasing one publishes the answer at /last while the other is pre-reveal
- Evidence: rooms.rs:846-886 (create_for checks used only), rooms.rs:925 (global take_home), routes.rs:639; probe test
- What would close it: Refuse create_for while another unreleased room holds the question, with a test

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-10-06)

Re-verified on main @ e239346: `create_for` (`room/src/rooms.rs:846`) checks only `used` (before any lock), then looks the question up under `questions`, then takes `rooms` and inserts. Nothing refuses a second room on a held question. `act` (`:890`) appends to `used` and sets the machine-wide `take_home` under the `rooms` lock. `run_again_for` (`:959`) acts the source room under `rooms`, drops it, then calls `create_for`.

## The predicate (ruling 1)

A room record holds question `q` iff `room.question_id() == q && room.phase() != Phase::Released && room.ended(now).is_none()`. `now` is the `now` passed to `create_for` (the caller's clock reading, as `act` uses). `lifecycle::verdict` returns `None` for `Released`, so the `!= Released` clause is needed on its own; a quiet or expired unreleased room cannot be acted on (`act` returns `Ended`), so it can never release or set `/last`. Code agrees with ruling 1; no deviation.

## Where the check sits, under which lock (ruling 2)

`create_for` becomes one critical section:

1. `let questions = lock(&self.questions);` — held until the insert. Lock order `questions` then `rooms` (as `schedule` documents), then `rooms` then `ended` (as today).
2. `let mut rooms = lock(&self.rooms);`
3. **Hold check** → `Refused(HELD)`.
4. **Used check** (moved under `rooms`) → `Refused(USED)`, sentence unchanged.
5. Question lookup → `Refused("No question is scheduled with that id.")`.
6. Code draw under `ended`, `Room::create`, insert, then drop all.

Decisions recorded:
- **The used check moves under `rooms`.** `act` appends to `used` while holding `rooms`, so checking `used` under `rooms` makes "released" and "used" one atomic fact for `create_for`. Today a create can read `used` = absent, a release of the holding room lands, and the create inserts a second room on a now-used question (G-10 broken). Under the lock that window is gone.
- **`questions` is held through the insert.** Today the question is cloned under `questions`, which is dropped before `rooms`; `schedule` could replace the record in between (no room holds it yet) and the room would run the stale record while the used ledger names the new one — exactly what `schedule`'s own doc comment says it prevents. Holding `questions` across closes that; it follows the documented order and costs nothing (no I/O inside).
- **Order of the checks: hold, then used, then not-scheduled.** Used-before-not-scheduled preserves today's precedence (an id that ran and is not scheduled still reads *already been run*). Hold-before-used is behaviourally invisible on correct code (a holding room is unreleased, so its question is not used — `used` is append-only from release), and it makes the *counts Released* mutation observable: a released room would produce the HELD sentence instead of the USED one.

## *Run it again* (ruling 3)

`run_again_for` already ends in `self.create_for(...)`, so it reaches the same check under the same lock. The source room is `Released` by then (`RunItAgain` is only the next action from `released`, `phase.rs:106`), so it is excluded by the predicate and never counts against itself. No change to `run_again_for` beyond its doc comment.

## `schedule()` (ruling 4)

Unchanged in behaviour. Doc comment only: say that a room holds its question for `schedule` until the record is deleted (released shells included), which is wider than `create_for`'s hold on purpose — and point at `create_for` for the room rule.

## The sentence (ruling 5)

SPEC §11 has no row for a create refusal (the existing *That question has already been run. Pick another.* is not in §11 either). New sentence, in its neighbour's voice:

> **Another room is running that question. Pick another.**

Constant `HELD` beside the existing literals in `create_for`. Carried to DONE under *Copy for the client*; marked pending in the PR.

## Tests (new file `room/tests/one_room.rs`, `Clocked` fixtures from `tests/common`)

| # | Case | Asserts |
|---|---|---|
| T1 | Probe P1 as a permanent test: room A on q3, put live; create B on q3 | B refused with HELD exactly; `room_count` 1; `take_home()` None; walk A to release → `take_home` is q3, and is the only one |
| T1b | Probe P1 after a previous release: release q3-again first, then A on q3 live, second create on q3 | HELD; `take_home` still q3-again |
| T2 | A on q3 released; create on q3 | USED sentence (unchanged); `run_again` from A onto q3 → USED |
| T3 | A on q3 left idle; advance `IDLE_QUIET` (no sweep) | new create on q3 Ok; also for a live room past `LATER_QUIET`; and after expiry (`ROOM_LIFETIME`) |
| T4 | Run it again: R released on q3-again, B live on q3; `run_again(R, q3)` | HELD; room count unchanged |
| T5 | A live on q3; create on q3-again | Ok (a different question is never blocked) |
| T6 | Every unreleased phase holds: for each of idle, live, closed, split, work, reveal | HELD |
| T7 | HTTP: in `canary.rs` style, `POST /rooms` twice on q3 → 409 with `body["reason"] == HELD`; `POST /rooms/{id}/run-it-again` onto a held question → 409 HELD | |
| T8 | Threads: fresh `Clocked` × 50 rounds, 8 threads behind a `Barrier`, each `create_room(q3)` | exactly one Ok, the rest HELD, `room_count` 1 |

Existing tests that create two rooms on one question (if any) will surface on the first run; each gets moved to distinct questions or an ended first room, never by weakening the rule.

## Mutations (each run with `gtimeout 900 just test-room`, reverted after)

- M1 hold check removed → T1, T4, T6, T7, T8.
- M2 predicate counts `Released` (drop `!= Released`) → T2 (HELD instead of USED), plus `canary.rs` run-it-again test.
- M3 predicate ignores `ended` → T3.
- M4 check before the lock, lock released before insert (check under its own short `rooms` lock, then re-lock to insert) → T8 only, probabilistically; reported as killed or survived with run count.
- M5 used check moved back before the lock → no deterministic test; report as survived (the release-during-create window needs an interleaving hook the code does not have) and argue the lock.

## Docs

- `room/README.md` §*Never twice* gains a paragraph *One room per question* naming HELD and the predicate.
- `docs/RUNBOOK.md` failure table: row after the USED row: `` `Another room is running that question. Pick another.` `` | *A room on this machine is open on that question and not yet released. Use that room (its saved host-screen address), or pick another question. A room that went quiet stops holding it.*

## Contract tension

None found. SPEC §4.6 *"Run it again creates a new room and the schedule refuses a used question"* is unaffected. `smoke`/`burst` 409 hints mention only the used case; out of scope (ruling 6, GAP-21) — noted in DONE.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: Sonnet subagent, 2026-10-06. No Critical. Baseline `just test-room` at e239346 (outside the sandbox; the Discord mock binds a port): 236 passed, 0 failed, 4 ignored.

- **Major 1 (M5/M6 survive) — accepted.** Add a structural test `create_for_checks_and_inserts_in_one_critical_section` in the style of `used.rs::g10_the_release_transition_is_the_only_writer`: in `create_for`'s source, `lock(&self.questions)` precedes `lock(&self.rooms)`, which precedes the hold check and `self.used.contains`, which precede `rooms.insert`; no `drop(rooms)` and no `drop(questions)` before the insert. This kills M4 (check under its own lock), M5 (used check before the lock) and M6 (questions dropped before insert) deterministically. T8 (threads) stays as the behavioural witness for M4, reported with its run count.
- **Major 2 (sentence near-collides with schedule's `A room is running …`) — accepted.** New sentence: **That question is open in another room. Pick another.** It mirrors its neighbour's shape (*That question has already been run. Pick another.*) and shares no lead words with the admin-channel refusal. RUNBOOK row: `` `That question is open in another room. Pick another.` `` | *Another room on this machine has that question and has not been released. Open that room's saved host-screen address instead, or pick another question. A room that went quiet stops holding it.*
- **Major 3 (smoke/burst 409 hint) — accepted as a DONE note.** Out of scope (ruling 6, GAP-21): smoke/burst answer any 409 with "restart it"; for the new refusal that advice is wrong, and a smoke run left unreleased on the night's question would block the real create until it goes quiet. Named in DONE as a follow-up.
- **Minor, M2's killers** — named as T2 plus `canary.rs` run-it-again test (`:203`).
- **Minor, T1 wording** — T1 asserts `take_home()` is `None` while A is live after B's refusal, then after A releases `take_home().unwrap().question_id == "q3"` and `used().all().len() == 1`.
- **Minor, T7** — split into T7a (`POST /rooms` twice → 409, reason) and T7b (run-it-again onto a held question → 409, reason).
- **Minor, fixed create order** — T1b and T4 create in a fixed order on q3/q3-again (Clocked has only those two).
- **Minor, missing cases** — schedule-still-refuses is covered by `admin.rs` (unchanged, ruling 4); add an assertion in T1 that `room_count` is unchanged after the refusal. Stale `now` across the awaited Discord check in `create_room_checked`: noted, accepted (a few seconds' staleness can only make a just-ended room still hold, which is the safe side).
- **Minor, host reload** — a reload of `/host?question=<id>` that creates again now gets the refusal while the first room is live; the RUNBOOK row tells the organizer to use the saved host-screen address. Intended.

## Reset 2026-10-06 by agent:delegator-pq54
