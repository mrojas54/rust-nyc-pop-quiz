# PQ-2: Shared web layer: tokens, fonts, well, type model

BUILDPLAN.md T-02 (M0).

`web/shared/`: tokens and fonts vendored from the design system; port `proto.js` — source well, syntax colour, trace renderer, the type model with measured refit and the clipped-edge, the phase strings from `SPEC.md` §11

Criteria: AC-99, AC-100, AC-40
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-01
BUILDPLAN notes: Built alone; three tickets consume it

Orchestrator notes: Put the ported §11 phase strings in `web/shared/copy` from the start so T-22 moves nothing (F-11). Port from `prototypes/_shared/proto.js`, `tokens.css`; fonts from the design system per SPEC §15 (D-14). The source well font size is behaviour (SPEC §5.2), never read off the prototype; every other size is the prototype's.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

---

# Plan (delegator, 2026-09-20)

Base: `origin/ai-c11-cc/repo-scaffold` @ `2ab95e31b78ee48a7067dc47904e98b28baf3d37` (PR #8).
Worktree: `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/shared-web-layer`, branch `ai-c11-cc/shared-web-layer`.

## 0. Where the thing I am porting actually lives

`proto.js` does **not** contain the type model. It carries `sourceCodeHtml` /
`rustColour` (the well and syntax colour), `traceStepOf` / `traceSourceHtml` /
`traceNoteHtml` / `traceSay` (the trace), `announce`, `escapeHtml`, `pct`,
`mountBadge` and `explainHtml`. The **type model — the derivation, the six-pass
measured refit, and the fit verdict — lives in `prototypes/C-projector-first.html`
lines 366–468** (`WALL_CANVAS_H`, `CAP_RATIO`, `MAX_FONT`, `CODE_AREA_H/W`,
`minLegibleFontPx`, `applyTypeModel`, `measureWell`).

So "port `proto.js`" is read as *port the shared prototype plumbing*, which is
`proto.js` plus that block of the wall prototype. Porting only `proto.js`
literally would ship T-02 with no `typemodel.js` and fail AC-100.

Out of the `proto.js` surface I deliberately do **not** port:

| Function | Why not |
|---|---|
| `mountBadge` | the `PROTOTYPE` badge is removed in the build (`SPEC.md` §5.1). |
| `explainHtml` | the three-beat explanation is the wall's and take-it-home's (T-05, T-12), not shared plumbing; it also reads room vote state, which `web/shared` must not know about. |
| `pct` | one line, used only by `explainHtml`. |

`escapeHtml` and `announce` **are** ported — `announce` is AC-83's polite live
region and all three surfaces need it, and every renderer here depends on
`escapeHtml`.

## 1. Files to create

All under `web/shared/` (mine alone this ticket) and `web/test/`.

| File | What it is |
|---|---|
| `web/shared/tokens.css` | `prototypes/_shared/tokens.css` §1 verbatim (lines 17–83) — colors, typography, spacing, base — plus the `.by-machine` / `.by-human` / `.provenance*` block (lines 277–285). One deliberate edit: see T-1. |
| `web/shared/fonts.css` | `@font-face` for Cascadia Mono and Instrument Serif, `font-display: swap`, local `fonts/` paths only. |
| `web/shared/fonts/` | the font binaries + `OFL.txt` per family + `fonts/README.md` (upstream URL, release tag, SHA-256 per file). |
| `web/shared/well.css` | the `.rn-src*` component block and the `.tk-*` syntax-colour block, ported from `tokens.css` lines 247–263 and 359–365. See T-2. |
| `web/shared/trace.css` | the `.trace*`, `.trace-values`, `.tval*`, `.trace-nav`, `.trace-dots` blocks, ported from `tokens.css` lines 324–351. See T-2. |
| `web/shared/dom.js` | `escapeHtml`, `announce` (AC-83). The two primitives every other module here needs. |
| `web/shared/phase.js` | `PHASES`, `rendersSource(phase)`, `colourAllowed(phase)`. AC-99's pure function. See T-6. |
| `web/shared/well.js` | `renderSource(el, source, {phase, hl, focus, size, label, meta})`, `sourceWellHtml(...)`, `rustColour`. Colour applied only when `colourAllowed(phase)`. |
| `web/shared/trace.js` | `traceStepOf`, `traceSourceHtml`, `traceNoteHtml`, `traceSay`, `clampStep`, `workMaxIndex(M)`, `revealEntryIndex(M)`. See T-7. |
| `web/shared/typemodel.js` | `ROOM_DEFAULTS`, `TYPE_CONSTANTS`, `floorPx()`, `derivedFontPx()`, `refit()`, `measureWell()`, `fitVerdict()`, `applyClippedEdge()`. See T-3, T-4, T-5. |
| `web/shared/copy.js` | every participant-facing string from `SPEC.md` §11 + §7.5, one keyed entry each, each with a comment naming its §11 row. |
| `web/shared/check.js` | `checkMark()` / `correctLabelHtml()` — AC-40's ✓-with-every-colour helper. |
| `web/shared/README.md` | what the module is, the classic-script/global contract, and the note that **T-19 mirrors `derivedFontPx`/`floorPx` in Python and tests against the same §5.2 worked examples**. |

### Files I change rather than create

- `web/test/layout.test.js` — **no change needed.** It asserts `web/` has exactly
  `shared|wall|buzzer|host|home` plus `test`; I add no top-level directory
  (`web/shared/fonts/` is nested). Verified by reading it, not assumed.

### Files I must not touch

`justfile`, CI config, `pyproject.toml`, `room/src/phase.rs`, `.env.example`,
and every contract file. `just test` already runs `test-web` → `node --test
web/test/`, so my tests are picked up with no aggregator edit. Confirmed against
`justfile:31`.

## 2. Tests, by criterion ID

Node's built-in runner, classic scripts loaded through `node:vm` per
`web/README.md`. One shared `web/test/_load.js` helper builds a context with
`window`/`document` stubs and evaluates a `web/shared/*.js` file into it.

| Test file | Criterion | What it asserts |
|---|---|---|
| `web/test/typemodel.test.js` | **AC-100** | `floorPx()` at the `15 / 8.44 / 20` default = 14.2 to 1 dp. The eight §5.2 worked examples to 1 dp: q3→22.1, q4→18.4, q7→22.1, q8→18.4 fit; q5→12.3, q6→12.3, q2→11.1, q1→6.9 all **below the floor**. The 46 cap binds on a 1-line, 10-char source. **The floor beats the cap**: with a configuration whose floor exceeds 46 the result is the floor, and the model reports it. `refit()` runs at most 6 passes, applies `k × 0.97`, and never goes below the floor. Geometry is built programmatically (`"x".repeat(42)` × 5 lines), never copied from a bank question, so no test hard-codes a program or its output. |
| `web/test/typemodel.test.js` | **AC-100** | `fitVerdict()` returns each of `fits` / `clipped_x` / `clipped_y` / `clipped_xy` from stubbed measurements, and `applyClippedEdge()` sets the red-edge class on the losing side(s) only. |
| `web/test/phase.test.js` | **AC-99** | `colourAllowed` true for `live`/`closed`/`split`, false for `work`/`reveal`/`idle`/`released`; `rendersSource` true for the first five, false for `idle`/`released`; both throw on an unknown phase rather than defaulting to permissive. |
| `web/test/well.test.js` | **AC-99** | Rendered HTML contains ≥1 `class="tk-` span in `live`, `closed`, `split`; **exactly zero** in `work` and `reveal`; `renderSource` emits no source element at all in `idle`/`released`. Counted over a fixture source that contains a keyword, a macro, a type, a string, a number and a comment, so a regression that drops one token class is caught. |
| `web/test/well.test.js` | AC-33 sibling | the line-number gutter is `2ch`, `<pre>` carries the derived size, and the markup has the `rn-src-scroll` region (the fit is measured on the wall by T-05/`test-full`; here the structure it measures is asserted). |
| `web/test/trace.test.js` | AC-97 / D-10 bound | `clampStep` honours a caller-supplied `maxIndex`; `workMaxIndex(M) === M-2` and `revealEntryIndex(M) === M-1` (0-based); stepping **both ways** works; at the work bound the last step's `values` (the `stdout`/resolving step) is unreachable. `M = 2` and `M = 1` degenerate cases clamp to 0 rather than going negative. |
| `web/test/trace.test.js` | §5.3 | `note`, `values` with `was`/`now` and `—` for absent, `Step N of M`, dots, and *Pause here.* on a `pivot` step. |
| `web/test/copy.test.js` | §11 completeness | every key in a checked-in `SECTION_11_ROWS` list resolves to a non-empty entry, and every entry in `copy.js` appears in that list — **completeness in both directions**, so neither a dropped string nor an invented one passes. |
| `web/test/copy.test.js` | AC-98 / AC-42 | no entry matches the §11 *Forbidden* patterns; no entry matches the §11.1 trope patterns. See T-9. |
| `web/test/check.test.js` | **AC-40** | the helper emits the ✓ glyph in its output whenever it emits a colour class, and there is no code path that yields the class without the glyph. |

`just test` must stay green and warm under 60 s (0.28 s today). The web suite is
pure JS with no I/O beyond reading the source files; I expect it in the tens of
milliseconds and will report the measured number.

## 3. Open choices I made

**C-1 · One module per concern, not one `shared.js`.** `PHILOSOPHY.md` §8
prefers the smaller mechanism, but three consumers each want a different subset
(the buzzer needs `copy` and `check` and neither `well` nor `typemodel`). Separate
classic scripts let each page load what it uses. No module graph, per
`web/README.md`.

**C-2 · Globals, not exports.** `web/README.md` is explicit: classic scripts, no
`"type": "module"`, tested through `node:vm`. Each file attaches to a single
`window.PopQuiz` namespace object it creates if absent, so four `<script src>`
tags in any order compose. Tests read the namespace out of the vm context.

**C-3 · Pure functions unrounded; round only at the edge.** See T-3.

**C-4 · The trace bound is a 0-based inclusive `maxIndex`.** See T-7.

**C-5 · Fonts: upstream OFL releases, checksummed.** `SPEC.md` §15 says the
fonts come from the design system's `assets/fonts/`, which is **not in this
repository** (`tokens.css:23-25` says so explicitly). The ticket directs me to
the official upstream releases instead. Cascadia Mono ships in
`microsoft/cascadia-code` releases; Instrument Serif in
`Instrument/instrument-serif`. Both OFL 1.1. The download needs network and the
client's approval; I ask once with the exact command visible, record URL + release
tag + SHA-256 per file in `fonts/README.md`, and commit the licences beside them.
**If the client declines the download**, I ship `fonts.css` and `fonts/README.md`
with the exact URLs and expected checksums, land everything else, and flag the
missing binaries in the completion comment rather than substituting a CDN (D-14)
or a look-alike font.

**C-6 · No `mvp/` or `prototypes/` file is edited or read at runtime.** The port
copies; it does not import across the boundary.

## 4. Contract tensions, and the side I take

**T-1 · `tokens.css` §1 is "verbatim" but line 22 is a Google Fonts CDN
`@import`.** D-14 says self-hosted, no font CDN; AC-77 runs a segment with
outbound network blocked; AC-102 requires the static file to make **zero**
network requests. *Side:* drop that one `@import` line and the three-line comment
above it, and replace them with `@import url("fonts.css");` pointing at the
vendored faces. Everything else in §1 is byte-identical. The `--font-heading` /
`--font-mono` stacks are untouched, so the fallback chain still works if a face
fails to load. Recorded as deviation 1.

**T-2 · The well and trace CSS is not in §1.** The deliverable list names only
`tokens.css` (§1 verbatim), but `.rn-src`, `.rn-src-line.hl/.dim`, `.tk-*`,
`.trace*` and `.tval*` live in §3/§6 of the prototype stylesheet — prototype-only
blocks the design system does not have yet. `well.js` and `trace.js` render markup
that is unstyled without them, so a deliverable that "ports the source well" and
ships no well styling is not the well. *Side:* ship `well.css` and `trace.css`,
ported verbatim from those blocks, beside their JS. This adds two files the ticket
did not name; it restyles nothing. Recorded as deviation 2.

**T-3 · Rounding.** The prototype rounds to one decimal *inside* the refit loop
(`Math.round(x*10)/10`, twice). `SPEC.md` §5.2's pseudocode rounds nowhere and
gives the worked examples "to one decimal" as presentation. *Side:* the pure
functions carry full precision; rounding happens only where a number is displayed
or asserted. Reason: T-19 reimplements this formula in Python and tests against
the same worked examples, and a JS-specific intermediate rounding is exactly the
kind of thing that makes two implementations disagree in the third decimal and
then in the first. `SPEC.md` outranks the prototype on behaviour, and a font size
is behaviour (the orchestrator's own note says so). Recorded as deviation 3.

**T-4 · `screen_h_in` and the 150 ratio.** The prototype computes
`screenHIn = screenWidthFt * 12 * 9/16` and treats `readRatio` as a live control.
`SPEC.md` §5.2 makes `screen_height_ft` its own organizer-set value
(`screen_h_in = screen_height_ft * 12`, "default = width × 9/16") and bakes 150
into the formula. *Side:* SPEC. Three room-configuration values
(`screen_width_ft`, `screen_height_ft`, `back_row_ft`), defaults `15 / 8.44 / 20`
exactly as §5.2 line 315 writes them; `READ_RATIO = 150` and `CAP_RATIO = 0.7`
exported as named constants, not configuration. Both readings give a 14.2 px floor
to one decimal (15 × 9/16 = 8.4375 → 14.222; 8.44 → 14.218), so no worked example
moves. HC-1 replaces all three with measured numbers.

**T-5 · "split" means two different things.** `SPEC.md` uses `split` as a
**phase** (the bars, after close) and assigns it the **reading** layout,
994 × 177. The prototype uses `split` as a **layout mode** (source beside
options, `CODE_AREA_H.split = 426`, `CODE_AREA_W.split = 564`) — the alternative
the client rejected at T-20 item 15 in favour of full-width. *Side:* SPEC. Two
code areas only — `READING = {w: 994, h: 177}` for `live`/`closed`/`split` and
`TRACE = {w: 994, h: 190}` for `work`/`reveal`. The prototype's split-layout
constants are not ported at all. This is the single highest-risk misreading in
the ticket and the naming in `typemodel.js` will say so in a comment.

**T-6 · AC-99 is three-valued, not two.** "Colour present in live/closed/split,
zero in work/reveal, **no source at all** in idle/released" cannot be expressed
by one boolean. *Side:* two predicates — `rendersSource(phase)` and
`colourAllowed(phase)` — and `renderSource()` emits nothing for a phase that
renders no source. Both throw on an unknown phase: a typo'd phase string must not
silently fall through to "colour on", which would be an AC-99 failure that looks
like a render bug.

**T-7 · The M-2 / M-1 bound has no stated index base.** AC-102 and AC-97 say
`work` carries "no step beyond `M-2`" and `reveal` "enters at `M-1`". Read as
**0-based indices** into `trace.steps`, consistent with D-10 and §5.3: the last
step (index `M-1`) is the resolving one whose `values` names `stdout`, so it is
exactly the step withheld until reveal, and `work` tops out one before it at
index `M-2` — displayed as *Step M-1 of M*. Read as 1-based, `work` would stop
two steps early and `reveal` would never show the resolving step at all, which
contradicts D-10. *Side:* 0-based. `workMaxIndex(M) = M-2` and
`revealEntryIndex(M) = M-1` are exported named functions so T-04a and T-05
cannot get the base wrong by hand, and the README states the convention.

**T-8 · The ticket text in Lattice says "PR base: origin/main"; the delegator
boot prompt says base `ai-c11-cc/repo-scaffold`, stacked on #8.** *Side:* the
boot prompt — it is the more specific and more recent instruction and explains
why (I need T-01's harness, and the Orchestrator retargets after #8 merges).
Noted so the Orchestrator can correct me cheaply if that is backwards.

**T-9 · The copy lint is T-22's, but §11.1 says a trope match in the copy module
"fails the build".** *Side:* I do not build the lint tool. I do add a test that
runs the §11 *Forbidden* patterns and the §11.1 trope patterns over my own
entries, because shipping a copy module I have not checked against the two lints
that exist to police it would hand T-22 a red suite on arrival. The patterns live
in my test file, not in a reusable linter; T-22 owns the tool and may delete my
copy of the patterns when it lands.

**T-10 · `explainHtml` uses the field name `e.argue`, which contains the
forbidden substring `argu`.** Not my deliverable and not a participant-facing
string, so not a defect I can fix here — but `SPEC.md` §11's *Forbidden* row
matches `argu` case-insensitively, and `SPEC.md` §3.1 names the third beat
`takeaway` while the prototype's data calls it `argue`. T-12/T-05 should use
`takeaway`. Raised to the Orchestrator by comment, not fixed.

## 5. Risks

- **Font binaries are the only network step.** Everything else is offline. If
  the download is refused or the upstream layout has moved, C-5's fallback
  applies and the ticket still lands.
- **`node:vm` + `document` stubs.** `measureWell()` reads `scrollWidth` /
  `clientWidth`, which no stub computes honestly. I test `refit()` against
  *injected* measurement functions rather than a fake DOM, and leave the real
  measurement to T-05 under `test-full` (which is where AC-100's row puts it:
  "`test-full`: the wall's measured overflow is zero where it reports *fits*").
  `typemodel.js` therefore takes its measurer as a parameter — which is also what
  makes it testable at all.
- **Copy verbatim-ness.** §11's table is dense with `‹placeholders›`, nested
  parentheticals and conditional variants. I transcribe by reading the row, not
  by regex-extracting it, and the completeness test is keyed off a hand-written
  row list so a silently dropped variant fails.

---

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: `general-purpose` / sonnet, 2026-09-20, given only the plan path and
the contract section paths. It was **not** given the delegator boot prompt, which
matters for R-1 and R-5. It independently recomputed the §5.2 arithmetic (all
eight worked examples and both floor values matched) and found a citation for the
trace bound that I did not have.

Seven findings: 1 Critical, 4 Major, 2 Minor. **All seven are upheld in whole or
in part. None is dismissed.**

### R-1 · Critical 1 — the font source

**Concern.** The plan sources both fonts from GitHub upstream OFL releases, while
the ticket, `SPEC.md` §15 and the Orchestrator notes all say the fonts are
vendored *from the design system's `assets/fonts/`*. Filed under "Open choices"
as if compliant rather than flagged as a tension. The reviewer also asked whether
the design system Artifact named in `DESIGN.md` (`claude.ai/design d608a53a…`)
had been tried.

**Resolution — upheld in substance; the substitution is mostly withdrawn.** Two
parts.

*(a) Provenance of the instruction.* It was not invented. The delegator boot
prompt §2 says, verbatim: "The design system's `assets/fonts/` is not in this
repository; obtain the files from the fonts' official upstream releases (both are
OFL) — the download prompts the client — and record the URL and checksum." The
reviewer could not see that. It should nonetheless have been recorded as a
tension with `SPEC.md` §15 rather than as a free choice, and that is corrected
below as **T-11**.

*(b) Substance — the reviewer's suggestion was right and I acted on it.* The
design system is published at `claude.ai/artifact/2k3zt9WWz6XUZRjkLDWvfi`
("Rust NYC Design System", 123 files). Read with the Artifact tool, it carries:

| Path in the design system | What it is |
|---|---|
| `project/fonts/CascadiaMono-VariableFont_wght.ttf` | 631,788 B, variable `wght` 200–700, upright |
| `project/fonts/CascadiaMono-Italic-VariableFont_wght.ttf` | 471,980 B, variable `wght` 200–700, italic |
| `project/assets/fonts/OFL.txt` | 4,490 B, SIL OFL 1.1 |
| `project/assets/Uploads/Cascadia_Mono/README.txt` | the upstream packaging README |
| `project/tokens.css` | the generated token file, **with its own `@font-face` rules** |

So **Cascadia Mono ships from the design system exactly as `SPEC.md` §15 says** —
fetched with the Artifact tool, no network download, no upstream substitution,
no client approval needed. Its `@font-face` rules are ported from the design
system's own `project/tokens.css` (`format("truetype")`, `font-weight: 200 700`,
`font-display: swap`, upright + italic) rather than written by me.

**Instrument Serif is not in the design system.** `--font-heading` names it, but
no font file exists anywhere in those 123 files; the prototype obtained it from
the Google Fonts CDN `@import` that D-14 forbids. Instrument Serif alone
therefore comes from its upstream OFL release, per the boot prompt, with URL,
release tag and SHA-256 recorded in `fonts/README.md`. That one download is the
ticket's only network step and the only thing needing client approval.

**Contract defect to report to the Orchestrator (not fixed here):** `SPEC.md`
§15 states that *both* Cascadia Mono and Instrument Serif "ship in
`web/shared/fonts/` from the design system's `assets/fonts/`". Only Cascadia Mono
is there. §15 is wrong about Instrument Serif, and D-14's "self-hosted, no font
CDN" cannot be satisfied for it from the named source.

**Also worth recording:** every token value in `prototypes/_shared/tokens.css` §1
was compared against the design system's `project/tokens.css` and matches — the
colour ramps, the spacing scale, the type scale, `--radius`, `--shadow-card`, and
both font stacks. §1's claim to be verbatim holds, so `tokens.css` can be ported
with confidence.

### R-2 · Major 2 — AC-100 is scoped to `test-full`, not `test`

**Concern.** The table bolds **AC-100** against a `node:vm` unit test, but
`EVALUATION.md:131` puts AC-100's proof in `test-full` (measured overflow on the
real wall) and `bank-audit` (the too-long flag). The plan's own Risks section
concedes this, contradicting the table.

