# Exact-head review (Orchestrator seat, Sonnet reviewer) — PQ-42 / PR #85 @ 9059a8478242c417a3c897c2749a67884d841ce8

Verdict: PASS-WITH-NITS. Reviewed HEAD 9059a847 (confirmed), base origin/main 2058d3e, 9 commits, 11 files, all cleared; copy.js, room/src, web/home, bank/questions, SPEC.md, EVALUATION.md, justfile, review.py, test_runner.py untouched. Run 2026-10-16 ~19:00 EDT, read-only against the delegator's worktree; mutations on a scratch copy under the reviewer's scratchpad (the worktree was never touched).

Critical: none. Major: none.

Minor
- Two normalizer steps unpinned (removing each passes every suite in both languages): the end-trim (copylint.py:147; copylint.js `.replace(/^ | $/g,"")`) — decides whether "\nNot the value." flags contrast/3, no fixture has leading whitespace; and U+200C (normalization.json probes only U+200B and U+200D). Fix: one fixture each.
- Zero-width set is narrow: U+2060, U+00AD, U+200E/F, U+FE0F (and U+2011, U+2212 as dash look-alikes) walk past both lints; the DONE defers only U+00B4, U+2032 and combining marks. Add to the deferred list or the strip set.
- Ruling 5 not fully met on failure: copylint.test.js:243 and :248 interpolate h.match of copy.js entries; :95 interpolates fixture text; test_copylint.py:224 compares Warning objects built from q3 prose, so a failing diff prints q3 text. Public or hand-written strings, failure path only; the pass-path report is clean.
- web/README.md:28-31 still says the web suite needs nothing to install; copylint.test.js now needs uv plus pipeline/.venv and fails without them (CI runs just setup first). Add a line. spawnSync has no timeout.
- Unpinned allowlist filter: with the MONTH entry's words filter removed the freeze still passes; nothing asserts every ALLOWLIST entry has a filter. Recommend assert(ALLOWLIST.every(a => a.words)).

NIT
- Host-status filter words(REASON) is four ordinary words; a literal "text" or "reason" would pass; the ui.status regex net backs it up.
- The MONTH filter includes "May" and applies to any home h1.
- home.test.js:285 retypes \bwrong\b|\bincorrect\b|✗.
- review.py:194 uses asdict, so review JSON warnings gain an id key (T-18 should know); the sentinel arrives as pattern:"" matched:<class>.
- Node's Unicode tables are 17.0 and Python's 15.0 (observed); the README discloses this.
- The sentinel discards earlier real warnings from the same record (matches ruling 4).
- The U+FEFF dialect case is now [] — accepted (FEFF renders as no gap).

Rulings: 1 met · 2 met (shared expect; JS-side cross-run only; no subprocess in test_copylint.py; test_runner passes) · 3 met with note (same steps, order, sets in both; a 40,000-string differential fuzz over normalize and check found 0 real differences; quotes from normalized text, M10/M19 killed) · 4 met (sentinel both languages, no message leak, [] records still []) · 5 met with note · 6 met with note (MONTHS allowlisted, not routed; host filter, COPY_FIELDS two-way pin, ui.status scan, README limits present; words() keeps filled placeholders) · 7 met · 8 met · 9 met · 10 met with note (survivors under Minor).

Counts observed: just test-web 303/303 in 1.67 s incl. the Python cross-run; copy-freeze 17/17; pytest test_copylint + test_runner 58 passed 0.39 s; full pipeline (ignoring tests/sandbox) 953 passed 12.8 s; Rust --test twins 18 passed; fixtures 73 cases + 5 records = 78, flagged 65 JS / 66 Python as claimed.

Reviewer mutations killed (≈30): NFKC dropped (JS) / identity (Py); sentinel → [] (Py; JS); notes field dropped (Py, JS); JS `--` and ` - ` not folded; Py backtick, U+2018, U+2013 dropped; Py \s instead of the JS class; U+200D dropped (both); quote from the original (both); sentinel leaks message (both); Py sentinel field forced to record; COPY_FIELDS loses lines / gains bogus; unmark hides filled placeholders; ui.status literal in host.js; host filter removed; "October" → "Oct." and a literal " ish" in the date heading; report line carries matched; cli/smoke.rs references copy::PROPOSED_*. Survived: trim removed (both), U+200C dropped (both), MONTH words filter removed.

Not confirmed by the reviewer: just test at 15 s warm (suites run separately); bank-audit identical to main (not run; audit.py does not import copylint and bank/questions is untouched); the delegator's 29-mutation tally; Rust beyond twins.

Process note from the reviewer: a PostToolUse hook injected orchestrator wind-down text into its tool results; it treated that as untrusted output and did not act on it.
