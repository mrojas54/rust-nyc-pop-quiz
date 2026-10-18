# PQ-77 checkpoint (2026-10-18, ~16:2xZ): review cap reached with one open Major; waiting for the client

- cwd `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/room-affirmed`, branch `ai-c11-cc/room-affirmed`,
  **rebased onto `origin/main` = 7573efa** (PR #92). Not pushed. HEAD `8e4e2d6`.
- Commits (all signed `G`): `dd71bc1` code+tests, `1f4f027` docs (ADR-0005 item 9 `[x]` only),
  `1a6a5d7` room refuses a bound record not accepted at review (Codex pass-1 Major), `8e4e2d6` wording (pass-2 Minor+NIT).
- Uncommitted on purpose: `.claude/settings.json`, `.claude/boot-prompt.md`, `.claude/resume-prompt.md`, this file.
- Lattice: `needs_human` (was in_progress). Attached: "Code review (Codex)", "Validation".
- Plan (binding): `/Users/michellerojas/rust-nyc-pop-quiz/.lattice/plans/task_01M4SJWQ5ZKMGDFD7A376K5VQA.md`.

## Evidence
- `just test` after rebase: rc 0 (69 s, rebuild); after 1a6a5d7: rc 0, 125 s (pipeline 123 s; machine busy, warm time
  still to measure). Rust 332+ / web 303 / pipeline 975 passed, 1 skipped. Admin suite 22/22 after 8e4e2d6.
- reserve: `1 ready` before and after.
- Mutations (each caught; log copy in the session scratchpad `mutations.txt`):
  M1 push accepts any state → admin `g12_a_push_not_affirmed…`, `g12_a_refused_push…`, `g12_an_arranged_push…`;
  M2 stale as bound → those + twins `the_affirmation_matches_every_shared_fixture`, `an_arranged_record_is_hashed…`;
  M3 affirmed_by None → admin `ac101_the_right_token…`, used `ac92_used_is_written_at_release`;
  M4 creation check removed → admin `g12_a_room_is_never_made…`;
  M5 bank_order ignored → twins ×2 + admin `g12_an_arranged_push…`;
  M6 schedule-time check skipped → 11 test_schedule tests (ac72_*, gap24_*, g10, g12, ac105);
  M7 sync drops affirmed_by → `test_sync_writes_who_affirmed_the_question_the_room_ran`;
  M8 acceptance ignored → admin `g12_a_push_not_affirmed…`, `g12_a_room_is_never_made…`.
- Local validation (room binary on 127.0.0.1:3077): 201 / 409 ×4 / 400 as designed; real `popquiz.schedule schedule q3`
  arranged push → `scheduled: new`. Create+release covered in-process (Discord needed for the binary).

## The open question for the client (review cap: 2 passes used)
Codex pass 2 Major, pre-existing (not introduced by this PR): `answers::load` checks only that the final trace step holds
`stdout`; `bank.validate_question` (bank.py:695) refuses a `stdout` row in an earlier step. So the room would accept a
record Python refuses (only a hand-crafted one; review never produces it). Options:
 (A) fix here: mirror that check in `answers::load` + a test (small; no third review, so it ships on tests only);
 (B) defer: record in DONE as a contract note and a deferred finding; open the PR as is.
Recommended: (B) — it is structural validation outside the affirmation rule, predates this ticket, and the guardrails
forbid a third review pass.

## Client ruling 2026-10-18 ~16:3xZ: **A — fix it here.** Attempted; blocked by a new prerequisite
- The check (room refuses a `stdout` row, stripped + lower-cased, in any step but the last, as bank.py:690-700 does)
  plus test `a_stdout_row_before_the_last_step_is_refused_as_python_refuses_it` is saved as
  `.claude/pq77-stdout-check.patch` (uncommitted; `git apply` it). Tree restored to HEAD 8e4e2d6, green.
- With it applied, ~20 room tests fail (admin ac61/ac47, host_page, wall_page, canary, twins …): the shared canary
  fixture `room/tests/common/mod.rs` `planted()` (lines ~208-227) deliberately plants a `stdout` row in a middle step
  (`PLANT_MIDDLE_STDOUT`) to prove G-3's walk filter. Python would refuse that record, so the room now does too.
- Fix needs the client's word because it changes the secrecy canary: drop the middle-stdout plant from `planted()`,
  and repoint the 4 `PLANT_MIDDLE_STDOUT` assertions (common/mod.rs, host_page.rs, wall_page.rs, twins.rs) to
  "refused at load"; keep the walk filter as defence in depth. Alternative: revert to B (defer).

## Client ruling 2026-10-18 16:16Z (in this panel): **"a + canary change"** — BINDING
Do it: `git apply .claude/pq77-stdout-check.patch`; drop the middle `stdout` plant (`PLANT_MIDDLE_STDOUT`) from
`planted()` in `room/tests/common/mod.rs`; repoint its 4 assertions (common/mod.rs, host_page.rs, wall_page.rs,
twins.rs) to "a record with a stdout row before the last step is refused at load"; keep the walk filter in
`answers.rs` as defence in depth (say so in its comment). `just test` green; show the mutation (check removed →
the new admin test and the repointed canary assertions fail). One signed commit. No third Codex pass (cap);
note in DONE that this fix ships on tests only, by the client's ruling. Then the steps below.

## Session 4 (resumed ~16:2xZ): ruling done
- `ef80d29` (signed G): stdout check + canary change. Beyond the 4 named assertions, the live canary
  (`canary_question()` / `Plants::middle_stdout`, `canary_scan/mod.rs` 3 uses) carried the same plant; dropped
  too, replaced by `canary_middle_stdout_is_refused()` (deviation 8, within the ruling's intent).
- `just test` rc 0, 28 s (rust 334, web 303, pipeline 975 + 1 skipped).
- M9 (check disabled): admin `a_stdout_row_before…`, twins `the_public_half_holds_nothing_sealed`, host_page
  `ac47_…`, wall_page `the_page_is_the_same_bytes…` fail. Canary binary under M9 not shown: the auto-mode
  classifier denied the bypassed (port-binding) run with a disabled check in the tree.
- Room `cargo clippy -D warnings` fails in `room/tests/room_record.rs` (drop of non-Drop), untouched, not gated
  by `just` or CI: deferred finding.
- Codex pass 3 (reviewed ef80d29): FAIL, 1 Major (Rust trim ≠ Python strip on U+001C–1F / U+200E–F) fixed in
  `305f90f` (py_strip + `the_stdout_name_is_stripped_as_python_strips_it`; Python agrees on both names; mutation
  trim() → that test fails); `just test` rc 0, 38 s. Open Minor: walk filter no longer independently tested.
  Verdict attached (art_01M57XNWWXA95VPR9G17B2Y7C3). No pass 4.
- `just test-full` running (log `/var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq77-test-full.log`).

- test-full: all suites passed except harness-full `burst_on_loopback_at_200` (known flake, failed on the one
  rerun too); lifecycle body passed. Pushed HEAD 305f90f; **PR #93** open; CI success; ticket `review`; DONE posted.
  **Stopped. Do not merge** — the client merges or tells the Orchestrator.

## Next (after the client's answer)
1. If A: implement, `just test`. 2. `just test-full` once (`pgrep -fl "cargo|just test-full"` first).
3. `lattice status PQ-77 in_progress`; re-read comments; push (client approves), PR, CI wait, attach, `review`, DONE.
Deviations so far: the six in the plan-era list (two commits not six; Option affirmed_by; organizer-tui lock line;
creation reason inline; q4/q7/q8 not hard-coded; mutations now shown) + 7. pass-1 type-strictness Major kept by design.
