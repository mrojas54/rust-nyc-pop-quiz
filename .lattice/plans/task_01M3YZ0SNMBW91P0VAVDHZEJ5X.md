# PQ-39: Bank re-verify follow-ups: q8's record, the bank README, three blind tests

Follow-up of PQ-38 (PR #40, merged at 1e23c84). Five Minors and three NITs from the exact-head review (artifact art_01M3YXXN85ECH6VMA1R5WJQ89E on PQ-38). (1) q8 keeps its August legacy record (compile_error_code E0502, no verified_at or verifier_version) although PR #36 re-authored its options and stripped its source's trailing newline. Run the pinned verifier: just verify bank/questions/q8.json --expect does_not_compile on the T-15a image, and let the machine write the record (AC-7; never by hand). If the verifier's rules keep a does-not-compile record legacy on purpose (D-22), say so in the DONE comment instead of forcing it. (2) bank/README.md:125-127 and :133-135 are false since PR #40: they say all four migrated records are still legacy and not re-verified, and that a re-run reproduces the files byte for byte. Correct them: q3 and q8 legacy (or q8 per item 1), q4 and q7 verifier-written, a re-run keeps what is in the bank. (3) pipeline/tests/test_migration.py test_a_rerun_leaves_the_bank_as_it_is cannot catch a history regression because bank/history.json is empty: removing the guard at migrate_mvp.py:526-528 keeps it green. Seed a non-empty history in the tmp copy, and add a partial-bank case (one file missing, only that one written). (4) test_the_correct_option_is_the_machines_output restates correct_index for q4 and q7 (bank.py:425-426), a tautology; nothing in CI re-derives their stdout. Add q4 and q7 recordings and replay them through verify, as q3 and q8 have. (5) pipeline/tests/test_audit.py:767 test_strict_fails_on_any_flag: no bank question carries a flag now, so it may pass only through a warn (audit.py:1059, 1478). Confirm what it exercises and build a flag in the copy as test_audit.py:744 does. NITs: migrate_mvp.py:520/:528 exists() would keep a truncated file forever (bank.py:796 append_question already refuses overwrite); commit 6cac9ec stripped q8's trailing newline under 'program unchanged' with only a test docstring saying so; q7 explains.legacy still names bob/amy/cal/dan (predates the PR, disclosed in review.reason). Acceptance: just test and just test-full green; each rewritten test fails when its guard is removed (show the mutation); no answer written by hand. Harness: just test (pipeline), just verify, just bank-audit. Not on the shared-file list.

# Plan (delegator, 2026-10-03)

Base: origin/main @ 2730806. Branch ai-c11-cc/bank-followups.

## (1) q8's record — the verifier replaces it
- Nothing in verify.py keeps a does-not-compile record legacy. D-22 (SPEC §3.2 :120-123) only describes
  what a *legacy* DNC record holds; SPEC :118-120 says a legacy record "is replaced whole — never merged —
  when T-15b re-verifies its question". verify() (verify.py:326-345) returns ACCEPTED with
  record(compile_error_code=codes) = rustc -Vv, edition, target_triple, flags, compile_error_code,
  verified_at, verifier_version. The existing q8 recording already shows that verdict.
- Expected: `just verify bank/questions/q8.json --expect does_not_compile` -> ACCEPTED, E0502, file rewritten
  by bank._write_json. No hand edit of `verified`.
- Only authored field touched in q8.json: `review.reason` (append the re-verify sentence the way q4/q7 do, and
  the NIT: say the source lost its trailing newline in 6cac9ec, no token changed).
- Tests that pinned "q8 is legacy" move with it (they describe the bank, not the contract):
  test_migration.py STILL_LEGACY -> ("q3",), REVERIFIED -> q4,q7,q8 (with reason assertions);
  test_q8_still_carries_the_record_the_migration_wrote -> q8's record now the verifier's; migration
  still writes the legacy one; codes equal; source equal modulo trailing newline; legacy explains kept.
  test_verify.py test_a_legacy_record_is_exempt_from_the_stale_check -> q3 only.
- bank/fixtures/receipts/q8-does-not-compile-legacy.json: keep (it is the only real legacy-DNC receipt case,
  D-22 rendering); rewrite its `_note` to say it is the record migrate_mvp writes for q8 and that q8 carried
  until re-verified. Add a test that it agrees with the bank: its `verified` == what migrate() writes for q8,
  and its expected_lines == receipt_lines(bank q8) (DNC list is the same three lines legacy or complete).

## (2) bank/README.md
- :125-127 -> q3 is the only legacy record; q4, q7, q8 carry records the pinned verifier wrote (dates).
- :133-135 -> migrate() writes only what is missing; re-run keeps existing question files and history; q3
  still reproduces byte for byte (test_the_committed_record_is_what_a_fresh_run_writes).
- :137 "The records are legacy" -> "The migration writes legacy records"; q8 paragraph :153 says it was the
  migrated record. :250-252 "Re-running the migration empties the history again" is false too (guard at
  migrate_mvp.py:528) -> fix. Note 6cac9ec trailing newline. Nothing else.

## (3) test_a_rerun_leaves_the_bank_as_it_is
- Seed a non-empty history in tmp copy (save_history with one entry built via dedupe's own API / History
  fields) — never bank/history.json. Add partial-bank case: delete tmp q4.json; migrate returns [q4 path];
  every other file byte-identical.
- Mutations: (a) drop `if not history_path(...).exists()` guard -> seeded-history test fails;
  (b) drop `if question_path(...).exists(): continue` -> both fail; (c) partial case: write-all-if-any-missing.

## (4) q4/q7 recordings
- cases.toml: programs q4="bank:q4", q7="bank:q7"; cases "q4 ran accepted", "q7 ran accepted".
- `uv run --no-sync python tests/fixtures/verify/record.py` on the image (rewrites every recording; all
  machine-written; fixtures/README table gets q4/q7 rows).
- test_the_correct_option_is_the_machines_output: for q4/q7 (and q3/q8 where recorded) replay verify on
  StubRunner(RECORDINGS) and derive correct_index from `with_record(question, verdict)`; assert it equals the
  bank's correct_index and that the replayed stdout equals the bank record's stdout.
- Mutation: change a q4 option text / swap bank stdout in a copy -> fails; show by editing the replay result.

## (5) test_strict_fails_on_any_flag
- Today: real bank has no flags; strict=1 only via warned=['tell: option position'] (audit.py:1059).
- Rewrite: copy bank, make q7 option 48 chars (not in reserve, as :744 does); assert q7 flagged, non-strict 0,
  strict 1, and exit_status({**report, "warned": []}, strict=True)==1 while the unflagged copy gives 0.
- Mutation: remove `or any(q["flags"] ...)` from exit_status (audit.py:1478) -> fails.

## NITs
- migrate_mvp.py :521/:528 exists(): guard should refuse an unreadable existing file (raise MigrationError)
  rather than keep or overwrite it — append-only like bank.append_question, but loud. Small; test it.
- q8 trailing newline: in q8 review.reason + README.
- q7 explains.legacy names bob/amy/cal/dan: frozen verbatim by test_the_mvp_explanation_is_kept_verbatim
  (AC-73); not a one-line fix; already disclosed in review.reason. Note only.

## Gates
just test-pipeline / test-room / test-web; test-verify-full once; CI test + test-full on PR head.
Contract tension: none — SPEC replaces legacy whole on re-verify; tests that pinned q8 legacy were
describing state, not contract.
