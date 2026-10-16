# web/shared

The pieces the wall, the buzzer, the host phone and take-it-home all consume.
Built **first and alone** (BUILDPLAN T-02) because three tickets depend on it.

Ported one-to-one from `prototypes/_shared/proto.js`, `prototypes/_shared/tokens.css`
and the type-model block of `prototypes/C-projector-first.html`. The prototype is
the binding visual contract (`DESIGN.md`); where this directory and the prototype
disagree on how something *looks*, the prototype wins and this is wrong. Where
they disagree on *behaviour*, `SPEC.md` wins.

## What is here

| File | What it is |
|---|---|
| `tokens.css` | The house brand — §1 of the prototype stylesheet, itself verbatim from the Rust NYC Design System. |
| `fonts.css`, `fonts/` | Cascadia Mono and Instrument Serif, self-hosted (D-14). Provenance and checksums in `fonts/README.md`. |
| `components.css` | Everything the prototype carries outside §1 that these surfaces need: the source well, the trace, the syntax colour, the provenance markers, the clipped edge. |
| `dom.js` | `escapeHtml`, and `announce()` — AC-83's polite live region. |
| `phase.js` | The seven phases, `rendersSource()`, `colourAllowed()`, `tracing()`. AC-99 as a pure function. |
| `well.js` | The source well, line numbers, highlight-and-dim, and phase-scoped Rust syntax colour. |
| `trace.js` | The trace renderer, and the step bound. |
| `typemodel.js` | The wall's derived type size, the measured refit, and the fit verdict. AC-100. |
| `copy.js` | Every binding string from `SPEC.md` §11, keyed — and the only place a page's words come from. |
| `check.js` | The ✓ helper. AC-40. |

## How to load it

Classic scripts and globals. There is deliberately **no `"type": "module"`**
anywhere in `web/` — the prototype loads its JS through plain `<script src>` and
the port stays that way, so the browser needs no bundler and there is no
`package.json`, no `node_modules` and nothing to install.

```html
<link rel="stylesheet" href="../shared/tokens.css">
<link rel="stylesheet" href="../shared/components.css">
<script src="../shared/dom.js"></script>
<script src="../shared/phase.js"></script>
<script src="../shared/well.js"></script>
<script src="../shared/trace.js"></script>
<script src="../shared/typemodel.js"></script>
<script src="../shared/copy.js"></script>
<script src="../shared/check.js"></script>
```

Everything attaches to one `window.PopQuiz` namespace, and every module reads
`PopQuiz.*` at call time rather than at load time, so the tags compose in any
order. `tokens.css` imports `fonts.css` itself; load `components.css` after it.

Load only what a surface needs — the buzzer wants `dom`, `copy` and `check` and
none of the rest.

## Three things a consuming ticket must not get wrong

**1. The phase decides the colour, and nothing else does.** AC-99 is
three-valued: colour on in `live`/`closed`/`split`, off in `work`/`reveal`, and
no source at all in `idle`/`released`. Ask `phase.js`; do not pass a `syntax`
flag. `renderSource()` returns `false` when the phase carries no source, so you
can branch without re-asking. An unknown phase throws rather than defaulting to
colour on.

**2. The trace bound is a 0-based inclusive index.** `SPEC.md` §3.4:
"`trace_step` — in `work` bounded to `0..M-2`; `reveal` enters at `M-1`." The
last step is the resolving one — its `values` names `stdout` — and D-10
withholds it until reveal. Use `workMaxIndex(M)` and `revealEntryIndex(M)`
rather than writing `M - 2` at a call site. The bound itself is room state and
belongs to T-04a; `trace.js` only takes it as a parameter.

**3. The wall's font size is derived, never read off a file.** `components.css`
sets 13px on `.rn-src pre`; that is the default for surfaces read at a human
distance (review, take-it-home) and **not** the wall's. The wall calls
`derivedFontPx()`, renders, measures, and calls `refit()`, on entering *every*
phase that renders the source.

## The type model, and its Python mirror

`SPEC.md` §5.2 is the source of truth. Three stages:

1. **Derive** — `floorPx()` from the room's measurements, `desiredFontPx()` from
   the source's shape, `derivedFontPx()` combining them. The floor wins over
   the 46px cap.
2. **Refit** — render, measure, shrink by `k × 0.97`, at most six times, never
   below the floor. `refit()` takes its measurer as a parameter, which is both
   how it stays testable without a browser and how the wall injects a real
   `measureWell()`.
3. **Verdict** — `fitVerdict()` returns `fits`, `clipped_x`, `clipped_y` or
   `clipped_xy`, and `null` for a well nobody measured. **`null` is not
   `fits`**: AC-100 makes *fits* a measured claim, so an unmeasured well has no
   verdict rather than an optimistic one. There is no §11 fit line for `null`.

**T-19 mirrors this in Python** for `bank-audit`, which fits every bank question
offline with no browser to measure in, so it reimplements stage 1 only. The three
functions it must match are `floorPx`, `sourceMetrics` and `desiredFontPx`; the
constants it must share are in `TYPE_CONSTANTS` and `CODE_AREA`; and **both
implementations test against the same worked examples in `SPEC.md` §5.2** —
q3 → 22.1, q4 → 18.4, q7 → 22.1, q8 → 18.4, and q5, q6, q2, q1 below the floor.
If a number moves here, it moves there.

`optionFits()` is the other half of what T-19 enforces: one line of at most 29
characters (D-15).

## Criteria

`web/test/` proves, under `just test`:

- **AC-99** in full — colour spans present in `live`/`closed`/`split`, exactly
  zero in `work`/`reveal`, no source element in `idle`/`released`.
- **AC-100**, the derivation and the verdict logic. The headline measured claim
  ("zero overflow wherever the state says *fits*") is T-05's under `test-full`,
  and the bank flag is T-19's under `bank-audit`.
