# PQ-88: Announce the host picker's 403 reason to screen readers

From PQ-87 (PR #137, merged 00c7753): the delegator's DONE listed it as not done, out of scope.

**Gap:** when /organizer/questions answers 403, the picker goes to state `denied` and sets `ui.status = statusLine(...)` (web/host/host.js:423-425 on 00c7753). PQ-87's PICK_SAID has no `denied` entry, so nothing is said through PQ.announce. The reason is drawn in `<p class="host-status" role="status">` (host.js:115), but repaint replaces that element with its text already in it, and a live region inserted with its content is usually not announced.

**First:** confirm whether the reason is spoken today (a11y.test.js live-region read-back, or the c11 browser). If it is, close with that evidence.

**What would close it:** on a change to `denied`, the room's reason (the status line text, no new copy) is said once through PQ.announce, following PQ-87's last-state-said guard; a repaint that keeps `denied` says nothing again. The same check for refused-create (host.js:460), which also only sets the status line.

**Criteria:** SPEC §10 (live region per state change), AC-83 pattern, SPEC §8.5. No AC change expected; if one is, stop and ask.

**Harness:** just test (web: host-pick.test.js, a11y.test.js), just a11y.

Held: released by the client by name. Re-verify the line numbers on the current main before planning.

# Plan (delegator, 2026-11-08)

## Is the reason spoken today? No.

- **Rendered, at `ab98b9b`:** the c11 browser loaded `web/` from a scratchpad stub whose `/organizer/questions` answers 403 `{refusal: wrong_role, reason: <host_denied_wrong_role>}`. After the page settled, `#host` held `<p class="host-status meta" role="status">That Discord account doesn't have the organizer role.</p>`, and the body held **no** `aria-live` element. `PQ.announce` creates its region on first call, so it was never called. The reason exists only in a `role="status"` node that `repaint` inserts with its text already in it, and screen readers usually stay silent for a live region inserted with its content. Screen-reader speech itself is out of reach here.
- **Code:** `PICK_SAID` (host.js:226) has no `denied` key, so the `paintCreate` guard (host.js:397-403) says nothing on the change to `denied`. Refused create (host.js:460) sets `ui.status` and calls `load()`, which says only the loading line.

## What each transition says (no new copy, no new region, no layout change)

| Change | Said through `PQ.announce` |
|---|---|
| loading → denied (403 on the list) | `ui.status`, the status line's own text (`statusLine(res, body)`: the room's `reason`, else `<status> <statusText>`) |
| ready → loading after a refused *Create a room* | the refusal's status line, **instead of** the loading line |
| loading → ready / empty / failed; Reload → loading | unchanged (PQ-87) |
| opening, sign in, a repaint that keeps the state | nothing |

**Guard:** PQ-87's `shown` check stays the only gate. `paintCreate(line)` takes an optional line. On a change of state it says `line` when given; otherwise, for `denied`, `ui.status`; otherwise `PICK_SAID[state]`. A repaint that keeps the state says nothing, `denied` included. `load(line)` passes its line to the first paint, and the refused branch calls `load(ui.status)`.

**Refused create: the reason instead of the loading line.** `PQ.announce` clears its region and sets the text 40 ms later. Two calls in one tick both land in the same later tick, and the second replaces the first (the same-tick note in `web/shared/dom.js`), so saying both would lose one of them, probably the reason. The reason is the line the organizer needs. The loading state is brief, and when the list comes back its own line follows (legend, empty, failed or denied). AC-83's "each state change once" still holds: the change to loading is said once, in the words that explain it. Not saying it at all would leave the refusal silent, which is this ticket's gap.

## Tests (RED on `ab98b9b`, GREEN after)

`web/test/host-pick.test.js`, using the `hearing()` and `scripted()` helpers:
1. A 403 says the room's reason once: `said` deep-equals `[reason]`.
2. A 403 with no `reason` in the body says the status-line fallback (`403`), so the text is always the status line's own.
3. Nothing re-says it: once the list is denied, the page's own events (a pick change, a *Create a room* press, more ticks) leave `said` at one entry. No code path repaints while denied today; this pins the guard.
4. Refused create says the reason in place of the loading line, then the legend: `[legend, reason, legend]`.

`web/test/a11y.test.js`:
- Add a `pick, denied` screen (`renderPick({status: 'FIXTURE-REASON'}, {state: 'denied'})`) to `hostScreens()`. Its AC-86 row is like `pick, loading`: no action. The status line is the whole screen (SPEC §8.5).
- Add a booted 403 to the AC-83 host tests, printing `"<reason>"` in the `pick, denied` cell.
- Change the `pick, refused` AC-83 expectation from `[legend, loading, legend]` to `[legend, "200", legend]`. In that test's script, *Create a room* is refused with a 200 and no room.
- `just a11y` passes and prints the cell.

## Rendered check

The same scratchpad stub (403 on the list), served on 127.0.0.1, in the c11 browser. After the fix, read back the text of the `aria-live` region `PQ.announce` adds to `<body>`: it should be the reason. A second stub mode answers 409 to `POST /rooms` after a 200 list, to read back the refused-create line. Nothing outside the scratchpad.

## Contract notes
- On a 403 the reason sits in two regions, the status `<p role="status">` and the announce region. A reader that does announce the inserted `<p>` may say it twice. Removing the `<p>`'s role would change markup that is out of scope, so it stays.
- Deferred, not done: a network error on *Create a room* (host.js:462) sets `ui.status` with the state still `ready`, so nothing is said. That path is not in this ticket.
