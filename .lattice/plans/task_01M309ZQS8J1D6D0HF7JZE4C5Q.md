# PQ-13: Discord OAuth and role check

BUILDPLAN.md T-10 (M2).

Discord OAuth: scopes, member lookup, role-**ID** check at room creation, the Administrator test, refresh-token rotation persisted, bounded retries, denial wording; **deletes the M1 stand-in** (the `dev-host-token` feature) and adds the test that a build without it accepts no stand-in token (`SPEC.md` §8.2)

Criteria: AC-64–70, G-9
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a–b
BUILDPLAN notes: Human track: app id, secret, guild id, role id. The live check against the client's account is this ticket's exit criterion and gates HC-1

Orchestrator notes: Human track H-3. The live check against the client's account is the exit criterion and gates HC-1, never HC-0.

HELD: needs H-3 the Discord app: id, secret, guild id, role id. Do not dispatch until the client releases it.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
