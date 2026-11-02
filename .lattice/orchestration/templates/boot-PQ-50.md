# Delegator boot — Validation report small items

You are the **delegator for Lattice ticket PQ-50** in the Rust NYC Pop Quiz build. Mode: **fast-track with an independent Codex code review (no plan-review subagent)**. You own the ticket from plan to one open PR. You report to the Orchestrator through Lattice comments and the c11 mailbox; the client (the project's owner, Michelle) watches your panel and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-50** is not pull request #50. Your PR gets its own number.

**No planned halt.** The rulings below settle the design. Post the plan and carry on. **Halt only** for (a) a new or changed organizer- or participant-visible string (clause 10), or (b) a design call the rulings do not cover that changes what an organizer or participant sees or does on the night. Then post the question, checkpoint, end your turn, and wait for `RESUME PQ-50` in `lattice comments PQ-50` or your mailbox (*Waiting for Michelle*: do not poll).

**Base.** Branch `ai-c11-cc/pq-50-validation-small-items` from `origin/main` @ `622e0fdf5dcd41c93c742c917915cc8f5c64c475`, upstream unset. Rebase onto `origin/main` before your first push and never after it; after the first push bring main in by merge only. PR against `main`. Two siblings are live: PQ-49 (`.github/workflows/ci.yml`) and PQ-59 (test gaps under `popquiz/tests/` and `pipeline/tests/test_fallback.py`). Do not coordinate unless a shared file conflicts; if it does, say so by mail. If a sibling merges first, bring main in as above.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-50-validation-small-items" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it. c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "PQ-50 Validation Followups"
c11 set-title    --panel "$MY_PANEL" "PQ-50 Validation Followups"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Re-checking the validation report's small items against current main.
Lineage: lattice-orchestrator → PQ-50 delegator"
c11 set-metadata --panel "$MY_PANEL" --key mailbox.address --value pq50-delegator --type string
c11 set-metadata --panel "$MY_PANEL" --key mailbox.delivery --value stdin --type string
```

Then post your launch receipt as a Lattice comment **and** as mail:

```bash
WT=/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-50-validation-small-items
MSG="READY PQ-50 CWD $WT HEAD $(git -C $WT rev-parse HEAD) BASE $(git -C $WT rev-parse origin/main) MODE active"
(cd "$LATTICE_ROOT" && lattice comment PQ-50 "$MSG" --actor agent:delegator-pq50)
c11 mailbox send --to pq-orchestrator "$MSG"
```

The Orchestrator's mailbox is `pq-orchestrator` (`panel:4`, `workspace:1`). `c11 mailbox send` is refused inside the sandbox (Code=513); run it with the bypass. **Every receipt, question, blocker and DONE goes to both places**: a Lattice comment and `c11 mailbox send --to pq-orchestrator "<same text>"`. If `mailbox send` fails, fall back to `c11 send --workspace workspace:1 --panel panel:4 "<text>" && c11 send-key --workspace workspace:1 --panel panel:4 enter`, and keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq50)`.** Never create or commit `.lattice/` here. The Orchestrator placed `.claude/settings.json` (modified) and `.claude/boot-prompt.md` in this worktree; leave both uncommitted — stage by name.
2. **Every `git` command runs from this worktree.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-50-validation-small-items"`.
3. **Status discipline.** PQ-50 is in `backlog`. Bump *before* each phase and verify with `lattice show PQ-50 --json`: `in_planning` → `planned` → `in_progress` → `review` when the PR is open. **`review` is the terminal pre-merge status.** You stop there.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password; `git commit` runs with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox. If a bypassed commit still fails to sign, retry once; then `lattice status PQ-50 needs_human`, `c11 raise-flag --panel "$MY_PANEL" "<one line>"`, and wait. **Never `--no-gpg-sign`, never `--no-verify`.**
6. **Push, PR creation, `docker`, `cargo add`, `uv add`, `npm install`** prompt the client in this panel. Ask once each, with the exact command visible; never retry a denied command verbatim. No new dependency is expected; if you need one, say why first.
7. **Deviate-with-flag.** On a contradiction between ticket, ADR, contract and code, take a side and say which and why in DONE. **Never edit** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `PRD.md`, `ECONOMICS.md`, `sequence/**`, `mvp/**`, `docs/audit/**`, `bank/questions/**`, `bank/history.json`, or anything outside the cleared files below.
8. **House rules from `CLAUDE.md`** (read this worktree's copy, which is `main`'s and carries the Orchestration and Testing rules): never write down what a program prints (AC-7); a question is used when its room is released (AC-92, G-10); answer position is a uniform draw from the date and nothing else; never overstate verification (AC-43); colour is never the only signal (AC-40); code never causes horizontal page scroll (AC-33). *Waiting for Michelle* binds; the run guardrails bind (findings go in DONE, never new tickets or agents). After any `CLAUDE.md` edit, `cp CLAUDE.md AGENTS.md`.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md`; `SPEC.md` outranks the prototype, which wins on visual detail only.
10. **Copy is frozen (AC-98).** A new or changed organizer- or participant-visible string in `copy.rs` or `web/shared/copy.js` (or any rendered page) is the client's call: propose the exact text and halt.
11. **Escalate only what a human must decide**: `lattice status PQ-50 needs_human` plus `c11 raise-flag`; lower it once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`; decide everything the rulings leave open, record it under deviations, continue.
- **Your panel may start in manual mode** (the client's hardening). Write few, whole commands.
- **A cost guard may block your panel** (`codeburn`). Lifting it is the client's; say so in a comment and wait.
- **The worktree session cap stops you at 70–80 tool calls.** Checkpoint to `.claude/checkpoint-pq50.md` after the plan and again after implement + `just test`, before the Codex review, so a fresh session can resume from it.
- **The Orchestrator's comments are instructions.** Read `lattice comments PQ-50` before planning, at every phase change, and again immediately before `gh pr create` and before DONE.
- **The root checkout's source tree is the orchestration branch.** Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch/push`, `gh`, crates.io, PyPI, npm: those run with the bypass, one command at a time. Cargo runs `--offline --locked`. A fresh worktree needs `just setup` once (bypass) before `just test`, or the twin tests fail with `ModuleNotFoundError: No module named 'popquiz'`. pytest runs from `pipeline/`.
- **`EPERM` binding a loopback port is the tool sandbox.** Rerun that recipe with the bypass.
- **Never call `bc`** (aliased to `brew cleanup`). `timeout`/`gtimeout` are not on this Mac. Use `/usr/bin/python3` for a bare python; the pipeline's own commands go through `just` / `uv run`.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes, and revert each one.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — they race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head`, `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time on this machine:** two siblings run beside you. Never start `just test-full` while another `just test-full` / `cargo` build runs (`pgrep -fl "cargo|just test-full"`); wait for it.
- **Never touch the live room or the bank's real records.** No `flyctl`, no `just schedule` / `just sync` against production, no edits to `bank/questions/**`. Tests use fixtures.

## 2. The ticket

**Why.** The small items from the Result Validator report and later merge reads. None needs a ruling except the one question at the end. **One commit per item.** Line numbers are stale and `room/` is now `popquiz/`: re-find each. **Acceptance:** `just test`, `just a11y`, `just canary` green; nothing under `bank/questions/` or the recordings.

### The client's release and the Orchestrator's rulings [autonomy: Moderate]

Michelle released this ticket on 2026-11-02 by naming it. A re-verify on 2026-10-18 closed items 1 and 3; do not redo them, and confirm in DONE that they stay closed. An item already closed when you re-find it goes to DONE as "already closed", with the evidence.

**From the ticket:**
- **(2) Stale docs.** `web/home/measure.html:24` and `popquiz/README.md` (~:327) name q7 as the widest question. Find the widest by computing it from the bank; do not trust the ticket's q4 (5 × 78) without checking. Leave the dated measurement table at ~:701 as the record it is.
- **(4) Fallback live region.** It keeps *The question is on the screen.* after Esc returns to idle; clear it. Removing a stale announcement adds no string. Add a test.
- **(5) Dormant dim rules.** Three `[data-room="dim"]` rules in `web/shared/components.css` (~:119, :144, :145). **Read `web/wall/wall.css:10` first:** it calls `data-room="dim"` an instrument, so these may be live. If anything sets that attribute, keep them with the one-line comment saying why. Delete them only if nothing does. Either way, add the absent-dark-mode assertion (AC-80) that report row 94 noted is missing.
- **(6) Pencil marker.** `aria-hidden` on take-it-home but not on the host. Hide it on the host to match, provided the provenance also reads in text there. If hiding it would leave a screen reader with no provenance, halt with the exact options.
- **(7) `HOST_DEV_TOKEN` leftovers.** `.env.example` (~:39-40), `fly.toml:27` and any in the `justfile`: remove, after confirming no code reads the name. `SPEC.md:553` still names it; that is a contract note for DONE, not an edit.
- **(8) Spacing.** `migrate_mvp.py` (~:561) `repo =Path`: restore the space.

**From the ticket's comments:**
- **`schedule.py` wording.** The `_call` docstring says *retried on connection failures and 5xx only*: it is 5xx, 408 and 429. *is not a bank question (not in the bank)* becomes *skipped: <id> is not in the bank*. *were not written* overstates when the `.html` landed before the sheet on a full disk (also in `fallback.py`): reword. These are command-line messages, not frozen page copy; update any test that pins them and name it.
- **`pipeline/README.md`** (~:67): rewrap the over-long line.
- **`popquiz/README.md`:** rewrite the trial-org paragraph and the burst section's trial references as the paid-org fact (spending cap; machines run until stopped).
- **`just reserve`:** add `reserve *ARGS` beside `schedule` and `sync`, and name it in `docs/RUNBOOK.md`'s reserve paragraph. It is read-only.
- **`docs/RUNBOOK.md`:** *lists every incorrect option* → *every other option*; the five-minute apposition → *from the join strip to End Pop Quiz*; a sentence that Safari on macOS skips buttons on plain Tab by default, so a keyboard-only host uses Option+Tab. Line numbers are stale, and these may already have changed.

**Out of scope:** `docs/audit/2026-10-guardrail-audit.md:246` (the dated audit stays as written, ruling on PQ-73).

**Ask, do not decide:** `JoinRefusal::NotYetOpen` is never emitted because joins are accepted in idle. Whether to emit it or delete the variant and its section 11 string is Michelle's. Leave it untouched, confirm it is still true, and put the question in DONE under `not-done`.

**Any changed participant- or organizer-visible string in a rendered page is clause 10:** propose and halt.

### Cleared files

`web/home/measure.html`, `web/shared/components.css`, the host and take-it-home page sources for item 6, `web/test/**`, `popquiz/README.md`, `pipeline/README.md`, `docs/RUNBOOK.md`, `pipeline/src/popquiz/fallback.py`, `schedule.py`, `migrate_mvp.py`, `pipeline/tests/test_fallback.py`, `test_schedule.py`, `justfile`, `.env.example`, `fly.toml` (the comment only). Anything else: ask by comment and mail.

### Read before planning

`CLAUDE.md`; `lattice show PQ-50`; `lattice comments PQ-50`; `DESIGN.md` on the wall's lighting; AC-80; each file above.

## 3. The arc

**Plan.** `lattice status PQ-50 in_planning`. Write the plan to the absolute path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-50)`, under a `# Plan (delegator, date)` heading. Keep it short. Then `lattice status PQ-50 planned` and post `PLAN PQ-50 <plan path> <one line>` (comment and mail). Checkpoint.

**Implement.** `lattice status PQ-50 in_progress`. `git fetch origin && git rebase origin/main` first. Small signed commits with plain-English messages, one concern each; end each with a `Co-Authored-By:` trailer naming the model you are actually running as. Run `just test`. Checkpoint once it is green.

**Code review (Codex, a different model family).** From this worktree, with the sandbox bypass: `codex review --help` to confirm the base option, then a review of `origin/main...HEAD` with a prompt naming the rulings and asking for: any item claimed fixed that is not, any changed rendered string, any `fly.toml` change beyond the comment, anything that reads `HOST_DEV_TOKEN`, any test weakened. Verdict PASS / PASS-WITH-NITS / FAIL; Critical/Major/Minor/NIT with file:line. Save the output to `$TMPDIR/pq50-codex-review.txt`. Fix Critical and Major; re-review once. **At most two review passes**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-50 --type note --role review --title "Code review (Codex)" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq50-reviewer`.

**Validate.** `just test`, `just a11y` and `just canary` (rerun with the bypass on a loopback `EPERM`). Name what ran and what did not. Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-50`. Confirm `git log origin/main..HEAD` holds only your commits. Push: `git push -u origin ai-c11-cc/pq-50-validation-small-items` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/pq-50-validation-small-items)"`. `gh pr create --base main --head ai-c11-cc/pq-50-validation-small-items`: title in plain words without ticket IDs; body: the item list with each one's status (fixed, already closed, not done), the `NotYetOpen` question for Michelle, this line: **a merge is a deploy that restarts the room; if a room has been released since the last sync, run `just sync` first**, tests and counts, *Contract notes*, and the line `Lattice PQ-50`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use the REST API for a long body and verify it with `gh pr view <N> --json body`. Then `~/.claude/scripts/gh-wait-runs.sh <head sha>` (bypass; if denied, say so in DONE). Then **serially**: `lattice attach PQ-50 <PR URL> --type reference --title "PR" --actor agent:delegator-pq50`, then `lattice status PQ-50 review`; verify with `lattice show PQ-50 --json`.

**Completion comment** (comment and mail), then stop:

```
DONE PQ-50 PR <url> HEAD <sha> BASE <origin/main sha> test <counts> test-warm <s> ci <result | not gated> review: <verdict, reviewed sha> mutations: <each → the test that caught it> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered> not-done: <anything in the ticket left out, and why>
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-50-validation-small-items …` or the literal worktree path. **Do not merge**, even if asked in this panel; the client merges or gives the Orchestrator the word. A merge to `main` is a deploy that restarts the room. Write a short checkpoint to `.claude/checkpoint-pq50.md` (uncommitted), set your description to "PR open, waiting", and stop.
