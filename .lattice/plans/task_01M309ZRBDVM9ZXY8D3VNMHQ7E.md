# PQ-18: Bank format and MVP migration

BUILDPLAN.md T-14 (M3).

Bank format (`SPEC.md` §3.1–3.3) and migration of the eight MVP questions: verified facts carried over **as `legacy` records — only the fields the MVP wrote, nothing back-filled** (`SPEC.md` §3.2, D-16), the MVP's `error_codes` renamed to `compile_error_code` on q8's record, `explains` three beats and `trace` drafted for organizer affirmation, `explanation` kept as `legacy`

Criteria: AC-13, AC-17, AC-73, AC-87
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-01
BUILDPLAN notes: **Only q3 migrates as authored** (touchpoint T-22, D-15). q4, q7 and q8 fit as programs but their options must be re-authored to one line of at most 29 characters and re-verified (the machine decides the new answer). q1, q2, q5 and q6 exceed the wall's 7-line capacity at the guessed room and stay in the MVP bank only, until re-authored or until the rehearsal's measurements raise the capacity.

Orchestrator notes: Only q3 migrates as authored (D-15). Never write down what a program prints: legacy records carry only what `mvp/2026-08-12/verified.json` holds, nothing back-filled.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
