# Live-room hosting and management refactor plan

Status: Proposed; implementation paused pending continuation.

Related decision: [Proposed MADR: live-room module boundaries and state ownership](live-room-module-boundaries-madr.md).

## Goal

Make live-room hosting and management easy to navigate by separating cohesive
responsibilities while preserving behavior, public APIs, and the existing
concurrency model. This plan incorporates the refactoring specialist's review.

## Scope and module responsibilities

All paths below are relative to `room/src/`.

| Module | Responsibility | Planned change |
|---|---|---|
| `host/auth.rs` | Permission to create or control a room | Clarify credential documentation |
| `host/discord.rs` | OAuth, organizer sessions, refresh coordination, authorization, retry orchestration | Keep backend logic together |
| `host/discord/config.rs` | Settings, secrets, retry policy, logging types | Finish existing extraction |
| `host/discord/http.rs` | HTTP/TLS transport, encoding, response parsing | Finish existing extraction |
| `rooms/state.rs` | State ownership, scheduling, host commands, release, projections, expiry | Extract participant operations |
| `rooms/participants.rs` | Join, answer, leave, participant views/counts, socket session lookup | Add private module |
| `rooms/create.rs` | Creation and run-again | Preserve existing boundary |
| `rooms/room.rs` | One room's state and transitions | Preserve existing boundary |
| `rooms/lifecycle.rs` | Expiry policy and clock abstraction | Preserve existing boundary |
| `server/routes/host.rs` | HTTP parsing, delegation, response mapping | Preserve existing boundary |

File size is a signal, not a target. Burst-report cleanup, frozen spike code,
actor conversion, persistence changes, and new service/repository layers are
outside this refactor.

## Implementation sequence

### 1. Establish the scoped baseline

- Inspect the working tree before editing and preserve unrelated/user changes.
- Remove only the assistant's premature burst-report extraction.
- Review the existing Discord extraction as part of this work.
- Locate the repository's architecture-decision convention before assigning a
  permanent identifier or moving the proposed MADR into an accepted ADR location.

### 2. Finish the Discord separation

- Keep OAuth, organizer records, sessions, refresh coordination, authorization,
  and retry orchestration together in `host/discord.rs`.
- Keep public configuration types in private `discord/config.rs` and preserve
  their existing public paths through re-exports.
- Keep transport, encoding, parsing, and their existing tests in private
  `discord/http.rs`.
- Limit transport types and operations needed by the parent to `pub(super)`;
  keep other implementation details private.

### 3. Extract participant handling as one responsibility

Add private `rooms/participants.rs` and move these existing operations into it:

- `AppState::join`, `answer`, and `leave`.
- `AppState::buzzer_for`, `session_count`, and `set_live_counts`.
- The `SessionTokens` implementation for `AppState`.

Declare `mod participants;` in `rooms/mod.rs`. Preserve `pub use state::AppState`
and every existing method signature. Reuse existing `pub(super)` state access;
introduce no new public fields, forwarding methods, or service abstraction.

Keep state ownership/setup, scheduling, host commands, release side effects,
projections, and sweeping in `state.rs`. The extraction should reduce it from
454 to roughly 300 lines, but cohesion is the acceptance criterion.

### 4. Preserve interfaces and execution semantics

- Preserve public paths, method signatures, response formats, and errors.
- Preserve existing lock scopes and ordering.
- Keep async Discord network operations and synchronous room mutations at their
  existing boundaries.
- Add no dependencies solely to facilitate the split.
- Use explicit imports and narrow internal visibility.

### 5. Improve navigation and explanations

Add a short responsibility map and operation guide to existing module docs:

| Operation | Main path |
|---|---|
| Sign in | Host route → Discord backend |
| Create room | Host route → organizer authorization → room creation |
| Host command | Host route → host-session check → room transition |
| Join or answer | Participant route → participant operation → session/count update |
| Expire room | Reaper → sweep → removal and tombstone |

Explain the distinction between an organizer session, which authorizes room
creation, and a room-host session, which controls an existing room.

Keep invariant comments beside the relevant code:

- Participant session changes and live counts update under the same room lock.
- Release, session clearing, used-record append, and take-home updates remain
  atomic.
- Creation and scheduling retain lock ordering.
- Host authorization precedes disclosure of ended-room state.
- Discord refresh remains serialized and survives caller cancellation.
- Credential boundaries and pre-reveal answer isolation remain intact.

### 6. Validate behavior and compatibility

From `room/`, run:

```sh
cargo test --offline --locked
cargo test --offline --locked --test lifecycle -- --ignored test_full_
cargo check --offline --locked --all-targets --all-features
```

The normal suite covers authorization, sessions, lifecycle, release/used records,
club isolation, transport, and compile-fail boundaries. The additional lifecycle
scenario checks operation through release while Discord is unavailable.

Review the exact diff for changed signatures, payloads, error ordering, lock
scopes, visibility, and state transitions. Check whitespace with `git diff
--check`. Use existing behavioral tests; add tests only for meaningful coverage
gaps, not to mirror file layout or moved implementation.

### 7. Complete bounded specialist review

Use one refactoring-specialist review of the completed scope and one re-review
after fixes if needed. If blockers remain after the second pass, report them
and stop for direction. Update the MADR status when the decision is accepted.

## Acceptance criteria

- New modules have cohesive responsibilities and remain private.
- Existing public paths and method signatures remain usable.
- No internals become public merely to enable the split.
- Critical authorization and concurrency invariants are preserved.
- Required checks pass; any unverified behavior is explicitly reported.
- Documentation lets a reader locate and trace the main operations.
- The final diff contains only the live-room refactor and its documentation.

## Current checkpoint

At the time this plan was saved:

- A draft Discord split into configuration and HTTP transport is in the working
  tree.
- The premature burst-report split is also still present and must be removed
  from this scope without disturbing other changes.
- The normal room suite passed against that earlier combined working tree.
  This does not verify the future participant extraction.
- Participant extraction and final specialist implementation review have not
  happened.
- These documents are proposed planning artifacts; saving them does not mark
  implementation or architectural acceptance complete.
