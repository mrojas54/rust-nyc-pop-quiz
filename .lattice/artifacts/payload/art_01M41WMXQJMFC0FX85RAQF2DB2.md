REVIEWING PQ-40 CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/a11y-followups HEAD d1db359d5e368f6b7b3e3cac698a3462c8b9a44d BASE cd4e37372aa7e7e52e1441d9c05249fd22abea4b

Verdict: PASS-WITH-NITS

Reviewed `git diff cd4e373 d1db359` (7 files, +264/-48): web/README.md, web/buzzer/buzzer.js, web/home/home.js, web/host/host.js, web/shared/components.css, web/test/a11y.test.js, web/test/a11y/dom.js. Ran `A11Y_MATRIX=1 node --test web/test/a11y.test.js` (31/31) and `node --test 'web/test/*.test.js'` (279/279) under node v22.22.3. Replicated ten of the ticket's mutations on copies under $TMPDIR (nothing in the worktree was written). No Critical and no guardrail-breaking Major.

## Critical

None.

## Major

None.

## Minor

1. Wrong colour named for the dimmed line numbers, and F-46's remedy points at a rule that does nothing.
   - web/shared/components.css:49 and web/test/a11y.test.js:759-760 say the line numbers "start at --text-muted". On the only two surfaces that render the well, `.wall .rn-src-ln` (web/wall/wall.css:112) and `.home-page .rn-src-ln` (web/home/home.css:92) override the base rule (components.css:41) to `--text-secondary` (#4a5568, 7.53:1 undimmed). The 3.57:1 in the suite is #4a5568 at opacity .70 (= #808895, measured 3.577:1). From `--text-muted` (#718096) the same dim would be about 2.5:1, so "start lighter" is false.
   - Proved by mutation: recolouring `.rn-src-ln` in components.css:41 changes nothing (still #808895, 3.57, 31/31). Removing the wall.css:112 override fails ac84_wall.
   - Consequence: the DONE comment's F-46 note ("a gutter colour change (components.css:41)") would be a no-op if acted on. The effective rules are wall.css:112 and home.css:92; `--text-primary` there would make the gutter 4.78:1 at .70.
   - Recommendation: say `--text-secondary` in both comments and correct the upstream F-46 record to name wall.css:112 / home.css:92. The suite itself is correct, because it measures the real cascade.

2. The host's count-only rewrite is a product-code change beyond the three listed asks, and its one coupling is guarded only by a comment.
   - web/host/host.js:265-283. `repaint()` slices `html` at `lastIndexOf('<div class="host-count">')` and writes the tail minus 6 chars (`</div>`) into the existing node. This is correct today (render() appends countHtml last, host.js:174; `esc()` makes a forged marker impossible; every non-count change alters the prefix and falls back to a full repaint). The ticket's DONE notes disclose it, and the new test (a11y.test.js:529-552) proves the same-node, no-focus-call, nothing-said-twice behaviour.
   - Gap: nothing asserts that render() still ends with the count. If something is appended after countHtml, a tick would slice that trailing element into `.host-count` and the tick test would still pass.
   - Recommendation: add one assertion in the host test that `H.render(p, ui)` ends with `'</div>'` and that its last `host-count` div closes the string.

3. SC 1.4.11 reading is an owner interpretation of AC-84 ("Text and essential UI meet WCAG AA").
   - README.md:94, a11y.test.js:836-844 and :867-873 print the `.btn` and `.buzz` edges (1.41:1) as "supplementary: its label identifies it" rather than holding them. This follows Understanding 1.4.11 for text-labelled buttons and was disclosed as deviation 3; before this PR those edges were not examined at all, so the PR adds information rather than relaxing a check.
   - It is still a reading the AC-84 owner has not ruled on; the buzzer's letters are the room's primary control.
   - Recommendation: send it upstream with F-46 for a ruling.

4. Matrix cells are mostly computed now, but not all.
   - a11y.test.js:441: the `live, saving` AC-82 cell is overwritten (the later mark replaces "6 ctl, ring ok", so that screen's native/name/ring result no longer prints). The "6 no-scroll" count is the whole test's restore count, not that screen's, and the cell is cut to "focus kept, 6 no-s" by the 18-char slice at :246.
   - Remaining literals: :315 ("ring ok"), :328 ("region, ring ok"), :625 ("6 strings, =live"). The asserts above them do run, so these are low-risk.
   - `still()` (:974-978) returns `m.length` after asserting `m` is empty, so every "N mv" cell is a constant 0, not a measurement.
   - Recommendation: key the focus cell on a screen the test really drives or on its own row, and widen the column or shorten the string.

## NIT

- web/test/a11y.test.js:767 says "components.css:47"; the rule is now at components.css:51 (47 is the comment's first line).
- web/test/a11y.test.js:959 `MOTION_PROPS` lacks `transition-duration` / `animation-duration`, so a per-screen guard would miss a duration-only transition. The sheet-level ac86_* test (:921, regex `^(animation|transition)(-|$)`) does catch it, so this is defence in depth only.
- web/README.md:129-134 expects "the focus is back on the primary action" after a pointer press on a step button. The code restores whichever control held the focus (`step:<slug>` if the engine focuses a button on click, primary if it does not), so the expected outcome is engine-dependent.
- F-46 lives only in .lattice/orchestration and code comments; DESIGN.md and USER_STORIES AC-84 are unamended (the ticket says that is upstream, which is consistent).

## Verified clean

- Pinned state: `git rev-parse HEAD` = d1db359d5e368f6b7b3e3cac698a3462c8b9a44d; merge commit parents 2205efb + cd4e373, signature status G; worktree clean except untracked `.claude/`; `git diff --check` clean.
- (1) preventScroll: every `.focus(` in non-test web code (buzzer.js:584 and :586, host.js:292, home.js:254) passes `{ preventScroll: true }`. No other focus(), scrollIntoView, autofocus or blur in product JS. The new tests assert `deepEqual(opts, {preventScroll:true})` over every recorded restore for buzzer (both sites), host and home. Mutations dropping it at buzzer.js:584, buzzer.js:586, host.js:292 and home.js:254 each fail exactly the matching test (ac82_buzzer refused-join, ac82_buzzer letters, ac82_host, ac82_home).
- (2) dim: components.css:51 is `opacity:.70`. Hand-computed #6c737f on #fff = 4.78:1 and #808895 on #fff = 3.577:1, matching the matrix. `DIM_GUTTER_FLOOR = 3.57` applies only to `.rn-src-ln` inside `.rn-src-line.dim` (a11y.test.js:780, :787); every other dimmed text is held at 4.5. Mutations: opacity .32 and .69 fail ac84_wall and ac84_home, `.71` passes, and `.70` plus a transition fails ac86_*. The suite covers wall work/reveal/unanimous frames and the static fallback, plus home at all three snapshots.
- (3) blind spots: role-on-scroller check (a11y.test.js:307-309) fails when `role="region"` is removed from pre.vv. The host count-tick test fails when the count-only branch is disabled. New silent-screen test for sign-in, create and refused-create; per-step announcement check on all three home snapshots; seven-distinct-actions check; `still()` guard fails on an added transition. The removed vacuous `||` assertion is correctly replaced by the ordered `deepEqual(said, expectWall(...))`, which fails if the wall says the letter before reveal.
- Merge, web/home/home.js: `git diff cd4e373 d1db359 -- web/home/home.js` is exactly two lines: `role="region"` at :168 and `preventScroll` at :254. PR #38's `p("row_compiler")` / `p("row_edition")` / `p("row_target")` / `p("row_flags")` / `p("row_miri")` / `p("miri_seeds")` / `p("miri_separately")` / `p("nothing_yet")` calls are intact (:168-186, :216), the PROPOSED table holds copy-module keys, and no page literal re-entered. The copylint and copy-freeze suites pass in the 279/279 run.
- Merge, web/buzzer/buzzer.js: diff vs main is exactly the two preventScroll lines. PR #42's join-recovery code (finish/timer/AbortController) adds no focus site. The merge changed nothing in a11y.test.js, host.js, components.css or dom.js (`git diff 2205efb d1db359` empty for those); the web/test/a11y/browser.html delta is main's (PR #42).
- Guardrails: AC-40 holds, since the highlight keeps its fill and 3 px bar, and "Step N of M" is still in words. AC-33 holds: no layout-affecting change; role/opacity/count-only rewrite do not alter widths. Secrecy holds: nothing in the diff alters the data shown at any phase; the dim only changes legibility of source already on the wall in `work`, and the host count line is unchanged text. AC-84 holds for text, with the F-46 exception scoped to dimmed line numbers. One-source lint: product-code diff adds no participant-facing literal.
- Claims: "a11y 31/31" and "test-web 279/279" reproduced. README AC-84 figures (.70, 4.78:1, 3.57:1) and AC-82 text match the code and tests. The comment "PQ-40 showed it failing on the wall's lines" reproduced (ac86_wall and ac86_home fail on an added transition).

## Not verifiable by reading

- The c11-browser scroll measurement (scrollY 150 to 150 with preventScroll, 150 to 316 without) and the 390x504 iframe setup.
- How older engines (iOS Safari before 15) treat the `preventScroll` option, and whether a programmatically focused `#pq-code` on a phone still shows the refusal paragraph without scrolling.
- Whether .70 reads as a clear dim next to the highlight on the projector (the client's pick from screenshots); the 22.1 px wall font size (the suite measures CSS px, 13, which is conservative on the 4.5 threshold).
- `just test-room` and `just test-pipeline` (not run; the ticket reports test-pipeline cannot start locally, CI green on the earlier head); push/CI state of d1db359.
