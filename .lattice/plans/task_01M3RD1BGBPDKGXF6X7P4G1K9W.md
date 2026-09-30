# PQ-38: Re-authored q4, q7, q8: machine-verify them and reconcile the pipeline tests

main is red since the client's PR #36 (1d721f4, 'Re-author q4/q7/q8 options for the 29-char wall rule', D-15). The records were rewritten by hand and 're-verified by running' outside the verifier; the pipeline's just test now fails: tests/test_migration.py:89 (q4.json on disk is not what the migration writes), test_migration.py (written['legacy'] KeyError for the re-authored records), test_migration.py:138 (rustc 1.98.1 in the record vs the MVP batch's 1.96.1), verified.stdout != mvp['answer'], tests/test_fit.py:107 (q4 source metrics (5,78) vs the expected (6,56)), tests/test_audit.py:756 (audit.main --date 2026-09-21 returns 0, expected 1). PR #37 and every later PR inherit the failure. House rule AC-7: correct answers come from a verified record written by the verifier, never by hand. Fix: run the pinned verifier (just verify, T-15a image) on q4, q7, q8 so the machine writes their verified blocks; treat them as re-authored (non-legacy, non-migrated) records; move the migration/fit/audit expectations to match; CI green on main. If the machine's answer differs from what PR #36 typed, stop at needs_human - the content is the client's. Never edit mvp/**.

# Plan (delegator, 2026-09-30)

## What is actually red (measured locally, temp venv on uv's cpython 3.12; the pyenv python is broken)
20 failures, not 6: test_migration (drift x3, legacy/version-line/runs-vs-MVP/answer-vs-MVP for q4,q7; "no middle beats" q4,q7,q8; review-note wording q4,q7,q8; q8 distractor-note), test_fit (q4/q7 shapes), test_audit (reserve flag premise), and two in test_verify.py:
- test_every_bank_question_passes_the_provenance_check - q4/q7 carry no verified_at+verifier_version. Fixed by the verifier write, no test change.
- test_every_program_has_a_recording_of_its_current_source[q8] - PR #36 dropped q8's source's trailing newline, so the stub recording's _source_sha256 no longer matches. Fix: re-record on the image with tests/fixtures/verify/record.py (the only sanctioned way; it rewrites every recording). Cleared path: pipeline/tests/fixtures/**.

## Steps
1. `just sandbox-build` (Docker, client approves), then `just verify bank/questions/q4.json --expect ran` and same for q7. Verifier replaces `verified` whole (D-16). If stdout != PR #36's typed stdout (q4 "-3 -1 | -4 1\n", q7 "['a', 'd', 'b', 'c']\n") or rejected -> needs_human + flag. Note: for --expect ran the verifier runs Stacked Borrows only (verify.py:407-436); Tree Borrows is asked only for declared UB. The boot prompt's "Miri under both borrow models" is the verifier's rule to decide, not mine - noted as deviation 1.
2. Append one sentence to q4/q7 review.reason: "Re-verified by the pinned verifier (just verify, rustc 1.98.1) on 2026-09-30." No other wording.
3. Re-record verify fixtures on the image (record.py) for q8's newline change.
4. test_migration.py: split MIGRATED into STILL_LEGACY=(q3,q8) and REAUTHORED=(q4,q7).
   - drift test: q3 exact bytes; q8: verified block == migration's, source equal modulo trailing newline (PR #36 dropped it), options may differ; q4/q7: non-legacy, verified_at+verifier_version present, not is_stale vs pin, check_provenance passes, review.reason says "Re-authored".
   - legacy / version-line tests parametrize over STILL_LEGACY. runs-and-miri-vs-MVP over [q3].
   - correct-option test: all four; answer = normalized_output(record stdout) for compiling, kind does_not_compile for q8; q3 also still equals MVP answer.
   - "no middle beats" test -> replaced: q4/q7/q8 now have a beat on every incorrect option (AC-95) - asserting what is true.
   - review-note test: q3 only via the generator's wording is migration-origin; for q4/q7/q8 assert no option breaks the 29-char rule now (D-15) and reason says re-authored. q8 distractor test: correct option unchanged (does_not_compile), q4/q7 correct option is the verifier's stdout.
5. migrate() re-run: save_question overwrites unconditionally (migrate_mvp.py:507-509) - a re-run would clobber q4/q7/q8's re-authoring AND q8 (still legacy verified, re-authored options). The boot's "skip if verified not legacy" guard would not protect q8. Guard taken: skip a question whose file already exists and differs from what the migration would write, reporting it as kept (deviation 2). Test added. history.json left as is (out of scope; noted).
6. test_fit.py: pin re-measured shapes q4 (5,78), q7 (6,67); both fit READING_AREA (measured 20.3 px / 18.4 px). SPEC 5.2 worked table (SPEC.md:285-286) still names the August shapes (6x56, 5x69) - deviation 3, route F-40. The WORKED parametrized table stays (it reproduces SPEC's own numbers).
7. test_audit.py: synthesize the flagged question in the tmp copy: q7's first incorrect option set to 48 chars, accepted+affirmed.

## Criteria
AC-6 (is_stale false against pin for q4/q7), AC-7 (check_provenance on all four; answer = verifier stdout), AC-11 (q8 codes kept), AC-95 (beats on every incorrect option), AC-100 (fit), D-15/D-16.

## Gates
full pipeline pytest locally (temp venv), just bank-audit, CI both jobs on PR head.
