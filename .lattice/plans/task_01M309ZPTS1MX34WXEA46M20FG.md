# PQ-5: Sessions and the answer store

BUILDPLAN.md T-04b (M1).

Sessions and the answer store: join, capacity, answer upsert while live, refusal after close, totals frozen at close, sessions dropped at release

Criteria: AC-30, AC-34–37, AC-46, AC-56
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a
BUILDPLAN notes: —

Orchestrator notes: Touches `room/src/phase.rs` (F-10): serialized after T-04a and before T-04c.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-26)

Base: `origin/ai-c11-cc/phase-machine` @ f7faccd. Branch `ai-c11-cc/sessions`.

## Files

| File | Change |
|---|---|
| `room/src/sessions.rs` (new) | `Token` (32 random bytes, hex, from `getrandom`), `Session { token, answer: Option<Letter> }` and nothing else, `SessionMap` implementing `rooms::Sessions`. `JoinRefusal` (the six §4.1 states, each mapped to its `copy::JOIN_FAIL_*` string), `parse_code`, `parse_letter`. |
| `room/src/rooms.rs` | Additive, under `// T-04b`: trait methods on `Sessions` (`join`, `upsert`, `answer_of`, `leave`, `counts`) with defaults so `NoSessions` and the test doubles stay as they are; `AppState` gains `capacity` (default 200) + `with_capacity`, and the default factory becomes `SessionMap`; `AppState::join`, `answer`, `leave`, `buzzer_for` — each updates `present`/`answered_live` inside the same lock. |
| `room/src/routes.rs` | One `// T-04b routes` block: `POST /join` `{code}` and `PUT /rooms/{id}/answer` `{letter}` with the session token as `Authorization: Bearer`. Existing routes untouched. |
| `room/src/view.rs` | `// T-04b`: `BuzzerPayload.yours: Option<Letter>` (skipped when `None`) and `buzzer_for(room, yours)`; `buzzer(room)` = `buzzer_for(room, None)`. |
| `room/src/lib.rs` | `pub mod sessions;` (one line — not in the cleared list; see tension 5). |
| `room/tests/sessions.rs` (new) | The criteria tests below. `common/mod.rs` gains a small `call` / `create` / `join` driver. |
| `room/README.md` | A *Sessions* section: routes, the response contract for saving/saved/failed, ghost sessions, what `leave` means. |

## Decisions

1. **Joins by phase.** `idle`…`reveal` accept a join (SPEC §4's idle buzzer is *You're in.*; `present keeps counting` after close implies late joins). `released` → **already ended**. Unknown code → **unknown**; bad shape → **malformed** (normalised: trim + upper-case before checking the 32-symbol alphabet); at capacity → **full**. **Not yet open** and **closed for inactivity** ship as variants with their strings but no phase reaches them in the seven-phase machine: T-11 sets inactivity; *not yet open* has no state to hang on (tension 1).
2. **Capacity** is checked against the session count before any token is drawn; a refused join creates, reserves and bumps nothing (AC-30). Default 200, `AppState::with_capacity(n)`.
3. **Ghost sessions.** A session record is only `token` + `answer`, so there is no "connected" flag. A session whose socket dropped is still a session: it counts in `present`, in capacity, and its answer counts in `totals`, until `leave` or release. `leave(token)` is T-04c's call when a socket is gone **for good** (its grace period elapsed — never on a mere drop, or AC-37's re-attach would lose the answer). It removes the session whole and frees its slot. This keeps `answered_live ≤ present` while live and makes capacity impossible to over-commit on re-attach.
4. **Answer response contract (AC-35/36, for T-06).** `200 {"saved": "B"}` → *saved — B*; `409 {"reason", "phase", "saved": "B"|null}` → closed (or not live yet): the saved answer restated; `401` empty → token does not resolve (never joined, or released): re-join; `400` → bad letter, nothing written; anything else / no response → *failed*, showing the last 200's letter. *saving* = a request in flight. Upsert is idempotent (same letter twice changes nothing, no revision bump).
5. **Counts** are recomputed from the map and written with `Room::set_live_counts` while the same `rooms` lock is held (calling `AppState::set_live_counts` from inside would re-lock). Same writer, one lock.
6. **Close/release** unchanged in T-04a: `close_snapshot` counts each session's final answer once; `release` clears the map, so no token resolves afterwards.
7. **`yours`** in the buzzer payload is the caller's own saved answer, only on the per-session paths (join response, `buzzer_for` for T-04c). The public `GET /rooms/{id}/buzzer` never carries it. No other session's answer, nothing before `split` beyond what exists.

