# PQ-67: Affirm q3 by the organizer's documented edit, and move the tests that pinned the unaffirmed bank

F-52 ruled 2026-10-10 by the client ('Organizer edit for q3'); the client affirmed q3 as queenrose54 ('affirm q3 queenrose54'). The edit to bank/questions/q3.json's review block (status accepted, affirmed_by queenrose54, affirmed_at 2026-10-10T16:41:43-04:00, reason updated to record the documented edit) is already uncommitted in worktree rust-nyc-pop-quiz-worktrees/affirm-q3 (branch ai-c11-cc/affirm-q3 off origin/main e4156ed). Dry run: reserve (nyc) 0 -> 1, schedule q3 --no-push writes the fallback and host sheet, bank-audit passes. Six pipeline tests pin the pre-ruling state and fail: test_migration.py::test_the_committed_record_is_what_a_fresh_run_writes, ::test_a_rerun_over_a_partial_bank_writes_only_what_is_missing, ::test_nothing_migrated_is_affirmed[q3]; test_schedule.py::test_ac72_an_unaffirmed_question_is_refused_and_nothing_is_sent_or_written, ::test_ac72_every_committed_record_is_refused_today, ::test_ac75_the_trend_counts_what_comes_in_and_what_went_out. Move them to the ruled state without weakening what they guard.


## Plan (agent:delegator-pq67)

- Commit 1: bank/questions/q3.json alone, values untouched — "Affirm q3: the organizer's documented edit (F-52)".
- test_migration golden: ORGANIZER_WRITES = status, affirmed_by, affirmed_at, difficulty_judged, reason. The drafted part of review must match; then the fresh record with the landed review overlaid is serialized like the bank writer and compared byte for byte with the file. A meta-test perturbs each non-organizer field (top-level and in review) and requires the compare to fail.
- Partial-bank rerun: rewritten q3 passes the same compare and is unaffirmed (a migration never affirms); everything else byte-identical.
- New test: a rerun over the committed bank keeps every organizer's review, and q3 stays affirmed. (Today's rerun test already passes bytewise, so this is not a production bug.)
- test_nothing_migrated_is_affirmed: parametrized over MIGRATED, reads a fresh migration's output, not the committed records.
- AC-72 refusal: q4, reset to its drafted review in the copy, so a later affirmation of q4 cannot move it. The every-record test runs on a bank copy: refused with the unaffirmed message iff unaffirmed; q3 schedules (exit 0, one push).
- AC-75 trend: every record in the copy is reset to drafted, then q3 affirmed, q4 accepted awaiting, q7 used; unreviewed = len(bank) - 3. Robust to #59's new records.
- Mutations (bounded): q3 minus affirmed_at; migrate_mvp writing affirmed_by; golden helper ignoring `explains`. Each reverted after.
