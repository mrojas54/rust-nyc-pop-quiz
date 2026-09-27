# PQ-10: Canary secrecy suite

BUILDPLAN.md T-08 (M1).

`canary`: plant secrets in answer, explanation, receipt, hint-before-live; scan every payload, frame, and page in every phase; post-close traffic carries no answer

Criteria: AC-32, AC-47, AC-48, AC-58, AC-60, AC-79, G-3, G-4, G-8
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a–07
BUILDPLAN notes: In-process scan in `test`; the deployed-room scan in `test-full`. Serialized on `justfile`

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-26)

Base: origin/main @ 445a40f (M1 room + #24 static fallback). Worktree
/Users/michellerojas/rust-nyc-pop-quiz-worktrees/canary, branch ai-c11-cc/canary.

## Files

| File | Change |
|---|---|
| `room/tests/common/mod.rs` | **Additive, under `// T-08`.** `Plants` (every canary value, each `CANARY-<NAME>-<16 hex>` with a per-process random suffix from `getrandom`, so no code can have been written to match one) and `canary_question()` / `canary_question_dnc()`. `POPQUIZ_ADMIN_TOKEN` plant: random per process, set into the process env once (`OnceLock`) and passed to every child (`node`); **never a literal in the repo**, so T-25's repo-wide token scan stays clean. Existing `planted()` / `PLANT_*` untouched (other suites use them). |
| `room/tests/canary_scan/mod.rs` (new, not a test target) | The shared scan core used by both suites: the harness (state with the canary questions + the **real** session map, served through `room::ws::serve` on `127.0.0.1:0`), the route walk, the per-surface rules, the frame drain. Included by `#[path]` from both test files. |
| `room/tests/canary_scan/page.js` (new) | The node driver: (a) `render` mode — loads the wall's scripts **as served** (fetched from the running server by following the page's own `<script src>`), renders every captured wall frame with `PQ.Wall.html`; (b) `buzzer` mode — loads the buzzer page's served scripts into a `node:vm` context and **mounts the real `buzzer.js`** with real `fetch`/`WebSocket` (node 22 globals) pointed at the server, wrapped to log every request, body and socket message; stdin commands (`wait <phase>`, `tap <L>`, `hint`, `drop`, `snap`), one JSON line out per command. |
| `room/tests/canary.rs` | Rewritten. The T-04a seam tests that are not scans (host screen per phase, host routes/401s, creation, run-it-again) are **kept verbatim**; the old `drive_and_scan` is replaced by the T-08 scan. Runs in `just test`. |
| `room/tests/canary_full.rs` (new) | `#[ignore]`, run by `test-full`: the same scan with **every HTTP request over a real TCP socket** (a minimal HTTP/1.1 client, `Connection: close`; no new crate), two rooms back to back (run-it-again into the dnc variant), plus the reconnect path. `CANARY_URL` set → fails with "T-09's `smoke` owns the deployed scan" (the `--url` hook). |
| `justfile` | `canary *ARGS`: runs `--test canary`; `--full` also runs `canary_full --ignored`; `--url <u>` exports `CANARY_URL` and runs `canary_full`, which refuses honestly until T-09. Replaces the "in-process scan only" echo. `test-full`'s dependency line gains `(canary "--full")` and its "Ran:" echo line is corrected. Reserved-name header unchanged (canary is not a reserved/pending name). |
| `room/README.md` | Replace *The canary seam* section with *Canary*: plants, surfaces, rules, unscanned list and why. |

## The plants (`canary_question()`, id `canary`, from q3 on disk)

Distinct per plant, per run: correct option text (E) = `P_CORRECT`, and the
synthetic verified record's `stdout` = `P_CORRECT\n` (rustc `"SYNTHETIC - no
compiler ran"`, like `planted_dnc`; the source is itself a plant, so the record
describes no program — nothing written down is a program's output);
`runs.count` = a random 5-digit N → receipt line `✓ Ran N times` is the
receipt plant `P_RECEIPT`; `exit_code: 0`; `explains.what`/`takeaway`; every
`why_tempting` (A–D); `hint`; `source` (a trailing `// P_SOURCE` comment on
line 1, so trace line numbers do not move); every trace step `note` (steps
0..M-2 each `P_NOTE_i`, the final step `P_RESOLVING_NOTE`); a non-stdout
trace value `P_TRACE_VALUE`; a `stdout` row in a middle step `P_MIDDLE_STDOUT`
and the final step's `stdout.now` = `P_CORRECT`. `canary_question_dnc()`: same
prose, `compile_error_code: [P_ERROR_CODE]` → receipt `✓ Error P_ERROR_CODE`,
D correct. Admin token plant `P_ADMIN`.

## What is scanned, at every revision (every host transition and every ← →)

For one room driven idle→live→closed→split→work(→ to the bound, one more
refused)→reveal(← through the whole trace, → back)→released→run-it-again:

