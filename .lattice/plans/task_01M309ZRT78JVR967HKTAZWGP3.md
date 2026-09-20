# PQ-22: Dedupe

BUILDPLAN.md T-17 (M3).

Dedupe: exact hash, normalized-AST fingerprint, near-duplicates by token-bigram Jaccard at the configured threshold (D-13) to the review queue, persistent history with visible size

Criteria: AC-14–18
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-14
BUILDPLAN notes: Serialized on `pyproject.toml`

Orchestrator notes: Writes `review.near_duplicate_of` (F-8 ruling); T-18 reads it.

Workflow mode: fast-track. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
