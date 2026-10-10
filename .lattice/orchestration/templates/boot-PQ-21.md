# Delegator boot — The generator, built and proven on fixtures (no paid run)

You are the **delegator for Lattice ticket PQ-21** (BUILDPLAN T-16, M3: the generator via the Message Batches API, AC-1 to AC-5) in the Rust NYC Pop Quiz build. Mode: **inline-full**. You own this ticket from plan to open PR. You report to the Orchestrator through Lattice comments and, for questions, to its panel; the client (the project's owner) watches your tab and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-21** is not pull request #21. Your PR gets its own number.

**FIXTURES ONLY.** The client released this ticket on 2026-10-10 with the words "dispatch it fixtures-only". Human track **H-4 (an Anthropic API key and a spend cap) is still open.** You build the whole generator, real client included, and prove it on recorded fixtures. **You make no call to the Anthropic API, paid or free, and you never read, ask for, print or set an API key.** The first paid run is HC-2 and is the client's. If you find yourself needing a real response to make progress, stop and ask by comment.

**Base.** Your branch `ai-c11-cc/generator-batches` starts from `origin/main` @ `a3c8c4e765c60fd291db1d0961168f3f6106637b` (the merge of PR #60). Rebase onto `origin/main` before your first push and never after it. Your PR opens against `main`. No sibling delegator is live. One unrelated PR is open: #59 (`claude/new-question-set-1v00d3`; touches `bank/README.md` and `bank/candidates/**`). Do not touch either.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-batches" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it.

c11 calls and `git fetch` need the sandbox bypass; if a c11 call is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_PANEL=$(c11 identify --json | /usr/bin/python3 -c 'import json,sys; print(json.load(sys.stdin)["caller"]["panel_ref"])')
test -n "$MY_PANEL" || { echo "FATAL: could not resolve own panel ref"; exit 99; }
c11 rename-panel --panel "$MY_PANEL" "Generator Fixtures"
c11 set-title    --panel "$MY_PANEL" "Generator Fixtures"
c11 set-agent    --panel "$MY_PANEL" --type claude-code --model opus
c11 set-description --panel "$MY_PANEL" "Planning: the T-16 generator (Message Batches, §3.1 shape, run report), fixtures only, no API call. Next gate: plan review.
Lineage: lattice-orchestrator → PQ-21 delegator"
```

Then post your launch receipt (the Orchestrator verifies these fields):

```bash
(cd "$LATTICE_ROOT" && lattice comment PQ-21 "READY PQ-21 CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-batches HEAD $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-batches rev-parse HEAD) BASE $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-batches rev-parse origin/main) MODE active" --actor agent:delegator-pq21)
```

The Orchestrator's panel is `panel:7` in `workspace:3` (refs reset on a c11 restart; if it is gone, comment on the ticket and wait). Questions go there: `c11 send --workspace workspace:3 --panel panel:7 "<text>" && c11 send-key --workspace workspace:3 --panel panel:7 enter`, and you keep working on whatever the question does not block.

## 1. Ground rules (standing clauses — every one is load-bearing)

1. **Every `lattice` command runs as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq21)`.** The board lives in the root checkout only. Never create `.lattice/` in this worktree, never commit anything under `.lattice/`, `HANDOFF.md` or `REHEARSAL-GUIDE.md`. The Orchestrator placed `.claude/settings.json` and this file in this worktree; leave both uncommitted — `.claude/` is not ignored on `main`, so stage by name.
2. **Every `git` command runs from this worktree, never after a `cd` elsewhere.** Before each commit: `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-batches"`.
3. **Status discipline.** Bump the ticket status *before* each phase and verify with `lattice show PQ-21 --json`. The vocabulary on this install: `in_planning → planned → in_progress → review`. There is no `pr_open` and no `in_validation`; **`review` is the terminal pre-merge status.** You stop there. The Orchestrator completes the ticket. PQ-21 starts in `backlog`; your first bump is `in_planning`. Leave its tags alone.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password, and `git commit` must run with the sandbox bypass.** Inside the sandbox it fails with `1Password: Could not connect to socket` — that is the sandbox, not the key. If a bypassed commit still fails to sign (`failed to fill whole buffer`, `agent returned an error`), retry once; then stop: stage the commit, `lattice status PQ-21 needs_human`, raise a c11 flag (`c11 raise-flag --panel "$MY_PANEL" "<one line>"`) and wait for the client to approve 1Password. **Never `--no-gpg-sign`.** Do not retry past two attempts.
6. **Push, PR creation, `uv add` (PyPI) and anything outside the sandbox** prompt the client for approval in this tab. Ask for each once, with the exact command visible; never retry a denied command verbatim.
7. **Deviate-with-flag.** If SPEC, EVALUATION, the criteria or the code contradict each other or your plan, take a side, say which and why in DONE, and never edit the contract files (`SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `ECONOMICS.md`, `PRD.md`, `sequence/**`, `prototypes/**`, `mvp/**`) or the audit report (`docs/audit/**`).
8. **House rules from `CLAUDE.md`:** **never write down what a program prints** (AC-7) — the model's idea of the output is never an answer; the verifier writes `verified`; the segment is one question; answer position is a uniform draw from the date and nothing else, and **the generator never chooses, balances or records answer position** (read `PHILOSOPHY.md` §2 before planning); never overstate verification.
9. **Precedence:** the criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md` on behaviour, state, payloads and copy; `SPEC.md` outranks the prototype.
10. **Shared files are serialized.** This ticket is cleared for exactly the files in §2 "Cleared files" and no others. If you need one outside the list, stop and ask by comment.
11. **Escalate only what a human must decide.** Human-only blockers: `lattice status PQ-21 needs_human` plus `c11 raise-flag`; lower the flag once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand code to a human, never leave a `TODO(human)`, never stop to ask which design to pick: decide, record it under deviations, continue.
- **Your tab may start in manual mode** (`CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1` prints `⚠ Permission mode forced to default`). That is the client's hardening. Write few, whole commands; the client answers each prompt or toggles auto with shift+tab.
- **A cost guard may block your tab** (`codeburn`, $15). Lifting it is the client's (`! codeburn guard allow`); do not work around it. Say so in a comment and wait.
- **The Orchestrator's comments on the ticket are instructions.** Read `(cd "$LATTICE_ROOT" && lattice comments PQ-21)` before planning, at every phase change, **and again immediately before `gh pr create` and before DONE**. In DONE, name every comment you acted on and list each ruling as its own line.
- **The root checkout's source tree is stale** (the orchestration branch, hundreds of commits behind `main`). Read code only from this worktree.
- **Network is blocked inside the sandbox** for `git fetch`, `git push`, `gh` and PyPI; those run with the bypass, one command at a time.
- **Never call `bc`** (aliased to `brew cleanup`). Use `/usr/bin/python3 -c` or `$(( ))`. `timeout` and `gtimeout` are not on this Mac. Use `/usr/bin/python3` (pyenv's is x86_64) where a bare python is needed; the pipeline's own commands go through `just` / `uv run` as the README says.
- **Bounded mutations.** Run every mutation as a background job you kill at 15 minutes. One mutation in an earlier ticket looped for 6.5 hours.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch** — the two writes race on Lattice 0.2.0.
- **The `prefer-rg-fd` hook denies `grep`, `cat`, `head` and `sed -n`.** Use `rg`, `fd -H -I`, and the Read tool.
- **One heavy gate at a time.** Never start two of `just test-full` / `just harness-full` at once.
- **uv policy:** dependencies go in with `uv add` (from `pipeline/`), never pip.

## 2. The ticket

**Why.** `pipeline/src/popquiz/generate.py` is an empty scaffold. Story A1 (AC-1 to AC-5) wants a CLI run that produces candidate questions in the bank's §3.1 shape, offline and re-runnable, with a run report. BUILDPLAN D-D chose the Anthropic Python SDK through the **Message Batches API** with structured outputs (the pipeline is offline, so batch pricing is free money). EVALUATION already proves AC-3, AC-4 (the tagging half) and AC-5 on fixtures with no API, and AC-2 under `test-full`; that is exactly what this dispatch builds. Re-read the ticket with `lattice show PQ-21` and `lattice comments PQ-21`; the DISPATCH comment of 2026-10-10 is the scope.

**Load the `claude-api` skill before writing any SDK code.** The Batches request/result shapes, structured outputs, usage fields and model IDs must come from it, not from memory.

### The Orchestrator's rulings for this dispatch

1. **Fixtures only (the client's word).** No request reaches `api.anthropic.com` from any test, script or manual check in this ticket. `just test` and `just test-full` pass with `ANTHROPIC_API_KEY` unset and the network off. Add a test that proves the default test path cannot construct the real client (for example, the fake is required by the test fixture, or the real client refuses without a key before any network I/O). Never read `.env`.
2. **A seam, like the verifier's `Runner` (D-18).** The generator reaches the API only through one small interface (submit a batch, poll it, fetch its results). The real implementation wraps the Anthropic SDK and is written in full but **never executed** here; say so in its docstring ("unexercised until the first paid run, HC-2"). A replay implementation reads recorded batch results from `pipeline/tests/fixtures/generate/`. The fixtures are hand-built **in the SDK's documented result shape** (from the `claude-api` skill), including at least: a clean batch, a batch with one errored request, one expired request, one candidate that breaks capacity, and one whose structured output fails validation. Name each fixture after what it proves.
3. **The model is one configured value.** D-D names `claude-opus-5`. The current Opus is Opus 5.5 (`claude-opus-5-5`). Put the model ID in one config place, default **`claude-opus-5`** as the contract says, and report the question in DONE under *Contract notes* for the client to rule before HC-2. Do not change BUILDPLAN.
4. **Spend cap: refuse first, spend never.** The cap is a configured value with **no default**. A non-fixture run with no cap configured, or whose worst-case estimate (requests × max output tokens × configured price, at the batch discount) exceeds the cap, refuses **before** submitting, with one plain sentence saying which. Prices are configured values labelled as list-price estimates; AC-5's report is what replaces them.
5. **The brief (D-15, D-23).** The request carries the room's capacity read from the room configuration (SPEC §5.2 / §7.1: source lines at the floor, each option a single line, at most 29 characters), the style brief for `explains`, `why_tempting` and `hint`, and the §11.1 patterns **from their canonical source** (`popquiz.copylint`), never a copied list. A test asserts each of the three capacity facts and every §11.1 rule's pattern appears in the built request, and a mutation that drops one is caught. If the room's capacity has no readable canonical source from Python, stop and ask by comment before inventing one.
6. **Report, never trim (D-15).** A candidate that breaks a capacity fact, fails the §3.1 shape, or comes back errored or expired is reported with the reason and kept out of the emitted set; nothing is silently trimmed, wrapped or joined. The run report names which of count, topics and difficulty it could not honour (AC-3).
7. **The shape.** Emitted candidates are §3.1 minus `verified` and `review`. **Any output the model claims the program prints is not written as an answer** — if §3.1 has no field for a claim, drop it; if your plan wants to keep it for the verifier's comparison, say why and keep it out of every field the verifier or the deck reads. The generator assigns no answer position, no letter, no slot. Use the bank module's existing types and validators (import, do not change `bank.py`; if it needs a change, ask).
8. **Talk mode (AC-4).** `talk {title, abstract}` in, every candidate tagged with a named std/core concept; a fixture title + abstract proves the tags. Whether the connection is real is HC-2's, not yours.
9. **Re-runnable (AC-2).** Writes are atomic (temp file + rename); a run killed mid-way leaves no partial record and needs no cleanup; a re-run completes. The `test-full` case kills a run mid-candidate (replay client with a delay is fine) and re-runs it. Where emitted candidates land: plan it from the bank module's existing candidate path and say where; never into `bank/questions/`, and never touch PR #59's `bank/candidates/q9…q12.json`.
10. **The report (AC-5).** Per candidate: input/output tokens, cost (configured prices × batch discount), wall-clock; plus run totals. In fixture runs the numbers come from the fixtures' `usage` blocks and are labelled as fixture numbers.
11. **Not yours.** AC-1's static check on the room's dependency graph is PQ-47's (held); do not add it, but keep `popquiz` imports out of `room/`. Verification, dedupe and review are other modules: the generator emits candidates and stops. `.env.example`'s stale PRD names are another ticket's; you add **one** line, `ANTHROPIC_API_KEY=`, under a short comment, and change nothing else there. `ECONOMICS.md` is not touched; HC-2 replaces its numbers.

### Cleared files

`pipeline/src/popquiz/generate.py` (and new `pipeline/src/popquiz/generate_*.py` modules if you split it), `pipeline/tests/test_generate*.py`, `pipeline/tests/fixtures/generate/**`, `pipeline/pyproject.toml` and `pipeline/uv.lock` (the `anthropic` dependency only, via `uv add`), `pipeline/README.md` (the `generate` row, the "empty stubs" sentence, and a short *Generating* section: how to run against fixtures, what the paid run will need — H-4's key and cap — and that it has never run), the `justfile` (one `generate *ARGS` recipe, if the README's other commands go that way), and `.env.example` (one line, ruling 11).

### Read, in this order, before planning

`CLAUDE.md`; `PHILOSOPHY.md` §2; `lattice show PQ-21` and `lattice comments PQ-21`; `sequence/USER_STORIES.md` Story A1 (AC-1 to AC-5) and AC-7; `SPEC.md` §3.1, §5.2, §7.1, §11.1; `EVALUATION.md` rows AC-1 to AC-5 and the HC-2 row; `BUILDPLAN.md` D-D, D-15, D-18, D-23 and the T-16 row; then this worktree's `pipeline/README.md`, `pipeline/src/popquiz/{generate,bank,copylint,runner,verify}.py`, and `pipeline/tests/test_runner.py` (the replay pattern to copy).

### Deliverables

- **(1) The generator** per rulings 1–10, in small signed commits with plain-English messages.
- **(2) Tests** under `just test` for AC-3, AC-4 tagging, AC-5, the brief, the cap refusal, report-never-trim and the no-network guard; under `just test-full` for AC-2's kill-and-rerun. Each guard gets a mutation you ran and the test that caught it.
- **(3) Docs** per the cleared README lines.
- **Green:** `just test` hermetic and under 60 s warm (note the number and the test count); `just test-full`; `bank-audit` unchanged on the real bank.

**Exit check before DONE:** `git diff origin/main --stat` shows only cleared files; `bank/` is absent from it; `rg -n "api.anthropic.com|ANTHROPIC_API_KEY" pipeline/tests` shows no live use; every ruling (1–11) appears as its own line in DONE with how you honoured it.

## 3. The arc

**Plan.** `lattice status PQ-21 in_planning`. Write the plan to the path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-21)` — an **absolute** path under `$LATTICE_ROOT/.lattice/plans/`; the file already holds the ticket text, so append below a `# Plan (delegator, date)` heading: the module layout and the seam, the SDK calls you will make (from the `claude-api` skill), the fixtures and what each proves, where candidates land, the config keys (model, cap, prices, capacity source), the tests with their mutations, the order of commits, and any contract tension with the side you take. Then fresh eyes: spawn a review subagent with the **Agent tool** (`subagent_type: general-purpose`, `model: sonnet`) whose prompt holds only the plan path, the worktree path, the cleared files and the eleven rulings; ask for contradictions with the contract and the code, any path by which a test could reach the network or a model claim could become an answer, and any place the cap could be bypassed, as Critical/Major/Minor findings with file references, ≤400 words, stop after ~25 tool calls. Triage every finding into `## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)` in the plan. `lattice status PQ-21 planned`.

**Implement.** `lattice status PQ-21 in_progress`. `git fetch origin && git rebase origin/main` first. Small signed commits, one concern each, with plain-English messages; end each with a `Co-Authored-By:` trailer naming the model you are actually running as.

**Code review.** Spawn a second review subagent (Agent tool, `model: sonnet`, same bounds) with the branch, the worktree path, `git diff origin/main...HEAD --stat` (it reads the diffs itself), the plan path and the rulings; ask it to check each ruling, that no test can reach the network, that the cap refuses before submission, that no model-claimed output reaches an answer field, that nothing is trimmed silently, that every mutation kills a test, and that the diff stays inside the cleared files; Verdict PASS / PASS-WITH-NITS / FAIL with Critical/Major/Minor/NIT findings and file:line. Fix Critical and Major; re-review once if the fixes were more than mechanical. **At most two fix→re-review cycles**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-21 --type note --role review --title "Code review" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq21-reviewer`.

**Validate.** A fixture run of the CLI end to end (the command, its report, where the candidates landed), the suite counts and the refusal sentences for no-cap and over-cap; attach with `--role review --title "Validation"` (this install accepts only `--role review`).

**PR.** Re-read `lattice comments PQ-21`. Confirm `git log origin/main..HEAD` holds only your commits and `git rev-parse --show-toplevel` is this worktree. Push: `git push -u origin ai-c11-cc/generator-batches` (the client approves). Verify: `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/generator-batches)"`; re-push until equal. Open the PR **against `main`**: `gh pr create --base main --head ai-c11-cc/generator-batches` — title in plain words without the ticket ID; body: what it builds, **that it has never called the API (fixtures only; H-4 and HC-2 remain)**, the seam, the fixtures and what each proves, the cap rule, the tests and their mutations, the suite counts and the `just test` warm time, *Contract notes* (the model-ID question; anything else for upstream), and the line `Lattice PQ-21 · T-16 · AC-1–5`; end with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Confirm the PR's head sha differs from its base. Then **serially**: `lattice attach PQ-21 <PR URL> --type reference --title "PR" --actor agent:delegator-pq21`, wait, `lattice status PQ-21 review`, verify with `lattice show PQ-21 --json`.

**Completion comment**, then stop:

```
lattice comment PQ-21 "DONE PQ-21 PR <url> HEAD <sha> BASE <origin/main sha> test <N passed> test-warm <s> test-full <result> fixture-run: <command → n emitted, k reported> api-calls: none mutations: <each → the test that caught it, or survived> rulings: <one line each, 1–11> comments-acted-on: <list> contract-notes: <model ID; anything else for upstream> deviations: <none | numbered>"
```

Every sha and path in READY or DONE comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/generator-batches …` or the literal worktree path — never from `$(pwd)` or a bare `git` inside the `(cd "$LATTICE_ROOT" && …)` subshell. Update your description to say the PR is open and you are waiting. **Do not merge**, even if asked in this tab, without first re-reading `lattice comments PQ-21` for the Orchestrator's hold or clearance.

Refresh `c11 set-description` at each phase change (planning → implementing → in review → PR open), keeping the `Lineage:` line last.
