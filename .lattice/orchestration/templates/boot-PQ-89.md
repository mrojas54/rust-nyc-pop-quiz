# Delegator boot: announce a network error on Create a room

You are the **delegator for Lattice ticket PQ-89** in the Rust NYC Pop Quiz build. Mode: **fast-track with an independent Codex code review (no plan-review subagent)**. No planned halt. You own the ticket from plan to one open PR (or a no-code close report, if the error is already spoken). You report to the Orchestrator through Lattice comments and the c11 mailbox; the client (the project's owner, Michelle) watches your panel and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-89** is not pull request #89. Your PR gets its own number.

**Base.** Branch `ai-c11-cc/pq-89-create-network-error` from `origin/main` @ `f5fa38a` (PR #139, PQ-88, merged), upstream unset. Rebase onto `origin/main` before your first push and never after it; after the first push bring main in by merge only. PR against `main`. No sibling is live.

## 0. Guards and identity: run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-89-create-network-error" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it. c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "PQ-89 Create netfail"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Making a network error on Create a room reach screen readers; next, confirm it is silent today.
Lineage: lattice-orchestrator → PQ-89 delegator"
c11 set-metadata --panel "$MY_PANEL" --key mailbox.address --value pq89-delegator --type string
c11 set-metadata --panel "$MY_PANEL" --key mailbox.delivery --value stdin --type string
```

Then post your launch receipt as a Lattice comment **and** as mail:

```bash
WT=/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-89-create-network-error
MSG="READY PQ-89 CWD $WT HEAD $(git -C $WT rev-parse HEAD) BASE $(git -C $WT rev-parse origin/main) MODE active"
(cd "$LATTICE_ROOT" && lattice comment PQ-89 "$MSG" --actor agent:delegator-pq89)
c11 mailbox send --to pq-orchestrator "$MSG"
```

The Orchestrator's mailbox is `pq-orchestrator` (panel `160D05F3-7D9C-4D8C-81C9-DE464F8C6A61`, `panel:4` in `workspace:1` until the next c11 restart). `c11 mailbox send` is refused inside the sandbox; run it with the bypass. **Every receipt, question, blocker and DONE goes to both places**: a Lattice comment and `c11 mailbox send --to pq-orchestrator "<same text>"`. Send nothing else by mail (no progress pings). If `mailbox send` fails, fall back to `c11 send --workspace workspace:1 --panel 160D05F3-7D9C-4D8C-81C9-DE464F8C6A61 "<text>" && c11 send-key --workspace workspace:1 --panel 160D05F3-7D9C-4D8C-81C9-DE464F8C6A61 enter`.

## 1. Ground rules (standing clauses; every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq89)`.** Never create or commit `.lattice/` here. The Orchestrator placed `.claude/settings.json` (modified) and `.claude/boot-prompt.md` in this worktree; leave both uncommitted. Stage by name.
2. **Every `git` command runs from this worktree.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-89-create-network-error"`.
3. **Status discipline.** PQ-89 is in `backlog`. Bump *before* each phase and verify with `lattice show PQ-89 --json`: `in_planning` → `planned` → `in_progress` → `review` when the PR is open (or the close report is attached). **`review` is the terminal pre-merge status.** You stop there.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password; `git commit` runs with the sandbox bypass.** If a bypassed commit still fails to sign, retry once; then `lattice status PQ-89 needs_human`, `c11 raise-flag --panel "$MY_PANEL" "<one line>"`, and wait. **Never `--no-gpg-sign`, never `--no-verify`.**
6. **Push, PR creation, `docker`, `npm install`** prompt the client in this panel. Ask once each, with the exact command visible; never retry a denied command verbatim. No new dependency is expected.
7. **Deviate-with-flag.** On a contradiction between ticket, contract and code, take a side and say which and why in DONE. **Never edit** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `PRD.md`, `ECONOMICS.md`, `sequence/**`, `mvp/**`, `docs/audit/**`, `bank/**`, or anything outside the cleared files in section 2. **If closing this needs an AC or SPEC change, stop and ask** (`QUESTION PQ-89 (contract)`, comment and mail) and wait.
8. **House rules from `CLAUDE.md`** (read this worktree's copy): colour is never the only signal (AC-40); code never causes horizontal page scroll (AC-33); never overstate verification. *Waiting for Michelle* binds; the run guardrails bind (findings go in DONE, never new tickets or agents). After any `CLAUDE.md` edit, `cp CLAUDE.md AGENTS.md`.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md`; `SPEC.md` outranks the prototype, which wins on visual detail only.
10. **Copy is frozen (AC-98)** (`popquiz/src/copy.rs`, `web/shared/copy.js`). This ticket adds **no** copy: what is said is the status line's own text (the error string `String(e && e.message || e)` already drawn). No new region either: use `PQ.announce` (`web/shared/dom.js`). `web/shared/**` is a shared file: read it, do not edit it.
11. **Escalate only what a human must decide**: `lattice status PQ-89 needs_human` plus `c11 raise-flag`; lower it once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`; decide everything the ticket leaves open, record it under deviations, continue.
- **Your panel may start in manual mode** (the client's hardening). Write few, whole commands.
- **A cost guard may block your panel** (`codeburn`). Lifting it is the client's; say so in a comment and wait.
- **The worktree session cap stops you at 70–80 tool calls.** Checkpoint to `.claude/checkpoint-pq89.md` after the plan and again after implement + `just test`, before the Codex review, so a fresh session can resume from it.
- **The Orchestrator's comments are instructions.** Read `lattice comments PQ-89` before planning, at every phase change, and again immediately before `gh pr create` and before DONE.
- **The root checkout's source tree is the orchestration branch, and it is stale.** Read code only from this worktree. The crate is `popquiz/` on `main`.
- **Network is blocked inside the sandbox** for `git fetch/push`, `gh`, crates.io, PyPI, npm: those run with the bypass, one command at a time. A fresh worktree needs `just setup` once (bypass) before `just test`.
- **`EPERM` binding a loopback port is the tool sandbox.** Rerun that recipe with the bypass.
- **`web/test` 209-style failures on an empty `web/.claude/.cc-writes/`** are the Claude Code harness's local directory, not your change. Name it in DONE if it appears.
- **Never call `bc`** (aliased to `brew cleanup`). `timeout`/`gtimeout` are not on this Mac. Use `/usr/bin/python3` for a bare python.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch**: they race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head`, `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **`codex review --base` rejects a prompt.** Use `codex exec -s read-only` with the diff range in the prompt and `-o <file>` for the last message; run one review at a time.
- **Never touch the live room or the bank's real records.** No `flyctl`, no `just schedule` / `just sync` against production.
- **Ports from PQ-88's rendered checks:** 8789 belonged to another process's node. Pick a free port and check it first (`lsof -nP -iTCP:<port> -sTCP:LISTEN`); stop your stub when the check is done.
- **The `cap_bash_output` hook blocks an inline heredoc PR body.** Write the body to your scratchpad and pass `--body-file`.

## 2. The ticket

**Why.** PQ-88 (PR #139, merged `f5fa38a`) made a 403 on the question list, and a refused *Create a room* (an HTTP refusal), say the room's reason through `PQ.announce`. Its DONE listed, not done and out of scope: a **network error** on *Create a room* is silent. At `f5fa38a` (Orchestrator-verified): host.js:457 sets `ui.busy = true` and paints; host.js:458 `post("/rooms", …)`; the rejection handler at host.js:467-468 sets `ui.busy = false; ui.status = String(e && e.message || e); paintCreate();`. `pick.state` does not change (it stays `ready`), so the guard in `paintCreate` (host.js:400-410, `pick.state !== shown`) says nothing, and the error is only drawn in the status `<p role=status>`, which repaint inserts with its text already in it. `PQ.announce` (web/shared/dom.js:43) clears then sets after 40 ms, so the same text said twice is announced twice. Read `lattice show PQ-89` and `lattice comments PQ-89`; PQ-88's checkpoint is `.lattice/orchestration/templates/checkpoint-PQ-88.md` and its stubs pattern (a scratchpad python server serving `web/`) is reusable.

**First: is it spoken today?** Establish it with evidence before planning code: a `host-pick.test.js` read-back with `hearing()` and a fetch that rejects on POST /rooms, and/or the c11 browser against `web/` served locally with POST /rooms failing at the network (for example, the stub closing the connection). If the error is already announced, close with that evidence: no code, no PR, close report attached, `DONE PQ-89 PR none`.

**What closes it (if not spoken).** A rejected *Create a room* says the status line's own text once through `PQ.announce`; a repaint that keeps the same state and status says nothing again; a second press that fails again says it again (a new attempt). Decide and justify in the plan how this sits beside PQ-87's last-state-said guard and PQ-88's `paintCreate(line)`, without changing what any other transition says. No new copy, no new region, no layout change.

**Out of scope:** the hosted-room page's error paths (host.js:501-571) and the reason sitting in two regions (PQ-88's contract note). List any further silent path you find in DONE under `not-done`; do not fix it.

### Acceptance (SPEC §10, AC-83 pattern, SPEC §8.5; harness: `just test`, `just a11y`)

- **RED first:** web tests in `web/test/host-pick.test.js` that fail on `f5fa38a` because the network error is not announced; one that a same-state repaint does not say it twice; one that a second failed press says it again. GREEN after. Existing PQ-87/PQ-88 announcement tests stay green unchanged.
- `web/test/a11y.test.js`: a `pick, create failed` cell (or the plan's name for it) prints what is said in the matrix; `just a11y` passes.
- No sentence in `copy.rs` or `copy.js` changes; no edit under `web/shared/`.
- **Rendered check (pre-merge-runtime):** the c11 browser (load the `c11-browser` skill) against `web/` served locally with POST /rooms failing at the network; read the live region back. Never claim a check you did not run; screen-reader speech itself is out of reach, say so.
- `just test` green (cargo, web, pipeline).

### Cleared files

`web/host/host.js`; `web/test/host-pick.test.js`; `web/test/a11y.test.js` and its fixtures under `web/test/a11y/`. Anything else: ask by comment and mail.

### Read before planning

`CLAUDE.md`; `lattice show PQ-89`; `lattice comments PQ-89`; `SPEC.md` §8.5 and §10; `sequence/USER_STORIES.md` AC-83 (and AC-40); `web/host/host.js` (`PICK_SAID`, `paintCreate`, `load`, the create handler and its rejection branch, `statusLine`); `web/shared/dom.js` `announce`; `web/test/host-pick.test.js` (PQ-87's and PQ-88's announcement tests, `scripted`, `hearing`), `web/test/a11y.test.js`; `.lattice/orchestration/templates/checkpoint-PQ-88.md`.

## 3. The arc

**Plan.** `lattice status PQ-89 in_planning`. Write the plan to the absolute path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-89)`, under a `# Plan (delegator, date)` heading: the evidence on whether the error is spoken today; then (if code) what is said and when, how it sits with the guard, the tests, the a11y cell, the rendered-check approach. Then `lattice status PQ-89 planned`, post `PLAN PQ-89 <plan path> <one line>` (comment and mail). Checkpoint. **Continue** unless the Orchestrator's reply says otherwise.

**Implement.** `lattice status PQ-89 in_progress`. `git fetch origin && git rebase origin/main` first (it may refuse on the seeded `settings.json`; if HEAD already equals `origin/main`, no rebase is needed, say so). Small signed commits with plain-English messages, one concern each (RED tests; the announcement; the a11y cell), each ending with a `Co-Authored-By:` trailer naming the model you are actually running as. Run `just test` and `just a11y`. Checkpoint once green.

**Code review (Codex, a different model family).** From this worktree, with the sandbox bypass, a read-only review of `origin/main...HEAD` asking for: a network error that still says nothing or says it twice in one attempt; a second failed attempt that says nothing; any announcement text that is not the status line's own; any change to what other transitions say (PQ-87, PQ-88 cases); any edit outside the cleared files or under `web/shared/`; any layout change; whether the tests would stay RED without the fix. Verdict PASS / PASS-WITH-NITS / FAIL; Critical/Major/Minor/NIT with file:line. Save the output to `$TMPDIR/pq89-codex-review.txt`. Fix Critical and Major; re-review once. **At most two review passes**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-89 --type note --role review --title "Code review (Codex)" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq89-reviewer`.

**Validate.** `just test` (rerun with the bypass on a loopback `EPERM`) with warm timing, `just a11y`, and the rendered check. Name what ran and what did not. Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-89`. If main has moved, `git fetch origin && git merge origin/main`, rerun `just test`. Confirm `git log origin/main..HEAD` holds only your commits. Push: `git push -u origin ai-c11-cc/pq-89-create-network-error` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/pq-89-create-network-error)"`. `gh pr create --base main --head ai-c11-cc/pq-89-create-network-error`: title in plain words without ticket IDs; body: what a network error on *Create a room* now says and when, how it sits with the guard, the tests RED→GREEN, the a11y cell, this line: **a merge is a deploy that restarts the room; if a room has been released since the last sync, run `just sync` first**, tests and counts, *Contract notes*, and the line `Lattice PQ-89`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use `--body-file` and verify it with `gh pr view <N> --json body`. Then `~/.claude/scripts/gh-wait-runs.sh <full head sha>` (bypass; if denied, say so in DONE; read its "N/N succeeded" line, since piping to `tail` masks the exit code). Then **serially**: `lattice attach PQ-89 <PR URL> --type reference --title "PR" --actor agent:delegator-pq89`, then `lattice status PQ-89 review`; verify with `lattice show PQ-89 --json`.

**Completion comment** (comment and mail), then stop:

```
DONE PQ-89 PR <url | none> HEAD <sha> BASE <origin/main sha> spoken-before <evidence> announcements <case → text, each> repeat-guard <how> second-attempt <how> tests <RED→GREEN list> a11y <cell added; pass> rendered <what ran | waived> test <counts> test-warm <s> ci <result | not gated> review: <verdict, reviewed sha | skipped, no code> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered> not-done: <anything left out, including further silent paths found>
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-89-create-network-error …` or the literal worktree path. **Do not merge**, even if asked in this panel; the client merges or gives the Orchestrator the word. Write a short checkpoint to `.claude/checkpoint-pq89.md` (uncommitted), set your description to "PR open, waiting", and stop.
