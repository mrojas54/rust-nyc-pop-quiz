# PQ-76: ADR-0005 item 8: rename the default club nyc to rustnyc

ADR-0005 Action Item 8: rename the default club nyc to rustnyc, with an alias for nyc and a ledger migration as the ADR describes. Back up ledgers before migrating.

# Plan (delegator, 2026-10-18)

Worktree `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/rustnyc-rename`, branch
`ai-c11-cc/rustnyc-rename` @ `c5c42f7` (= `origin/main`). Mode inline-full.

## Outcome and acceptance check

The default club's slug is `rustnyc` everywhere it is stored or emitted. `nyc`
still works everywhere a slug comes in, and reads as `rustnyc`. A question used
under `nyc` is never unused under `rustnyc`. Check: the tests and mutations
below, `just test` green, `just test-full` once, `reserve` count unchanged (1
ready before, 1 after).

## Criteria

- **AC-103** (a club hosts only with its own role; an unknown club is refused like a missing role): `nyc` is not "unknown", it is the default club.
- **AC-104** (two clubs can run the same question): unchanged; `la` tests stay.
- **AC-105 / D-26** (never twice is per club): `nyc` and `rustnyc` and an absent club are one club.
- **AC-92 / G-10** (used at release): the room writes the used record at release, canonical slug only.
- ADR-0005 *The club slug as id, and `rustnyc` (item 8)*, three musts: alias for at least one release; ledger and bank rewritten in the same deploy; `just sync` before the deploy restarts the room.
- `EVALUATION.md` names no club criterion by number (`rg -n -i club` finds none for AC-103..105); the stories file is the authority (clause 9).

## The tracked bank

`rg -n '"used"|"club"' bank/questions/` finds nothing at `c5c42f7`: q3, q4, q7, q8
hold no used record and no club. The migration has nothing to rewrite in the
tracked bank today; it exists for the records `just sync` writes from the old
room before the deploy, and for any bank copy the client holds outside git.

## Design: one canonicalising parser per language

### Rust (`room/src/models/club.rs`)

- `DEFAULT_CLUB = "rustnyc"`.
- New `pub const CLUB_ALIASES: &[(&str, &str)] = &[("nyc", "rustnyc")]`: old slug → the club it became (ADR-0005 item 8). Doc says: for at least one release; removing it is a later step.
- `ClubSlug::parse(raw)` validates the shape, then maps an alias to its canonical slug. `ClubSlug`'s field is private and its only constructors are `parse` and `default_club` (and it has no `Deserialize`), so **no `ClubSlug` can hold `nyc`**: not `Room.club`, not the OAuth pending map, not the used ledger, not `/admin/used`.
- Every entry point already goes through `ClubSlug::parse` or `default_club()`, which is why one change covers them:
  | Entry point | Code | After |
  |---|---|---|
  | `POST /rooms {club}` (host page *Create a room*, *Run it again*) | `server/routes/host.rs` `club_of` | `nyc` → `rustnyc` |
  | `GET /auth/discord?club=` (sign-in; OAuth state round trip keeps the parsed `ClubSlug` in the pending map, `host/discord/mod.rs` `begin`) | `host.rs` `club_of` | canonical slug stored; the redirect back omits the default club (`club_param`), so the host page comes back as `/host?question=q3` |
  | `GET /last/{club}` | `server/routes/pages.rs` `last_club` | `/last/nyc` = `/last/rustnyc` = `/last` |
  | `PUT /admin/clubs/{club}/questions/{id}` | `server/admin.rs` | `nyc` → default club's schedule |
  | `POPQUIZ_CLUBS` entries | `Clubs::parse` | `nyc=…` and `rustnyc=…` both refused with the existing `Duplicate` error (ruling 3); the error names the slug as the operator typed it |
  | host page `?club=nyc` (`web/host/host.js` passes the raw value through) | no JS change | the server canonicalises; only the comment in `host.js` changes |
- `view.rs` `club_urls` and `host.rs` `club_param` compare with `DEFAULT_CLUB`; canonical, so unchanged.
- The room's never-twice check (`rooms/create.rs` `self.used.contains(club, …)`, `UsedLedger::contains`) compares canonical slugs, so a release under `?club=nyc` refuses a later `?club=rustnyc` create and the reverse.

### Python (`pipeline/src/popquiz/bank.py`)

- `DEFAULT_CLUB = "rustnyc"`; `CLUB_ALIASES = {"nyc": "rustnyc"}`; `canonical_club(slug) -> str`; `club_slug(raw) -> str | None` (shape check with `CLUB_SLUG`, then canonical), the Rust `parse` twin.
- `question_from_dict`: each used record's club is read through `club_slug` (absent → `DEFAULT_CLUB`); the duplicate check runs on canonical clubs, so a record holding both `nyc` and `rustnyc` is refused as *one club appears twice*. `Used.club` is always canonical, so `save_question` / `question_to_dict` write `rustnyc`.
- `Question.used_by(club)` canonicalises its argument.
- `schedule.py`: `--club` canonicalised once after `parse_args` (shape refused as now), so `schedule`, `reserve`, `sync`, `push` (path: `/admin/questions/{id}` for the default), `home_link_for` (`/last` for the default) all see `rustnyc`. `merge_used` reads each ledger entry's club through `club_slug` (absent → default; a non-slug is refused), so **sync from a room still running the old build writes `rustnyc`** (ruling 5).
- `audit.in_reserve` goes through `used_by`, so it follows.

