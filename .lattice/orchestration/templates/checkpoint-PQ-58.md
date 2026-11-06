# PQ-58 checkpoint (uncommitted) — 2026-11-06, session 3: GAP-6 implemented

- Worktree: /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-58-provenance-followups
- Branch ai-c11-cc/pq-58-provenance-followups, HEAD e689a17 (all signed), base origin/main c810d0d (unchanged at ~08:35Z). Not pushed; upstream unset; no PR.
- Lattice: PQ-58 in_progress. Plan: /Users/michellerojas/rust-nyc-pop-quiz/.lattice/plans/task_01M461K1EZJE7SWYCVEMC37P2Z.md

## Commits
- bde3d25 GAP-5: stale-pin refusal (see earlier sessions; sentence quoted in plan).
- 7dc6e3f GAP-6 code + tests: runner.RecordingRunner; verify.{RECORDINGS, source_sha256, record_sha256, recordings_dir, write_recording}, verify.main --recordings (default <candidate's bank>/recordings), writes the recording after the record; schedule._recording_refusal (after check_question), schedule.replay + --replay flag, seam schedule._sandbox_runner; fixtures.verify.verify_cases.verify_bank; test-full q8 replay in tests/sandbox/verify/test_verify_full.py.
- e689a17 docs: SPEC 3.2 legacy edit + Recording paragraph, SPEC 7.2 paragraph, EVALUATION AC-7 test clause (all verbatim from Session 2 rulings); bank/README.md, pipeline/README.md, one RUNBOOK line.

## Organizer sentences (new)
- `{qid} has no recording - re-verify it with \`just verify\` before it is scheduled (AC-7)` (approved)
- `{qid} does not match its recording (<its source | its verified record> changed after \`just verify\` wrote it) - re-verify …`
- `{qid} does not match its recording (replayed through the stub runner, it gives a different <fields> | it is rejected: <reason>) - re-verify …`
- `{qid}'s recording cannot be read (<Error>) - re-verify …`; `{qid}'s recording does not replay (<why>) - re-verify …`
- `{qid} was not replayed: \`--replay\` needs the sandbox (<why>); run \`just sandbox-build\`, or schedule without \`--replay\``
- `{qid} does not match a new run in the sandbox (a different <fields> | it is rejected: <reason>) - re-verify …`

## Verification
- `just test` rc 0 at e689a17's tree, warm 28 s (sandbox off): pipeline 1274 passed 1 skipped (+17 new), web 306 pass 0 fail, cargo 37 result blocks ok, 0 FAILED. First run failed only web/test/runbook.test.js ("every repository path the runbook names exists": RUNBOOK named `bank/recordings/`, which does not exist until `just verify` runs); RUNBOOK line reworded, rerun green.
- New tests RED before the code (no recording API: AttributeError), GREEN after.
- NOT run: `just test-full` (needs Docker; the q8 real replay is unrun locally, CI's test-full job will run it). No real `just verify` or `just schedule` run (never against the bank/room).

## Deviations (for DONE)
1. The recording check sits AFTER check_question, not before as the plan said, so every existing refusal reads as it did (test_ac7_the_reason_is_the_checks_own_words relies on it). Stale check unchanged and still before.
2. `--replay` does not compare which of the pin's supported hosts the run was on (`target_triple`, the `host:` line of `rustc`): pin.toml leaves platform unpinned, CI test-full is x86_64, q8's record is aarch64. The stub replay (part 1) compares every field except verified_at and verifier_version.
3. The flag name, not ruled: `--replay` (plan's recommendation).
4. Test banks (test_schedule `bank`, test_review `bank_dir`) are re-verified through the verifier over the fixture recordings by verify_bank, with fixed verified_at 2026-10-01T00:00:00Z so re-verify tests still see the time move; questions with no fixture recording (review's `tropes` etc.) are left as they are.
5. test_ac6_a_legacy_record_is_not_refused_for_its_pin now asserts the legacy record is refused for no recording and not for its pin (Session 2 ruling: legacy refused too).

## Open question for the client (posted)
- `popquiz review edit` re-verifies through verify.main in a scratch bank and copies only the question back, so the scratch recording is lost: after a review edit the question is refused ("does not match its recording" / "no recording") until `just verify` runs on it. Fails closed. Fixing it is ~3 lines in pipeline/src/popquiz/review.py (`_verify_into_bank`: copy the scratch recording into <bank>/recordings), which is not a cleared file. Ask: clear review.py for that, or leave it (record in DONE).

## Reported, not fixed (for DONE)
- reserve_lines (audit.in_reserve) counts a stale or recording-less record as ready; fallback.bake and `just demo` (verify --check) check neither the pin nor the recording.
- ruff not installed in the pipeline venv.
- After merge: q3, q4, q7, q8 are all refused until the client runs `just verify` on each (Docker); the reserve schedules nothing until then.

## Rulings (client, in panel, UTC)
- 2026-11-06T07:41Z: "gap 6 - recording-id plus true replay behind feature flag, plan first and show me, gap 7 - a" (GAP-7 A: no code).
- 2026-11-06T07:55Z: "approve, 2a, 3deny untilk reverified 4 yes".
- 2026-11-06 ~08:05-08:27Z (board 08:29:29Z): "refuse legacy too", "reverify all qestions, emoty reserve", "both"; wording approved, applied verbatim in e689a17.
- 2026-11-06 ~08:36Z: "go" on this session's plan (implement GAP-6, stop at commits + green just test + checkpoint).
- 2026-11-06 (after ~09:00Z): "RESUME PQ-58 review-py yes" → review.py CLEARED for one change: `_verify_into_bank` copies the scratch `recordings/<id>.json` into `<bank_dir>/recordings/` alongside the question (only on accept). RED test first in tests/test_review.py: after `review edit` that re-verifies, `schedule.refusal(q, recordings=bank_dir/"recordings") is None` and the recording's `_record_sha256` matches. Posted to board + mail.

## Session 4 (2026-11-06, after 09:10Z)
- 6665708 review.py: `_verify_into_bank` copies the scratch recording into <bank>/recordings with the question, only on accept. Tests in tests/test_review.py: gap6 edit + gap6 reverify RED before (bank kept the old recording: _record_sha256 mismatch), GREEN after; gap6 refused-edit-keeps-recording guard (passes before and after). test_review + test_schedule rc 0.
- Codex review pass 1: `codex review --base` rejects a custom prompt (exit 2, no review); rerun as `codex exec -s read-only -C <wt> -o <scratchpad>/pq58-codex-review.txt`. Output lives in the session scratchpad, not $TMPDIR (the bypassed shell's TMPDIR differs from the sandbox's).
- 09:16Z: that `codex exec` (pid 36006, reviewing HEAD 6665708) still running; final message lands in /private/tmp/claude-501/-Users-michellerojas-rust-nyc-pop-quiz-worktrees-pq-58-provenance-followups/0b6effab-219e-4726-9091-e68437008ebc/scratchpad/pq58-codex-review.txt, log beside it. Every background wait this session (bypassed bash, two Monitors) ended within ~30 s without its wait completing, so nothing is watching it. A third Monitor did emit per-minute lines, then said "review file written" at ~09:18Z while the file did not exist and the log it reported (577 KB) was larger than the real one (536 KB): treat Monitor events about this scratchpad as unreliable; check the file with a bypassed `ls` instead. Session stopped at the worktree session cap's first warning (60 calls).
- `just test` not yet run on 6665708: PQ-80 was running `just test` at 09:15Z (one heavy gate at a time).

## Codex review pass 1 of 2 (reviewed HEAD 6665708): FAIL
Full text: /private/tmp/claude-501/-Users-michellerojas-rust-nyc-pop-quiz-worktrees-pq-58-provenance-followups/0b6effab-219e-4726-9091-e68437008ebc/scratchpad/pq58-codex-review.txt (copy it into the attach; scratchpad is session-scoped).
- MAJOR 1 verify.py:762 + justfile:367: `just demo` runs `verify --check` (internal provenance only) then uploads, so a stale or recording-less record reaches the room that way. NEEDS MICHELLE: justfile is not cleared. Options: (a) add the pin + recording checks to `verify --check` (verify.py is cleared under GAP-6; justfile unchanged); (b) leave it, record in DONE. Recommend (a).
- MAJOR 2 schedule.py:235: replay compares the recording's toolchain banner against the current pin's Miri version even when Miri never ran (compile-failure q8), so a Miri-only pin bump refuses a record `is_stale` correctly accepts. Fix in cleared schedule.py, RED test first.
- Minor schedule.py:605: reserve_lines/in_reserve report 4 ready while refusal refuses all 4 (already in "Reported, not fixed").
- Minor schedule.py:236: a recording missing a step field (e.g. compile.exit_code) raises KeyError from StubRunner, a traceback instead of the refusal. Fix: catch it as "does not replay".
- NIT review.py:423: verifier output names the deleted scratch recording path; rewrite it to the bank path like the candidate path.
- Confirmed by Codex: no existing sentence or copy.rs/copy.js string changed; all files cleared; fallback.bake checks neither but only runs behind the scheduler gate.
- Lesson: background-task notices here arrive LATE and out of order (the "completed" notices at 09:14-09:16Z were for earlier tasks), not early. Wait for the notice naming the task you are waiting on.

## Ruling, session 4
- 2026-11-06 (~09:25Z): "RESUME PQ-58 demo a" → MAJOR 1 option (a): `verify --check` (verify.py `_check_file`) also refuses a stale-pin record and a missing/mismatched recording, with the same sentences `schedule.refusal` uses; justfile unchanged. Posted to board + mail.

## Next session (session 5): the fixes, then re-review
1. RED tests first, then fix: Major 1 (a) in verify.py `_check_file` (+ a test that `verify --check` refuses a stale record and a recording-less one, and passes a matching one); Major 2 in schedule.py ~:235 (a compile-failure record, Miri never ran, survives a Miri-only pin bump in the stub replay; a Miri-ran record is still refused); the Minor KeyError at schedule.py ~:236 (malformed recording → "does not replay" refusal, no traceback); the NIT at review.py ~:423 (rewrite the scratch recording path to the bank path). Leave the reserve-count Minor reported.
2. Update pipeline/README.md and docs/RUNBOOK.md only if they describe `just demo`'s checks (one line each at most). Commit signed, by concern.
3. Codex pass 2 of 2: `codex exec -s read-only -C <wt> -o <scratchpad>/pq58-codex-review-2.txt -` with the pass-1 findings and fixes in the prompt. Do NOT use `codex review --base` with a prompt (exit 2). Wait for the notice naming that task id. If blockers remain after pass 2, stop and report.
4. Validate, PR, DONE as below.

## Session 5 (2026-11-06, ~10:00Z)
- Commits (signed): 0c75be7 replay fixes (Major 2: stub replay uses the recording's Miri when verified.miri is None; Minor: KeyError/TypeError/AttributeError -> "{qid}'s recording does not replay (a step is malformed: ...)"); 41dd64a Major 1 (a): schedule.record_refusal (pin, check_question, recording; refusal calls it after the reserve checks) + verify._check_file runs it (function-local import), --check help text, bank/README.md:152 one line; 0fa5773 NIT review.py names the bank's recording path.
- RED before / GREEN after: 4 test_verify --check tests (+ bank-question parity test replacing "passes every bank question": the real bank now gets the scheduler's answer, i.e. refused for no recording), 3 test_schedule tests (Miri-ran guard passes both ways), 1 test_review test. test_verify + test_schedule + test_review rc 0 (309 tests).
- Codex pass 2 of 2 launched in background (task b2lg4evlc), prompt + output in /private/tmp/claude-501/-Users-michellerojas-rust-nyc-pop-quiz-worktrees-pq-58-provenance-followups/3766e644-e0ca-4e63-ab00-3a5187c2bea7/scratchpad/pq58-codex-{prompt-2.md,review-2.txt,review-2.log}.
- `just test` rc 0 at HEAD 0fa5773 (sandbox off, warm real 49.3 s): pipeline 1284 passed 1 skipped (pre-existing ProseWarning q3 hint), web 306 pass 0 fail, cargo 37 "test result: ok" blocks, 0 FAILED. Log: <scratchpad above>/just-test.log.
- Session stopped at the worktree session cap's first warning (60 calls) while Codex pass 2 (b2lg4evlc) was still running.

## Next session (session 6)
1. Read <scratchpad above>/pq58-codex-review-2.txt (if missing: check with a bypassed `pgrep -fl codex`; if no codex process, rerun the same `codex exec ... - < pq58-codex-prompt-2.md` once, since pass 2 never reported). This is pass 2 of 2: if it is FAIL with Critical/Major, STOP and report to Michelle (no third pass). If PASS / PASS-WITH-NITS, attach: `lattice attach PQ-58 --type note --role review --title "Code review (Codex)" --inline "<verdict, reviewed HEAD 0fa5773>" --actor agent:delegator-pq58-reviewer`, then attach Validation (the just test line above) serially.
2. Re-read `lattice comments PQ-58`; `git fetch origin`; if origin/main moved (PQ-61 lands first), `git rebase origin/main` (never pushed yet, so rebase is allowed) and rerun `just test`.
3. Ask Michelle to approve: `git push -u origin ai-c11-cc/pq-58-provenance-followups`, then `gh pr create` (REST body, verify with `gh pr view --json body`), gh-wait-runs, attach PR, status review, DONE (comment + mail).
- New consequence for DONE: `just demo` (default q3) now refuses every committed question until q3/q4/q7/q8 are re-verified.
- RUNBOOK and popquiz/README.md don't describe demo's record checks: unchanged. justfile comment at :366 still says "The provenance check (AC-7)": justfile not cleared, reported.

## Session 6 (2026-11-06, ~10:10Z)
- Codex pass 2 of 2 (reviewed 0fa5773): PASS-WITH-NITS, no Critical/Major; one Minor = the deferred reserve-count (reserve_lines). Text: /private/tmp/claude-501/-Users-michellerojas-rust-nyc-pop-quiz-worktrees-pq-58-provenance-followups/3766e644-e0ca-4e63-ab00-3a5187c2bea7/scratchpad/pq58-codex-review-2.txt
- Attached: art_01M6R5CHFH81NPF3VNWYVFPGCJ "Code review (Codex)", art_01M6R5CR5524MHSGZTWAM067FY "Validation".
- origin/main still c810d0d (PQ-61 not landed): no rebase, no new just test needed. Board has no new orchestrator instruction. Branch = 7 signed PQ-58 commits, 14 files.
- WAITING on Michelle: approve `git push -u origin ai-c11-cc/pq-58-provenance-followups`, then `gh pr create`.
- Michelle: "go" (~10:15Z). Pushed, remote head = 0fa57735a3f62d6307c270d32227b6b4d1b91d04. PR #135 https://github.com/mrojas54/rust-nyc-pop-quiz/pull/135 opened via REST, body verified, OPEN MERGEABLE.
- Orchestrator: MAIN MOVED (PQ-61 #134 merged, origin/main 2e6e20c), then CORRECTION: branch already pushed, do NOT rebase. Left as is; merge-tree clean against 2e6e20c.
- CI at 0fa5773: just test SUCCESS, just test-full SUCCESS. PR attached (art_01M6RAT0EXDJFMR2CFQB2F56AY), PQ-58 status review (verified), DONE posted (board + mail). Panel description "PR #135 open, waiting". RUN COMPLETE; nothing pending but the client's merge.
- (superseded) gh-wait-runs running in background (task bs9ula24r). Remaining: CI result; then serially `lattice attach PQ-58 <PR URL> --type reference --title "PR"`, `lattice status PQ-58 review`, verify `lattice show PQ-58 --json`; re-read comments; DONE (comment + mail); set description "PR open, waiting".

## Older resume line
Resume: `Read .claude/boot-prompt.md and .claude/checkpoint-pq58.md, then continue PQ-58 from the boot arc's "Code review" step.` review.py change is DONE (6665708). Read the Codex review file above (if missing and pid 36006 is gone, rerun the same `codex exec` once). Then fix Critical/Major, at most one re-review of origin/main...HEAD (≤2 passes, save $TMPDIR/pq58-codex-review.txt, attach), Validate (just test warm + attach), PR (client approves push and gh pr create), gh-wait-runs, attach PR, status review, DONE (comment + mail).
