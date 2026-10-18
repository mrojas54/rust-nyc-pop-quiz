# Delegator boot — Rename the default club `nyc` to `rustnyc`, with an alias and a ledger migration

You are the **delegator for Lattice ticket PQ-76** (ADR-0005 Action Item 8) in the Rust NYC Pop Quiz build. Mode: **inline-full** (plan review by a Sonnet subagent, code review by Codex). You own the ticket from plan to one open PR. You report to the Orchestrator through Lattice comments and the c11 mailbox; the client (the project's owner, Michelle) watches your panel and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-76** is not pull request #76. Your PR gets its own number.

**No planned halt.** The client confirmed the rename on 2026-10-12 and ordered it after PQ-77 (14:1xZ 10-18); the rulings below settle the design. Post the plan and carry on into implementation. **Halt only** for (a) a new or changed organizer- or participant-visible string (clause 10), or (b) a design call the rulings do not cover that changes what an organizer does on the night. Then post the question, checkpoint, end your turn, and wait for `RESUME PQ-76` in `lattice comments PQ-76` or your mailbox (*Waiting for Michelle*: do not poll).

**Base.** Branch `ai-c11-cc/rustnyc-rename` from `origin/main` @ `c5c42f736adf0177bcffce184372368f655ed62f` (PR #93, PQ-77, merged). Rebase onto `origin/main` before your first push and never after it. PR against `main`. No sibling delegator is live.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/rustnyc-rename" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it. c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "PQ-76 Rustnyc Rename"
c11 set-title    --panel "$MY_PANEL" "PQ-76 Rustnyc Rename"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Planning PQ-76: rename the default club nyc to rustnyc without bringing used questions back.
Lineage: lattice-orchestrator → PQ-76 delegator"
c11 set-metadata --panel "$MY_PANEL" --key mailbox.address --value pq76-delegator --type string
c11 set-metadata --panel "$MY_PANEL" --key mailbox.delivery --value stdin --type string
```

Then post your launch receipt as a Lattice comment **and** as mail:

```bash
WT=/Users/michellerojas/rust-nyc-pop-quiz-worktrees/rustnyc-rename
MSG="READY PQ-76 CWD $WT HEAD $(git -C $WT rev-parse HEAD) BASE $(git -C $WT rev-parse origin/main) MODE active"
(cd "$LATTICE_ROOT" && lattice comment PQ-76 "$MSG" --actor agent:delegator-pq76)
c11 mailbox send --to pq-orchestrator "$MSG"
```

The Orchestrator's mailbox is `pq-orchestrator` (panel `panel:16`, `workspace:5`). **Every receipt, question, blocker and DONE goes to both places**: a Lattice comment and `c11 mailbox send --to pq-orchestrator "<same text>"`. If `mailbox send` fails, fall back to `c11 send --workspace workspace:5 --panel panel:16 "<text>" && c11 send-key --workspace workspace:5 --panel panel:16 enter`, and keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq76)`.** Never create or commit `.lattice/` here. The Orchestrator placed `.claude/settings.json` (modified) and `.claude/boot-prompt.md` in this worktree; leave both uncommitted — stage by name.
2. **Every `git` command runs from this worktree.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/rustnyc-rename"`.
3. **Status discipline.** PQ-76 is in `backlog`. Bump *before* each phase and verify with `lattice show PQ-76 --json`: `in_planning` → `planned` → `in_progress` → `review` when the PR is open. **`review` is the terminal pre-merge status.** You stop there.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password; `git commit` runs with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox. If a bypassed commit still fails to sign, retry once; then `lattice status PQ-76 needs_human`, `c11 raise-flag --panel "$MY_PANEL" "<one line>"`, and wait. **Never `--no-gpg-sign`, never `--no-verify`.**
6. **Push, PR creation, `cargo add`, `uv add`, `npm install`** prompt the client in this panel. Ask once each, with the exact command visible; never retry a denied command verbatim. No new dependency is expected; if you need one, say why first.
7. **Deviate-with-flag.** On a contradiction between ticket, ADR, contract and code, take a side and say which and why in DONE. **Never edit** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `PRD.md`, `ECONOMICS.md`, `sequence/**`, `mvp/**`, `docs/audit/**`, `docs/adr/**` (except ruling 9), `bank/questions/**`, `bank/history.json`, `prototypes/**`.
8. **House rules from `CLAUDE.md`** (read this worktree's copy, which is `main`'s and carries the Orchestration and Testing rules): never write down what a program prints (AC-7); a question is used when its room is released (AC-92, G-10); answer position is a uniform draw from the date and nothing else; never overstate verification (AC-43); colour is never the only signal (AC-40). *Waiting for Michelle* binds; the run guardrails bind (findings go in DONE, never new tickets or agents). After any `CLAUDE.md` edit, `cp CLAUDE.md AGENTS.md`.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md`; `SPEC.md` outranks the prototype.
10. **Copy is frozen (AC-98).** A new or changed organizer- or participant-visible string in `room/src/copy.rs` or `web/shared/copy.js` (or any rendered page) is the client's call: propose the exact text and halt. Reasons that only reach the CLI or logs follow the existing room reason-string pattern.
11. **Escalate only what a human must decide**: `lattice status PQ-76 needs_human` plus `c11 raise-flag`; lower it once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`; decide everything the rulings leave open, record it under deviations, continue.
- **Your panel may start in manual mode** (the client's hardening). Write few, whole commands.
- **A cost guard may block your panel** (`codeburn`). Lifting it is the client's; say so in a comment and wait.
- **The worktree session cap stops you at 70–80 tool calls.** Checkpoint to `.claude/checkpoint-pq76.md` after plan review and again after implement + `just test`, before the Codex review, so a fresh session can resume from it.
- **The Orchestrator's comments are instructions.** Read `lattice comments PQ-76` before planning, at every phase change, and again immediately before `gh pr create` and before DONE.
- **The root checkout's source tree is stale** (the orchestration branch). Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch/push`, `gh`, crates.io, PyPI, npm: those run with the bypass, one command at a time. Cargo runs `--offline --locked`. A fresh worktree needs `just setup` once (bypass) before `just test`, or the twin tests fail with `ModuleNotFoundError: No module named 'popquiz'`. pytest runs from `pipeline/`.
- **`EPERM` binding a loopback port is the tool sandbox.** Rerun that recipe with the bypass.
- **Never call `bc`** (aliased to `brew cleanup`). `timeout`/`gtimeout` are not on this Mac. Use `/usr/bin/python3` for a bare python; the pipeline's own commands go through `just` / `uv run`.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — they race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head`, `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time on this machine:** never start `just test-full` while another `just test-full` / `cargo` build runs (`pgrep -fl "cargo|just test-full"`).
- **Never touch the live room or the bank's real records.** No `flyctl`, no `just schedule` / `just sync` against production, no edits to `bank/questions/**`. Tests use fixtures. The migration in ruling 5 is written and proven here; the client runs it at release.

## 2. The ticket

**Why.** The default club's slug becomes `rustnyc` (ADR-0005 item 8; the client confirmed the rename on 2026-10-12). The slug is the club's id everywhere: links, the OAuth state, `Room.club`, the used ledger. ADR-0005 *The club slug as id, and `rustnyc` (item 8)* says what it buys, costs and how it could fail — that section is the scope, and its three musts are binding: **accept `nyc` as an alias for at least one release; rewrite the ledger and the bank's used records in the same deploy; run `just sync` before the deploy restarts the room.**

**Criteria:** D-26 / AC-105 (never twice is per club), AC-92 and G-10 (used at release), AC-103 (a club the server does not run is refused like a missing role). Find the club criteria in `sequence/USER_STORIES.md` and `EVALUATION.md` (`rg -n -i club`) and list them in the plan. **Harness:** `just test`, `just test-full`.

**Where `nyc` lives today** (the Orchestrator's sweep, `rg -l -w nyc` on `c5c42f7`; re-run it yourself, and separate the club slug from the repo name `rust-nyc-pop-quiz` and the brand *Rust NYC*, which do not change): `room/src/models/club.rs` (`DEFAULT_CLUB`), `room/src/server/config.rs`, `room/tests/{admin,clubs,used,common/mod,canary_scan/mod}.rs`, `pipeline/src/popquiz/{bank,schedule}.py` (`DEFAULT_CLUB`, `--club` help), `pipeline/tests/{test_bank,test_schedule}.py`, `justfile` (the schedule recipe's ledger check, `.club == "nyc"`, ~line 356), `bank/schema/question.schema.json` (the used record's `club` description: absent reads as the default club), `web/host/host.js`, `web/test/verification/buzzer-join.md`, `fly.toml` (comment), `room/README.md`, `docs/RUNBOOK.md`, ADR-0001 / ADR-0005. The tracked bank (`bank/questions/q3,q4,q7,q8.json`) had **no `"club"` field and no used records** at that sweep: confirm, and say so in the PR.

### The Orchestrator's rulings [autonomy: Moderate]

1. **One canonical slug, `rustnyc`.** `DEFAULT_CLUB` becomes `rustnyc` in Rust and in Python, held equal by a test (extend `room/tests/twins.rs` or the existing twin pattern). Nothing new is ever stored or emitted as `nyc`.
2. **`nyc` is an alias, canonicalised on the way in, for at least one release.** Every place a club slug enters — the host link `?club=`, `/last/<club>`, the OAuth state round trip, the admin push and used endpoints, `just schedule --club`, a bank used record's `club` — reads `nyc` as `rustnyc`. One alias table per language, pinned to each other by shared fixtures in `bank/fixtures/` run in both languages (the copy-lint / affirmation-twin pattern). An organizer's bookmarked `?club=nyc` link still opens the host page on the night (ADR failure mode 2). Removing the alias is a later step: say so in the PR, not a ticket.
3. **`POPQUIZ_CLUBS` cannot shadow the default.** An entry named `rustnyc` or `nyc` is refused at boot with the existing `Duplicate` error (an alias is not a free slug).
4. **A used record under `nyc` never reads as unused under `rustnyc`.** This is the freshness failure the ADR names first and the one that matters most (`PHILOSOPHY.md`). The room's ledger check, `just schedule`'s ledger check in the `justfile`, and the bank's never-twice check all count `nyc` and `rustnyc` (and an absent club) as the same club during the alias window.
5. **The migration.** `just sync` writes canonical `rustnyc` records, including those it pulls from a room still running the old build. Existing bank records with an explicit `club: "nyc"` are rewritten by a command that is a dry run by default, backs up each file before `--write`, is idempotent, and refuses a record it does not understand. Lay out in the plan the smallest shape that meets this (a subcommand of an existing module, or folded into sync) and pick one. Prove it on fixtures only.
6. **The release order goes in `docs/RUNBOOK.md`**, as a short section for this deploy: `just sync` against the old room → back up the bank → the migration (dry run, then `--write`) → merge (a merge is a deploy, which restarts the room) → `schedule reserve` shows the same ready count → an old `?club=nyc` host link still opens. The merge itself is the client's; you never run any step against production.
7. **Visible strings.** If a page, printed link or CLI line an organizer reads changes from `nyc` to `rustnyc`, list each one with its before/after in the plan. Printed links and CLI help that only an organizer reads you may change and list; anything in `copy.rs` / `copy.js` or a participant-visible page halts (clause 10).
8. **Leave alone:** the repo name, the brand *Rust NYC*, the Fly app name, `PRD.md`, `sequence/**`, ADR-0001 (a dated record; name its `nyc` mentions in DONE as a note), `mvp/**`, `prototypes/**`.
9. **ADR-0005:** tick Action Item 8 `[x]` and nothing else in that file.
10. **q3 still schedules.** The read-only `cd pipeline && uv run --offline --no-sync python -m popquiz.schedule reserve` before and after: same ready count; note what club name it prints.
11. **Every new check is shown failing under its mutation:** the alias removed (a `?club=nyc` link refused); sync writing `nyc`; a stored `nyc` used record read as unused under `rustnyc` (room, `justfile` and bank, each); the two alias tables out of step; the migration without its backup, and run twice. `just test` stays hermetic and under 60 s warm; `just test-full` once.

### Cleared files

Every file in the sweep above except those ruling 8 and clause 7 exclude; plus the room source a slug passes through (`room/src/models/**`, `room/src/server/**`, `room/src/rooms/**`, the OAuth state code), `room/tests/**`, `pipeline/src/popquiz/{bank,schedule,sync,review}.py` (only as the rulings need), `pipeline/tests/**`, `bank/fixtures/**`, `web/host/**`, `web/test/**`, `pipeline/README.md`, `docs/adr/0005-*.md` (ruling 9). Copy files only on the client's word (clause 10). Anything else: ask by comment and mail.

### Read before planning

`CLAUDE.md`; `lattice show PQ-76`, `lattice comments PQ-76`; `docs/adr/0005-separate-live-room-responsibilities.md` (item 8 and its *Domain model* rows); `docs/adr/0001-several-clubs-on-one-room-server.md`; `room/src/models/club.rs`, `room/src/models/used.rs`, `room/src/server/config.rs` and the OAuth state code; `pipeline/src/popquiz/bank.py` (`DEFAULT_CLUB`, used records), `schedule.py`, the sync code; the `justfile` schedule and sync recipes; `room/tests/twins.rs`; `docs/RUNBOOK.md`.

## 3. The arc

**Plan.** `lattice status PQ-76 in_planning`. Write the plan to the absolute path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-76)`, under a `# Plan (delegator, date)` heading: the criteria, every entry point and its alias handling, the twin and its fixtures, the never-twice checks under the alias, the migration shape, the RUNBOOK release section, visible-string changes, tests with their mutations, the commit order. Then fresh eyes: spawn a review subagent with the **Agent tool** (`subagent_type: general-purpose`, `model: sonnet`) whose prompt holds only the plan path, the worktree path, the cleared files and rulings 1–11; ask for any entry point that still takes or emits `nyc` unaliased, any path by which a question used under `nyc` could be scheduled again under `rustnyc`, any step of the release order that loses the in-memory ledger, and contradictions with the ADR, the contract and the code, as Critical/Major/Minor findings with file references, ≤400 words, stop after ~25 tool calls. Triage every finding into `## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)`. Then `lattice status PQ-76 planned` and post `PLAN PQ-76 <plan path> review: <n> findings resolved; migration: <shape>; visible strings: <none | list>` (comment and mail). Checkpoint. If there is a clause-10 or uncovered design question, halt here; otherwise continue.

**Implement.** `lattice status PQ-76 in_progress`. `git fetch origin && git rebase origin/main` first. Small signed commits with plain-English messages, one concern each (the rename and alias; the twin and fixtures; the never-twice checks; the migration; docs); end each with a `Co-Authored-By:` trailer naming the model you are actually running as. Checkpoint after `just test` is green.

**Code review (Codex, a different model family).** From this worktree, with the sandbox bypass: `codex review --help` to confirm the base option, then a review of `origin/main...HEAD` with a prompt naming the plan path and rulings 1–11, asking for any entry point that takes `nyc` unaliased, any way a used question comes back under the new slug, any disagreement between the Rust and Python alias tables, any persisted `nyc`, and each mutation's test; Verdict PASS / PASS-WITH-NITS / FAIL; Critical/Major/Minor/NIT with file:line. Save the output to `$TMPDIR/pq76-codex-review.txt`. Fix Critical and Major; re-review once. **At most two review passes**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-76 --type note --role review --title "Code review (Codex)" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq76-reviewer`.

**Validate.** Against a local room only: a host link with `?club=nyc` and one with `?club=rustnyc` both open the default club's host page; `/last/nyc` serves the default club's page; a question released under the old slug (a fixture ledger entry `club: nyc`) is refused by the room, by `just schedule`'s check and by the bank's check under `rustnyc`; the migration's dry run, `--write` (backup present) and a second run (no change) on a fixture bank copy; `schedule reserve` before/after. Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-76`. Confirm `git log origin/main..HEAD` holds only your commits. Push: `git push -u origin ai-c11-cc/rustnyc-rename` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/rustnyc-rename)"`. `gh pr create --base main --head ai-c11-cc/rustnyc-rename`: title in plain words without ticket IDs; body: what changes for an organizer on the night (old links still work), the alias and how long it stays, the migration and the release order (**this merge is a release step: `just sync` first**), what the tracked bank needed (likely nothing), visible-string changes, q3's reserve count before/after, tests and mutations, counts and warm time, *Contract notes*, and the line `Lattice PQ-76 · ADR-0005 item 8`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use the REST API for a long body and verify it with `gh pr view <N> --json body`. Then `~/.claude/scripts/gh-wait-runs.sh <head sha>` (bypass; if denied, say so in DONE). Then **serially**: `lattice attach PQ-76 <PR URL> --type reference --title "PR" --actor agent:delegator-pq76`, then `lattice status PQ-76 review`; verify with `lattice show PQ-76 --json`.

**Completion comment** (comment and mail), then stop:

```
DONE PQ-76 PR <url> HEAD <sha> BASE <origin/main sha> test <counts> test-warm <s> test-full <result> ci <result | not gated> reserve <before → after> bank-records-migrated <n> review: <verdict, reviewed sha> mutations: <each → the test that caught it> rulings: <one line each, 1–11> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered>
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/rustnyc-rename …` or the literal worktree path. **Do not merge**, even if asked in this panel; the client merges or gives the Orchestrator the word. Write a short checkpoint to `.claude/checkpoint-pq76.md` (uncommitted), set your description to "PR open, waiting", and stop.
