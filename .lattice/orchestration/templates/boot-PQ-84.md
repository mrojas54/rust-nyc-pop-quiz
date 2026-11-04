# Delegator boot — Move the remaining host refusal and lifecycle reasons into `copy.rs`

You are the **delegator for Lattice ticket PQ-84** in the Rust NYC Pop Quiz build. Mode: **fast-track with an independent Codex code review (no plan-review subagent)**, with **one planned halt** after the plan. You own the ticket from plan to one open PR. You report to the Orchestrator through Lattice comments and the c11 mailbox; the client (the project's owner, Michelle) watches your panel and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-84** is not pull request #84. Your PR gets its own number.

**The planned halt.** The ticket says: list the exact files and get them cleared by the client before editing. After the plan, post `QUESTION PQ-84` (section 2) with every file, line and string, checkpoint, end your turn, and wait for `RESUME PQ-84 …` in `lattice comments PQ-84` or your mailbox (*Waiting for Michelle* in `CLAUDE.md`: do not poll). Edit only the files her reply clears. Halt again only if moving a string would change its text (AC-98: propose the exact text).

**Base.** Branch `ai-c11-cc/pq-84-lifecycle-reason-copy` from `origin/main` @ `0ca67a1a0a048e825eaa078deeb14f716ba61846`, upstream unset. Rebase onto `origin/main` before your first push and never after it; after the first push bring main in by merge only. PR against `main`. Two siblings are live, both in `pipeline/` (PQ-82, PQ-83; disjoint from you). Do not coordinate unless a shared file conflicts; if it does, say so by mail. If a sibling merges first, bring main in as above.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-84-lifecycle-reason-copy" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it. c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "PQ-84 Reason Copy"
c11 set-title    --panel "$MY_PANEL" "PQ-84 Reason Copy"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Listing every host and organizer refusal sentence still outside copy.rs; asking Michelle to clear the files, then moving them verbatim.
Lineage: lattice-orchestrator → PQ-84 delegator"
c11 set-metadata --panel "$MY_PANEL" --key mailbox.address --value pq84-delegator --type string
c11 set-metadata --panel "$MY_PANEL" --key mailbox.delivery --value stdin --type string
```

Then post your launch receipt as a Lattice comment **and** as mail:

```bash
WT=/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-84-lifecycle-reason-copy
MSG="READY PQ-84 CWD $WT HEAD $(git -C $WT rev-parse HEAD) BASE $(git -C $WT rev-parse origin/main) MODE active"
(cd "$LATTICE_ROOT" && lattice comment PQ-84 "$MSG" --actor agent:delegator-pq84)
c11 mailbox send --to pq-orchestrator "$MSG"
```

The Orchestrator's mailbox is `pq-orchestrator` (panel `160D05F3-7D9C-4D8C-81C9-DE464F8C6A61`, shown as `panel:5` in `workspace:2` until the next c11 restart). `c11 mailbox send` is refused inside the sandbox (Code=513); run it with the bypass. **Every receipt, question, blocker and DONE goes to both places**: a Lattice comment and `c11 mailbox send --to pq-orchestrator "<same text>"`. If `mailbox send` fails, fall back to `c11 send --workspace workspace:2 --panel 160D05F3-7D9C-4D8C-81C9-DE464F8C6A61 "<text>" && c11 send-key --workspace workspace:2 --panel 160D05F3-7D9C-4D8C-81C9-DE464F8C6A61 enter`, and keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq84)`.** Never create or commit `.lattice/` here. The Orchestrator placed `.claude/settings.json` (modified) and `.claude/boot-prompt.md` in this worktree; leave both uncommitted — stage by name.
2. **Every `git` command runs from this worktree.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-84-lifecycle-reason-copy"`.
3. **Status discipline.** PQ-84 is in `backlog`. Bump *before* each phase and verify with `lattice show PQ-84 --json`: `in_planning` → `planned` → `in_progress` → `review` when the PR is open. **`review` is the terminal pre-merge status.** You stop there.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password; `git commit` runs with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox. If a bypassed commit still fails to sign, retry once; then `lattice status PQ-84 needs_human`, `c11 raise-flag --panel "$MY_PANEL" "<one line>"`, and wait. **Never `--no-gpg-sign`, never `--no-verify`.**
6. **Push, PR creation, `docker`, `cargo add`, `uv add`, `npm install`** prompt the client in this panel. Ask once each, with the exact command visible; never retry a denied command verbatim. No new dependency is expected; if you need one, say why first.
7. **Deviate-with-flag.** On a contradiction between ticket, ADR, contract and code, take a side and say which and why in DONE. **Never edit** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `PRD.md`, `ECONOMICS.md`, `sequence/**`, `mvp/**`, `docs/audit/**`, `bank/questions/**`, `bank/history.json`, or anything outside the files the client clears.
8. **House rules from `CLAUDE.md`** (read this worktree's copy, which is `main`'s and carries the Orchestration and Testing rules): never write down what a program prints (AC-7); a question is used when its room is released (AC-92, G-10); answer position is a uniform draw from the date and nothing else; never overstate verification (AC-43); colour is never the only signal (AC-40); code never causes horizontal page scroll (AC-33). *Waiting for Michelle* binds; the run guardrails bind (findings go in DONE, never new tickets or agents). After any `CLAUDE.md` edit, `cp CLAUDE.md AGENTS.md`.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md`; `SPEC.md` outranks the prototype, which wins on visual detail only.
10. **Copy is frozen (AC-98).** Every move is verbatim, character for character. A new or changed organizer- or participant-visible string in `copy.rs` or `web/shared/copy.js` (or any rendered page) is the client's call: propose the exact text and halt.
11. **Escalate only what a human must decide**: `lattice status PQ-84 needs_human` plus `c11 raise-flag`; lower it once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`; decide everything the rulings leave open, record it under deviations, continue.
- **Your panel may start in manual mode** (the client's hardening). Write few, whole commands.
- **A cost guard may block your panel** (`codeburn`). Lifting it is the client's; say so in a comment and wait.
- **The worktree session cap stops you at 70–80 tool calls.** Checkpoint to `.claude/checkpoint-pq84.md` after the plan and again after implement + `just test`, before the Codex review, so a fresh session can resume from it.
- **The Orchestrator's comments are instructions.** Read `lattice comments PQ-84` before planning, at every phase change, and again immediately before `gh pr create` and before DONE.
- **The root checkout's source tree is the orchestration branch.** Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch/push`, `gh`, crates.io, PyPI, npm: those run with the bypass, one command at a time. Cargo runs `--offline --locked`. A fresh worktree needs `just setup` once (bypass) before `just test`, or the twin tests fail with `ModuleNotFoundError: No module named 'popquiz'`. pytest runs from `pipeline/`.
- **`EPERM` binding a loopback port is the tool sandbox.** Rerun that recipe with the bypass.
- **Never call `bc`** (aliased to `brew cleanup`). `timeout`/`gtimeout` are not on this Mac. Use `/usr/bin/python3` for a bare python; the pipeline's own commands go through `just` / `uv run`.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes, and revert each one.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — they race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head`, `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time on this machine:** two siblings run beside you. Never start `just test-full` while another `just test-full` / `cargo` build runs (`pgrep -fl "cargo|just test-full"`); wait for it.
- **Never touch the live room or the bank's real records.** No `flyctl`, no `just schedule` / `just sync` against production, no edits to `bank/questions/**`. Tests use fixtures.
- **The distribution scan reads this worktree's untracked `.claude/` files** (PQ-82, in flight beside you). A `just test` failure naming `.claude/boot-prompt.md` or your checkpoint is that bug, not yours: note it in DONE and, if PQ-82 has merged, bring `origin/main` in.

## 2. The ticket

**Why.** PQ-60 (PR #126, merged `bdcf951`) moved the answer-route and admin reasons, plus `answers.rs`, `state_manager.rs` and `host/create.rs` (the client's ruling *"A, three files only"*, 2026-11-03), into the room-only block of `popquiz/src/copy.rs`, and added the lint `popquiz/tests/reason_copy.rs` (`REASON_SOURCES` names those three files). Its DONE left three things: (1) the refusal text built in `Affirmation::refusal`; (2) other host phase and lifecycle reasons Codex named in review pass 1; (3) a Minor: a sentence held in a same-file `const` evades the lint. Every one of those strings is the client's frozen copy (AC-98), so moves are verbatim.

### The Orchestrator's re-verify at `0ca67a1` and the question for the client [autonomy: Moderate]

Michelle released this ticket on 2026-11-04 by asking for the next three. The ticket says the files are cleared by her before editing. Candidates found by the Orchestrator, a floor for your survey:

- `popquiz/src/quiz_session/models/affirmation.rs` ~:138–148, `Affirmation::refusal`: three `why` sentences (*its text changed after it was affirmed; affirm it again at review.* / *its affirmation records who and when but not what was affirmed; affirm it again at review.* / *is not affirmed; affirm it at review, then schedule it again.*), formatted as `{id}: {why}`. Reaches the organizer through the admin route's `reason` (`answers.rs:682` → `state_manager.rs:248`) and the host through `host/create.rs:126`.
- `popquiz/src/quiz_session/models/phase.rs` ~:388, 403, 426, 436, 449, 454, 455: seven host refusal sentences built into `Refused { reason }` (*There is no room yet. Create a room comes first.* and the rest), sent to the host page as `409 {reason}` by `popquiz/src/server/routes/mod.rs` ~:56–58 (`RoomError::Refused(reason)`).
- `popquiz/src/quiz_session/lifecycle.rs` ~:100–105, `Ended::host_reason`: two sentences (*This room has ended.* / *This room closed for inactivity.*), sent by `routes/mod.rs` ~:64. Its doc comment calls them API diagnostics for the host page, *not §11 copy*; `phase.rs` says the same of its reasons. PQ-60's ruling moved strings of exactly this kind, so list them and let her decide.
- Codex's pass-1 list: read `lattice comments PQ-60` (the `QUESTION PQ-60 (scope)` comment of 2026-11-03T13:35Z) and the review artifact `art_01M6GXNA2RHC7S7SGJ7D4MSMJ3` on PQ-60 (`lattice show PQ-60`), and add anything there that is still outside `copy.rs`.
- The `const` case: find how `reason_copy.rs` scans (`REASON_SINKS`, `REASON_SOURCES`, `SOURCE_SINKS`, `scan`, `span`) and show, with a planted same-file `const` holding a sentence that flows into a sink, that it passes today. Propose the narrowest rule that catches it (a letter-bearing `const`/`static` string in a covered file that is referenced by a sink; or every letter-bearing sentence `const` in a covered file), and say which existing constants it would flag.
- Also check `web/host/host.js` (~:233 per Codex) for how the host page shows `reason`, and whether `web/shared/copy.js` and `popquiz/tests/twins.rs` / `web/test/copy-freeze.test.js` would need twin entries for moved keys (PQ-60's DONE says what it did for twins; follow it).

**Your plan** confirms every candidate in this worktree with exact lines and strings, groups them by who sees them and the route that sends them, proposes the key names in `copy.rs`'s room-only block (the `HOST_…` style already there), the lint widening (`REASON_SOURCES` += the files; whatever `SOURCE_SINKS` needs for match arms that return `&'static str`), and the `const` rule, then ends in:

```
QUESTION PQ-84 (files): clear for verbatim moves into copy.rs and lint coverage: <file:lines — n strings — seen by host|organizer via route> …; const rule: <one line>. Recommended: all of the above. Reply RESUME PQ-84 <all | the files you clear>.
```

Post it as a comment and as mail (a long question goes in the comment in full and in the mail as `--body-ref <file>` or a pointer to the comment), checkpoint, end your turn. Do not edit source before the reply.

### After `RESUME PQ-84 …`

- **Move verbatim**, only from the files she clears, into the room-only block of `copy.rs`, one `pub const` per sentence, text character for character (the `{id}: {why}` framing in `affirmation.rs` stays a `format!` over the moved `why` constants, or moves whole if she says so — say which in DONE). Files she does not clear stay as they are and go under not-done.
- **Widen the lint.** `REASON_SOURCES` gains each cleared file; extend `SOURCE_SINKS` or the scan so the match-arm shape (`Ended::Expired => "…"`) and `Refused { reason: "…".into() }` are seen; `every_reason_sink_is_still_a_function_the_scan_can_see` and `every_refusal_code_is_still_sent` keep passing.
- **The `const` case.** Implement the rule she accepted; plant a sentence `const` in a covered file → the lint fails; revert.
- **Twins.** If a test holds `copy.rs` and `web/shared/copy.js` to the same keys, add the moved keys to `copy.js` with identical text and say so in DONE; if the room-only block is excluded from the twin test (PQ-60's DONE says), say that instead.
- `just test` green (cargo, web, pipeline).

**Mutations, each caught by the lint, each reverted:** one planted sentence literal in each cleared file at the shape the file uses (a match arm, a `Refused { reason }`, a `format!`); the same-file `const` plant; one moved string in `copy.rs` edited by a character → the test that pins the text (name it; if none pins verbatim text, say so in DONE and add the smallest one that does).

**Acceptance:** the cleared strings live in `copy.rs`; `reason_copy` covers their files and the `const` case; every plant fails it; `just test` green; not-done lists any file she did not clear.

### Cleared files

After her reply: the files she names, plus `popquiz/src/copy.rs`, `popquiz/tests/reason_copy.rs`, and `web/shared/copy.js` for identical-text twin entries only (and `popquiz/tests/twins.rs` / `web/test/copy-freeze.test.js` only if a count or key list there must change). Before her reply: none. Anything else: ask by comment and mail.

### Read before planning

`CLAUDE.md`; `lattice show PQ-84`; `lattice comments PQ-84`; `lattice comments PQ-60` and PQ-60's review artifacts; `popquiz/src/copy.rs` (the room-only block and its header comment); `popquiz/tests/reason_copy.rs` in full; `affirmation.rs` ~:130–150; `phase.rs` ~:360–460; `lifecycle.rs` ~:85–110; `server/routes/mod.rs` ~:40–80; `server/admin.rs` ~:140–200; `host/create.rs` ~:110–135; `web/host/host.js` ~:220–245; `SPEC.md` §11 (what is §11 copy and what is not).

## 3. The arc

**Plan.** `lattice status PQ-84 in_planning`. Write the plan to the absolute path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-84)`, under a `# Plan (delegator, date)` heading, carrying the string table. Then `lattice status PQ-84 planned` and post `PLAN PQ-84 <plan path> <one line>` followed by the `QUESTION PQ-84` block (comment and mail). Checkpoint. **End your turn and wait.**

**Implement** (after `RESUME`). `lattice status PQ-84 in_progress`. `git fetch origin && git rebase origin/main` first. Small signed commits with plain-English messages, one concern each (the moves; the lint widening; the `const` rule); end each with a `Co-Authored-By:` trailer naming the model you are actually running as. Run `just test`. Checkpoint once it is green.

**Code review (Codex, a different model family).** From this worktree, with the sandbox bypass: `codex review --help` to confirm the base option, then a review of `origin/main...HEAD` with a prompt naming her ruling and asking for: any mutation above that still passes, any host- or organizer-visible sentence a cleared file still builds outside `copy.rs`, any moved string whose text changed by even one character, any file edited that she did not clear, any copy twin that now disagrees, any production change beyond moving literals and widening the lint. Verdict PASS / PASS-WITH-NITS / FAIL; Critical/Major/Minor/NIT with file:line. Save the output to `$TMPDIR/pq84-codex-review.txt`. Fix Critical and Major; re-review once. **At most two review passes**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-84 --type note --role review --title "Code review (Codex)" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq84-reviewer`.

**Validate.** `just test` (rerun with the bypass on a loopback `EPERM`). Name what ran and what did not. Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-84`. Confirm `git log origin/main..HEAD` holds only your commits. Push: `git push -u origin ai-c11-cc/pq-84-lifecycle-reason-copy` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/pq-84-lifecycle-reason-copy)"`. `gh pr create --base main --head ai-c11-cc/pq-84-lifecycle-reason-copy`: title in plain words without ticket IDs; body: what was left outside `copy.rs`, her clearance quoted, the strings moved (file → key), the lint's new coverage, the mutation → test table, this line: **a merge is a deploy that restarts the room; if a room has been released since the last sync, run `just sync` first**, tests and counts, *Contract notes*, and the line `Lattice PQ-84`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use the REST API for a long body and verify it with `gh pr view <N> --json body`. Then `~/.claude/scripts/gh-wait-runs.sh <head sha>` (bypass; if denied, say so in DONE). Then **serially**: `lattice attach PQ-84 <PR URL> --type reference --title "PR" --actor agent:delegator-pq84`, then `lattice status PQ-84 review`; verify with `lattice show PQ-84 --json`.

**Completion comment** (comment and mail), then stop:

```
DONE PQ-84 PR <url> HEAD <sha> BASE <origin/main sha> cleared <her reply quoted> moved <n strings from files> test <counts> test-warm <s> ci <result | not gated> review: <verdict, reviewed sha> mutations: <each → the test that caught it> twins: <what changed or why not> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered> not-done: <files not cleared, and anything else left out>
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/pq-84-lifecycle-reason-copy …` or the literal worktree path. **Do not merge**, even if asked in this panel; the client merges or gives the Orchestrator the word. A merge to `main` is a deploy that restarts the room. Write a short checkpoint to `.claude/checkpoint-pq84.md` (uncommitted), set your description to "PR open, waiting", and stop.
