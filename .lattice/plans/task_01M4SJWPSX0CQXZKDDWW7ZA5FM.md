# PQ-74: ADR-0005 item 6: move room files to the Target layout

docs/adr/0005-separate-live-room-responsibilities.md, Action Item 6 and the 'Target layout' section (the one source for the layout). Behaviour-preserving moves only.

# Plan (delegator, 2026-10-30)

**Outcome.** The room's source sits in the ADR's tree under `room/src/`. Nothing changes for anyone on the night. Base `9c718c3`.
**Acceptance check.** `just test`, `just test-full`, `cargo doc` no new warnings; the moved-bodies proof script prints only the allowed differences; every path scan shown failing on a planted violation.

## 1. Source → target (every file in `room/src/`)

| Source | Target | Note |
|---|---|---|
| `lib.rs` | `lib.rs` | module decls, re-exports, the `rooms` path module |
| `copy.rs` | `copy.rs` | unchanged |
| `cli/*` (7 files) | `cli/*` | not moved; ADR: "`cli/` is outside this ADR". One comment path updated in `room_client.rs:63` |
| `connections/{mod,requestlog,sessions,ws}.rs` | same | not moved |
| `server/{main,mod,admin,config,reaper}.rs`, `server/routes/{mod,connections,host,pages,pipeline,wall}.rs` | same | not moved; one comment path in `routes/connections.rs:119` |
| `rooms/mod.rs` | `quiz_session/mod.rs` | map text updated, doctests byte-identical |
| `rooms/state.rs` | `quiz_session/state_manager.rs` | |
| `rooms/open.rs`, `seams.rs`, `errors.rs`, `lifecycle.rs` | `quiz_session/` same names | |
| `rooms/room.rs` | `quiz_session/models/rooms.rs` | `Room`, also receives `Totals` |
| `rooms/create.rs` | `quiz_session/host/create.rs` | `random_bytes`, `hex` move up to `quiz_session/mod.rs` |
| `rooms/participants.rs` | `quiz_session/participant/{mod,join,vote,leave,buzzer,counts,tokens}.rs` | split below |
| `models/{mod,answers,club,phase,question,used,view}.rs` | `quiz_session/models/` same names | |
| `models/affirmation.rs` | `quiz_session/models/affirmation.rs` | **ADR does not list it** (added by item 9 after the ADR was written). Ruling 7: smallest placement, beside its siblings. Flag in DONE. |
| `host/{mod,auth}.rs`, `host/discord/{mod,config,http}.rs` | `quiz_session/host/` same names | |

### The participant split

| Function | File |
|---|---|
| `join` | `participant/join.rs` |
| `answer` | `participant/vote.rs` |
| `leave` | `participant/leave.rs` |
| `buzzer_for` | `participant/buzzer.rs` |
| `set_live_counts`, `session_count` | `participant/counts.rs` |
| `impl ws::SessionTokens for AppState` | `participant/tokens.rs` |

The module doc and the `T-04b` invariant comment (one `rooms` lock per session change) go to `participant/mod.rs`, whole. Each function has its own one `rooms` lock today; `join` takes `ended` under `rooms` and that stays in the one function. No lock order forbids this split.

## 2. Module tree and the public surface

```rust
// lib.rs
mod quiz_session;              // private: adds no new public path
pub use quiz_session::{host, lifecycle, models};
pub use host::{auth, discord};
pub use models::{affirmation, answers, club, phase, question, used, view};
pub mod rooms {                // the old public path, explicit
    pub use crate::quiz_session::{lifecycle, new_code, AnswerError, Acted, AppState, CloseSnapshot,
        Created, Fit, Joined, LiveCounts, NoSessions, Opened, PublicView, Room, RoomError, Scheduling,
        Sessions, Totals, Urls, CODE_ALPHABET, ROOM_LIFETIME};
}
```

