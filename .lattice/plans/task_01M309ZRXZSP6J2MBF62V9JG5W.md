# PQ-23: Review surface

BUILDPLAN.md T-18 (M3).

Review surface (local, served by the CLI): one screen per candidate, accept/reject-with-reason/edit-and-reverify, difficulty judged, **affirm** as a blocking gate with who/when, middle-beat and quoted-output checks; §11.1 trope matches highlighted beside `explains`, `why_tempting` and `hint` as warnings that block nothing (`SPEC.md` §7.4, D-23)

Criteria: AC-19–22, AC-72–74, AC-88, AC-95, G-12
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-14, T-15b
BUILDPLAN notes: —

Orchestrator notes: Reads `near_duplicate_of` written by T-17.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
