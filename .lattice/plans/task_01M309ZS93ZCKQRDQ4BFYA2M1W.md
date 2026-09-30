# PQ-26: Burst and smoke in CI

BUILDPLAN.md T-21 (M4).

`burst` and `smoke` in `test-full`, run against the deployed room in CI; failed-request logging the client can read after a meetup

Criteria: AC-41, AC-52–55
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-09
BUILDPLAN notes: Serialized on `justfile` (`burst`, `test-full`)

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-30)

## The side taken on (a): grow a shared room driver, keep two bins

`smoke.rs` already speaks the room's real protocol end to end, and the spike's `burst.rs`
already owns the measurement: nearest-rank p95 with its CI rank, `marginal`, raw samples,
the Exit split (0 pass / 1 missed / 2 invalid), JSON on stdout. Neither alone is the harness.

- **New `room/src/bin/room_client.rs`** (a `#[path]` module, not a bin; `autobins = false`
  already allows this): smoke's `Target`, keep-alive `Http` over TCP/TLS, `read_chunked`,
  `open_ws`, the per-socket `Watcher` (first arrival per phase, last frame per phase, an
  optional per-frame hook), `all_reach`, `Rng`. Moved out of smoke.rs verbatim; smoke keeps
  its secrecy scan by passing its hook.
- **`smoke.rs`** uses the shared module; behaviour unchanged. Gains `--out <path>` (a JSON
  report: pass/fail, the report lines, failures, substrate) so CI can upload it. `run()`
  becomes callable from a test (pub, takes the organizer credential as a parameter — main
  still reads it from `POPQUIZ_ORGANIZER_SESSION` only).
- **The spike's client is kept, renamed:** `git mv burst.rs spike-burst.rs`, bin
  `spike-burst`, still `required-features = ["spike"]`, so the T-03 reports in
  `room/spike/reports/` stay reproducible against `spike-server`. Its header gains one line
  pointing at the new `burst`.
- **New `burst.rs`** (bin `burst`, `required-features = ["burst"]`, feature
  `burst = ["smoke"]` — the same TLS client, no new crate): the spike's `Stats`,
  `ci_ranks`, `marginal`, `verdict`, `Exit`, `r2` carried over (copied; the spike copy
  stays frozen with the spike), driven over the room's protocol by `room_client`.

### What `burst` does against `<url>` (room protocol, one room)

1. `pages_answer`-style wait for the machine, then `POST /rooms {question_id}` with the
   organizer session (default question `q3`, `--question <id>` to change it); 401/403/409
   map to "could not run" (exit 2) with smoke's wording.
2. Wall + host sockets; N participants (`--participants`, default 200, 1–200) `POST /join`
   and attach a buzzer socket; each keeps one kept-alive HTTP connection for writes.
   Connect/join times go to diagnostics, never to write latency. Every socket awaits `idle`;
   the host's `present` must reach N.
3. `put-on-screen` → `live` on every socket.
4. **AC-54, the deadline burst in isolation:** `--cycles` (default 2) per shape
   (`uniform`, `spike` = last 50 ms; `--burst-shape`), each cycle every participant writes
   one letter at an offset inside `--window-ms` (2000), nothing else in flight, a quiet
   gap between cycles. Every write timed on its already-open connection; `send_lag` =
   actual send instant − scheduled instant, reported, never added. Each response must be
   `200 {saved: <letter>}`. Worst shape's p95 drives AC-54 (as the spike).
5. **AC-53, the full segment:** churn (`--churn-secs`, default 20): a third change their
   minds at random instants; then the final deadline burst (every participant's final
   answer, uniform inside the window). AC-53's population = every accepted write in
   `live` (isolated cycles + churn + deadline burst).
6. `close-answers` → `closed` everywhere. **AC-52 per session:** every participant sends one
   more write; each must be `409` with `saved` restating that session's own final letter
   (the per-session check the spike's fingerprint made — two sessions swapping answers
   fails it). Those refusals are never latency samples.
7. `show-split`: every buzzer's split frame `counts.totals` = the expected per-letter
   totals of the finals, `answered` = N, sum = N.
8. `walk-it`, step forward to M-2, then **AC-41:** `reveal`, timed from the host's POST
   issue to each buzzer's first `reveal` frame on one process clock; n = N samples; a
   missing receipt is an invalid run. Then `release`, awaited on every socket (a full
   segment ends at release).
