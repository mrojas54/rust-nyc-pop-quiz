# PQ-25: Schedule and sync

BUILDPLAN.md T-20 (M3).

`popquiz schedule` / `sync`: push the affirmed question to the server over the §8.3 channel and write the **static fallback** file and its **host sheet** with T-26's build; pull `used`; reserve count, trend, and the low-reserve warning; a meetup from reserve with no generation

Criteria: AC-75–77, AC-89, AC-92, AC-101, AC-102, `SPEC.md` §12, G-12
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-11, T-18, T-19, T-25, T-26
BUILDPLAN notes: Human track: H-11, for the first deployed push

Orchestrator notes: Calls T-26's build and host-sheet functions; pushes over T-25's channel. H-11 (admin token) is needed only for the first deployed push, not to build or test.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
