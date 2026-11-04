# PQ-82: Distribution scan skips untracked .claude/ files

Found by the PQ-47 delegator on 2026-11-03, after PQ-62 (PR #125) merged. distribution_lint in pipeline/src/popquiz/audit.py walks the working tree, so in a delegator worktree it reads the untracked .claude/ files (checkpoint-*.md, boot-prompt.md) and flags the test counts written in them as distribution tells. CI is unaffected (those files are never committed), but a local just test in a worktree can fail. Fix: add .claude to DISTRIBUTION_SKIP_DIRS (audit.py:899) or walk only git-tracked files; decide which with the client. Acceptance: a planted .claude/checkpoint.md containing a percentage in a temp repo is not reported, a test fails without the fix, real-tree scan still 0 hits / 0 unreadable, just test green. Touch only audit.py and test_audit.py.

# Plan (delegator, 2026-11-04)

## Facts re-verified in the worktree at 0ca67a1
- `git ls-tree -r origin/main` lists seven committed `.claude/` files: `agents/refactoring-specialist.md`, `agents/rust-specialist.md`, `hooks/install-flyctl.sh`, `hooks/install-flyctl.test.sh`, `hooks/install-just.sh`, `hooks/install-just.test.sh`, `settings.json`. `.gitignore:41` ignores only `.claude/settings.local.json`.
- `participant_facing_files` (`audit.py:997`) prunes `DISTRIBUTION_SKIP_DIRS` (`:899`) at any depth and `bank` at the root. Real-tree run here: 343 files scanned, 0 hits, 0 unreadable, and the scanned list includes the untracked `.claude/boot-prompt.md` (it trips nothing only because it holds no percentage beside a kind word; a checkpoint with test counts does).
- `test_runner.py:46,64` forbids `subprocess` in `src/popquiz` and `tests` except `MAY_START_A_PROCESS` = {`sandbox.py`, `generate_cli.py`}; the check is two-way.
- Every distribution test in `test_audit.py` (`:500-650`) builds a `tmp_path` tree that is not a git repository.

## Option A (recommended): `.claude` joins `DISTRIBUTION_SKIP_DIRS`
- Diff: `+ ".claude",` in the set, and the comment above it amended (it says "Nothing in them is authored"; `.claude/agents/*.md` is): build output, caches, and the agent harness's own directory, none of it served, rendered or read by a participant.
- Leaves coverage: the seven committed files above, plus any `.claude/` at any depth (M3).

## Option B: walk only git-tracked files
- Diff: `participant_facing_files` lists `git ls-files -z` via `subprocess` when `repo` is a git work tree, else falls back to today's walk; `audit.py` added to `MAY_START_A_PROCESS` with a reason.
- Costs: a new process-starting module in the pipeline; the `tmp_path` tests only exercise the fallback walk, so the git path needs a test that runs `git init` (another subprocess, in `tests/`); a file not yet `git add`ed escapes the local scan until CI.

## After the ruling
1. RED: add `.claude/checkpoint.md` to `test_skipped_and_exempt_paths_are_not_read` (TELL content, 0 unreadable). Fails today on `(hits, scanned, unreadable) == ([], [], [])`.
2. Fix in `audit.py` only (plus the allowlist line under B).
3. GREEN; real-tree scan 0/0 with boot-prompt and checkpoint present; `just test`.
4. Mutations: M1 revert -> new case fails; M2 `.claude-notes/checkpoint.md` -> caught (add to `test_a_tell_anywhere_authored_is_caught`); M3 under A `docs/.claude/x.md` is skipped (reported, not fought).
5. Codex review, validation, PR, `review`.
