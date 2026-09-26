# PQ-6: WebSocket transport and reconnect

BUILDPLAN.md T-04c (M1).

Transport: one broadcast per room over WebSockets, reconnect with the same session token, host and wall subscriptions

Criteria: AC-37, AC-41, AC-81
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a
BUILDPLAN notes: —

Orchestrator notes: Touches `room/src/phase.rs` (F-10): serialized after T-04b.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-26)

Branch `ai-c11-cc/transport` off `origin/ai-c11-cc/phase-machine` @ f7faccd (PR #15).

## Files

| File | Change |
|---|---|
| `room/src/ws.rs` (new) | `SessionTokens` trait + `NoTokens`; `SessionId`; `Transport` (the hub: per-room channels, publish, the buzzer registry); the three upgrade handlers; the notify middleware; `serve()` (TCP_NODELAY via `tap_io`). |
| `room/src/routes.rs` | One delimited `// T-04c routes` block before `with_state`: `GET /rooms/{id}/ws/wall`, `/ws/buzzer`, `/ws/host`; a layer that supplies a default `Transport` when none is layered on; a layer that pokes the room's channel after every non-GET request under `/rooms/{id}/…`. Additive only. |
| `room/src/lib.rs` | `pub mod ws;` (and the module list line in the doc comment). |
| `room/Cargo.toml`, `Cargo.lock` | `axum` gains `ws`; `tokio` gains `sync`, `time`; `futures-util` non-optional; `spike` keeps `tokio-tungstenite` + `rustls`. Dev-dep `tokio-tungstenite = "0.29"` (already in the lock) for the test client. No new packages expected; `cargo check --features spike` kept green. |
| `room/tests/transport.rs` (new) | hermetic loopback tests (below). |
| `room/tests/transport_full.rs` (new) | the 200-buzzer AC-81 test, `#[ignore]`d so `test` never runs it. |
| `room/tests/common/mod.rs` | under `// T-04c`: `TestTokens` (token → session, saved answer, a `gone` log). |
| `room/README.md` | a *Transport* section: routes, attach protocol, frame shape, close codes, the seam for T-04b, the mapping for T-21's `burst`. |
| `justfile` | new recipe `test-transport-full` and its name on the `test-full:` dependency line. Nothing else. |

## Design

- **Channel: `tokio::sync::watch`, one per viewer kind per room (three per room).** Every frame is the *whole* state, not a delta, so a subscriber only ever needs the latest one. `watch` gives exactly that: a slow phone skips superseded states instead of lagging, and there is no `Lagged` path where a reveal can be dropped (the spike had to count those). A `broadcast` would buffer stale states per receiver for no benefit. Values are pre-serialized `Arc<str>`: each payload is built and serialized **once per revision per viewer kind**, in one place (`Transport::changed`), and every subscriber gets the same bytes.
- **What triggers a push.** `rooms.rs` has no change hook and is not mine to edit, and polling `revision()` is forbidden. So: `Transport::changed(room_id)` reads the room once under the rooms lock (revision + `view::wall`/`view::buzzer`/`view::host`), and publishes only if the revision is newer than what the channel last carried (monotonic check under the hub lock, so two racing pokes can never publish out of order and a no-op poke sends nothing). It is called (1) by a middleware after every non-GET request whose path is `/rooms/{id}/…` — all eight host actions, and T-04b's/T-05's HTTP writers if their routes sit before the T-04c block — and (2) by the transport itself after every call into the session map (`gone`, attach). Any other writer (T-04b if it takes answers over the socket, T-11's reaper) calls `Transport::changed`. Documented as the seam.
- **Frame shape.** `{"t":"state","revision":N, …the view payload flattened…}` — the payload's own `phase` is the frame's only `phase` (AC-81: exactly one). A buzzer's **attach** frame alone adds `"session":{"saved":<letter|null>}`, so broadcasts stay shared bytes.
- **Attach protocol.** `wall`: no credential (public projection, same as `GET …/wall`); unknown room → `404` before upgrade. `buzzer` and `host`: after upgrade the client's first text message is `{"t":"attach","token":"…"}` (host: its bearer through `HostAuth::authorize_host`; buzzer: its session token through `SessionTokens::resolve`). Chosen over a query string because `host_resume_url` deliberately keeps the host session in a `#fragment` out of logs; a URL credential would undo that. A bad credential closes with `4401` and says nothing else (AC-70); no attach within 10 s closes `4408`.
- **On attach, full current state first** (§4.3, AC-37): the transport pokes the room, subscribes, then sends the channel's current frame before anything else; then one frame per new revision.
- **Reconnect / replacement.** Per room, `session → (connection id, kick)`. A second socket for the same session kicks the first (close `4000`, "replaced"), and the replaced socket does **not** call `gone` — only the socket that is still current does, when it closes.
- **The seam for T-04b.** `trait SessionTokens { fn resolve(&self, room, token) -> Option<SessionId>; fn saved(&self, room, session) -> Option<Letter>; fn gone(&self, room, session); }` — room-scoped because T-04b's map is per room (`rooms::Entry`). `NoTokens` resolves nothing (the default until wiring, so buzzers are refused `4401` rather than served blind). T-04b's map implements it; the Orchestrator wires it when the second PR merges.
- **Where the transport lives.** `routes()` only receives `Arc<AppState>` and `AppState` is not mine, so the `Transport` travels as an axum `Extension`: callers layer `Extension(Transport::new(state, tokens))` over `router_with(state)`; the T-04c block inserts a default `Transport` (with `NoTokens`) only when none is present, so `router()` and the binary serve wall and host sockets unchanged.
- **TCP_NODELAY** on every accepted socket via `ListenerExt::tap_io` in `ws::serve(listener, router)`, which the tests use. `main.rs` is not cleared for this ticket (T-09 owns the bind), so T-09 switches `axum::serve` to `room::ws::serve` — flagged, not done.
- **No polling, no per-client timers** besides the one 10 s attach deadline, which is per connection setup, not per frame.

## Tests, by criterion

- **AC-81 (small, `test`)** `transport.rs`: wall + host + 3 buzzers attached on a loopback listener; drive every transition idle→released plus `←`/`→` steps through the HTTP routes; after each, every subscriber reads exactly one frame, its `revision` is the previous + 1, and all five carry the same `phase` (the expected one). A refused action (`409`) produces no frame; after the last transition no stray frame arrives. Each frame, less `t` and `revision`, equals the `GET …/{viewer}` projection — ties the socket to the canary-scanned payloads.
- **AC-37 (small, `test`)**: a buzzer whose test-map session saved `B` attaches in `live`; drop the socket, `gone` is recorded; close answers while it is away; re-attach with the same token: the **first** frame is `state` at the current revision, phase `closed`, `session.saved == "B"`. Replacement: a second socket with the same token closes the first with `4000`, no `gone` for it, and the second keeps receiving.
- **Refusals (`test`)**: host with a wrong bearer → `4401`, no state; buzzer with an unknown token → `4401`; unknown room → `404`.
- **AC-81 as written (`test-full`)** `transport_full.rs` (`#[ignore]`): wall + 200 buzzers; for every transition all 201 report the new phase as their next frame at revision + 1, within 5 s. Recipe `test-transport-full` raises `ulimit -n` (400+ fds in one process) and runs `cargo test --offline --locked --test transport_full -- --ignored`.
- **AC-41**: the reveal broadcast is this path; the p95 number stays `burst`'s (T-21) against the deployed room — not claimed here. README maps `burst`'s `{"t":"reveal","reveal_id"}` to `{"t":"state","phase":"reveal","revision"}`.

## Contract tensions (side taken)

1. The ticket header says *"Touches `room/src/phase.rs` (F-10): serialized after T-04b"*; the boot prompt says never edit `phase.rs`. The boot prompt is later and specific: `phase.rs` untouched.
2. Sending a session its own saved answer on attach vs. G-4/AC-58 (*"nothing per-person leaves the phone after close"*; EVALUATION: *"no **request** carries the participant's answer after close"*). The attach frame goes server → that phone only, and §4 requires the closed buzzer to show *"last saved answer"* after a reconnect, which needs it. Taken: send it, on the attach frame only, to its own session only. Flag for T-08's post-close scan.
3. `resolve(token)` in the boot prompt vs. a per-room map: the trait takes the room id too.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: sonnet subagent, 2026-09-26. 2 Critical, 3 Major, 4 Minor.

1. **(Critical) AC-37 is bound to `test-full`, the plan proves it in `test` silently.** Resolution: `test-full` depends on `test`, so the small reconnect test runs in both; in addition `transport_full.rs` re-runs reconnect at scale (20 of the 200 buzzers drop and re-attach mid-segment; each re-attach's first frame is current state with its saved answer). Named in the completion comment as the AC-37 hook.
2. **(Critical) The poke layer only wraps routes registered before it.** Resolution: the T-04c block is placed last, immediately before `with_state`, with a comment saying it must stay last; `README.md` says the same; the completion comment tells the Orchestrator to keep T-04b's block above it when wiring. Belt and braces: `Transport::changed` is public and documented for any writer that is not an HTTP route. The AC-81 test drives every route in `host_routes()`, so a host route that stops poking fails it.
3. **(Major) No teardown for per-room transport state.** Resolution: a room's hub entry is dropped when `changed` finds the room gone (T-11 deletes rooms; subscribers then get close `4404`), and when its last subscriber disconnects. Tested: entry count returns to zero after every socket closes.
4. **(Major) AC-37 test couples two hand-matched doubles.** Accepted as the limit of a parallel seam: the test proves the transport's contract (first frame is current state; `saved` comes from the map). The real-map proof is the wiring step when the second PR merges; flagged in notes.
5. **(Major) "Poke after attach" vs. a read-only trait.** Resolution: the trait doc says `resolve` is the attach itself (T-04b may mark the session connected and move `present`); hence the poke after it. `saved` stays a read.
6. **(Minor) 10 s attach deadline untested.** Resolution: `Transport::with_attach_timeout` for tests; a test asserts close `4408` with no state frame.
7. **(Minor) ulimit floor.** Resolution: the recipe raises the soft limit to 4096 and fails before running if it is still under 1024 (burst's own floor).
8. **(Minor) attach-frame exception to shared bytes.** Resolution: README states it — the buzzer attach frame is parsed, gains `session`, and is re-serialized for that one socket; broadcasts are never personalized.
9. **(Minor) host multiplicity.** Resolution: hosts are not keyed and never kick each other — several host devices may be attached at once (AC-50: a second device attaches to the same room). Said in the README.

## Reset 2026-09-26 by agent:delegator-pq6
