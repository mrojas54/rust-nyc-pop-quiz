# PQ-27: Copy freeze and lints

BUILDPLAN.md T-22 (M4).

The copy freeze: every participant-facing string from `SPEC.md` §11 in one module; the forbidden-copy lint and the §11.1 trope check in `test`, the trope check failing the build on any match in the copy module (D-23)

Criteria: AC-98, AC-59, AC-42, G-5
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-05–07, T-12
BUILDPLAN notes: —

Orchestrator notes: Creates or completes `web/shared/copy` (F-11): serialized on `web/shared`. The forbidden list and the §11.1 patterns are SPEC §11's, verbatim.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-30)

Base: `origin/ai-c11-cc/a11y-sweep` @ b0dda3e. Baselines on the base: `node --test 'web/test/*.test.js'` 243/243 in 1.6 s; the pipeline suite
passes under the pyenv 3.13.3 interpreter by absolute path (`PYTHONPATH=src …/3.13.3/bin/python3 -m pytest --ignore=tests/sandbox`, 4.6 s),
while `just test-pipeline` itself cannot start here (uv cache is outside the sandbox; the pinned 3.12.10 lacks libintl). CI runs the recipe.

## 1. One source for the patterns (AC-98, AC-42, G-5, D-23)

- **`web/shared/copylint.js`** (new, classic script on `PQ`, like every shared module): `PQ.COPYLINT = { FORBIDDEN, TROPES, forbidden(text), tropes(text), check(text) }`.
  Patterns are held as **source strings verbatim from SPEC**: the nine Forbidden-row patterns (`\|` in the markdown table unescaped to `|`) and the
  five §11.1 groups (contrast ×5, filler ×2, signpost ×4, reassurance ×4, flattery ×1). Compiled with flag `i` (no `u`).
  `check(text)` returns `[{lint: 'forbidden'|'trope', group, pattern, match}]`.
- **`pipeline/src/popquiz/copylint.py`** (new, stdlib `re` only): the same source strings, `FORBIDDEN`, `TROPES`, `forbidden_matches`,
  `trope_matches`, a frozen `Warning(field, group, pattern, matched)` dataclass, and `check_prose(record) -> list[Warning]`.
