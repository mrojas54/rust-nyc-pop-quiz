# Run state — Rust NYC Pop Quiz build (lattice-orchestrator)

Opened 2026-09-20. Phase 0 approved by the client the same day. Phase 0 record:
`phase0-trail.md` in the orchestrator's scratchpad (copied below where it matters).

## Configuration

| Key | Value |
|---|---|
| Autonomy | Moderate |
| N (max concurrent delegators) | 4 |
| PR merge policy | Leave at `review`; the client merges, or says the word and the Orchestrator merges with a merge commit |
| PR base | `origin/main` (PR #7 merged the contract amendments at `7abfce0`) |
| Git remote (verified) | `origin` = `git@github.com:mrojas54/rust-nyc-pop-quiz.git` |
| Terminal pre-merge status | **`review`** (classic preset: no `pr_open`, no `in_validation`) |
| Status vocabulary | `backlog · in_planning · planned · in_progress · review · done · blocked · needs_human · cancelled` |
| WIP limits | `in_progress` 10, `review` 29 (raised from 5 at init, F-4) |
| Ticket fidelity | Verbose |
| plan_review_mode / review_mode | `--mode single`, `LATTICE_SPAWN_BACKEND=headless` |
| Master Validator | On |
| Result Validator | On |
| Auto-close surfaces | On |
| c11 workspace | `workspace:1`, Orchestrator at `pane:1` (`surface:$C11_SURFACE_NUM`), Phase 0 document at `surface:22` |
| Runtime/browser approval scope | Delegators may start local servers and drive the c11 browser against them. Push, PR, deploy, merge, Docker and `flyctl` run outside the sandbox with the client's approval, one command at a time |
| Delivery-receipt path | `.lattice/orchestration/delivery-receipts.jsonl` |
| Run branch | `ai-c11-cc/lattice-build` (holds `.lattice/`; created from `ced5cb2`, fast-forwarded to `7abfce0`) |
| Models | Delegators Opus 5; headless reviews Sonnet 5; Result Validator Opus 5 |
| Held tickets (tag `held`) | T-09 (H-2 DNS; fallback host allowed) · T-10 (H-3 Discord app) · T-16 (H-4 API key) |
| Stop points | End of M1 → HC-0 to the client · before every push, PR, deploy, merge |
| Sandbox carve-out | `.claude/settings.json` re-allows reading `.env.example` (F-5), committed so worktrees inherit it |

## Standing clauses in every boot prompt

Criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Port `slot_for_day` only, never `slot_for_meetup`. `verify.py` never calls Miri; T-15b adds it. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Commit signing goes through 1Password: retry once, then escalate to the client, never bypass. Shared files are serialized: `justfile`, `pyproject.toml`, CI config, `web/shared`, `room/src/phase.rs`, `.env.example`.

## Workspace panes (c11 refs)

| Role | Ref |
|---|---|
| main_view_area | `pane:1` (Orchestrator `surface:8`; Phase 0 doc `surface:22`) |
| control_surface | `surface:23` (Lattice Board browser tab in `pane:1`) |
| delegate_view_area_1..3 | The window cannot split (`pane_too_small`); delegators open as tabs of `pane:1`, soft cap 15. `pane:5` is the client's own terminal and is never used for spawns |
| lattice_dashboard_port | 48731 (log: `$TMPDIR/lattice-dashboard-48731.log`) |

## Install facts pinned after init

- Lattice 0.2.0 has **no** `plan-review`, `code-review`, `claim`, `needs-human` or `review-status` commands. Reviews come from a Sonnet review subagent spawned inside the delegator's session with the Agent tool; verdicts attach via `lattice attach --role review`. Escalation is `lattice status <ID> needs_human` plus a c11 flag.
- `lattice plan <ID>` prints the plan path; `lattice complete <ID> --review "…"` runs the review → done ceremony (Orchestrator only).
- Delegators launch as `claude --model opus --permission-mode auto` in their worktree; the sandbox applies, and pushes, PRs, Docker and c11 calls prompt the client in the delegator's tab.
- Boot prompts are staged in the Orchestrator's scratchpad (`…/scratchpad/prompts/<ID>-<slug>.md`) with the Clause-1 cwd guard on line one.
- Worktrees live at `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/<slug>` on branches `ai-c11-cc/<slug>` from `origin/main`.

## Tickets in scope

| Ticket | Lattice | Title | Milestone | Mode | Status | Branch base |
|---|---|---|---|---|---|---|
| T-01 | PQ-1 | Repo scaffold, justfile, CI | M0 | inline-full | backlog | origin/main |
| T-02 | PQ-2 | Shared web layer: tokens, fonts, well, type model | M0 | inline-full | backlog | origin/main |
| T-03 | PQ-3 | Burst spike on Fly | M0 | sub-agent-full | backlog | origin/main |
| T-04a | PQ-4 | Phase machine and sealed answers module | M1 | sub-agent-full | backlog | origin/main |
| T-04b | PQ-5 | Sessions and the answer store | M1 | inline-full | backlog | origin/main |
| T-04c | PQ-6 | WebSocket transport and reconnect | M1 | inline-full | backlog | origin/main |
| T-05 | PQ-7 | The wall | M1 | inline-full | backlog | origin/main |
| T-06 | PQ-8 | The buzzer | M1 | inline-full | backlog | origin/main |
| T-07 | PQ-9 | The host phone | M1 | inline-full | backlog | origin/main |
| T-08 | PQ-10 | Canary secrecy suite | M1 | inline-full | backlog | origin/main |
| T-09 | PQ-11 | Deploy to Fly with the host stand-in | M1 | fast-track | backlog · **held** | origin/main |
| T-26 | PQ-12 | Static fallback and host sheet | M1 | inline-full | backlog | origin/main |
| T-10 | PQ-13 | Discord OAuth and role check | M2 | inline-full | backlog · **held** | origin/main |
| T-11 | PQ-14 | Room lifecycle and the used ledger | M2 | inline-full | backlog | origin/main |
| T-12 | PQ-15 | Take it home page | M2 | fast-track | backlog | origin/main |
| T-13 | PQ-16 | Accessibility sweep | M2 | fast-track | backlog | origin/main |
| T-25 | PQ-17 | Admin channel for the pipeline | M2 | inline-full | backlog | origin/main |
| T-14 | PQ-18 | Bank format and MVP migration | M3 | inline-full | backlog | origin/main |
| T-15a | PQ-19 | Verification sandbox image | M3 | inline-full | backlog | origin/main |
| T-15b | PQ-20 | Verifier with Miri and the pin | M3 | sub-agent-full | backlog | origin/main |
| T-16 | PQ-21 | Generator via Message Batches | M3 | inline-full | backlog · **held** | origin/main |
| T-17 | PQ-22 | Dedupe | M3 | fast-track | backlog | origin/main |
| T-18 | PQ-23 | Review surface | M3 | inline-full | backlog | origin/main |
| T-19 | PQ-24 | Bank audit and the slot function | M3 | sub-agent-full | backlog | origin/main |
| T-20 | PQ-25 | Schedule and sync | M3 | inline-full | backlog | origin/main |
| T-21 | PQ-26 | Burst and smoke in CI | M4 | inline-full | backlog | origin/main |
| T-22 | PQ-27 | Copy freeze and lints | M4 | inline-full | backlog | origin/main |
| T-23 | PQ-28 | Guardrail audit | M4 | sub-agent-full | backlog | origin/main |
| T-24 | PQ-29 | Organizer runbook | M4 | fast-track | backlog | origin/main |

Dependencies: 89 `depends_on` links on the board, one per BUILDPLAN dependency (T-23 on everything). Longest chain 10.

## Amendments routed upstream to tone-architect (not authored here)

- **F-2.** `EVALUATION.md` HC-0 trigger cites `burst` green, but `burst` is T-21 (M4). Proposed one-line amendment: HC-0's burst evidence is T-03's spike measurement re-run against the deployed skeleton. Client accepted this reading 2026-09-20; the run proceeds on it.
- **F-8.** `SPEC.md` §3.1 lists the review surface as the writer of `review`, but §7.3 has dedupe marking `near_duplicate_of`. Ruled for the run: T-17 writes it, T-18 reads it. Proposed amendment: name dedupe as that field's writer in §3.1.

## Decision log (append-only)

- 2026-09-20 [autonomy: Moderate] Run branch `ai-c11-cc/lattice-build` created from `ced5cb2` per the client's instruction; not from `main`, not from the empty `ai-c11-cc/build-orchestration`.
- 2026-09-20 [client] `fly auth whoami` = logged in. T-03 released from the hold list.
- 2026-09-20 [client] PR #7 (`ai-c11-cc/contract-amendments` → `main`) opened and merged with a merge commit at `7abfce0`. Every ticket PR bases on `origin/main`.
- 2026-09-20 [client] `.env.example` sandbox carve-out via `sandbox.filesystem.allowRead` in the committed `.claude/settings.json`.
- 2026-09-20 [client] F-2 reading accepted; F-4 WIP raise accepted; config and validation plan approved as drafted.
- 2026-09-20 [autonomy: Moderate] Lattice `opinionated` preset checked: identical statuses, transitions and limits to `classic`; only `display_names` differ. `classic` kept as instructed.
- 2026-09-20 [autonomy: Moderate] Held tickets carry the tag `held` in `backlog` because `backlog → blocked` is not a legal transition on this preset; the dispatch loop never spawns a `held` ticket.
- 2026-09-20 [autonomy: Moderate] T-19's ticket lists AC-23a and AC-23b explicitly (F-7). T-17 owns `near_duplicate_of` (F-8). T-02 lands ported strings in `web/shared/copy` (F-11).
- 2026-09-20 [autonomy: Moderate] Run files committed on `ai-c11-cc/lattice-build` at `e3d9e50` (signed; 1Password signing worked from this session outside the sandbox).
- 2026-09-20 [autonomy: Moderate] PQ-1 at `review`: PR #8 (`ai-c11-cc/repo-scaffold` @ `2ab95e3` on `7abfce0`), `just test` warm 0.28 s, cold 12.21 s. Five deviations flagged by the delegator, all accepted: a `setup` recipe owning the one network step; `.gitignore` untouched (Orchestrator amendment R-7: never ignore `.lattice/` or `HANDOFF.md`); no `rust-toolchain.toml` (slot left for T-15a's pin); validation notes attach as `--role review` because the install accepts no other role; root `README.md` corrected. CI unproven until the PR runs it.
- 2026-09-20 [autonomy: Moderate] **Policy tightening.** The delegator's push and PR went through auto mode; whether the client approved them in the tab is not knowable from here. `.claude/settings.json` now carries `ask` rules for `git push`, `gh pr create`, `gh pr merge`, `fly`, `flyctl` and `docker`, so those always prompt. The file is copied into every worktree at creation and left uncommitted there; it reaches `main` with the orchestration branch.
- 2026-09-20 [autonomy: Moderate] Wave 2 dispatched by press-ahead off `origin/ai-c11-cc/repo-scaffold` @ `2ab95e3`: PQ-2 (shared web layer), PQ-3 (burst spike), PQ-18 (bank format), PQ-19 (sandbox image). Their PRs base on `ai-c11-cc/repo-scaffold`; **retarget each to `main` after #8 merges and before its branch is deleted.** PQ-3's sub-agent-full runs inline in one tab (planner and reviewers as Agent-tool subagents) because the window cannot split.
- 2026-09-20 [autonomy: Moderate] Two cross-language rulings recorded for the validator: the receipt (§7.5) is one Python function in `popquiz.receipt` plus a JavaScript twin in `web/shared` (T-05), both tested against one fixture set at `bank/fixtures/receipts/`; the type-model formula lives in `web/shared/typemodel.js` and T-19 mirrors it in Python against the same worked examples.
- 2026-09-20 [autonomy: Moderate] PQ-3 flagged three ticket-text vs brief divergences. Ruled for all wave-2 tickets by comment: the brief governs (PR base is the scaffold branch until #8 merges; fly/push/PR run in the delegator's tab under the `ask` rules). Third point was a real gap: worktree sandboxes could not write the root board, so every `lattice` write needed a bypass. `.claude/settings.json` now adds `sandbox.filesystem.allowWrite` for the board directory; re-seeded into all five worktrees.
- 2026-09-20 [client: "merge 8"] PR #8 merged to `main` with a merge commit, `0742d35`, after both CI jobs passed on `2ab95e3`. PQ-1 completed on the board. Wave 2 told to rebase onto `origin/main` and base their PRs on `main`; the scaffold branch is left on `origin` until all four have rebased. No stacked PRs existed yet, so nothing needed retargeting.
- 2026-09-20 [autonomy: Moderate] Footgun: never `git switch` in the root checkout while the board has uncommitted changes (git refuses, correctly; the working tree is the database). Update the local `main` ref with `git fetch origin main:main` instead.
- 2026-09-20 [autonomy: Moderate] Phase 1 opened. PQ-1 (T-01) dispatched at surface:24 in worktree `repo-scaffold`, branch `ai-c11-cc/repo-scaffold` from `origin/main` @ `7abfce0`. Wave 2 (T-02, T-03, T-14, T-15a) spawns when PQ-1 reaches `review` (press-ahead), branching off the in-review scaffold branch.

## Run-time footguns

| Symptom | Cause | Mitigation |
|---|---|---|
| `git status` prints `.env.example: Operation not permitted`; `git switch` prints `D .env.example` | Sandbox denies `**/.env.*`, which matches `.env.example` | Carve-out in `.claude/settings.json` (`sandbox.filesystem.allowRead`). Never `git add -A` regardless |
| `c11` socket timeouts (10 s) mid-session | Transient; the app recovered within minutes | Retry once after a minute; do not spawn while it is down |
| `c11 new-pane` refuses: `pane_too_small` | The Orchestrator's window is narrow | Delegate surfaces open as tabs of an existing pane, not new panes, unless the client widens the window |
| A freshly launched delegator sits on "allow reading from …/scratchpad/prompts" | The boot prompt lives outside the delegator's worktree, so auto mode asks once before reading it | Stage each boot prompt inside the worktree at `.claude/boot-prompt.md` (beside the seeded settings file, left uncommitted); if a tab is already waiting, the Orchestrator answers Yes with `send-key enter` |
| `c11 new-surface` times out after ~10 s but the surface exists | Socket contention while several surfaces spawn | Always `c11 tree` before retrying a create; launch into the surfaces that appeared rather than creating more |
