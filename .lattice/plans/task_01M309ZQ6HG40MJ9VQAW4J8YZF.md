# PQ-8: The buzzer

BUILDPLAN.md T-06 (M1).

The buzzer: join by link or code, six failure states, capacity refusal, letters, saving/saved/failed, reconnect, the hint hidden in the live payload, the count on split/work/reveal, the released line

Criteria: AC-28–32, AC-34–38, AC-48, AC-58, AC-83, AC-85, AC-94
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-02, T-04a–c
BUILDPLAN notes: —

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-26)

Branch `ai-c11-cc/buzzer` off `origin/ai-c11-cc/wire-sessions` @ 2bdb0fd (PQ-32's wiring).

## Files

| File | New/changed | What |
|---|---|---|
| `web/buzzer/index.html` | new | A shell: `<main id="buzzer">`, a `<noscript>`, links `/shared/tokens.css`, `/shared/components.css`, `/join/buzzer.css`; scripts `/shared/dom.js`, `/shared/copy.js`, `/shared/check.js`, `/join/buzzer.js`. No participant string typed into it (every string comes from `copy.js`). |
| `web/buzzer/buzzer.css` | new | The prototype C buzzer panel (`.buzz-head`, `.buzz-grid`, `.buzz`, `.buzz-verdict`, `.buzz-foot`) ported to `web/shared/tokens.css` names, laid out at a 375 px design width, every control `min-height: var(--touch-target)` (44 px, AC-85). **Not ported:** `.buzz.mine-wrong` (red border on the participant's own choice — AC-94 forbids it; the prototype loses to the criterion). `prefers-reduced-motion`: no animation exists to disable. No horizontal scroll (the buzzer has no code; `overflow-wrap:anywhere` on text). |
| `web/buzzer/buzzer.js` | new | Classic script on `window.PopQuiz.Buzzer`. A **pure core** (state + reducers + `view(state)` → HTML string) and a thin **shell** (DOM, `fetch`, `WebSocket`, `sessionStorage`, `announce`) whose I/O is injected so the core runs in `node:vm` with no DOM. |
| `room/src/routes.rs` | changed, `// T-06 pages` block only, above `// T-04c routes` | `GET /join` (the HTML), `GET /join/buzzer.js`, `GET /join/buzzer.css` (`include_str!`, correct content types, `cache-control: no-cache`), and `GET /{code}` — the short link — `303` to `/join?code=<CODE>` when the segment is a well-formed room code, else `404`. |
| `web/test/buzzer.test.js` | new | The web half (below). |
| `web/test/buzzer_replay.js` | new (not `*.test.js`) | A replay driver: reads a JSON event log on stdin, applies it through `buzzer.js`'s core, prints the derived view after each event. Used by `room/tests/buzzer_page.rs`. |
| `room/tests/buzzer_page.rs` | new | The room half (below); a `/shared/{file}` helper for its own serving until PQ-7 is merged. |
| `room/README.md` | one line | under a *Pages* heading: `GET /join` the buzzer. |

## The client's state machine (buzzer.js core)

State: `{screen: join|room, code, roomId, token, frame, saved, submit: idle|saving|saved|failed, pending, conn: attached|paused, hintShown, refusal, lastAnnounced}`.

- **Join** (AC-28, AC-29, AC-30). Code from `?code=` (auto-submitted once) or typed into the one field (`room code` label, `join` button, *or open the link on the screen* beneath). `POST /join {code}`; `201` → keep `{room_id, token, code}` in `sessionStorage` (try/catch; memory fallback), attach. A refusal body `{refusal, reason}` → the message from `copy.js` by slug (`malformed`→`join_fail_malformed`, `unknown`, `not_yet_open`, `already_ended`, `closed_for_inactivity`→`join_fail_closed_inactivity`, `full`), the field kept for a retry. A reload with a stored token for the same code re-attaches instead of joining again (no ghost second session).
- **Attach / reconnect** (AC-37). `GET /rooms/{id}/ws/buzzer`, first message `{"t":"attach","token"}`. Until the first frame: `conn=paused`, letters disabled, the line *paused — reconnecting…* (with *your answer ‹X› is safe* when `saved`). The attach frame's `session.saved` becomes `saved`; later frames carry no `session` and never touch `saved`. On close: `4401` → token dead → back to the join screen (stored token dropped); `4404` → the join screen with the *already ended* message; `4000` (replaced by a newer tab) → stay paused, no reconnect (else two tabs ping-pong); anything else → paused, reconnect with capped backoff (0.5 s → 5 s).
- **Answer** (AC-34, AC-35, AC-36). Only while `frame.phase==='live' && conn==='attached'`. Tap → `submit=saving`, `pending=X`, `PUT /rooms/{id}/answer {letter}` with the bearer. One in flight at a time; a tap during flight replaces the queued letter (last write wins, sent when the flight lands). Outcome per the README table: `200 {saved}` → `saved`, `submit=saved`; `409 {phase, saved}` → `saved=resp.saved`, `submit=idle` (render from `phase`/`saved`, never from `reason`; the next frame moves the phase); `401` → join screen; other 4xx / 5xx / network → `submit=failed`, `saved` unchanged, *couldn't save. Your last answer, ‹X›, is safe.* + **Try again** (re-sends `pending`). With no answer yet: *tap a letter*. The submission line is **exactly one** of *tap a letter / saving… / saved — X / couldn't save…* whenever the phase is `live` (AC-35), and the pressed letter is `saved` (the pending one gets a distinct *saving* style, with the text line carrying the state, not colour).
- **Hint** (AC-48, D-8). `frame.hint.text` rendered only after the tap, from the frame already held; *Only you can see this. Nobody is told you looked.* No request. Reset when the phase leaves `live`.
- **Phases** (render from every frame; SPEC §4 buzzer column, §11):
  - `idle`: room code in the header, ↑, *You're in…*, foot *no account · no name · no score*.
  - `live`: five letter buttons (`aria-pressed`), submission line, hint.
  - `closed`: letters locked (disabled), *answers are closed* · *you said ‹X›* / *you didn't answer*.
  - `split` / `work` / `reveal` with an answer: *Look up.* + the phase line; the count `n = totals[saved]` with *people said ‹X›, including you.*; split adds *Five different readings…*, work *Nothing to do…*, reveal the ✓ line and *You and ‹n−1› other people…* / *…1 other person…* / *You were the only one…* + *The host is reading out the why now.*; foot *computed on this phone · never sent anywhere*. **Computed client-side from `counts.totals` and `saved`**; nothing is sent (AC-58).
  - the same three without an answer: *‹k› people didn't answer, you included.* (k = present − answered; k=1 → *You didn't answer.*) + the phase's line.
  - `reveal`'s ✓ line: `copy.buzzer_reveal_it_was` rendered through `check.correctHtml` (glyph + `rn-correct` colour atomically, AC-40), the copy's own leading ✓ taken off so it appears once. **Never** ✗, red, *wrong*, or any mark on the participant's own letter (AC-94).
  - `released`: ✓ glyph (not the correct-answer colour) and *Nothing about you was recorded.*
- **Live region** (AC-83): on each phase change the §11 string (`live_question_on_screen`, `live_answers_closed`, `live_split_on_screen`, `live_walking_through`, `live_revealed` with Y, `live_released`; `idle` has none in §11), and `live_saving` / `live_saved` / `live_save_failed` / `live_hint_shown` on those transitions, through `PopQuiz.announce`. A reconnect frame of the same phase does not re-announce.
- **Never source, never trace** (G-8, AC-32): the view reads only `phase, code, counts, correct, hint, session`; nothing in `buzzer.js` reads `source`, `trace`, `options`.

## Tests by criterion

`web/test/buzzer.test.js` (`just test-web`), loading `dom.js, copy.js, check.js, buzzer/buzzer.js` via `node:vm` (extending `_load`'s approach locally, not editing `_load.js`'s module table — it is T-02's; if a one-line group is cleaner I ask first):
- AC-28: the join screen has one `<input>` and one button; `?code=` pre-fills and submits; typed code submits.
- AC-29/AC-30: each of the six refusal slugs renders its own §11 string, distinct from the other five, with the field still there.
- AC-35: property-style walk over every response class (200, 409 closed, 401, 400, 500, network) and every interleaving of tap/response/frame while `live` — the rendered view carries exactly one of the four submission strings.
- AC-36: after a failure the line names the last saved letter and a *Try again* control that re-sends; `saved` unchanged.
- AC-34: five taps then close — the last `200` wins; after close a tap sends nothing.
- AC-37: close → paused line, letters disabled; attach frame with `session.saved` → saved restored, paused gone; 4401 → join screen; 4000 → no reconnect.
- AC-48: tapping *Show me a hint* shows `frame.hint.text` with **zero** calls on the injected `fetch`/`send`.
- AC-58: split/work/reveal count line from totals + saved (fixture payloads shaped like `view::buzzer`), with the stub transport recording that no request carries the letter after close.
- AC-83: the announcements per transition, verbatim, once each.
- AC-85: `buzzer.css` gives every `button`/`input` `min-height` ≥ 44 px (via `--touch-target`, asserted to be 44px in `tokens.css`).
- AC-94 (+AC-40): at `reveal` with `saved ≠ correct` the HTML contains no ✗, no `wrong`/`incorrect`, no `--error`/`--red`/`red` class, and the participant's letter has no mark; the ✓ line carries glyph and class together; grep `buzzer.{js,css,html}` for ✗ and red tokens.
- G-8/AC-32: `buzzer.js` source contains no `.source`/`.trace`/`.options` read; render of a frame with a planted `source`/`trace` shows neither.

`room/tests/buzzer_page.rs` (`just test-room`):
- `GET /join` 200 `text/html`, references the shared and buzzer assets; `/join/buzzer.js` `text/javascript`, `/join/buzzer.css` `text/css`.
- AC-28 by link: `GET /<code>` → 303 `Location: /join?code=<code>`; a non-code segment → 404; the room's `join_url` path redirects to the page.
- Through the wiring: create (TestAuth) → join by code → attach with the token → live → answer → close → drop → split → re-attach with the same token; the event log (join response, frames, PUT responses, close) piped through `node web/test/buzzer_replay.js`, whose derived view after each step matches the server (phase, saved letter, paused/attached, count = server totals). Node is required by `just test` already (`test-web`); a missing `node` fails the test, never skips it.
- G-8/AC-32 (the canary's idea): the page HTML, the two assets and every buzzer frame and HTTP body collected in that run contain neither q3's source text nor any trace step `note`, and no key named `source`/`trace`.
- An `#[ignore]`d `serve_for_browser` test: the router + TestAuth + a `/shared` helper on a loopback port, printing the join and host URLs — used for validation in the c11 browser, never in `test`.

## Choices made

1. **`/{code}` short-link redirect.** `rooms.rs` writes `join_url = {base}/{code}` (not `?code=`), and the wall prints it; the Orchestrator's split says the link carries the code as `?code=` on `/join`. Both hold if `/{code}` redirects to `/join?code=`. Registered in my block, only for a well-formed code (else 404), so it cannot shadow PQ-7/PQ-9's static paths (`/host`, `/wall/{id}`, `/shared/{f}` — static and multi-segment routes win in axum 0.8 anyway). Flagged in the completion comment; an alternative is `rooms.rs` changing `join_url`, which is not mine.
2. **Assets under `/join/…`** (`/join/buzzer.js`, `/join/buzzer.css`): the page's own path, so no collision with `/rooms/{id}/buzzer` or siblings.
3. **Strings rendered client-side from `copy.js`**, not from `payload.lines`: the phone must vary the lines by its own answer (AC-58), which the server cannot know; the phase and counts come from the payload. `payload.lines` stays as-is (not mine).
4. **Token in `sessionStorage`**, keyed to the room code: survives a reload in the tab, dies with the tab; nothing per-person persists (AC-57).
5. **`4000` replaced → no reconnect**, to avoid two tabs fighting.

## Contract tensions

- Prototype C's `.buzz.mine-wrong` red border and *It was E.* without ✓ lose to AC-94 and SPEC §11 (*✓ It was ‹Y›.*).
- Prototype C's live foot *tap to change it…* and closed *locked in* are not §11 strings; §11's foot (*no account · no name · no score*) is used.
- `join_url` shape vs `?code=` — choice 1.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: Sonnet subagent, 2026-09-26. Six findings.

1. **Critical — `web/test/buzzer_replay.js` is out of scope.** *Resolution:* no separate file. The replay driver lives in `web/test/buzzer.test.js` behind `BUZZER_REPLAY=1` (a top-level `return` before any test registers); `room/tests/buzzer_page.rs` spawns `node web/test/buzzer.test.js` with that env. Done.
2. **Major — `.btn`, `.btn-primary`, `.panel`, `.meta` are not in `web/shared`.** *Resolution:* `buzzer.css` defines its own, scoped under `.buzzer`, ported from `prototypes/_shared/tokens.css` onto the shared token names, `.btn` at `min-height: var(--touch-target)`. The AC-85 test checks `.buzzer .btn`. Done.
3. **Major — Direction C has no join screen.** *Resolution:* added as a contract tension. The join form is built from §11's three strings (`buzzer_join_label/button/beneath`) and the six refusals, in the buzzer's own type and tokens: one centred field and one full-width primary button, with the line beneath. B-phone-first's form was not copied; it is not the binding direction. Recorded as a deviation in the completion comment.
4. **Minor — `<noscript>` would need a typed string.** *Resolution:* no `<noscript>`. The shell types no string, and a test asserts that.
5. **Minor — `4404` covers both release and inactivity expiry.** *Resolution:* the close carries no reason, and after release the socket normally sees `4401` first. So `4404` maps to *already ended*, the plainer of the two, and this is noted as a simplification. When T-11 lands the inactivity close, it can give it its own close code if it wants the other sentence.
6. **Minor — there was no *Pages* heading in the README.** *Resolution:* the boot prompt says "one line under *Pages*", so the heading plus its one line is the intended change. Kept.

## Reset 2026-09-26 by agent:delegator-pq8
