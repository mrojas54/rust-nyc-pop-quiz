# ADR-0004: Source-scanning guardrails walk the tree and read concerns, not file names

**Status:** Accepted (PR #66, draft until merged)
**Date:** 2026-10-12
**Deciders:** Claude, while moving the files for Michelle's request; not yet reviewed
**Contract:** G-3, G-9, G-10, AC-55, AC-57, AC-101, D-24 (all unchanged)

## Context

About ten test files read `room/src` as text to hold a rule: the seal, the one
create check, the used-ledger's single writer, the admin token's single reader,
no fallback route, no participant identity in any record, the copy twin. They
were written against a flat folder and name files such as `rooms.rs` and
`routes.rs`.

Moving files does not fail such a test loudly. A scan that lists `src/` without
recursing passes with fewer files in view, so the guardrail weakens silently.
Moving the first batch of files in #66 made two scans (AC-57, the copy twin)
pass while reading fewer files, and a third fail only because it also asserts
a count.

## Decision

- **Every scan walks `room/src` recursively.** `cli/` is skipped where the scan
  never read `bin/` before. Fixed: `no_fallback_route`, the AC-57 identity scan
  (`lifecycle.rs`), the copy twin check (`twins.rs`), `used.rs`'s writer scan.
- **Scans of the routes read each file on its own.** `common::route_files()`
  returns every file under `server/routes/` (subfolders included, `mod.rs`
  first) as `(path, text)`, so a span never runs into the next file and a
  failure names the file at fault. `common::routes_source()` joins them, only
  for "does this string appear anywhere" checks.
- **A rule that named a file now names the file that holds the concern.** The
  admin token may appear only in `routes/pipeline.rs`; a machine is assigned only
  in `rooms/room.rs`; a question is opened only in `rooms/open.rs`; the ledger is
  appended only in `rooms/state.rs`, behind the parting built in `room.rs`.
- **Scans lean on files, not marker blocks.** The admin-token rule treats all
  of `pipeline.rs` as the admin block, and the participant check reads all of
  `connections.rs`. `mod.rs` still writes `// see T-06 pages` at call sites, so
  no search for a marker finds a call site by mistake.

## Options Considered

### Option A: Walk the tree; name the concern's file (chosen)
**Pros:** a moved file stays in view; rules read like the architecture;
stricter in two places.
**Cons:** a rule must be edited when its concern moves.

### Option B: Keep the old file names as thin re-export files
Scans pass unchanged. **Rejected:** they would pass because they read a stub.

### Option C: Replace the scans with a compiler-enforced module layout
Stronger, and a large redesign. Out of scope; the scans stay a complement to the
compiler, as `tests/boundary.rs` says of itself.

## Consequences

**Benefits:** the guardrails read more of the tree than before. Two got
stricter (machine assignment; vault opening). The pattern is documented, so the
next move does not repeat the silent weakening.

**Harder / accepted:** a new route file or a new `rooms/` file is picked up by
the walk with no edit; a rule that hard-codes a file must change with it. A
non-recursive `read_dir` in a new test would repeat the mistake: prefer
`common::room_sources(skip)` (every `.rs` under `room/src`) or
`common::route_files()` (the route files, one text each, subfolders included).

## Action Items

1. [x] Make the four scans recursive; add `routes_source()` (PR #66)
2. [x] Re-point the rules to `pipeline.rs`, `room.rs`, `open.rs`, `state.rs`
3. [x] Shared walk helper: `common::room_sources` and `common::route_files`; the
   old copies in `boundary`, `used`, `admin`, `lifecycle`, `twins` and
   `no_fallback_route` use it. `auth.rs`'s `shipped_files` still walks
   `.github/` and the manifests, which are not `.rs` under `src/`.
4. [x] The AC-67 scan reads all of `connections.rs`, and the admin-token rule
   treats all of `pipeline.rs` as the admin block, so neither rests on markers.
5. [x] The canary's route table and `served_files` parse each route file on
   their own (no span runs into the next file).

## Where it lives

`room/tests/{boundary,used,admin,auth,twins,lifecycle,no_fallback_route}.rs` ·
`room/tests/common/mod.rs` · `room/tests/canary_scan/mod.rs`