- **Pinned to SPEC, not to each other by hand**: both suites parse `SPEC.md` (the Forbidden row's backticked patterns; the §11.1 fenced block)
  and assert their module's sources equal it, in order. So neither can drift from SPEC, hence not from each other.
- **Regex-dialect equivalence.** JS `i` without `u`: `\b`/`\w` ASCII-only, `\s` = JS's Unicode whitespace set, `[^.!?]{0,N}` counts UTF-16 code units.
  Python compiles with `re.ASCII | re.IGNORECASE` (ASCII `\b`, `\w`, ASCII-only case folding, which is identical for these all-ASCII
  patterns) and rewrites each `\s` in the **compiled** form (the held source stays verbatim) to JS's explicit whitespace class
  `[\t\n\v\f\r    -     　﻿]`. No lookarounds in any pattern.
  **Where they cannot be identical:** `{0,90}`/`{0,40}` count UTF-16 units in JS and code points in Python, so a span of astral characters
  (emoji) near the bound can match in Python and not JS. That case is a fixture with per-language expectations and a comment, never hidden.
- **`bank/fixtures/copy-lint/`** (new), hand-written from SPEC, never recorded from the code (the `receipts` precedent):
  `retired.json` (AC-42's seven retired strings, each with the groups it must fire), `near-misses.json` (the two allowed
  near-misses: filled *‹n› of us said ‹X›* and *Why ‹n› of us said ‹X›* — no Forbidden match), `forbidden.json` (one string per Forbidden
  pattern), `clean.json` (one clean near-miss string per trope group, matching nothing), `dialect.json` (NBSP after a full stop, a
  non-ASCII letter against `\b`, a Kelvin-sign/long-s case-fold case, and the astral-span divergence). Each case: `text`, expected pattern ids
  (`forbidden/<n>` or `<group>/<n>`), and, only in `dialect.json`, `expect_js`/`expect_py` where they legitimately differ.

## 2. The lints in `test`

- **`web/test/copylint.test.js`** (new): SPEC pin; every fixture file's expectations; every group fires on `retired.json` and every retired
  string is caught; near-misses allowed; **over the copy module every entry clean of both lints (any match fails)** — the trope check fails
  the build (D-23).
- **`web/test/copy.test.js`** (rewrite): keeps completeness, row, placeholder and PQ-34 tests; drops its private transcription (lines 150–271)
  in favour of `PQ.COPYLINT`.
- **`pipeline/tests/test_copylint.py`** (new): SPEC pin; the same fixtures; `check_prose` over q3-shaped records (field naming
  `explains.what`, `explains.takeaway`, `options[i].why_tempting`, `hint`; `explains.legacy`, `source`, `options[].text` never checked);
  `check_prose` never raises on malformed/partial records (None, missing keys, non-string fields, a `Question` dataclass or a dict);
  a test over **every record in `bank/questions/`** that emits each warning through `warnings.warn(ProseWarning(...))` so pytest's
  warnings summary prints them — information, never a failure (q3's hint *"The key word is one…"* is expected to show).
- `just test` picks both up by glob (`node --test 'web/test/*.test.js'`, pytest `testpaths = ["tests"]`) — no `justfile` edit.

## 3. The freeze (G-5, SPEC §11 "authored here and only here")

**Moves into `copy.js`**, words unchanged, under a marked `PROPOSED-§11` section and a `COPY_ROWS` row `PROPOSED-§11 (not yet in SPEC)`, keys prefixed `proposed_`:
- `home.js` `PROPOSED` (F-34): `nothing_yet`, `miri_separately`, `row_compiler`, `row_edition`, `row_target`, `row_flags`, `row_miri`, `miri_seeds`.
  `PQ.Home.PROPOSED` stays as a read-through getter (key → `PQ.t(...)`) so `home.test.js` (not mine) stays green.
- `trace.js` (F-38): `aria-label` *previous step* / *next step*.
- Found by the inventory, not in the brief (each a QUESTION, see §7): `trace.js` `traceSay`'s *‹name› is now ‹now›*; `check.js`'s default
  sr-only *Correct*; `well.js`'s default label *Source code*.
**Retyped §11 strings become key lookups** (same output): `trace.js` *Step ‹N› of ‹M›* → `wall_trace_step`, *Pause here.* → `wall_trace_pivot`,
default label → `wall_live_well_header`; `traceSay`'s *Step N of M* → `wall_trace_step`.
**`room/src/copy.rs`** gains the `PROPOSED_*` constants and `ALL` entries in file order (twins test pins both directions); doc comment's stale
`tests/copy_mirror.rs` reference → `tests/twins.rs`. `twins.rs`: a test that `ALL`'s `proposed_` keys are exactly copy.js's PROPOSED row.
**`web/test/_load.js`**: `copy.js` added to the `trace`, `well`, `check`, `typemodel` groups, since those modules now read `PQ.t` at call time
(deviation: a test-harness file not named in §2; no behaviour change).

**`web/test/copy-freeze.test.js`** (new) — the structural test:
- Loads every surface in a `node:vm` context where `copy.js` is rewritten at load so each value becomes a marker `⟦key ‹p›…⟧` (placeholders
  kept so `fill` works; applied at source so load-time captures like `HOST_PHASE_LABEL` are markers too). Wall frames from the room fixtures
  (`web/wall/fixtures/q3-phases.json`) carry the room's filled strings; each string field that fully matches a real §11 template is mapped to
  its marker (which also proves the room sends §11 strings), everything else is data.
- Renders the same screen set as the a11y suite (wall every phase/step incl. unanimous; static fallback every reachable {phase, step}; buzzer
  through its reducer incl. every refusal, paused, hint; host sign-in/create/refused/every phase; take-it-home null + every step of each
  snapshot), plus every live-region string (wall `announcement`, buzzer `announce` effects, `HOST_PHASE_LABEL`, `traceSay`).
- Collects text nodes and the participant-facing attributes (`aria-label`, `title`, `alt`, `placeholder`, `aria-description`), removes every
  marker, and fails on any remaining **letter-bearing word** that is not in that screen's input data vocabulary. Glyph-only strings (→ ← ● ↑ ✎ ·
  ✓ —) carry no language and are out of scope, stated.
- Allowlist: printed on every run; expected entries: the host status line (room `reason`, HTTP status text, exception text — §7 finding).
- Page shells: each `web/{wall,buzzer,host,home}/index.html` `<title>` equals `title_wordmark` or `title_wordmark · host_title_role|buzzer_title_role`
  (§11 Title row); no other literal text in their bodies. `measure.html` and `web/test/a11y/browser.html` are dev harnesses, excluded by name.
- Source check: no surface file (`web/{wall,buzzer,host,home}/**/*.js`, `web/shared/*.js` except copy.js) holds a string literal equal to a
  multi-word §11 value (a retyped string), comments stripped.

## 4. AC-98's second half (in `copy-freeze.test.js`)

- Every control (`button`, `a`, `input`, `[role=button]`) on every screen, by marker: its key ∈ {host actions, sign-in, join, hint, retry} or it is
  a letter A–E or a step glyph. No control or string (real text) matches a contribution vocabulary
  (`speak|say|tell|share|discuss|compare|volunteer|ask|turn to|raise|hand|neighbou?r|partner`) — checked over the real COPY values and every
  control label.
- Counts: host shows only `count_joined`/`count_answered` (+ step N/M), wall only joined/answered, split totals and the most-chosen line; no
  screen carries a per-person identifier; take-it-home carries none of the count markers and its snapshot fixture has no count-like field.
- Never waits: the host's primary action is enabled with `answered: 0, present: 0` in `live` (Close answers needs no answers), and `split`,
  `work`, `reveal` render with zero votes; no surface schedules a prompt (no `setTimeout`/`setInterval` in a render path that yields copy).
- Absence of response: no non-answer marker on any surface but the participant's own buzzer (the room never says who did not answer).

## 5. AC-59

`buzzer_released` (*Nothing about you was recorded.*) was **removed by the client at HC-0 (PQ-34, 2026-09-27)** and `copy.test.js` pins its absence;
§11 has no *Buzzer, released* row. The brief asks for it as a module key served at release. **Side taken: the client's later decision** — I do
not re-add a string the client cut, and I author none. The test asserts what the released buzzer does serve: the head only, no count, no line,
and the live region's `live_released`. QUESTION to the Orchestrator (AC-59's `felt` half has no product line now; HC-1/HC-3 decide).