9. Report JSON (stdout, and `--out`): `schema`, `substrate` (`loopback` when the host is
   127.0.0.1/::1/localhost, else `deployed`), `quotable`, per criterion
   `{pass|null, threshold, stats{…samples_ms}}`, `ac52` reconciliation, diagnostics
   (connect/join ms, send lag, ulimit, errors), notes, and a `substrate_note`: a loopback
   run never claims AC-53's deployed-substrate clause. Exit per the spike's verdict;
   invalid when clients < N, ulimit < 1024, harness errors, a missing reveal receipt,
   send-lag p95 > 25 ms. The spike's "distinct Fly machine ids" check is dropped: the room
   exposes no instance id (README, *For T-21's burst*) and fly.toml runs one machine; a
   second machine would surface as 404s on join/attach → harness errors → invalid.

`just burst URL *ARGS` → `cargo run --release --offline --locked --features burst --bin
burst -- --url URL ARGS` with `ulimit -n 4096`, reading `POPQUIZ_ORGANIZER_SESSION` like
smoke. `PENDING := ""` (mechanism kept, comment says it holds nothing now; `_pending` and
`test-full`'s loop handle an empty list). The reserved-name header gains `just burst` and
`just smoke`. The smoke recipe's stale `HOST_DEV_TOKEN` comment is corrected to
`POPQUIZ_ORGANIZER_SESSION`.

## (b) In `test-full`: a room the job starts in-process

- **New `room/tests/harness_full.rs`**, declared `[[test]] required-features = ["burst"]`
  so `just test` (no features) never compiles it — the 60 s budget holds by construction.
  It `#[path]`-includes `src/bin/smoke.rs` and `src/bin/burst.rs` (as burst.rs already
  includes spike-server for its tests), uses `tests/common` (`TestAuth`, `ORGANIZER`, `q3()`):
  binds `127.0.0.1:0`, builds `AppState::new(TestAuth, vec![q3()], Urls{base: http://addr})`,
  serves `room::router_with(state)` through `room::ws::serve` — the real router, real
  sessions, real sockets. One fresh room server per run (q3 runs once per machine).
  Two `#[ignore]`d tests:
  - `burst_on_loopback`: 200 participants, short churn; asserts the report's shape,
    `substrate == "loopback"`, every client connected, zero harness errors, every reveal
    receipt, AC-52 exact (per-session restatement + totals), every burst write scheduled
    inside its window; latency numbers **printed, labelled loopback, not asserted**
    (a shared CI runner's p95 is not the room's).
  - `smoke_on_loopback`: smoke's whole run (all checks must pass) with 50 participants.
- `just test-full` gains a dependency `harness-full` (recipe: `ulimit -n 4096`, then
  `cargo test --offline --locked --features burst,spike --bins --test harness_full --
  --include-ignored --nocapture`), which also type-checks and unit-tests the smoke, burst,
  spike-server and spike-burst bins (closing PQ-3's noted gap). The "Ran:" list adds
  "burst and smoke against an in-process room on loopback (numbers labelled loopback)".
  The trailing "T-21 adds the deployed runs…" line is replaced by where the deployed run
  lives (the dispatch workflow / the laptop form).
- CI `test` job's *The smoke client* step: **kept**, extended to the burst bin's unit tests
  (`--features burst --bin smoke --bin burst --test smoke_config`) — cheap, catches a
  broken harness on the fast job. `test-full` job needs no new step (`just test-full` runs it).

## (c) `.github/workflows/deployed-burst.yml` — `workflow_dispatch`, the client's to start

Inputs: `url` (default `https://rustnyc-popquiz.fly.dev`), `participants` (default 200).
Secrets: `POPQUIZ_ORGANIZER_SESSION`, `POPQUIZ_ADMIN_TOKEN` (set right before, deleted
after; never echoed — passed via `env:` only, `set +x`). Steps: checkout, just, cargo cache,
`just setup`; schedule: `PUT /admin/questions/q3` with `bank/questions/q3.json` (for smoke)
and `PUT /admin/questions/burst-q3` with q3's record under the id `burst-q3` (jq), because
a question runs once per machine and both runs need one; a harness-only id can never be
retired from the bank by a later `popquiz sync` (the house rule about false retirement).
Then `just burst <url> --question burst-q3 --participants N --out reports/burst.json` and
`just smoke <url> --participants N --out reports/smoke.json` (both run even if the first
fails; job fails if either did), upload `reports/` as an artifact, and a final step summary
telling the client to `fly apps restart rustnyc-popquiz` and delete the two secrets. No Fly
token in CI. `permissions: contents: read`.

## (d) AC-55: failed-request logging

New `room/src/requestlog.rs` (lib.rs gains `pub mod requestlog;`): per-room counters and the
two line shapes, one JSON object per line on stderr (Fly collects stderr; the same sink
discord.rs uses). **Deviation:** the brief says a `tracing` line; `tracing` is in the lock
but no subscriber is (`tracing-subscriber` is not), so a `tracing` event would print
nothing without a new crate. The room already logs as plain lines on stderr (discord.rs
`Stderr`), so these are JSON lines with stable field names; `fly logs` shows them as is.

What counts as a **participant request** (the denominator): `POST /join`,
`PUT /rooms/{id}/answer`, `GET /rooms/{id}/buzzer`, and each buzzer socket upgrade.
What counts as **failed** (the numerator), each one line
`{"event":"participant_request_failed","room":<id>,"kind":…,"route":…,"status"|"phase":…}`:
- any `5xx` on a participant route (`kind: "server_error"`) — the room failed the phone;
- a buzzer socket that ended mid-segment **without** the phone closing it: a read error or
  EOF with no close frame, or the room dropping a socket it could not send to within the
  send timeout (`kind: "socket_dropped"`, `phase` = the room's phase then).
Not counted, and why: `409` refusals (closed room, already ended, full), `404` unknown
room/code, `401` bad token, `400` bad body — each is the room answering correctly; a close
frame from the phone (it navigated away or reloaded); `REPLACED` (the same phone
re-attached); `ROOM_GONE` after release/expiry. `/join` isn't under `/rooms/{id}` — its
failures log with `room: null` and are counted against the room the code resolves to when
it resolves (else only the line).
At release: one `{"event":"participant_requests","room":<id>,"failed":F,"total":T}` and the
room's counters are dropped. Counters are a process-wide map keyed by room id (ids are
unique random, so parallel tests don't collide); a room that expires without release emits
no summary (stated in the README). Hooks: one middleware `.layer` in routes.rs over the
participant routes, and in ws.rs `run()` the drop classification (a flag on how the loop
ended) + one call. No token, session, code, answer or letter is ever a field.
Test seam: `requestlog::capture()` returns the lines emitted since for a given room (a
process-wide tap next to stderr), used by the tests below.
README gains **After a meetup**: `fly logs -a rustnyc-popquiz --no-tail | grep
participant_requests` (and the failed lines), rate = failed/total, AC-55's bar < 0.1 %.

## Tests by criterion

- AC-54/53/41 decision on recorded samples (in `test`? No — the bins are feature-gated;
  these run via `--features burst --bin burst` in the CI *smoke client* step and in
  test-full): `verdict` pass / miss per criterion / invalid precedence / marginal only on an
  all-pass; `Stats` nearest-rank, CI outward rounding, marginal edge (== threshold);
  report shape (every key, `samples_ms` round-trip, `quotable:false` ⇒ `pass:null`),
  `substrate` from the URL; args parsing. Carried from the spike tests where they apply.
  **Deviation:** the brief says "tests in `test` for the report's shape and the
  pass/miss/marginal decision". `just test` compiles no feature bins (the 60 s rule by
  construction), so these live in the bins' unit tests, run by CI's `test` job step and
  by `test-full`; putting burst's logic in the lib would ship harness code in the image.
- AC-52/54/41 mechanics end to end: `harness_full.rs` (test-full), above.
- AC-55: `room/tests/requestlog.rs` (in `test`): a 5xx is not reachable on demand, so the
  classification is unit-tested in `requestlog.rs`; over loopback sockets: a buzzer whose
  TCP drops mid-`live` is one failed line; a phone's close frame, a replacement, and a
  `409` after close are none; release emits the summary with the right failed/total;
  every captured line scanned for the session token, the host session, the code and the
  letters written. The canary's full walk (`canary_scan`) also scans every captured
  request-log line for every plant.
- AC-64 scan stays green (no stand-in names in the new workflow or src); canary fast+full.

## Files

Create: `room/src/bin/room_client.rs`, new `room/src/bin/burst.rs`,
`room/src/requestlog.rs`, `room/tests/harness_full.rs`, `room/tests/requestlog.rs`,
`.github/workflows/deployed-burst.yml`. Rename: `burst.rs` → `spike-burst.rs`. Change:
`room/src/bin/smoke.rs`, `room/Cargo.toml` (feature `burst`, bins, `[[test]]`),
`room/src/lib.rs`, `room/src/routes.rs`, `room/src/ws.rs`, `room/tests/canary_scan/mod.rs`,
`justfile`, `.github/workflows/ci.yml`, `room/README.md` (Deploying/Smoke, a new *Burst*
section, *After a meetup*, the spike section's "not here" paragraph).
No new crate; `Cargo.lock` unchanged. Not touched: `copy.rs`, `twins.rs`, `web/**`,
`pipeline/**`, `Dockerfile`, `fly.toml`, `.env.example` (no new variable name in the room;
the workflow's secrets reuse the two existing names).

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Baseline before any change: `cargo test --offline --locked` in room/ green, 14.0 s warm
(outside the Bash sandbox, which refuses the Discord mock's socket bind — the sandbox, not the code).

1. **Critical — smoke on the real q3 would falsely retire it.** Concern accepted. Resolution: smoke
   gains `--question <id>` (default `q3`); the id must name a record with q3's content, because
   smoke's secrecy scan reads q3's secrets from the embedded record. The dispatch workflow schedules
   q3's record relabelled `smoke-q3` and `burst-q3` (jq rewrites `.id` only; the room validates no
   id pattern and the schema has none) and runs each harness on its own id, so no bank id is ever
   released on the deployed machine by CI. The README and the job summary say: `fly apps restart`
   before anyone hosts **and before any `popquiz sync`**. `popquiz sync` does not exist yet (T-20);
   whether it tolerates a harness id in `GET /admin/used` is T-20's to decide — flagged to the
   Orchestrator in the DONE comment, not claimed here.
2. **Critical — the trial org stops the machine (~5 min).** Accepted. The workflow builds both bins
   first, then schedules, bursts and smokes back to back (a tight sequence, no build in between). The
   workflow's defaults keep burst short (`--cycles 1`, `--churn-secs 10`), well inside the window;
   README and the job summary state the window. A 404 on the room's routes or a dropped connection
   mid-run is reported as "the room is gone — the machine may have been stopped or restarted (the
   trial's stop)", never as "a second machine".
3. **Major — report/decision tests must be in `test`.** Accepted. `Stats`, `ci_ranks`, `marginal`,
   `verdict`, `Exit`, `r2` and the report builder live in `room/src/bin/burst_report.rs`
   (serde_json only), `#[path]`-included by `burst.rs` and by a new **ungated**
   `room/tests/burst_report.rs`, which `just test` runs: pass/miss/marginal/invalid decisions on
   recorded samples, report shape, `quotable:false ⇒ pass:null`, substrate labelling. No sockets.
   Nothing enters the lib or the image.
4. **Major — capture tap must not record in production.** Accepted. `requestlog` records nothing
   unless a test calls `requestlog::start_capture()`; default is stderr only.
5. **Major — route template and allowed fields.** Accepted. Lines carry a fixed route name
   (`join`, `answer`, `buzzer_state`, `buzzer_socket`), never the URI; fields are exactly `event`,
   `room`, `route`, `kind`, `status` / `phase`, and in the summary `failed`, `total`, `ended`. The
   room id is the one identifier (the brief asks for it). The tests assert the exact key set and
   scan every line for the join token, host session, room code and the letters written; the canary
   walk scans the captured lines for every plant.
6. **Major — AC-55 is server-visible only.** Accepted: *After a meetup* says the rate is a floor —
   requests that never reached the room (venue wifi dropping them, timeouts before the edge) are
   invisible to it, and a burst of 401s or a machine restart mid-meetup is investigated from
   `fly logs` separately.
7. **Major — summary only at release.** Accepted in part. Also emitted when the transport sees the
   room gone (`ws.rs` `changed()`: the sweep/expiry path already goes through it) with
   `ended: "gone"`; release emits `ended: "released"`; whichever comes first removes the counters.
   A machine stopped by the trial emits nothing — stated in the README. No change to `rooms.rs`/
   `lifecycle.rs`.
8. **Major — rename/new files outside the named list.** They are inside the cleared scope
   (`room/**` except copy.rs/twins.rs; `room/tests/**` except twins.rs), but logged as deviations:
   the spike's client moves to `spike-burst.rs` (bin `spike-burst`), and every reference is updated
   (README spike section and reproduction commands, Cargo comments, `spike-server.rs` header,
   `room/spike/reports/README.md` if it names the bin).
9. **Minor — `--include-ignored` scope.** Accepted: `harness-full` runs the bins' unit tests
   (`--features burst,spike --bins`) and `--test harness_full -- --ignored` as two cargo calls.
10. **Minor — fd count.** ~800 fds in-process at 200; the recipe raises to 4096 and the report's
    ulimit rule (< 1024 invalid) stays; loopback test asserts ulimit ≥ 1024 was met.
11. **Minor — upgrade status.** Stated in README: a buzzer socket's failures are counted by the drop
    classification only (the upgrade itself is 101 or 404).
12. **Minor — `tracing` deviation.** Kept and flagged in DONE as F-39-adjacent.
13. **Minor — warm time.** Measured before (14.0 s) and after; reported in DONE and the PR.
14. **Minor — the admin token on a command line.** The workflow feeds the header to curl through
    `--config -` from a `printf` builtin, so the token is never in argv; secrets only via `env:`.
15. **Minor — AC-53 disclosure.** The `test-full` "Ran:" line, the report's `substrate_note` and the
    README all say AC-53's deployed-substrate clause stays open until the client's dispatch (or
    laptop) run.

## Reset 2026-09-30 by agent:delegator-pq26