- `room::rooms::*` is the exact `pub` set of today's `rooms/mod.rs`, listed by name rather than `pub use quiz_session as rooms`, so `room::rooms::host` and `room::rooms::models` do not appear. `room::host`, `room::models` and the crate-root paths stay. **No public path added, none lost.**
- `crate::rooms::…` inside the crate keeps resolving through that module, so bodies and most imports do not change. Those imports stay as they are (moves are moves).
- Doctests keep `room::rooms::Room`, byte-identical.

## 3. Every `pub(super)` and what it becomes

| Today | Items | After |
|---|---|---|
| `rooms/room.rs` | `Frozen` + 3 fields, `Parting` + 3 fields, `Room`'s 15 fields, `take_parting` (the ADR's "26") | `pub(in crate::quiz_session)` |
| `rooms/state.rs` | `Entry` + 2, `SessionFactory`, `AppState`'s 12 fields, `Tombstone` + 3, `lock` | unchanged spelling: `pub(super)` from `quiz_session/state_manager.rs` reaches `quiz_session` and its children |
| `rooms/create.rs` | `random_bytes`, `hex` | move to `quiz_session/mod.rs` as **private** `fn` (visible to its descendants; `pub(super)` there would reach the crate root, which widens) |
| `host/discord/{config,http}.rs` | `expose`, `matches`, transport items | unchanged: their parent is still `discord` |
| `models/view.rs` (`mod sealed`, `mod opened`), `models/answers.rs` (`vault::seal`) | | unchanged: inner modules |
| `server/routes/*` | | not moved |

New module-visibility lines, each no wider than needed:
- `host/mod.rs`: `pub(super) mod create;` (private path today; `quiz_session/mod.rs` re-exports `new_code`, `Created`).
- `models/mod.rs`: `pub(in crate::quiz_session) mod rooms;` (`Room` keeps its path through re-exports; `room::models::rooms` is not a new public path).
- `quiz_session/mod.rs`: `mod participant;`, `mod state_manager;`, `mod open;` etc. stay private; `pub mod host; pub mod models; pub mod lifecycle;`.
- `Totals` (a type alias) moves to `models/rooms.rs`: the ADR says `quiz_session/mod.rs` owns no type. `ROOM_LIFETIME` and `CODE_ALPHABET` are constants and stay in `quiz_session/mod.rs`.

### Contract note: "the same modules as today" is not literally true

ADR-0005 says `pub(in crate::quiz_session)` is "the same modules as today, named instead of implied", and ruling 3 says visibility does not widen. The spelling does what the ruling names, but `models/*` and `host/*` were **outside** `rooms/` and are **inside** `quiz_session/` under the decided tree. So `models/view.rs`, `models/used.rs`, `host/auth.rs` and `host/discord/*` gain the *ability* to name `Room`'s and `AppState`'s internals. Nothing outside the crate gains anything, and `connections/`, `server/`, `cli/` gain nothing. Michelle decided the tree; no spelling of this tree avoids it (`pub(in ...)` must name an ancestor, and the sibling sets are not expressible). **Decision: follow the ruling and the ADR, make no code that uses the reach, prove with a grep that no moved file outside the old `rooms/` set reads those fields, and flag in DONE.** The seal is unaffected: reading the vault still needs a `RevealWitness` (the `compile_fail` doctests and `tests/boundary.rs`).

## 4. Forced edits inside moved bodies (the complete allowed list)

Beyond `use` lines, `mod` lines and the visibility above:

