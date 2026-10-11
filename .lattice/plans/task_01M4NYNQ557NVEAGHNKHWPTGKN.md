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

# Plan (delegator-pq69, 2026-10-11)

**Deliverable, one sentence:** one PR against `main`, titled "Add a claude -p subscription backend to the generator, and close the PR #61 review items". It adds `pipeline/src/popquiz/generate_cli.py` beside Batches and fixes F1–F5. It is proven on a fake `claude` only, and `just test` is green.

Base `origin/main` @ 170cfd4. Worktree `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-claude-cli`.

## Sources relied on
- `claude --help` (2.1.296, this machine):
  - `--tools <tools...>`: `""` disables all tools.
  - `--setting-sources <sources>`: user, project, local.
  - `--strict-mcp-config`: only `--mcp-config` servers load.
  - `--safe-mode`: CLAUDE.md, skills, plugins, hooks, MCP, custom commands and agents are disabled; "Auth, model selection … work normally".
  - `--disable-slash-commands`.
  - `--no-session-persistence` (print only).
  - `--output-format json`, `--json-schema <schema>`, `--system-prompt <prompt>`, `--effort <level>`, `--model <model>`.
  - `--bare`: "Anthropic auth is strictly ANTHROPIC_API_KEY … (OAuth and keychain are never read)", so it is never used.
- Output shape: Claude Code docs (headless, agent-sdk SDKResultMessage, authentication, errors), per a claude-code-guide subagent.
  - Fields: `type:"result"`, `subtype` (`success` | `error_max_turns` | `error_during_execution` | `error_max_budget_usd` | `error_max_structured_output_retries`), `is_error`, `result`, `structured_output` (where `--json-schema` output lands), `duration_ms`, `duration_api_ms`, `num_turns`, `total_cost_usd`, `usage`, `modelUsage`, `session_id`.
  - Unconfirmed: the key names inside `usage`, and the exact JSON of a usage-limit failure. The docs give only the strings "You've hit your session limit · resets …" plus non-zero exit or `is_error`.
  - **The fixtures are documented, not observed, until HC-2.** The code docstring, the fixture README and the PR say so.
- Auth precedence (docs): Bedrock/Vertex/Foundry switches > ANTHROPIC_AUTH_TOKEN > ANTHROPIC_API_KEY > apiKeyHelper > CLAUDE_CODE_OAUTH_TOKEN > subscription login. "In -p the key is always used when present."

## 1. The client (`generate_cli.py`, rulings 1–3, 7)
`ClaudeCliClient` implements PR #61's `BatchClient` seam (submit / poll / results):
- `submit(requests)` keeps the requests and returns `cli-<run_id>`. Nothing starts.
- `poll` returns `ended`.
- `results()` is lazy. It runs **one `claude` process per request, in order**, translates each JSON result into the SDK batch-result shape, and the existing `classify` reads it. There is one classifier and one brief builder: the client consumes the `params` that `build_requests` made (system, user message, schema, model, effort).

**argv.** Built as a list and passed to `subprocess.run`, never through a shell:

    [bin, "-p", "--model", model, "--output-format", "json", "--json-schema", json.dumps(schema), "--system-prompt", system, "--effort", effort, "--tools", "", "--setting-sources", "", "--strict-mcp-config", "--safe-mode", "--disable-slash-commands", "--no-session-persistence"]

- The prompt goes on stdin.
- `cwd` is a fresh `tempfile.TemporaryDirectory()` per candidate.
- A per-candidate timeout (900 s). A timeout kills that child and stops the run.

**env (billing guard).** The child gets `{k: v for k, v in os.environ.items() if k not in STRIPPED}`, where STRIPPED is:
- `ANTHROPIC_API_KEY` and `ANTHROPIC_AUTH_TOKEN`
- `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX` and `CLAUDE_CODE_USE_FOUNDRY`
- `ANTHROPIC_BASE_URL`, which reroutes requests.

`CLAUDE_CODE_OAUTH_TOKEN` is kept: it is the subscription path. `apiKeyHelper` is a settings key, and `--setting-sources ""` plus `--safe-mode` keep user settings from loading it. The client never reads a value by name.

**Binary.** `shutil.which("claude")` or an explicit `--claude-bin`. The report records the resolved path. If none is found, the run refuses in one sentence.

**No fallback.** The `--backend batches|claude-cli` flag picks exactly one factory; the default is `batches`. No code path builds the other client after a failure. A test makes the CLI client fail and asserts `AnthropicBatchClient` was never built, and the reverse.

