# PQ-61 checkpoint (delegator) — waiting on the ruling

cwd /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-61-deployed-burst-secrets
branch ai-c11-cc/pq-61-deployed-burst-secrets, HEAD b56d4d9 (one commit on origin/main c810d0d), not pushed, upstream unset.
Lattice status: in_progress. Plan: /Users/michellerojas/rust-nyc-pop-quiz/.lattice/plans/task_01M461K1VEHR6CTSJJQ71PHWPT.md

## Done
- READY, PLAN, QUESTION posted (Lattice comment and pq-orchestrator mail).
- Guard: popquiz/tests/harness_config.rs, four new tests (no existing test changed), commit b56d4d9.
- Plants, all reverted (git status clean on .github afterwards), each RED on its own test:
  - 1 ci.yml step reads secrets.POPQUIZ_ADMIN_TOKEN -> the_burst_secrets_are_named_by_no_other_workflow
  - 2 new job without environment reads the session -> the_burst_secrets_are_read_only_in_the_job_bound_to_the_environment
    (the existing the_deployed_workflow_is_confirmed_and_bound_to_an_environment stays GREEN on this plant: it compares lines by text, and the planted line copies one from the bound job)
  - 3 push trigger -> the_deployed_workflow_runs_only_by_hand
  - 4a echo of the secret expression in run, 4b jq --arg with the token, 4c printf of the token to a file -> the_burst_secrets_reach_a_step_only_through_env
- just test green. Warm wall time: 60.4 s before, 27.9 s after. Sibling load drives that difference; the new tests take under 0.01 s.
- README Burst (1184-1221) and RUNBOOK 174-181 agree with the header; no change. The only disagreeing sentence is deployed-burst.yml:6 ("repository secret"), asked in the QUESTION.

## Ruled (2026-11-06, panel 22): "RESUNE PQ-61 A and yes"
- Applied: SPEC §8.3 clause and §8.4 verbatim (reflowed to 80 cols, words unchanged) at 1e38b6d; header line 6 at 688e935. No EVALUATION line.
- just test green after (30.6 s). Next: Codex review (pass 1 of at most 2), validate, push, PR.

## Codex review pass 1 (codex exec read-only; `codex review --base` refuses a prompt): FAIL, five Majors (denylist bypasses)
- Fixed: guard rewritten to pin the scripts of the three secret-holding steps (HOLDING_STEPS), read raw lines, match names in any case, ban wholesale reads, GITHUB_ENV/GITHUB_PATH/shell changes.
- 22 plants (6 original + 16 from the review) all RED, restored clean ($TMPDIR/pq61-plants2.log). just test green, 26.9 s.
- The fix is STAGED in harness_config.rs, not committed: the signed commit failed with "1Password: failed to fill whole buffer". Waiting for Michelle to unlock and say go; then retry the commit exactly once.
- Next: Codex re-review (pass 2 of 2, the last), validate, push, PR.

## Committed 950c4e9 (signing worked after the client's second go). Codex pass 2 (the cap): FAIL, four Majors
- BASH_FUNC_printf%% env entry in a holding step; ~/.curlrc trace from an earlier step; toJSON(true && secrets) / folded multiline in other workflows; `environment: >-` block-scalar name in other workflows. Text: $TMPDIR/pq61-codex-review2.txt (bypass TMPDIR).
- Status needs_human, flag raised. Waiting for the client: (a) a third fix pass, or (b) PR as is with these as known limits. Not pushed.

