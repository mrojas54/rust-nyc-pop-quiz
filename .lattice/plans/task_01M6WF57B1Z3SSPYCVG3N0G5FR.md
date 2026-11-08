# PQ-87: Announce the host picker's loading, empty and failed states to screen readers

From PQ-80 (PR #136, merged ae4d8a0): the delegator's DONE listed it as a finding not acted on.

**Gap:** the /host question picker (SPEC §8.5) draws its loading, empty and failed notes (web/host/host.js:210-212, copy keys host_pick_loading, host_pick_empty, host_pick_load_failed) but never announces them. SPEC §10 requires a polite live region per state change; the host page already announces phase changes through the shared PQ.announce region (web/shared/dom.js:43, host.js:476), the picker does not.

**What would close it:** each picker state change (loading → list, loading → empty, loading → failed, Reload → loading) announces its existing copy line once through PQ.announce. No new copy, no new region, no layout change. RED first: a web test that fails on main because the announcement is missing; extend the a11y screens (just a11y) for the picker states.

**Check while there:** the drawn 44 px radio's focus ring (594d5cd, never seen by eye, no Codex review).

**Criteria:** SPEC §10 (live region per state change), AC-83 pattern, SPEC §8.5. No AC change expected; if one is, stop and ask.

**Harness:** just test (web: host-pick.test.js, a11y.test.js), just a11y.

Held: released by the client by name. Re-verify the line numbers on the current main before planning.

# Plan (delegator, 2026-11-08)

Re-verified at ae4d8a0: host.js:210-212 draw the notes; host.js:476 is the only PQ.announce; dom.js:43 announce (40 ms clear-then-set).

## Transitions and what each says (existing keys only, through PQ.announce)

| Transition | Says | Key |
|---|---|---|
| opening → loading | nothing | — |
| loading → list | *Question* | `host_pick_legend` |
| loading → empty | *No question is ready for this club today.* | `host_pick_empty` |
| loading → failed | *Couldn't load the questions.* | `host_pick_load_failed` |
| Reload (or the reload after a refused create) → loading | *Loading questions…* | `host_pick_loading` |
| loading → denied (403) | nothing new; the reason is the status line (unchanged, deferred) | — |

- **loading → list announces.** SPEC §10 says "polite live region per state change" with no exception, so silence there is not allowed by its wording. There is no "list loaded" key; `host_pick_legend` (*Question*) is the name of the group that just appeared, the same word a reader says on entering it. No new copy.
- **Opening is silent.** The loading line is on the screen as the page loads and is read with it; the ticket's list names Reload → loading, not opening. host.js:473's first-payload rule is for phase labels, which are never shown.

## Repeat guard

`shown` = the picker state last said, starting at `loading`. `paintCreate` announces only when a session is present and `pick.state !== shown`, then sets `shown`; announce before repaint, as host.js:476 does. Same-state repaints (the change event's fallback repaint, *Create a room* going in flight) say nothing.

## Tests

host-pick.test.js (RED on ae4d8a0, GREEN after), capturing `PQ.announce`:
1. opening says nothing; loading → list says *Question*, once.
2. loading → empty says `host_pick_empty`.
3. loading → failed (503 and network) says `host_pick_load_failed`.
4. empty → Reload → list says `[empty, loading, legend]`.
5. a same-state re-render (pick, create in flight) leaves the log at exactly `[legend]`.

a11y.test.js: the picker screens (`pick, loading`, `pick`, `pick, empty`, `pick, failed`) already exist from PQ-80; they gain AC-83 cells from a booted run. The "say nothing" test keeps `sign in` silent and updates the pick and refused-create cases to what they now say.

## Focus ring

`.host-phone :focus-visible` (host.css:44) gives the drawn radio a 2 px `--focus-ring` outline, offset 2 px. That's a shape, not only a colour, at the token the buttons use (3:1). Check it by eye; edit host.css only if it's missing.

## Rendered check

A stub server in the scratchpad serves `web/` and answers `/organizer/questions` by club (`ok`, `empty`, `failed`, `slow`). Drive `/host?club=…#tok` in the c11 browser: three notes, plus Tab onto a radio. Screenshots go in the scratchpad. No sign-in needed.