## 6. Refusal reasons (PQ-4's note)

Buzzer: a join refusal and a 409 answer both carry a sentinel `reason`; the test proves the sentinel never reaches the view (rendered from §11
keys by code). Host: `host.js` `statusLine` renders the room's `reason` verbatim (and HTTP status text / exception text / close codes) in
`.host-status`; two reasons are §11 keys (the denials), others are diagnostics such as phase.rs's *"There is no room yet. Create a room comes
first."* — **finding, QUESTION**, not fixed (behaviour change, and a string I would have to author). The test pins it to that one element as
the allowlist's one entry.

## 7. QUESTIONs for the ticket (posted as a comment at implementation)

Q1 AC-59 vs PQ-34. Q2 host status renders room `reason`/HTTP text verbatim. Q3 PROPOSED additions beyond F-34/F-38: `traceSay`'s *is now*,
`check.js` *Correct*, `well.js` *Source code*. Q4 SPEC §11 runs the Forbidden lint over question prose but gives it no consequence; I report
Forbidden matches in `check_prose` as warnings alongside tropes (never a failure).

## 8. Docs

`web/shared/README.md` *Editing the copy* → *Copy*: §11 first, then the key, then the Rust mirror; `PROPOSED-§11`; the two lints, where each
runs, and each consequence (module: fail; prose: warning on the review screen via `check_prose`).

## 9. Exit

`just test-web`, pipeline suite (stand-in locally, CI for the recipe), `just test-room` (twins), `just canary`; `git diff origin/main --stat`
shows no `room/src` file but `copy.rs`.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: Sonnet subagent, 2026-09-30. Verdict: no Critical; regex-dialect claim confirmed sound for the 9 + 16 patterns.

1. **Exit diff vs base** — concern: `git diff origin/main --stat` includes #37's stack while #37 is open. Resolution: during the build the
   scope check is `git diff origin/ai-c11-cc/a11y-sweep...HEAD --stat` (no `room/src` file but `copy.rs`); the `origin/main` form is re-run
   after #37 merges and I merge main in. Both reported in DONE.
2. **AC-59 unsatisfied** — Resolution: unchanged side (the client's HC-0 cut stands; I author no string), but raised as QUESTION Q1 in a ticket
   comment at the start of implementation, not only at DONE, and named in the PR body as an open criterion needing a decision before merge
   (amend AC-59/EVALUATION row, or restore a line through §11).
3. **Forbidden over prose has no failure** — Rejected as a build failure: the brief fixes `check_prose` as never raising and the bank test as
   "information, not a failure"; making committed bank prose a build gate would be a contract change. Forbidden matches are reported in
   `check_prose` with `group="forbidden"`; Q4 asks the Orchestrator whether committed bank prose should fail.
4. **No producer for the review-screen warning** — Accepted: docs say `check_prose` is the hook T-18 (PQ-23) must call; nothing renders it yet.
   Plan §8 wording corrected accordingly.
5. **Curly apostrophes evade** — Accepted: `check()`/`check_prose` normalize U+2018/U+2019 to `'` before matching, in both languages; patterns
   stay verbatim. Fixture case in `dialect.json`. Listed as a deviation.
6. **Trace notes unlinted** — SPEC §11 names the homes (`explains` minus legacy, `why_tempting`, `hint`); notes are not among them. Not linted;
   raised as Q5.
7. **Tropes lack per-pattern positives** — Accepted: new `bank/fixtures/copy-lint/tropes.json`, one positive per trope pattern (16), so every
   compiled pattern (incl. the `\s` rewrite) is exercised.
8. **Whitespace class** — Accepted: built from explicit `\u` escapes in code; `dialect.json` carries one case per JS-`\s` code point used as
   the separator after a full stop, run through both engines.
9. **Freeze leniency** — Accepted: the limit (a literal word that also occurs in that screen's data passes) is stated in the test header and
   README; per-screen data vocabulary (not global) keeps it small.
10. **Room-authored strings outside copy.rs** — room/** beyond copy.rs is PQ-26's; stated as a limit; the phase.rs reason is part of Q2.
11. **PROPOSED is unauthored copy** — the brief prescribes the `PROPOSED-§11` block; each entry is listed in the Q3 comment for a SPEC §11
    amendment upstream (F-34, F-38, plus the three new ones).

## Reset 2026-09-30 by agent:delegator-pq27
