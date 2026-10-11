# PQ-69: Generator: a claude -p client on the subscription, beside Message Batches

Client ruling 2026-10-11: the generator gets a second backend, headless Claude Code (claude -p) on the client's subscription. Message Batches stays. AC-1-5 apply unchanged (SPEC 7.1 names no transport).

Rulings:
1. One more client behind PQ-21's submit/poll/results seam. Backend is an explicit config value or flag (batches | claude-cli). Batches stays the default. Neither backend ever falls back to the other.
2. One process per candidate: claude -p --model <the one model config value> --output-format json --json-schema <the candidate schema>, prompt on stdin, cwd an empty temp dir, no tools (--tools ""), --no-session-persistence. No hooks, CLAUDE.md, MCP servers or plugins may load. Check which flags achieve that against claude --help on this machine and record the check in the PR. Never --bare: it uses only ANTHROPIC_API_KEY and skips the keychain, so it cannot reach the subscription.
3. Billing guard: the child env is a copy with ANTHROPIC_API_KEY (and ANTHROPIC_AUTH_TOKEN) removed. If it is present, claude -p bills the API instead of the plan. A test asserts the child never sees it. The code never reads or logs the key.
4. Usage cap: a required max-candidates-per-run with no default. Refuse before the first call if it is unset. A usage-limit or rate-limit error stops the run cleanly: completed candidates are written atomically and a rerun picks up the rest (AC-2).
5. AC-5 report: tokens and wall-clock as measured. Cost is claude's total_cost_usd, labelled API-equivalent, not billed. Never present it as spend.
6. Unchanged from PQ-21: model-claimed output never lands in an answer field; verify writes the stdout row; capacity facts and the 11.1 patterns are in the brief; report, never trim.
7. Hermetic tests: a fake claude executable on PATH, replaying the fixtures. No real claude -p call from just test, CI or the delegator. The first real run is the client's, and it is reviewed under HC-2.
8. Contract, in this PR: replace BUILDPLAN D-D's first paragraph's model line and add this amendment under D-D, verbatim: '*Amended 2026-10-11 (client).* The model is Claude Opus 5.5 (claude-opus-5-5, $4 in / $20 out per million tokens list). A second backend runs headless Claude Code (claude -p) on the client's subscription, behind the same seam: no API key and no batch discount; it consumes plan usage, so its cap is a candidate count and its per-candidate cost is reported as API-equivalent, not spend. The child process never sees ANTHROPIC_API_KEY, or the run bills the API. Neither backend falls back to the other.' Update the T-16 row's human-track note to 'API key and spend cap (batches) or a logged-in claude CLI (claude-cli)'. Add one line to ECONOMICS: subscription runs consume plan usage, not dollars.
9. A short operator note in the pipeline README covering how to run each backend, and the ANTHROPIC_API_KEY billing trap.

Serialized with PQ-68 (held) on generate.py and pyproject.toml.
