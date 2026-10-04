REVIEWING PQ-39 CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/bank-followups HEAD b313cb28b162ba48b8d74ed742afcd9006a0c558 BASE 27308068fbd9a251113d5756293a96c94a693ca6

Verdict: PASS-WITH-NITS

Scope reviewed: `git diff 2730806 b313cb2` (22 files, four commits 7e6353f, 0c3d2be, ca78693, b313cb2, all signed `G`). `git rev-parse HEAD` in the worktree equals the pinned HEAD. Read-only: nothing in the worktree was written (`git status` after my runs shows only the pre-existing untracked `.claude/`; `pipeline/.pytest_cache` is the delegator's, mtime 19:41). All mutations ran on a throwaway copy under `$TMPDIR/mut`, never the worktree.

## Findings

### Critical

None. No hand-typed stdout, verdict or record found (details under "Verified clean").

### Major

None.

I considered whether the hashmap-order recording change is a Major under the brief's "any recorded output changed" rule and concluded it is not; see Minor 2.

### Minor

1. **bank/README.md:59-64, :76-82, :371-375 still say things that are false today.** The ticket's item 2 was "corrected to the truth after PR #40 and this change", and the plan fenced it with "Nothing else". The targeted passages (:125-129, :135-140, :158-163, :258-259) are correct, but:
   - :371 "As of this ticket, q4, q7 and q8 are flagged for option length, as the table above says." is false. The branch's own new test (`test_audit.py:767`, docstring and first assertion) proves no bank question carries a flag since D-15's re-authoring, and the delegator's validation says the same of `run_audit` on the real bank.
   - :59-64 (present-tense "Here?" column: q4, q7 and q8 "yes, needs work", "4 of 5 options are two lines") and :76-82 ("That is q4 and q7. q8's too-long option is a distractor ... only that one option needs replacing") are T-14-era findings that PR #36 resolved.
   - Recommendation: reword :371-375 (the option-position warning sentence after it is still true) and either date-stamp the :59-64 table as "at T-14" or update it; or say in the PR body that these are left as T-14 history on purpose.

2. **hashmap-order's recorded stdout changed and 7e6353f does not say so.** `pipeline/tests/fixtures/verify/recordings/hashmap-order.json` run:1 to run:5 are five fresh random HashMap iteration orders (all five still distinct). It is the only recording outside q4/q7 whose non-timing content changed. This is inherent: `record.py` rewrites every recording, and that case exists to show output varying. The verdict is unchanged (`output_varied`, distinct outputs > 1), and nothing pins the values: `test_verify.py:130-138` computes `distinct` from the recording itself, and the Rust twins test reads only q3, panics, ub-both and q8. The 7e6353f message says "re-records every case ... every case gave the verdict cases.toml names" but not that the other ten files changed only in `recorded_at`, `wall_clock_s`, and (hashmap-order) the random draw. Recommendation: put that one sentence in the PR body so the 10-file churn does not read as unexplained.

### NIT

1. **bank/questions/q8.json:15** `why_tempting` changed `—` to a literal em dash. This is the machine rewrite normalizing through `bank._write_json` (`ensure_ascii=False`), so JSON-equal and benign, but 0c3d2be says "only review.reason was edited by hand" and does not mention it. One clause in the PR body would do.
2. **q7 `explains.legacy` still names bob/amy/cal/dan** (ticket NIT 3). Not touched, by design: frozen by `test_the_mvp_explanation_is_kept_verbatim` (AC-73), already disclosed in q7's `review.reason`, and the plan says "note only". Make sure the PR body or DONE comment says it was left on purpose.
3. **pipeline/tests/test_audit.py:789** `assert not flagged[...]["in_reserve"]` (and the flag-free precondition on the clean copy) depends on q7 staying unaffirmed in the real bank. When an organizer affirms q7 the copy is in the reserve and this test fails on a legitimate state change. The sibling test at :744 sets `review` explicitly. Recommendation: have the strict test clear `status` and the `affirmed_*` fields in its copy.
4. **pipeline/src/popquiz/migrate_mvp.py:556-576** `main()` does not catch the new `MigrationError`, so a damaged bank file gives a traceback (non-zero exit, message legible). The `migrate()` docstring at :509-512 says a file in the bank "is kept, never overwritten" and does not mention the new refuse-if-unreadable branch (it is in `_already_held`'s docstring). Cosmetic.
5. **pipeline/tests/test_migration.py:42-44** imports `question_to_dict`, `quoted_outputs`, `question_path` out of alphabetical order, though b313cb2 says it "sorts an import". No lint gate exists (no ruff config, no justfile recipe), so cosmetic.
6. `.claude/` (boot-prompt.md, settings.json, .cc-writes) is untracked in the worktree. Not in the branch; just do not `git add -A`.

## Verified clean

- **Machine-written, q8 record (the main check).** `bank/questions/q8.json` loaded with `bank.question_from_dict` and re-serialized with `bank.question_to_dict` plus `json.dumps(indent=2, ensure_ascii=False) + "\n"` is **byte-identical** to the committed file, so field names, order and formats are exactly what `bank._write_json` (the call at `verify.py:693`) emits. Field order: rustc, edition, target_triple, flags, compile_error_code, verified_at, verifier_version.
  - `rustc` is the full `-Vv` text (no trailing newline, as `parse_toolchain` joins it); release 1.98.1 and commit-hash 48a229ce... equal `pin.toml`; host aarch64-unknown-linux-gnu is in `target_triples`; flags equal the `[pin.flags]` values.
  - `verifier_version` "popquiz-verify 7b1bc351e4dd" equals `verify.verifier_version()` recomputed over `verify.py`, `runner.py` and `sandbox.py` at HEAD, and none of those three is in the diff. `verified_at` "2026-10-03T23:38:21Z" has the `_utc_now` format and falls between the ticket's in_progress event (22:28Z) and the commits (23:45Z).
  - `verify.is_stale(record, pin)` is False; `expect_from_record` gives `does_not_compile`; `correct_index` is 4 (the `does_not_compile` option). The record has none of runs, stdout, exit_code or miri, as SPEC 3.2 and D-22 require of a complete does-not-compile record.
  - `review.reason` is the only authored field touched, and its claims check out (6cac9ec did strip the trailing newline from q8's source: the `git show 6cac9ec` diff shows it).
- **Replay check, field by field.** `verify.verify(question, StubRunner(RECORDINGS), expect=expect_from_record(...))` over the new recordings returns an accepted verdict whose record equals the bank's record on **every field except `verified_at`** for q4, q7 and q8 (including `verifier_version` and the full `-Vv`). q4 and q7 were verified on 2026-09-30 in PR #36 and not touched here, so two independent machine runs agree. q3 differs only where it is legacy, as it should.
- **q4.json and q7.json recordings.** Format matches `record.py` exactly (`_recorded` block, `_source_sha256`, `toolchain`, `compile`, `run:1`..`run:5`, `miri:stacked_borrows:0`, `recorded_from`, `wall_clock_s`). `_source_sha256` equals the sha256 of the bank source for all 12 recordings. `recorded_at` is the same instant (2026-10-03T23:39:18Z) in all 12 files, consistent with one `record.py` run, 57 s after the q8 verify. Unpinned sanity: local rustc 1.96.1 (macOS, not the verifier) prints `-3 -1 | -4 1` for q4 and `['a', 'd', 'b', 'c']` for q7 and refuses q8 with E0502. That corroborates the recordings but is not verification.
- **Re-recording of the other ten files.** Compared old (2730806) with new (b313cb2) with `recorded_at` and `wall_clock_s` removed. Identical for loops-forever, miri-differs, miri-unsupported, panics, q3, q8, syntax-error, ub-both, ub-sb-only: same keys, same image, host, rustc release, commit-hash, Miri version, `_source_sha256`, every `stdout`, `stderr`, `exit_code`, `stopped_by`. Only hashmap-order's five stdout strings differ (Minor 2). No verdict changed: every case still gives the verdict `cases.toml` names, and the whole non-sandbox suite is green. The fixtures README now says "All twelve ... 2026-10-03", which matches the files (the old "2026-09-26" was already stale against 2026-09-30).
- **Guardrails.** Only allowed paths changed (bank/README.md, bank/questions/q8.json, bank/fixtures/receipts/q8-does-not-compile-legacy.json, pipeline/tests/**, migrate_mvp.py). justfile, pyproject.toml, uv.lock, .github, SPEC, mvp/, `bank/history.json`, `mvp/answer-history.json` are untouched. The receipt fixture's only change is its `_note`; `verified` and `expected_lines` are as before. The new tests build their seeded history, damaged files and flag in `tmp_path` copies. No test or doc adds a typed program output. Miri wording is unchanged (AC-43). q3 stays legacy, so `test_a_legacy_record_is_exempt_from_the_stale_check` reduced to q3 still covers the exemption (the early return is kind-independent).
- **Tests.** `cd pipeline && .venv/bin/pytest --ignore=tests/sandbox -p no:cacheprovider`: **642 passed** at HEAD (matches the delegator's count). (`uv run` itself is blocked in my sandbox by the uv cache path, so I used the venv's pytest directly; same interpreter and lockfile.)
- **Mutations, run on a copy, each restored from the worktree afterwards:**

  | Mutation | Item | Result |
  |---|---|---|
  | migrate rewrites the empty history unconditionally (`if True:`) | 3 | `test_a_rerun_leaves_the_bank_as_it_is`, partial-bank and refuse-unreadable all FAIL |
  | migrate rewrites history silently (no `written` entry) | 3 | **old** rerun test PASSES (blind); new tests FAIL (grown history emptied) |
  | question guard removed (`if False:`) | 3 | rerun, partial-bank, refuse-unreadable FAIL |
  | hoisted "any file missing rewrites all four" | 3 | `test_a_rerun_over_a_partial_bank_writes_only_what_is_missing` FAILS (3 extra writes) |
  | `_already_held` reduced to `path.exists()` | NIT | `test_a_rerun_refuses_a_bank_file_it_cannot_read` FAILS (DID NOT RAISE) |
  | self-consistent hand edit in q4.json (stdout and the matching option text changed together) | 4 | `check_provenance` ACCEPTS it; `test_the_correct_option_is_the_machines_output[q4]` FAILS |
  | q4 recording's stdout edited everywhere | 4 | same test FAILS |
  | q4 bank source edited, recording kept | 4 | `test_verify.py::test_every_program_has_a_recording_of_its_current_source[q4]` FAILS ("re-record") |
  | `exit_status` strict ignores flags | 5 | `test_strict_fails_on_any_flag` FAILS (`assert 0 == 1`); the **old** strict test PASSES (blind, passed through the option-position warning) |
  | `exit_status` strict ignores warnings but fails on flags | 5 | new test still passes, as it should (it sets warnings aside) |

  One caveat from my own run: a first "partial" mutation that evaluated the all-exist check lazily survived, because q3 is written before q4 is examined. That was my mutation's fault, not the test's; the hoisted version is caught. Note the old rerun test did catch an *unconditional* history write via `migrate(...) == []`, so "blind" is true only for a silent empty-shape rewrite; the new test closes that.
- **Ticket coverage.** (1) q8 re-verified, nothing keeps a does-not-compile record legacy (`verify.py:326-345`), the plan's D-22 reasoning holds. (2) README :125-129, :135-140, :142-165, :258-259 are true against the code (Minor 1 lists what is left). (3) done, (4) done, (5) done. NITs: exists() guard done, trailing-newline note done (q8 `review.reason` and README), q7 legacy names deliberately not changed (NIT 2).
- **Other consumers.** `room/tests/twins.rs:216-252` reads recordings q3, panics, ub-both and q8, and every bank question's `correct()`. Those four recordings are unchanged in stdout, exit code and steps, and the twins file's own `Q8` constant has the same shape as the bank's new q8 record. Nothing under `room/` or `web/` reads q8's legacy flag (`room/tests/take_home.rs` and `common/mod.rs` use q3 only).

## Not verifiable by reading

- That the recordings and the q8 record came from the Docker image and not from typing. By inspection a faithful fake would look identical. What I could do is listed under the machine-written bullets (byte-identical serializer round trip, replay equality against independent 2026-09-30 records, digest equality, coherent timestamps, an unpinned local rustc agreeing). Closing it fully needs `just test-verify-full` on the image.
- `just test-verify-full`, `just test-full`, `test-room` (cargo), `test-web` and CI on the exact PR head: not run here (no Docker, no cargo target writes). The delegator's validation artifact reports 18 passed for test-verify-full and `just test` rc 0; I did not rely on it.
- `tests/sandbox/*` (excluded per the brief).
- Whether the image tag's digest (`25a914853547`) is the T-15a image built from today's `pin.toml`; only the recorded strings are visible.
