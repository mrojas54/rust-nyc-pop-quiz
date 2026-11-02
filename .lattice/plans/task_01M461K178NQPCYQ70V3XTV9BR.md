# Plan (delegator, 2026-11-01)

**Outcome.** A Forbidden phrase, or a lint crash, in any real bank question's prose fails `just test`. Tropes stay warnings. The review screen is unchanged.

**Re-verified on e073ec7.** `check_prose` still only warns. Over the bank: q3 `hint` signpost/3 (a trope), q4, q7 and q8 clean, no Forbidden match. Ruling 6 holds, no halt.

**The test** (`pipeline/tests/test_copylint.py`).
- `_blocking_lines(report)`: every report line whose warning group is not a 11.1 trope group. Forbidden and `lint-error` both land here, and so would any future lint family (fails closed). Lines carry the stem, field and rule id only, so no prose reaches CI logs.
- `test_no_committed_question_has_a_forbidden_match_or_a_lint_crash`: walks the real bank through `_bank_report`.
- `test_the_walk_found_the_bank_and_read_prose_from_every_question`: fails on an empty walk, on a question with no prose read (a renamed field would otherwise read as clean), and on a non-JSON entry in `bank/questions/` that the walk would skip. No fixed count.
- In-memory probes, so the guard stays tested after the mutations are reverted: the audit's sentence in each of the five homes blocks; a trope in each home does not; a crash blocks. Synthetic records, no program output (AC-7).

**Prose homes.** `copylint.prose_fields`: `explains.what`, `explains.takeaway`, `options[].why_tempting`, `hint`, `trace.steps[].note`. Wider than SPEC 11's list by the notes (F-47), which goes in Contract notes.

**Docs.** `web/shared/README.md` table row and the crash line; the `copylint.py` module docstring and `check_prose` docstring; the `review.py` docstring; this test file's header, `ProseWarning` docstring and report comment. Each states both halves: the review screen warns, `just test` fails.

**Outside the cleared files.** `web/test/copylint.test.js:12-13` says "never a failure" and `web/test/copylint.js:28` states the warning half only. Comments only. Asked by comment and mail; work continues. If unanswered at PR, listed under not-done.

**Mutations** (each reverted, `git status` confirms `bank/questions/**` clean).
1. M5d2: the audit's sentence in q4 `explains.what` must fail the new test.
2. The same sentence in one q4 `trace.steps[].note` must fail it.
3. The same sentence in one q4 `why_tempting` must fail it.
4. A trope in q4 `explains.what` must not fail it.
