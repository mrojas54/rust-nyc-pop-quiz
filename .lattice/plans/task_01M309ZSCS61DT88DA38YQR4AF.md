# PQ-27: Copy freeze and lints

BUILDPLAN.md T-22 (M4).

The copy freeze: every participant-facing string from `SPEC.md` §11 in one module; the forbidden-copy lint and the §11.1 trope check in `test`, the trope check failing the build on any match in the copy module (D-23)

Criteria: AC-98, AC-59, AC-42, G-5
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-05–07, T-12
BUILDPLAN notes: —

Orchestrator notes: Creates or completes `web/shared/copy` (F-11): serialized on `web/shared`. The forbidden list and the §11.1 patterns are SPEC §11's, verbatim.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
