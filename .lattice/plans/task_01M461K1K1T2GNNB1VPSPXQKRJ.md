# PQ-59: Room and phase test gaps: fit after release, the host sheet phase order, the route-table scan

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-10** (G-10, Minor, new): fit is writable after release; the guard is untested
- Evidence: rooms.rs:438; routes.rs:364 (no auth); M10d survived
- What would close it: A test that a released room ignores PUT fit and its revision does not move

**GAP-15** (Phase inv., Minor, new): The host sheet's phase order is a third literal no test ties to G-6
- Evidence: fallback.py:90; M6h survived
- What would close it: Assert fallback.PHASES equals phase.js's order

**GAP-17** (G-9, Minor, new): The route-table tests read routes.rs only
- Evidence: canary_scan/mod.rs:566-592; admin.rs:292; M9a2 survived
- What would close it: Walk the built Router, or scan every file that calls .route/.nest/.fallback

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-11-02)

Outcome: one PR, tests only, each new test shown failing under its mutation. Base 622e0fd.

1. **GAP-10.** `popquiz/tests/wall_page.rs`, beside the fit tests: walk a room to release through the routes, then `PUT /rooms/{id}/fit` with a different verdict. Assert 204, revision unchanged, host's fit unchanged, used record's fit unchanged. Mutation M10d: drop the `!= Released` guard in `record_fit`.
2. **GAP-15.** `pipeline/tests/test_fallback.py`: read `web/shared/phase.js`, parse `var PHASES = [...]`, fail loudly on no match or an empty list, assert equal to `fallback.PHASES`. Mutation M6h: swap `closed`/`split` in `fallback.py`, then separately in `phase.js`.
3. **GAP-17.** `common::router_files()` over `room_sources(&[])`: every file under `popquiz/src` that calls `.route(`, `.nest(`, `.fallback(` (and the `_service` forms, which those strings miss). `route_table()` reads those files, not `server/routes/` alone. A `.nest`/`.fallback` it cannot read panics. `cli/spike-server.rs` is the one named exemption: its own feature-gated binary and router; the exemption must stay true. Scanner self-tests with a planted source keep the guard live. The admin prefix check also covers router files outside `server/routes/`. Mutation M9a2: `/ops/used` in `lib.rs`, plus the same route in a new file.
4. **AC-71 wall clause.** Read: `wall.test.js:156-176` asserts q3's explanation prose is absent. It does not assert a sentence *about* the explanation is absent, which is what the clause says and what report row 41 asked for. Show that by planting `home_machine_only` in the receipt with the test unchanged. If it survives, add the assertion. Revert.
5. Keep the test name `every_route_in_routes_rs_is_scanned_or_listed`: `popquiz/README.md` and the audit name it and neither is cleared. Report the README's now-narrower wording in DONE.

A failing test that shows a real room bug: halt, no source fix.

## Reset 2026-11-02 by agent:delegator-pq59
