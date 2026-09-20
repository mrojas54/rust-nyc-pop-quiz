# PQ-14: Room lifecycle and the used ledger

BUILDPLAN.md T-11 (M2).

Lifecycle: 4 h expiry, room deleted at release, totals with expiry, **`used` written at release**, take-it-home rebuild at release, *Run it again* refuses a used question

Criteria: AC-56, AC-57, AC-69, AC-92, G-4, G-10
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04b
BUILDPLAN notes: **The only writer of the used-question ledger.**

Orchestrator notes: The only writer of the used-question ledger (G-10). Touches `room/src/phase.rs` (F-10): serialized.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
