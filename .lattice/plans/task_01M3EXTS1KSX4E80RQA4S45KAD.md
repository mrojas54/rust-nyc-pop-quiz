# PQ-32: Wire the session map into the transport

Follow-up minted by the Orchestrator 2026-09-26. T-04b (PQ-5) and T-04c (PQ-6) were built in parallel off the phase-machine branch, each with its own seam. Once both PRs are on main: make T-04b's session map implement ws::SessionTokens (resolve(room, token), saved(room, session), gone(room, session); a later resolve wins over a racing gone), layer Extension(Transport::new(state, map)) over router_with, keep the // T-04c routes block last in routes(), have every non-HTTP room writer call Transport::changed, and run both tickets' test suites plus an end-to-end in-process test: join, answer, transitions, reconnect with the same token (AC-37, AC-81 at small scale). Criteria AC-37, AC-81, AC-46. Depends on PQ-5 and PQ-6. Fast-track. PR base: origin/main. Blocks PQ-7, PQ-8, PQ-9 in practice: they consume the wired room.

# Plan (delegator, 2026-09-26)

## Shape

The session map is per room and lives inside `AppState`'s rooms lock (`Entry { room, sessions: Box<dyn Sessions> }`), so `SessionTokens` is implemented by **an adapter over `AppState`** (`impl ws::SessionTokens for AppState`), not on `SessionMap` itself: a `SessionMap` knows no room id and is reachable only under the lock. The adapter delegates to two new `rooms::Sessions` methods that `SessionMap` implements.

## Files

- `room/src/sessions.rs` (// PQ-32)
  - `SessionMap` gains one map-level field, `ids: std::hash::RandomState` (per map, not per session: `Session` keeps exactly `token` + `answer`, AC-57's compile-time check untouched).
  - `SessionId` for a token = `ids.hash_one(token)`. Keyed and one-way, so the handle carries no token material; stable for the session's life.
  - `join` redraws a token whose id collides with an existing session's (same loop that already redraws a colliding token), so within a room id ↔ token is a bijection. O(n ≤ 200) per join.
  - `impl Sessions`: `session_id(token) -> Option<SessionId>` (token names a session) and `saved_by_id(id) -> Option<Option<Letter>>` (scan ≤ 200, on attach only). `saved` returns that session's own answer, nothing else (AC-57).
- `room/src/rooms.rs` (additive, // PQ-32)
  - `Sessions` gains `session_id` and `saved_by_id` with `None` defaults (NoSessions / FakeSessions compile unchanged).
  - `impl crate::ws::SessionTokens for AppState`: `resolve` → the room's `session_id(token)` (a released room's map is empty → `None`); `saved` → `saved_by_id(id).flatten()`; `gone` → **no-op**.
- `room/src/routes.rs`
  - T-04c block: the default `Transport` is `Transport::new(state.clone(), state.clone())` — the real map — instead of `NoTokens`. Kept through the existing `provide` layer (insert-if-absent) rather than a literal `Extension` layer in `router_with`, because an inner `Extension` layer would overwrite a test's outer one (`TestTokens`, `NoTokens`). Block stays last.
  - T-04b block: `join` takes `Extension<ws::Transport>` and calls `transport.changed(&room_id)` on success. **Found in the audit:** `POST /join` is not under `/rooms/{id}/…`, so the `notify` layer does *not* publish it — the join's `present` bump would otherwise sit unpushed until the next write (AC-46).
- `room/src/lib.rs` — doc: `router()` / `router_with` serve the wired transport over the real session map.
- `room/src/ws.rs` — docs only: `NoTokens` is for tests; the default is `AppState`.
- `room/tests/transport.rs` — rename `without_a_session_map_…` to the wired truth (default router: walls served, an unknown token refused `4401`).
- `room/tests/wiring.rs` (new) — real map, loopback listener, default router (no Extension layered): wall + host attached → `POST /join` → wall & host get exactly one frame, `present` 1 (AC-46) → buzzer attaches with the returned token → first frame = full current state = `GET …/buzzer` projection, `session.saved` null (AC-37) → host `put-on-screen` → `PUT …/answer` → one frame each for wall/host/buzzer per revision, host `answered_live` 1 (AC-46) → a second buzzer attach carries `saved` from the real map (AC-37) → drive through `closed` (one frame per revision each, one `phase` across all — AC-81) → drop the socket → counts unchanged (gone changes nothing; ghost keeps its slot) → re-attach with the same token → first frame current state, `saved` intact (AC-37) → `release-room` → the token resolves to nothing (re-attach closed `4401`; answer `401`).
- `room/README.md` — the "until T-04b wires in … NoTokens" and "tests use TestTokens" sentences become the wired truth; one *Wiring* paragraph under *The transport*; Sessions' `leave` rule unchanged.

Not touched: `answers.rs`, `twins.rs`, `phase.rs`, `main.rs`, `view.rs` (attach frame already carries `session.saved`; `yours` stays on the HTTP path), `Cargo.*`, justfile, `transport_full.rs` (built on `TestTokens`; left as is, run once).

## Writer audit (every non-HTTP room writer calls `Transport::changed`)

Writers of `Room.revision`: `Room::act` (host routes under `/rooms/{id}/…` → notify ✓), `Room::set_live_counts` via `AppState::join` (`POST /join` → **not** notify → fixed explicitly), `AppState::answer` (`PUT /rooms/{id}/answer` → notify ✓), `AppState::leave` (no caller yet; T-11's grace/reaper will call it and must call `changed`), `AppState::set_live_counts` (no caller), `Room::record_fit` (no route yet; T-05). `run_again` makes a new room; the old one's revision doesn't move. No non-HTTP writer exists today.

## Choices / deviations

1. Adapter over `AppState`, not `SessionMap` (why above).
2. `SessionId` = keyed hash of the token under a per-map `RandomState`, collisions redrawn at join; no per-session field.
3. `gone` is a no-op: a drop changes no count and removes nothing (PQ-5's binding rule). "A later `resolve` wins over a racing `gone`" is therefore free.
4. Wired by default through `provide`'s default rather than an `Extension` layer in `router_with` (why above).
5. `join` pokes the transport itself (notify's path filter misses `/join`).

Criteria: AC-37, AC-46, AC-81 (small scale, real map). Existing suites: `tests/sessions.rs`, `tests/transport.rs` stay green; `transport_full.rs` run once.

## Reset 2026-09-26 by agent:delegator-pq32
