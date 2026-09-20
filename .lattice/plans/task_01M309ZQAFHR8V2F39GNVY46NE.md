# PQ-9: The host phone

BUILDPLAN.md T-07 (M1).

The host phone: one screen per phase, one primary action, counts, trace stepping, the read-aloud script with the beats' *human* provenance marker (AC-74), resume on refresh/device change; in M1 *Create a room* presents the stand-in host token carried in the page URL (`SPEC.md` §8.2)

Criteria: AC-45–47, AC-49, AC-50, AC-74
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-02, T-04a–c
BUILDPLAN notes: —

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