1. HTTP projections `GET /rooms/{id}/wall|buzzer|host` (+ host without / with
   wrong bearer: 401 empty).
2. Mutating probes, each response scanned: `POST /join` (a fresh session: its
   buzzer payload), `PUT /rooms/{id}/answer` (a probe session), `PUT
   /rooms/{id}/fit`, the host action's own response.
3. Socket frames, **every** frame each socket received since the last stop:
   one wall, one host, three buzzers (real tokens, answers A/B/C in live);
   drained until each equals the viewer's current projection (minus `t`,
   `revision`, and the buzzer attach frame's `session`), so no stop is scanned
   before its frame arrived.
4. Pages and every script/style they load, fetched each stop: `/wall/{id}`,
   `/host`, `/host/{id}`, `/join`, `/join?code=`, `/{code}` (redirect +
   `Location`), and every asset in `routes.rs`'s lists; fonts once per room
   (compile-time bytes) and asserted byte-identical at each stop.
5. Rendered wall HTML (node, `PQ.Wall.html`) for every wall frame.
6. The real buzzer page's rendered HTML and its request log per phase.

**Route walk:** the test parses every `.route("…")` literal and generated path
family from `room/src/routes.rs` and fails if one is neither driven by the scan
nor on the scan's written *unscanned* list — a new route cannot escape.

## Rules (by criterion)

- **G-3 / AC-47 / AC-60** — before `reveal`, on every surface: no
  `P_RESOLVING_NOTE`, `P_WHAT`, `P_TAKEAWAY`, any why, `P_RECEIPT`,
  `P_ERROR_CODE`, `P_MIDDLE_STDOUT`, receipt heading/lines, ✓; no key
  `correct|kind|why_tempting|explains|receipt|reveal|read_aloud|mark`; no
  `values` row named `stdout`; `P_CORRECT` only inside the wall's
  `options[4].text` (exactly one occurrence per wall payload in
  live/closed/split/work, zero on host/buzzer/pages); every option object
  exactly `{letter,text}` in bank order. Host pre-reveal: none of the answer
  plants (AC-47).
- **AC-48** — `P_HINT` absent from every surface before `live` and from
  `closed` on; in `live` present in every buzzer projection, buzzer frame and
  join response, absent from wall/host payloads, frames and pages. Taking the
  hint on the real page makes no request and no socket message, and no wall or
  host frame arrives because of it.
- **G-8 / AC-32** — every buzzer-class surface (projection, frames, join and
  answer responses, the buzzer page and its scripts, the page's rendered
  HTML), every phase: no `P_SOURCE`, no `P_NOTE_*`, `P_RESOLVING_NOTE`,
  `P_TRACE_VALUE`, `P_MIDDLE_STDOUT`, no option text (incl. `P_CORRECT`), no
  `source|trace|options|step` key.
- **AC-97 / G-6** — in `work`: wall `colour:false`, no ✓, no receipt, no
  stdout row, `trace.at ≤ M-2` at every step, one more → refused (409).
- **AC-79** — the served wall HTML and every rendered wall frame: no
  `<button <input <select <textarea <a  <form onclick onkeydown
  contenteditable`.
- **AC-58 / G-4** — the real buzzer page: after its `closed` frame arrives, no
  request it makes carries the answer: no `PUT …/answer`, no body with a
  `letter`, every socket message exactly `{t:"attach",token}`; a tap after
  close sends nothing; the split's *n people said X, including you* in the
  rendered HTML equals the broadcast total for its own letter (computed on the
  phone; no payload carries `yours`/`saved` except the attach frame's own
  `session.saved`). A server-side request log (a test middleware on the
  router) confirms nothing bearing the page's token reached the server after
  close except the socket upgrade.
- **AC-101 (canary half)** — `P_ADMIN` in no payload, frame, page, rendered
  HTML or node log, in any phase.
- **Positive controls (a silent no-op cannot pass)** — at `reveal`: wall has
  `P_CORRECT` in the final step's stdout, `P_RESOLVING_NOTE`, `P_RECEIPT`,
  `reveal.correct:"E"`, ✓; stepping back shows `P_MIDDLE_STDOUT` and every
  `P_NOTE_i`; host has `P_WHAT`, `P_TAKEAWAY` and the named option's why;
  buzzer `correct:"E"`. In `live` the buzzer carries `P_HINT` and the wall
  `P_SOURCE`. The dnc run's wall receipt carries `P_ERROR_CODE`. The scanner
  itself is checked: a planted string injected into a copy of a payload is
  caught (a self-test of the rule function).
- **released** — every plant gone from every surface (released closes the
  answers again).

## test-full additions (`canary_full.rs`)

