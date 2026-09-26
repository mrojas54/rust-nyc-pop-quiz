# room

The live half. One room per meetup, one question, seven phases from `idle` to
`released`, driven from the host's phone and shown on the wall. Rust, `axum` and
`tokio`, one small machine on Fly.io.

## Running it

From the repository root:

    just setup
    just test       # the room's tests, along with the pipeline's and the web's
    just canary     # just the in-process secrecy seam

Or directly:

    cd room && cargo test
    cd room && cargo run     # serves on 127.0.0.1:3000

## What is here

`router()` serves the room: create a room, one route per host action, and the
public state query for the wall, the buzzer and the host. The modules are the
ones named in `BUILDPLAN.md` section 2:

| Module | What it does | Ticket |
|---|---|---|
| `phase` | The machine: `idle → live → closed → split → work → reveal → released`, host actions only, no skipping, `trace_step` | T-04a |
| `answers` | **The sealed one.** Correct option, receipt, explanation, the resolving step | T-04a |
| `question` | The public half of a question | T-04a |
| `rooms` | The room record's shape and its phase-driven writes; the seams for sessions and the socket | T-04a (shape), T-04b (sessions) |
| `sessions` | Participant sessions and the answer store: join, capacity, the upsert, `leave` | T-04b |
| `view` | The public state query: wall, buzzer and host payloads | T-04a (data), T-05/06/07 (pages) |
| `copy` | SPEC §11's strings, mirrored from `web/shared/copy.js` | T-04a, T-22 (lint) |
| `auth` | The `HostAuth` seam: may this bearer create a room, may it host this one | T-04a (seam), T-09 (stand-in), T-10 (Discord) |
| `ws` | One broadcast per room: wall, buzzers, host | T-04c |

As built, the binary schedules no question and authorizes nobody (`DenyAll`),
so it can create no room. T-09 wires the HC-0 mock question and the §8.2
stand-in behind `HostAuth`; T-25 replaces seeding with
`PUT /admin/questions/{id}`.

### Routes

| Route | What | Credential |
|---|---|---|
| `POST /rooms` `{question_id}` | *Create a room* | `HostAuth::authorize_create` |
| `POST /rooms/{id}/put-on-screen`, `close-answers`, `show-split`, `walk-it`, `reveal`, `release`, `step-back`, `step-forward` | one per host action, plus `←`/`→` | the room's host session |
| `POST /rooms/{id}/run-it-again` `{question_id}` | a **new** room; this one stays released | `authorize_create`, same organizer |
| `GET /rooms/{id}/wall`, `/buzzer` | the public state query | none |
| `GET /rooms/{id}/host` | the host's projection | the room's host session |

A refusal is `409 {"reason": …}` in plain words; a missing or wrong credential
is `401` with no body; an unknown room is `404`.

## Pages

- `GET /host`, `GET /host/{room_id}` — the host phone (T-07, `web/host/`). `/host?question=<id>#<credential>` is *Create a room* (§8.2: the credential rides in the fragment and is sent as the bearer); `/host/{room_id}#<host session>` is one screen per phase, and is the resume link (AC-50).

## The phase machine

