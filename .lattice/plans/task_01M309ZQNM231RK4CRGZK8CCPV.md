# PQ-12: Static fallback and host sheet

BUILDPLAN.md T-26 (M1).

The static fallback: `mode: "static"` in the wall (no socket, the phase held locally, the question baked in), the `Space`/`←`/`→`/`Esc` driver, and a one-file build with fonts and tokens inlined that makes no network request; the same fixture renders identically in both modes; and the **host sheet** function, a question record to plain text, that T-20 calls beside the build (`SPEC.md` §12, D-21, D-24)

Criteria: AC-102, AC-97, AC-99
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-02, T-05
BUILDPLAN notes: Not part of HC-0's trigger, but the client opens the file offline at HC-0 and reads the sheet against it. Edits `web/wall/`; consumes `web/shared/`

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
