# PQ-40: Accessibility follow-ups: focus without scroll, the dimmed trace lines, the suite's blind spots

Follow-up of PQ-16 (PR #37, merged at 475700c). Source: the exact-head review artifact art_01M3YZJWY721D3M8FYP9CSH2X3 on PQ-16. MUST LAND BEFORE THE NEXT DEPLOY. (A) MAJOR, new in #37: focus is restored on every repaint with focus() and no preventScroll - web/host/host.js:271, web/buzzer/buzzer.js:579 and :581, web/home/home.js:254. On the host in reveal the primary button sits below the read-aloud text (host.js:173-174), so a repaint can scroll the host past the beats. Fix: focus({preventScroll:true}) at each site, a test that fails without the option, and a real-browser check on the host in reveal. (B) MAJOR, F-46, the client's ruling pending: web/shared/components.css:47 .rn-src-line.dim{opacity:.32} is 1.84:1 and fails AC-84; the wall has no full-contrast copy of those lines during work or reveal. The client asked to SEE OPTIONS FIRST: render the wall's trace at opacity .32, .55 and .70 (computed ratios 1.84, 3.17, 4.78 against 11.99 at full strength - recompute them, do not copy), attach the three screenshots with their ratios, then stop for the client's pick (needs_human plus a flag). Do not change the value before the pick. After the pick: AA chosen -> change the opacity, remove the dim exemption from the suite and assert the floor; look kept -> scope the exemption to .rn-src-line.dim, state the ratio in a comment at the CSS rule, and report F-46 for upstream. (C) Minors: web/test/a11y.test.js:658 exempts any element with a dim class and asserts no floor; web/home/home.js:168 pre.vv has tabindex and aria-label but no role (role=region, as web/shared/well.js:116); matrix cells are literal strings (a11y.test.js:617,633,822,834,841,847), :525 cannot fail alone, :519 is dead code - derive the cells from the assertions; focus restore plus PQ.announce may make a screen reader re-speak the control (host.js:271, home.js:254) - check and say what you could verify; the non-text edge list is hand-picked - examine .btn and .buzz edges (about 1.4:1 on --border-default) and say whether SC 1.4.11 applies. NITs: the buzzer join submit gets no focus key. Out of scope here: justfile:63 (a11y runs twice, a11y *ARGS gone) - the justfile is PQ-26's until PR #39 merges; DESIGN.md's missing amendments for the colour shifts - contract, upstream. Acceptance: AC-84, AC-85, AC-33, AC-40; just test and just a11y green; each new or rewritten check shown to fail under its mutation. Harness: just a11y, just test-web, just test.

# Plan (delegator, 2026-10-02)

Base origin/main @ 475700c. Line numbers re-found at that sha.

## (B) first — the dim options (F-46), no source change before the pick
- Serve the repo root (`python3 -m http.server 8766 --bind 127.0.0.1`), open
  `web/test/a11y/browser.html?surface=wall&frame=work` in the c11 browser (the live
  wall's mount on q3's work frame, which dims outside the focus region). Size the
  browser so the 1120×630 canvas renders at full size.
- For each of .32/.55/.70: inject a page-only `<style>.rn-src-line.dim{opacity:V}</style>`
  via `eval` (components.css untouched), screenshot, save under the scratchpad.
- Ratios: a node script using `web/test/a11y.test.js`'s own cascade path
  (dom.js + wcag.js: W.blend(color, bg, opacity) then W.contrast) on work[0]'s html,
  with the opacity overridden — dimmed text and the highlighted line.
- Attach three files (serially), PICK comment, needs_human + flag. Wait for the pick;
  continue with (A) and (C) meanwhile.
- After pick: AA → change components.css:47, remove the exemption, dim lines held to
  the normal floor; else → keep, scope exemption to `.rn-src-line.dim`, assert the
  chosen ratio as a floor, comment at the CSS rule, report F-46 upstream.

## (A) focus without scroll — AC-82 regression guard, AC-33 spirit
- `focus({ preventScroll: true })` at host.js:271, buzzer.js:579 and :581, home.js:254.
- dom.js `Node.focus(opts)` records `opts` on `ownerDocument.focusCalls`.
- a11y.test.js: the three existing focus-survives-repaint tests assert every
  restore call carried `{ preventScroll: true }`. Mutation: drop the option at
  each site → the test fails.
- Browser: host via browser.html?surface=host at a phone-sized window, step to
  reveal, scroll to the read-aloud text, trigger a repaint (harness), read scrollY
  before/after with eval. WKWebView: synthetic Tab moves no focus, so focus is set
  by `.focus()` in eval, which is the same call the repaint makes.

## (C) blind spots
1. a11y.test.js:658 — the dim exemption keys on `.rn-src-line.dim` ancestry only;
   floor per (B). Mutation: a non-trace `.dim` element with low contrast → held, fails.
2. home.js:168 — `role="region"` on pre.vv; ac82_home asserts it. Mutation: remove role.
3. Matrix cells derived from what ran (:617 host create/sign-in boot says nothing
   — booted and recorded; :633 home — traceSay per screen asserted; :822/:841/:847
   — per-screen motion count from the cascade, asserted 0, plus the matched cue;
   :834 — the matched "Step N of M"). :525 removed (deepEqual pins every string,
   comment says so); :519 dead `name`/`void` removed. Each shown failing under a
   named mutation.
4. Double announcement: host phase change = label (live region) + focus on the
   new action (different words, intended). Same-phase repaint (count tick) re-focuses
   a fresh copy of the same button → screen reader re-speaks it. Fix in host.js:
   when only the count line changed, update `.host-count` in place, so the focused
   node survives; test asserts no focus call and the same node on a count tick.
   home.js:254 — step redraw re-focuses the nav button then announces the step;
   keeping to the two spots (PQ-27 owns the rest), noted, not restructured.
5. .btn/.buzz edges (~1.4:1): SC 1.4.11 asks for contrast of what is *required*
   to identify a control; a labelled button is identified by its text (Understanding
   1.4.11, "text buttons"), the primary by its fill. Reasoning into the suite comment
   and README; no token change.
6. NIT: buzzer join submit gets focus key "join" (keyOf + focusTarget: two lines,
   not one — noted).

## Files
web/host/host.js, web/buzzer/buzzer.js, web/home/home.js (two spots),
web/shared/components.css (:47 + comment, after the pick), web/test/a11y.test.js,
web/test/a11y/dom.js, web/README.md.

## Criteria
AC-84 (dim floor, text pairs), AC-85 (unchanged, rerun), AC-82 (focus restore,
preventScroll), AC-83 (announce matrix), AC-86 (motion cells), AC-33/AC-40 (no
sideways scroll, no colour-only — rerun).

## Tension
- `.btn` edge: AC-84 lists non-text at 3:1 for "edges that identify a control";
  I take the side that a labelled button's edge is not identifying (1.4.11).
- (B) below-AA pick would contradict AC-84 → F-46 upstream, not a criteria edit.

## Reset 2026-10-02 by agent:delegator-pq40
