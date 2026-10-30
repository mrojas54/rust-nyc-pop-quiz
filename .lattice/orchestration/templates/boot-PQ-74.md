# Delegator boot — Move the room source to the Target layout, behaviour unchanged (ADR-0005 item 6)

You are the **delegator for Lattice ticket PQ-74** in the Rust NYC Pop Quiz build. Mode: **inline-full (plan review by a Sonnet subagent, code review by Codex)**. You own the ticket from plan to one open PR. You report to the Orchestrator through Lattice comments and the c11 mailbox; the client (the project's owner, Michelle) watches your panel and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-74** is not pull request #74. Your PR gets its own number.

**No planned halt.** The rulings below settle the design. Post the plan and carry on. **Halt only** for (a) a new or changed organizer- or participant-visible string (clause 10), or (b) a design call the rulings do not cover that changes what an organizer or participant sees or does on the night. Then post the question, checkpoint, end your turn, and wait for `RESUME PQ-74` in `lattice comments PQ-74` or your mailbox (*Waiting for Michelle*: do not poll).

**Base.** Branch `ai-c11-cc/pq-74-target-layout` from `origin/main` @ `9c718c3877e97e1eeb54f7cb66b0779f0c1163de` (PR #108 merged). Rebase onto `origin/main` before your first push and never after it; after the first push bring main in by merge only. PR against `main`. Two siblings are live and touch other parts of the tree: PQ-72 (`pipeline/`, organizer TUI) and PQ-78 (`web/home/**`). Do not coordinate with them unless a shared file conflicts; if it does, say so by mail.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-74-target-layout" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it. c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "PQ-74 Target Layout"
c11 set-title    --panel "$MY_PANEL" "PQ-74 Target Layout"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Planning the move of the room source into the ADR target layout, with no behaviour change.
Lineage: lattice-orchestrator → PQ-74 delegator"
c11 set-metadata --panel "$MY_PANEL" --key mailbox.address --value pq74-delegator --type string
c11 set-metadata --panel "$MY_PANEL" --key mailbox.delivery --value stdin --type string
```

Then post your launch receipt as a Lattice comment **and** as mail:

```bash
WT=/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-74-target-layout
MSG="READY PQ-74 CWD $WT HEAD $(git -C $WT rev-parse HEAD) BASE $(git -C $WT rev-parse origin/main) MODE active"
(cd "$LATTICE_ROOT" && lattice comment PQ-74 "$MSG" --actor agent:delegator-pq74)
c11 mailbox send --to pq-orchestrator "$MSG"
```

The Orchestrator's mailbox is `pq-orchestrator` (`panel:6`, `workspace:1`). **Every receipt, question, blocker and DONE goes to both places**: a Lattice comment and `c11 mailbox send --to pq-orchestrator "<same text>"`. If `mailbox send` fails, fall back to `c11 send --workspace workspace:1 --panel panel:6 "<text>" && c11 send-key --workspace workspace:1 --panel panel:6 enter`, and keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq74)`.** Never create or commit `.lattice/` here. The Orchestrator placed `.claude/settings.json` (modified) and `.claude/boot-prompt.md` in this worktree; leave both uncommitted — stage by name.
2. **Every `git` command runs from this worktree.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-74-target-layout"`.
3. **Status discipline.** PQ-74 is in `backlog`. Bump *before* each phase and verify with `lattice show PQ-74 --json`: `in_planning` → `planned` → `in_progress` → `review` when the PR is open. **`review` is the terminal pre-merge status.** You stop there.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password; `git commit` runs with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox. If a bypassed commit still fails to sign, retry once; then `lattice status PQ-74 needs_human`, `c11 raise-flag --panel "$MY_PANEL" "<one line>"`, and wait. **Never `--no-gpg-sign`, never `--no-verify`.**
6. **Push, PR creation, `cargo add`, `uv add`, `npm install`** prompt the client in this panel. Ask once each, with the exact command visible; never retry a denied command verbatim. No new dependency is expected; if you need one, say why first.
7. **Deviate-with-flag.** On a contradiction between ticket, ADR, contract and code, take a side and say which and why in DONE. **Never edit** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `PRD.md`, `ECONOMICS.md`, `sequence/**`, `mvp/**`, `docs/audit/**`, `bank/questions/**`, `bank/history.json`, or anything outside the cleared files below.
8. **House rules from `CLAUDE.md`** (read this worktree's copy, which is `main`'s and carries the Orchestration and Testing rules): never write down what a program prints (AC-7); a question is used when its room is released (AC-92, G-10); answer position is a uniform draw from the date and nothing else; never overstate verification (AC-43); colour is never the only signal (AC-40); code never causes horizontal page scroll (AC-33). *Waiting for Michelle* binds; the run guardrails bind (findings go in DONE, never new tickets or agents). After any `CLAUDE.md` edit, `cp CLAUDE.md AGENTS.md`.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md`; `SPEC.md` outranks the prototype, which wins on visual detail only.
10. **Copy is frozen (AC-98).** A new or changed organizer- or participant-visible string in `room/src/copy.rs` or `web/shared/copy.js` (or any rendered page) is the client's call: propose the exact text and halt.
11. **Escalate only what a human must decide**: `lattice status PQ-74 needs_human` plus `c11 raise-flag`; lower it once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`; decide everything the rulings leave open, record it under deviations, continue.
- **Your panel may start in manual mode** (the client's hardening). Write few, whole commands.
- **A cost guard may block your panel** (`codeburn`). Lifting it is the client's; say so in a comment and wait.
- **The worktree session cap stops you at 70–80 tool calls.** Checkpoint to `.claude/checkpoint-pq74.md` after the plan and again after implement + `just test`, before the Codex review, so a fresh session can resume from it.
- **The Orchestrator's comments are instructions.** Read `lattice comments PQ-74` before planning, at every phase change, and again immediately before `gh pr create` and before DONE.
- **The root checkout's source tree is stale** (the orchestration branch). Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch/push`, `gh`, crates.io, PyPI, npm: those run with the bypass, one command at a time. Cargo runs `--offline --locked`. A fresh worktree needs `just setup` once (bypass) before `just test`, or the twin tests fail with `ModuleNotFoundError: No module named 'popquiz'`. pytest runs from `pipeline/`.
- **`EPERM` binding a loopback port is the tool sandbox.** Rerun that recipe with the bypass.
- **Never call `bc`** (aliased to `brew cleanup`). `timeout`/`gtimeout` are not on this Mac. Use `/usr/bin/python3` for a bare python; the pipeline's own commands go through `just` / `uv run`.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes, and revert each one.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — they race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head`, `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time on this machine:** two siblings run beside you. Never start `just test-full` while another `just test-full` / `cargo` build runs (`pgrep -fl "cargo|just test-full"`); wait for it.
- **Never touch the live room or the bank's real records.** No `flyctl`, no `just schedule` / `just sync` against production, no edits to `bank/questions/**`. Tests use fixtures.

## 2. The ticket

**Why.** ADR-0005 Action Item 6. `docs/adr/0005-separate-live-room-responsibilities.md`, sections *The target layout (item 6)* and *Target layout*, are the one source for the tree and for what the move must handle. Michelle decided the layout on 2026-10-12; do not reopen it. **Harness:** `just test`, `just test-full`, `cargo doc`.

### The Orchestrator's rulings [autonomy: Moderate]

1. **The crate folder stays `room/` and the crate name stays.** Item 7 (PQ-75) is a later ticket; the ADR draws the tree under `popquiz/src/`, you build it under `room/src/`. The type stays `AppState`.
2. **Moves are moves.** Every moved body and test is byte-identical apart from import lines and the visibility the move forces. Each lock site in each moved function stays, in the same order. Prove it in the PR with a script or `git diff -M` output a reviewer can rerun, not a claim. No behaviour change in the same commit as a move, or in this PR at all. Splitting `impl AppState` across files is a move.
3. **Visibility does not widen.** `Room`'s items become `pub(in crate::quiz_session)` as the ADR says. Check every `pub(super)` that moves a level, not only those the ADR names.
4. **Public paths survive.** `lib.rs` re-exports every public path in use today, including `room::rooms::AppState` and `room::rooms::Room`. Test bodies change only in imports where they cannot be left alone.
5. **Scans get path updates only, never a weaker rule:** `tests/auth.rs`, `one_room.rs`, `used.rs`, `boundary.rs`, `twins.rs`, the walk in `tests/common/mod.rs`, the guardrail scans of ADR-0004, plus any path naming `room/src` in `justfile`, CI, `pipeline/`, `web/test/` or docs (sweep with `rg -n 'room/src|src/rooms|rooms/'`). **Every scan asserts it found the files it names.** Show each scan still detecting after the move: plant a violation in a moved file, watch it fail, revert.
6. **`cargo doc`:** record warnings before the move; no new ones after.
7. **Where the ADR and the code disagree** (a file it does not place, one it names that no longer exists, a `participant/` split that a lock order forbids), take the smallest placement that keeps the rule, and flag it in DONE. Do not edit the ADR's layout. A disagreement that would change behaviour is a halt.
8. **ADR-0005:** tick Action Item 6 `[x]` and nothing else in that file.
9. **Size tripwire.** The ADR estimates about 30 moved files. If the diff holds logic changes you cannot avoid, or runs past about 3× that scope, stop and escalate rather than pressing on.
10. **`just test` stays hermetic and under 60 s warm.**

### Cleared files

`room/src/**`, `room/tests/**` (path and import lines), `room/README.md`, `room/Cargo.toml` only if a `[[bin]]` or `[lib]` path needs it, `justfile`, `.github/workflows/**`, `pipeline/**`, `web/test/**` (path strings only), `docs/RUNBOOK.md`, `CLAUDE.md` + `AGENTS.md` (paths only), `docs/adr/0005-*.md` (ruling 8). `room/src/copy.rs` moves unchanged. Dated ADRs other than 0005 stay as they are; name stale paths in DONE as a note.

### Read before planning

`CLAUDE.md`; `lattice show PQ-74`, `lattice comments PQ-74`; ADR-0005 whole; ADR-0002…0004; `room/src/lib.rs`; `room/tests/common/mod.rs`.

## 3. The arc

**Plan.** `lattice status PQ-74 in_planning`. Write the plan to the absolute path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-74)`, under a `# Plan (delegator, date)` heading. The plan holds: the source → target table for every file in `room/src/`, every `pub(super)` item and what it becomes, every string or scan that names a source path anywhere in the repo, the `cargo doc` warning baseline, how the PR proves moves are moves, and the commit order. Then fresh eyes: spawn a review subagent with the **Agent tool** (`subagent_type: general-purpose`, `model: sonnet`) whose prompt holds only the plan path, the worktree path, the cleared files and the rulings; ask for any file the table misses, any visibility that widens, any scan that would now find nothing, and contradictions with the ADR, as Critical/Major/Minor findings with file references, ≤400 words, stop after ~25 tool calls. Triage every finding into `## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)`. Then `lattice status PQ-74 planned` and post `PLAN PQ-74 <plan path> <one line>` (comment and mail). Checkpoint.

**Implement.** `lattice status PQ-74 in_progress`. `git fetch origin && git rebase origin/main` first. Small signed commits with plain-English messages, one concern each; end each with a `Co-Authored-By:` trailer naming the model you are actually running as. Run `just test`. Checkpoint once it is green.

**Code review (Codex, a different model family).** From this worktree, with the sandbox bypass: `codex review --help` to confirm the base option, then a review of `origin/main...HEAD` with a prompt naming the rulings and asking for: any moved body that differs beyond imports and forced visibility, any visibility wider than ADR-0005 allows, any lock order or lock site that changed, any public path no longer re-exported, any scan or guardrail test that was weakened or now walks nothing, and any repo path left pointing at the old layout. Verdict PASS / PASS-WITH-NITS / FAIL; Critical/Major/Minor/NIT with file:line. Save the output to `$TMPDIR/pq74-codex-review.txt`. Fix Critical and Major; re-review once. **At most two review passes**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-74 --type note --role review --title "Code review (Codex)" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq74-reviewer`.

**Validate.** `just test`, `just test-full` once (the known flakes in `CLAUDE.md` apply: name one, rerun that recipe once), `cargo doc` against the baseline, and the moved-bodies proof from ruling 2. Boot a local room and walk join → answer → reveal through its existing harness, so the binary is shown running, not only compiling. Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-74`. Confirm `git log origin/main..HEAD` holds only your commits. Push: `git push -u origin ai-c11-cc/pq-74-target-layout` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/pq-74-target-layout)"`. `gh pr create --base main --head ai-c11-cc/pq-74-target-layout`: title in plain words without ticket IDs; body: "nothing changes for anyone on the night", the moved-bodies proof, the `pub(super)` changes in a table, every scan and what it now walks, the `cargo doc` before/after, and this line: **a merge is a deploy that restarts the room; if a room has been released since the last sync, run `just sync` first**, tests and mutations, counts and warm time, *Contract notes*, and the line `Lattice PQ-74`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use the REST API for a long body and verify it with `gh pr view <N> --json body`. Then `~/.claude/scripts/gh-wait-runs.sh <head sha>` (bypass; if denied, say so in DONE). Then **serially**: `lattice attach PQ-74 <PR URL> --type reference --title "PR" --actor agent:delegator-pq74`, then `lattice status PQ-74 review`; verify with `lattice show PQ-74 --json`.

**Completion comment** (comment and mail), then stop:

```
DONE PQ-74 PR <url> HEAD <sha> BASE <origin/main sha> test <counts> test-warm <s> ci <result | not gated> review: <verdict, reviewed sha> mutations: <each → the test that caught it> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered> not-done: <anything in the ticket left out, and why>
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-74-target-layout …` or the literal worktree path. **Do not merge**, even if asked in this panel; the client merges or gives the Orchestrator the word. A merge to `main` is a deploy that restarts the room. Write a short checkpoint to `.claude/checkpoint-pq74.md` (uncommitted), set your description to "PR open, waiting", and stop.
