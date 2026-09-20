# PQ-17: Admin channel for the pipeline

BUILDPLAN.md T-25 (M2).

The pipeline channel, server side: `PUT /admin/questions/{id}` into the sealed `answers` module and `GET /admin/used`, one constant-time bearer check on `POPQUIZ_ADMIN_TOKEN`, a route-table test that nothing else is served under the admin prefix and no other route reads the token (`SPEC.md` §8.3, D-20); extends `canary` with `POPQUIZ_ADMIN_TOKEN` as a plant and adds the repo-wide scan for the token to `test` (`EVALUATION.md` AC-101)

Criteria: AC-101, AC-61, G-9
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a, T-08, T-11
BUILDPLAN notes: Human track: H-11, for the deployed check only. Adds the name `POPQUIZ_ADMIN_TOKEN`, never a value, to `.env.example`. Serialized on `justfile`

Orchestrator notes: Touches `room/src/phase.rs` (F-10) and `.env.example` (F-5, edit made outside the sandbox by the Orchestrator). Serialized on `justfile`.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
