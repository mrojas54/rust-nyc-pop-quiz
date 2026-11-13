# ADR-0006: Unreleased questions live in SQLite on a Fly volume behind the admin API

**Status:** Proposed (Queen approved starting step 0.0 on 2026-11-13; this ADR becomes Accepted when she merges it)
**Date:** 2026-11-13
**Deciders:** Queen chose the store on 2026-11-12, the mock datastore on 2026-11-10 and `/last/{club}` staying public on 2026-11-13; Claude wrote this record
**Contract:** amends AC-101 and G-10 by dated note (see "Contract changes"). AC-23, AC-56, AC-57, AC-61, AC-7 and G-4 are unchanged.

## Goal

> Unreleased questions are readable only by the room and the organizer, and the schedule and used ledger survive a deploy.

Every step below is tested against that one sentence. Half 1 is the schedule and the used ledger surviving a deploy. Half 2 is unreleased questions staying private.

## Context

- `bank/questions/`, `bank/recordings/`, `bank/candidates/` and `bank/history.json` are in the public repo and its history. The source is runnable, so the whole question is secret, not only the answer. Eight records are already public and stay as examples and test data.
- The room keeps scheduled questions and the used ledger in memory. Every merge to `main` deploys, and a deploy wipes both.
- `/last` and `/last/{club}` are public by decision and show the last question a club ran, with its answer.
- The admin channel (AC-101) today accepts a scheduled question and returns the used record.

## Decision

- **One SQLite file on a small Fly volume** mounted at `/data`, opened only by the room. The file has no endpoint and no vendor.
- **The admin API is the only way to the bank.** The organizer's pipeline reads and writes questions through new `/admin/bank/...` routes behind the existing bearer check. A backup route returns a consistent copy of the file.
- **The datastore sits behind a trait**, injected as `HostAuth` is. `just test` injects in-memory SQLite running the real migrations. There is no Docker in the inner loop.
- **The correct option is never a column.** It is derived from the latest verification row, as `bank.correct_index()` does today (AC-7). `verifications` is append-only, written by one route.
- **A live night runs from memory.** The room loads upcoming meetups and the used list at start, and writes `used` once at release (AC-77). If the volume or file cannot be opened, the room refuses to start rather than run with an empty ledger.
- **`used` has one live row per club and question.** The database refuses a second run for one club and allows another club (AC-105). Rows are never deleted. A void mark takes a row out of the count, with a reason.
- **Live room state, participants, host sessions and secrets stay out of the store** (AC-56, AC-57, G-4).
- **Migrations are numbered SQL files** built into the binary and applied at start, additive only. The room copies the file before applying one.

## Non-goals

- Gating `/last`, or expiring links (decided 2026-11-13).
- Postgres, MongoDB, DynamoDB, Redis or a private bank repo (decided 2026-11-12; reasons in `store-options-considered`, kept outside this public repo).
- Docker in `just test` (decided 2026-11-10).
- A second machine, or live room state in the store.
- Per-person answers, totals or participant accounts.
- Changing the answer slot rule (AC-23).
- New questions, or edits to question text.
- Wall, phone and host page design.
- Rewriting git history to remove the eight public questions.

## Steps

Steps run in order. Each is one thread, one pull request, one visible outcome. Queen runs the hand checks and merges. A merge deploys. Route, table and recipe names below are proposals and may change in the PR that builds them.

| Step | Outcome | Acceptance check |
|---|---|---|
| **0.0 Contract** | The repo states the goal, the non-goals and each step's check before any code changes. | This ADR is merged. AC-101 carries its dated amendment. G-10 carries the void note. Only `docs/adr/`, `SPEC.md`, `sequence/USER_STORIES.md` and `sequence/run-state.md` changed. |
| **1.0 Durable room** | A deploy no longer forgets the schedule or the used ledger. | Schedule, restart, the room still creates. Release, restart: a repeat is refused for that club and allowed for another. `just test` stays offline and inside 60 s. |
| **2.0 Backup** | A copy of the store exists off the volume, and a restore has run once. | `just sync` writes a copy outside the repo, the copy opens, and the restore drill starts a throwaway room with the restored ledger. A request with no token gets no backup. |
| **3.0 Bank in the room** | A new question goes from draft to scheduled without touching the public repo. | Every existing record keeps its affirmation digest after save and load. A new question runs end to end with no new file under `bank/`. A guard test fails if any record beyond the eight public ones is committed. |
| **4.0 Clubs and organizers** | A club is added without a deploy, and the room records which organizer created a room. | A new club hosts with its own role. Another club's role and an unknown club both answer `wrong_role` (AC-103). *4.0 serves neither half of the goal; keep or defer is open and not decided here.* |

## Contract changes

- **AC-101** keeps its text. A dated note says the admin channel also returns bank records and a backup, and accepts bank saves, verifier records and voids. Everything else in AC-101 holds: the same credential, no other route accepts it, a wrong credential reveals nothing, and the credential is never in the repository, a payload, a page or a log. The token now opens the whole bank, so it matters more than before.
- **G-10** keeps its text. A dated note adds the void: a used row can be marked void with a reason by an authenticated admin action. A void row stays and no longer counts as a run for the club. The release transition is still the only writer of new `used` rows, and a void only marks an existing one. Wording is proposed and is settled by the step 1.0 PR.

## Consequences

**Benefits:** unreleased questions stop being public from step 3.0 on. Deploys stop wiping the schedule and ledger from 1.0 on. The database enforces one run per club.

**Costs and risks:**

- One disk on one machine. Fly snapshots and a laptop copy are the backup. Neither is real until the restore drill has run.
- The admin token becomes the key to the whole bank.
- The room must never log an `/admin` request or response body. This is not checked yet.
- Until step 3.0 lands, a new question is written to a directory outside the repo and not committed.
- The guardrail scans for the ledger's only writer and the sealed boundary must cover the SQL as well (ADR-0003, ADR-0004). Editing one needs a card first.
- Adding the SQLite crate needs one online setup, since builds are `--offline --locked`.

**Not decided here:** whether an already-public question may still run on a real night (assumed yes, as today), and whether `rooms` stays in step 1.0.