1. `models/rooms.rs` body line: `super::lifecycle::verdict(` → `crate::quiz_session::lifecycle::verdict(` (`ended`).
2. Doc links that use `super::`: `room.rs:322` `[`super::AppState::act`]` → `[`crate::rooms::AppState::act`]`. `seams.rs` and `errors.rs` stay one level under `quiz_session`, unchanged.
3. Comments/docs that name paths (`rooms/create`, `rooms/room.rs`, `rooms/state.rs`, `host/discord/`, the file tables in `rooms/mod.rs`, `lib.rs` and `room.rs:67-68`'s `pub(super)`).

## 5. Every string or scan naming a source path

**Scans (path updates only):**

| File | Today | After |
|---|---|---|
| `tests/boundary.rs:92,94,101` | `models/phase.rs` | `quiz_session/models/phase.rs` |
| `:108,176-178,218` | `models/answers.rs` | `quiz_session/models/answers.rs` |
| `:179,180,226,231` | `rooms/room.rs` | `quiz_session/models/rooms.rs` |
| `:181,222,283,288,294` | `rooms/open.rs` | `quiz_session/open.rs` |
| `:244` | `models/view.rs` | `quiz_session/models/view.rs` |
| `:275` | `starts_with("rooms/")` | **see below** |
| `:302` | `rooms/lifecycle.rs` | `quiz_session/lifecycle.rs` |
| `tests/used.rs:168,179` | `rooms/state.rs` | `quiz_session/state_manager.rs` |
| `:180` | `rooms/room.rs` | `quiz_session/models/rooms.rs` |
| `:193` | `starts_with("rooms/")` | `starts_with("quiz_session/")`: strictly wider |
| `:204` | `models/used.rs` | `quiz_session/models/used.rs` |
| `tests/one_room.rs:270` | `room/src/rooms/create.rs` | `room/src/quiz_session/host/create.rs` (a missing file is a panic, so it cannot go quiet) |
| `tests/auth.rs:277,308` | `host/discord/` | `quiz_session/host/discord/`; keeps its "requires mod, config, http" assertion |
| `tests/common/mod.rs` | walks `room/src` | unchanged: `server/routes/` did not move, the walk is whole-tree |

**`boundary.rs:275`, `within_rooms_only_open_rs_names_the_sealed_answer_types`.** `quiz_session/` alone would take in `models/answers.rs` (which defines them) and fail for a reason that is no rule. The rule's subject is "the room's files". New filter: `name.starts_with("quiz_session/") && (!name.starts_with("quiz_session/models/") || name == "quiz_session/models/rooms.rs")`. That covers every file the old filter covered, plus `host/auth.rs` and `host/discord/*`, which name neither today (checked by rg). Wider, never weaker. It still asserts `open.rs` names them.

**Walk scans with no path literal (no edit needed):** `tests/admin.rs`, `no_fallback_route.rs`, `lifecycle.rs:594`, `twins.rs:458`, `canary_scan/mod.rs`. `room_sources` asserts > 10 files found.

**`tests/lifecycle.rs:594` note.** It splits a line at the first `:` and strips only `pub ` and `pub(crate) `. It already misses `pub(super) x:` fields (head stays `pub(super) x`); `pub(in crate::quiz_session) x:` is missed the same way (head `pub(in crate`). Neither weaker nor stronger on the fields that move: a pre-existing gap. Ruling 5 allows path updates only. Report in DONE, do not fix here.

**Doc comments in cleared files:** `tests/boundary.rs:3-4`, `tests/used.rs:165,178`, `room/README.md:60-101,205,254,351-352,570,768-769`, `lib.rs:17,30`, `models/used.rs:80`, `connections/requestlog.rs:8`, `cli/room_client.rs:63`, `server/routes/connections.rs:119`, `pipeline/src/popquiz/{schedule,bank}.py`, `pipeline/tests/test_affirmation_twin.py:5`, `web/test/{host,copy-freeze}.test.js`, `CLAUDE.md:151` + `AGENTS.md` (`room/src/used.rs`, already stale). `Dockerfile`/`.dockerignore` name `room/src` only: still right. `justfile`, CI: no source path below `room/src`.

**Not cleared, left stale and named in DONE:** `room/Cargo.toml:120,123` comments (no bin path needs it), `web/shared/phase.js:6` comment, `docs/adr/0001…0004`, `sequence/**`, `SPEC/BUILDPLAN/PRODUCT`. ADR-0005's own "Where it lives" stays per ruling 8.

## 6. `cargo doc` baseline

`cargo doc --offline --locked --no-deps` at `9c718c3`: rc 0, **0 warnings** (`$TMPDIR/pq74-doc-base.log`). After: 0 warnings, or the new ones are a failure.

## 7. How the PR proves moves are moves

A rerunnable Python script in the PR body, with no dependency, comparing `git show 9c718c3:<old>` to `HEAD:<new>` across all of `room/src` excluding `cli/`:

1. **Line multiset:** every line at base vs head, after removing `use`/`mod`/`pub use` lines and normalizing `pub(super)`/`pub(in crate::quiz_session)`. Prints the lines only on one side; the PR quotes the output, and each remaining line is a doc-path line or the forced edit of §4.
2. **Lock sequence per function:** `lock(&self.x)` sequence by `fn` name; base and head must be equal.
3. **Tests moved:** `#[test]` names per tree are equal; `room/tests` diff limited to path strings by `git diff --stat` and a printed diff.

Plus `git log --follow` works per file: each commit is a pure `git mv` + forced lines, similarity ≥ 90% except the split.

## 8. Commit order (each compiles; tests updated with its move)

1. `rooms/` → `quiz_session/`, `state.rs` → `state_manager.rs`, `pub mod rooms` path module in `lib.rs`; scans.
2. `models/` → `quiz_session/models/`; scans.
3. `room.rs` → `models/rooms.rs`; visibility; `random_bytes`, `hex` up; `Totals` to it; scans.
4. `host/` → `quiz_session/host/`; `create.rs` → `host/create.rs`; scans.
5. `participants.rs` → `participant/*`.
6. Doc maps and path strings (README, module docs, pipeline, web/test, CLAUDE.md + AGENTS.md).
7. ADR-0005: tick item 6.

## 9. Scan-detection proof (ruling 5)

For each of auth AC-65, one_room create_for, used writers, boundary within-rooms, boundary witness, boundary machine assignment, twins copy, common walk: plant a violation in the moved file, run that one test, watch it fail, `git checkout` it. Each run bounded under 15 minutes (background job). Reported as "plant → test that caught it".

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Sonnet subagent, read-only, no build. No Critical; one Major; Minors below. Its compile claims are read-through only; `cargo check` at each commit is the proof.

1. **Major, accepted.** The inline `pub mod rooms` in `lib.rs` gets a `///` doc comment. §6's gate still decides.
2. **Minor, accepted.** The `boundary.rs:275` filter is a logic edit, not a path update, so §7.3's "path strings only" is wrong for that one line. The proof prints it as the one named exception; it covers everything the old filter covered, and more.
3. **Minor, accepted.** `used.rs:190`'s comment ("the whole `rooms` module") is reworded with its `starts_with` edit.
4. **Minor, accepted, and replaced with a stronger proof.** §3's grep is dropped. Every `models/*`, `host/auth.rs` and `host/discord/*` body is byte-identical to base apart from `use` lines (the §7 script shows it), and those bodies compiled at base with no access to `Room`'s or `AppState`'s internals. So none of them uses the new reach.
5. **Minor, accepted.** §3's contract note also covers `random_bytes` and `hex`: private in `quiz_session/mod.rs`, they are nameable from `models/`, `host/` and `participant/`.
6. **Minor, no change.** `tests/lifecycle.rs:594` is a pre-existing gap; reported in DONE.
7. **Minor, accepted.** §5's doc list is not authoritative: commit 6 ends with a fresh `rg` sweep over the cleared files, and whatever it finds is fixed there. `web/shared/README.md:125` names `room/src/copy.rs`, which does not move.
8. **Minor, accepted.** Doctests: the count at head must equal base, and each `compile_fail` keeps its compiling twin.
9. **Minor, no change.** ADR-0005's stale lines stay under ruling 8.
