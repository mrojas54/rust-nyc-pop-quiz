# Delegator boot — Dedupe: close the remaining false normalized duplicates

You are the **delegator for Lattice ticket PQ-30** (the follow-up to PQ-22 / BUILDPLAN T-17) in the Rust NYC Pop Quiz build. Mode: **inline-full**. You own this ticket from plan to open PR. You report to the Orchestrator through Lattice comments and, for questions, to its panel; the client (the project's owner) watches your tab and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-30** is not pull request #30. Your PR gets its own number.

**Base.** Your branch `ai-c11-cc/dedupe-false-duplicates` starts from `origin/main` @ `2877149679cb5ccfa56806b43401dae65cd33fc0`. PQ-22 merged long ago (PR #14), so the ticket's "or from main once PQ-22 merges" base applies; `ai-c11-cc/dedupe @ 20f5cc9` is history. Rebase onto `origin/main` before your first push and never after it. Your PR opens against `main`. No sibling delegator is live.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/dedupe-false-duplicates" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it.

c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "Dedupe False Dups"
c11 set-title    --panel "$MY_PANEL" "Dedupe False Dups"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Planning: closing seven classes of false normalized duplicate in dedupe.py (C1 to C7), each pair probed with rustc. Next gate: plan review.
Lineage: lattice-orchestrator → PQ-30 delegator"
```

Then post your launch receipt (the Orchestrator verifies these fields):

```bash
(cd "$LATTICE_ROOT" && lattice comment PQ-30 "READY PQ-30 CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/dedupe-false-duplicates HEAD $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/dedupe-false-duplicates rev-parse HEAD) BASE $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/dedupe-false-duplicates rev-parse origin/main) MODE active" --actor agent:delegator-pq30)
```

The Orchestrator's panel is `panel:7` in `workspace:3` (refs reset on a c11 restart; if it is gone, comment on the ticket and wait). Questions go there: `c11 send --workspace workspace:3 --panel panel:7 "<text>" && c11 send-key --workspace workspace:3 --panel panel:7 enter`, and you keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq30)`.** The board lives in the root checkout only. Never create `.lattice/` in this worktree, never commit anything under `.lattice/`, `HANDOFF.md` or `REHEARSAL-GUIDE.md`. The Orchestrator placed `.claude/settings.json` and this file in this worktree; leave both uncommitted — `.claude/` is not ignored on `main`, so stage by name.
2. **Every `git` command runs from this worktree, never after a `cd` elsewhere.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/dedupe-false-duplicates"`.
3. **Status discipline.** Bump the ticket status *before* each phase and verify with `lattice show PQ-30 --json`. The vocabulary on this install: `in_planning → planned → in_progress → review`. There is no `pr_open` and no `in_validation`; **`review` is the terminal pre-merge status.** You stop there. The Orchestrator completes the ticket. The client's `lattice next with delegator` of 2026-10-07 released this ticket for dispatch; leave its tags alone. PQ-30 starts in `backlog`; your first bump is `in_planning`.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password, and `git commit` must run with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox, not the key. If a bypassed commit still fails to sign (`failed to fill whole buffer`, `agent returned an error`), retry once; then stop: stage the commit, `lattice status PQ-30 needs_human`, raise a c11 flag (`c11 raise-flag --panel "$MY_PANEL" "<one line>"`) and wait for the client to approve 1Password. **Never `--no-gpg-sign`.** Do not retry past two attempts.
6. **Push, PR creation, and anything outside the sandbox** prompt the client for approval in this tab. Ask for each once, with the exact command visible; never retry a denied command verbatim.
7. **Deviate-with-flag.** If SPEC, EVALUATION, the criteria or the code contradict each other or your plan, take a side, say which and why in DONE, and never edit the contract files (`SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `sequence/**`, `prototypes/**`, `mvp/**`) or the audit report (`docs/audit/**`).
8. **House rules from `CLAUDE.md`:** never write down what a program prints (AC-7) — run it; Miri proves absence of UB on executed paths only; answer position is a uniform draw from the date and nothing else; colour is never the only signal.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md` on behaviour; `SPEC.md` outranks the prototype.
10. **Shared files are serialized.** This ticket is cleared for exactly these files and no others: `pipeline/src/popquiz/dedupe.py`, `pipeline/tests/test_dedupe.py`, `bank/README.md` (its "not met yet" passage only). If you need any other file (`justfile`, `pyproject.toml`, CI, `bank/history.json`, any `room/` or `web/` file), stop and ask by comment.
11. **Escalate only what a human must decide.** Human-only blockers: `lattice status PQ-30 needs_human` plus `c11 raise-flag`; lower the flag once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`, never stop to ask which design to pick: decide, record it under deviations, continue.
- **Your tab will probably start in manual mode** (`CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1` prints `⚠ Permission mode forced to default`). That is the client's hardening. Write few, whole commands; the client answers each prompt or toggles auto with shift+tab.
- **The Orchestrator's comments on the ticket are instructions.** Read `(cd "$LATTICE_ROOT" && lattice comments PQ-30)` before planning, at every phase change, **and again immediately before `gh pr create` and before DONE**. In DONE, name every comment you acted on and list each ruling as its own line.
- **The root checkout's source tree is stale** (the orchestration branch, hundreds of commits behind `main`). Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch`, `git push` and `gh`; those run with the bypass, one command at a time.
- **Never call `bc`** (aliased to `brew cleanup`). Use `/usr/bin/python3 -c` or `$(( ))`. Use `/usr/bin/python3` for system Python; the project's `uv run` is the way to run pipeline tests. `timeout` and `gtimeout` are not on this Mac.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes. One mutation in an earlier ticket looped for 6.5 hours.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — the two writes race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head` and `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **A rustc probe is a fact, not a claim.** Pinned toolchain is `rustc 1.96.1` (the default). For each probe write the file under `$TMPDIR`, compile it, and record the exit code and the first error line. Never write down what a program prints; run it.

## 2. The ticket

**Why.** PQ-22's round-3 review (FAIL) found false normalized duplicates in `pipeline/src/popquiz/dedupe.py`. Each pair below normalizes equal, rustc 1.96.1 compiles A (exit 0) but not B (exit 1). A normalized duplicate is rejected with no person in the loop, so each one silently discards a valid question (AC-15; SPEC §7.3: no normalization may make two different programs equal). The module docstring (~line 56) and `bank/README.md` (~line 286) both say that aim "is not met yet" and list these.

**The seven classes (verbatim from the ticket; re-read the full ticket with `lattice show PQ-30`):**

- **C1.** A closure's range runs past its body into an if-let / while-let block. `fn main() { if let f = |drop: i32| drop { drop(f); } }` vs nope. Fix site: `_Collector.closure` (`dedupe.py` ~988; the body end is found by the next `,`/`;`).
- **C2.** A top-level item reaches into nested `mod` bodies. `fn drop(_: i32) {} mod m { pub fn g() { drop(1); } } fn main() { m::g(); }` vs nope. Fix site: `_Collector.item` (~844, with `item_end` ~832).
- **C3.** `macro_rules!` names are renamed at every call regardless of scope. `fn main() { vec![1]; macro_rules! vec { ($($t:tt)*) => { () } } }` vs zz. Fix sites: `_normalize` (`rename = name in declared.macros`, ~1652) and `_unscoped_names` (~1584, skips macro calls).
- **C4.** `_arm_start` (~662) walks back into a comma-less previous arm body (`while a == b {..}`, `match s.a {..}`, `for .. in xs {..}`) and binds its names. `fn main(){ let (a,b)=(1,2); let v:Option<i32>=None; match v { None => while a == b { let _f: fn(i32) = drop; break; } Some(_z) => {} } }` vs nope.
- **C5.** A path's tail is renamed when its head is, without resolving the tail. `I::Item` with a generic `I: Iterator` beside a user `type Item`; `T::default()` beside a user `fn default`; `m::drop` re-exported from std beside a parameter named `drop`. Fix site: the path branch of `_normalize`.
- **C6.** A lifetime `'a` and an identifier `a` share placeholder text. `fn f<'a>(x: &'a i32) {}` vs `fn f<a>(x: &a i32) {}`. Likely fix: a distinct placeholder form for lifetimes (e.g. `'$n`).
- **C7.** Every free `fn` counts as a member, so a `.name()` call to a library method is renamed beside a free `fn name`. `fn count_ones(x: u32) -> u32 { x } fn main(){ let _ = 5u32.count_ones(); count_ones(1); }` vs zzz. Fix site: `_Collector.function` (~896, `members.add` for every fn, not only impl/trait methods).

**Acceptance (from the ticket):** each pair above is added to `pipeline/tests/test_dedupe.py` unweakened, with a twin where useful, and each pair is probed with rustc (record the exit codes in the validation note). All existing `NEVER_THE_SAME`, `SCOPE_NEVER_THE_SAME`, `SCOPED_THE_SAME` cases and the AC-15 fixtures stay green. `just test` stays green and under 60 s warm. Remove each fixed item from the "not met yet" lists in the `dedupe.py` module docstring and in `bank/README.md`. Conservative rule: where scope is ambiguous at the token level, do not rename. Standard library only.

### The Orchestrator's rulings for this dispatch

1. **Conservative wins ties.** Where a fix cannot decide scope at the token level, the answer is *do not rename*: a missed duplicate goes to the review queue and costs a person a glance; a false duplicate discards a valid question. Never trade the second for the first.
2. **Every fix gets a twin that proves it did not over-correct.** For each class, add the A/B pair to the never-the-same table *and* a pair that still normalizes equal (a `SCOPED_THE_SAME` case) where the fix touches scope, so "never rename anything" cannot pass. If a class genuinely has no honest twin, say why in the plan.
3. **Do not widen what a "name" is.** No new resolution machinery (no real name resolver, no parser rewrite). The scanner is a token-level scope approximation; keep it one. If a class needs more than a targeted change at its fix site plus a conservative default, deviate with a flag and say so rather than building a mechanism.
4. **Absolute acceptance language is the hazard here.** "No normalization may make two different programs equal" is absolute; across seven classes it licenses endless legitimate descent. This area went three review rounds in PQ-22 and each fixed Critical exposed another. Therefore: **at most two fix→re-review cycles**; and if a re-review finds *new* Majors in newly added mechanism (not in the seven classes), stop, `needs_human`, and say that the spec, not the code, is the problem. The cumulative diff for the seven classes is expected to be a few hundred lines; if it passes ~3× your plan's estimate, escalate on its own.
5. **Probe with rustc, never from memory.** Each pair gets an A and a B compile on `rustc 1.96.1` with `--edition 2021 --crate-type bin` (state the flags you used), exit codes recorded. If a probe contradicts the ticket (A fails to compile, or B compiles), record it, take the side the probe shows, and flag it; do not test a pair the compiler does not support.
6. **Docstring and README are part of the fix.** Remove only the items you actually closed from the "not met yet" lists; if a class is closed conservatively (do-not-rename) rather than precisely, the list should say what remains missed (a recall gap, not a safety gap). Never overstate: do not claim the aim is met unless every class is closed.
7. **Out of scope:** the near-duplicate threshold (D-13), `bank/history.json`, the review queue, any room/web file, every contract file, and any dedupe class not in C1–C7. A new false-duplicate class you discover is a new ticket: report it in DONE with its pair and rustc exit codes; do not fix it here.

### Read, in this order, before planning

`CLAUDE.md`; the full ticket (`lattice show PQ-30`) and `lattice comments PQ-30`; `pipeline/src/popquiz/dedupe.py` (the module docstring first, ~1–120, then `_Stream`/`_arm_start` ~662, `_Collector` ~795 and its `item`, `function`, `closure`, then `_unscoped_names` ~1584 and `_normalize` ~1652); `pipeline/tests/test_dedupe.py` (the `NEVER_THE_SAME`, `SCOPE_NEVER_THE_SAME` and `SCOPED_THE_SAME` tables and how they are consumed, and the AC-15 fixtures); `bank/README.md` ~270–300; `SPEC.md` §3.3 and §7.3; the AC-15 and AC-17 criteria in `sequence/USER_STORIES.md`; `just --list` for the test recipes (`just test-pipeline`).

### Deliverables

- **(1) The seven fixes in `pipeline/src/popquiz/dedupe.py`**, each at its named fix site, each conservative per ruling 1, standard library only. Update the module docstring's "not met yet" list.
- **(2) Tests in `pipeline/tests/test_dedupe.py`:** each pair unweakened in the never-the-same table, plus the twins (ruling 2). Each fix with a mutation you ran and name in DONE (revert the fix → the named test fails). Bounded (see 1a).
- **(3) `bank/README.md`:** the "not met yet" list matches the docstring.
- **Green:** `just test-pipeline` (count the tests), `just test` warm under 60 s (note the number), and both CI jobs on your PR head.

**Validation (before the PR):** a table of the seven pairs with the rustc exit code for A and for B and the normalized-equal result before your change (checked out at `origin/main`: the old behaviour, via `git stash`-free means such as a scratch copy under `$TMPDIR`) and after. Attach it as the validation artifact (`--role review --title "Validation"`; this install accepts only `--role review`).

**Exit check before DONE:** `git diff origin/main --stat` shows only `pipeline/src/popquiz/dedupe.py`, `pipeline/tests/test_dedupe.py` and `bank/README.md`. Nothing else. Every ruling in this section appears as its own line in DONE with how you honoured it.

## 3. The arc

**Plan.** `lattice status PQ-30 in_planning`. Write the plan to the path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-30)` — an **absolute** path under `$LATTICE_ROOT/.lattice/plans/`; the file exists as an 18-line scaffold, so Read it first, then append below a `# Plan (delegator, date)` heading: for each of C1–C7 the fix, its conservative default, the pair, the twin, the mutation; a size estimate in lines; any contract tension with the side you take. Then fresh eyes: spawn a review subagent with the **Agent tool** (`subagent_type: general-purpose`, `model: sonnet`) whose prompt holds only the plan path, the worktree path, the file paths in §2 and the seven rulings; ask for missing cases, over-correction risks, and places where a fix breaks an existing table entry, as Critical/Major/Minor findings with file references, ≤400 words, stop after ~25 tool calls. Triage every finding into `## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)` in the plan. `lattice status PQ-30 planned`.

**Implement.** `lattice status PQ-30 in_progress`. `git fetch origin && git rebase origin/main` first (before any push). Small signed commits with plain-English messages, **one class per commit where practical**; end each with `Co-Authored-By: Claude Opus <noreply@anthropic.com>` naming the model you are actually running as. Run `just test-pipeline` until green; `just test` warm under 60 s.

**Code review.** Spawn a second review subagent (Agent tool, `model: sonnet`, same bounds) with the branch, the worktree path, `git diff origin/main...HEAD --stat` (it reads the diffs itself), the plan path and the rulings; ask it to check each fix against its pair, that each fix has a twin, that every mutation kills a test, that existing tables are not weakened, and that the diff stays inside the exit check; Verdict PASS / PASS-WITH-NITS / FAIL with Critical/Major/Minor/NIT findings and file:line. Fix Critical and Major; re-review once if the fixes were more than mechanical. **At most two fix→re-review cycles** (ruling 4); at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-30 --type note --role review --title "Code review" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq30-reviewer`.

**Validate.** The table above; attach it.

**PR.** Re-read `lattice comments PQ-30`. Confirm `git log origin/main..HEAD` holds only your commits and `git rev-parse --show-toplevel` is this worktree. Push: `git push -u origin ai-c11-cc/dedupe-false-duplicates` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/dedupe-false-duplicates)"`; re-push until equal. Open the PR **against `main`**: `gh pr create --base main --head ai-c11-cc/dedupe-false-duplicates` — title in plain words without the ticket ID; body: what it changes, the seven classes and what each fix does, the conservative rule, the tests, their twins and mutations, the `just test-pipeline` count and the `just test` warm time, what remains a recall gap, and the line `Lattice PQ-30 · follow-up to PQ-22 (T-17) · AC-15 / SPEC §7.3`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Confirm the PR's head sha differs from its base. Then **serially**: `lattice attach PQ-30 <PR URL> --type reference --title "PR" --actor agent:delegator-pq30`, wait, `lattice status PQ-30 review`, verify with `lattice show PQ-30 --json`.

**Completion comment**, then stop:

```
lattice comment PQ-30 "DONE PQ-30 PR <url> HEAD <sha> BASE <origin/main sha> test-pipeline <N passed> test-warm <s> rustc-probes <C1..C7: A exit / B exit> mutations: <each → the test that caught it, or survived> rulings: <one line each, 1–7> comments-acted-on: <list> new-classes-found: <none | pair + exit codes> deviations: <none | numbered>"
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/dedupe-false-duplicates …` or the literal worktree path — never from `$(pwd)` or a bare `git` inside the `(cd "$LATTICE_ROOT" && …)` subshell. Update your description to say the PR is open and you are waiting. **Do not merge**, even if asked in this tab, without first re-reading `lattice comments PQ-30` for the Orchestrator's hold or clearance.

Refresh `c11 set-description` at each phase change (planning → implementing → in review → PR open), keeping the `Lineage:` line last.