### The twin (ruling 1, 2)

`bank/fixtures/clubs/aliases.json`: `{"default": "rustnyc", "aliases": {"nyc": "rustnyc"}, "slugs": [[raw, canonical-or-null], …]}` with `nyc`, `rustnyc`, `la`, `NYC`, `-la`, `""`, `la/../nyc`, a 25-byte name. Read by `room/tests/twins.rs` (new test: `DEFAULT_CLUB`, `CLUB_ALIASES` as a map, and `ClubSlug::parse` for every slug) and by a new `pipeline/tests/test_club_twin.py` (same against `DEFAULT_CLUB`, `CLUB_ALIASES`, `club_slug`). Each side equals the fixture, so the two tables equal each other.

### The `justfile` check (ruling 4)

The ledger check the boot prompt calls "the schedule recipe's" is in the **`demo` recipe** (`justfile` ~356); `schedule` runs `popquiz.schedule`, which is covered above. The jq filter becomes `any(.[]; (.club == "rustnyc" or .club == "nyc") and .question_id == $q)` and the message says `rustnyc`. `test_club_twin.py` extracts that filter from the `justfile` and runs `jq` on ledgers whose entry is `nyc`, `rustnyc` (found) and `la` (not found), taking the default and every alias from the fixture, so the third copy is pinned too. jq is on the Mac and on GitHub's ubuntu runners; the test fails, not skips, without it.

## The migration (ruling 5)

**Shape: a `migrate-club` subcommand of `popquiz.schedule`** (the CLI that already owns `--bank` and `sync`), plus a `just migrate-club *ARGS` recipe. Not folded into sync: sync needs a room and a token; this must run on a bank alone.

