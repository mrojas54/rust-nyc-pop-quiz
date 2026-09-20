# PQ-11: Deploy to Fly with the host stand-in

BUILDPLAN.md T-09 (M1).

Deploy: Fly app, `popquiz.rustnyc.org` (or fallback host), the short link carrying the code, `smoke`; the skeleton is built with the `dev-host-token` feature, `HOST_DEV_TOKEN` is set as a Fly secret, and the host URL is printed once (`SPEC.md` §8.2, D-19)

Criteria: AC-28, AC-64 (stand-in)
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-05–07
BUILDPLAN notes: Human track: DNS. Serialized on `justfile` (`smoke`). Adds the name `HOST_DEV_TOKEN`, never a value, to `.env.example`

Orchestrator notes: DNS may be absent: BUILDPLAN allows the Fly-provided hostname as fallback. Adds the name `HOST_DEV_TOKEN` to `.env.example`, a file the sandbox cannot read: that one-line edit is made by the Orchestrator outside the sandbox (F-5). The host URL is printed once, to the Orchestrator, never into a log or comment.

HELD: needs H-2 DNS (fallback host allowed) and a Fly deploy. Do not dispatch until the client releases it.

Workflow mode: fast-track. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.
