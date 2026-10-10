# Delegator boot — q3's affirmation lands, and the tests that pinned the unaffirmed bank move with it

You are the **delegator for Lattice ticket PQ-67** in the Rust NYC Pop Quiz build. Mode: **fast-track** (single session, inline self-review; the Orchestrator runs the exact-head review). You own this ticket from plan to open PR. You report to the Orchestrator through Lattice comments and, for questions, to its panel; the client (the project's owner) watches your tab and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-67** is not pull request #67.

**Base.** Branch `ai-c11-cc/affirm-q3`, already checked out in this worktree, off `origin/main` @ `e4156ed` (the merge of PR #57). **The worktree already holds one uncommitted change, made by the Orchestrator on the client's word: `bank/questions/q3.json`'s `review` block.** Do not discard it. Rebase onto `origin/main` before your first push and never after it. Your PR opens against `main`. One unrelated PR is open: #59 (four new candidate questions). If it adds `bank/questions/*` records that change the reserve or trend counts your tests assert, say so in DONE; do not touch its files.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/affirm-q3" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)" && git status --short
```

If the first line fails, HALT and say so. `git status --short` must show ` M bank/questions/q3.json` (and possibly `.claude/` files); if it does not, HALT and comment.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "Affirm q3"
c11 set-title    --panel "$MY_PANEL" "Affirm q3"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Planning: land the client's q3 affirmation (F-52) and move six tests that pinned the unaffirmed bank. Next gate: self-review.
Lineage: lattice-orchestrator → PQ-67 delegator"
(cd "$LATTICE_ROOT" && lattice comment PQ-67 "READY PQ-67 CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/affirm-q3 HEAD $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/affirm-q3 rev-parse HEAD) BASE $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/affirm-q3 rev-parse origin/main) MODE active" --actor agent:delegator-pq67)
```

c11 calls and `git fetch` need the sandbox bypass. The Orchestrator's panel is `panel:7` in `workspace:3`. Questions: `c11 send --workspace workspace:3 --panel panel:7 "<text>" && c11 send-key --workspace workspace:3 --panel panel:7 enter`.

## 1. Ground rules

The standing clauses of this run apply exactly as written in `$LATTICE_ROOT/.lattice/orchestration/templates/boot-PQ-48.md` §1 and §1a — read that file's §1 and §1a now, substituting **PQ-67** for PQ-48, actor `agent:delegator-pq67`, and this worktree's path. In short: every `lattice` command as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq67)`; stage by name; signed commits with the sandbox bypass, never `--no-gpg-sign`, two attempts max; push and PR need the client's approval in this tab; never edit contract files; `review` is the terminal status; read `lattice comments PQ-67` before planning, before `gh pr create` and before DONE; no `grep`/`cat`/`head`/`sed -n`; bounded mutations (kill at 15 min).

## 2. The ticket

Read `lattice show PQ-67`. **F-52** was ruled 2026-10-10 by the client: q3 is affirmed by the organizer's documented edit of its `review` block, and the client said "affirm q3 queenrose54". The Orchestrator made that edit. With it, `just schedule q3 --date 2026-10-21 --no-push --out "$TMPDIR/x"` reports `reserve (nyc): 1 ready` (0 on `main`) and `just bank-audit` passes. Six pipeline tests pin the pre-ruling state and fail (names in the ticket).

### Rulings

1. **The affirmation is the client's act.** Do not change `status`, `affirmed_by` or `affirmed_at` in q3.json. Commit that file **alone, first**, message: `Affirm q3: the organizer's documented edit (F-52)`.
2. **The migration golden tests** compare the committed record with what a fresh migration writes. `review` is not the migration's to own once an organizer has written it (schema: *written by the review surface*). Compare everything **except** what an organizer may write in `review` (`status`, `affirmed_by`, `affirmed_at`, `difficulty_judged`, and the `reason` an organizer replaces); keep every other byte compared. Confirm, and show by a test, that a migration rerun over the committed bank does **not** overwrite q3's affirmation. If it would, stop and comment: that is a production bug, not a test change.
3. **`test_nothing_migrated_is_affirmed`** keeps its guard on the **migration's output** (a fresh run writes no affirmation, for every id including q3). It stops asserting that the committed q3 is unaffirmed.
4. **AC-72 tests.** The refusal test keeps refusing an unaffirmed question with nothing sent or written; use q4 (or a fixture) instead of q3. `test_ac72_every_committed_record_is_refused_today` becomes data-driven: every committed record is refused **iff** it lacks an affirmation, and q3 schedules. Do not hard-code "q3 is the only one".
5. **AC-75 trend.** Derive the expected counts from the committed bank or a fixture, not from hand-typed numbers that the next affirmation breaks again. The reserve line is `1 ready` on this branch.
6. **No production code.** Cleared files: `bank/questions/q3.json` (already edited; commit only), `pipeline/tests/test_migration.py`, `pipeline/tests/test_schedule.py`, and a new `pipeline/tests/` fixture file if you need one. Anything else: stop and comment.
7. **Mutations** (name each and the test that caught it in DONE): q3 without `affirmed_at` → a test fails; a migration that writes `affirmed_by` → `test_nothing_migrated_is_affirmed` fails; the golden compare ignoring a non-`review` field → a test fails.

### Green

`just test-pipeline` (count), `just test` warm under 60 s (time and count), `just bank-audit` passes, the `--no-push` dry run above prints `1 ready` and writes nothing tracked (`git status --short` clean after it, apart from `.claude/`).

## 3. The arc

`lattice status PQ-67 in_planning` → a short plan appended to the path `lattice plan PQ-67` prints (which test changes how, the rerun-overwrite check, the mutations) → `planned` → `in_progress` → commits (q3 first, then tests) → inline self-review of `git diff origin/main...HEAD` against rulings 1–7 → push `git push -u origin ai-c11-cc/affirm-q3` (client approves) and verify remote = local → `gh pr create --base main --head ai-c11-cc/affirm-q3`, title *Affirm q3 for the first meetup from the built app*, body: the client's ruling and affirmation, the reserve 0 → 1, the test moves and why each still guards what it did, mutations, counts, the line `Lattice PQ-67 · F-52`, ending with `🤖 Generated with [Claude Code](https://claude.com/claude-code)` → serially `lattice attach PQ-67 <PR URL> --type reference --title "PR"`, then `lattice status PQ-67 review`.

```
lattice comment PQ-67 "DONE PQ-67 PR <url> HEAD <sha> BASE <sha> test-pipeline <N> test-warm <s> bank-audit <result> dry-run <reserve line> mutations: <each → test> rulings: <1–7, one line each> comments-acted-on: <list> deviations: <none | numbered>"
```

Shas from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/affirm-q3 …` only. **Do not merge.**