- Default is a dry run: prints `would rewrite <id>: used[<n>].club nyc -> rustnyc` per record, or `nothing to migrate: no used record names an old club slug`. Writes nothing.
- First, every `bank/questions/*.json` must load through `question_from_dict`; any that does not is refused by name and **nothing is written** (refuses a record it does not understand). It also refuses a raw `used` that is neither a list nor an object.
- The rewrite is done on the raw JSON (key order kept, only club values change) and written with `bank._write_json` (the bank's format). Absent clubs are left absent: they already read as the default.
- `--write`: before touching a file, copies it byte-for-byte to `--backup-dir` (default `bank/backups/club-rename-<UTC YYYYMMDDTHHMMSSZ>/`, outside `bank/questions/` so `load_bank` never reads it), refusing if a backup file already exists (never overwrites). Then writes, re-loads, and checks each rewritten record's used clubs equal the canonical clubs it read.
- Idempotent: a second run finds nothing and writes nothing, makes no backup directory.
- Exit 0 on success / nothing to do, 1 on a refusal.

## RUNBOOK release section (ruling 6)

New section *Releasing the club rename (`nyc` → `rustnyc`)* under *Deploy before the rehearsal*:
1. On the PR branch (so sync writes `rustnyc`): `just sync --room https://rustnyc-popquiz.fly.dev` against the room still running the old build.
2. Back up the bank: `cp -R bank ~/popquiz-bank-backup-<date>`.
3. `just migrate-club` (dry run), then `just migrate-club --write`; commit any changed `bank/questions/` record onto the PR branch, so the merge carries code and records together.
4. Merge (a deploy, which restarts the room).
5. `cd pipeline && uv run --offline --no-sync python -m popquiz.schedule reserve` shows the same ready count, now as `reserve (rustnyc)`.
6. An old `?club=nyc` link still opens: `/last/nyc` serves the same page as `/last`, and `/host?question=<id>&club=nyc` signs in and creates a room for the default club.
7. Then `just schedule` (a deploy forgets every scheduled question).
Plus: the alias stays at least one release; removing it is a later step.

## Visible strings (ruling 7) — none in `copy.rs` / `copy.js` / a participant page

The host page, wall, buzzer and `/last` never print a club slug (`rg -n -w nyc room/src/copy.rs web/shared/copy.js` is empty), and printed host links omit the default club, so they are unchanged. Organizer-only CLI lines that change:

| Where | Before | After |
|---|---|---|
| `reserve` / `schedule` / `sync` header | `reserve (nyc): 1 ready …` | `reserve (rustnyc): 1 ready …` |
| `sync` line | `used: q3 by nyc on …` | `used: q3 by rustnyc on …` |
| `schedule` refusal | `q3 was already used by nyc on …` | `… by rustnyc on …` |
| `--club` help | `the club the question is for (default nyc): …` | `the club the question is for (default rustnyc; nyc reads as rustnyc): …` |
| `--home-link` help | `… /last for nyc, …` | `… /last for rustnyc, …` |
| `just demo` refusal | `… used ledger for nyc, …` | `… used ledger for rustnyc, …` |
| room boot error (logs) | `club "nyc" repeats a slug …` | unchanged text; still names what was typed |

No clause-10 halt.

## Docs

`room/README.md` (env table, routes table, clubs section: default `rustnyc`, `nyc` an alias), `fly.toml` comment, `bank/schema/question.schema.json` (the club description: absent reads as `rustnyc`; `nyc` reads as `rustnyc`), `docs/RUNBOOK.md` (reserve snapshot line + release section), `pipeline/README.md` if it names the club, `web/host/host.js` comment, ADR-0005 item 8 `[x]` only. ADR-0001's `nyc` mentions stay (a dated record), named in DONE.

## Tests and their mutations (ruling 11)

| New check | Mutation | Caught by |
|---|---|---|
| `?club=nyc` creates a default-club room; `/auth/discord?club=nyc` redirects; `/last/nyc` = `/last` | alias removed from `CLUB_ALIASES` | `room/tests/clubs.rs` new alias test; canary `/last/nyc` |
| room: release under `nyc`, create under `rustnyc` refused as used (and reverse); `/admin/used` says `rustnyc` | alias removed (nyc then refused as an unknown club, not as used) | `clubs.rs` |
| `POPQUIZ_CLUBS` `nyc=` / `rustnyc=` refused | alias removed (`nyc=` then accepted) | `club.rs` unit + `clubs.rs` |
| sync from an old-build ledger (`club: nyc`) writes `rustnyc` | `merge_used` stops canonicalising | `test_schedule.py` |
| bank: stored `club: nyc` used record reads as used under `rustnyc`; `nyc`+`rustnyc` together refused | loader stops canonicalising | `test_bank.py` |
| `justfile` demo: an `nyc` ledger entry counts for `rustnyc` | filter back to one slug | `test_club_twin.py` |
| twin | alias added to one table only | `twins.rs` / `test_club_twin.py` |
| migration: dry run writes nothing; `--write` backs up; second run no-op; unreadable record refused | backup step removed; rewrite made non-idempotent (e.g. backup dir always created / re-written) | `test_schedule.py` migration tests |

Each mutation is applied by hand, its test seen failing, then reverted; recorded in DONE.

## Commit order

1. `room`: rename the default club to rustnyc with nyc as an alias (club.rs, room tests).
2. `pipeline`: the same in bank/schedule, sync writes rustnyc (bank.py, schedule.py, tests).
3. Twin: shared alias fixture read by both languages, plus the justfile check (fixture, twins.rs, test_club_twin.py, justfile).
4. Migration subcommand + recipe + tests.
5. Docs: RUNBOOK release section, READMEs, schema description, fly.toml, host.js comment, ADR-0005 item 8.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Sonnet review, 2026-10-18: 0 Critical, 3 Major, 6 Minor. All resolved:

- **M1 (gap between sync and merge).** RUNBOOK step 1 says: no room is created or released between the sync and the merge; if one was, run `just sync` again immediately before merging.
- **M2 (reserve baseline).** The "same ready count" check compares against `reserve` printed right after step 1's sync (sync prints it first), not an older number. Ruling 10's before/after in this PR is taken on the tracked bank, with no sync, where the two are the same.
- **M3 (rollback).** RUNBOOK adds: fix forward. A rollback to the old build must first restore `bank/questions/` from the step-2 backup (and re-record any night synced since as `nyc`), because the old pipeline reads `rustnyc` as another club.
- **m1 (Duplicate names the typed slug).** Implemented: `Clubs::parse` keeps the trimmed raw slug for every error it reports, so `nyc=…` reports `"nyc"`.
- **m2 (`Used(club="nyc")` built directly).** `Used.__post_init__` canonicalises `club`, so no `Used` holds `nyc` however it is built.
- **m3 (test harness hard-codes `nyc`).** `room/tests/common/mod.rs` NYC_ONLY checks `rustnyc`; `used.rs`, `admin.rs` expectations become `rustnyc`; some tests keep `nyc` on the way in to prove the alias.
- **m4 (host.js echoes `?club=nyc` into the URL and session storage).** Kept as is, deviation recorded: it echoes the organizer's own link back, the server canonicalises every use, and a JS alias table would be a third copy. Not a value the system stores.
- **m5 (doc comments).** `room/src/server/config.rs:15`, `room/src/models/club.rs:9` and the `club.rs` unit test are updated.
- **m6 (ADR says "rewrite the ledger").** The room's ledger is in memory and dies at the deploy; it is synced, not rewritten. Sync writes canonical, and the migration rewrites the bank. Said in DONE.
