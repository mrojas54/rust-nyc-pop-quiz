# Closeout audit: Rust NYC Pop Quiz build (2026-09-20 to 11-08)

Draft, 2026-11-08. Sources: run-state.md (decision log, footgun catalog, amendments, dated notes),
841 comments on 89 ticket event logs, 35 archived delegator checkpoints, agents.md, both
validation reports, `origin/main` history. Every finding below cites evidence that was re-read
for this audit; counts are lower bounds. Nothing outside `.lattice/` has been edited.

## Run facts

- Tickets PQ-1 to PQ-89 (29 contract tickets, the rest follow-ups and audit gaps); PRs #8 to #141;
  174 merge commits on `main`. Autonomy Moderate, N = 4, every merge on the client's word.
- Phase 2 ran twice: interim at `0a4b6fb` (10-04), final at `e239346` (10-05). The build then
  continued as follow-up tickets to 11-08.
- Pauses: weekly usage 09-22 and 09-26, the 5-hour window 09-30, idle gaps 10-05 to 10-10,
  10-24, 11-06; Orchestrator context ceiling (~300k) forced four mid-gate wind-downs.
- F-2 to F-59: contract gaps routed upstream, never authored here.

## Findings (failure mode → why it matters → fix)

Route key: **S** = this skill (`references/*.md`), **P** = project `CLAUDE.md` (+ `cp` to
`AGENTS.md`), **L** = project `LESSONS.md` (new), **R** = `runs-ledger.md` only (already a rule
elsewhere).

