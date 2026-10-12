# PQ-71: Live-room refactor: split the Discord backend and extract participant operations

Behaviour-preserving refactor of room/src per the client's plan and proposed MADR (attached; copies in .lattice/orchestration/plans/). Ordered as the plan orders it:
1. Baseline from origin/main (4946538 at minting).
2. Discord separation: host/discord.rs keeps OAuth, organizer sessions, refresh coordination, authorization and retry orchestration; private host/discord/config.rs (settings, secrets, retry policy, logging types; public paths kept by re-export) and private host/discord/http.rs (transport, encoding, parsing and their tests; pub(super) only where the parent needs it).
3. Participant extraction: private rooms/participants.rs takes AppState::join, answer, leave, buzzer_for, session_count, set_live_counts and the SessionTokens impl (state.rs:312-486 on main). pub use state::AppState and every signature unchanged; no new public fields, forwarding methods or service layer.
4. Interfaces and semantics preserved: paths, signatures, payloads, errors and error order, lock scopes and ordering, async/sync boundaries; no new dependencies.
5. Navigation docs: responsibility map and operation guide in existing module docs; organizer session vs room-host session; the six invariant comments beside their code.
6. Validate from room/: cargo fmt --check; cargo test --offline --locked; cargo test --offline --locked --test lifecycle -- --ignored test_full_ (AC-69 with auth down); cargo check --offline --locked --all-targets --all-features; git diff --check.
7. One refactoring-specialist review (.claude/agents/refactoring-specialist.md) plus at most one re-review; blockers after that stop for direction.
Out of scope: burst-report, frozen spike code, actor conversion, persistence, service/repository layers.
Acceptance: the plan's seven criteria; the diff holds only this refactor and its docs.
Orchestrator notes: (a) the plan's checkpoint draft (Discord split + a premature burst-report split) is in no reachable worktree or branch, so step 2 is written fresh from main and nothing burst-report is touched; (b) state.rs is 486 lines after #77's rustfmt, not 454; (c) ADR convention is docs/adr/NNNN-slug.md, next free 0005; ADR-0002 (group the room source by who uses it) is the related prior decision; the MADR is promoted only when the client accepts it; (d) touches no file PQ-23 touches; land before the held room tickets (PQ-47, PQ-55, PQ-59, PQ-60, PQ-63).

# Plan (delegator, 2026-10-12)

