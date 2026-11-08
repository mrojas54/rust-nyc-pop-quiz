# Proposal for c11's `lattice-orchestrator` skill: closeout lessons from the Rust NYC Pop Quiz build

From the closeout audit of 2026-11-08 (`closeout-audit.md` beside this file, which has the
evidence for each finding). The installed copy is c11-managed (`.c11-skill.json`: c11 1.8.1,
`skill_version` 2, `source_sha256` `3298e5fb…94d3`), so this is a proposal for the skill's
source, not a local edit. Nothing installed was changed.

**Applying it.** The patch below is a unified diff against that installed copy, with paths
`a/lattice-orchestrator/…`. From the directory that holds `lattice-orchestrator/`:
`patch -p1 < lattice-orchestrator-closeout.patch`. Dry-run verified: it applies cleanly and
the result is byte-identical to the edited copy. Four files, +35 / −5.

| Finding (audit #) | Where |
|---|---|
| 1 Hand-rolled syntax scanners don't converge; 10 cap escalation carries severity + default | `orchestrator.md` Reviews, the review-cycle bullet |
| 2 Clear the guard tests that read the files in scope | `intake.md` Minting tickets |
| 3 An amendment greps every contract file | `SKILL.md` Autonomy levels |
| 4 One surface per validation-plan row; computed tag tally | `intake.md` Validation plan |
| 5 Bare guard form (the old one trips the classifier) | `orchestrator.md` Identity Block, Clause 1 |
| 6 Launch once, no relaunch on a parser's `||` | `orchestrator.md` Spawning |
| 7 Read input state in the same command as the send | `orchestrator.md` Reading delegators |
| 8 Re-read the thread before the PR | `orchestrator.md` Clause 19 (new) |
| 9 Read the first screen after launch (manual mode) | `orchestrator.md` Spawning |
| 11 A delegator, not a tutor; seeded `outputStyle` and boot prompt in the worktree | `orchestrator.md` Clause 20 (new), Worktree prep |
| Signing probe before a fleet | `orchestrator.md` Worktree prep |
| The run's story | `runs-ledger.md`, new top entry |

Audit finding 12 was project-specific and landed in the repo (PR #142, `d5c811b`).

**Not in the patch, noticed while editing.** Clause 11 says a 60 s `/loop` tick while the
dispatch loop says 270 s, so the two contradict each other. And `orchestrator.md`'s Reviews
section describes `lattice plan-review` / `code-review` / `review-status`, which Lattice 0.2.0
does not have. This run had no evidence on either.

## Patch

```diff
diff -ru a/lattice-orchestrator/SKILL.md b/lattice-orchestrator/SKILL.md
--- a/lattice-orchestrator/SKILL.md	2026-11-08 10:17:55
+++ b/lattice-orchestrator/SKILL.md	2026-11-08 10:18:44
@@ -67,7 +67,7 @@
 - **Moderate** *(default)* — decide-and-log routine calls; surface non-trivial architectural choices, scope expansions, and mild irreversibility for approval.
 - **Minimal** — surface at every phase transition; the operator is driving.
 
-Every autonomous decision lands in run-state's append-only decision log, tagged with the autonomy level that authorized it. An operator ruling that changes a contract criterion is amended into the contract when it is made: the terminal audit judges the text, not the decision log.
+Every autonomous decision lands in run-state's append-only decision log, tagged with the autonomy level that authorized it. An operator ruling that changes a contract criterion is amended into the contract when it is made: the terminal audit judges the text, not the decision log. The amendment greps every contract file (SPEC, EVALUATION, the stories, the copy module) for the old string or rule and changes each hit in the same change; amending one file leaves its sibling contradicting it.
 
 ## Layout (inside c11)
 
diff -ru a/lattice-orchestrator/references/intake.md b/lattice-orchestrator/references/intake.md
--- a/lattice-orchestrator/references/intake.md	2026-11-08 10:17:55
+++ b/lattice-orchestrator/references/intake.md	2026-11-08 10:18:41
@@ -42,6 +42,7 @@
 - **Fidelity:** verbose (full description, acceptance criteria by ID, "Plan: filled in by delegator's plan phase", depends-on) or minimal (one line + BUILDPLAN anchor). Either way the ticket must reference its SPEC criteria IDs — the Result Validator maps audit rows through them.
 - A shared pattern several tickets will use (a UI stack, a helper layer) has one named owner ticket; the others consume it and build no parallel module. Without an owner, parallel builders each write their own.
 - Unavoidable shared-file edits (from the BUILDPLAN) flagged in the affected tickets so dispatch serializes them; shared enumerations and replaced objects (Contract checks) name their canonical source and landing order in each writer's ticket.
+- **Clear the guard tests that read the files in scope.** A test that walks a route table, diffs a schema, holds an allowlist or reads a config helper fails on a correct change outside the ticket's cleared files, and each one costs a question-and-ruling round trip. The brief lists them and clears one-line registrations in them up front. Keep the project's map of file → guard test (in its lessons file) and add a row when a new guard appears.
 
 ## External work and delivery receipts (pilot convention)
 
@@ -98,6 +99,7 @@
   remain smoke-side; for `external-oracle` rows, name the oracle, who supplies it, and
   whether approval already covers the interaction.
 - Every criterion gets ≥1 row (a criterion too vague to row-ify goes back to the contract's author). Verification methods are concrete and reproducible — "looks correct" is not a method. Pass conditions are single-line and testable — if you can't write one, the row is a wish. The artifact column names the PR via ticket ID; the Validator resolves it.
+- One surface per row. A pass condition that joins a built and an unbuilt surface (or two ledgers, two pages) can only ever score Partial; split it so each half passes or fails on its own. The tag tally in the footer is computed from the table, never typed.
 - The Validator will not invent rows; what's written here is exactly what gets audited.
 - Operator reviews the draft: row count, coverage, sharpen or accept (default accept-as-is).
 
diff -ru a/lattice-orchestrator/references/orchestrator.md b/lattice-orchestrator/references/orchestrator.md
--- a/lattice-orchestrator/references/orchestrator.md	2026-11-08 10:17:55
+++ b/lattice-orchestrator/references/orchestrator.md	2026-11-08 10:18:31
@@ -10,14 +10,16 @@
 
 ```bash
 MY_PANEL=$(c11 identify --json | python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
-test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
+[ -n "$MY_PANEL" ] || exit 99   # FATAL: could not resolve own panel ref
 ```
 
+Guards take this bare form. A `|| { echo "…"; exit 99; }` group trips the permission classifier's "brace with quote character (expansion obfuscation)" heuristic, even under `--permission-mode auto`, and stops a fresh session on its first command.
+
 then uses `--panel "$MY_PANEL"` on every panel-scoped write. Ticket-bound roles additionally claim **before** titling — `(cd "$REPO_ROOT" && lattice claim <TICKET-ID> --surface "$MY_PANEL" --actor agent:<id>)` — because claim auto-renames the panel and the explicit title must win. Then set identity with **both** `c11 rename-panel` and `c11 set-title` (single-call propagation is unreliable), plus `set-agent` and `set-description`. `lattice unclaim` releases; claim bindings are liveness hints, not truth — they don't survive restarts.
 
 ## Standard Clauses (baked into every delegator and sub-agent prompt)
 
-1. **Worktree assertion, line 1:** `test "$(pwd)" = "<abs-worktree>" || { echo "FATAL: wrong cwd"; exit 99; }`. On mismatch, HALT — do not `cd` to the expected worktree, do not improvise; the bug is at the spawn side and downstream repair only hides it. Line 1 because that's the only point where `pwd` reflects the launch cwd unmolested.
+1. **Worktree assertion, line 1:** `[ "$(pwd)" = "<abs-worktree>" ] || exit 99   # FATAL: wrong cwd` (bare form; see the Identity Block). On mismatch, HALT — do not `cd` to the expected worktree, do not improvise; the bug is at the spawn side and downstream repair only hides it. Line 1 because that's the only point where `pwd` reflects the launch cwd unmolested.
 2. **Environment:** `export LATTICE_SPAWN_BACKEND=headless` and `export LATTICE_ROOT=<primary-checkout-root>` — never `$PWD`. A worktree carries its own `.lattice/` from its branch point; writing to that divergent board surfaces later as duplicate short IDs and unmapped tickets.
 3. **Status discipline:** bump ticket status BEFORE starting each phase; only the delegator bumps (sub-agents post a completion comment and stop); verify bumps with `lattice show --json`; re-bump after triage roundtrips. Status drift is the #1 silent-failure mode of well-meaning delegators.
 4. **Re-fetch at phase boundaries; never rebase a pushed branch.** `git fetch <remote>` and record "working against <remote>/main @ <sha>". Before the first push, rebase freely. After it, bring in `<remote>/main` (or a landed anchor) by merge only: others may already stand on the branch, and every rewrite forces each dependent to restack, then theirs.
@@ -40,6 +42,8 @@
 16. **Questions go to the parent's panel.** Every question, decision request and receipt is sent with `c11 send … && c11 send-key … enter` to the named parent panel (a refused send exits nonzero and types nothing, so the Enter never runs; do not press it by hand, and tell the operator instead of retrying blind), and the child keeps working on whatever the question does not block. A question left only on the child's own screen is never read; "idle until you answer" on its own screen is this failure, however clearly the brief said otherwise.
 17. **The CI window (shared runner).** A branch push without a PR costs no CI; opening or updating a PR starts a gate run. When the gate shares one runner and its wall budget is tight, children push freely but open or update a PR only when the Orchestrator grants the window by name. With headroom, the window is off and runs overlap.
 18. **Shared enumerations are edited at their canonical source.** A child adding a value to a shared allow-list or replacing a shared object uses the canonical list and landing order its ticket names, never a copy from the last migration it saw.
+19. **Re-read the thread before the PR.** Immediately before `gh pr create` (and again before DONE), re-read the ticket's comments; DONE names each ruling and comment it acted on. Reading "at every phase change" misses a comment that lands mid-phase.
+20. **A delegator, not a tutor.** Finish every piece yourself; never hand a function to a human (`TODO(human)`, "Learn by Doing"). The worktree's seeded settings set `"outputStyle": "default"`, because a delegator otherwise inherits the operator's user-level output style.
 
 ## Spawning: atomic cwd binding
 
@@ -51,6 +55,10 @@
   && c11 send-key --workspace $WS --panel $NEW_PANEL enter
 ```
 
+**Launch once; never put a relaunch on the `||` of a parser.** `launch-agent … --json 2>&1 | jq … || launch-agent …` launches twice whenever a stderr warning (such as `warning[linked_worktree_cwd]`) reaches `jq`: the parse fails and the fallback starts a second agent beside the first. Read the plain `OK panel:N` line, take UUIDs afterwards from the area's panel listing, and count panels per ticket; close a duplicate before it passes its first prompt.
+
+**Read the first screen after launch.** A launch line asking for auto mode can still come up in manual mode (an operator hardening such as `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB` forces it). If the first screen is a permission prompt, flag that panel to the operator by name in the same tick; the seat never changes a delegator's permission mode itself.
+
 The chained send + explicit `send-key enter` (`&&`, so Enter runs only after a successful send) is the durable Claude-to-Claude handoff. A send into a panel that shows an operator draft or a question/plan dialog is refused (`input_guard_refused`, nothing typed); a fresh panel you just created never is. Stage prompts at `<worktree>/.lattice/tmp-prompts/<phase>-prompt.md` (physically bound to the worktree); a `/tmp/<proj>-<n>-<phase>-prompt.md` path is acceptable only with an atomic launch plus the receiver guard (Standard Clause 1).
 
 ## Worktree prep (at dispatch)
@@ -59,8 +67,10 @@
 git worktree add <repo>-worktrees/<ticket-slug> -b <branch> <base>
 ```
 
-Base is `<remote>/main` — or the parent's branch for press-ahead children. Then **propagate gitignored credentials**: copy the root checkout's `.env` (and `~/.netrc`-style material where the project needs it) into the worktree at create time; without it, impl phases die mid-tool-call on confusing "missing secret" errors.
+Base is `<remote>/main` — or the parent's branch for press-ahead children. Then **propagate gitignored credentials**: copy the root checkout's `.env` (and `~/.netrc`-style material where the project needs it) into the worktree at create time; without it, impl phases die mid-tool-call on confusing "missing secret" errors. Seed the worktree's project settings (uncommitted) with the run's permission rules and `"outputStyle": "default"` (Clause 20), and stage the boot prompt inside the worktree (`.claude/boot-prompt.md`): a prompt outside it costs a read-permission prompt before the first turn.
 
+**Probe signing before a fleet.** Where commits are signed through an agent that asks a desktop app to approve (1Password's `op-ssh-sign`), make one signed commit from a freshly spawned panel before dispatching. A spawned tab whose approval dialog is never answered stalls every delegator's commit at once, and the Orchestrator's own session signing fine proves nothing about the tabs.
+
 ## The dispatch loop
 
 Tick body: (1) refresh — run-state, Lattice board, `c11 tree`, rewrite agents.md active table; (2) surface escalations — re-banner **every tick** while `needs_human`/`blocked` stands (a banner that scrolled away 30 minutes ago is the same as silence); (3) press-ahead audit over unspawned tickets; (4) landing-train pass if auto-merge is enabled; (5) auto-close finished panels, meaning every PR of that builder is merged (`c11 close-panel --panel <builder panel>` — it reaps children; `/quit` does not, and orphaned review subprocesses can keep spawning areas after merge; also kill any process whose cwd is a merged ticket's worktree, since an orphaned test worker outlives its delegator); (6) spawn next available delegators, routed to the lightest-loaded delegate area; (7) `ScheduleWakeup` — one pending wake at a time.
@@ -76,6 +86,7 @@
 - Real stall → `c11 send` an "ORCHESTRATOR NOTE: cost frozen N ticks — report status and continue" chained with `send-key enter` (`c11 send … && c11 send-key … enter`). Never trust `send-key enter` alone — the TUI sometimes swallows synthetic Return; always pair it with a fresh `send`. If the send is refused the panel shows a draft or dialog: press nothing, and surface it to the operator.
 - Auth halt (`⎿ Not logged in · Please run /login` in a deep screen read — typically after the operator swaps accounts mid-run) → once restored, send "auth restored, retry the tool call, resume /loop".
 - Queued-but-unsubmitted text (cost moves slightly, input box shows stuck content) → a new `send` replaces the buffer unless the guard refuses it because the stuck text reads as a draft; clear the line (`c11 send-key ctrl+c`) only when it is the agent's own staged text, and leave an operator's alone.
+- **Read the input state in the same command as the send.** A send decided on a screen read minutes old lands as a *queued message* if the operator started a turn meanwhile, and a queued `/clear` wipes that turn's session when it ends. Check `c11 input-state` (or read the screen) inside the same command as the send, never earlier; never send `/clear` to a panel the operator may be driving. To dequeue, `send-key up` loads the queued text into the composer, then backspace it; Escape and ctrl+c interrupt the running turn.
 - **After a usage-limit reset or any fleet-wide outage, nudge every panel**: delegators, their sub-agent panels (planners, implementers, fixers), in-process review agents and captains. Verify each shows fresh commits or activity within one tick. A nudged delegator whose implementer died waits forever, and one whose work is done may be waiting on a CI window you never gave it.
 - **A subtitle unchanged for over 45 minutes means read the screen.** The description is self-reported and goes stale exactly when an agent is stuck or waiting on you.
 - Two consecutive dead sends → the session is dead; surface to the operator and offer a respawn from the latest commit. Dead-session state recovery itself belongs to c11 (workspace persistence + session-resume hook), not the Orchestrator.
@@ -106,7 +117,7 @@
 - **The 600-second rule (HARD).** Code-review invoked from a worktree fails often enough that the fallback is documented behavior, not an exception. Wrap it: `(cd <WORKTREE> && timeout 600 bash -c "LATTICE_SPAWN_BACKEND=headless lattice code-review <ID> --mode single --base <remote>/main --actor agent:<id>-reviewer")` (macOS without coreutils: `gtimeout`, or background-job + kill). On RC 124, an empty diff, or a vacuous review — pivot immediately to the **own-reviewer fallback**: compute the diff yourself (`git log <remote>/main..HEAD --stat` + per-file diffs), write a review in the standard shape (Verdict PASS / PASS-WITH-NITS / FAIL; Critical/Major/Minor/NIT findings with file:line + recommendation), attach it `--role review`, and note "own-reviewer fallback, CLI hung/empty" in the completion comment and decision log.
 - **Base is `<remote>/main`, never bare `main`.** Post-merge they differ; bare `main` produces an empty diff that reads as a clean review.
 - **A fired review is not a finished review — the gate FAILS OPEN.** The review runner can die without a trace the task ever sees (600s timeout, `claude -p` session-limit exit, the firing session killed): `.lattice/review_state/<task>.json` then says `running` forever, and the completion policy passes on ANY lifetime `review`-role evidence — including a FAIL artifact from a previous rework cycle. (2026-07-11, acetate 0.4.0 sweep: one review dead 223 min while reporting `running`, caught only by a merge agent's voluntary cold re-review; one rework merged with only its pre-rework FAIL attached; a third recovered only because the delegator reviewed inline after a 600s timeout.) So: after every review invocation, before advancing, confirm a NEW `--role review` artifact exists that **postdates this cycle's `→ review` transition**, **names the reviewed commit** (== branch HEAD), and **carries a PASS verdict**. At merge time, re-run the same check — a rework cycle invalidates all earlier review evidence. Diagnosis kit: `lattice show <ID> --json` (artifact list + event times), `lattice review-status <ID>`, `ps -p <started_by_pid>` (state `running` + dead pid = dead review), `.lattice/review_state/failures.jsonl` (where timeouts and session-limit exits land — a `session limit` stderr means every subsequent spawn will die too until the limit resets; go straight to own-reviewer). Dead review → own-reviewer fallback (above). Never wait on a `running` claim past ~12 min, never advance on stale evidence.
-- **Review cycles must converge — the cycle limit is a circuit breaker, not a pacing suggestion (HARD).** Cap fix→re-review cycles at 2 per ticket, 3 absolute. At the cap, stop and escalate to the operator even when every finding is concrete and no single finding needs a human decision — "each finding is individually actionable" is exactly how a divergent loop disguises itself. Watch the finding *class* across cycles: prior Majors confirmed fixed but new Majors appearing in newly-added mechanism means the spec, not the code, is the problem — usually absolute acceptance language ("fail closed under any crash," "correct under any reordering") applied across a process/socket/filesystem boundary, which licenses infinite legitimate descent; that is a scope defect only the operator can resolve, never something to force past. Plan reviews should flag absolute acceptance language as a scope hazard before impl begins. Companion tripwire: cumulative diff exceeding ~3× the plan's size estimate escalates on its own, regardless of review verdicts. (2026-07-31, C11-188: eight cycles, eight FAILs, ~10k lines for what was a callback-guard bug; the orchestrator overrode its own 3-cycle guard three times with "no human decision required," and the run ended in a full revert. Full story: `docs/aar-c11-188-attention-loop.md`.)
+- **Review cycles must converge — the cycle limit is a circuit breaker, not a pacing suggestion (HARD).** Cap fix→re-review cycles at 2 per ticket, 3 absolute. At the cap, stop and escalate to the operator even when every finding is concrete and no single finding needs a human decision — "each finding is individually actionable" is exactly how a divergent loop disguises itself. Watch the finding *class* across cycles: prior Majors confirmed fixed but new Majors appearing in newly-added mechanism means the spec, not the code, is the problem — usually absolute acceptance language ("fail closed under any crash," "correct under any reordering") applied across a process/socket/filesystem boundary, which licenses infinite legitimate descent; that is a scope defect only the operator can resolve, never something to force past. Plan reviews should flag absolute acceptance language as a scope hazard before impl begins. The same holds for a guardrail that polices source syntax with a regex or a hand-written tokenizer: it cannot match the language's grammar, so each fix exposes the next bypass (2026-11, one scanner failed review passes 2 through 9). The plan names the real parser (`syn`, a YAML library) or scopes the check to the shapes the tree contains plus a documented refusal; at the cap, the recommendation is that structural change, not another pass. The cap-reached escalation lists the open findings by severity and carries a recommended default, so the operator rules once instead of re-deriving the question. Companion tripwire: cumulative diff exceeding ~3× the plan's size estimate escalates on its own, regardless of review verdicts. (2026-07-31, C11-188: eight cycles, eight FAILs, ~10k lines for what was a callback-guard bug; the orchestrator overrode its own 3-cycle guard three times with "no human decision required," and the run ended in a full revert. Full story: `docs/aar-c11-188-attention-loop.md`.)
 - **A green gate proves the tests pass, not that they still detect.** For a change to the test harness, the gate or a sweep, the review demonstrates each claim: a two-file probe for state crossing files, a planted failure (mutation) for each guard, then reverts. Green CI on such a change is not evidence until that review passes.
 - **Amendment blocks.** Never proceed to impl with untriaged plan-review findings — the impl agent stalls (correctly) on stale guidance. Triage each finding (obvious / evolutionary / complex → `needs_human`) and append to the plan file: `## N. Plan-Review Cycle K Resolutions (AUTHORITATIVE — overrides earlier text on conflict)` with per-finding concern / resolution / section affected. Impl prompts state that the latest Resolutions block is binding. Re-review only when findings were architectural or numerous.
 
diff -ru a/lattice-orchestrator/runs-ledger.md b/lattice-orchestrator/runs-ledger.md
--- a/lattice-orchestrator/runs-ledger.md	2026-11-08 10:17:55
+++ b/lattice-orchestrator/runs-ledger.md	2026-11-08 10:19:12
@@ -2,6 +2,23 @@
 
 The run log behind the skill's rules. Each entry preserves the story that shaped (or validated) a rule; the skill itself carries only the timeless statement. New entries accrue at closeout audits; a footgun mitigated three times in one run, or seen across two runs, promotes to a permanent rule in the references with its story recorded here.
 
+## Rust NYC Pop Quiz build (2026-09-20 to 11-08)
+
+29 contract tickets plus follow-ups and audit gaps (PQ-1 to PQ-89), PRs #8 to #142, 174 merge commits on `main`. Moderate autonomy, N = 4, Opus delegators, every merge on the client's word with a same-minute gate (OPEN / MERGEABLE / CLEAN, `--match-head-commit`, a wait on the merge sha's runs, then the receipt); no PR merged on stale evidence. Phase 2 ran twice (interim 10-04, final 10-05); the build then continued as released follow-ups for a month. Pauses: weekly usage 09-22 and 09-26, the 5-hour window 09-30, and four Orchestrator wind-downs at the ~300k context ceiling, two of them mid merge gate. Closeout audit 11-08 over 841 ticket comments, 35 delegator checkpoints and the run-state history.
+
+- **Hand-rolled syntax scanners never converged.** PQ-47's import guard failed Codex passes 2 through 9, each a new compiler-accepted bypass (a U+200E mark between `use` and `openai`); PQ-85 failed passes 2 to 5, restarted on `syn`, and failed its passes 2 and 3 on `macro_rules!` shapes; PQ-61's YAML line scanner failed three passes and ended on a YAML library. The cap fired every time and the same "one more pass or ship" question was re-asked each time. Source of the parser-or-scoped-check rule and the severity-plus-default cap escalation.
+- **Sixteen clearance round trips.** Correct changes failed guard tests outside the cleared files: a canary route table, a schema drift test, a config-vars smoke test, a process-start allowlist (PQ-13, PQ-15, PQ-23, PQ-70). Each was predictable from the files in scope. Source of the clear-the-guard-tests intake rule.
+- **One contract file amended, its sibling not.** SPEC §4 changed and AC-39 kept the old words (F-49); SPEC §11's host strings changed and the EVALUATION row kept the pre-checkpoint ones (F-53). The Result Validator failed rows on the stale text. Source of the grep-every-contract-file amendment rule.
+- **The plan scored its own rows Partial.** Five validation-plan rows joined a built and an unbuilt surface in one pass condition, one joined two ledgers, and the footer miscounted its own tags (F-58). Source of one-surface-per-row.
+- **The skill's own guard stopped delegators.** `test … || { echo "FATAL"; exit 99; }`, as printed in Clause 1 and the Identity Block, tripped the permission classifier's expansion-obfuscation heuristic on the first command of a fresh session, even in auto mode. Source of the bare-guard form.
+- **Two panels per ticket.** A `launch-agent --json 2>&1 | jq … || launch-agent …` line launched three tickets twice each (11-06): a stderr warning broke the parse and the fallback fired beside a live agent. Source of launch-once.
+- **A queued `/clear`.** A send decided on a two-minute-old screen read landed as a queued message in a panel the client had just started a turn in; the queued `/clear` would have wiped it (11-06). Source of read-input-state-in-the-same-command.
+- **A mid-phase comment missed at PR time.** A ruling posted while a delegator was implementing never reached its PR (#48, 10-04); the next brief re-read the thread before `gh pr create` and it held. Source of Clause 19.
+- **Manual mode at birth.** At least seven panels launched with `--permission-mode auto` came up in manual mode under the operator's `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB` and sat at their first prompt until someone looked. Source of read-the-first-screen.
+- **Delegators as tutors.** Two delegators stopped to ask a human to write a function: they inherited the operator's Learning output style (09-20). Source of Clause 20 and the seeded `outputStyle`.
+- **Signing stalled the fleet.** 25 tickets carry signing BLOCKED comments; on 09-20 and 09-26 every delegator's commit failed at once while the Orchestrator's own session signed fine (the desktop app's approval dialog never reached the spawned tabs, or the sandbox denied the agent socket). Source of the signing probe.
+- **Kept, because they worked.** The comment protocol (READY / PLAN / QUESTION / BLOCKED / DONE with HEAD, BASE and warm test time) and READY field checks; contract defects found at planning and routed upstream as 42 numbered gaps, never edited by a builder; mutation and planted-failure checks (27 of 29 mutants killed at pass 1 on one ticket, 30 of 30 plants red on another); audit findings minted `held` for the client to release by name; UTC-stamped ruling logs inside each checkpoint, which made fresh-seat resumes unambiguous.
+
 ## Ralph Prize V1 build (2026-09-29 to 10-01)
 
 39 contract tickets plus 47 follow-ups (86 tickets, RP-V1-1 to 86), Sonnet-high builders, an Opus orchestrator, auto-merge on one Forgejo runner on Atlas with a gate of under 30 s wall at load (60 s from 09-30 15:02). Every plan ticket was merged about 25 hours in (09-30 22:27 PT); 110 PRs merged (#185 to #299) and 132 commits landed on main. Checkpoints W0, WA and WB passed; the live 100-story rehearsal passed s00 to s13 on v1; the Result Validator read YELLOW (147 Pass, 4 Partial, 1 Fail of 152); v1.00.000 was tagged on 20b3f3c1 at 10-01 03:34 PT. Two usage stalls cost about 1h45m and 1h05m of the whole fleet. Times are PT from run-state; entries stamped 02:45 to 04:25 on 09-30 ran about 1.5 h fast.
```
