# PQ-9: The host phone

BUILDPLAN.md T-07 (M1).

The host phone: one screen per phase, one primary action, counts, trace stepping, the read-aloud script with the beats' *human* provenance marker (AC-74), resume on refresh/device change; in M1 *Create a room* presents the stand-in host token carried in the page URL (`SPEC.md` §8.2)

Criteria: AC-45–47, AC-49, AC-50, AC-74
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-02, T-04a–c
BUILDPLAN notes: —

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-26)

Worktree `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/host-phone`, branch `ai-c11-cc/host-phone`, base `origin/main` @ `cac818b`.

## What exists and what the page consumes

- `GET /rooms/{id}/host` (host bearer) → `view::HostPayload`: `phase`, `code`, `label`, `primary {action,label}`, `present`, `answered?`, `fit?`, `first_screen? {resume, not_a_guarantee[2]}` (idle only), `step? {at,m,label,note,back,forward,can_back,can_forward}` (work, reveal), `read_aloud? {heading, what, middle, takeaway}` (reveal only).
- `POST /rooms/{id}/<slug>` for `put-on-screen close-answers show-split walk-it reveal release step-back step-forward` (host bearer, no body) → the host payload. `POST /rooms/{id}/run-it-again {question_id}` (create bearer) → `201 Created`. `POST /rooms {question_id}` (create bearer) → `201 {id, code, join_url, host_session, host_resume_url}`.
- `GET /rooms/{id}/ws/host`, attach `{"t":"attach","token":<host session>}`, then whole-state frames `{"t":"state","revision",…host payload…}`; close codes 4401/4404/4408.
- `host_resume_url` = `{base}/host/{id}#{host_session}` (rooms.rs) — the resume link **is** the page URL; the session rides in the fragment and never rotates (AC-50).

## Files

