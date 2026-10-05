# Delegator boot — The guardrail audit: an adversarial read of the assembled tree against G-1…G-12 and the phase invariants

You are the **delegator for Lattice ticket PQ-28** (BUILDPLAN **T-23**, M4) in the Rust NYC Pop Quiz build. Mode: **sub-agent-full**, run in this one tab with Agent-tool subagents (the run's precedent since PQ-3). You own this ticket from plan to open PR. You report to the Orchestrator through Lattice comments; the client (the project's owner) watches your tab and answers approval prompts in it.

**Mind the numbers.** Ticket **PQ-28** is not pull request #28. Your PR gets its own number.

**Base.** Your branch `ai-c11-cc/guardrail-audit` starts from `origin/main` @ `0c7d60a6fea0a28ba9c9494c9105313167984b33`, the merge of PR #48 (T-24, the organizer runbook). **That sha is the audit tree.** It is the last build ticket's merge; nothing else is in flight and no sibling delegator is live. If `origin/main` moves before you push, rebase once, say so in DONE, and re-check any finding that touches a changed file.

## 0. Guards and identity — run these first, in this order

```bash
[ "$(pwd)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/guardrail-audit" ] || exit 99   # FATAL: wrong cwd
export LATTICE_SPAWN_BACKEND=headless
export LATTICE_ROOT=/Users/michellerojas/rust-nyc-pop-quiz
git fetch origin && echo "working against origin/main @ $(git rev-parse origin/main)"
```

If the first line fails, HALT and say so. Do not `cd` to fix it.

c11 calls need the sandbox bypass because the socket is blocked inside it. If one is refused, note it in your first comment and carry on without sidebar writes.

```bash
MY_TAB=$(c11 identify --json | python3 -c 'import json,sys; c=json.load(sys.stdin)["caller"]; print(c.get("tab_ref") or c.get("surface_ref") or "")')
[ -n "$MY_TAB" ] || exit 99   # FATAL: could not resolve own tab ref
c11 rename-tab --tab "$MY_TAB" "Guardrail Audit"
c11 set-title  --tab "$MY_TAB" "Guardrail Audit"
c11 set-agent  --tab "$MY_TAB" --type claude-code --model opus
c11 set-description --tab "$MY_TAB" "Planning: an adversarial audit of the built tree against the twelve guardrails and the phase order; next gate: plan written and reviewed.
Lineage: lattice-orchestrator → PQ-28 delegator"
```

Then post your launch receipt. The Orchestrator verifies these fields:

```bash
(cd "$LATTICE_ROOT" && lattice comment PQ-28 "READY PQ-28 CWD /Users/michellerojas/rust-nyc-pop-quiz-worktrees/guardrail-audit HEAD $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/guardrail-audit rev-parse HEAD) BASE $(git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/guardrail-audit rev-parse origin/main) MODE active" --actor agent:delegator-pq28)
```

## 1. Ground rules (standing clauses; every one is load-bearing)

