# PQ-26: Burst and smoke in CI

BUILDPLAN.md T-21 (M4).

`burst` and `smoke` in `test-full`, run against the deployed room in CI; failed-request logging the client can read after a meetup

Criteria: AC-41, AC-52–55
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-09
BUILDPLAN notes: Serialized on `justfile` (`burst`, `test-full`)

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