**Cross-flag refusals.**
- `--cap-usd` with `claude-cli` is refused ("a claude-cli run is capped by --max-candidates, not dollars").
- `--max-candidates` with `batches` is refused.
- `--replay` with `claude-cli` is refused.

## 2. Usage cap and stop (ruling 4, AC-2)
- `--max-candidates N` is required for `claude-cli` and has no default. It is checked in `run()` before the client is built, on new and resumed runs alike:
  - Unset: "Refusing to start: no candidate cap is configured; a claude-cli run needs --max-candidates."
  - Over the cap: "Refusing to start: the run asks for 12 candidates, over the 10-candidate cap." It is refused, never trimmed.
- **Stop rule.** A non-zero exit, `is_error: true`, a timeout or unparseable stdout stops the run at that candidate. There is no retry and no fallback.
  - The candidate is reported with the reason. The usage-limit or rate-limit text is quoted as the CLI gave it, trimmed to one line.
  - Later candidates are reported as "not run: the run stopped at cNN; rerun with the same --run-id".
  - The report gets a top-level `stopped` sentence, which `print_report` prints first. The exit code is 1.
- **Resume.** Each successful child's translated result is cached atomically at `<run>/_run/cli/<custom_id>.json`. A rerun under the same `--run-id` serves the cached results without spawning and runs only the rest.
  - Candidate records are written atomically by `run()`, as they are today.
  - `sweep_partials` also covers `_run/cli`.

## 3. Report (ruling 5, AC-5, F4)
- Each client class declares `numbers`:
  - `ReplayBatchClient`: `"fixture"`
  - `AnthropicBatchClient`: `"measured"`
  - `ClaudeCliClient`: `"api-equivalent"`
- `run()` drops the `fixture=` parameter and takes `numbers` from the factory, which carries the client type (`client_factory.numbers`, set by a small `factory(cls, …)` helper). That is how F4 derives fixture-ness from the type.
- Spend is checked only for batches: a cap is needed when `numbers == "measured"`. The fixture rule is unchanged.
- The state identity keeps `backend` and `numbers` in place of `fixture`, so a fixture run id still never resumes a real run.
- **CLI rows:**
  - `input_tokens` and `output_tokens` come from the JSON's `usage`.
  - `wall_clock_s` is `duration_ms / 1000`, measured per candidate.
  - `api_equivalent_usd` is `total_cost_usd`.
  - There is no `cost_usd` and no summed dollar total: the totals carry tokens only.
  - `numbers_note`: "API-EQUIVALENT, NOT BILLED: tokens, wall-clock and total_cost_usd as the claude CLI reported them; the run consumed subscription usage, not dollars". The report also records `claude_bin`.
- Batches rows are unchanged.

## 4. PQ-68 items
- **F1.** In `_read_trace`, a non-`stdout` value whose `now` equals the text of any option of kind `output` is a model-claimed output under another name.
  - It is discarded and reported, the same way as the `stdout` discard.
  - Every remaining value's `now` is added to the report row's `quoted_spans`, so AC-73's review comparison sees it.
  - Mutation: drop the option-text comparison. Expected kill: `test_a_value_naming_an_option_text_is_discarded`.
- **F2.** Chosen: **refuse in `bank.validate_question`.** A `stdout` row (stripped, lower-cased name) on any step but the last is rejected.
  - Why: only the verifier writes `stdout`, and only on the last step, so a row anywhere else was never observed. Stripping it would delete authored content silently ("report, never trim").
  - The bank holds 4 records and none has such a row, so nothing breaks.
  - `verify.with_record` output therefore fails validation for such a candidate.
  - Mutation: remove the check. Expected kill: the bank test plus a verify test.
- **F3.** Real-process SIGKILL under `just test-full` (**needs clearance**, see the question comment).
  - Location: `tests/sandbox/generate/` with a conftest gated on `POPQUIZ_GENERATE_FULL`, and a `test-generate-full` recipe in `test-full`.
  - The test starts `python -m popquiz.generate --backend claude-cli` against a fake `claude` that blocks on c02. It SIGKILLs the generator while c02 is in flight, then reruns with a fast fake.
  - It asserts: no partial record, a complete second run, and the c01 cache reused (c01 is not spawned again).
  - It asserts the room is unaffected: a sha256 digest of every file under `bank/` and `room/` plus `mvp/answer-history.json` is identical before and after.
  - If not cleared, F3 is listed as owed.
