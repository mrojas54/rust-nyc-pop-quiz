# PQ-42: Copy lint follow-ups: the trace notes, the freeze's blind spots, lint bypasses

Follow-up of PQ-27 (PR #38, merged at a599e80). Source: artifact art_01M3Z8N3FYVWJW0H8JFBMWT73V on PQ-27. (A) MAJOR, F-47: the lint's prose_fields (pipeline/src/popquiz/copylint.py:157-180) reads explains.what, explains.takeaway, why_tempting and hint - SPEC's list - but not trace.steps[].note, generated prose that the wall, host and take-it-home all show; AC-98 covers every participant-facing string and outranks SPEC's list. Extend prose_fields (and the JS twin) to trace.steps[].note with fixtures in bank/fixtures/copy-lint; both implementations must agree; SPEC's field list goes upstream as F-47 (do not edit the contract). (B) Freeze blind spots (web/test/copy-freeze.test.js): :118 strips each marker with its filled placeholder, so the MONTHS literal (web/home/home.js:60-68, the date heading) is invisible - declare it in the allowlist or route it through copy.js; allowlist entry 1 (:312-316, the host status line) has no words filter, so any literal assigned to host ui.status passes - add words; COPY_FIELDS (:85) treats any new room copy field as data - pin the field list; list every limit (including common words counted as data on take-it-home and static) in web/shared/README.md instead of 'two entries'. (C) Lint bypasses in both languages (copylint.js:95-97, copylint.py:126-129): collapse whitespace, apply NFKC, strip zero-width characters and normalize apostrophe and dash variants in normalize, quote from the normalized text, and add fixtures for each probe (newline, double space, NBSP, zero-width, U+02BC, backtick, fullwidth letters, -- and ' - '). (D) copylint.py:193-194 turns a lint crash into [] which reads as clean (pinned by tests/test_copylint.py:179-183): return a sentinel warning and pin that instead. (E) tests/test_copylint.py:199 prints the matched prose fragment to CI logs: print the rule id and field only, so a private bank never leaks prose into logs. NITs: retyped pattern lists at web/test/home.test.js:277-289, room/tests/phase_table.rs:255 and copy-freeze.test.js:464 (pin to one source where cheap); README.md:50-51 should say check, well and trace need copy.js at call time; room/tests/twins.rs:403-412 scans only the top level of room/src (walk src/bin too). q3's hint trips the signpost trope: a correct warning, the client's call, leave it. Acceptance: AC-98, AC-57..AC-60; just test green and under 60 s; each new check shown failing under its mutation; both lint implementations agree on every fixture. Harness: just test-web, just test (pipeline).

# Plan (delegator, 2026-10-16)

Worktree `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/copy-lint-followups`, branch `ai-c11-cc/copy-lint-followups` off `origin/main` @ 2058d3e. No copy string, no question, nothing under `room/src/` changes.

## 1. Notes field (ruling 1, F-47)

- **Python** `prose_fields`: after `hint`, read `trace` → `steps` (list/tuple) → each step's `note` (str) as field `trace.steps[{i}].note`. `_get` already reads dicts and dataclasses, so `bank.Question` (trace: Trace, steps: tuple[TraceStep]) works unchanged. Docstring names the fifth home and cites AC-98 over SPEC's list (F-47).
- **JS** `web/test/copylint.js` gains `proseFields(record)` and `checkProse(record)`, line-for-line the Python order: `explains.what`, `explains.takeaway`, `options[i].why_tempting`, `hint`, `trace.steps[i].note`. Same rules, severities (warning), quoting.
- `Warning` gains `id` (the rule id, e.g. `signpost/3`), needed by (E) and the sentinel. Nothing outside copylint/tests constructs `Warning` (checked: only `generate.py` imports copylint, for `RULES`).
- **Fixture** `bank/fixtures/copy-lint/notes.json`: `{"about", "records": [{"record", "expect": [[field, id], …], "note"}]}`: one clean trace (expect `[]`), one trace with one violating step per family (forbidden, contrast, filler, signpost, reassurance, flattery), clean steps between, and one full-shape record whose only violation is a note (so the note is not reached via another field).

## 2. Normalization (ruling 3) — identical in both `normalize`, applied in this order

| # | Step | Python | JS |
|---|---|---|---|
| 1 | NFKC (fullwidth → ASCII, NBSP/ideographic space → space) | `unicodedata.normalize("NFKC", t)` | `t.normalize("NFKC")` |
| 2 | strip zero-width U+200B, U+200C, U+200D, U+FEFF | `re.sub` | `.replace` |
| 3 | whitespace runs (JS `\s` class, incl. newline) → one space; trim | `JS_WHITESPACE+` | `/\s+/g` |
| 4 | apostrophes U+2018, U+2019, U+02BC, backtick → `'` | | |
| 5 | dashes U+2013, U+2014, `-{2,}`, and ` - ` (space-hyphen-space after step 3) → `—` (U+2014, the form contrast/1 spells) | | |

Match **and quote** from the normalized text (lengths change, so the old index-into-original is gone). The JS test that pinned "quoted from the original, curly apostrophe and all" now pins the normalized quote (`That's fine`). Normalization only widens what matches: every step maps a variant onto the spelling SPEC's patterns already use; none removes a letter or a word boundary that a pattern needs (zero-width removal joins `wr​ong` into `wrong`, which is the point).

**Fixture** `bank/fixtures/copy-lint/normalization.json`, one case per probe (nine): newline and double space and NBSP (`Don't⏎worry`, `Don't␣␣worry`, `Don't worry` → reassurance/4), zero-width (`wr​ong` → forbidden/6), U+02BC and backtick (`Thatʼs fine` / ``That`s fine`` → reassurance/1), fullwidth (`ｗｒｏｎｇ` → forbidden/6), `--` and ` - ` (`It's not a copy -- it's a move` → contrast/1). A Python test pins that every probe is a bypass of the old lint: the raw text through the compiled rules without normalizing yields a different id list.

## 3. The two lints agree (ruling 2)

Both suites read the same exact file list (now eight: + `notes.json`, `normalization.json`), and both assert their own results equal the fixture's `expect`. In addition, **each suite runs the other implementation** over every fixture and compares results case by case:
- Python: `subprocess.run([node, "-e", <script>])` loads `web/test/copylint.js` and prints `{file: [ids per case]}` plus `checkProse` results for `notes.json`; `node` from `shutil.which`, missing → fail (never skip).
- JS: `child_process.spawnSync("uv", ["run", "--offline", "--no-sync", "python", "-c", <script>], {cwd: pipeline})`; missing → fail.
- The one allowed difference is `dialect.json`'s documented case (`expect_js` ≠ `expect_py`); any other difference fails **both** suites. Hermetic (local processes, no network); `just test` already needs node and the pipeline venv in the same CI job.

## 4. Crash sentinel (ruling 4)

`check_prose` tracks the field it is on; any exception returns exactly one `Warning(field=<field or "record">, id="lint-error", group="lint-error", pattern="", matched=<exception class name>)` — no prose, no partial results. JS `checkProse` the same object shape. The existing never-raises parametrized records do not crash (they are legitimately empty) and keep `[]`; a new test plants an exception (monkeypatched `check`/`_get`, a record whose attribute access raises) and pins the sentinel in both languages.

## 5. No prose in logs (ruling 5)

The bank report in `test_copylint.py` emits `"{stem} {field}: {id}"`. A guard test builds the same report lines through one helper and asserts each fullmatches `^\S+ [A-Za-z0-9_.\[\]]+: ([a-z]+/\d+|lint-error)$` and contains no matched fragment. The JS suite prints no bank prose (it lints only copy.js, public, and fixtures).

## 6. Freeze blind spots (ruling 6) — `web/test/copy-freeze.test.js`

- `words()` strips the marker's `⟦key` and brackets and any unfilled `‹p›`, but **keeps filled placeholder values**, so a literal filled into a template (the date heading's `spokenDate`) is seen; `vocabulary()` uses the same function, so room-filled values stay data.
- Allowlist entry for MONTHS: `{surface: 'home', where: <the masthead heading>, words: [the twelve month names]}`; a test pins that each `spokenDate` month output is within the filter and that the month word is actually seen on the home screens (so the entry cannot go stale).
- Entry 1 (host status line) gains `words: words(REASON)`; plus a source scan: `web/host/host.js` assigns no letter-bearing string literal to `ui.status`.
- `COPY_FIELDS` pinned both ways: walking the wall frames, the static bake and the take-it-home fixtures, every field whose string value fully matches a §11 template must be in `COPY_FIELDS`, and every `COPY_FIELDS` member must carry one somewhere; a synthetic frame with an undeclared field carrying a filled template must fail the check.
- `web/shared/README.md`: the freeze's limits listed by name (data-vocabulary words per screen incl. common words on take-it-home and static, glyphs, room reasons outside copy.rs, unreached paths, each allowlist entry), replacing "two entries"; the lint table gains `trace.steps[].note` with F-47; normalization noted.

## 7. NITs (ruling 7)

- `web/test/home.test.js` §11 test: replace the retyped regex lists with `copylint.check` over the **values** `PQ.t(key)` (the old loop tested the key names, not the strings — a finding).
- `web/shared/README.md`: `check`, `well` and `trace` call `PQ.t` at call time, so `copy.js` must be loaded before they are called.
- `room/tests/twins.rs`: `room_sources(&["cli"])` → `room_sources(&[])` so the proposed-key scan reads the clients too (`src/bin` became `src/cli`; no cli file names `copy::PROPOSED_`).
- Left, with reason: `room/tests/phase_table.rs` forbidden substrings (no one-line pin: room/src holds no list and reading SPEC from Rust is not a one-liner); `copy-freeze.test.js` AC-98 `ask` regex (the criterion's own verbs, a deliberate second net, not a retype of the row).

## 8. Mutations (ruling 10), each run under a 15-minute kill

Notes dropped (py; js) · each normalize step removed (py ×5; js ×5) · sentinel back to `[]` (py; js) · report prints `matched` · `words()` back to whole-marker strip · MONTHS entry removed · status line gains a literal word · `ui.status = "Try again"` in host.js · `COPY_FIELDS` loses `pause` · a proposed copy value gains "wrong" (home.test) · a cli file names a proposed key (twins) · one rule removed from one language → both suites' agreement tests fail. All reverted; `git diff` clean after.

## 9. Commits

1 notes field + checkProse + notes fixture · 2 normalization + probes · 3 cross-language agreement · 4 crash sentinel · 5 log guard · 6 freeze blind spots + README · 7 NITs.

## Contract notes

- F-47: AC-98 outranks SPEC §11's four-field lint list; notes linted; SPEC not edited, goes upstream.
- "One fixture per probe" read as one case per probe in one `normalization.json` (matches the existing one-file-per-theme layout).

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Review: Sonnet, 2026-10-16, 2 Critical / 5 Major / 4 Minor. All triaged:

- **C1 dialect.json under NFKC — accepted.** Pre-checked: exactly three existing cases change, all in dialect.json: `ask ſomeone` → forbidden/2 and Kelvin `the Key part` → signpost/3 (NFKC folds the look-alike; a caught bypass, expectations updated), and `It moves.﻿Not the value.` → `[]` (ruling 3 strips U+FEFF, so it renders and reads as `moves.Not`; recorded as a deviation). To keep the engine-dialect guard live, each dialect case may carry `expect_raw` (the compiled rules over the **un-normalized** text); both suites check `expect_raw` (default `expect`) through the raw rules, so `JS_WHITESPACE` and JS `\s` stay tested. A code comment says step 3 uses the JS class, not `str.isspace` (the U+001C case).
- **C2 Warning shape — accepted, no new clearance needed.** `id` is additive: `review.py:194` passes `asdict(w)` through, so the review report gains an `id` key; `test_review.py` reads `field`/`group` only and is run unchanged as proof. `review.py` is not edited. The sentinel appears in the report only on a crash, which is the point. The `check_prose` docstring is rewritten (never `[]` on a crash).
- **M3 Unicode versions — partly accepted.** Fixtures are written with `\uXXXX` escapes (JSON `ensure_ascii`) and use only characters assigned in Unicode ≤ 3.2 with NFKC mappings unchanged since; the cross-run payload carries `unicodedata.unidata_version` and `process.versions.unicode` and the agreement test names both in its failure message. Agreement is pinned on fixtures; drift on unpinned characters is a documented limit (README).
- **M4 bypasses left — partly accepted.** ` - ` → ` — ` and `--` → `—` both keep surrounding text as is (consistent). Zero-width chars are **stripped** per ruling 3 (so `wr​ong` is caught; `turn​to` → `turnto` is not — documented). U+00B4, U+2032 and combining marks are outside the ruling: deferred list. `matched` now quotes normalized text; no consumer indexes it (review.py passes it through).
- **M5 words() — accepted as a risk to enumerate.** Implementation runs the suite after the `words()` change and lists every new leak with its provenance before deciding; vocabulary is computed from the same marked data as today (filled values from the room are data). If the change surfaces leaks beyond the date heading, those are reported, not allowlisted blind.
- **M6 COPY_FIELDS — accepted.** The pin reads only templates with at least three literal words (no short-template false matches), and checks declared fields both ways; the `ui.status` scan is documented as a regex net (concatenation can slip past), alongside the words filter which is the real guard.
- **M7 mutations — accepted.** Added: sentinel field name dropped, log-line shape regex loosened, the "bypass of the old lint" test (normalize made identity in the raw path), rule removal in **each** direction, home.test `PQ.t` values check, twins.rs `room_sources(&[])`.
- **Minors — accepted.** `node` is resolved from PATH as `just test-web` does, missing → fail; the JS spawn of `uv run --offline --no-sync` requires `pipeline/.venv` (`just setup`), missing → fail; whitespace probes gain U+2028 and U+3000; F-47 cited in the `copylint.js` header.
