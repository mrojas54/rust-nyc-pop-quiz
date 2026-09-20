# PQ-20: Verifier with Miri and the pin

BUILDPLAN.md T-15b (M3).

The verifier in code: inherit `verify.py`'s procedure after audit; **add Miri** (strict provenance, Tree Borrows for UB-intended), the flag set, target triple, the §3.2 record, **the pin comparison and stale rejection, and the `Runner` seam with its stub and recorded fixtures** (`SPEC.md` §7.2, D-17, D-18); `verify <program>`

Criteria: AC-6–11, AC-13, AC-87, G-2
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-14, T-15a
BUILDPLAN notes: Audit ticket for `verify.py`. Serialized on `pyproject.toml` with T-16, T-17

Orchestrator notes: `mvp/tools/verify.py` never calls Miri (confirmed by grep); this ticket adds it. The pin is read from T-15a's definition, never re-declared. Audit ticket: read `verify.py` before inheriting.

Workflow mode: sub-agent-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
