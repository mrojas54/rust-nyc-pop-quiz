# PQ-1: Repo scaffold, justfile, CI

BUILDPLAN.md T-01 (M0).

Repo scaffold: `room/` in whichever stack **H-1 decided** (Rust crate, or a Workers project), `pipeline/` (uv project, Python 3.12), `web/`, `bank/`, `justfile` with `test` (≤60 s, hermetic) and `test-full`, CI running both. States what `canary` runs as in `test`: an **in-process** harness (the router driven without a socket), with the deployed scan in `test-full`. States that `test` never runs the pinned `rustc` or Miri as a verification step and never Docker (building the room crate uses the machine's own `cargo`) — the verifier is reached through a stub `Runner` with recorded fixtures (`SPEC.md` §7.2, D-18) — and that `test-full` re-runs those cases on the T-15a image, so CI installs Docker for `test-full` only

Criteria: —
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): **H-1**
BUILDPLAN notes: Shared files: `justfile`, `pyproject.toml`, CI config

Orchestrator notes: Stack is decided: Rust `axum` crate in `room/`, not a Workers project (the row text is stale on that point, finding F-6). CI defines the `test-full` job and runs what exists; T-15a and T-21 extend it (F-12). Measure `just test` cold and warm and report both numbers in the completion comment; if the warm run exceeds 60 s that is a contract defect, stop and escalate (F-3).

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