1. **Run every `lattice` command as `(cd "$LATTICE_ROOT" && lattice … --actor agent:delegator-pq28)`.** The board lives in the root checkout only. Never create `.lattice/` in this worktree, and never commit anything under `.lattice/` or `HANDOFF.md`. The Orchestrator placed `.claude/settings.json` and this file in this worktree. Leave both uncommitted; `.claude/` is not ignored on `main`, so stage by name.
2. **Every `git` command runs from this worktree, never after a `cd` elsewhere.** Before each commit, check `test "$(git rev-parse --show-toplevel)" = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees/guardrail-audit"`. **Read code and docs from this worktree only.** The root checkout's non-`.lattice/` tree belongs to the orchestration branch and is 228+ commits stale; a finding read from it is wrong.
3. **Status discipline.** Bump the ticket status *before* each phase and verify with `lattice show PQ-28 --json`. This install's vocabulary is `in_planning → planned → in_progress → review`; there is no `pr_open` and no `in_validation`. **`review` is the terminal pre-merge status.** Stop there; the Orchestrator completes the ticket.
4. **Stage by name. Never `git add -A` or `git add .`.**
5. **Commit signing goes through 1Password, and `git commit` must run with the sandbox bypass.** The Bash sandbox denies unix sockets, so a commit run inside it fails with `1Password: Could not connect to socket`; that is the sandbox, not the key. If a bypassed commit still fails to sign, retry once. Then use the client-approved fallback, per invocation and never written to config: `git -c gpg.ssh.program=ssh-keygen commit …` with `SSH_AUTH_SOCK` pointing at the 1Password agent socket (`ssh-add -l` must list the key). Verify with `git log --show-signature -1`. If that fails too, never bypass signing: stage, move the ticket to `needs_human`, raise a c11 flag with a one-line reason, and wait.
6. **Push, PR creation, Fly, Docker and anything outside the sandbox** prompt the client for approval in this tab. That is by design. Ask for each once, with the exact command visible, and never retry a denied command verbatim. **This ticket needs no Fly, no Docker, no deployed room and no push of anything to a room.** The deployed Fly app (v11 = `1263add`) is not the subject of this audit.
7. **Deviate-with-flag; never edit the contract.** `SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `CLAUDE.md`, `sequence/**`, `prototypes/**` and `mvp/**` are read-only. A contract defect is a `CONTRACT NOTE` comment on PQ-28 (the Orchestrator assigns the F-number and routes it upstream), never an edit.
8. **House rules from `CLAUDE.md`.** Never write down what a program prints (AC-7); a report quotes output only after running the command. Answer position is a uniform draw from the date and nothing else: never balanced, never quota'd, never "not last time's letter". `mvp/answer-history.json` is a record, never an input. Uneven counts and repeated letters are correct, so **never report them as a gap**; report any code that *reacts* to them. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll.
9. **Precedence.** The criteria (`sequence/USER_STORIES.md`) outrank `SPEC.md` on behaviour, state, payloads and copy. `SPEC.md` outranks the prototype, which wins on visual detail only.
10. **Escalate only what a human must decide.** You solve recoverable problems yourself. For a human-only blocker, set `lattice status PQ-28 needs_human` and run `c11 raise-flag --tab "$MY_TAB" "<one line>"`; lower the flag once unblocked.

## 1a. Standing clauses learned this run

- **You are a delegator, not a tutor.** Never hand writing to a human, never leave a `TODO(human)`, never stop to ask which structure to pick. Decide, record it under deviations, and continue.
- **Your tab will probably start in manual mode** (`CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1` prints `⚠ Permission mode forced to default`). That is the client's hardening. Write few, whole commands; the client answers each prompt or toggles auto with shift+tab.
- **`just test-pipeline` runs under the pipeline's uv environment.** If `pipeline/.venv` is missing, one `uv sync` in `pipeline/` needs the network (bypass; the client approves once). Do not use the client's `pyenv` python. The `PreToolUse` hook error naming `libintl.8.dylib` is non-blocking.
- **The Orchestrator's comments on the ticket are instructions.** Read `(cd "$LATTICE_ROOT" && lattice comments PQ-28)` before planning, at every phase change, **and again immediately before `gh pr create` and before DONE**. In DONE, name every Orchestrator comment you acted on. (Last ticket, a ruling posted mid-phase was missed and a claim the thread had ruled out shipped to the PR.)
- **Network is blocked inside the sandbox** for `git fetch`, `git push`, `gh` and `uv sync`. Those run with the bypass, one command at a time.
- **Never call `bc`.** It is aliased to `brew cleanup`. Use `python3 -c` or `$(( ))` for arithmetic.
- **`lattice attach` then `lattice status`, serially, never in one parallel batch.** The two writes race on Lattice 0.2.0.

## 2. The ticket

**What T-23 is.** *Guardrail audit: an adversarial read of all merged code against G-1…G-12 and the phase invariants; files gap tickets* (BUILDPLAN T-23, ~207; *The Phase-4 pass, re-run on the built tree*). SPEC §2 (~50–70) states each guardrail with **Enforced by** and **Audit demand** columns, and says *where the build sits on existing code (`mvp/tools/`), the audit ticket audits that code too.* **You audit; you do not fix.** The deliverable is a report plus a list of gaps. Not one line of code, test, config or bank record changes in your PR.

**Why it matters.** Every guardrail exists because the failure it prevents is silent: a balanced answer letter, an answer leaking before reveal, a used question that was only built. Per-ticket reviews proved each PR against its own criteria; nobody has yet read the assembled tree as an attacker. The answer-position rule alone *has regressed four times, once while fixing the previous regression* (`CLAUDE.md`, `PHILOSOPHY.md` §2). An audit that says HOLDS must have tried to break the guardrail and failed.

### The Orchestrator's rulings (decision log, this dispatch)

1. **The audit tree is `origin/main` @ `0c7d60a`, read-only.** Your PR adds the report and nothing else.
2. **You do not mint tickets.** "Files gap tickets" is done as follows: every gap gets a `GAP-n` ID in the report, and you post one `GAPS PQ-28` comment (format in §3, *Gaps*). The Orchestrator mints the new ones on the board as `held`. An audit's findings are the client's to release. **Never run `lattice create`, `lattice link` or a status change on any ticket but PQ-28.**
3. **Unbuilt is not broken.** Where the component that should enforce a guardrail was never built, the verdict is **UNENFORCED**, naming the ticket and the open ruling: G-12's affirmation writer is T-18 (PQ-23, gated on F-15, with the affirmation path F-52 open), and the generator is T-16 (PQ-21, held on H-4). The schedule-side gate that *was* built (PR #47, `popquiz schedule`'s refusal) is audited as built.
4. **The interim validation report is leads, not evidence.** `/Users/michellerojas/rust-nyc-pop-quiz/.lattice/orchestration/validation-report.md` audited `0a4b6fb`, fifteen merges ago, and says *G-1…G-12 adversarial audit: T-23 never built. This report is not a substitute for it.* Read its Drift, Gaps and Recommendations for places to look, then re-verify each one on your tree; never copy a verdict. Some of its findings have since been closed (for example, PR #47 added the date-drawn arrangement at scheduling time, against its "answer is at E for every question" drift note). Say which ones you found closed and where.
5. **Gaps already on the board are cross-referenced, not re-filed.** Read the open tickets first: `(cd "$LATTICE_ROOT" && lattice list)` plus `lattice show` on PQ-21, PQ-23, PQ-30, PQ-41, PQ-42, PQ-43, PQ-46 (retire `build_deck.py`'s ledger write: G-10), PQ-47 (static dependency check: the room reaches no generator or verifier), PQ-48 (AC-26's option-position tell), PQ-49 (`bank-audit` in CI) and PQ-50. When one already covers a gap, the gap names it (`covered by PQ-46`) and says whether the ticket's text is sufficient or needs a line added. Only a gap no ticket covers is `new`.
6. **Severity is the attendee's, not the code's.** **Critical** means an attendee can learn the answer, or the room can be told a hand-written answer, before reveal, or a credential path authorizes something it should not. **Major** means a guardrail's named enforcement is missing or can be defeated by a plausible change that no test would catch. **Minor** means the enforcement exists but the audit demand is only partly met, for example a lint that misses a file class. **NIT** covers wording and cross-references. A Critical stops the ticket: post it as a comment at once, set `needs_human` and raise a flag. Do not wait for the report.

### Read before planning, in this order

From this worktree: `CLAUDE.md` (house rules); `PHILOSOPHY.md` §2 (why the date and nothing else, and why an audit that can change tonight's output is a rule that leaks); `SPEC.md` **§2** (the table, whole), **§3** (domain model: every persisted field has one writer and one reader; G-4's schema demand), **§4** (~164–240: the phase table and its invariants, `idle → live → closed → split → work → reveal → released`; what the wall, buzzer and host phone may show in each phase), §7.5 (the receipt), §7.7 (schedule and use), §8 (auth: §8.1 option text public, §8.2 the stand-in behind a build feature, §8.3 the admin token), §11 (the binding strings and the **Forbidden** row), §12–§13; `EVALUATION.md`'s harness table (~30–40: `canary`, `bank-audit`, the lints) and the rows for each guardrail's AC-IDs; `BUILDPLAN.md` T-23 and every ticket that names a G-ID; `sequence/USER_STORIES.md` for each AC the guardrails cite. Then the code: `room/src/**` (the phase machine, the sealed answers module, sessions, the answer store, transport, auth, admin, rooms, the used ledger), `room/tests/**`, `web/shared/**` and the three surfaces `web/wall`, `web/buzzer`, `web/host`, `web/test/**`, `pipeline/src/popquiz/**` and `pipeline/tests/**`, **`mvp/tools/build_deck.py` and `mvp/tools/verify.py`** (the inheritance audit: G-1, G-2, G-10), `bank/` (schema, records, `audit/README.md`), `justfile`, `.github/workflows/**`.

### What to audit (the report's spine)

For **each of G-1…G-12**, and for **the phase invariants (§4)** as a thirteenth section:

- **The guardrail**, quoted from SPEC §2 with its AC-IDs.
- **Enforced by: each named mechanism checked.** For every item in the column: does it exist (file:line)? Does it actually enforce, or only appear to? **Prove the answer by trying to break it.** Make the smallest plausible change that violates the guardrail (an extra parameter on `slot_for_day`, a `correct` index in a pre-reveal payload, a skipped transition, a hand-edited `correct` with no provenance, an admin route outside the token check, a participant string that asks someone to speak), run the test or lint that should catch it, record which test failed or that **nothing failed**, then revert. Name every mutation. A mechanism whose mutation survives is a gap.
- **Audit demand: met or not**, item by item, with evidence. Examples: G-1's "every PR touching the slot path re-runs the perfect-memory simulation" asks whether CI or `just test` actually runs it on such a PR. G-10's "audit and retire `build_deck.py`'s write" asks whether the write is still there.
- **Reachability, the attacker's question.** For the secrecy guardrails (G-3, G-4, G-8), go beyond the test suite: enumerate every route, WebSocket message type and static asset a participant device can reach in each phase (`room/src` route table and message enums), and confirm each one against the guardrail. A canary that scans payloads it knows about does not prove there is no payload it does not know about.
- **Verdict**: `HOLDS` · `HOLDS-WITH-GAPS` · `BROKEN` · `UNENFORCED` (ruling 3), followed by its gaps.

**Phase invariants (§4):** exactly the seven phases; every transition a host action; none skippable, none reversible except where §4 says so (*Run it again* opens a new room); wall, buzzer and host render from one phase value (AC-81); and in each phase, what each surface may show (`work` carries no ✓, receipt or colour). Mutate the transition table and confirm the state-machine test fails.

**Cross-cutting checks** (each its own short section): every persisted field has one writer and one reader (§3; list orphans). The admin channel (T-25) and `popquiz schedule`/`sync` (PR #47) are new credential and data paths since most guardrail tests were written, so audit them against G-9, G-10 and G-12 explicitly. Look for any rule that reads history, the ledger or past letters and could change tonight's output (PHILOSOPHY §2); that is BROKEN wherever it sits, including in an audit or a lint.

### Deliverables

- **(1) `docs/audit/2026-10-guardrail-audit.md`**: the report, in plain English in the repo's register (short sentences, no hedging, no marketing; the root `README.md` is the voice sample). It contains a header (audit tree sha, date, the commands run with their exit codes and test counts), a summary table (one row per G-ID plus *Phase invariants*: verdict, gap count by severity), one section per guardrail as above, the cross-cutting checks, the gap list (§3 format), *Closed since the interim report* (ruling 4), *What I could not verify* (anything that needs Docker, Fly, a deployed room, a phone or a human, with the reason), and the mutation table (mutation → file:line → test that caught it, or **survived**).
- **(2) `docs/audit/README.md`**: three lines saying what this directory holds and that an audit is advisory and never a CI gate.
- **Nothing else.** No code, test, lint, justfile, CI, bank or doc changes beyond those two files. If a gap's fix is one line, it is still a gap ticket.

### Validation (before the PR)

- `just test` green on the untouched tree (note the warm time and the per-suite counts); `just canary`; `just bank-audit` (its known WARN stays a WARN; quote it); `just secret-scan`. Quote only what you saw.
- Every mutation applied, the named test run, then reverted with `git restore <file>`. Run `git diff --quiet && git status --short` before the next mutation and before every commit. **After the last mutation, `git diff origin/main --stat` must show only your two docs files.** If a mutation needs a build (`cargo test -p …`), run only the narrowest test that should catch it.
- Attach the transcript (commands, exit codes, counts, the mutation table) as the validation artifact.

### Exit check before DONE

`git diff origin/main --stat` shows exactly `docs/audit/2026-10-guardrail-audit.md` and `docs/audit/README.md`, both new. No contract file, `.lattice/`, `.claude/`, `HANDOFF.md` or `REHEARSAL-GUIDE.md` appears in the diff. Every ruling in §2 appears as its own line in DONE with how you honoured it. Every mutation is listed with its outcome.

### Out of scope

Fixing anything. Minting or editing any ticket but PQ-28. The deploy and the deployed room. Docker-only verifier paths (list them under *could not verify* if a guardrail's proof needs them). The AC-26 tell's rule (PQ-48's open question; report what the code does, not what it should do). `REHEARSAL-GUIDE.md`.

## 3. The arc (sub-agent-full, in this tab)

**Plan.** Run `lattice status PQ-28 in_planning`. Write the plan to the path printed by `(cd "$LATTICE_ROOT" && lattice plan PQ-28)`. It is an **absolute** path under `$LATTICE_ROOT/.lattice/plans/`; the file exists with the ticket text at the top. Append below a `# Plan (delegator, date)` heading. The plan covers: per G-ID, the enforcement mechanisms you expect to find (with paths) and **the mutation you will try against each**; the reachability enumeration method for G-3/G-4/G-8; the auditor split below; the existing tickets you will cross-reference; and the report outline. **Plan review:** spawn one Agent subagent (`subagent_type: general-purpose`, `model: sonnet`, read-only, told the worktree path and the plan path, capped at 600 words) to find mechanisms you missed, mutations that would not actually test the guardrail, and guardrails you scoped too narrowly. Triage its findings into a `## Plan-Review Cycle 1 Resolutions (AUTHORITATIVE)` block in the plan, then run `lattice status PQ-28 planned`.

**Audit (implement).** Run `lattice status PQ-28 in_progress`, then `git fetch origin`. Fan out **four read-only auditor subagents in parallel** (Agent tool, `subagent_type: general-purpose`, `model: sonnet`). Give each the worktree path, its path globs, its guardrails' SPEC rows verbatim, the plan's mutation list for its guardrails, and a cap of 1,200 words. Each returns per mechanism: exists (file:line), the enforcing test, its candidate gaps, and the reachability list where it applies. **Auditors read only; they make no edits and run no mutations.**

- **A: answer integrity.** G-1, G-2, G-10, G-11, G-12. `pipeline/src/popquiz/**`, `pipeline/tests/**`, `mvp/tools/build_deck.py`, `mvp/tools/verify.py`, `bank/**`, `justfile`, `.github/workflows/**`.
- **B: secrecy.** G-3, G-4, G-8. `room/src/**`, `room/tests/**`, `web/shared/**`, `web/buzzer/**`, `web/wall/**`, `web/test/**`.
- **C: phase and receipt.** G-6, the §4 invariants, G-7. `room/src/phase.rs` and its callers, `room/tests/**`, `web/shared/**`, `pipeline/src/popquiz/receipt*`, `bank/fixtures/receipts/**`.
- **D: copy and auth.** G-5, G-9. `web/shared/copy.js` and its lint, `web/**`, `room/src/auth*`, `room/src/admin*`, `room/src/rooms*`, the route table, `Cargo.toml` features, `pipeline/src/popquiz/schedule.py` (the token path).

**You run every mutation yourself**, serially, in this worktree, one at a time with revert and a clean-tree check between them. Auditors propose; you prove. Then adversarially re-check every candidate gap: try to show it is *not* a gap (a test elsewhere catches it, the SPEC permits it). Only a gap that survives goes in the report. Write the report and the README. Small signed commits with plain-English messages, each ending with a `Co-Authored-By:` trailer naming the model you are actually running as (your status line shows it).

**Review.** Spawn one fresh-eyes review subagent (Agent tool, `subagent_type: general-purpose`, **`model: opus`**: this is the final adversarial read) with the worktree path, the branch, `git diff origin/main...HEAD --stat`, the plan path, SPEC §2 and §4, and the rulings in §2. Ask it to check that every HOLDS verdict is backed by a mutation that was tried and caught; that no verdict was copied from the interim report; that every gap has file:line evidence and a severity per ruling 6; that existing tickets are cross-referenced rather than re-filed; that no house rule is violated (above all, no unevenness reported as a gap); and that the diff is the two docs files only. Ask for a Verdict of PASS, PASS-WITH-NITS or FAIL, with Critical/Major/Minor/NIT findings and file:line. Fix Critical and Major; re-review once if the fixes were more than mechanical. **At most two fix→re-review cycles**; at the cap, stop and escalate. Attach the final verdict naming the reviewed commit: `lattice attach PQ-28 --type note --role review --title "Code review" --inline "<verdict, reviewed HEAD <sha>>" --actor agent:delegator-pq28-reviewer`.

**Validate.** Attach the transcript from §2's *Validation* with `lattice attach PQ-28 --type note --role review --title "Validation" --inline "…" --actor agent:delegator-pq28`. This install accepts only `--role review`.

**Gaps.** Post one comment, after the review and before the PR:

```
GAPS PQ-28 audit-tree 0c7d60a count <n> (critical <a> major <b> minor <c> nit <d>)
GAP-1 | <G-ID> | <severity> | <covered by PQ-NN (sufficient | needs: …) | new> | <one-line title> | <file:line evidence> | <what would close it>
…
```

**PR.** First re-read `lattice comments PQ-28` (§1a). Confirm `git log origin/main..HEAD` holds only your commits and `git rev-parse --show-toplevel` is this worktree. Push with `git push -u origin ai-c11-cc/guardrail-audit` (the client approves). Verify with `git fetch origin && test "$(git rev-parse HEAD)" = "$(git rev-parse origin/ai-c11-cc/guardrail-audit)"`, and re-push until equal. Open the PR **against `main`** with `gh pr create --base main --head ai-c11-cc/guardrail-audit`. The title is in plain words without the ticket ID. The body gives the summary table, the gap counts by severity, the line *Advisory: this PR changes no code; gaps are filed as held tickets by the Orchestrator*, the test counts, and `Lattice PQ-28 · BUILDPLAN T-23`, and ends with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Confirm the PR's head sha differs from its base. Then **serially**: `lattice attach PQ-28 <PR URL> --type reference --title "PR" --actor agent:delegator-pq28`, wait, `lattice status PQ-28 review`, and verify with `lattice show PQ-28 --json`.

**Completion comment**, then stop:

```
lattice comment PQ-28 "DONE PQ-28 PR <url> HEAD <sha> BASE <origin/main sha> audit-tree 0c7d60a verdicts: <G-1 …, …> gaps: <counts> mutations: <n tried, n caught, n survived> test: <counts, warm s> rulings: <one line each> comments-acted-on: <list> deviations: <none | numbered>"
```

Every sha and path in a READY or DONE receipt comes from `git -C /Users/michellerojas/rust-nyc-pop-quiz-worktrees/guardrail-audit …`, never from a bare `git` inside the `(cd "$LATTICE_ROOT" && …)` subshell. Update your description to say the PR is open and you are waiting. **Do not merge.** Do not address the client in comments; the tab is where they talk to you.

Refresh `c11 set-description` at each phase change (planning → auditing → in review → PR open), keeping the `Lineage:` line last.