### 1. Hand-rolled syntax scanners never converge under review (S + L)
- **Evidence:** PQ-47 Codex passes 2 to 9 all FAIL, each a new compiler-accepted bypass
  ("a U+200E mark between 'use' and 'openai'", 11-03). PQ-85 passes 2 to 5 FAIL, then a restart
  on `syn` that still failed pass 3 on `macro_rules!` shapes (11-05/06). PQ-61's YAML line scanner,
  passes 1 to 3 FAIL, ended by switching to `serde_yaml_ng` (PR #134). PQ-66 needed passes 3 and 4.
- **Why:** a regex or tokenizer can't match rustc's or YAML's grammar, so every fix exposes the next
  bypass. The review cap fires but the decision is re-asked ticket by ticket.
- **Fix:** a guardrail that polices source syntax parses with the real parser (`syn`, a YAML
  library), or is scoped to the shapes the tree actually contains plus a documented refusal. The plan
  names which before implementation; a plan review flags a hand-rolled grammar as a scope hazard,
  like absolute acceptance language.

### 2. Clearance lists miss the guard tests that read the touched files (S + L)
- **Evidence:** 16 tickets raised a clearance QUESTION. PQ-13 (09-28): `room/tests/smoke_config.rs`
  and the canary route-table guard. PQ-15: the same canary guard. PQ-23 (10-12): the schema drift
  test. PQ-70 (11-05): `test_runner.py`'s `MAY_START_A_PROCESS` allowlist. PQ-50: `static.js`.
- **Why:** each costs a QUESTION/ruling round trip, and every one was predictable from which
  files the change touches.
- **Fix:** when writing a brief, list the registry and guard tests that read the files in scope
  (route tables, schema drift, allowlists, config helpers) and clear one-line registrations in
  them up front.

### 3. One contract file amended, its siblings not (S)
- **Evidence:** F-49 (SPEC §4 amended by PQ-37, AC-39 still says "letter and count"); F-53 (SPEC
  §4/§11 host strings changed by PQ-36, EVALUATION's AC-45 row not); F-50 (copy in a PROPOSED table
  outside §11); F-58's EVALUATION rows naming a surface that changed. The Result Validator failed
  rows on the stale text.
- **Why:** the terminal audit judges the text; a ruling applied to one file leaves another
  contradicting it.
- **Fix:** an amendment, or a ruling that changes a string or rule, greps every contract file
  (SPEC, EVALUATION, USER_STORIES, the copy module) for the old text and changes all of them in
  the same PR.

### 4. Validation-plan rows mixing a built and an unbuilt surface can only score Partial (S)
- **Evidence:** F-58: rows 26, 44, 57, 65, 110 joined two surfaces in one pass condition; row 107
  joined two ledgers; the footer miscounted its own tags (8/74/44 stated, 7/76/43 actual).
- **Fix:** one surface per row in the validation-plan template, and the tag tally computed, not typed.

### 5. The skill's own cwd guard trips the permission classifier (S)
- **Evidence:** run-state:804: `test … || { echo "FATAL"; exit 99; }` stops a fresh delegator with
  "Contains brace with quote character (expansion obfuscation)", even in auto mode. The skill
  still prints that form at `references/orchestrator.md:13` and `:20`.
- **Fix:** `[ "$(pwd)" = "<abs-worktree>" ] || exit 99` with the message in a comment, in both places.

### 6. A relaunch chained after a parser's `||` launches every ticket twice (S)
- **Evidence:** 11-06 07:1xZ, `panel:19` to `24` for three tickets (run-state:783): a stderr
  warning reached `jq`, `jq` failed, the fallback launched a second agent.
- **Fix:** launch once with no fallback, read the plain `OK panel:N` line, count panels after.

### 7. A send decided on an old screen read lands as a queued message (S)
- **Evidence:** 11-06 02:54Z, a queued `/clear` was about to wipe a session the client had just
  started a turn in (run-state:807, :87).
- **Fix:** read `c11 input-state` in the same command as the send; never send `/clear` to a panel
  the operator may be driving. Dequeue with `send-key up` then backspaces, never Escape or ctrl+c.
  (The existing "queued-but-unsubmitted" bullet covers a different case: the agent's own text.)

### 8. Comments that land mid-phase are missed at PR time (S)
- **Evidence:** PR #48 (10-04 23:0x): a ruling comment posted mid-phase was not acted on;
  run-state:756 names the lesson; PQ-28's brief carried the fix and it held.
- **Fix:** the DONE step re-reads the ticket's comments before `gh pr create` and names each one it
  acted on.

### 9. Delegators come up in manual mode and sit at the first prompt (S)
- **Evidence:** at least 7 instances (09-21 PQ-20, 10-04 the Result Validator, 11-05 ×3, 11-06 ×3).
  Cause: `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB` forces the default mode on a `claude` started through
  `c11 send` (run-state:805).
- **Fix:** after launch, read the panel's first screen; if it shows a manual-mode prompt, flag the
  panel to the operator by name in the same tick. The seat never changes the mode itself.

### 10. The review cap's BLOCKED comment re-asks the same decision every time (S)
- **Evidence:** at least 9 tickets reached the cap (PQ-29, 47, 55, 59, 61, 66, 70, 77, 85); each asked "one more pass or ship with the gap" from
  scratch (PQ-61 11-06 08:18Z, PQ-66, PQ-77, PQ-89).
- **Fix:** the cap-reached comment lists open findings by severity and carries a recommended
  default; when the class of finding is #1's, the recommendation is the structural change, not
  another pass.

### 11. Delegators inherit the operator's Learning output style (S)
- **Evidence:** PQ-2 and PQ-18 stopped to ask a human to write a function (09-20, run-state:790).
- **Fix:** seed `"outputStyle": "default"` in the worktree's settings and say "you are a delegator,
  not a tutor" in the boot prompt.

### 12. Project test environment facts that cost delegators a round each (P)
- **Evidence:** `just test` needs the sandbox off for more than loopback binds: the organizer PTY
  test ("out of pty devices", PQ-23) and xcrun (PQ-42). The 200-buzzer flake's cause is the macOS
  listen backlog, `kern.ipc.somaxconn=128` (two ticket comments). `codex review --base` refuses a
  custom prompt with exit 2 and no review (PQ-58, PQ-61, PQ-66); the working form is
  `codex exec -s read-only -C <worktree> -o <scratchpad>/<file> -`. Output written from a
  sandbox-off shell lands in a different `$TMPDIR` (PQ-58, PQ-61).
- **Fix:** add the PTY/xcrun cases to the existing EPERM line, the cause to the known-flakes line, and
  the Codex review recipe plus the scratchpad-not-`$TMPDIR` rule to Testing.

### Already rules elsewhere (R: ledger story only)
- **Commit signing stalls:** 25 tickets, fleet-wide on 09-20 and 09-26. Now covered by the global
  CLAUDE.md (socket allowlist, retry once, never bypass). New for the skill, generic: probe a signed
  commit from a spawned panel before dispatching a fleet. (S, one line.)
- **Usage windows and the context ceiling:** the skill's landing-capacity line and the global
  orchestrator-context rule already cover these.
- **Never rebase a pushed branch:** Clause 4; followed throughout.

## Kept (worked repeatedly)

- The comment protocol (READY / PLAN / QUESTION / BLOCKED / DONE with HEAD, BASE, warm test time)
  and READY field verification.
- The same-minute merge gate: OPEN / MERGEABLE / CLEAN, `--match-head-commit`, `gh-wait-runs.sh` on
  the merge sha, then the receipt. No PR merged on stale evidence.
- Planning-time contract defects routed upstream (42 F-IDs), never edited by a delegator.
- Mutation and planted-failure checks (PQ-42 killed 27 of 29 at pass 1; PQ-61's 30 plants all RED).
- Dated, UTC-stamped ruling logs inside each checkpoint; fresh-seat resumes from them.
- Audit findings minted `held`: the client releases them by name.

## Routing (ruled by the client 2026-11-08)

| Destination | Findings | Outcome |
|---|---|---|
| Project `CLAUDE.md` → `cp` to `AGENTS.md` | 12 | PR #142, merged `d5c811b` |
| Project `LESSONS.md` (new) | 1, 2, 3 with this repo's guard-test map | PR #142, merged `d5c811b` |
| Skill (`SKILL.md`, `references/orchestrator.md`, `references/intake.md`) | 1–11, signing probe | `closeout-skill-proposal.md` + `lattice-orchestrator-closeout.patch` beside this file, for c11's skill source. The skill is c11-installed (`.c11-skill.json`) and ignored by claude-skills, so no local edit |
| Skill `runs-ledger.md` | the run's entry | same patch |
| New Lattice tickets | none: every finding is a rule, not code work | — |

## Deferred (noticed, not in scope)

- `references/orchestrator.md` Clause 11 says a 60 s `/loop` tick while the dispatch loop says 270 s.
  The two contradict each other; this run has no evidence on it.
- `references/orchestrator.md:105-108` describe `lattice plan-review` / `code-review` / `review-status`,
  which Lattice 0.2.0 does not have (run-state:654). Install drift, not a lesson from this run.