**Resolution — upheld.** The label overclaimed. The table is corrected to read
**"AC-100 — derivation and verdict logic only"**, with the scope stated inline:
the measured-overflow half is T-05's under `test-full`, and the bank flag is
T-19's under `bank-audit`. What T-02 owns and can prove under `test` is that
`floorPx`, `derivedFontPx`, `refit` and `fitVerdict` compute what §5.2 specifies —
which is the thing the other two hooks will call. No test claims more than
`just test` can run.

### R-3 · Major 3 — AC-40 names four surfaces, none of which exist yet

**Concern.** `EVALUATION.md:102` requires the ✓ on the wall, the buzzer, take-it-home
and the review surface. `check.test.js` can only show the shared helper is
internally consistent.

**Resolution — upheld.** Relabelled **"AC-40 — the shared helper only"**. The
four consuming surfaces are T-05, T-06, T-12 and the review ticket, and each must
carry its own AC-40 assertion. `web/shared/README.md` will say so, so the
consuming tickets inherit the obligation rather than assuming T-02 discharged it.

### R-4 · Major 4 — AC-42's positive proof is not covered

**Concern.** `EVALUATION.md:104` requires a fixture of the seven retired strings
proving *every trope pattern group fires*. The plan tests only that its own copy
is clean.

**Resolution — upheld, and adopted as a strengthening of my own test.** Without
the positive fixture, "no entry matches the patterns" can pass because my
transcription of the patterns is broken — the assertion would be vacuous. So
`copy.test.js` gains the seven retired strings from `EVALUATION.md:104` (*That is
not a room getting it wrong — that is a room…*, *Not the explanation — that one is
human*, *listen, don't read*, *that's fine*, *the most interesting answer in the
room*, *The word doing all the work*, *The bit worth talking about*) and asserts
each of the five pattern groups — `contrast`, `filler`, `signpost`,
`reassurance`, `flattery` — fires on at least one of them. Labelled **"AC-42 —
partial: pattern transcription only"**; T-22 owns the row and the reusable lint.

