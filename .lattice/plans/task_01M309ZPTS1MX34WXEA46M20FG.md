# PQ-5: Sessions and the answer store

BUILDPLAN.md T-04b (M1).

Sessions and the answer store: join, capacity, answer upsert while live, refusal after close, totals frozen at close, sessions dropped at release

Criteria: AC-30, AC-34–37, AC-46, AC-56
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a
BUILDPLAN notes: —

Orchestrator notes: Touches `room/src/phase.rs` (F-10): serialized after T-04a and before T-04c.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
