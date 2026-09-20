# PQ-3: Burst spike on Fly

BUILDPLAN.md T-03 (M0).

**Spike — the burst.** Minimal `axum` WebSocket room with 200 synthetic clients; the deadline burst in isolation; p95 write and reveal-fan-out measured on a deployed Fly machine. Go/no-go on D-A.

Criteria: AC-52, AC-53, AC-54, AC-41
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-01
BUILDPLAN notes: **Blocks M1.** If p95 fails, D-A option 2 is re-opened before any room code is written.

Orchestrator notes: Fly account is logged in (client confirmed 2026-09-20). The deploy itself runs outside the sandbox with the client's approval: prepare everything, then hand the exact `fly` commands to the Orchestrator. Attach p95 write and reveal fan-out numbers as an artifact; they are HC-0's burst evidence (F-2). Go/no-go on D-A is the client's call: report, do not decide.

Workflow mode: sub-agent-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