### R-5 · Major 5 — the provenance CSS is §5, not §1

**Concern.** The plan folds `.by-machine` / `.by-human` / `.provenance*`
(lines 277–285) into "tokens.css §1 verbatim", but §1 ends at line 84 and that
block sits under "§5 — Design-system components". Structurally identical to the
well/trace CSS, which *was* flagged as tension T-2 — inconsistent treatment. AC-74,
which that block styles, is not one of T-02's criteria.

**Resolution — upheld on the fact, and the inconsistency is removed.** Confirmed
by reading the section headers: §1 is lines 18–84, §2 begins at 86, §5 at 218,
§6 at 288. The block genuinely is not §1. (The instruction is real — the boot
prompt §2 says tokens.css is "§1 … including `.provenance.machine` /
`.provenance.human` from that file" — but the reviewer is right that it needed
the same treatment as T-2.)

**The file layout changes** so that all non-§1 CSS is treated alike:

| File | Contents |
|---|---|
| `web/shared/tokens.css` | **§1 only**, lines 18–83, verbatim but for the `@import` (R-6), plus an `@import url("fonts.css")`. |
| `web/shared/fonts.css` | the `@font-face` rules for both families. |
| `web/shared/components.css` | **everything not in §1**, each block prefixed by a comment naming its source section and line range: the `.rn-src*` well (§5, 247–263), the `.tk-*` syntax colour (§6, 359–365), the `.trace*` / `.tval*` / `.trace-dots` trace styles (§6, 324–351), and the `.by-machine` / `.by-human` / `.provenance*` provenance block (§5, 277–285). |

