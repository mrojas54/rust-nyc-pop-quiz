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
