# PQ-28: Guardrail audit

BUILDPLAN.md T-23 (M4).

**Guardrail audit** — an adversarial read of all merged code against G-1…G-12 and the phase invariants; files gap tickets

Criteria: G-1…G-12
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): everything above
BUILDPLAN notes: The Phase-4 pass, re-run on the built tree

Orchestrator notes: Runs last, on the assembled tree. Files gap tickets; does not fix.

Workflow mode: sub-agent-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
