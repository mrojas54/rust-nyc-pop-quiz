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
| `copy.js` | Every binding string from `SPEC.md` §11, keyed. |
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
  patterns (AC-98) nor the trope patterns (§11.1). The reusable lints are T-22's.

## Editing the copy

`SPEC.md` §11 is where participant-facing strings are authored, "here and only
here". To add one: put it in §11 first, then port it into `copy.js` with a
comment naming its row and add its key to `COPY_ROWS`. The completeness test
walks that index in both directions, so a string in one and not the other fails.
