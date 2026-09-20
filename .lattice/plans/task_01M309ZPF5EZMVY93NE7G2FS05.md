# PQ-2: Shared web layer: tokens, fonts, well, type model

BUILDPLAN.md T-02 (M0).

`web/shared/`: tokens and fonts vendored from the design system; port `proto.js` — source well, syntax colour, trace renderer, the type model with measured refit and the clipped-edge, the phase strings from `SPEC.md` §11

Criteria: AC-99, AC-100, AC-40
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-01
BUILDPLAN notes: Built alone; three tickets consume it

Orchestrator notes: Put the ported §11 phase strings in `web/shared/copy` from the start so T-22 moves nothing (F-11). Port from `prototypes/_shared/proto.js`, `tokens.css`; fonts from the design system per SPEC §15 (D-14). The source well font size is behaviour (SPEC §5.2), never read off the prototype; every other size is the prototype's.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