## RULED: third fix pass (client: "9a0", read as "(a)"), 2026-11-06. RESUME HERE in a fresh session.
Read .claude/boot-prompt.md section 1 and 1a for the rules (bypass for c11, git, lattice, commit; stage by name; never --no-gpg-sign). Status in_progress, flag lowered.
Fix plan in popquiz/tests/harness_config.rs (new code only; no existing test changes):
1. Pin every step of the deployed-burst job, not only the three holding steps: each step's keys, env entries and run script must equal an expected table (extend HOLDING_STEPS to all steps, or pin the job's raw lines from `  deployed-burst:` to the end). Covers BASH_FUNC_* env entries and an earlier step writing ~/.curlrc.
2. In every workflow: every `${{ }}` expression whose squashed text contains `secrets` must be one of an allowlist (`secrets.FLY_API_TOKEN` in deploy.yml as `FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}`; the four burst forms in deployed-burst.yml). Covers toJSON(true && secrets).
3. In every workflow: refuse an expression split over lines, and refuse a block-scalar indicator (`|`, `>` and chomping variants) as the value of `environment:`, `name:` under environment, or any key whose next lines hold `${{`. Covers `environment: >-` and folded expressions.
4. Plants: rerun $TMPDIR/pq61-plants2.py (22) plus the four new ones (BASH_FUNC_printf%% env entry; a step writing ~/.curlrc before scheduling; DUMP: ${{ toJSON(true && secrets) }} in ci.yml; `environment: >-` + `deployed-burst` in ci.yml; also a folded `>-` toJSON over two lines). Bounded at 15 min, reverted.
5. just test; commit (signed); then Codex review pass 3 via `codex exec -s read-only -C <wt> -o $TMPDIR/pq61-codex-review3.txt -` (codex review --base refuses a prompt). Attach the verdict. If pass 3 still fails, stop and report; do not loop.
6. Then Validate, push (client approves), PR per boot section 3, gh-wait-runs, attach PR, status review, DONE (comment + mail).
Contract notes for DONE: the EVALUATION line was not applied; the existing the_deployed_workflow_is_confirmed_and_bound_to_an_environment compares by line text and misses a copied line (left unchanged); binaries and third-party actions are outside the guard.

## Third fix pass (fresh session, 2026-11-06), in progress, uncommitted
- harness_config.rs: HOLDING_STEPS and step_spans removed; new BURST_PIN (include_str of popquiz/tests/fixtures/deployed-burst.pinned.yml, a byte copy of the workflow) and SECRET_READS (five lines). New tests: the_deployed_workflow_is_the_one_checked_against_the_contract (whole-file pin), every_workflow_reads_secrets_only_on_the_listed_lines (allowlist, one-line expressions, no backslash outside a script, block scalars only `run: |`/`path: |`).
- cargo test --test harness_config: 11 passed on the real workflows.
- Plants: scratchpad pq61-plants3.py (the 22 old ones plus N1..N8), log pq61-plants3.log in this session's scratchpad.
- Plants: all thirty RED, workflows restored clean. just test green, 28 s warm (cargo 379, web 306, pipeline 1252 passed, 1 skipped). One fmt fix before that.
- Committed 7001097 (signed). Codex pass 3 running in background: output /var/folders/gb/0dxb2vbx6ls104m2g7c82s_00000gn/T/pq61-codex-review3.txt (bypass TMPDIR). FAIL means stop and report; PASS means attach, Validate, ask for the push, PR.
- Client ruled "a": reran pass 3 with neutral wording. Result FAIL at 7001097 (text: bypass TMPDIR pq61-codex-review3.txt; attached to Lattice). Majors: in_block() opens a block on a comment line ending ': |' (escape check skipped, \u escapes in another workflow); flow-style job with environment: deployed-burst in another workflow. Minors: SECRET_READS is text-only, not placement; false failures on restore-keys: |, a backslash in a one-line run:, 'run: | # comment'. NIT README:1200.
- BLOCKED again, needs_human, flag raised. Options put to the client: (a) another line-shaped pass (refuse flow collections, skip comments in in_block, allow restore-keys/one-line run), (b) YAML parser dev-dependency (cargo add, approval), (c) PR as is with these as contract notes. HEAD 7001097, not pushed.
- (superseded) BLOCKED: Codex pass 3 gave no verdict. It exited 1 with "This content was flagged for possible cybersecurity risk" (log pq61-codex-log3.txt:7880 in the bypass TMPDIR; no -o file written). Status needs_human, flag raised. Waiting for the client: (a) rerun once with the boot's neutral review wording, no request for exploit text, or (b) PR with pass 3 recorded as not run. HEAD 7001097, not pushed.

