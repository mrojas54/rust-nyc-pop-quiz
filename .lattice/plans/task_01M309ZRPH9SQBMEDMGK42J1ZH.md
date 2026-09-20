# PQ-21: Generator via Message Batches

BUILDPLAN.md T-16 (M3).

The generator: Claude Opus 5 via Message Batches, structured outputs into the §3.1 shape, count/topics/difficulty honoured or reported, talk mode, per-candidate cost and wall-clock, re-runnable

Criteria: AC-1–5
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-14
BUILDPLAN notes: Human track: API key, spend cap. Serialized on `pyproject.toml`. The generator's brief carries the room's capacity — source lines at the floor, each option a single line, at most 29 characters (`SPEC.md` §7.1, D-15). The brief also carries the style rules and the §11.1 patterns (D-23).

Orchestrator notes: Human track H-4. Buildable and testable on fixtures without the key; the first paid run is HC-2.

HELD: needs H-4 an Anthropic API key and spend cap. Do not dispatch until the client releases it.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
