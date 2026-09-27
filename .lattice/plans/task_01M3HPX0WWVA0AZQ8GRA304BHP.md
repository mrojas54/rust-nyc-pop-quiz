# PQ-34: HC-0 copy trim: strip narration from the buzzer and wall

HC-0 finding, 2026-09-27, the client's ruling during the first deployed drive: "remove any text that is absolutely unnecessary. no need to narrate the demo. intuitive workflow rather than guided." Named instance: the buzzer's reveal line *The host is reading out the why now.* (copy.js buzzer_reveal_host_reading and buzzer_noanswer_reveal; SPEC §11 buzzer-reveal rows; §5 phase table; prototype line 776). Scope: (1) remove that sentence everywhere it appears; (2) sweep every buzzer, wall and host string in web/shared/copy.js for narration of what someone else is doing or what will happen next, and propose the list of removals in the plan for the client's yes before touching them; keep every line that carries state the reader needs (the count, ✓ It was Y, You didn't answer, the room code, the join address); (3) amend SPEC §11 and the §5 phase table and the prototype in the same PR under the client's ruling, so the copy contract and the build stay one thing; (4) update the string tests. Colour is never the only signal; never let a removed line take a state signal with it. PR against main, fast-track, one Sonnet code review. Dispatch waits until the client's HC-0 drive ends so further findings can join this ticket.

# Plan (delegator, 2026-09-27)

Base: origin/main @ 66a962c. Branch ai-c11-cc/copy-trim. Fast-track.

## Findings folded in
1. HC-0 #1 — buzzer reveal narration *The host is reading out the why now.*
2. HC-0 #2 — buzzer footer *no account · no name · no score*.
3. HC-0 #3 — host page: title "Pop Quiz Host"; the two not-a-guarantee sentences rewritten for a human; phase label kept if it reads as state.

## Removal / rewrite list (for the client's yes in the tab; lines not approved stay)

Ruled (no approval needed):
- R1 `buzzer_reveal_host_reading` "The host is reading out the why now." — narration of the host.
- R2 `buzzer_noanswer_reveal` "✓ It was ‹Y›. The host is reading out the why now." — key removed; no-answer reveal renders `buzzer_reveal_it_was` ("✓ It was ‹Y›."), glyph + colour kept (AC-40).
- R3 `buzzer_foot` "no account · no name · no score" — footer gone from join, idle, live, closed, released, paused screens. The promise stays in behaviour (AC-28/AC-57); PR body notes the amendment.

Proposed sweep:
- S1 `buzzer_idle` "You're in. Everything happens on the screen at the front — look up." → shorten to "You're in." (state kept; the rest narrates).
- S2 `buzzer_split_where` / S3 `buzzer_noanswer_split` "Where the room landed." — labels what the count already says.
- S4 `buzzer_split_readings` "Five different readings. Nobody knows what anyone picked." — commentary.
- S5 `buzzer_work_walking` / S6 `buzzer_noanswer_work` "We're walking it through." — narrates the host.
- S7 `buzzer_work_nothing` "Nothing to do. Nobody knows the answer yet." — reassurance / what next.
- S8 `buzzer_reveal_on_screen` "The answer is on the screen." — the phone shows ✓ It was Y itself.
- S9 `buzzer_foot_computed` "computed on this phone · never sent anywhere" — same kind as R3; AC-58 stays proven by the canary, not by the sentence.
- S10 `buzzer_reveal_company`, `buzzer_reveal_company_one`, `buzzer_reveal_only_one` — repeat the count directly above them ("‹n› people said X, including you."). Borderline.
- S11 `buzzer_join_beneath` "or open the link on the screen" — guidance. Borderline.

