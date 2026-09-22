# Agents — Rust NYC Pop Quiz build

Active table is overwritten each tick; Lattice and `c11 tree` are ground truth.

## Active

| Role | Ticket | Surface ref | Pane ref | Branch | Worktree | Phase | Last seen | Spawned at |
|---|---|---|---|---|---|---|---|---|
| Orchestrator | — | surface:8 | pane:1 | ai-c11-cc/lattice-build | (root checkout) | Phase 1 dispatch, wave 2 | 2026-09-20 | 2026-09-20 |
| Delegator (sub-agent-full inline, Opus) | PQ-3 / T-03 Burst spike | surface:30 | pane:1 | ai-c11-cc/burst-spike (off repo-scaffold @ 2ab95e3) | …-worktrees/burst-spike | in_planning (receipt verified @ 2ab95e3) | 2026-09-20 | 2026-09-20 |
| Delegator (inline-full, Opus) | PQ-19 / T-15a Sandbox image | surface:32 | pane:1 | ai-c11-cc/sandbox-image (off repo-scaffold @ 2ab95e3) | …-worktrees/sandbox-image | in_planning (receipt verified @ 2ab95e3) | 2026-09-20 | 2026-09-20 |
| Delegator (sub-agent-full inline, Opus) | PQ-24 / T-19 Bank audit and the slot function | surface:34 | pane:1 | ai-c11-cc/bank-audit (off bank-format @ b4d946c) | …-worktrees/bank-audit | booting | 2026-09-21 | 2026-09-21 |
| Delegator (fast-track, Opus) | PQ-22 / T-17 Dedupe | surface:35 | pane:1 | ai-c11-cc/dedupe (off bank-format @ b4d946c) | …-worktrees/dedupe | booting | 2026-09-21 | 2026-09-21 |

### Archived (run history)

| Actor | Ticket | Outcome | Notes |
|---|---|---|---|
| agent:delegator-pq1 (Opus, inline-full, surface:24) | PQ-1 / T-01 Repo scaffold | `review`, PR #8 open, head `2ab95e3` on `7abfce0` | 5 commits. `just test` warm 0.28 s / cold 12.21 s. Plan review: 3 Major, 2 Minor, 1 NIT, all resolved. Code review round 1: one Major (shell-out guard overclaimed), fixed with a negative control. Deviations: `setup` recipe; no `rust-toolchain.toml`; `--role validation` rejected by the install; README corrected. Anomaly: push/PR approval path unverifiable under auto mode → `ask` rules added to project settings. CI unproven until the PR runs. |
| agent:delegator-pq2 (Opus, inline-full, surface:29) | PQ-2 / T-02 Shared web layer | `review`, PR #9 open, head `223a201` on `0742d35` | 5 signed commits, 25 files, 103 tests, browser validation 34/34. Plan review 0C/3M/6m. Code review PASS-WITH-NITS after 3 rounds. Deviations: Google Fonts @import dropped (F-17); components.css and phase.js added; no rounding in the refit loop (T-19 mirrors); screen_height_ft configured; prototype split-column layout not ported. Anomalies: Learning-style handoff stall; 1Password signing outage cleared by client with ssh-keygen recipe. |
| agent:delegator-pq18 (Opus, inline-full, surface:31) | PQ-18 / T-14 Bank format | `review`, PR #10 open, head `b4d946c` on `0742d35` | 5 signed commits, CI green, `just test` warm 0.66 s. Code review PASS-WITH-NITS. Deviations: shared receipt fixtures under bank/fixtures/receipts/; `why_tempting` on Option not under `explains` (F-18); `correct` derived not stored; re-authoring note in review.reason; receipt_detail returns facts not copy; schema enforced by drift test, no validator (flagged for T-19); synthetic receipt fixtures labelled. Anomalies: F-15 contract defect found; 1Password signing outage cleared by client. |
