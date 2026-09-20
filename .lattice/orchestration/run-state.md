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
| main_view_area | `pane:1` (Orchestrator; Phase 0 doc `surface:22`) |
| control_surface | see decision log (layout constrained; filled at first dispatch) |
| delegate_view_area_1..3 | see decision log |
| lattice_dashboard_port | see decision log |

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

## Run-time footguns

| Symptom | Cause | Mitigation |
|---|---|---|
| `git status` prints `.env.example: Operation not permitted`; `git switch` prints `D .env.example` | Sandbox denies `**/.env.*`, which matches `.env.example` | Carve-out in `.claude/settings.json` (`sandbox.filesystem.allowRead`). Never `git add -A` regardless |
| `c11` socket timeouts (10 s) mid-session | Transient; the app recovered within minutes | Retry once after a minute; do not spawn while it is down |
| `c11 new-pane` refuses: `pane_too_small` | The Orchestrator's window is narrow | Delegate surfaces open as tabs of an existing pane, not new panes, unless the client widens the window |