| File | What |
|---|---|
| `web/host/index.html` (new) | The one page for both `/host` and `/host/{room_id}`. Loads `/shared/tokens.css`, `/shared/fonts.css`, `/shared/components.css`, `/host/host.css`, `/shared/dom.js`, `/shared/phase.js`, `/shared/copy.js`, `/host/host.js`. A `<main id="host">` the script renders into; a `noscript` line is not added (no §11 string). |
| `web/host/host.css` (new) | The prototype's host-phone visual detail ported: `.phone-body` column at a phone width (max 480 px = `--container-max`), `.host-step`, `.host-h` (Instrument Serif 24 px), `.stack`, `.panel`, `.btn`, `.btn-primary` (44 px min target), the code panel (serif 34 px amber), `.ratio` counts footer, `.beat`/`.beat-company`, `.trace-nav`. Values from `prototypes/C-projector-first.html` + `prototypes/_shared/tokens.css`. `overflow-wrap:anywhere` on the resume link so nothing scrolls sideways (AC-33). |
| `web/host/host.js` (new) | Classic script on `window.PopQuiz` like `web/shared`. Pure parts exported for tests: `PQ.host.parseLocation(loc)` → `{mode:"create", token, question}` or `{mode:"room", roomId, session}`; `PQ.host.render(payload, ui)` → HTML string; `PQ.host.actionLabel(slug)`; `PQ.host.ACTIONS` (slug → copy key). Impure part `PQ.host.boot(window)`: create flow, socket with reconnect, POSTs. |
| `web/test/host.test.js` (new) | Node suite, loads `web/shared` via `_load.js` then evaluates `web/host/host.js` into the same vm sandbox. |
| `room/src/routes.rs` | Only a `// T-07 pages` block above `// T-04c routes`: `GET /host`, `GET /host/{room_id}` → `index.html`; `GET /host/host.js`, `GET /host/host.css` — all `include_str!("../../web/host/…")`, correct `content-type`, `cache-control: no-store` and `referrer-policy: no-referrer` on the HTML. |
| `room/tests/host_page.rs` (new) | The room-side proofs below; serves `/shared/*` in the test only through a local helper router merged onto `room::router_with(...)` (PQ-7 owns the real route). |
| `room/README.md` | `## Pages` section header (PQ-7's words) if absent, plus one line for `/host` and `/host/{room_id}`. |

## Behaviour

1. **`/host` — the first screen, before a room exists.** Reads the stand-in credential from the URL fragment (`/host#<HOST_DEV_TOKEN>`, §8.2) and the question id from `?question=<id>`. Shows the idle phase label *before the question*, the one primary action *Create a room*, and §8.1's two sentences. *Create a room* → `POST /rooms {question_id}` with `Authorization: Bearer <token>`. On `201` the page stores `{token, question}` in `sessionStorage` (for *Run it again*, this tab only), then `location.replace(host_resume_url)` → the room screen. The dev token leaves the address bar at that moment. On failure: the server's `reason` if it sent one, else the status line (`401 Unauthorized`) — no new prose authored.
2. **`/host/{room_id}#<session>` — one screen per phase.** Reads the session from the fragment; missing fragment → the refused state (status line). Paints from `GET /rooms/{id}/host`, then attaches `/rooms/{id}/ws/host` and re-renders on every frame (counts move live, AC-46). On a dropped socket shows §11's *paused — reconnecting…* (`buzzer_reconnecting`) and reconnects with backoff (0.5 s → 8 s); 4401/4404 stop and show the status. Every screen: phase label from `PQ.HOST_PHASE_LABEL[phase]` (AC-49), the room code in every phase but `released` (joins are refused there, AC-49 "where returning is possible"), counts, the fit line from `live` on when the payload carries it, and **exactly one** primary button whose label is `copy.js`'s for `payload.primary.action` (never the server's string retyped; a test asserts the two agree).
   - `idle`: code, present count, *Put it on the screen*, the resume line (`host_first_resume` filled with the room's resume link = `location.href`, rendered as a link) and §8.1's two sentences (from `copy.js`).
   - `live`/`closed`/`split`: counts; the primary.
   - `work`/`reveal`: `Step N of M` + dots + the step's words (`step.note`), `←` `→` buttons disabled per `can_back`/`can_forward` (secondary controls, not phase transitions), then the primary.
   - `reveal`: *Read it aloud* panel — `.panel.by-human` with a `.provenance.human` marker (✎ glyph; dashed orange rule) around the three beats `what` / `middle` (heading *Why ‹n› of us said ‹X›* or *Why nobody said anything else*, text only when present) / `takeaway` (AC-74).
   - `released`: *Run it again* → `POST /rooms/{id}/run-it-again {question_id}` with the stored create token; `201` → navigate to the new room's `host_resume_url`; refusal → the server's reason. With no stored credential (a second device) → navigate to `/host` (the create screen), which needs the URL T-09 printed.
3. **No answer preview (AC-47).** The page fetches only the host projection and the host socket; it never requests `/wall` or `/buzzer`. Before reveal the host payload has no `correct`, no `read_aloud`, no resolving step (the room guarantees it; the page renders only what it is handed).
4. **Resume (AC-50).** Refresh keeps the fragment → same session. Opening the resume link on a second device attaches it with the same session; hosts are not keyed in `ws.rs`, so both control the room.
5. **Announce** each phase change through `PQ.announce` with the phase label (polite live region, AC-83 spirit); keyboard: native buttons, visible focus from the design system.

## Tests (by criterion)

- `web/test/host.test.js`:
  - AC-49/AC-45: for each of the seven phases render a fixture payload (hand-built JSON shapes, no program output) → exactly one `data-primary` button, its label = `copy.js` for that phase's next action, the phase label present; the code present in every phase but `released`.
  - AC-45: the union of primaries over the seven phases plus *Create a room* (the `/host` screen) equals exactly the eight §11 host actions; `←`/`→` render only in `work`/`reveal`; no other `<button>` anywhere.
  - AC-47: every pre-reveal render contains no `✓`, no `read-aloud`, no `data-provenance`; `host.js` source names no `/wall`, `/buzzer` or `/answer` URL.
  - AC-74: the reveal render wraps the beats in `.by-human` with `.provenance.human`; the middle beat's two variants.
  - `parseLocation` for `/host?question=q3#tok`, `/host/abc#sess`, missing fragment.
  - AC-46: `render` with changed `present`/`answered` changes the counts text (the socket wiring is proven room-side).
- `room/tests/host_page.rs`:
  - The pages serve: `GET /host`, `/host/{id}`, `/host/host.js`, `/host/host.css` → 200 with the right content types; the HTML references only `/shared/*` and `/host/*` assets, and every referenced asset serves (through the test-only `/shared` helper).
  - AC-47 (the canary's idea): on `planted()` drive a room through every phase as the host, fetching the page and `GET /rooms/{id}/host` in each; before `reveal` no payload and no page carries any plant, a `correct` key, `read_aloud`, or a `stdout` row; at `reveal` the payload does carry `read_aloud` (so the scan is not vacuous).
  - AC-45: each phase's host payload `primary.action` is the one route that succeeds; every other host action route is refused (409) in that phase.
  - AC-50: two host sockets attached with the same session (the resume link's fragment) both receive frames; an action through either device moves both; `host_session` in the resume link is unchanged from creation to release; `GET` after "refresh" (a new request with the same fragment session) still authorizes.
  - AC-46: a join moves `present` on the host socket; an answer moves `answered` while live.
- `just test-room` and `just test-web` (the pipeline suite cannot start on this machine — client env); warm time recorded.

## Open choices made

1. **The question id rides in `?question=`** on `/host`: §8.2 only fixes the token in the fragment; `POST /rooms` needs a question id and no route lists scheduled ones. T-09 prints `/host?question=<id>#<token>`. Flagged to the Orchestrator.
2. **Resume is the URL fragment, not a cookie.** `rooms.rs` builds the resume link with the session in the fragment and `ws.rs` keeps the credential out of URLs and logs for that reason; a refresh keeps the fragment. EVALUATION AC-50 says "cookie survives" — the fragment is the mechanism the merged code chose; the proof is the same (refresh → same session).
3. **The create screen shows the idle label** (*before the question*): AC-49 wants a phase label on every host screen and there is no label for "no room yet".
4. **Counts use §11's `‹answered› of ‹present› in the room answered`** in every phase (no host-specific count string exists). In `idle` the payload carries no `answered`; nobody can have answered yet, so it renders `0 of ‹present›…`, which is true. The code panel's caption reuses §11's `room code`.
5. **The human provenance marker is the ✎ glyph + dashed rule** (components.css `.by-human`, `.provenance.human`): §11 authors no marker label, and the prototype's sentence is not in §11.
6. **Errors show the server's `reason` or the HTTP status line** — no authored prose outside §11.
7. **Reconnecting** reuses §11's `paused — reconnecting…`.
8. **`/host/host.js` and `/host/host.css`** share the `/host/` prefix with `/host/{room_id}`; axum's router prefers the static segment. A room id is 32 hex chars, so it can never equal `host.js`.

## Contract tensions

- EVALUATION AC-50 "cookie" vs rooms.rs fragment — sided with the merged code (above).
- §8.2 names only the token's carrier; the question id's carrier is a new URL contract for T-09 (above).

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: fresh-eyes subagent (Sonnet), 2026-09-26. No Critical findings; it confirmed empirically that axum 0.8.9 / matchit 0.8.4 routes `/host/host.js` ahead of `/host/{room_id}`.

1. **Major: the resume link ignored the server's `first_screen.resume`.** Resolution: the idle screen renders `payload.first_screen.resume` verbatim (the server's §11 fill of `host_resume_url`), never a client-derived `location.href`. The URL inside it is made a link by stripping the `copy.js` prefix of `host_first_resume`; a web test asserts the server string equals `fill(host_first_resume, {resume link: url})` for the fixture, and a room test asserts the served payload's resume link is `{base}/host/{id}#{host_session}` — i.e. this page's own URL.
2. **Major: reconnect backoff unproven.** Resolution: backoff is a pure exported function `PQ.host.backoff(attempt)` (500 ms doubling, capped at 8000 ms), unit-tested; `boot` calls it. Terminal close codes (4401, 4404) are a pure `PQ.host.isTerminalClose(code)`, also tested.
3. **Major: `?question=` is a contract T-09 has not built.** Resolution: kept, and raised to the Orchestrator by `lattice comment` now, before implementing, so T-09 prints `/host?question=<id>#<token>`. `parseLocation` is tested for the shape.
4. **Minor: reusing `buzzer_reconnecting` on the host phone is a cross-row borrow.** Resolution: acknowledged; listed as a deviation beside the counts-line borrow in the DONE comment.
5. **Minor: the page-level plant scan is mostly vacuous.** Resolution: kept as a cheap guard against anyone inlining question data into the page later, but described honestly; the page test's real content is the served routes, content types, asset references, AC-45's refusals per phase, AC-46's live counts and AC-50's two devices. The payload scan's host half overlaps `canary.rs` by design (the ticket asks for it).
6. **Minor: the web-test load mechanism.** Resolution: `host.test.js` calls `_load.js`'s `load('all', {timers:false})`, takes the returned `sandbox`, and `vm.runInContext(fs.readFileSync('web/host/host.js'), sandbox)`; `_load.js` is not edited.

## Reset 2026-09-26 by agent:delegator-pq9