- **F4.** Covered in §3.
- **F5.**
  - The brief renders "Each option is a single line…" when `cap.option_lines == 1`, otherwise "Each option is at most N lines".
  - `capacity_breaks` compares the line count with `cap.option_lines` instead of testing `"\n" in text`.
  - Mutation: hard-code the 1 again. Expected kill: a test with `dataclasses.replace(cap, option_lines=2)`.
- **F6.** I ask the client once in the tab, before the push.

## 5. Tests (hermetic)
- `pipeline/tests/fixtures/generate/claude/`:
  - `fake_claude.py`: replays `cli-*.json` by candidate index or a `FAKE_CLAUDE_SCRIPT` env var. It records its argv, its cwd and the names of the env vars it saw (`ANTHROPIC_API_KEY` present or absent), never a value.
  - Fixtures: `cli-clean.json`, `cli-invalid.json`, `cli-usage-limit.json` (is_error, exit 1), `cli-nonzero-exit.json`, `cli-records.json`.
  - The test writes a `claude` sh wrapper into `tmp_path` (`exec <sys.executable> fake_claude.py`) and sets `PATH=<tmp>:/usr/bin:/bin`.
- Guard test: with `PATH=/usr/bin:/bin`, `shutil.which("claude")` is None, and the client refuses.
- Billing test: the parent has `ANTHROPIC_API_KEY=sk-test-not-a-key`; the child's recorded env names exclude it and every other STRIPPED name. Mutation: stop stripping. Expected kill: that test.
- Further tests:
  - The argv contains each flag above, never `--bare`; there is no `shell=True` (AST check).
  - The cap refuses before the build (unset, over), and the refusal sentences are pinned.
  - Usage-limit stop, then rerun completes with `submit` not called and c01 not respawned.
  - Report labels and no summed dollars.
  - No fallback.
  - Importing `generate_cli` starts nothing.
  - The existing brief tests are parametrized to run through the CLI's argv too.
- Mutations are run one at a time as background jobs, each killed at 15 min.

## 6. Contract and docs (rulings 8, 9)
- BUILDPLAN D-D: the model line becomes "Claude Opus 5.5 (`claude-opus-5-5`)". The ticket's amendment paragraph is added verbatim under D-D. Nothing else in D-D changes; its $5/$25 price line stays, and the amendment supersedes it.
- The T-16 row's human-track note becomes "Human track: API key and spend cap (batches) or a logged-in claude CLI (claude-cli)."
- ECONOMICS gets one line under "Who pays": "Subscription runs consume plan usage, not dollars."
- `pipeline/README.md`, Generating: both backends, the billing trap, "the CLI backend has never made a real call".

## 7. Commit order
1. F5
2. F2
3. F4 (client `numbers`, `run()` signature)
4. The CLI client + fake + tests
5. CLI flag/caps/report
6. F1
7. test_runner exemption (if cleared)
8. F3 (if cleared)
9. Contract
10. README

## Contract tensions (sides taken)
- T1. **Process guard.** `test_runner.py` permits one process-starting module. Ruling 7 cannot be met without listing `generate_cli.py`, so I asked. Side taken: list exactly that path, two-directional.
- T2. The exit check says `rg ANTHROPIC_API_KEY pipeline/src` shows only stripping code. The Batches backend's existing `os.environ.get("ANTHROPIC_API_KEY")` in `main` (PR #61) predates this ticket and is that backend's rule. Side taken: leave it, the CLI path never reads it, and name it in DONE.
- T3. `--setting-sources ""` is the documented way to load no settings, but the empty value is unverified against the real binary (no real call). `--safe-mode` backs it up; HC-2 confirms both.
- T5. **Children killed with the run** (plan-review M1). A plain SIGKILL of the generator cannot kill its children. A `claude` child orphaned that way finishes its one candidate on plan usage, and the rerun runs that candidate again. That costs at most one candidate over the cap per kill. It is documented, not hidden.
- T4. A fake-CLI run is labelled `api-equivalent`, because the client cannot know its binary is fake. The report's `claude_bin` path shows the fixture. This is said in the PR.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)
The reviewer (sonnet, fresh eyes) found 0 Critical, 6 Major and 8 Minor findings. All 14 are resolved here.

**Major findings**
- **M1, orphaned children.** Accepted.
  - Fix: each child is spawned with `subprocess.Popen(..., start_new_session=True)` and `communicate(input, timeout)`. On timeout, or on any exception (KeyboardInterrupt included) while a child runs, the parent calls `os.killpg(child.pid, SIGKILL)`, reaps it, and re-raises or stops.
  - Residual risk: a SIGKILL of the generator itself cannot reach the child. That is T5, and it is documented in the README and the PR.
  - The F3 test reaps the fake by its process group.
