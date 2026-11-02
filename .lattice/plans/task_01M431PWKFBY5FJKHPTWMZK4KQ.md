# PQ-49: Run just bank-audit in CI

From the Result Validator report (Drift, smaller items): CI runs just test but never just bank-audit, so the audit's checks over the real bank reach CI only through pytest's fixtures. Add a CI step that runs just bank-audit on the committed bank and fails the job on a FAIL (a WARN passes, as the recipe does). The report is organizer-only and gitignored; do not upload it as an artifact (AC-25). Touches .github/workflows/** (a serialized shared file: ask before editing). Acceptance: the step runs on a PR and on main, green on main @ the merge of this ticket; a planted fifth record written answer-last in a throwaway branch turns it red (name the mutation). HELD: minted from the audit; released on the client's word.

# Plan (delegator, 2026-11-02)

**Outcome.** A PR's `just test` job goes red when `just bank-audit` FAILs over the committed bank, and shows only the verdict in the log.

**Baseline (ruling 1).** `just bank-audit` on 622e0fd: exit 0, `bank-audit: passed`, no FAIL or WARN lines.

**The step (rulings 2, 4).** One named step, "Audit the committed bank", in the `test` job of `.github/workflows/ci.yml`, directly after `- run: just test`, so it is outside the recipe and its 60 s budget. Triggers untouched.

**The log (ruling 3).** The recipe's stdout carries organizer-only detail: each tell's worst rule and hit rate ("pick does_not_compile: 1 hit vs 0.80"), the generator's letter counts, and on a finding the ids of UB answers and flagged questions. So all of it goes to `$RUNNER_TEMP/bank-audit.log`, which is not uploaded, and the step prints a fixed line chosen from the exit status. It does not tail the log: a traceback's last line could carry anything. The report under `bank/audit/` stays gitignored and is not uploaded.

**Exit status.** `if just bank-audit > log 2>&1; then echo passed; else status=$?; echo FAILED; exit "$status"; fi`. No `|| true`, no pipeline. WARN exits 0 in the audit, so it passes.

**Mutation (ruling 5).** Plant in a throwaway copy under `$TMPDIR`, never in this tree: first the fifth answer-last record the ticket names. It may no longer FAIL since PQ-45 (the position tell reads the wall's arrangement, not stored order). If it passes, find the smallest plant that FAILs, without touching the audit. Run the step's exact script on both, with a WARN case if one comes cheaply.

**Docs.** `pipeline/README.md` and `docs/RUNBOOK.md` say nothing about CI and the audit, so no edit is due.

**Tests pinning ci.yml.** `popquiz/tests/harness_config.rs` (test-full's timeout, `--test harness_rules` in `test`) and `popquiz/tests/auth.rs` (a name scan). Adding a step changes neither pinned fact, so there is nothing to ask.

**Limit (ruling 6).** The red run on the real step is not shown in CI; the local run of the same script is the evidence.