- **AC-40**, the shared helper only — that no path yields the colour class
  without the ✓. The four surfaces the criterion names (wall, buzzer,
  take-it-home, review) are T-05, T-06, T-12 and the review ticket, and **each
  carries its own AC-40 assertion**. T-02 does not discharge it for them.
- The §11 copy is complete in both directions and matches neither the Forbidden
  patterns (AC-98) nor the trope patterns (§11.1) — see *Copy*, below.

## Copy

**§11 first, then the key, then the mirror.** `SPEC.md` §11 is where
participant-facing strings are authored, "here and only here". To add one: put
it in §11 first; port it into `copy.js` with a comment naming its row and its
key in `COPY_ROWS`; then add the same constant to `room/src/copy.rs` (the key
upper-cased, same string, same place in `ALL`). `copy.test.js` walks
`COPY_ROWS` both ways and `room/tests/twins.rs` walks the two files both ways,
so a string in one place and not the other fails `just test`. Change words the
same way, from §11 down; never here first.

**The freeze.** A page gets its words from `copy.js` and nowhere else — through
`PQ.t(key, values)` or, for the wall, from the room, which fills the same
templates from `copy.rs`. `web/test/copy-freeze.test.js` proves it by drawing
every screen of the wall, the static fallback, the buzzer, the host phone and
take-it-home with every copy value swapped for a marker naming its key, and
failing on any word left that is not the screen's own data — `aria-label`s and
live-region strings included. A value filled into a template's placeholder is
read like any other word, so a literal a page fills in is seen.

Its allowlist prints on every run. Each entry names a surface, an element and
the only words it lets through, with its reason:

- **Host status line** (`p.host-status.meta`): the room's `reason` shown
  verbatim, an open question (Q2). Only the fixture reason's words pass; a
  source scan also checks `host.js` assigns no worded literal to `ui.status`
  (a regex net: concatenation or indirection can slip past it).
- **Take-it-home machine row** (`span.v`): the rustc flag spelling, eight
  compiler words.
- **Take-it-home date heading** (`h1`): the twelve month names from `home.js`'s
  `MONTHS`, which fill `home_heading_question`.

`COPY_FIELDS`, the room fields whose strings are §11 templates the room
filled, is pinned both ways against the wall, static and take-it-home
fixtures: a field carrying a template of three or more words that is not in
the list fails, and so does a listed field no fixture fills.

What it cannot see, by name:

- **A word that is also the screen's data.** Each screen's vocabulary is its
  own data (the question, the code, the fixture's notes), so a literal that
  matches a data word passes. On take-it-home and the static fallback that
  data is a whole question's prose, so common words (`the`, `is`, `a`, …)
  count as data there and a literal made of them passes.
- **Glyphs** with no letter in them (`← → ● ↑ ✎ · ✓ —`): not words.
- **The room's own strings outside `copy.rs`** (its refusal `reason`s): data
  here; the host status line is where one reaches a page.
- **Paths no fixture draws.** The render test sees what the screens it draws
  show; the source scan for retyped §11 values reads multi-word values only.
- **Short templates in the `COPY_FIELDS` pin.** A new field filled from a one-
  or two-word template is not detected (a full match that short is as likely
  ordinary data).

**`PROPOSED-§11`.** Strings a page already showed that §11 does not author yet
sit in a marked block at the end of `COPY`, words unchanged, under the row
`PROPOSED-§11 (not yet in SPEC)`, keys prefixed `proposed_`. They are routed
upstream (F-34, F-38 and the three T-22 found) and leave the block for their
row the day §11 adopts them. Nothing new goes in: a new sentence is a §11
change.

**The two lints.** `web/test/copylint.js` (test-only, so not here: this directory holds what pages load and the room serves) holds SPEC's patterns verbatim; its Python
twin `pipeline/src/popquiz/copylint.py` holds the same text. Each suite reads
`SPEC.md` and fails if its copy differs, and both run the hand-written fixtures
in `bank/fixtures/copy-lint/`; the JavaScript suite also runs the Python twin
over every fixture and fails on any difference (one direction: the pipeline
suite starts no process). Python is compiled to JavaScript's behaviour
(ASCII `\b` and case folding, JavaScript's `\s`); the one case the engines
cannot agree on, a `{0,N}` span over astral characters, is a fixture of its
own.

Both normalize first, the same five steps in the same order: NFKC; zero-width
characters (U+200B–U+200D, U+FEFF) removed; whitespace runs, newlines included,
to one space; U+2018, U+2019, U+02BC and the backtick to `'`; `--`, U+2013,
U+2014 and a spaced hyphen to `—`. A match is quoted from the normalized text.
NFKC comes from each runtime's Unicode tables (Python's `unicodedata`, Node's
ICU); the fixtures use characters whose mapping is long settled, and a
character newer than one runtime's tables may normalize differently there.

A lint that crashes returns one `lint-error` warning (the field and the
exception's class), never no warnings.

| Over | Lints | Consequence | Where |
|---|---|---|---|
| The copy module (`copy.js`, `PROPOSED-§11` included) | Forbidden + §11.1 | Any match **fails `just test`** | `web/test/copylint.test.js` |
| Question prose: `explains.what`, `explains.takeaway`, each `why_tempting`, `hint`, every `trace.steps[].note` (never `explains.legacy`, `source` or option text). The notes are F-47: AC-98 outranks SPEC §11's four-field list. | Forbidden + §11.1 | A **warning**, never a failure; `copylint.check_prose(record)` never raises | The review screen (§7.4) will show them beside the text — T-18 calls `check_prose`; nothing renders them yet. `pipeline/tests/test_copylint.py` runs it over every committed question and prints each warning's field and rule id, never the prose. |


