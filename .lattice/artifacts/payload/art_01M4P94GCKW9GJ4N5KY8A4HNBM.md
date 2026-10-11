Verdict: PASS-WITH-NITS, reviewed HEAD 2a64233140a3b7c632ce1b33e5030a387e50716d (opus, Orchestrator exact-head review, read-only). Exact-head CI: gh-wait-runs 1/1 success.

HEAD confirmed. Hermetic run with ANTHROPIC_API_KEY/ANTHROPIC_AUTH_TOKEN unset (test_generate_cli, test_generate, test_bank, test_verify, test_runner): 307 passed, 0 failed. No real claude, no API call.

R1 met: generate.py:1316 default=BATCHES; each factory builds exactly one client type (generate.py:1344-1373); no fallback, tested both directions and after stop (test_generate_cli.py:358-387).
R2 met: generate_cli.py:103-117 Popen argv list, no shell, stdin prompt, fresh temp cwd, start_new_session; no --bare; --tools "" --setting-sources "" --strict-mcp-config --safe-mode --disable-slash-commands --no-session-persistence.
R3 met: child_env generate_cli.py:82-88 drops ANTHROPIC_*, CLAUDE_CODE_USE_*, AWS_BEARER_TOKEN_BEDROCK; test_generate_cli.py:155-196 asserts by name and that the sentinel value appears in no file. Only key read in src is the Batches client's existing one (generate.py:1373). No key logging.
R4 met: cap checked before client construction and on resume, no default (generate.py:1114-1122, 1155-1160); Stop caught generate_cli.py:184-190, no retry; rerun tested test_generate_cli.py:239-262.
R5 met: total_cost_usd per candidate labelled API-equivalent, never summed (generate.py:1015, 1033-1035, 1050-1054).
R6 met: only structured_output read (generate_cli.py:242-250); classify unchanged; same build_requests for both backends (test_generate_cli.py:184-185).
R7 met in test_generate_cli.py: autouse PATH=/usr/bin:/bin, asserts which claude is None, blocks sockets (:115-130); kill test passes the fake via --claude-bin.
R8 met: D-D amendment and T-16 note match the ticket character for character (script-checked); ECONOMICS line is the ticket's wording, capitalised; no edit to SPEC, EVALUATION, DESIGN, PHILOSOPHY or sequence/.
R9 met: pipeline/README.md "On the subscription" covers both backends, the billing trap, no-real-call status.
F1 met (generate.py:751). F2 met (bank.py:604-611, refuses not strips; tests in test_bank, test_verify). F3 met (tests/sandbox/generate/test_kill_rerun.py: real SIGKILL, rerun, hashes bank/, room/ minus target, mvp/answer-history.json). F4 met (Factory, generate.py:557-581). F5 met (generate.py:308, 336).
F6 NOT MET (owed): .env.example line is the client's hand and was not added.
Scope: every file in the diff is cleared; test_runner.py has exactly the one edit (:64-67); justfile has only test-generate-full and the test-full dependency.

Critical: none. Major: none.
Minor:
1. Real-claude guard lives in one file (test_generate_cli.py:115), not a conftest; a future test elsewhere building ClaudeCliClient(None, ...) or passing an absolute --claude-bin would reach the real claude. Nothing does today.
2. SIGTERM/SIGHUP to the generator orphans the child (default SIGTERM skips the except BaseException kill, generate_cli.py:218); child is in its own session, keeps consuming plan usage up to 900 s. README mentions only kill -9.
3. Non-Stop exceptions escape as tracebacks: OSError from Popen (binary removed, E2BIG) or UnicodeDecodeError from communicate(text=True) are not GenerateError, so main (generate.py:1388) doesn't catch them; the decode case calls communicate() again inside _kill. Finished candidates stay cached.
4. DEFAULT_TIMEOUT_S=900 is not settable from the CLI.
NIT:
1. generate.py:1187: claude-cli run killed between "submitting" state and submit is told to "check the Console" about a batch and can't resume under that run id (nothing spent).
2. BUILDPLAN D-D model line says Opus 5.5 but keeps "$5 in / $25 out" beside the amendment's $4/$20; T-16 row still reads "Claude Opus 5". Within ruling 8's literal scope; reconcile upstream.
3. --safe-mode, --setting-sources "", --effort unverified against the real binary (by rule). A rejected flag fails safe: stop at c01, non-zero exit. HC-2 confirms.

Disagreements with the delegator review: it cites generate_cli.py:131-140 for the SIGKILL window (class header; the window is communicate return :214 to write_atomic :192); it misses Minor 2 and Minor 3; it frames the guard gap as absolute-path only (it is also confined to one file); it reports F1-F5 met without flagging F6 as owed.