Same rules; HTTP over real sockets; two rooms (the second via run-it-again
onto `canary-dnc`); reconnect: three buzzers answer A/B/C in `live`; in
`closed` buzzer B's socket is dropped and a new one attaches with B's token:
the attach frame's `session.saved` is `B`, the frame contains no other
session's letter as `saved`/`yours`, and the old socket closes with 4000 only
if still open; later broadcasts carry no `session`.

## Open choices made

1. **Leak = stop.** If any rule fails against the code on main, the test
   stays failing, the ticket goes `needs_human` with the payload and phase.
2. The admin-token plant is runtime-random, never a literal (T-25's repo scan).
   The room reads no such variable today (no admin route exists until T-25),
   so the assertion is honest but currently can only fail if a payload
   echoes the environment; README says so.
3. `exit_code` renders nowhere (the receipt never reads it; `receipt_lines`)
   and cannot carry a string; it is asserted as a key nowhere in any payload
   rather than planted as text.
4. Pages are compile-time bytes: scanned every stop anyway (cheap); fonts
   once per room plus a byte-equality check each stop.
5. node is already required by `just test` (test-web, and `buzzer_page.rs`
   spawns it); the canary's page driver uses it the same way.
6. New files outside the cleared list (`canary_scan/mod.rs`, `page.js`) are
   new, owned by T-08, not shared — flagged as a deviation.

## Contract tension

- BUILDPLAN/boot ask for a plant in the **correct answer text**; EVALUATION's
  `canary` row says `verified.stdout` is never a canary because option texts
  are public (AC-62). Side taken: plant it, and assert the *join* instead —
  it appears pre-reveal only as the wall's option E text, exactly once, never
  on host or buzzer, never in a stdout row. Both documents are satisfied.
- EVALUATION says the admin-token plant arrives "once T-25 lands"; the boot
  prompt puts it in T-08. Side taken: plant it now (cheap, and T-25 inherits a
  scan that already covers every surface).
- `test` scans over the loopback listener (sockets) as well as in-process
  HTTP; EVALUATION calls the `test` half "in-process". Side taken: the boot
  prompt's `Live`-harness instruction — the sockets are loopback and hermetic.

## Tests by criterion

AC-32, AC-47, AC-48, AC-58, AC-60, AC-79, AC-97, AC-101(canary half), G-3,
G-4, G-8 — each rule above is its own named `assert!` message, and each
criterion has at least one test function name carrying its ID.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: Sonnet, fresh context, contract paths only.

1. **Critical — the deployed-room scan is handed to `smoke`, contradicting
   EVALUATION's `canary` row and BUILDPLAN T-08 ("the deployed room's real
   frames and pages … in `test-full`").** *Resolution:* the Orchestrator's
   dispatch instruction for this ticket reads "The deployed-room variant is
   T-09's `smoke`; leave the hook (a documented `--url` flag) and say so." That
   is followed, but not by redefining `smoke`: `canary_full`'s scan core is
   written against a server base URL, and the deployed variant is **the canary
   pointed at T-09's URL**, which T-09 wires into `smoke`. It cannot run today
   for two reasons outside this ticket: nothing can create a room on a deployed
   server until T-09's stand-in token (§8.2), and nothing can put the planted
   question on it until T-25's admin push (§8.3) — the canary has nothing to
   plant. `just canary --url <u>` therefore fails naming both. Listed under
   *Contract tension* in the PR and the completion comment, for the
   Orchestrator to route (EVALUATION's `canary` row promises more in
   `test-full` than can exist before T-09 + T-25).
2. **Major — `test-full` never scans a deployed room.** Same resolution as 1;
   `test-full` runs the whole scan through real TCP and WebSocket connections
   to a room served on loopback by the same `ws::serve` the binary uses. Said
   plainly in README and PR: "a running server over real sockets", never
   "deployed".
3. **Major — AC-101's canary half is vacuous today and AC-101 is not in T-08's
   criteria.** *Resolution:* agreed. The PR and README claim only that the
   plant is set and every surface is scanned for it; it cannot fail until a
   room reads the token (T-25). AC-101 is listed as "scaffold for T-25", not as
   proven. T-25's row ("extends `canary` with `POPQUIZ_ADMIN_TOKEN` as a
   plant") becomes "turns the existing plant live" — flagged for the
   Orchestrator in the completion comment (finding 6).
4. **Major — §8.3's "no log line" clause has no check.** *Resolution:* the
   room writes no log line today (its only output is `main`'s bind line), so
   there is nothing to capture; the server's *request* log the canary keeps is
   scanned for the token like every other surface. Log-line capture belongs
   with the code that first reads the token (T-25). Stated in README.
5. **Minor — AC-79's `test` half.** `wall_page.rs` already asserts the served
   wall has no control; the canary adds the rendered frames. Attribution fixed
   in README.
6. **Minor — T-25 friction.** Noted for the Orchestrator (see 3).