Rewrite (finding 3):
- W1 `not_a_guarantee_options_public` → "The answer is always one of the five options on the screen."
- W2 `not_a_guarantee_host_honest` → "You aren't shown the answer. That keeps you honest. It isn't a security guarantee. Anyone who reads Rust can work it out from the code."
  Both facts kept (AC-62: options public, answer among them; AC-63: expert host can infer, not a security guarantee). Trope check (§11.1) clean. SPEC §8.1 paragraph that quotes them verbatim updated.
- T1 host `<title>` "Host · Rust NYC Pop Quiz" → "Pop Quiz Host". Open: also a visible "Pop Quiz Host" heading above the phase label (client decides; it is a new §11 row).

Kept, with reason:
- "Look up." (buzzer split/work/reveal heads) — the phone's one pointer to where the content is; the client's screenshot did not rule it.
- `buzzer_released` "Nothing about you was recorded." — AC-59's visible form (felt, HC-1).
- `buzzer_hint_shown` — PHILOSOPHY §9 "asking for help costs nothing"; a newcomer needs to be told nobody sees it.
- Wall work beat panel "Let's walk it. / Still no answer. / Nobody has to say anything." — DESIGN.md voice names it, AC-98's release line; DESIGN is not editable.
- Wall released line "the question, the walk-through and the why — at your own pace." — what's behind the link, SPEC §5.5 / T-20 item 13.
- Wall "still open", "Pause here.", receipt, counts; host phase labels (AC-49, incl. "before the question" as state), actions, fit line, resume line (AC-50), read-aloud; live region (AC-83 verbatim, screen reader only); refusals, submission, reconnecting, closed lines.

## Files
- web/shared/copy.js (COPY, COPY_ROWS), web/buzzer/buzzer.js (render; foot element omitted where no foot; noanswer reveal via itWas), web/buzzer/buzzer.css only if a removed element leaves a gap, web/host/index.html (title), web/host/host.js only if a visible title is approved.
- web/test/copy.test.js, buzzer.test.js, host.test.js (pins).
- SPEC.md §5 table cells (idle/split/work/reveal buzzer), §8.1 paragraph (W1/W2 verbatim quote), §11 rows.
- prototypes/C-projector-first.html buzzer strings (lines ~740–780).
- BLOCKED on Orchestrator clearance: room/src/copy.rs (mirror consts + ALL; twins.rs enforces), room/src/view.rs (BuzzerPayload.lines/foot carry removed consts). Asked by comment.
- room/tests/*: any assertion on removed strings.

## Tests by criterion
AC-40 (✓ glyph + colour on it_was in both reveal paths) — buzzer.test.js; AC-94 (no ✗) unchanged; AC-49 (host phase label present) host.test.js; AC-62/63 (two facts present) copy.test.js; AC-58/59 unchanged; copy completeness test both directions; twins mirror test; fallback rebuilt and diffed (PQ-12 method); rg exit check per removed string.

## Contract tensions
- room/src/** mirror vs. scope → asked Orchestrator; will not touch without clearance.
- DESIGN.md row 5 names the wall's work lines; kept rather than edit DESIGN.

## Client ruling (tab, 2026-09-27) — supersedes the list above
"everything on the keep list, except error saving, reconnecting and screen reader, remove host labels and counts unless on projector" / "remove before the question, replace evertything in that section on this and subsequent pages with Pop Quiz Host".
Final set: all R/S lines; plus Look up (x4), buzzer_released, buzzer_hint_shown, buzzer own count + said line, noanswer count (k>1 form); wall work beat lines, wall released line; host heading → new key host_title "Pop Quiz Host" on every host screen and <title>; host counts + fit line off the host phone; host_first_resume + both not_a_guarantee off. Phase labels survive only as live-region announcements. Orchestrator cleared room/src/copy.rs + view.rs, strings only; foot → Option<&'static str>.

## Reset 2026-09-27 by agent:delegator-pq34

## Reset 2026-09-27 by agent:delegator-pq34

## Reset 2026-09-27 by agent:delegator-pq34

## Reset 2026-09-27 by agent:delegator-pq34
