# PQ-88: Announce the host picker's 403 reason to screen readers

From PQ-87 (PR #137, merged 00c7753): the delegator's DONE listed it as not done, out of scope.

**Gap:** when /organizer/questions answers 403, the picker goes to state `denied` and sets `ui.status = statusLine(...)` (web/host/host.js:423-425 on 00c7753). PQ-87's PICK_SAID has no `denied` entry, so nothing is said through PQ.announce. The reason is drawn in `<p class="host-status" role="status">` (host.js:115), but repaint replaces that element with its text already in it, and a live region inserted with its content is usually not announced.

**First:** confirm whether the reason is spoken today (a11y.test.js live-region read-back, or the c11 browser). If it is, close with that evidence.

**What would close it:** on a change to `denied`, the room's reason (the status line text, no new copy) is said once through PQ.announce, following PQ-87's last-state-said guard; a repaint that keeps `denied` says nothing again. The same check for refused-create (host.js:460), which also only sets the status line.

**Criteria:** SPEC §10 (live region per state change), AC-83 pattern, SPEC §8.5. No AC change expected; if one is, stop and ask.

**Harness:** just test (web: host-pick.test.js, a11y.test.js), just a11y.

Held: released by the client by name. Re-verify the line numbers on the current main before planning.
