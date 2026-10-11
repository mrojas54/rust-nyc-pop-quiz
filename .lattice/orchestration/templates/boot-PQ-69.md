# Delegator boot — The generator's second backend (`claude -p` on the subscription), plus the PR #61 follow-ups

You are the **delegator for Lattice ticket PQ-69**, with **PQ-68 folded into the same run** (BUILDPLAN T-16, M3: the generator, AC-1 to AC-5) in the Rust NYC Pop Quiz build. Mode: **inline-full**. You own both tickets from plan to one open PR. You report to the Orchestrator through Lattice comments and, for questions, to its panel; the client (the project's owner) watches your tab and answers approval prompts in it.

**Mind the numbers.** Tickets **PQ-69** and **PQ-68** are not pull requests #69 and #68. Your PR gets its own number. PR #61 (merged, `170cfd4`) is the generator you are extending.

**NO REAL MODEL CALLS.** The client ruled on 2026-10-11 that the generator gets a second backend, headless Claude Code (`claude -p`) on the client's subscription, beside Message Batches. You build it and prove it with a **fake `claude` executable** on fixtures. **You never run `claude -p` against the real service, never call the Anthropic API, and never read, ask for, print or set an API key.** Running `claude --help` (and `claude --version`) to check flags is allowed and expected. The first real run is the client's (HC-2). If you need a real response to make progress, stop and ask by comment.

**Base.** Your branch `ai-c11-cc/generator-claude-cli` starts from `origin/main` @ `170cfd4ee461e1084e50956b29f05310b51b11dc` (the merge of PR #61). Rebase onto `origin/main` before your first push and never after it. Your PR opens against `main`. No sibling delegator is live and no other PR is open.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-claude-cli" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it.

c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "Generator CLI"
c11 set-title    --panel "$MY_PANEL" "Generator CLI"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Planning: claude -p subscription backend beside Batches (PQ-69) + PR #61 follow-ups (PQ-68). No real calls. Next gate: plan review.
Lineage: lattice-orchestrator → PQ-69 delegator"
```

Then post your launch receipt (the Orchestrator verifies these fields):

```bash
(cd "$LATTICE_ROOT" && lattice comment PQ-69 "READY PQ-69 (+PQ-68) CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-claude-cli HEAD $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-claude-cli rev-parse HEAD) BASE $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-claude-cli rev-parse origin/main) MODE active" --actor agent:delegator-pq69)
```

The Orchestrator's panel is `panel:7` in `workspace:3` (refs reset on a c11 restart; if it is gone, comment on the ticket and wait). Questions go there: `c11 send --workspace workspace:3 --panel panel:7 "<text>" && c11 send-key --workspace workspace:3 --panel panel:7 enter`, and you keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq69)`.** The board lives in the root checkout only. Never create `.lattice/` in this worktree, never commit anything under `.lattice/`, `HANDOFF.md` or `REHEARSAL-GUIDE.md`. The Orchestrator placed `.claude/settings.json` and this file in this worktree; leave both uncommitted — `.claude/` is not ignored on `main`, so stage by name.
2. **Every `git` command runs from this worktree, never after a `cd` elsewhere.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-claude-cli"`.
3. **Status discipline.** Bump **PQ-69's** status *before* each phase and verify with `lattice show PQ-69 --json`. The vocabulary on this install: `in_planning → planned → in_progress → review`. There is no `pr_open`; **`review` is the terminal pre-merge status.** You stop there. PQ-69 and PQ-68 both start in `planned`, and this install allows no move back to `in_planning` from there (valid next: `in_progress`, `review`, `blocked`, `needs_human`, `cancelled`). So **planning happens without a status bump**; your first bump is `in_progress` at Implement. **PQ-68 moves with PQ-69:** bump it to the same status right after PQ-69 each time. The Orchestrator completes both. Leave both tickets' tags alone.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password, and `git commit` must run with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox, not the key. If a bypassed commit still fails to sign (`failed to fill whole buffer`, `agent returned an error`), retry once; then stop: stage the commit, `lattice status PQ-69 needs_human`, raise a c11 flag (`c11 raise-flag --panel "$MY_PANEL" "<one line>"`) and wait for the client to approve 1Password. **Never `--no-gpg-sign`.** Do not retry past two attempts.
6. **Push, PR creation, `uv add` (PyPI) and anything outside the sandbox** prompt the client for approval in this tab. Ask for each once, with the exact command visible; never retry a denied command verbatim.
7. **Deviate-with-flag.** If SPEC, EVALUATION, the criteria or the code contradict each other or your plan, take a side, say which and why in DONE. **The contract files are read-only except for the exact edits PQ-69 ruling 8 names** (BUILDPLAN D-D and the T-16 row; one ECONOMICS line), made with the ticket's wording verbatim. Never edit `SPEC.md`, `EVALUATION.md`, `DESIGN.md`, `PHILOSOPHY.md`, `PRD.md`, `sequence/**`, `prototypes/**`, `mvp/**` or `docs/audit/**`, and nothing else in BUILDPLAN or ECONOMICS.
8. **House rules from `CLAUDE.md`:** **never write down what a program prints** (AC-7) — the model's idea of the output is never an answer; the verifier writes `verified` and the trace's stdout value; the segment is one question; answer position is a uniform draw from the date and nothing else, and **the generator never chooses, balances or records answer position**; never overstate verification.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md` on behaviour, state, payloads and copy; `SPEC.md` outranks the prototype.
10. **Shared files are serialized.** This run is cleared for exactly the files in §2 "Cleared files" and no others. If you need one outside the list, stop and ask by comment.
11. **Escalate only what a human must decide.** Human-only blockers: `lattice status PQ-69 needs_human` plus `c11 raise-flag`; lower the flag once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`, never stop to ask which design to pick: decide, record it under deviations, continue.
- **Your tab may start in manual mode** (`CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1` prints `⚠ Permission mode forced to default`). That is the client's hardening. Write few, whole commands; the client answers each prompt or toggles auto with shift+tab.
- **A cost guard may block your tab** (`codeburn`, $15). Lifting it is the client's (`! codeburn guard allow`); do not work around it. Say so in a comment and wait.
- **The Orchestrator's comments on both tickets are instructions.** Read `lattice comments PQ-69` and `lattice comments PQ-68` before planning, at every phase change, **and again immediately before `gh pr create` and before DONE**. In DONE, name every comment you acted on and list each ruling as its own line.
- **The root checkout's source tree is stale** (the orchestration branch). Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch`, `git push`, `gh` and PyPI; those run with the bypass, one command at a time.
- **The sandbox denies reading or writing `.env` and `.env.*` in this worktree.** Do not try to edit `.env.example`; see ruling F6.
- **Never call `bc`** (aliased to `brew cleanup`). Use `/usr/bin/python3 -c` or `$(( ))`. `timeout` and `gtimeout` are not on this Mac. Use `/usr/bin/python3` where a bare python is needed; the pipeline's own commands go through `just` / `uv run` as the README says.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes. One mutation in an earlier ticket looped for 6.5 hours.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — the two writes race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head` and `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time.** Never start two of `just test-full` / `just harness-full` at once.
- **uv policy:** dependencies go in with `uv add` (from `pipeline/`), never pip. This run should need no new dependency; if it does, say why.

## 2. The tickets

**Why.** PR #61 built the generator with one model backend, the Message Batches API, behind a submit/poll/results seam with a replay client. H-4 (an API key and spend cap) is still open, so the generator cannot run for real. The client ruled that headless `claude -p` on their subscription becomes a **second** backend behind the same seam. PQ-68 holds the review leftovers from PR #61, in the same files, so it rides along. Re-read both with `lattice show PQ-69`, `lattice show PQ-68` and their comments; PQ-69's description holds its nine rulings and the D-D amendment text, PQ-68's its six items.

**Load the `claude-api` skill only if you touch the Batches client.** For `claude -p`, take the flags from `claude --help` on this machine and the output JSON shape from the Claude Code docs (ask a `claude-code-guide` subagent for the `--output-format json` result shape and where `--json-schema` output lands). Record both sources in the plan. **The fixtures' `claude -p` output is documented, not observed, until HC-2; say so in the code and the PR.**

### PQ-69 — the Orchestrator's rulings (the ticket's nine, restated for the plan)

1. **One more client behind PR #61's seam.** Backend is an explicit config value or CLI flag, `batches` | `claude-cli`. **Batches stays the default. Neither backend ever falls back to the other**, on any error.
2. **One `claude -p` process per candidate**, prompt on stdin, `--model <the one model config value>`, `--output-format json`, `--json-schema <the candidate schema>`, no tools, `--no-session-persistence`, cwd a fresh empty temp dir. No hooks, CLAUDE.md, MCP servers, plugins or user settings may load: pick the flags from `claude --help` (`--tools`, `--setting-sources`, `--strict-mcp-config`, …), and record the help text you relied on in the plan and PR. **Never `--bare`**: its help says auth is strictly `ANTHROPIC_API_KEY`, so it cannot reach the subscription. Invocation goes through `subprocess` with an argument list, never a shell string.
3. **Billing guard.** The child's env is a copy of the parent's with `ANTHROPIC_API_KEY` and `ANTHROPIC_AUTH_TOKEN` removed (consider the provider-switch vars such as `CLAUDE_CODE_USE_BEDROCK` / `CLAUDE_CODE_USE_VERTEX` too, and say what you chose). If the key reaches the child, `claude -p` bills the API instead of the plan. A test runs the fake `claude` with the key set in the parent and asserts the child never sees it; a mutation that stops stripping is caught. The code never reads, logs or prints the key's value.
4. **Usage cap.** For `claude-cli`, a required max-candidates-per-run with **no default**; refuse before the first call, in one plain sentence, if it is unset. The dollar spend cap stays the Batches backend's rule, unchanged. A usage-limit or rate-limit result (non-zero exit, or `is_error` in the JSON) stops the run cleanly: completed candidates are already written atomically, the report says why it stopped, and a rerun picks up the rest (AC-2). Never retry in a loop.
5. **AC-5 report.** Tokens and wall-clock as measured from the JSON. Cost is the JSON's `total_cost_usd`, labelled **API-equivalent, not billed**; never presented as spend, never summed into a "spent" total.
6. **Unchanged from PQ-21.** Model-claimed output never lands in an answer field; verify writes the stdout row; capacity facts and the §11.1 patterns (from `popquiz.copylint`) are in the brief; report, never trim; talk mode; atomic writes. The CLI backend uses **the same brief builder** as Batches — one source, and the existing brief tests cover both backends.
7. **Hermetic tests.** A fake `claude` executable (a small script under `pipeline/tests/fixtures/generate/`, put first on `PATH` by the test) replays fixtures named after what they prove: at least a clean candidate, a structured output that fails validation, an `is_error` usage-limit result, a non-zero exit, and one that records the argv and env it received. `just test` and CI never reach the real `claude`; add a guard test that the real binary cannot be resolved on the default test path (or that the client refuses without an explicit binary path in tests).
8. **Contract, in this PR, verbatim from PQ-69's description:** the D-D amendment paragraph under D-D (and D-D's model line), the T-16 row's human-track note, one ECONOMICS line. No other contract edits.
9. **Operator note** in `pipeline/README.md`: how to run each backend, the `ANTHROPIC_API_KEY` billing trap, and that the CLI backend has never made a real call.

### PQ-68 — folded items (from the PR #61 exact-head review)

- **F1 (Minor).** The stdout filter matches by name only (`generate.py` ~651). Also run any trace value's `now` strings through the AC-73 quoted-output comparison, so a model-claimed output under another name never reaches a trace. Mutation + test.
- **F2 (Minor).** `verify.py` ~529-547 leaves `stdout` rows on non-last trace steps untouched. Pick one: strip them, refuse them in verify, or have `validate_question` reject them. Say which and why. Mutation + test.
- **F3 (Minor).** AC-2's kill-and-rerun is an in-process KeyboardInterrupt. Add a `just test-full` case that runs the generator as a real subprocess (fixture/fake client), sends it SIGKILL mid-candidate, reruns it, and asserts no partial record and a complete second run. Also assert the room is unaffected: nothing under `bank/questions/` or the room's state changes (name exactly what you compare).
- **F4 (NIT).** `run()` takes `fixture` and `client_factory` separately (`generate.py` ~964-1020); derive "fixture-ness" from the client type. With a third client (`claude-cli`), keep the labelling honest: fixture numbers, API-equivalent numbers and real batch numbers are each labelled as what they are.
- **F5 (NIT).** The single-line option rule is hard-coded (`generate.py` ~324); read `cap.option_lines` (or whatever the capacity object calls it).
- **F6 (owed line).** `.env.example` needs one `ANTHROPIC_API_KEY=` line under a short comment (Batches backend only). The sandbox blocks you. **Ask the client once in your tab** to add it by hand on this branch before your push; if they do, commit it by name; if not, list it as owed in DONE.

Line numbers are from `f4fb271`; re-find them, don't trust them.

### Cleared files

`pipeline/src/popquiz/generate.py` and `pipeline/src/popquiz/generate_*.py` (new modules allowed, e.g. a `generate_cli.py` client), `pipeline/src/popquiz/verify.py` and `pipeline/src/popquiz/bank.py` (F2 only), `pipeline/tests/test_generate*.py`, `pipeline/tests/test_verify.py`, `pipeline/tests/test_bank.py`, `pipeline/tests/fixtures/generate/**`, `pipeline/pyproject.toml` / `pipeline/uv.lock` (only if a dependency is truly needed), `pipeline/README.md` (the generating section), the `justfile` (generate recipe flags only, if needed), `BUILDPLAN.md` and `ECONOMICS.md` (ruling 8's exact edits only), `.env.example` (F6, the client's hand).

### Read, in this order, before planning

`CLAUDE.md`; `PHILOSOPHY.md` §2; `lattice show PQ-69`, `lattice comments PQ-69`, `lattice show PQ-68`, `lattice comments PQ-68`, and PQ-21's review artifact (`lattice show PQ-21` lists it; the exact-head review is `art_01M4NREDB32Y2P8NYJCB5RKMN7`); `sequence/USER_STORIES.md` Story A1 (AC-1 to AC-5), AC-7, AC-73; `SPEC.md` §3.1, §5.2, §7.1, §11.1; `EVALUATION.md` rows AC-1 to AC-5 and HC-2; `BUILDPLAN.md` D-D, D-15, D-18, D-23 and the T-16 row; `ECONOMICS.md`; then this worktree's `pipeline/README.md`, `pipeline/src/popquiz/{generate*,bank,copylint,verify}.py`, and `pipeline/tests/test_generate*.py`; then `claude --help`.

### Deliverables

- **(1) The `claude-cli` backend** per rulings 1–7 and **(2) F1–F5**, in small signed commits with plain-English messages, one concern each.
- **(3) Tests** under `just test` for each ruling and F1, F2, F4, F5; under `just test-full` for F3. Each guard gets a mutation you ran and the test that caught it.
- **(4) Contract and docs** per rulings 8 and 9.
- **Green:** `just test` hermetic and under 60 s warm (note the number and the test count); `just test-full`; `bank-audit` unchanged on the real bank.

**Exit check before DONE:** `git diff origin/main --stat` shows only cleared files; `bank/` is absent from it; `rg -n "ANTHROPIC_API_KEY" pipeline/src` shows only the stripping code and no read of its value; no test resolves the real `claude`; every ruling (1–9) and F1–F6 appears as its own line in DONE with how you honoured it.

## 3. The arc

**Plan.** (No status bump; both tickets stay `planned`.) Write the plan to the path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-69)` — an **absolute** path under `$LATTICE_ROOT/.lattice/plans/`; append below a `# Plan (delegator, date)` heading: the client module and how it fits the seam, the exact argv and env handling (with the `claude --help` lines relied on), the JSON shape and its source, the fixtures and the fake `claude`, the usage-cap and stop behaviour, F1–F6 each with its approach, the tests with their mutations, the commit order, and any contract tension with the side you take. Then fresh eyes: spawn a review subagent with the **Agent tool** (`subagent_type: general-purpose`, `model: sonnet`) whose prompt holds only the plan path, the worktree path, the cleared files, the nine rulings and F1–F6; ask for contradictions with the contract and the code, any path by which a test could reach the real `claude` or the network, any path by which the API key could reach the child or a model claim could become an answer, and any place either cap could be bypassed, as Critical/Major/Minor findings with file references, ≤400 words, stop after ~25 tool calls. Triage every finding into `## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)` in the plan, then post a one-line `PLAN PQ-69 <plan path> review: <n> findings resolved` comment.

