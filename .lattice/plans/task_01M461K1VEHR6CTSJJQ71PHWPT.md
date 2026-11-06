# PQ-61: Deployed burst keeps an organizer session and the admin token as repo secrets

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-19** (G-9, Minor, new): Deployed burst keeps an organizer session and the admin token as repo secrets
- Evidence: .github/workflows/deployed-burst.yml:7-11,58-67,100
- What would close it: Name it in SPEC 8 as a credential path, or pass them as one-run inputs

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-11-06)

## Re-verification at c810d0d (`.github/workflows/deployed-burst.yml`)

- Triggers: `on:` has one key, `workflow_dispatch` (inputs `url`, `participants`, `confirm`), lines 33-47. No push, no schedule.
- `confirm` job (59-70): no `environment:`, no `secrets.`; compares the input with the sentence.
- `deployed-burst` job (72-218): `needs: confirm`, `environment: deployed-burst` (75).
- Every read of the two names is a `${{ secrets.… }}` value of an `env:` key in that job: 86, 87 (`!= ''` booleans only), 133 (`POPQUIZ_ADMIN_TOKEN`), 157 and 172 (`POPQUIZ_ORGANIZER_SESSION`).
- Shell use: the admin token appears once in a script, `printf 'header = "Authorization: Bearer %s"\n' "$POPQUIZ_ADMIN_TOKEN" |` (140), piped to `curl --config -`, so it is never an argument; the session is read by the `burst`/`smoke` binaries from the environment, never named in a script.
- Header: line 28-31 says the secrets are the `deployed-burst` environment's, with a required reviewer and a main-only branch rule. **Drift inside the header:** lines 5-7 still say "set as a **repository** secret right before a run". The step summary (210) already says `gh secret delete … --env deployed-burst`.
- Other workflows: `ci.yml` reads no secret (its one mention of `deployed-burst.yml` is a comment, line 90); `deploy.yml` reads only `FLY_API_TOKEN`.
- So the repository-secrets half of GAP-19 has already moved to an environment. What remains is the contract: SPEC §8 does not list this holder, and §8.3 says the admin token is *never in the repository*.

Note: the burst adds no new authorization *function*. The organizer session is checked by `discord.rs` and the admin token by `AdminToken::check`, as today. It adds a **holder** of two existing credentials.

## The reserved decision (QUESTION PQ-61, credential path)

**A (recommended): name it in SPEC §8.** Two edits:

1. §8.3, the sentence "a Fly secret that the organizer's local pipeline configuration also holds and that is **never in the repository**" gains, after it: "(its one other holder is the deployed burst's environment, §8.4)".
2. A new subsection after §8.3:

> ### 8.4 The deployed burst (T-21, AC-53)
>
> AC-53's run against the deployed substrate, `.github/workflows/deployed-burst.yml`, carries two credentials: the organizer session of a signed-in `/host` page (`POPQUIZ_ORGANIZER_SESSION`, one person's, 12 hours) and the admin token (`POPQUIZ_ADMIN_TOKEN`, §8.3). Both are secrets of the `deployed-burst` GitHub **environment**, never repository secrets: the client sets them right before a run and deletes them right after. The environment's required reviewer and its main-only branch rule keep a workflow edited on a branch from reading them. The workflow runs only on `workflow_dispatch`; only the job that declares that environment reads them, and only into an `env:` mapping; neither is echoed, put on a command line or written to a file. The functions that already check them still do: the burst adds a holder, not a path. A `test` keeps the workflow to this paragraph.

EVALUATION row: none strictly needed (AC-53's row is about the measurement). If wanted, append to the AC-53 row: "The run's two credentials are held as `SPEC.md` §8.4 says; `test`: `harness_config.rs` keeps the workflow to it."

Placement: PQ-80 also touches §8.3/§8.4. I land first; if PQ-80 adds its own §8.4 it renumbers on its merge of main.

**B: one-run inputs.** Worse: `workflow_dispatch` inputs are not masked; they sit in plaintext in the run's event payload (`github.event.inputs`), in the Actions UI's run summary and in the API, readable by anyone with read access to the repository for the run's retention (90 days by default), and `${{ inputs.x }}` in a script would interpolate them into the script text. It also drops the required-reviewer gate, which today is what keeps a branch's workflow away from them.

**C: contract note only.** No change; GAP-19 stays partly met.

Also asked: may I change the header's "set as a repository secret" (deployed-burst.yml:6) to "set as a secret of the `deployed-burst` environment"? It contradicts the header's own line 28 and the README. One line; outside the cleared files otherwise.

## The guard (no ruling needed)

Home: `popquiz/tests/harness_config.rs`, the suite that already holds line-shaped checks on both workflows (PQ-41), no YAML parser in the lock. It runs in `just test` (`test-room` = `cargo test --offline --locked`). Added as new test functions; no existing test changes.

Conditions, and the plant that shows each RED (each a reverted background mutation, killed at 15 minutes):

1. **No other workflow reads the names.** Every file in `.github/workflows/` other than `deployed-burst.yml`: no code line (comments skipped) names `POPQUIZ_ORGANIZER_SESSION` or `POPQUIZ_ADMIN_TOKEN`, and none declares `environment: deployed-burst`. Plant: a step in `ci.yml` with `env: X: ${{ secrets.POPQUIZ_ADMIN_TOKEN }}`.
2. **Read only inside a job that declares `environment: deployed-burst`.** Each job of `deployed-burst.yml` that names either secret has that line; no `toJSON(secrets)` or `secrets: inherit` anywhere. Plant: a new job `leak:` (no environment) whose step reads `secrets.POPQUIZ_ORGANIZER_SESSION`.
3. **Only `workflow_dispatch`.** The line is exactly `on:` and its only child key is `workflow_dispatch:`. Plant: add `  push:` under `on:`.
4. **Only through an `env:` mapping.** Every `${{ … }}` naming either secret is the value of a key in an `env:` block (not `run:`, not `with:`); in scripts, `$POPQUIZ_ADMIN_TOKEN` appears only as the argument of a `printf` piped to stdin, and `$POPQUIZ_ORGANIZER_SESSION` never. Plants: (a) `echo "${{ secrets.POPQUIZ_ADMIN_TOKEN }}"` in a run script; (b) `jq --arg t "$POPQUIZ_ADMIN_TOKEN" …`; (c) `printf '%s' "$POPQUIZ_ADMIN_TOKEN" > "$RUNNER_TEMP/t"` (a file write).

Timing: measure warm `just test` before and after.

## README / RUNBOOK

- `popquiz/README.md` *Burst*, From CI (1184-1221): environment (1193, 1202-1204), required reviewer (1208), main-only (1209-1210), set before a run (1211), delete after (1219-1220). Agrees with the header's line 28-31; no change planned.
- `docs/RUNBOOK.md:174-181`: says never during a meetup and defers the rest to the README *Burst*. Silent on the secrets, not in disagreement; no change planned.
- The only disagreeing sentence is the workflow header's own line 6 (asked above).

## Reset 2026-11-06 by agent:delegator-pq61

## Reset 2026-11-06 by agent:delegator-pq61

## Reset 2026-11-06 by agent:delegator-pq61
