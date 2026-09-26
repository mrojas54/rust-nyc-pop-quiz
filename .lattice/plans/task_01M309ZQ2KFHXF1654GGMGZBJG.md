# PQ-7: The wall

BUILDPLAN.md T-05 (M1).

The wall: seven phases on the 1120×630 canvas, full-width source with options beneath, type model, split bars, reveal with **the receipt rendered by the one step-list function** (§7.5), released with link + QR, no interaction

Criteria: AC-33, AC-39, AC-40, AC-74, AC-78, AC-79, AC-99, AC-100, G-7
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-02, T-04a–c
BUILDPLAN notes: —

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-26)

Base: origin/main @ cac818b. Branch ai-c11-cc/wall. Worktree /Users/michellerojas/rust-nyc-pop-quiz-worktrees/wall.

## What the wall renders, phase by phase (from `view::wall`, which is the payload)

The page is a pure function of one wall frame. `phase` decides the view; nothing else is cached between frames (a reload shows the same thing, the frame is the whole state).

| Phase | Canvas (1120×630, scaled as a unit, §5.1) |
|---|---|
| idle | brand line top-left; title card `title` (*Time for a pop quiz.*) at 62px serif; join strip `join` with the link portion in 30px serif amber (prototype `.joinstrip b`). No source (AC-99). |
| live | well (colour on, header `well_header`), full width; options block beneath: two-column grid, five `.opt` chips (letter chip 40px + one line text at 24px, D-15), fixed 190px; strip `join` (*join @ ‹link› · still open*). |
| closed | same well + options; strip `strip` (*answers are closed*) and nothing else. |
| split | same well (colour on); the block beneath shows the five bars (`split.bars`, `n · p%`, `display:block` fills, no mark) instead of the options — prototype split; strip `split.line` (*‹answered› of ‹present› in the room answered*). |
| work | well, **no colour**, highlight-and-dim from `trace.step.lines/focus`; fixed 203px block beneath in two columns: left the trace note (`Step N of M` + *Pause here.* on a pivot, the note clamped to three lines, the values table, dots — `traceNoteHtml(..., {noNav:true})`, no buttons), right the beat panel (`beats`: **Let's walk it.** / Still no answer. / Nobody has to say anything.). No ✓, no receipt, no bars (the payload has no split in work). |
| reveal | well, no colour, trace entering at M-1 and steppable (host-driven: the frame's `trace.at`); 203px block in three columns: trace note/values/dots; the bars with the correct bar marked (✓ glyph + green border, via `applyCorrect`/`correctHtml`, never colour alone, AC-40, §5.4) and the middle line (`reveal.middle.line`: *‹n› of us said ‹X›* or *Nobody read it another way.*) beside the named option's bar; the correct option chip (letter ✓ + option text) and the receipt: heading `reveal.receipt.heading` under the machine provenance marker (`.provenance.machine`, `.by-machine`, AC-74), then each of `reveal.receipt.lines` verbatim, one per line. **No explanation text** (AC-39). Strip `split.line`. |
| released | title card: *Let's go to the bar.* serif; the link at 40px mono in the `.endlink` box; `released.line`; a real QR of `released.link` (120px, `<svg>` from a vendored encoder). No join strip, nothing else. |

Brand line top-left on every phase (SPEC §4, §5.1). The prototype's top-right phase labels (*read it*, *the answer*…) are **not** in SPEC §11, so they are not rendered (copy rule).

Human-provenance marker (AC-74): the wall renders no human-reviewed prose (AC-39 keeps the explanation off the wall; the step note is the only prose, and it is the trace, not the explanation). The wall therefore carries the machine marker on the receipt and never the human one; AC-74's "distinct provenance markup on wall, host phone, take-it-home" is satisfied on the wall by the machine marker being present on machine fact and absent from everything else. The boot prompt's "the beats with the human-provenance marker" I read as the host phone's beats (PQ-9), since the wall's `beats` are the three fixed §11 lines, not authored prose — see deviation D3.

## Files

New:
- `web/wall/index.html` — the page shell: `<link>`s to `/shared/tokens.css`, `/shared/components.css`, `/wall/wall.css`; `<script src>` for `/shared/{dom,phase,check,well,trace,typemodel,copy}.js`, `/wall/qr.js`, `/wall/wall.js`. No inline data, no room state, no strings beyond `<title>`. The page is identical in every phase, so it cannot carry a pre-reveal answer.
- `web/wall/wall.css` — the projector frame and the wall's own classes ported from `prototypes/C-projector-first.html` (`.proj-*`, `.opt`, `.bars`, `.bar-*`, `.joinstrip`, `.title-card`, `.endcard/.endlink/.endsub/.endqr`, `.workbeat`, `.receipt`) and `prototypes/_shared/tokens.css` §5–6 pieces not in components.css (`.receipt`, `.projector`). Prototype `--alias` names mapped to §1 names. `data-room="dim"` not ported (not a product mode). PROTOTYPE badge removed (§5.1).
- `web/wall/wall.js` — classic script on `window.PopQuiz.Wall`:
  - `html(frame, {fontPx, fit})` → the canvas inner HTML. **Pure**: no DOM, no socket, no clock. T-26's `mode: "static"` drives the same function from a local state object (README says so).
  - `layoutFor(phase)` → reading | trace | none (via `PQ.codeAreaFor`).
  - `mount(root, {connect})` — browser only: owns the scale-to-viewport (`transform: scale`, never width:100%), renders each frame, then on every phase that renders source runs `derivedFontPx` → render → `measureWell` → `refit` (≤6 passes, never below the floor) → `fitVerdict` → `applyClippedEdge` (red edge on the side that lost content). Refit is re-run after `document.fonts.ready`. Announces phase changes via `PQ.announce` with the §11 live-region strings where one exists.
  - `connect(roomId, onFrame)` — `new WebSocket(ws(s)://host/rooms/{id}/ws/wall)`, no attach message (the wall needs none), reconnect with backoff; each frame replaces the whole state.
  - Room configuration: `PQ.ROOM_DEFAULTS` (15 / 8.44 / 20) — the payload carries no room measurements yet; noted.
- `web/wall/qr.js` — a small QR encoder (byte mode, ECC level M, versions 1–10, all eight masks with the standard penalty score), written for this page and licensed with the repo; renders an `<svg>` of `<rect>`s. No CDN (AC-77). Proven in validation by decoding its output with macOS CoreImage (`CIDetector` QR) in the c11 browser's rendering, and in `test` by a golden matrix that the decoder cross-checked.
- `web/wall/fixtures/q3-phases.json` — q3 wall payloads for every phase (and every work/reveal trace step, and the nobody-else variant), **produced by `room/tests/wall_page.rs` from the real `view::wall`**, never typed. The Rust test asserts the committed file equals what the room serves (regenerate with `UPDATE_WALL_FIXTURES=1`), so the web suite runs on real payloads and the two cannot drift.
- `web/test/wall.test.js` — loads `web/shared/*.js` + `web/wall/{qr,wall}.js` into a node:vm context (reusing `_load.js`'s `load('all')` sandbox) and asserts on `Wall.html(frame)`:
  - every phase → its structure (the table above) on the q3 fixture payloads;
  - AC-99: `tk-` spans present in live/closed/split, **zero** in work/reveal, no `.rn-src` in idle/released;
  - AC-40: the correct bar and the correct option carry the ✓ glyph and the correct class together; nothing outside reveal carries ✓; no ✗ anywhere;
  - AC-79: no `<button`, `<input`, `<select`, `<textarea`, `<a `, `onclick`, `tabindex` on anything but the well's scroll region (which `well.js` makes focusable — AC-82's keyboard scroll, not a control) in any phase; the page `index.html` likewise;
  - AC-39: reveal HTML contains no `explains` text (from q3.json), carries ✓, totals, the middle line, the receipt;
  - AC-74: the receipt sits under `.provenance.machine`; no `.provenance.human` on the wall;
  - G-7: for every `bank/fixtures/receipts/*.json` with `expected_lines`, a reveal frame with `reveal.receipt.lines = expected_lines` renders exactly those lines, in order, and nothing else under the heading (the lines come from the one Rust function; the wall authors none);
  - work: step index shown ≤ M-2 (`Step 5 of 6` max for q3), no `stdout` in values, no ✓/receipt; reveal enters at `Step 6 of 6`;
  - copy: every visible string in every phase's HTML is either payload data or a `PQ.COPY` value (checked by stripping tags and matching against the union) — the brand line is the one listed exception (deviation D2), which is linted against the Forbidden and trope patterns;
  - AC-33: the canvas CSS has `overflow:hidden` on the page and the well; the well is never `overflow-x: auto` on the wall (clips and reports instead);
  - QR: golden matrix for a fixed URL; finder/timing patterns; format bits BCH-valid.
- `room/tests/wall_page.rs` — in-process (`router_with`, `tower::ServiceExt`, no socket): `GET /wall/{id}` is 200 `text/html` and contains the shared asset URLs; `GET /wall/{unknown}` is 404; every `/shared/*` file and `/shared/fonts/*` serves with the right content type (css/js/ttf) and the exact bytes on disk; unknown asset 404; `/wall/wall.js`, `/wall/wall.css`, `/wall/qr.js` serve. Drives a planted room (`common::planted()`) through every phase with `FakeSessions`, fetching the page **and** the wall payload each phase: the page never contains `stdout`, ✓, a plant, or a receipt line, before reveal or ever (it is static); the wall payload before reveal has no `values` entry named `stdout` (`common::stdout_rows == 0`). Also the fixture golden above. Uses `tests/common` read-only.

Changed:
- `room/src/routes.rs` — a `// T-05 pages` block **above** `// T-04c routes`: `GET /wall/{room_id}` (404 via `state.with_room` when missing, else `include_str!("../../web/wall/index.html")`, `text/html; charset=utf-8`, `cache-control: no-store`); `GET /wall/wall.js|wall.css|qr.js`; `GET /shared/{file}` (a `match` over the fixed file list, `include_str!`, `text/css` / `text/javascript` / `text/markdown`-free: only css+js); `GET /shared/fonts/{file}` (`include_bytes!`, `font/ttf`). No new crate, no ServeDir. Static `/wall/wall.js` beside param `/wall/{room_id}` is fine in matchit 0.8 (static wins); room ids are 32 hex chars so no clash.
- `room/README.md` — *Pages* section: `/wall/{room_id}`, `/host/{room_id}` + `/host`, `/join`; `/shared/{file}` and `/shared/fonts/{file}`; that the wall's rendering is a pure function of a frame so T-26's static mode can drive it.
- `justfile` — only if the measured-layout run gets a recipe: `test-wall-layout` under `test-full` is **not** added, because the run needs the c11 browser (no Chrome on this machine or in CI); the measurement is recorded in the Validation note and the README instead. See deviation D5.
- `web/test/layout.test.js` — untouched (web/wall already registered).
- `room/Cargo.toml` / `Cargo.lock` — untouched (nothing needed).

## Tests by criterion

| ID | Where |
|---|---|
| AC-33 | wall.test.js (CSS: no page scroll, well clips) + measured layout (validation, c11 browser) |
| AC-39 | wall.test.js reveal: ✓, totals, middle line, receipt present; no explains text |
| AC-40 | wall.test.js: ✓ glyph with the correct class on bar and option; no ✗ |
| AC-74 | wall.test.js: receipt under `.provenance.machine`; no human marker on wall |
| AC-78 | felt, HC-1; type model applied (derivedFontPx + refit) — measured numbers recorded |
| AC-79 | wall.test.js + wall_page.rs: no interactive control in any phase or in the page; page carries no pre-reveal answer |
| AC-99 | wall.test.js: tk- spans live/closed/split, zero work/reveal, no source idle/released |
| AC-100 | validation: every bank question (q3, q4, q7, q8) at the default room, measured in the c11 browser (WKWebView) at 1120×630: font px, passes, overflow, verdict; red edge rendered where clipped |
| G-7 | wall.test.js: rendered lines == payload lines for every receipt fixture; the wall has no receipt function |

## Open choices made

1. Layout of the 203px block in work (2 columns) and reveal (3 columns: trace | bars+middle | correct option+receipt). The prototype's full-width work/reveal **overlaps** (measured in the c11 browser: `.trace` at y=330 h=189 over `.proj-side` at y=322 h=203 in work; in reveal the side is 402px tall over the well). A one-to-one copy would ship an overlap; I keep every element the prototype and §4 show, inside §5.2's fixed 203px reserve.
2. split shows bars in place of options (prototype), 2-col grid with bars left.
3. Released link displayed without the scheme (prototype shows `popquiz.rustnyc.org/last`); the QR encodes the full `released.link`.
4. Join strip: the link part of `join` bolded by splitting on `PQ.COPY.wall_idle_join`/`wall_live_join` around `‹link›` — no string retyped.
5. Reconnect: exponential backoff 0.5 s → 5 s; the last frame stays on screen while reconnecting (the wall has no error UI; nothing in §11 for it).
6. Fit reporting: pending the Orchestrator (comment posted). Default: verdict computed and shown as the red edge on the wall; not sent to the room.

## Contract tensions and the side taken

- D1 (G-7 vs boot text): boot names "web/shared's twin of receipt_lines"; none exists and web/shared is not editable. Side: the one function is `answers::receipt_lines` (Rust), the payload carries its lines, the wall renders them verbatim. Comment posted.
- D2 (copy rule vs SPEC §4/§5.1 brand line): brand line rendered; no copy.js key; flagged.
- D3 (boot "beats with the human-provenance marker" vs AC-39): the wall renders no human prose; human marker is the host phone's.
- D4 (`/shared/{file}` vs fonts in a subdirectory): add `/shared/fonts/{file}`.
- D5 (test-full measured layout): no headless Chrome here or in CI; measured in the c11 browser and recorded, not wired into `just test-full`.
- D6 (fit route): rooms.rs not cleared; asked.
- D7 (prototype overlap in work/reveal): non-overlapping layout inside §5.2's reserve.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: general-purpose subagent (sonnet), 2026-09-26. One Critical, two Major, two Minor.

1. **Critical — AC-33/AC-100's `test-full` measured claim downgraded to a manual note, not named in the harness.**
   Concern: EVALUATION marks both `autonomous` under `test-full`; web/shared/README assigns the headline measured claim to T-05; the justfile's rule is that a suite that cannot run is *named*, never silently absent.
   Resolution: the measured run is **built** — `web/wall/measure.html?run=1` runs the real wall code (mount → derive → render → measure → refit → verdict → edge) over every bank question (q3, q4, q7, q8) in all five source phases on the real q3 payloads and emits a JSON report (font px, passes, text box, overflow, verdict, edge, page overflow). It needs a real layout engine, and this machine and CI have no headless browser (no Chrome/Chromium installed; network-free `test`). So: `justfile` gains a `wall-layout` recipe that serves the repo and prints the URL, and `test-full`'s report names it on its own line as run in a browser, not headless — the suite is named, not dropped. The recorded run goes in the Validation attachment and `room/README.md` (Pages). First run (c11 browser, WKWebView, default room 15/8.44/20, floor 14.22px): every question fits in every phase, overflow 0 on both axes, page overflow 0. Flagged to the Orchestrator as a deviation needing sign-off (wiring a headless browser into `test-full` is T-21's deploy-side harness or a follow-up, not a new dependency PQ-7 may add).

2. **Major — the reveal "most-chosen incorrect option" line has no prototype precedent.**
   Resolution: recorded as its own deviation (D8): the element is new UI required by SPEC §4/§4.5 and AC-95/AC-39 and carried by the payload (`reveal.middle.line`); styled as plain 18px text beneath the reveal bars, no colour, no mark on that bar (§9: nobody is marked wrong). Not presented as a port.

3. **Major — D6 leaves the host's fit line empty by default.**
   Resolution: asked the Orchestrator (comment 19:56Z). The wall already computes and exposes the verdict (`mount(..., {onFit})`), so the route is a small addition once `AppState::record_fit` is cleared. If no answer arrives before the PR, the PR ships without the route, says so as the first deviation, and the DONE comment lists it as the open item — merge is the Orchestrator's call, not mine.

4. **Minor — criteria proofs not yet written.** Resolution: noted; they are the Implement phase (wall.test.js, wall_page.rs, the fixture golden).

5. **Minor — `with_room` on every page request plus `no-store`.** Resolution: kept. The 404 for a room that does not exist is required by the ticket, and `no-store` stops a projector laptop from showing yesterday's page after a deploy. One mutex read per page load.

Additional (found while prototyping, before review returned):
- D9: SPEC §5.2's trace text box is 994×190; the built wall measures 994×182 in `work`/`reveal` (the prototype's 190 came from a well that overflowed its container into the overlapping block). The refit is measured, so this costs one refit pass (q3: derived 23.8 → fitted 22.2px), never a clip. Reading layout measures exactly 994×177 in `live`, matching §5.2 and bank-audit.
- Join strip and released link display without the URL scheme (`join @ 127.0.0.1:3000/…`); the QR encodes the full link.
- QR encoder independently verified: macOS CoreImage `CIDetector` decodes my output exactly for versions 2, 3, 7 and a UTF-8 URL.

## Reset 2026-09-26 by agent:delegator-pq7
