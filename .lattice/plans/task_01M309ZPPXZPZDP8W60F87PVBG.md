# PQ-4: Phase machine and sealed answers module

BUILDPLAN.md T-04a (M1).

The phase machine: `idle→…→released`, host-only transitions, no skipping, `trace_step`; **the sealed `answers` module** unreachable from the public state query, proven by a type/module boundary test

Criteria: AC-45, AC-47, AC-61, AC-81, AC-93, AC-97, G-3, G-6
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-03
BUILDPLAN notes: Built alone and first; `room/src/phase.rs` is consumed by everything in M1

Orchestrator notes: `room/src/phase.rs` is consumed by everything in M1: build it alone and first. AC-61 is a type or module boundary proof, not a review.

Workflow mode: sub-agent-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