This replaces the plan's `well.css` + `trace.css` split — three CSS files, not
four, and one legible rule ("§1 is tokens.css; everything else is
components.css"), which also serves `PHILOSOPHY.md` §8. Nothing is restyled.
Superseded: the `well.css` and `trace.css` rows in §1's file table, and T-2's
two-file remedy. **AC-74 is noted as not T-02's criterion**; the provenance
classes ship because three consuming tickets need them styled, and the criterion
is proven by whoever renders them.

### R-6 · Minor 6 — the stale comment is below the `@import`, not above it

**Resolution — upheld, factual.** `prototypes/_shared/tokens.css`: line 21 is the
`/* --- tokens/fonts.css --- */` section header *above* line 22's `@import`;
lines 23–25 are the Cascadia-Mono / SF-Mono fallback comment *below* it. T-1 is
corrected: **keep** line 21, **drop** line 22 (the CDN `@import`), and **replace**
lines 23–25 with a comment naming the vendored faces and pointing at `fonts.css`.
The comment is replaced rather than deleted because what it says — that the fonts
are not vendored and the stack falls back to SF Mono — stops being true in this
repository and would mislead the next reader.

### R-7 · Minor 7 — "the first five" is wrong; it is the middle five

**Resolution — upheld.** `SPEC.md` §63 (G-6) gives the order
`idle → live → closed → split → work → reveal → released`, so `idle` is first and
renders no source. `rendersSource` is true for the **middle five** — `live`,
`closed`, `split`, `work`, `reveal` — and false for `idle` and `released`. The
boolean values in the plan were right; only the phrasing was wrong.

### T-11 · New tension, raised by R-1 (replaces open choice C-5)

**`SPEC.md` §15 and the boot prompt disagree about where Instrument Serif comes
from, and §15 is the one that is wrong.** §15 says both fonts ship from the
design system's `assets/fonts/`; the design system has Cascadia Mono and not
Instrument Serif. *Side:* Cascadia Mono from the design system (§15 honoured
exactly); Instrument Serif from its upstream OFL release (the boot prompt's
direction, and the only way to honour D-14's "self-hosted, no font CDN" at all).
Both recorded in `fonts/README.md` with source, version and SHA-256. Reported to
the Orchestrator as a contract defect in `SPEC.md` §15. **If the client declines
the Instrument Serif download**, the ticket still lands: `fonts.css` keeps the
face declared with its fallback stack (`Georgia, 'Times New Roman', serif`),
`fonts/README.md` records the expected URL and checksum, and the gap is named in
the completion comment. No CDN, no look-alike substitute.

### T-7 gains a citation

The reviewer found what the plan inferred: **`SPEC.md`:146 states the convention
outright** — "`trace_step` — in `work` bounded to `0..M-2`; `reveal` enters at
`M-1`". T-7's 0-based reading is therefore the contract's own words, not an
inference from D-10, and `web/shared/README.md` will cite `SPEC.md` §3.4 rather
than argue the case.

### Unchanged by the review

The §5.2 arithmetic (independently reproduced), the T-4 and T-5 SPEC-over-prototype
resolutions, the files-not-to-touch list, `layout.test.js` compatibility, and
AC-99's `test` scoping — the reviewer checked each and raised nothing.

### Found outside the review, while verifying the harness

`web/README.md:25` documents the web suite as `node --test web/test/`. On Node 22
that resolves the directory as a CommonJS module path and fails with
`Cannot find module …/web/test` before any test runs. The `justfile` is correct
(`node --test 'web/test/*.test.js'`, line 46) and `just test` is green, so only
the documented command is wrong. `web/README.md` is **not** in this ticket's
cleared file list, so it is reported to the Orchestrator, not fixed here.

## Reset 2026-09-21 by agent:delegator-pq2
