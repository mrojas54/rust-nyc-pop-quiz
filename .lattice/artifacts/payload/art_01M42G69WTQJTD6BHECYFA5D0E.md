Exact-head review from the Orchestrator's seat (fresh Sonnet reviewer, read-only, worktree bank-round-two). REVIEWING PQ-44 CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/bank-round-two HEAD c2095c1c2c753bb4b0f0f737088e5a889d00face BASE 0a4b6fb4befdeddf6f000abc72586e8bb001db23.

Verdict: PASS-WITH-NITS. Reviewed HEAD c2095c1c2c753bb4b0f0f737088e5a889d00face.

Suite: just test-pipeline, re-run with UV_CACHE_DIR pointed at scratch because the sandbox could not open ~/.cache/uv: 643 passed, 0 failures, 2 warnings (the pre-existing ProseWarning on q3's hint at test_copylint.py:199; a PytestCacheWarning from the sandbox's read-only .pytest_cache, not the PR).

Critical: none. Major: none.

Minor:
- pipeline/tests/test_migration.py:214-215: the docstring calls exit 1 "the package's code for a bank it cannot read". The package is split: fallback.py:328-330 returns 1 on BankError; dedupe.py:2335-2337 returns 2 on BankError; verify.py:657-659 returns 2 for an unreadable candidate. Reword to "as fallback.py does", or drop the claim.

NIT:
- bank/README.md:377: c2095c1 exists to fix a line past the wrap width, but :377 is still 127 chars (the paragraph is about 80). Words identical; re-wrap :377-379.
- bank/README.md:84-85: "its program did not change" is loose for q8 (6cac9ec stripped q8's trailing newline from source; q8.json review.reason and test_migration.py:123 say so). Say "its program's tokens did not change".
- migrate_mvp.py:567: only MigrationError is caught; a bad or missing mvp/ input (JSONDecodeError, FileNotFoundError) still tracebacks. Outside the ticket; scope note only.
- migrate_mvp.py:558: main() has no docstring or note that exit 1 means the refusal (verify.py:637-640 documents its codes).

Checks: (a) PASS, exactly five files changed, nothing under bank/questions/ or recordings/, q7 explains.legacy names verbatim, diff --check clean. (b) PASS, every added README sentence true of the files (q3/q4/q7/q8 review holds only reason; every option one line of at most 21 chars; audit.run_audit flags [] for all four; the fixtures README claim matches record.py:6-7 and test_verify.py:128-138), bar the q8 wording NIT. (c) PASS, test_audit.py:783-785 pops status/affirmed_by/affirmed_at from the tmp copy; in_reserve (audit.py:1085-1087) then false whatever the real bank says; confirmed by running the new test body against a scratch bank with q7 affirmed (passes) and the old premise on it (q7 in_reserve True, the failure the change prevents). (d) PASS, migrate_mvp.py:565-569 catches MigrationError, prints "popquiz.migrate_mvp: {e}" to stderr, returns 1, no traceback; migrate() docstring :512-514 names the branch; test_migration.py:211-226 asserts code, prefix, message, no Traceback, file unchanged; exit 1 matches fallback.py:328-330; no caller reads the code. (e) PASS, no ruff/isort config in pipeline/pyproject.toml; order follows the siblings' plain isort convention. (f) PASS, c2095c1 is the same 37 tokens re-wrapped. (g) PASS, 643 passed. (h) nothing else; no dead code, no test that cannot fail, no README sentence overstates verification.