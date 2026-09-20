# PQ-16: Accessibility sweep

BUILDPLAN.md T-13 (M2).

`a11y`: keyboard, live regions per phase and the private hint, AA contrast, 44 px, reduced motion, over every surface and phase

Criteria: AC-82–86
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-05–07
BUILDPLAN notes: Serialized on `justfile` (`a11y`)

Workflow mode: fast-track. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
