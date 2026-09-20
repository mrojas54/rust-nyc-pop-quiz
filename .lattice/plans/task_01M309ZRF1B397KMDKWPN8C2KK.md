# PQ-19: Verification sandbox image

BUILDPLAN.md T-15a (M3).

The sandbox: a Docker image with the pinned toolchain and nightly Miri — **the pin (`rustc -Vv` release and commit-hash, the nightly's date) is defined here, once** (`SPEC.md` §7.2, D-17) — run with no network, memory/CPU/pids limits, read-only root, tmpfs work dir, hard timeout; the AC-12 fixture suite proven on it

Criteria: AC-12
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-01
BUILDPLAN notes: Serialized on `justfile`

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