Base: `origin/main` @ 4673e12 (moved from 4946538 by PR #78: `AGENTS.md`/`CLAUDE.md` only, nothing under `room/`). Branch rebased onto it before implementation.

## Module-file convention (ruling 2)

Every directory module in `room/src` uses `mod.rs` (`connections/`, `host/`, `models/`, `rooms/`, `server/`, `server/routes/`); none uses the 2018 `foo.rs` + `foo/` layout. Following the crate: **`host/discord.rs` is renamed `host/discord/mod.rs`** (pure rename commit, 100 % similarity), then split.

## host/discord/mod.rs (stays): OAuth, organizer records & sessions, refresh coordination, authorization, retry orchestration

- Module doc (unchanged text, plus a short "Where things are" pointer to `config`/`http`).
- `mod config; mod http;` (private) and `pub use config::{Endpoints, Log, Retry, Secret, Settings, Stderr};` — every public type keeps `room::discord::X` / `room::host::discord::X`.
- `use http::{encode, form, Client, Req, Resp};`
- Constants `SCOPES`, `CALLBACK_PATH`, `STATE_COOKIE`, `SESSION_LIFETIME`, `SIGN_IN_WINDOW` (pub, unchanged), `REFRESH_MARGIN`, `MAX_PENDING`, `MAX_SESSIONS` (private).
- Records: `Organizer`, `Session`, `Pending`, `Store`; `lock`, `random_hex`, `pub fn is_question_id`.
- `Discord`, `Inner`, `Finish`, `impl Discord {serving, new, begin, finish, organizer_count}`, `impl HostAuth for Discord`.
- Payload shapes `Member` (pinned by the AC-65 scan), `Tokens`, `User`, `OAuthError`, and `Outage`.
- `impl Inner {token_url, session_user, refresh_lock, forget, access, check, refresh, call}` — `call` is retry orchestration and stays here.
- Test `question_ids` stays (it tests `is_question_id`).

## host/discord/config.rs (new, private): settings, secrets, retry policy, logging types

- `pub struct Secret` + `impl Secret { pub fn new; pub(super) fn expose; pub(super) fn matches }` — `expose`/`matches` were private; the move forces `pub(super)` (parent: `client_secret.expose()`, token reads, `matches` in `finish`/`session_user`; `http` is a child of `discord`, so it sees `pub(super)` too). Not visible outside `host::discord`.
- `pub struct Settings`, `pub struct Endpoints` + `impl {discord, at}`, `pub struct Retry` + `impl Default`, `pub trait Log`, `pub struct Stderr` + `impl Log`.
- Imports: `std::time::Duration`, `subtle::ConstantTimeEq`.
- No locks, no tests (none exist for these types).

## host/discord/http.rs (new, private): HTTP/TLS transport, encoding, response parsing

- `const MAX_BODY` (private).
- `pub(super) fn encode`, `pub(super) fn form` (encoding).
- `pub(super) struct Req` (fields private) + `impl Req { pub(super) fn get; pub(super) fn form }`.
- `pub(super) struct Resp { pub(super) status, retry_after, body }` + `impl Resp { pub(super) fn json; pub(super) fn retry_after }`.
- `trait Io` + blanket impl (private), `pub(super) struct Client` (field private) + `impl Client { pub(super) fn new; pub(super) async fn send }`, `async fn read_chunked` (private).
- Tests moved whole: `encode_leaves_unreserved_alone`, `retry_after_reads_the_header_then_the_body`, `chunked_bodies`.
- Imports: `std::sync::Arc`, `std::time::Duration`, `serde::Deserialize`, `tokio::io::{…}`, `super::Secret`.
- Every visibility widening is private→`pub(super)` and forced by the parent's use; nothing wider.

"Response parsing" is read as the HTTP framing plus the generic `Resp::json` / `Resp::retry_after` decoders. The Discord payload shapes (`Member`, `Tokens`, `User`, `OAuthError`) stay with the authorization that interprets them: `Member` is AC-65's subject and the scan pins it beside the check.

## rooms/participants.rs (new, private `mod participants;` in rooms/mod.rs)

- `impl AppState { set_live_counts, join, answer, leave, buzzer_for, session_count }` — bodies, signatures and doc comments identical, original order; the T-04b section comment moves with them.
- `impl crate::ws::SessionTokens for AppState { resolve, saved, gone }` with its PQ-32 section comment.
- Imports: `crate::phase::Phase`, `super::state::{lock, AppState}`, `super::{AnswerError, Joined, LiveCounts, RoomError}`.
- Uses only existing `pub(super)` access: `AppState.{rooms, ended, capacity}`, `Entry.{room, sessions}`, `Tombstone.{code, ended}`, `state::lock`. No visibility change, no new field, no forwarding method.
- state.rs keeps: `Entry`, `SessionFactory`, `AppState` + fields, `Tombstone`, `lock`, setup/builders, clock/ledger/take-home/clubs/room_count/ended_room, `sweep`, `schedule`/`schedule_for`, `act` (release side effects), `record_fit`, `with_room`, `with_hosted_room`. Drops `Phase`, `AnswerError`, `Joined`, `LiveCounts` from its imports (486 → ~310 lines).

## Lock sites per moved function (before = after; no lock moves, none added)

| Function | Sites, in order |
|---|---|
| `set_live_counts` | `lock(rooms)` |
| `join` | `lock(rooms)`; on unknown code, nested `lock(ended)` (rooms → ended, as `sweep` and `create_for`) |
| `answer` / `leave` / `buzzer_for` / `session_count` | `lock(rooms)` |
| `SessionTokens::resolve` / `saved` | `lock(rooms)`; `gone` none |
| Discord: all `lock(store)`, `lock(refreshing)`, `refresh.lock().await` | stay in `discord/mod.rs`; `config.rs`/`http.rs` take no locks |

## Source-scan tests (ruling 7)

- `tests/auth.rs::ac65_the_member_record_is_read_for_roles_alone` reads `room/src/host/discord.rs`. After the rename that path is gone; the path becomes **every `.rs` file under `room/src/host/discord/`** (walked, ADR-0004), so the forbidden-words half keeps covering all Discord code after the split rather than shrinking to the parent. The `struct Member` assertion is unchanged.
- `tests/used.rs` (append only in `rooms/state.rs`), `tests/boundary.rs` (walk `rooms/`): unaffected — `participants.rs` holds no `.append(`, `UsedRecord`, `Revealed`, `Middle`, `.vault`, `machine =`.

## Docs (ruling 5)

- `rooms/mod.rs`: responsibility map of `rooms/` files incl. `participants.rs` + the operation guide table.
- `host/mod.rs`: organizer session vs room-host session; the discord/{mod,config,http} map.
- `host/auth.rs`: credential-documentation clarification (doc only).
- Invariant comments beside code: (1) participants.rs section comment (sessions + counts under one `rooms` lock); (2) `act` release side effects atomic; (3) `schedule_for`/`create_for` lock order (already stated; add the invariant tag in `schedule_for`'s doc); (4) `act`/`with_hosted_room`: host check before ended-state disclosure; (5) `authorize_create`/`check`: refresh serialized, own task survives cancellation; (6) `Secret` / `Discord` tokens: credential boundary, and `participants.rs` buzzer paths: pre-reveal isolation via `view::buzzer_for`.
- `room/README.md` *Source layout*: `host/discord/` and `rooms/participants.rs`; the "tokens never leave `discord.rs`" sentence → the `discord` module. `server/routes/host.rs:106` comment likewise. `lib.rs` module doc: discord line mentions the split.

## ADR

`docs/adr/0005-separate-live-room-responsibilities.md`, house format, Status Proposed, refines ADR-0002, notes the ADR-0004 scan path edit, supersedes nothing.

## Commit order

1. Rename `host/discord.rs` → `host/discord/mod.rs` + AC-65 scan path (one commit so every commit is green).
2. Discord config → `config.rs`.
3. Discord transport → `http.rs` (with its tests).
4. Participants → `rooms/participants.rs`.
5. Module docs, invariant comments, README map.
6. ADR-0005 (Proposed).

## Tensions with the client's plan

- Plan names `host/discord.rs` + `discord/{config,http}.rs`; ruling 2 says follow the crate's convention, which is `mod.rs`. Taking the convention: `host/discord/mod.rs`. Not cleared by name; it is the cleared `discord.rs` renamed.
- Plan step 3 expects ~300 lines; cohesion is the criterion (ruling 3).
- Stale `discord.rs` mentions in files outside the cleared list (`room/Cargo.toml` comments, `connections/requestlog.rs` doc, `docs/adr/0001`, `docs/audit/**`) are left and listed as deferred.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: rust-specialist (sonnet), 2026-10-12. 0 Critical, 2 Major, 9 Minor.

1. **Major — AC-65 scan must be a strict superset.** Accepted. `tests/auth.rs::ac65_…` walks every `.rs` under `room/src/host/discord/`, and per file: strips `//` lines; locates `struct Member {` within that file only (closing-`}` search cannot cross a file boundary); asserts exactly one file defines it and its fields are exactly `["roles: Vec<String>,"]`; applies the forbidden-word check to every file; asserts the walk found `mod.rs`, `config.rs` and `http.rs` (in the rename commit, `mod.rs` only; the list grows with each split commit); message names the file. The assertion set is the old one plus two.
2. **Major — intra-doc links in moved docs.** Accepted. `participants.rs` imports `AppState` (links `AppState::leave` resolve); `cargo doc --offline --no-deps` with `RUSTDOCFLAGS=-D warnings` is run before each docs/ADR commit and at the exit check, compared with main's baseline.
3. Minor — `Secret::{expose, matches}` private→`pub(super)`: as planned.
4. Minor — `Resp`: only `status` is read by the parent, so only `status` is `pub(super)`; `retry_after`/`body` stay private (the test constructing `Resp` lives in `http`). Overrides the earlier line giving all three fields `pub(super)`.
5. Minor — `Req` fields stay private: as planned.
6. Minor — moved `http` tests use `use super::*;` plus the imports they need (`BufReader`, `Duration` reach them through `super::*` since `http.rs` imports both).
7. Minor — `question_ids` stays in `mod.rs`: as planned.
8. Minor — unused imports in `mod.rs` after the split (`tokio::io`, `MAX_BODY`, `subtle`) are dropped in the commit that moves their last user.
9. Minor — `join` keeps both guards live in the original order (rooms → ended): moves are byte-identical.
10. Minor — `state.rs` drops `Joined`, `AnswerError`, `LiveCounts`, `Phase` imports after the move: confirmed by compile with `-D warnings`.
11. Minor — stale `discord.rs` mentions: fix `README.md:77,144` and `server/routes/host.rs:106` (cleared); defer `Cargo.toml:115,119` and `connections/requestlog.rs:8` (not cleared).

## Reset 2026-10-12 by agent:delegator-pq71