**Implement.** `lattice status PQ-69 in_progress` (PQ-68 the same). `git fetch origin && git rebase origin/main` first. Small signed commits; end each with a `Co-Authored-By:` trailer naming the model you are actually running as.

**Code review.** Spawn a second review subagent (Agent tool, `model: sonnet`, same bounds) with the branch, the worktree path, `git diff origin/main...HEAD --stat` (it reads the diffs itself), the plan path, the rulings and F1–F6; ask it to check each, that no test can reach the real `claude` or the network, that the key never reaches the child, that both caps refuse before the first call, that no model-claimed output reaches an answer field, that neither backend falls back to the other, that nothing is trimmed silently, that every mutation kills a test, that the contract edits match the ticket verbatim, and that the diff stays inside the cleared files; Verdict PASS / PASS-WITH-NITS / FAIL with Critical/Major/Minor/NIT findings and file:line. Fix Critical and Major; re-review once if the fixes were more than mechanical. **At most two fix→re-review cycles**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-69 --type note --role review --title "Code review" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq69-reviewer`.

**Validate.** A fixture run of the CLI end to end on each backend (the command, its report, where the candidates landed), the suite counts, and the refusal sentences for: no cap (each backend), over cap (Batches), and a usage-limit stop (claude-cli). Attach with `--role review --title "Validation"`.

**PR.** Re-read `lattice comments PQ-69` and `PQ-68`. Confirm `git log origin/main..HEAD` holds only your commits and `git rev-parse --show-toplevel` is this worktree. Push: `git push -u origin ai-c11-cc/generator-claude-cli` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/generator-claude-cli)"`; re-push until equal. Open the PR **against `main`**: `gh pr create --base main --head ai-c11-cc/generator-claude-cli` — title in plain words without ticket IDs; body: what it builds, **that it has made no real model call (fake `claude` and fixtures; the first real run is the client's, HC-2)**, how the subscription path avoids API billing, both caps, the fixtures and what each proves, F1–F6, the contract edits, the tests and their mutations, the suite counts and the `just test` warm time, *Contract notes*, and the line `Lattice PQ-69 + PQ-68 · T-16 · AC-1–5`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Use the REST API for a long body and verify it with `gh pr view <N> --json body`. Confirm the PR's head sha differs from its base. Then **serially**: `lattice attach PQ-69 <PR URL> --type reference --title "PR" --actor agent:delegator-pq69`, wait, `lattice status PQ-69 review`, then the same attach and status for PQ-68; verify both with `lattice show … --json`.

**Completion comment** on PQ-69 (and a one-line pointer on PQ-68: `DONE folded into PQ-69, PR <url>`), then stop:

```
lattice comment PQ-69 "DONE PQ-69 (+PQ-68) PR <url> HEAD <sha> BASE <origin/main sha> test <N passed> test-warm <s> test-full <result> fixture-runs: <batches → …; claude-cli → …> real-calls: none mutations: <each → the test that caught it, or survived> rulings: <one line each, 1–9> folded: <F1–F6, one line each> comments-acted-on: <list> contract-notes: <…> deviations: <none | numbered>"
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-claude-cli …` or the literal worktree path — never from `$(pwd)` or a bare `git` inside the `(cd "$LATTICE_ROOT" && …)` subshell. Update your description to say the PR is open and you are waiting. **Do not merge**, even if asked in this tab, without first re-reading `lattice comments PQ-69` for the Orchestrator's hold or clearance. Then write a short checkpoint to `.claude/checkpoint-pq69.md` (uncommitted) and stop.

Refresh `c11 set-description` at each phase change (planning → implementing → in review → PR open), keeping the `Lineage:` line last.
