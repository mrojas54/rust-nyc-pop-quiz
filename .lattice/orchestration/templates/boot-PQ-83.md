# Delegator boot — `PRD.md`: mark the superseded category weights

You are the **delegator for Lattice ticket PQ-83** in the Rust NYC Pop Quiz build. Mode: **fast-track with an independent Codex code review (no plan-review subagent)**, with **one planned halt** after the plan. You own the ticket from plan to one open PR. You report to the Orchestrator through Lattice comments and the c11 mailbox; the client (the project's owner, Michelle) watches your panel and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-83** is not pull request #83. Your PR gets its own number.

**The planned halt.** `PRD.md` is a prior artifact and the wording is the client's. After the plan, post `QUESTION PQ-83` (section 2) carrying the exact proposed text for every passage, checkpoint, end your turn, and wait for `RESUME PQ-83 …` in `lattice comments PQ-83` or your mailbox (*Waiting for Michelle* in `CLAUDE.md`: do not poll). Her reply may rewrite your text; apply hers character for character.

**Base.** Branch `ai-c11-cc/pq-83-prd-weights` from `origin/main` @ `0ca67a1a0a048e825eaa078deeb14f716ba61846`, upstream unset. Rebase onto `origin/main` before your first push and never after it; after the first push bring main in by merge only. PR against `main`. Two siblings are live: PQ-82 (`DISTRIBUTION_SKIP_DIRS` in the same `audit.py` and `test_audit.py`; it lands **before** you — once it has merged, bring `origin/main` in before your push) and PQ-84 (`popquiz/src/**`, `popquiz/tests/reason_copy.rs`; disjoint). Do not coordinate unless a shared file conflicts; if it does, say so by mail.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-83-prd-weights" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it. c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "PQ-83 PRD Weights"
c11 set-title    --panel "$MY_PANEL" "PQ-83 PRD Weights"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Surveying every superseded category-weight passage in PRD.md; proposing the exact wording for Michelle, then halting.
Lineage: lattice-orchestrator → PQ-83 delegator"
c11 set-metadata --panel "$MY_PANEL" --key mailbox.address --value pq83-delegator --type string
c11 set-metadata --panel "$MY_PANEL" --key mailbox.delivery --value stdin --type string
```

Then post your launch receipt as a Lattice comment **and** as mail:

```bash
WT=/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-83-prd-weights
MSG="READY PQ-83 CWD $WT HEAD $(git -C $WT rev-parse HEAD) BASE $(git -C $WT rev-parse origin/main) MODE active"
(cd "$LATTICE_ROOT" && lattice comment PQ-83 "$MSG" --actor agent:delegator-pq83)
c11 mailbox send --to pq-orchestrator "$MSG"
```

The Orchestrator's mailbox is `pq-orchestrator` (panel `160D05F3-7D9C-4D8C-81C9-DE464F8C6A61`, shown as `panel:5` in `workspace:2` until the next c11 restart). `c11 mailbox send` is refused inside the sandbox (Code=513); run it with the bypass. **Every receipt, question, blocker and DONE goes to both places**: a Lattice comment and `c11 mailbox send --to pq-orchestrator "<same text>"`. If `mailbox send` fails, fall back to `c11 send --workspace workspace:2 --panel 160D05F3-7D9C-4D8C-81C9-DE464F8C6A61 "<text>" && c11 send-key --workspace workspace:2 --panel 160D05F3-7D9C-4D8C-81C9-DE464F8C6A61 enter`, and keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq83)`.** Never create or commit `.lattice/` here. The Orchestrator placed `.claude/settings.json` (modified) and `.claude/boot-prompt.md` in this worktree; leave both uncommitted — stage by name.
2. **Every `git` command runs from this worktree.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-83-prd-weights"`.
3. **Status discipline.** PQ-83 is in `backlog`. Bump *before* each phase and verify with `lattice show PQ-83 --json`: `in_planning` → `planned` → `in_progress` → `review` when the PR is open. **`review` is the terminal pre-merge status.** You stop there.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password; `git commit` runs with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox. If a bypassed commit still fails to sign, retry once; then `lattice status PQ-83 needs_human`, `c11 raise-flag --panel "$MY_PANEL" "<one line>"`, and wait. **Never `--no-gpg-sign`, never `--no-verify`.**
6. **Push, PR creation, `docker`, `cargo add`, `uv add`, `npm install`** prompt the client in this panel. Ask once each, with the exact command visible; never retry a denied command verbatim. No new dependency is expected; if you need one, say why first.
7. **Deviate-with-flag.** On a contradiction between ticket, contract and code, take a side and say which and why in DONE. **Never edit** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `ECONOMICS.md`, `sequence/**`, `mvp/**`, `docs/audit/**`, `bank/questions/**`, `bank/history.json`, or anything outside the cleared files below. **`PRD.md` is cleared for this ticket only, and only with the client's wording.**
8. **House rules from `CLAUDE.md`** (read this worktree's copy, which is `main`'s and carries the Orchestration and Testing rules; read *Standing of `PRD.md`* twice): never write down what a program prints (AC-7); a question is used when its room is released (AC-92, G-10); answer position is a uniform draw from the date and nothing else; never overstate verification (AC-43); colour is never the only signal (AC-40); code never causes horizontal page scroll (AC-33). *Waiting for Michelle* binds; the run guardrails bind (findings go in DONE, never new tickets or agents). After any `CLAUDE.md` edit, `cp CLAUDE.md AGENTS.md`.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md`; `SPEC.md` outranks the prototype, which wins on visual detail only. `PRD.md` is below all of them; where it disagrees with `sequence/research/00-synthesis.md`, the synthesis wins.
10. **Copy is frozen (AC-98).** `PRD.md` is not a rendered page, so AC-98 does not bind it; the client's approval of the wording does.
11. **Escalate only what a human must decide**: `lattice status PQ-83 needs_human` plus `c11 raise-flag`; lower it once unblocked.
12. **The scan guards a public repo against a learnable tell** (`PHILOSOPHY.md` §2, the area that regressed four times). The published weights are the tell this ticket removes or marks; the exemption that hid them shrinks to exactly what the client keeps.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`; decide everything the rulings leave open, record it under deviations, continue.
- **Your panel may start in manual mode** (the client's hardening). Write few, whole commands.
- **A cost guard may block your panel** (`codeburn`). Lifting it is the client's; say so in a comment and wait.
- **The worktree session cap stops you at 70–80 tool calls.** Checkpoint to `.claude/checkpoint-pq83.md` after the plan and again after implement + `just test`, before the Codex review, so a fresh session can resume from it.
- **The Orchestrator's comments are instructions.** Read `lattice comments PQ-83` before planning, at every phase change, and again immediately before `gh pr create` and before DONE.
- **The root checkout's source tree is the orchestration branch.** Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch/push`, `gh`, crates.io, PyPI, npm: those run with the bypass, one command at a time. Cargo runs `--offline --locked`. A fresh worktree needs `just setup` once (bypass) before `just test`, or the twin tests fail with `ModuleNotFoundError: No module named 'popquiz'`. pytest runs from `pipeline/` as `uv run --offline --no-sync pytest`.
- **`EPERM` binding a loopback port is the tool sandbox.** Rerun that recipe with the bypass.
- **Never call `bc`** (aliased to `brew cleanup`). `timeout`/`gtimeout` are not on this Mac. Use `/usr/bin/python3` for a bare python; the pipeline's own commands go through `just` / `uv run`.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes, and revert each one.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — they race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head`, `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time on this machine:** two siblings run beside you. Never start `just test-full` while another `just test-full` / `cargo` build runs (`pgrep -fl "cargo|just test-full"`); wait for it.
- **Never touch the live room or the bank's real records.** No `flyctl`, no `just schedule` / `just sync` against production, no edits to `bank/questions/**`. Tests use fixtures.
- **The distribution scan reads this worktree's untracked `.claude/` files** (PQ-82, in flight beside you). A `just test` failure naming `.claude/boot-prompt.md` or your checkpoint is that bug, not yours: note it in DONE and, if PQ-82 has merged, bring `origin/main` in.

## 2. The ticket

**Why.** `PRD.md` still states the version-one answer-category weights 79.99% / 0.01% / 20% as a product constant. `CLAUDE.md` (*Standing of `PRD.md`*) records that a fixed, published distribution is superseded (`PHILOSOPHY.md` §2: a published distribution is a learnable shortcut), and PQ-62 exempted `PRD.md` as a whole from the distribution scan (`DISTRIBUTION_EXEMPT_PATHS` in `pipeline/src/popquiz/audit.py` ~:914, entry `"PRD.md"`) rather than editing a prior artifact. The repo is public.

### The Orchestrator's re-verify at `0ca67a1` and the question for the client [autonomy: Moderate]

Michelle released this ticket on 2026-11-04 by asking for the next three. The edit is hers to word. Passages found by the Orchestrator (`rg -n '79\.99|0\.01|20%|26\.663|0\.007%|0\.013%|0\.25 percentage' PRD.md`), to be re-surveyed by you — the list below is a floor, not the ceiling:

- ~:344–348 (the sampler's fixed weight vector: 79.99 / 0.01 / 20), ~:349–353 (the nonuniform label priors: 26.663%, 0.01%, 20%, and the 20% position line), ~:363 (the vector versioned as an immutable constant), ~:772–778 (the million-draw simulation targets: 0.007%–0.013%, 0.25 points of 20%), ~:898–899 (enforcement of the 79.99/0.01/20 priors), ~:974–975 (the 0.01% target as an intentional, learnable prior).

Two shapes to put to her, each with the **exact replacement text per passage** in your question:

- **Option 1 — mark.** A one-line *Superseded* note placed above each passage, the passage kept verbatim. The numbers stay in a public file, so the `PRD.md` path exemption cannot simply go: say what replaces it (one `LINT_EXEMPTIONS` entry per remaining line that the scan flags — count them by running the scan with the path exemption removed — or the path entry kept with a rewritten reason), and name that count.
- **Option 2 — strike.** Each numeric passage replaced by one sentence that records what stood there and where the supersession is written, e.g. *Superseded: version one sampled answer categories from a fixed, published weight vector; no distribution is published now (`PHILOSOPHY.md` §2, `CLAUDE.md` "Standing of `PRD.md`").* The `PRD.md` path exemption is removed; the scan over the real tree reports 0 hits.
- The Orchestrator recommends **2**: the numbers are the tell, the supersession is already written in two places, and the file's own status line says *Draft*. She may choose 1, or her own wording.

**Your plan** lists every passage with its line numbers at this base and the proposed replacement under each option, then ends in:

```
QUESTION PQ-83 (wording): <1|2> recommended because <one line>. Passages: <n> at PRD.md:<lines>. Option 2 text per passage: <…>. Option 1 note text: <…> and <k> line exemptions. Reply RESUME PQ-83 <1|2> with any wording changes.
```

Post it as a comment and as mail (a long question goes in the comment in full and in the mail as `--body-ref <file>` or a pointer to the comment), checkpoint, end your turn. Do not edit `PRD.md` before the reply.

### After `RESUME PQ-83 …`

- **Apply her wording** exactly, passage by passage. Nothing else in `PRD.md` changes: no reflow, no list renumbering, no title or status edits.
- **Narrow the exemption.** Under 2: delete the `"PRD.md"` entry from `DISTRIBUTION_EXEMPT_PATHS`. Under 1: replace it per her choice. Either way the scan over the real tree from this worktree reports 0 hits and 0 unreadable, and `test_every_path_exemption_names_something_in_the_tree` still passes.
- **A test pins the outcome.** If no existing test proves `PRD.md` is now scanned, add a small one in `pipeline/tests/test_audit.py`: `PRD.md` is in `participant_facing_files(repo)` (or, under 1, that each exempted line still matches its reviewed text — `test_the_exempt_line_is_still_what_was_reviewed` shows the shape).
- `just test` green.

**Mutations, each caught, each reverted:** (M1) put one struck number back (`20% undefined behavior`) → the distribution check reports `PRD.md:<line>`; (M2) restore the `"PRD.md"` path entry → the new test (or `test_every_path_exemption…`, if it now checks for stale reasons) fails, or say which assertion would have to exist and add it.

**Acceptance:** the client approved the exact wording (quote her reply in DONE); the `PRD.md` path exemption is narrowed or removed; the scan reports 0 hits on the real tree; `just test` green.

### Cleared files

`PRD.md` (her wording only), `pipeline/src/popquiz/audit.py` (the `PRD.md` entry of `DISTRIBUTION_EXEMPT_PATHS` and, under option 1, `LINT_EXEMPTIONS` entries only), `pipeline/tests/test_audit.py`. Anything else: ask by comment and mail.

### Read before planning

`CLAUDE.md` (*Standing of `PRD.md`*, *House rules*); `PHILOSOPHY.md` §2; `lattice show PQ-83`; `lattice comments PQ-83`; `PRD.md` in full (it is long; read the passages above in context, and `rg` for every number); `audit.py` ~:890–1000 (`DISTRIBUTION_EXEMPT_PATHS`, `LINT_EXEMPTIONS`, `RATIO`, `KIND_WORD`, `_exempt`); `test_audit.py` ~:480–700; `lattice comments PQ-62` for the 2026-11-03 survey ruling that created the exemption.

## 3. The arc

**Plan.** `lattice status PQ-83 in_planning`. Write the plan to the absolute path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-83)`, under a `# Plan (delegator, date)` heading, carrying the passage table. Then `lattice status PQ-83 planned` and post `PLAN PQ-83 <plan path> <one line>` followed by the `QUESTION PQ-83` block (comment and mail). Checkpoint. **End your turn and wait.**

**Implement** (after `RESUME`). `lattice status PQ-83 in_progress`. `git fetch origin && git rebase origin/main` first. Small signed commits with plain-English messages, one concern each (the `PRD.md` edit; the exemption and its test); end each with a `Co-Authored-By:` trailer naming the model you are actually running as. Run `just test`. Checkpoint once it is green.

**Code review (Codex, a different model family).** From this worktree, with the sandbox bypass: `codex review --help` to confirm the base option, then a review of `origin/main...HEAD` with a prompt naming her ruling and asking for: any `PRD.md` change beyond the approved passages, any approved passage whose text differs from her reply by a character, any number from the superseded vector still present anywhere in `PRD.md`, whether the exemption table and its reasons are still true, any mutation above that still passes. Verdict PASS / PASS-WITH-NITS / FAIL; Critical/Major/Minor/NIT with file:line. Save the output to `$TMPDIR/pq83-codex-review.txt`. Fix Critical and Major; re-review once. **At most two review passes**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-83 --type note --role review --title "Code review (Codex)" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq83-reviewer`.

**Validate.** `just test` (rerun with the bypass on a loopback `EPERM`), plus the real-tree scan line. Name what ran and what did not. Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-83`. If PQ-82 has merged, `git merge origin/main` and rerun `just test`. Confirm `git log origin/main..HEAD` holds only your commits. Push: `git push -u origin ai-c11-cc/pq-83-prd-weights` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/pq-83-prd-weights)"`. `gh pr create --base main --head ai-c11-cc/pq-83-prd-weights`: title in plain words without ticket IDs; body: why the weights were a tell, her ruling quoted, the passages changed, what the exemption table now says, the mutation → test table, this line: **a merge is a deploy that restarts the room; if a room has been released since the last sync, run `just sync` first**, tests and counts, *Contract notes*, and the line `Lattice PQ-83`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use the REST API for a long body and verify it with `gh pr view <N> --json body`. Then `~/.claude/scripts/gh-wait-runs.sh <head sha>` (bypass; if denied, say so in DONE). Then **serially**: `lattice attach PQ-83 <PR URL> --type reference --title "PR" --actor agent:delegator-pq83`, then `lattice status PQ-83 review`; verify with `lattice show PQ-83 --json`.

**Completion comment** (comment and mail), then stop:

```
DONE PQ-83 PR <url> HEAD <sha> BASE <origin/main sha> ruling <1|2, her reply quoted> passages <n> test <counts> test-warm <s> ci <result | not gated> review: <verdict, reviewed sha> mutations: <each → the test that caught it> exemption: <what the table says now> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered> not-done: <anything in the ticket left out, and why>
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-83-prd-weights …` or the literal worktree path. **Do not merge**, even if asked in this panel; the client merges or gives the Orchestrator the word. A merge to `main` is a deploy that restarts the room. Write a short checkpoint to `.claude/checkpoint-pq83.md` (uncommitted), set your description to "PR open, waiting", and stop.
