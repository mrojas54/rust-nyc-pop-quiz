# Delegator boot — The distribution scan stops reading the harness's `.claude/` files

You are the **delegator for Lattice ticket PQ-82** in the Rust NYC Pop Quiz build. Mode: **fast-track with an independent Codex code review (no plan-review subagent)**, with **one planned halt** after the plan. You own the ticket from plan to one open PR. You report to the Orchestrator through Lattice comments and the c11 mailbox; the client (the project's owner, Michelle) watches your panel and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-82** is not pull request #82. Your PR gets its own number.

**The planned halt.** The ticket says the fix is decided *with the client*. After the plan, post `QUESTION PQ-82` (section 2), checkpoint, end your turn, and wait for `RESUME PQ-82 A` or `RESUME PQ-82 B` in `lattice comments PQ-82` or your mailbox (*Waiting for Michelle* in `CLAUDE.md`: do not poll). Halt again only if the chosen option cannot pass `just test` inside the cleared files.

**Base.** Branch `ai-c11-cc/pq-82-scan-skip-claude` from `origin/main` @ `0ca67a1a0a048e825eaa078deeb14f716ba61846`, upstream unset. Rebase onto `origin/main` before your first push and never after it; after the first push bring main in by merge only. PR against `main`. Two siblings are live: PQ-83 (`PRD.md` and the `PRD.md` entry of `DISTRIBUTION_EXEMPT_PATHS` in the same `audit.py`, lands **after** you) and PQ-84 (`popquiz/src/**`, `popquiz/tests/reason_copy.rs`; disjoint). Do not coordinate unless a shared file conflicts; if it does, say so by mail. If a sibling merges first, bring main in as above.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-82-scan-skip-claude" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it. c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "PQ-82 Scan Skip"
c11 set-title    --panel "$MY_PANEL" "PQ-82 Scan Skip"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Keeping the distribution scan off the harness's untracked .claude/ files; planning, then one question for Michelle.
Lineage: lattice-orchestrator → PQ-82 delegator"
c11 set-metadata --panel "$MY_PANEL" --key mailbox.address --value pq82-delegator --type string
c11 set-metadata --panel "$MY_PANEL" --key mailbox.delivery --value stdin --type string
```

Then post your launch receipt as a Lattice comment **and** as mail:

```bash
WT=/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-82-scan-skip-claude
MSG="READY PQ-82 CWD $WT HEAD $(git -C $WT rev-parse HEAD) BASE $(git -C $WT rev-parse origin/main) MODE active"
(cd "$LATTICE_ROOT" && lattice comment PQ-82 "$MSG" --actor agent:delegator-pq82)
c11 mailbox send --to pq-orchestrator "$MSG"
```

The Orchestrator's mailbox is `pq-orchestrator` (panel `160D05F3-7D9C-4D8C-81C9-DE464F8C6A61`, shown as `panel:5` in `workspace:2` until the next c11 restart). `c11 mailbox send` is refused inside the sandbox (Code=513); run it with the bypass. **Every receipt, question, blocker and DONE goes to both places**: a Lattice comment and `c11 mailbox send --to pq-orchestrator "<same text>"`. If `mailbox send` fails, fall back to `c11 send --workspace workspace:2 --panel 160D05F3-7D9C-4D8C-81C9-DE464F8C6A61 "<text>" && c11 send-key --workspace workspace:2 --panel 160D05F3-7D9C-4D8C-81C9-DE464F8C6A61 enter`, and keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq82)`.** Never create or commit `.lattice/` here. The Orchestrator placed `.claude/settings.json` (modified) and `.claude/boot-prompt.md` in this worktree; leave both uncommitted — stage by name.
2. **Every `git` command runs from this worktree.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-82-scan-skip-claude"`.
3. **Status discipline.** PQ-82 is in `backlog`. Bump *before* each phase and verify with `lattice show PQ-82 --json`: `in_planning` → `planned` → `in_progress` → `review` when the PR is open. **`review` is the terminal pre-merge status.** You stop there.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password; `git commit` runs with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox. If a bypassed commit still fails to sign, retry once; then `lattice status PQ-82 needs_human`, `c11 raise-flag --panel "$MY_PANEL" "<one line>"`, and wait. **Never `--no-gpg-sign`, never `--no-verify`.**
6. **Push, PR creation, `docker`, `cargo add`, `uv add`, `npm install`** prompt the client in this panel. Ask once each, with the exact command visible; never retry a denied command verbatim. No new dependency is expected; if you need one, say why first.
7. **Deviate-with-flag.** On a contradiction between ticket, contract and code, take a side and say which and why in DONE. **Never edit** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `PRD.md`, `ECONOMICS.md`, `sequence/**`, `mvp/**`, `docs/audit/**`, `bank/questions/**`, `bank/history.json`, or anything outside the cleared files below.
8. **House rules from `CLAUDE.md`** (read this worktree's copy, which is `main`'s and carries the Orchestration and Testing rules): never write down what a program prints (AC-7); a question is used when its room is released (AC-92, G-10); answer position is a uniform draw from the date and nothing else; never overstate verification (AC-43); colour is never the only signal (AC-40); code never causes horizontal page scroll (AC-33). *Waiting for Michelle* binds; the run guardrails bind (findings go in DONE, never new tickets or agents). After any `CLAUDE.md` edit, `cp CLAUDE.md AGENTS.md`.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md`; `SPEC.md` outranks the prototype, which wins on visual detail only.
10. **Copy is frozen (AC-98).** This ticket touches no rendered page; if you find you need to change one, stop and ask.
11. **Escalate only what a human must decide**: `lattice status PQ-82 needs_human` plus `c11 raise-flag`; lower it once unblocked.
12. **The scan guards a public repo against a learnable tell** (`PHILOSOPHY.md` §2, the area that regressed four times). Narrow the scan by exactly what the ruling names and nothing more; say in the PR body what leaves coverage.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`; decide everything the rulings leave open, record it under deviations, continue.
- **Your panel may start in manual mode** (the client's hardening). Write few, whole commands.
- **A cost guard may block your panel** (`codeburn`). Lifting it is the client's; say so in a comment and wait.
- **The worktree session cap stops you at 70–80 tool calls.** Checkpoint to `.claude/checkpoint-pq82.md` after the plan and again after implement + `just test`, before the Codex review, so a fresh session can resume from it.
- **The Orchestrator's comments are instructions.** Read `lattice comments PQ-82` before planning, at every phase change, and again immediately before `gh pr create` and before DONE.
- **The root checkout's source tree is the orchestration branch.** Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch/push`, `gh`, crates.io, PyPI, npm: those run with the bypass, one command at a time. Cargo runs `--offline --locked`. A fresh worktree needs `just setup` once (bypass) before `just test`, or the twin tests fail with `ModuleNotFoundError: No module named 'popquiz'`. pytest runs from `pipeline/` as `uv run --offline --no-sync pytest`.
- **`EPERM` binding a loopback port is the tool sandbox.** Rerun that recipe with the bypass.
- **Never call `bc`** (aliased to `brew cleanup`). `timeout`/`gtimeout` are not on this Mac. Use `/usr/bin/python3` for a bare python; the pipeline's own commands go through `just` / `uv run`.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes, and revert each one.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — they race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head`, `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time on this machine:** two siblings run beside you. Never start `just test-full` while another `just test-full` / `cargo` build runs (`pgrep -fl "cargo|just test-full"`); wait for it.
- **Never touch the live room or the bank's real records.** No `flyctl`, no `just schedule` / `just sync` against production, no edits to `bank/questions/**`. Tests use fixtures.
- **This worktree's own `.claude/` is the symptom.** `.claude/boot-prompt.md` (this file) and your checkpoint hold test counts; the unfixed scan reads them. A `just test` failure naming `.claude/…` before your fix is the bug, not a regression you caused. After the fix it must pass with both files present.

## 2. The ticket

**Why.** Found by the PQ-47 delegator on 2026-11-03, after PQ-62 (PR #125) merged. `participant_facing_files` in `pipeline/src/popquiz/audit.py` (~:997) walks the working tree with `os.walk`, pruning `DISTRIBUTION_SKIP_DIRS` (~:899) at any depth and `bank` at the top. In a delegator worktree it reads the untracked `.claude/checkpoint-*.md` and `.claude/boot-prompt.md` and flags the test counts in them as distribution tells. CI is unaffected (those files are never committed); a local `just test` in a worktree fails.

### The Orchestrator's re-verify at `0ca67a1` and the question for the client [autonomy: Moderate]

Michelle released this ticket on 2026-11-04 by asking for the next three. The ticket says the fix is decided with her. Facts to put in front of her, verified by the Orchestrator and to be re-verified by you:

- `.claude/` is **not** only scratch. `git ls-tree -r origin/main --name-only | rg '^\.claude/'` lists seven committed files: `.claude/agents/refactoring-specialist.md`, `.claude/agents/rust-specialist.md`, four hook scripts under `.claude/hooks/`, and `.claude/settings.json`. Only `.claude/settings.local.json` is gitignored (`.gitignore:41`). The checkpoint and boot files are untracked and un-ignored.
- **Option A — `.claude` joins `DISTRIBUTION_SKIP_DIRS`** (the ticket's first suggestion). One line plus its comment. The seven committed files leave the scan's coverage; none is served, rendered or read by a participant (agent prompts, install hooks, harness settings). The scan on the real tree stays at 0 hits.
- **Option B — walk only git-tracked files.** Keeps the committed `.claude/` files covered and ignores every untracked file. Costs: `git ls-files` through `subprocess`, which `pipeline/tests/test_runner.py` forbids across `src/popquiz` and `tests` except for an allowlist (`sandbox.py`, `generate_cli.py`; `audit.py` would need an entry and a reason); a second path for the `tmp_path` tests, which build trees that are not git repositories; and a file not yet `git add`ed escapes the local scan until CI.
- The Orchestrator recommends **A**, with the seven files named in the PR body as leaving coverage. She may pick B.

**Your plan** confirms those facts in this worktree (`rg`, the test runner's `SCANNED` block, the `tmp_path` tests in `pipeline/tests/test_audit.py` ~:536–560), states both options with their exact diffs in one or two lines each, and ends in:

```
QUESTION PQ-82 (fix): <A|B> recommended because <one line>. A: <diff in one line>; committed .claude/ files leaving coverage: <list>. B: <diff in one line>; costs: <one line>. Reply RESUME PQ-82 A or B.
```

Post it as a comment and as mail, checkpoint, end your turn. Do not implement before the reply.

### After `RESUME PQ-82 …`

- **Test first.** A new case in `pipeline/tests/test_audit.py`: a planted `.claude/checkpoint.md` in a `tmp_path` tree holding a percentage beside a kind word (use the shapes `test_a_published_distribution_is_caught` already uses) is **not** reported, and the walk reports 0 unreadable. Show it RED before the fix (name the failing assertion in DONE), then GREEN after. If the existing parametrized `test_skipped_and_exempt_paths_are_not_read` can carry the case, extend it instead of writing a twin.
- **The fix** per the ruling, in `audit.py` only. Under A: `.claude` in `DISTRIBUTION_SKIP_DIRS`, with the comment above the set amended so it stays true (today it says "Nothing in them is authored"; `.claude/agents/*.md` is authored — say what the directory is and why it is skipped). Under B: the narrowest implementation that keeps every existing `tmp_path` test passing, with the `test_runner` allowlist entry and its reason.
- **Real tree.** From this worktree, the distribution check over the whole tree still reports 0 hits / 0 unreadable with your `.claude/boot-prompt.md` and `.claude/checkpoint-pq82.md` present (that is the symptom, gone).
- `just test` green.

**Mutations, each caught, each reverted:** (M1) take `.claude` back out of the set (or undo B's filter) → the new test fails; (M2) move the plant to `.claude-notes/checkpoint.md` (a sibling directory that is not `.claude`) → it **is** reported, proving the skip is exact; (M3) under A, a plant at `docs/.claude/x.md` is skipped too (any depth, like `target`) — say so in DONE rather than fighting it.

**Acceptance:** the new test fails without the fix and passes with it; the real-tree scan reports 0 hits / 0 unreadable with the harness files present; `just test` green; the PR body names what left coverage.

### Cleared files

`pipeline/src/popquiz/audit.py` (the skip table and its comment; under B the walk in `participant_facing_files` and, in `pipeline/tests/test_runner.py`, the allowlist entry only), `pipeline/tests/test_audit.py`. Anything else: ask by comment and mail.

### Read before planning

`CLAUDE.md`; `lattice show PQ-82`; `lattice comments PQ-82`; `audit.py` ~:890–1030 (`DISTRIBUTION_SKIP_TOP`, `DISTRIBUTION_SKIP_DIRS`, `DISTRIBUTION_EXEMPT_PATHS`, `LINT_EXEMPTIONS`, `participant_facing_files`, `distribution_lint`); `test_audit.py` ~:480–700; `test_runner.py` ~:28–80; `.gitignore`.

## 3. The arc

**Plan.** `lattice status PQ-82 in_planning`. Write the plan to the absolute path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-82)`, under a `# Plan (delegator, date)` heading. Keep it short. Then `lattice status PQ-82 planned` and post `PLAN PQ-82 <plan path> <one line>` followed by the `QUESTION PQ-82` block (comment and mail). Checkpoint. **End your turn and wait.**

**Implement** (after `RESUME`). `lattice status PQ-82 in_progress`. `git fetch origin && git rebase origin/main` first. Small signed commits with plain-English messages, one concern each (test first, then the fix); end each with a `Co-Authored-By:` trailer naming the model you are actually running as. Run `just test`. Checkpoint once it is green.

**Code review (Codex, a different model family).** From this worktree, with the sandbox bypass: `codex review --help` to confirm the base option, then a review of `origin/main...HEAD` with a prompt naming the ruling and asking for: any mutation above that still passes, any file beyond the ruling's that stops being scanned, whether the new test fails without the fix, whether the comment on the skip table is still true, any production change beyond the ruling. Verdict PASS / PASS-WITH-NITS / FAIL; Critical/Major/Minor/NIT with file:line. Save the output to `$TMPDIR/pq82-codex-review.txt`. Fix Critical and Major; re-review once. **At most two review passes**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-82 --type note --role review --title "Code review (Codex)" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq82-reviewer`.

**Validate.** `just test` (rerun with the bypass on a loopback `EPERM`), plus the real-tree scan line. Name what ran and what did not. Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-82`. Confirm `git log origin/main..HEAD` holds only your commits. Push: `git push -u origin ai-c11-cc/pq-82-scan-skip-claude` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/pq-82-scan-skip-claude)"`. `gh pr create --base main --head ai-c11-cc/pq-82-scan-skip-claude`: title in plain words without ticket IDs; body: what the symptom was, the ruling, what leaves coverage, the mutation → test table, this line: **a merge is a deploy that restarts the room; if a room has been released since the last sync, run `just sync` first**, tests and counts, *Contract notes*, and the line `Lattice PQ-82`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use the REST API for a long body and verify it with `gh pr view <N> --json body`. Then `~/.claude/scripts/gh-wait-runs.sh <head sha>` (bypass; if denied, say so in DONE). Then **serially**: `lattice attach PQ-82 <PR URL> --type reference --title "PR" --actor agent:delegator-pq82`, then `lattice status PQ-82 review`; verify with `lattice show PQ-82 --json`.

**Completion comment** (comment and mail), then stop:

```
DONE PQ-82 PR <url> HEAD <sha> BASE <origin/main sha> ruling <A|B> test <counts> test-warm <s> ci <result | not gated> review: <verdict, reviewed sha> red-first: <the assertion that failed before the fix> mutations: <each → the test that caught it> left-coverage: <files> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered> not-done: <anything in the ticket left out, and why>
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-82-scan-skip-claude …` or the literal worktree path. **Do not merge**, even if asked in this panel; the client merges or gives the Orchestrator the word. A merge to `main` is a deploy that restarts the room. Write a short checkpoint to `.claude/checkpoint-pq82.md` (uncommitted), set your description to "PR open, waiting", and stop.