`phase::apply(state, command) -> Result<Applied, Refused>` is the only way a
phase changes, and it is pure: no clock, so no timer and no auto-advance. Each
phase has exactly one action that leaves it (`Phase::next_action`, which is also
the host screen's one primary action), so nothing can be skipped: seven legal
transitions (*Create a room* into `idle`, then one per phase), *Run it again*
from `released` (a new room; this one never moves again), and `←`/`→`, which
step the trace and never change the phase. `work` enters at step `0` and stops
at `M-2`; `reveal` enters at `M-1`, the step that prints, and may step the whole
trace (D-10). Every other pair is refused with a reason. `Room::act` is the one
place a room's machine is reassigned, from `apply`'s result.
`tests/phase_table.rs` checks the whole 8 × 10 table cell by cell.

## The seams for T-04b and T-04c

- **`rooms::Sessions`** — T-04b implements it over its session map.
  `close_snapshot()` is called by the close transition, once; the room freezes
  `answered` and `totals` from it and decides §4.5's middle beat then.
  `release()` is called at release.
- **`AppState::set_live_counts(room, LiveCounts { present, answered_live })`**
  — the counts' writer from outside the rooms lock. T-04b's own join, leave
  and upsert already hold that lock, so they write the same two fields through
  `Room::set_live_counts` before letting go (see *Sessions* below). `present`
  keeps moving after close; the frozen `answered` does not.
- **`Room::accepts_answers()`** — `true` only in `live`; T-04b's upsert asks it.
- **`Room::revision()`** — bumped on every change; T-04c broadcasts on it.
- **`view::wall` / `view::buzzer` / `view::host`** — the three payloads T-04c
  pushes. Each carries exactly one `phase`, read from the room's one phase value
  (AC-81).

## Sessions (T-04b)

`sessions::SessionMap` is the `rooms::Sessions` implementation, and
`AppState::new` uses it by default. A session is a `token` and an
`answer ∈ A..E | none` and nothing else (AC-57): an exhaustive pattern in
`sessions.rs` stops the crate compiling if a field is added. Sessions live in
memory beside their room and are dropped whole at release (AC-56).

| Route | What | Credential |
|---|---|---|
| `POST /join` `{code}` | a session: `201 {room_id, token, buzzer}` | none |
| `PUT /rooms/{id}/answer` `{letter}` | the answer upsert | the session token, as `Authorization: Bearer` |

**Joining (§4.1).** The code is trimmed and upper-cased, then must be six
symbols from the code alphabet. Any phase from `idle` to `reveal` admits a
join: the idle buzzer is *You're in.*, and `present` keeps counting after close.
A refusal is `{refusal, reason}`, where `refusal` is the name the buzzer
branches on and `reason` is its §11 sentence. It is `404` for `unknown` and
`409` for the rest:

| `refusal` | When |
|---|---|
| `malformed` | the code is not six alphabet symbols |
| `unknown` | no room has that code |
| `already_ended` | the room is `released` |
| `full` | the room holds `capacity` sessions (200; `AppState::with_capacity`) |
| `not_yet_open` | ships with its string; no phase of the seven reaches it |
| `closed_for_inactivity` | ships with its string; T-11 sets the condition |

Capacity is checked before a token is drawn, so a refused join creates,
reserves and moves nothing (AC-30).

**Answering (§4.3): the response contract T-06 renders from.** The phone shows
*saving…* while a `PUT` is in flight, then exactly one of these:

| Response | Means | The buzzer shows |
|---|---|---|
| `200 {saved}` | stored; last write wins; the same letter twice is a no-op | *saved — ‹saved›* |
| `409 {reason, phase, saved}` | not `live`. `saved` is the stored answer (or `null`), restated | from `phase` and `saved`: *answers are closed · you said ‹saved›* / *you didn't answer* |
| `401`, no body | the token names no session: never joined, left, or released | re-join |
| `400 {reason}` / any other 4xx | a malformed write; nothing was stored | *couldn't save. Your last answer, ‹X›, is safe.* |
| no response, 5xx | — | the same *couldn't save…*, where ‹X› is the last `200`'s letter |

The `409` is a superset of the generic `{reason}`: a client that reads only
`reason` still works, but render from `phase` and `saved`, never from `reason`.
Every non-`200` leaves the stored answer as it was (AC-36).
`tests/sessions.rs::ac35_response_contract` holds the server to the table. The
phone half (exactly one of the three on screen) is T-06's.

**Counts.** Join, upsert and leave recompute `present` (sessions that exist) and
`answered_live` (those holding an answer) from the map. They write both through
`Room::set_live_counts` inside the same lock, so the host never sees counts that
disagree with the sessions (AC-46).

**Ghost sessions, and what `leave` means.** A session has no "connected" flag;
the record is only its token and its answer. A session whose socket dropped
therefore stays a session: it counts in `present`, holds a capacity slot, and
its answer counts in `totals`. `AppState::leave(room, token)` removes it
whole. **T-04c calls `leave` only when a socket is gone for good** (its
re-attach grace has run out), never on a mere drop, or AC-37's re-attach with
the same token would lose the answer. Keeping the ghost's slot also means a
re-attach can never push a room past capacity. While `live`,
`answered_live ≤ present` always holds.

**The seams T-04c uses:** `AppState::buzzer_for(room, token)` returns the
buzzer payload with that session's own saved answer in `yours`; it is `Denied`
when the token resolves to no session. `AppState::leave` is described above.
`yours` appears only on these per-session paths. The public
`GET /rooms/{id}/buzzer` never carries it, and it never names another session's
answer.

**Close and release.** `close_snapshot` counts each session's final answer
once. `release` empties the map, so afterwards no token resolves and a join is
`already_ended`. `tests/sessions.rs::ac52_in_process_reconciliation` checks
200 sessions in-process. It supplements AC-52, which is `burst`'s against the
deployed room, and does not discharge it.

## The transport (T-04c)

`src/ws.rs`. One broadcast per room to the wall, the buzzers and the host, and
reconnect with the same session token.

### Sockets

| Route | Credential | Close codes |
|---|---|---|
| `GET /rooms/{id}/ws/wall` | none — the public projection `GET …/wall` already serves | `4404` room gone |
| `GET /rooms/{id}/ws/buzzer` | first message `{"t":"attach","token":"<session token>"}` → `SessionTokens::resolve` | `4401`, `4408`, `4000`, `4404` |
| `GET /rooms/{id}/ws/host` | first message `{"t":"attach","token":"<host session>"}` → `HostAuth::authorize_host` | `4401`, `4408`, `4404` |

An unknown room is `404` before the upgrade. `4401` is a missing, malformed or
wrong credential and says nothing else (AC-70); `4408` is no attach message
within 10 s; `4000` is *replaced* — a newer socket attached for the same
session. The credential is a message, not a query string, because the host's
resume link keeps its session in a `#fragment` to keep it out of logs.

### Frames

Every frame is the viewer's **whole** state:

    {"t":"state","revision":N, …view::wall / view::buzzer / view::host, flattened…}

The payload's own `phase` is the frame's only `phase` (AC-81). Less `t` and
`revision`, a frame is byte-for-byte the `GET /rooms/{id}/<viewer>` projection
the canary scans; `tests/transport.rs` asserts that at every revision.

- **On attach, the current state comes first** (§4.3, AC-37), then one frame per
  new revision. The client leaves *paused* on that first frame (T-06).
- **A buzzer's attach frame, and only that one,** adds
  `"session":{"saved":<letter|null>}` — its own saved answer, so a phone that
  reconnects in `closed` can show it. That frame is parsed and re-serialized for
  that one socket; broadcasts are never personalized.
- Hosts are not keyed: several host devices may be attached at once (AC-50).
- A socket that cannot take a frame within 10 s is dropped; that bound is per
  send, and nothing is ever sent because time passed.

### One broadcast per room

Each room has one `tokio::sync::watch` channel per viewer kind, holding
pre-serialized frames. A room's three payloads are built and serialized once
per revision, in `Transport::changed`, and every subscriber of that kind gets
the same bytes. `watch`, not `broadcast`: each frame is a whole state, so a slow
phone skips superseded ones instead of lagging behind them, and no receiver can
drop a reveal the way the spike's `broadcast` could (it had to count `Lagged`).
The channel exists only while someone is subscribed.

**What triggers a push.** `rooms.rs` has no change hook, and polling is out.
`Transport::changed(room_id)` publishes only if `Room::revision()` moved, so a
poke that changed nothing sends nothing. It is called:

- by the `notify` layer after every non-GET request under `/rooms/{id}/…` — all
  host actions, and any HTTP writer whose routes are registered **above the
  `// T-04c routes` block in `routes.rs`, which must stay last** (`layer` wraps
  only the routes already registered);
- by the transport after each call into the session map;
- by anything else that writes a room outside an HTTP route — T-04b if it takes
  answers over a socket, T-11's reaper. Such a writer calls `changed` itself.

### The seam for T-04b

    pub trait SessionTokens {
        fn resolve(&self, room_id: &str, token: &str) -> Option<SessionId>; // the attach
        fn saved(&self, room_id: &str, session: SessionId) -> Option<Letter>;
        fn gone(&self, room_id: &str, session: SessionId); // its current socket closed
    }

**T-04b's session map implements it**, and is wired in by layering
`Extension(Transport::new(state, map))` over `router_with(state)`. Until then
the router supplies a `Transport` over `NoTokens`: walls and hosts are served,
every buzzer is refused `4401`. `gone` is not called for a replaced socket.
It is called with no transport lock held, so a socket that dies at the very
instant its session re-attaches can report `gone` just after the new
`resolve`; T-04b's map should let the later `resolve` win when counting
`present`.
The tests use `TestTokens` (`tests/common`).

### `TCP_NODELAY`

`ws::serve(listener, router)` sets it on every accepted socket (the spike's
finding: without it a 40 ms delayed-ACK mode reads as the server being slow).
The tests serve through it. `main.rs` still calls `axum::serve`; T-09, which
owns the bind, switches it.

### For T-21's `burst`

The reveal broadcast is the path AC-41 measures; its p95 is `burst`'s, against
the deployed room, and is not claimed by these tests. The mapping from the
spike's frames:

| Spike | Room |
|---|---|
| `GET /ws`, one socket for everything | `GET /rooms/{id}/ws/buzzer`, then `{"t":"attach","token"}` |
| `{"t":"reveal","reveal_id":n,…}` | `{"t":"state","phase":"reveal","revision":n,…}` — key on `phase` and use `revision` as the id |
| `{"t":"hello",…}` | the attach frame (`t:"state"`); no instance field yet |
| padded 2048-byte reveal | the real reveal payload |
| `control` frames | the HTTP host routes with the host bearer |

### Tests

- `tests/transport.rs` (in `test`): wall + host + three buzzers through every
  transition — one frame per revision each, one phase across all (AC-81); drop
  and re-attach — current state first, saved answer intact (AC-37);
  replacement; refusals; the attach deadline; teardown.
- `tests/transport_full.rs` (`#[ignore]`d; `just test-transport-full`, inside
  `test-full`): the wall and 200 buzzers agree at every transition, and twenty
  drop in `closed` and resume in `split` (AC-81 and AC-37 as written).

## The sealed module, and how the proof works

`answers::load` reads a bank record and splits it on the spot. The public half
(`question::PublicQuestion`) holds option texts in arrival order, the source,
the hint, and trace steps `0..M-2` with any `stdout` row removed. Everything
that joins an option to the verified output goes into a vault. That covers the
correct option (derived the way `bank.correct_index` derives it), the receipt
(the twin of `receipt.receipt_lines`, tested against the same
`bank/fixtures/receipts/*.json`), the beats, every `why_tempting`, and the full
trace.

The vault is a private module nested inside `answers`, and its fields are
private to it. Rust field privacy is per module, so not even the rest of
`answers.rs` can read them. Its one read, `open`, demands a
`phase::RevealWitness`. `Machine::revealed()` mints that witness, only in
`reveal`, and nothing else can build one: its field is private, and `unsafe` is
forbidden crate-wide. The witness borrows the machine, and `Room::open()`
borrows the room, so nothing opened in `reveal` survives a phase change. The
three projections branch on `room.open()`, not on the phase. The pre-reveal
builders receive a `PublicView`, which has no path to the vault.

The proof is in two halves, and neither is "we reviewed it":

1. **`compile_fail` doctests**, in `src/phase.rs`, `src/answers.rs` and
   `src/rooms.rs`, cover six attempts to cross the boundary:
   - forge a witness;
   - forge a machine in `reveal`;
   - read a room's question;
   - read the vault;
   - open the vault from a `PublicView`;
   - hold what was opened across `act`.

   Each is paired with an ordinary doctest, its twin, which compiles the same
   paths and differs only in the forbidden line. So a rename or typo breaks the
   twin loudly rather than letting the `compile_fail` pass for the wrong reason.
   They carry no error codes: on stable, `compile_fail,E0xxx` silently stops
   running.
2. **`tests/boundary.rs`** is the in-crate half that a doctest cannot see. It
   reads the source and asserts:
   - the witness is built in one place;
   - the vault holds only `seal`, `judge` and `open`, and every read takes the
     witness;
   - nothing sealed derives or implements `Debug`, `Clone`, `Copy` or
     `Serialize`;
   - nothing outside `answers.rs` touches the vault;
   - `Room::act` alone reassigns the machine;
   - the pre-reveal builders take only a `PublicView`.

Both halves were mutation-checked when they were written: opening each boundary
in a scratch copy turned exactly the matching test red.

`tests/canary.rs` is the runtime complement. It drives `router_with()` through
every phase. Before `reveal`, it asserts that no payload for any viewer carries:
- a ✓, a receipt line, the explanation or any `why_tempting`;
- the resolving step, a `stdout` values entry, or a step beyond `M-2`;
- the hint (before `live`, and outside the buzzer's `live` payload).

It also asserts that every option object is exactly `{letter, text}` in arrival
order, and that the buzzer never carries source, trace or option text. At
`reveal` it asserts that the plants do appear. The plants are T-04a's seam:
**T-08** replaces them with its canary set and extends the scan to pages and
socket frames, and **T-25** adds `POPQUIZ_ADMIN_TOKEN`. G-6's machine half is
proven here; its full canary half is T-08's.

### Choices worth knowing

- **Option order.** The room never arranges options: the pushed record's order
  is the wall's order. The date-drawn arrangement (AC-23) belongs to the
  pipeline's push (T-20). `bank/questions/q3.json` is in bank order, which is
  fine for tests but is not a record to put in front of a room.
- **The correct-option rule mirrors `bank.correct_index` exactly.** It matches
  by kind for a does-not-compile record, and otherwise by the option equal to
  `stdout` less one trailing newline. If the rule changes, both twins change
  together.
- **Copy.** `src/copy.rs` has one constant per `copy.js` key, with the same
  name upper-cased. `tests/twins.rs` compares the two in both directions, so
  T-22's lint has matching keys to compare. Edit SPEC §11 first, then
  `copy.js`, then `copy.rs`.

## The toolchain

This crate builds with whatever `cargo` the machine has, and is not pinned.

That is not an oversight, and it is worth being precise about because the two
things are easy to confuse: **the verification pin is a different pin.** The
full `rustc -Vv` release and commit-hash, plus the nightly's date for Miri, are
defined once by T-15a where the sandbox image is built, and they exist so that a
question's verified answer can be tied to an exact compiler. Nothing about
building this server needs that, and putting a `rust-toolchain.toml` here would
both invite the confusion and make `just test` reach for the network on any
machine that did not happen to have the pinned toolchain already.

## The canary seam

`tests/canary.rs` drives the router in-process, with no socket.
T-04a extended it to drive every phase in order for all three viewers (above).
T-08 lands its canary set on top: canaries planted in the resolving trace step's
`note`, the explanation, the receipt and the hint, asserted absent from every
pre-reveal payload, pages and frames included. T-25 adds the admin token as a
fifth plant.

The other half of that hook — scanning the deployed room's real frames and pages
— runs in `just test-full` once T-09 has something deployed.

## The burst spike (T-03)

The one real engineering risk in this project is the deadline write burst:
200 people answering inside the two seconds before the host closes answers.
`BUILDPLAN.md` §5 retires it at T-03, **before any room code is written**, and §3
is explicit about why — if p95 misses, D-A option 2 (Cloudflare Durable Objects)
re-opens and the substrate decision is taken again. So this spike exists to be
allowed to fail, and everything below is built so that a failure is impossible
to miss and a pass is impossible to overstate.

Two throwaway binaries, both behind the default-off `spike` feature:

| Bin | What it is |
|---|---|
| `spike-server` | One room, in memory, `idle → live → closed` plus a reveal broadcast. Not the phase machine — T-04a writes that on a blank page. |
| `burst` | The load client: 200 WebSockets, timing, a JSON report, a non-zero exit on a miss. |

### Running it

```sh
# Both bins are feature-gated, so the inner loop never builds them (see below).
cd room
cargo test  --features spike          # the hermetic tests: estimator, verdict, reconciliation
cargo build --release --features spike --bin spike-server --bin burst

# Against a loopback server — proves the harness, never a headline number.
PORT=8099 ./target/release/spike-server &
./target/release/burst --url http://127.0.0.1:8099 --clients 12 --cycles 2 \
  --churn-secs 3 --reveals 2 --window-ms 500

# Against the deployed machine — this is where the real numbers come from.
./target/release/burst --url https://rustnyc-popquiz-spike.fly.dev \
  --clients 200 --seed 20260920 --out spike/reports/$(date -u +%F).json
```

`--help` lists every flag. Exit codes carry meaning and are worth knowing:

| Exit | Means |
|---|---|
| `0` | all four criteria pass as measured |
| `1` | **a criterion missed** — the server did not meet the threshold |
| `2` | **the run is invalid; do not quote its numbers** — fewer than 200 clients connected, an ack timed out, a reveal receipt never arrived, more than one Fly machine answered, `ulimit -n` under 1024, or the laptop's own send lag exceeded 25 ms |

Separating 1 from 2 is the point. Without it, a broken harness reads as a failing
server and a failing server can be excused as a broken harness. The report says
the same thing in its own fields: an invalid run carries `"quotable": false` and
each criterion's `pass` is `null`, not judged. A pass whose confidence interval
crosses its threshold still exits 0, but the verdict notes call it `MARGINAL` in
words and drop the clean-pass sentence.

`cargo test --features spike` includes four loopback tests that serve the real
spike server in-process on `127.0.0.1:0` and drive whole segments through it:
last-write-wins, frozen totals, post-close refusal, every reveal received, exact
reconciliation, the join past capacity refused `full`, a deliberately invalid run
left unjudged, and the report surviving a round trip. They prove the harness. They
never produce a headline number.

### Where the numbers come from

**The headline figures for AC-41, AC-53 and AC-54, and AC-52 at 200 sessions,
come from a run against the deployed Fly machine, from a laptop.** No loopback
number is ever quoted as any of them — `EVALUATION.md`'s AC-53 row says it
outright: *run against the deployed substrate, not a local mock.* Committed runs
live in `spike/reports/`.

The client-side network in those runs is the laptop's. **Venue wifi is AC-55's
oracle and it is settled at HC-4 in October, not here**; the report records a
failure rate but labels it as the laptop's link.

### How it tries not to lie

A load test that flatters its subject is worse than none, so the specific ways
this one could have been wrong are each defended against, and the defenses are
visible in the report rather than asserted here:

- **The write clock** starts on an already-open, already-handshaken connection
  and stops when that socket reads the matching `ack` — one clock, one task. TLS
  and connection setup land in `diagnostics.connect_ms` and can never leak into a
  write latency.
- **The ack means the write is in the authoritative map**, not that a frame
  arrived: the server drops the room lock and then acks.
- **`send_lag_ms`** — the laptop's own scheduling delay — is measured, reported,
  and never added to write latency. 200 real phones do not queue behind each
  other. Above 25 ms at p95 the run is invalid, because at that point the laptop
  is what was measured.
- **Both burst shapes run, and the worst drives AC-54.** `uniform` spreads the
  200 writes over the window; `spike` puts all of them in its last 50 ms. Both
  satisfy `SPEC.md` §9's *200 writes inside 2 s*, but AC-54 calls the burst *the
  highest-risk moment in the system*, and letting only the gentler shape reach
  the exit code would mean the harder one could never affect the go/no-go.
- **`p95` travels with its own uncertainty.** At n=200 it is the 190th sample —
  one slow client moves it. The report prints the rank, its 95% interval (ranks
  183–197, rounded outward), and the max; a pass whose upper bound crosses the
  threshold is flagged `marginal` rather than rounded into a clean pass.
- **Every raw sample ships** in `samples_ms`, so any percentile can be recomputed
  by someone who does not trust this code.
- **AC-52 is checked four ways** — per-letter totals, `answered`, the applied
  `seq` sum, and an XOR fingerprint over `(session, letter, seq)`. The first
  three are aggregates and two sessions swapping answers would slip past all of
  them; the fingerprint is what makes it a per-session claim. The server still
  exposes no per-session answer.
- **`TCP_NODELAY` on both ends.** axum does not set it (`tap_io` is the seam);
  without it a 40 ms delayed-ACK mode appears in the histogram and reads as the
  server being slow.
- **The reveal frame is padded to 2048 bytes**, because a fan-out measured on a
  60-byte frame is a measurement that lies by being easy.
- **`auto_stop_machines` is off** in `fly.spike.toml`, and every `hello` frame
  carries `FLY_MACHINE_ID`. If more than one machine ever answered, room state
  would be split across two maps and AC-52 would be reconciling against a
  fiction — so the run is invalidated rather than reported.
- **The report is written before the exit code is computed**, so a failing run
  still leaves a complete artifact behind.

### Deploying the spike

`fly.spike.toml` and `spike/Dockerfile` describe a **throwaway** app,
`rustnyc-popquiz-spike` in `ewr` on the smallest shared-cpu machine. This is not
the room's deploy — T-09 owns that, along with `room/fly.toml`,
`popquiz.rustnyc.org` and `smoke`.

The single most important line in `fly.spike.toml` is
`http_service.concurrency.hard_limit`. Fly's default is **25 connections**; at
200 WebSockets the proxy would shed connections before the server was under any
strain, and the spike would report a p95 miss belonging to the config rather
than to Rust on Fly — a false no-go on the substrate decision.

```sh
cd room
fly launch --no-deploy --copy-config --config fly.spike.toml \
  --name rustnyc-popquiz-spike --region ewr --dockerfile spike/Dockerfile
fly deploy --remote-only --config fly.spike.toml --app rustnyc-popquiz-spike \
  --dockerfile spike/Dockerfile --ignorefile spike/.dockerignore .
fly machine stop <ID> -a rustnyc-popquiz-spike   # leave it scaled to zero
```

`fly launch` may drop a `fly.toml`, `Dockerfile` or `.dockerignore` into `room/`.
None of those belong to this ticket; check `git status` afterwards and delete
them.

### What is deliberately not here

`just burst` is still the `_pending` stub. **T-21** wires this client into the
recipe and into `test-full`; `--url`, JSON on stdout and a non-zero exit on a
miss are already in place so that is a small change.

The cost of that, stated plainly rather than left to be discovered: because both
bins sit behind `required-features = ["spike"]` — which is exactly what keeps
`just test` inside its 60 s budget — **neither `test` nor `test-full`
type-checks them until T-21 lands.** Until then the gate is by hand, and it is
the two `--features spike` commands at the top of this section.
