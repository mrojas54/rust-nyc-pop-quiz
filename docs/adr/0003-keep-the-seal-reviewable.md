# ADR-0003: Keep the sealed read path small and the seal's proof whole

**Status:** Accepted (PR #66, draft until merged)
**Date:** 2026-10-12
**Deciders:** Michelle asked for `rooms.rs` split and for `open.rs`; Claude chose where the seal's lines fall; not yet reviewed
**Contract:** G-3, AC-61 (pre-reveal answer isolation), unchanged

## Context

The room never lets a payload carry the join between an option and the verified
output before `reveal`. The proof has two parts: the compiler (field privacy and
`compile_fail` doctests) and `tests/boundary.rs`, which reads the source and
refuses the ways around it. Splitting `rooms.rs` and the view code could weaken
either part, so each split had to say where the seal's lines fall.

## Decision

1. **`rooms/open.rs` holds the one way to the answers**: `Room::open`,
   `Room::public`, `Opened` and `PublicView`, 62 lines. It sits beside
   `create.rs` (the one way in): one door in, one door to the answers.
2. **The boundary rule got tighter, not looser.** "Only one file may call
   `.question.open(`" now names `rooms/open.rs`. Before, it named `rooms.rs`,
   which also held the app state. "A machine is assigned only in one file" now
   names `rooms/room.rs`, so `AppState` (now `state.rs`) is held to it too.
3. **Privacy is unchanged in kind.** The `Room` and `AppState` fields that
   siblings read are `pub(super)`: visible inside the `rooms` module and nowhere
   else, as before the split. The `compile_fail` doctests stay in `rooms/mod.rs`
   and still run from outside the crate.
4. **`models/view.rs` is not split.** `tests/boundary.rs` reads `mod sealed` (the
   pre-reveal builders for wall, buzzer and host) as one block and refuses any
   mention of the opened room in it. Splitting by viewer would scatter that block
   across files. The file's header now says where each part is.

## Options Considered

### Option A: `open.rs` as its own file; `view.rs` whole (chosen)
**Pros:** the seal's two reviewable pieces are one 62-line file and one file read
as a block; the scan is stricter.
**Cons:** `view.rs` stays at 677 lines.

### Option B: Leave `open` in `room.rs`
Fewer files. **Rejected:** `room.rs` also writes the machine; the scan could not
tell the two jobs apart.

### Option C: Split `view.rs` by viewer
Smaller files. **Rejected** for now: it changes how the proof is read, and the
proof is the one place a rewrite is not worth the risk. Revisit only with a
rewritten scan.

## Consequences

**Benefits:** a reviewer checking G-3 reads `open.rs` and `view.rs`'s `sealed`
module, not 1,266 lines. A new file in `rooms/` that opens a question, or
assigns a machine, fails a test instead of passing review by luck.

**Imports are explicit.** Each `rooms/` file lists its own `use` lines. `rooms/mod.rs`
holds the module list, the `pub use` re-exports, the shared `Totals` type and
two constants, with no imports for its children. A boundary test
(`within_rooms_only_open_rs_names_the_sealed_answer_types`) fails if any other
`rooms/` file names `Revealed` or `Middle`.

**Harder / accepted:** `pub(super)` is the price of a sibling file. A future
file in `rooms/` can read a `Room` field; the scans, not the compiler, stop it
from writing the machine or opening the vault.

## Action Items

1. [x] `open.rs`, tightened scans, doctests kept (PR #66)
2. [ ] Revisit the `view.rs` split if the seal scan moves to a folder read

## Where it lives

`room/src/rooms/open.rs` · `room/src/rooms/mod.rs` (doctests) ·
`room/src/models/view.rs` · `room/tests/boundary.rs`
