# PQ-87: Announce the host picker's loading, empty and failed states to screen readers

From PQ-80 (PR #136, merged ae4d8a0): the delegator's DONE listed it as a finding not acted on.

**Gap:** the /host question picker (SPEC §8.5) draws its loading, empty and failed notes (web/host/host.js:210-212, copy keys host_pick_loading, host_pick_empty, host_pick_load_failed) but never announces them. SPEC §10 requires a polite live region per state change; the host page already announces phase changes through the shared PQ.announce region (web/shared/dom.js:43, host.js:476), the picker does not.

**What would close it:** each picker state change (loading → list, loading → empty, loading → failed, Reload → loading) announces its existing copy line once through PQ.announce. No new copy, no new region, no layout change. RED first: a web test that fails on main because the announcement is missing; extend the a11y screens (just a11y) for the picker states.

**Check while there:** the drawn 44 px radio's focus ring (594d5cd, never seen by eye, no Codex review).

**Criteria:** SPEC §10 (live region per state change), AC-83 pattern, SPEC §8.5. No AC change expected; if one is, stop and ask.

**Harness:** just test (web: host-pick.test.js, a11y.test.js), just a11y.

Held: released by the client by name. Re-verify the line numbers on the current main before planning.