- **M2, F4 breaks the measured-path tests.** Accepted.
  - Fix: test doubles declare their type. `class MeasuredReplay(ReplayBatchClient): numbers = "measured"` lives in test_generate.py. Every `fixture=False` test uses it, so the cap-required and over-cap tests still go through the measured path.
  - `go()` builds the factory from the client's type.
  - New test: the state identity holds `backend` and `numbers`, and a fixture run id still refuses to resume as measured, and the reverse. This is the old test with the new doubles.
- **M3, the binary guard must be autouse.** Accepted.
  - Fix: an autouse fixture in both test_generate.py and test_generate_cli.py sets `PATH=/usr/bin:/bin`. It asserts at setup that `shutil.which("claude")` is None, and a test fails if it is not.
  - Tests that need the fake prepend only their own `tmp_path` bin directory.
  - The client also refuses a binary that resolves to nothing, in one sentence.
  - F3 passes `--claude-bin <fake>` explicitly.
- **M4, the env denylist is incomplete.** Accepted in part.
  - Ruling 3 says the child env is "a copy … with X removed", so I keep a denylist. I widen it to cover whole families: every variable starting with `ANTHROPIC_` (key, auth token, base URL, custom headers, model, Foundry key …), every `CLAUDE_CODE_USE_*`, and `AWS_BEARER_TOKEN_BEDROCK`.
  - Why not an allowlist: one could break keychain and proxy access in ways nobody can see before HC-2.
  - The settings-`env` re-injection path is covered by `--setting-sources ""` plus `--safe-mode`. It is listed as an **HC-2 check** in the README and the PR: confirm that `--setting-sources ""` is accepted and that the first run shows subscription usage, not API usage.
  - The test sets several family members in the parent and asserts that none reaches the child, while an unrelated variable does.
- **M5, the seam has no "stopped" channel.** Accepted.
  - Fix: the CLI client exposes `stopped: str | None`, set when it stops. `run()` reads `getattr(client, "stopped", None)`. Requests with no entry after a stop are reported as "not run: the run stopped at cNN; rerun with the same --run-id to pick it up", not "no result came back".
  - Per-candidate measurements travel in each entry under a `measured` key (`wall_clock_s`, `api_equivalent_usd`), which `classify` copies onto the `Outcome`.
- **M6, the report code assumes batch rows.** Accepted.
  - Fix: `build_report` and `print_report` branch on `numbers == "api-equivalent"`. CLI rows carry no `prices` block, no `list_prices` call, no `cost_usd` and no dollar total.
  - Each CLI row's `wall_clock_s` is its own `duration_ms`. The total `wall_clock_s` is the sum of the rows'.
  - `print_report` prints "API-equivalent $x, not billed" per row, and the totals carry tokens and seconds only.

**Minor findings**
- **is_error granularity.** Accepted.
  - `is_error` with subtype `error_max_structured_output_retries` is a per-candidate failure: it is reported, not cached, and the run continues.
  - Every other `is_error`, a non-zero exit, a timeout, unparseable stdout and missing `structured_output` on success all stop the run. That is ruling 4's "non-zero exit, or is_error" with the one documented per-candidate exception, and it is listed as a deviation.
- **F1's reach.** Accepted, and stated in the PR. Exact option-text matching catches the shape of claim PQ-68 names. A paraphrase or embedded claim still reaches review as authored content.
  - The row's `quoted_spans` gains value spans, but `bank.quoted_outputs` (what T-18 reads) is untouched, because bank.py is cleared for F2 only. T-18 owns that follow-up, and it is noted.
- **F2's bank claim.** Verified before planning: a Python scan of `bank/questions/*.json` found 4 files and 0 non-last `stdout` rows.
- **`sweep_partials`.** Extended to `_run/cli`. The cache is written with `write_atomic`.
- **Old state files.** They lack `backend` and `numbers`, so they refuse to resume as "different request". That is the safe direction, and accepted.
- **`--model` aliases.** The CLI path never calls `list_prices`, so the model value is passed through as given.
- **Cached reported candidates.** A child that succeeded but whose output fails validation is cached, so it is never retried under the same run id. This matches Batches, where a reported result is not resubmitted. Accepted as designed.
- **`structured_output` only.** Accepted: the client never parses `result` prose. A missing `structured_output` stops the run as unparseable.