## Tests (`room/tests/sessions.rs`)

- **AC-28/29**: join by code (lower-case too) succeeds; each of malformed, unknown, already ended, full returns its own §11 string over HTTP; the two unreachable variants are asserted distinct and mapped to their strings.
- **AC-30**: capacity 200, 200 joins then the 201st refused `full`; session count and `present` still 200, no token created. Also with capacity 3.
- **AC-34**: five changes while live → the last is saved; close; a write is refused `409` with the saved letter restated; totals show that last letter.
- **AC-36**: a bad write (letter `F`, missing body, wrong token) leaves the previous answer intact.
- **AC-46**: host payload `present` / `answered` move on join, upsert and leave while live.
- **AC-52 (in-process)**: 200 sessions answer, a third change their minds, close: `totals` equal the per-session finals exactly, sum to `answered`.
- **AC-56**: after release, no token resolves (`401`), join says *already ended*, the map is empty.
- **AC-57**: a source scan of `sessions.rs` asserts `struct Session` has exactly `token`, `answer`; plus an exhaustive destructure in `sessions.rs` that stops compiling if a field is added.
- **`yours`**: the join response carries only the caller's letter; the public buzzer has no `yours`.

## Contract tensions (side taken)

1. *Not yet open* has no phase: rooms are born in `idle` and the idle buzzer says *You're in.* Side: §4's table — idle joins succeed; the variant ships unreached. Flag to the Orchestrator.
2. Ticket header says it *touches `room/src/phase.rs` (F-10)*; the boot prompt forbids editing it. Side: the boot prompt — nothing in T-04b needs `phase.rs`.
3. AC-58: *no request carries the participant's answer after close*. The server refuses such a write, but the guarantee that none is sent is T-06's/T-08's (letters locked in `closed`).
4. Boot suggests `/rooms/{code}/join`; axum cannot mount `{code}` beside `{id}` at the same segment. Side: `POST /join {code}` — also exactly the one-field form.
5. `lib.rs` is not in the cleared list but a new module needs one `pub mod sessions;` line. Taken as implied by "put it in `room/src/sessions.rs`".

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: fresh-eyes subagent (sonnet), 2026-09-26. No Critical findings.

- **Major 1 — counts bypass the documented `AppState::set_live_counts` seam.** Concern: README and `rooms.rs` doc name `AppState::set_live_counts` as T-04b's call; the plan writes `Room::set_live_counts` under the held lock. Resolution: accepted as a deviation and listed (tension 6). `AppState::set_live_counts` stays public for any caller outside the lock; the README and the `rooms.rs` module doc are updated to say T-04b's own paths write through `Room::set_live_counts` inside the one lock, so the room's counts and the map can never be observed apart.
- **Major 2 — AC-35 has no test.** Resolution: add `ac35_response_contract`: a scripted sequence (join, answer, change, bad letter, wrong token, close, write after close) where every response is classified into exactly one of saved / refused-with-saved / unknown-session / rejected, and after each the server's saved answer (`buzzer_for`'s `yours`) equals the last `200`'s letter; every `409` restates exactly that letter. The rendering half (exactly one of saving/saved/failed on screen) is T-06's and is said so.
- **Minor 1 — `view.rs` gets a function too, not just a field.** Resolution: listed as tension 7. `buzzer_for` is the least that can pass a per-session value in; `buzzer(room)` is unchanged in behaviour.
- **Minor 2 — AC-52 in-process check overclaims.** Resolution: the test is named `ac52_in_process_reconciliation` and the README/PR say it is a supplementary in-process check; AC-52 itself is `burst`'s, against the deployed room.
- **Minor 3 — a second 409 shape.** Resolution: documented in README's routes table and the sessions section: the answer route's `409` is `{reason, phase, saved}`, a superset of the generic `{reason}`, so a client that reads only `reason` still works.

Added tensions: **6.** counts written via `Room::set_live_counts` under the lock (Major 1). **7.** `view::buzzer_for` added beside the one field (Minor 1).

## Reset 2026-09-26 by agent:delegator-pq5
