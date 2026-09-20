# PQ-10: Canary secrecy suite

BUILDPLAN.md T-08 (M1).

`canary`: plant secrets in answer, explanation, receipt, hint-before-live; scan every payload, frame, and page in every phase; post-close traffic carries no answer

Criteria: AC-32, AC-47, AC-48, AC-58, AC-60, AC-79, G-3, G-4, G-8
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a–07
BUILDPLAN notes: In-process scan in `test`; the deployed-room scan in `test-full`. Serialized on `justfile`

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
