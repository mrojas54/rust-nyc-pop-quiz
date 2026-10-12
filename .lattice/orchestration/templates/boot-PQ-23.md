# Delegator boot — Review decisions and the affirm gate, in the organizer TUI

You are the **delegator for Lattice ticket PQ-23** (BUILDPLAN T-18, M3) in the Rust NYC Pop Quiz build. Mode: **inline-full**. You own the ticket from plan to one open PR. You report to the Orchestrator through Lattice comments and, for questions, to its panel; the client (the project's owner, Michelle) watches your tab and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-23** is not pull request #23. PR **#64** (merged, `57ae216`) is the organizer TUI you are extending. Your PR gets its own number.

**The affirmation is a human act.** You build the gate and prove it on fixtures. **You never affirm, accept, reject or edit any real question under `bank/questions/`**, and your PR changes nothing under `bank/questions/`. q3's hand affirmation (F-52) is an existing record to display and keep, never rewritten. q4, q7 and q8 are affirmed later, by the client, through what you build.

**Base.** Your branch `ai-c11-cc/organizer-review-affirm` starts from `origin/main` @ `4946538bf31d419e0a9af1d1ca161f3252ba08fe`. Rebase onto `origin/main` before your first push and never after it. Your PR opens against `main`. **One sibling delegator is live: PQ-71** (`ai-c11-cc/live-room-refactor`, `room/src/**` and `docs/adr/`). You touch nothing under `room/` or `docs/adr/`. If you ever need a file it owns, stop and ask.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/organizer-review-affirm" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it.

c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "Review Gate TUI"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Planning review decisions and the affirm gate in the organizer TUI. No real question is affirmed. Next gate: plan review.
Lineage: lattice-orchestrator → PQ-23 delegator"
```

Then post your launch receipt (the Orchestrator verifies these fields):

```bash
(cd "$LATTICE_ROOT" && lattice comment PQ-23 "READY PQ-23 CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/organizer-review-affirm HEAD $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/organizer-review-affirm rev-parse HEAD) BASE $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/organizer-review-affirm rev-parse origin/main) MODE active" --actor agent:delegator-pq23)
```

The Orchestrator's panel is `panel:7` in `workspace:3` (refs reset on a c11 restart; if it is gone, comment on the ticket and wait). Questions go there: `c11 send --workspace workspace:3 --panel panel:7 "<text>" && c11 send-key --workspace workspace:3 --panel panel:7 enter`, and you keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq23)`.** The board lives in the root checkout only. Never create `.lattice/` in this worktree, never commit anything under `.lattice/`. The Orchestrator placed `.claude/settings.json` (tracked on main, now modified) and `.claude/boot-prompt.md` in this worktree; leave both uncommitted — stage by name.
2. **Every `git` command runs from this worktree.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/organizer-review-affirm"`.
3. **Status discipline.** PQ-23 is in `backlog`. Bump *before* each phase and verify with `lattice show PQ-23 --json`: `in_planning` at Plan, `planned` after plan review, `in_progress` at Implement, `review` when the PR is open. There is no `pr_open`; **`review` is the terminal pre-merge status.** You stop there; the Orchestrator completes the ticket. Leave tags alone.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password, and `git commit` must run with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox, not the key. If a bypassed commit still fails to sign, retry once; then stop: `lattice status PQ-23 needs_human`, raise a c11 flag (`c11 raise-flag --panel "$MY_PANEL" "<one line>"`) and wait. **Never `--no-gpg-sign`.** Do not retry past two attempts.
6. **Push, PR creation, `uv add` and `cargo add`** prompt the client for approval in this tab. Ask for each once, with the exact command visible; never retry a denied command verbatim.
7. **Deviate-with-flag.** If SPEC, EVALUATION, the criteria or the code contradict each other or your plan, take a side, say which and why in DONE. **Never edit** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `PRD.md`, `ECONOMICS.md`, `sequence/**`, `mvp/**`, `docs/audit/**`, `bank/questions/**`, `room/**` or `docs/adr/**`.
8. **House rules from `CLAUDE.md`:** **never write down what a program prints** (AC-7) — the verifier writes `verified` and the trace's stdout value, never the TUI and never you; answer position is a uniform draw from the date and nothing else, and **review never chooses, balances or records answer position**; never overstate verification (Miri proves absence of UB on executed paths only); colour is never the only signal (AC-40) — in a terminal too. *Waiting for Michelle* binds (checkpoint, cancel your own timers, end the turn; never poll for her answer), and so do the run guardrails (no new tickets or agents for findings unless asked: findings go in DONE).
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md` on behaviour, state, payloads and copy; `SPEC.md` outranks the prototype.
10. **Shared files are serialized.** You are cleared for exactly the files in §2 "Cleared files". If you need another, stop and ask by comment.
11. **Escalate only what a human must decide.** Human-only blockers: `lattice status PQ-23 needs_human` plus `c11 raise-flag`; lower the flag once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`, never stop to ask which design to pick: decide, record it under deviations, continue.
- **Your tab may start in manual mode.** That is the client's hardening. Write few, whole commands; the client answers each prompt or toggles auto.
- **A cost guard may block your tab** (`codeburn`). Lifting it is the client's; do not work around it. Say so in a comment and wait.
- **The Orchestrator's comments are instructions.** Read `lattice comments PQ-23` before planning, at every phase change, **and again immediately before `gh pr create` and before DONE**. PQ-23 already carries four: the GAP-24 audit, F-52, the 2026-10-11 narrowing ruling and the 2026-10-12 gate clearance. In DONE, name every comment you acted on and list each ruling as its own line.
- **The root checkout's source tree is stale** (the orchestration branch). Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch`, `git push`, `gh`, PyPI and crates.io; those run with the bypass, one command at a time. Cargo runs `--offline --locked`; if the registry cache is cold, `just setup` once (bypass) and say so. If `pipeline/.venv` is missing, `uv sync` once from `pipeline/`.
- **Never call `bc`** (aliased to `brew cleanup`). `timeout`/`gtimeout` are not on this Mac. Use `/usr/bin/python3` where a bare python is needed; the pipeline's own commands go through `just` / `uv run`.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes. One mutation in an earlier ticket looped for 6.5 hours.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — the two writes race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head` and `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time.** Never start two of `just test-full` / `just harness-full` at once. The sibling delegator may run its own; if a gate is slow or a port is busy, wait and rerun rather than chase it.
- **uv policy:** dependencies go in with `uv add` (from `pipeline/`), never pip. This run should need no new dependency on either side; if it does, say why before adding.

## 2. The ticket

**Why.** PR #64 gave the organizer a TUI (`prototypes/organizer-tui/`): preview, the trace step navigator, explicit reveal, and scheduling through the Python scheduler (`scheduling.rs` runs `python -m popquiz.schedule` via `POPQUIZ_PYTHON` or `pipeline/.venv/bin/python`). Its own README says review decisions are not implemented and bank records stay unchanged. This ticket adds exactly that. Re-read `lattice show PQ-23` and all four comments: the 2026-10-11 RULING is the scope.

**Scope, as screens in the organizer TUI:** accept; reject with a reason; edit and re-verify; difficulty judged; **affirm** as a blocking gate that records who and when; middle-beat and quoted-output checks; SPEC §11.1 trope matches shown beside `explains`, `why_tempting` and `hint` as warnings that block nothing (D-23); the `near_duplicate_of` display from T-17. **Criteria:** AC-19–22, AC-72–74, AC-88, AC-95, G-12 (and their `EVALUATION.md` rows).

### The Orchestrator's rulings [autonomy: Moderate]

1. **One writer, in Python.** Every bank write goes through the pipeline's existing bank and verify code. Build it as a `popquiz.review` CLI (today `pipeline/src/popquiz/review.py` is a 4-line stub) that the TUI runs as a subprocess with an argument list, exactly as `scheduling.rs` runs `popquiz.schedule`; reuse that plumbing. No bank writer in Rust. If your plan argues otherwise, put the argument in the PLAN comment and wait for a ruling.
2. **The verifier writes verified output, never the TUI, never review** (AC-7). Edit-and-re-verify saves the edit, marks the record `edited`, then calls the existing verify path; the TUI shows what verify wrote. `verify.py` is called, not changed; if it must change, ask.
3. **Affirm is blocking and attributable.** The gate refuses without an organizer identity and records `affirmed_by` + `affirmed_at` (the schema q3 already uses). The identity comes from an explicit source the organizer sets (a flag or env var you name); **there is no default identity**, and the refusal is one plain sentence. Never write `affirmed_by` on any real question, in code, test or commit.
4. **GAP-24: bind the affirmation to the text it affirmed.** Record a digest of `explains`, `why_tempting` and the trace at affirm time; any edit to them sets `review.status` to `edited` and clears the affirmation; the reader side (`bank.py` validation, and the affirmation checks at `schedule.py:150` and the audit's `in_reserve`) rejects a digest that no longer matches. **q3 has no digest** (affirmed by hand before this ticket): treat a digest-less affirmation as a legacy record — still valid, so the nyc reserve keeps reading 1, displayed as *affirmed (unbound, before GAP-24)* — and name it in DONE. Whether to require re-affirmation of legacy records is the client's call, not yours.
5. **Trope warnings come from `popquiz.copylint`** (one source; the TUI does not keep its own pattern list) and block nothing.
6. **q4's trace is on `main`** (PR #75 drafted it and tightened its eight notes). Display it; do not edit q4. q8 also waits on F-15; show it as blocked, not as an error.
7. **Fixtures, not the bank.** Tests run against a temp copy of fixture records (`pipeline/tests/fixtures/review/**`), never `bank/questions/`. The real verifier (pinned rustc, Miri, Docker) runs only under `just test-full`; `just test` uses the existing replay/fake path. Assert after the suite that `git status --porcelain bank/` is empty.
8. **TUI tests** follow `tests/preview.rs` and `tests/terminal_repeat.py`; `just test-organizer` (tests, clippy `-D warnings`, fmt, build) stays green and inside `just test`'s ≤60 s. Every gate gets a mutation you ran and the test that caught it: affirm without identity; affirm of an unaccepted record; edit not clearing the affirmation; stale digest accepted by the reader; the TUI writing `verified`.
9. **Copy.** New organizer-facing strings follow SPEC §11 and the existing TUI voice; colour is never the only signal. No new copy keys in the room or the web client.

### Cleared files

`prototypes/organizer-tui/**` (src, tests, README, docs; `Cargo.toml`/`Cargo.lock` only after asking), `pipeline/src/popquiz/review.py`, `pipeline/src/popquiz/bank.py` (affirmation binding and validation), `pipeline/src/popquiz/schedule.py` and `pipeline/src/popquiz/audit.py` (the affirmation checks only, ruling 4), `pipeline/tests/test_review*.py`, `pipeline/tests/test_bank.py`, `pipeline/tests/test_schedule*.py` and `pipeline/tests/test_audit*.py` (affirmation tests only), `pipeline/tests/fixtures/review/**`, `pipeline/README.md` and `bank/README.md` (the review sections), the `justfile` (`test-organizer` only, if needed). **Not** `bank/questions/**`, `verify.py`, `room/**`, `docs/adr/**`.

### Read, in this order, before planning

`CLAUDE.md`; `PHILOSOPHY.md`; `lattice show PQ-23` and `lattice comments PQ-23`; `sequence/USER_STORIES.md` for AC-19–22, AC-72–74, AC-88, AC-95; `SPEC.md` §3.3, §7.4, §11 and §11.1; `EVALUATION.md` rows for those IDs and G-12; `BUILDPLAN.md` T-18 and D-23; `docs/audit/2026-10-guardrail-audit.md` GAP-24; then this worktree's `prototypes/organizer-tui/{README.md,docs/*,src/*,tests/*}`, `pipeline/src/popquiz/{review,bank,schedule,audit,copylint,verify,dedupe}.py`, `bank/README.md`, and `bank/questions/q3.json`, `q4.json` (read only).

### Deliverables

- The review CLI and the TUI screens per rulings 1–9, in small signed commits with plain-English messages, one concern each.
- Tests and mutations per rulings 7–8.
- **Green:** `just test` hermetic and under 60 s warm (note the time and the counts); `just test-full` once; `just bank-audit` unchanged on the real bank (the nyc reserve still reads 1).

**Exit check before DONE:** `git diff origin/main --stat` shows only cleared files and nothing under `bank/questions/`; `rg -n 'affirmed_by' prototypes/organizer-tui pipeline/src` shows no literal identity; `rg -n '"verified"|verified\[' prototypes/organizer-tui/src` shows no write; every ruling (1–9) appears as its own line in DONE.

## 3. The arc

**Plan.** `lattice status PQ-23 in_planning`. Write the plan to the path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-23)` — an **absolute** path under `$LATTICE_ROOT/.lattice/plans/`; append below a `# Plan (delegator, date)` heading (the file may already hold an empty template): the review CLI's subcommands and their arguments, the record fields each one writes, the digest and its inputs, the legacy rule, the TUI screens and key bindings, the subprocess plumbing reused from `scheduling.rs`, the fixtures, the tests with their mutations, the commit order, and any contract tension with the side you take. Then fresh eyes: spawn a review subagent with the **Agent tool** (`subagent_type: general-purpose`, `model: sonnet`) whose prompt holds only the plan path, the worktree path, the cleared files and rulings 1–9; ask for contradictions with the contract and the code, any path by which the TUI or review could write `verified` or a model claim into an answer field, any way the affirm gate or the digest could be bypassed, and any test that could touch `bank/questions/`, as Critical/Major/Minor findings with file references, ≤400 words, stop after ~25 tool calls. Triage every finding into `## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)` in the plan, then `lattice status PQ-23 planned` and post `PLAN PQ-23 <plan path> review: <n> findings resolved`.

**Implement.** `lattice status PQ-23 in_progress`. `git fetch origin && git rebase origin/main` first. Small signed commits; end each with a `Co-Authored-By:` trailer naming the model you are actually running as.

**Code review.** Spawn a second review subagent (Agent tool, `model: sonnet`, same bounds; the project's `rust-specialist` for the TUI half is welcome) with the branch, the worktree path, `git diff origin/main...HEAD --stat` (it reads the diffs itself), the plan path and rulings 1–9; ask it to check each ruling and criterion, the one-writer rule, that the gate and the digest refuse what they should, that q3 still schedules, that nothing reaches `bank/questions/`, that every mutation kills a test, and that the diff stays inside the cleared files; Verdict PASS / PASS-WITH-NITS / FAIL with Critical/Major/Minor/NIT findings and file:line. Fix Critical and Major; re-review once if the fixes were more than mechanical. **At most two fix→re-review cycles**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-23 --type note --role review --title "Code review" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq23-reviewer`.

**Validate.** A fixture walk-through end to end (accept → affirm refused without identity → affirm → edit clears it → re-verify → re-affirm), with the commands, what each wrote, and a terminal recording or screenshots in `prototypes/organizer-tui/recordings/` if the existing recording tooling makes it cheap; the suite counts; `bank-audit` on the real bank. Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-23`. Confirm `git log origin/main..HEAD` holds only your commits. Push: `git push -u origin ai-c11-cc/organizer-review-affirm` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/organizer-review-affirm)"`. Open the PR **against `main`**: `gh pr create --base main --head ai-c11-cc/organizer-review-affirm` — title in plain words without ticket IDs; body: what it builds, **that no real question was accepted, rejected, edited or affirmed**, the one-writer design, the digest and the legacy rule for q3, the tests and their mutations, the suite counts and the `just test` warm time, *Contract notes*, and the line `Lattice PQ-23 · T-18 · AC-19–22, AC-72–74, AC-88, AC-95, G-12`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use the REST API for a long body and verify it with `gh pr view <N> --json body`. Then **serially**: `lattice attach PQ-23 <PR URL> --type reference --title "PR" --actor agent:delegator-pq23`, wait, `lattice status PQ-23 review`; verify with `lattice show PQ-23 --json`.

**Completion comment**, then stop:

```
lattice comment PQ-23 "DONE PQ-23 PR <url> HEAD <sha> BASE <origin/main sha> test <counts> test-warm <s> test-full <result> bank-audit <result, nyc reserve N> real-questions-touched: none mutations: <each → the test that caught it, or survived> rulings: <one line each, 1–9> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered>"
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/organizer-review-affirm …` or the literal worktree path — never from `$(pwd)` or a bare `git` inside the `(cd "$LATTICE_ROOT" && …)` subshell. Update your description to say the PR is open and you are waiting. **Do not merge**, even if asked in this tab, without first re-reading `lattice comments PQ-23` for the Orchestrator's hold or clearance. Then write a short checkpoint to `.claude/checkpoint-pq23.md` (uncommitted) and stop.

Refresh `c11 set-description` at each phase change (planning → implementing → in review → PR open), keeping the `Lineage:` line last.
