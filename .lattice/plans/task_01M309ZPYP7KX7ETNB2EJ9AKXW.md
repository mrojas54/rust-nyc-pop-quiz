# PQ-6: WebSocket transport and reconnect

BUILDPLAN.md T-04c (M1).

Transport: one broadcast per room over WebSockets, reconnect with the same session token, host and wall subscriptions

Criteria: AC-37, AC-41, AC-81
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a
BUILDPLAN notes: —

Orchestrator notes: Touches `room/src/phase.rs` (F-10): serialized after T-04b.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
