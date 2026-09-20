# Agents — Rust NYC Pop Quiz build

Active table is overwritten each tick; Lattice and `c11 tree` are ground truth.

## Active

| Role | Ticket | Surface ref | Pane ref | Branch | Worktree | Phase | Last seen | Spawned at |
|---|---|---|---|---|---|---|---|---|
| Orchestrator | — | surface:8 | pane:1 | ai-c11-cc/lattice-build | (root checkout) | Phase 1 dispatch, wave 2 | 2026-09-20 | 2026-09-20 |
| Delegator (inline-full, Opus) | PQ-2 / T-02 Shared web layer | surface:29 | pane:1 | ai-c11-cc/shared-web-layer (off repo-scaffold @ 2ab95e3) | …-worktrees/shared-web-layer | in_planning (receipt verified @ 2ab95e3) | 2026-09-20 | 2026-09-20 |
| Delegator (sub-agent-full inline, Opus) | PQ-3 / T-03 Burst spike | surface:30 | pane:1 | ai-c11-cc/burst-spike (off repo-scaffold @ 2ab95e3) | …-worktrees/burst-spike | in_planning (receipt verified @ 2ab95e3) | 2026-09-20 | 2026-09-20 |
| Delegator (inline-full, Opus) | PQ-18 / T-14 Bank format | surface:31 | pane:1 | ai-c11-cc/bank-format (off repo-scaffold @ 2ab95e3) | …-worktrees/bank-format | in_planning (receipt verified @ 2ab95e3) | 2026-09-20 | 2026-09-20 |
| Delegator (inline-full, Opus) | PQ-19 / T-15a Sandbox image | surface:32 | pane:1 | ai-c11-cc/sandbox-image (off repo-scaffold @ 2ab95e3) | …-worktrees/sandbox-image | in_planning (receipt verified @ 2ab95e3) | 2026-09-20 | 2026-09-20 |

### Archived (run history)

| Actor | Ticket | Outcome | Notes |
|---|---|---|---|
| agent:delegator-pq1 (Opus, inline-full, surface:24) | PQ-1 / T-01 Repo scaffold | `review`, PR #8 open, head `2ab95e3` on `7abfce0` | 5 commits. `just test` warm 0.28 s / cold 12.21 s. Plan review: 3 Major, 2 Minor, 1 NIT, all resolved. Code review round 1: one Major (shell-out guard overclaimed), fixed with a negative control. Deviations: `setup` recipe; no `rust-toolchain.toml`; `--role validation` rejected by the install; README corrected. Anomaly: push/PR approval path unverifiable under auto mode → `ask` rules added to project settings. CI unproven until the PR runs. |