## (was) Waiting for
RESUME PQ-61 A|B|C (lattice comments PQ-61 or mailbox pq61-delegator). Also whether line 6 of the workflow header may be fixed.

## On resume
- A: apply the approved SPEC §8.3 clause and §8.4 verbatim (+ EVALUATION AC-53 line if approved); update the guard's messages to cite §8.4; line-6 fix if approved. Commit, just test.
- B: switch to inputs plus header (not recommended).
- C: nothing to the contract.
- Then the Codex review (at most two passes), Validate, push (client approves), PR, gh-wait-runs, attach, status review, DONE.

## Restart with structural YAML policy — 2026-11-06
- User: "restart and come up with more robust solution". Replaced the line scanner; dependency addition and signed commits authorized through tool approval.
- CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-61-deployed-burst-secrets; branch ai-c11-cc/pq-61-deployed-burst-secrets; HEAD 385d7fc.
- RED f11f8b5: six negative regression groups fail against placeholder; two positive groups pass. GREEN 385d7fc: serde_yaml_ng dev-dependency, decoded tree policy in tests/workflow_secrets.rs; burst pin retained; original harness_config.rs restored byte-for-byte to origin/main.
- All eight policy tests pass, including both review-3 cases; malformed fixtures separated from valid negative cases; semantic-negative fixtures must parse before testing rejection.
- just test passed: Rust 381, web 306, pipeline 1252 passed, one skipped. Warm 45.91 s; initial rebuild 86.33 s. No production workflow run.
- Independent review restart pass 1 running, agent 01a1b04c-b925-7711-bd2d-0d6a756218b2. At most one fix/re-review remains for this restart.
- Origin fetched, main remains c810d0d. Branch not pushed; no PR yet. Next: review result, VERIFY/DOCS commits, push and open PR, verify CI.

## Cleaner design — current state, 2026-11-06
- User: "restart and design solution with less duct tape". Current worktree:
  `/Users/michellerojas/.codex/worktrees/89a7/pq-61-deployed-burst-secrets`.
- Branch `codex/pq-61-workflow-policy`, HEAD
  `d74fbfbd69852e4e5983bc871992e17b7df6c5b5`, pushed. Original branch preserved.
- PR https://github.com/mrojas54/rust-nyc-pop-quiz/pull/134, base main c810d0d.
  PR body and head read back. Lattice PQ-61 is `review`; review and PR attached.
- Parser implementation retained; removed raw-byte equality and Fly step-index
  coupling. Compare decoded burst workflow with reviewed baseline. Harmless YAML
  presentation passes; decoded scripts and ordered steps remain protected.
- RED abca64b: both new acceptance tests failed as intended. GREEN 0e9e814.
  VERIFY 442ed20. DOCS d74fbfb. All signed. Design:
  `popquiz/tests/fixtures/workflow-secrets.md`.
- Local `just test`: Rust 384, web 306, pipeline 1252 passed / 1 skipped;
  organizer terminal tests, fmt and Clippy passed. Fresh build 117.43 s;
  warm 30.74 s. Focused policy 11 and harness 5 passed. Logs:
  `/private/tmp/pq61-policy-verify.log`, `/private/tmp/pq61-policy-warm.log`.
- Independent restart review: pass 1 PASS-WITH-NITS; all three nits fixed;
  pass 2 PASS at d74fbfb. Reviewer closed. Review cap reached, no more passes.
- CI run 42203240988: `just test` and `just test-full` SUCCESS at d74fbfb.
  PR readback: OPEN, head d74fbfb, base main, mergeStateStatus CLEAN.
  Bounded CI watch exited 0; no timers or background jobs remain for this run.
  Outcome complete: reviewed PR open. Next is the owner's PR/merge decision.
- No production workflow run or credentials read. Baseline detects execution
  changes; shell runtime, binaries, action internals, caches, and GitHub-side
  settings still require their own review. No merge authorized or attempted.
