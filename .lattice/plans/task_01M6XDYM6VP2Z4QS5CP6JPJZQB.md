# PQ-89: Announce a network error on Create a room to screen readers

From PQ-88 (PR #139, merged f5fa38a): its DONE and PR body listed this as not done, out of scope.

**Gap:** a network error on *Create a room* says nothing to screen readers. At f5fa38a, web/host/host.js:467-468 (the `post("/rooms", …)` rejection) sets `ui.busy = false; ui.status = String(e && e.message || e); paintCreate();`. `pick.state` is unchanged (still `ready`), so PQ-87's last-state-said guard in `paintCreate` (host.js:400-410) says nothing; the error is only drawn in `<p class="host-status" role="status">`, which repaint inserts with its text already in it (usually not announced; PQ-88 established this for the 403 case).

**First:** confirm with evidence that it is silent today (host-pick.test.js `hearing()` read-back, or the c11 browser with POST /rooms failing at the network). If it is spoken, close with that evidence.

**What would close it:** a rejected *Create a room* says the status line's own text (the error string already shown; no new copy, no new region) once through PQ.announce; a repaint that keeps the same state and status says nothing again; a second press that fails again says it again (it is a new attempt). Same treatment PQ-88 gave a refused create.

**Out of scope:** the hosted-room page's error paths (host.js:501-571), the two-region double-read of the reason (PQ-88 contract note). List any further silent path found in DONE; do not fix it.

**Criteria:** SPEC §10 (live region per state change), AC-83 pattern, SPEC §8.5. No AC change expected; if one is, stop and ask.

**Harness:** just test (web: host-pick.test.js, a11y.test.js), just a11y.

# Plan (delegator, 2026-11-08)

## Evidence: silent today
At f5fa38a, four new tests in `web/test/host-pick.test.js` and the extended `ac83_host` pre-room test in `web/test/a11y.test.js` fail (`node --test web/test/host-pick.test.js web/test/a11y.test.js`: 68 tests, 64 pass, 4 fail). Each `hearing()` read-back after a POST /rooms that rejects at the network holds only `['Question']`: the legend said when the list arrived, and nothing for the failure. The error text (`offline` / `FIXTURE-NETWORK`) is on screen in the status line. So: not spoken; code needed.

## What is said, and when
- A *Create a room* rejected at the network says the status line's own text, `String(e && e.message || e)`, once, through `PQ.announce`. No new copy, no new region.
- A repaint that keeps the state and the line (a pick, re-picking) says nothing again.
- A second press that fails again says it again: each press is a new attempt.

## How it sits with PQ-87's guard and PQ-88's `paintCreate(line)`
One change in `paintCreate`: a `line`, when given, is always said (and `shown` still takes the state). Today `line` is said only when the state changes. The one existing caller, `load(line)` after a refused create, always changes the state (ready → loading), so what it says is unchanged. The rejection branch becomes `paintCreate(ui.status)`. With no `line`, PQ-87's last-state-said guard is exactly as before, so a repaint says nothing. The guard decides when a *state* is said; `line` is an event (a response to a press), said once per press because only the press's own response handler passes it. No other call site passes `line`, so no other transition's speech changes. Considered and rejected: tracking the last status said (it would silence a second identical failure, which must be said), and calling `PQ.announce` directly in the handler (a second path around the one place that decides what the picker says).

## Tests (RED at f5fa38a, GREEN after)
- host-pick: a network error on create says the status line as drawn, once (`[legend, 'offline']`, the list stays); after it, re-picks say nothing again; a second failed press says it again (`[legend, 'offline', 'offline']`, two POSTs).
- a11y: new `pick, create failed` host screen (renderPick with the error status, list ready), so it runs through the AC-82…AC-86 matrix; the `ac83_host` pre-room test gains that case, booted for real, POST rejecting with `FIXTURE-NETWORK`; its matrix cell prints `"Question", "FIXTURE-NETWORK"`.
- Every PQ-87/PQ-88 announcement test stays unchanged and green.

## Rendered check
Scratchpad python stub on a free port (checked with lsof) serving `web/`, answering GET /organizer/questions with two placeholder questions and closing the connection without a response on POST /rooms. c11 browser at `/host#FIXTURE`: pick, press *Create a room*, read the live region (`.sr-only[role=status]`) and the status line back with `browser eval`; press again and read again. Screen-reader speech itself is out of reach and will be said so.

## Files
`web/host/host.js` (paintCreate guard and comment; rejection branch); `web/test/host-pick.test.js`; `web/test/a11y.test.js`. Nothing under `web/shared/`, no copy change.

## Reset 2026-11-08 by agent:orchestrator
