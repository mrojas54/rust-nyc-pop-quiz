# PQ-53: The provenance check is not on the build path, so an uncommitted or local bank schedules without it

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-4** (G-2, Major, new): The provenance check is not on the build path, so an uncommitted or local bank schedules without it
- Evidence: check_provenance (verify.py:577) runs in CI over committed files only (test_verify.py:425-428); schedule.refusal (schedule.py:127) and fallback.bake (fallback.py:105) skip it; the replay covers MIGRATED only (migrate_mvp.py:76); M2a survived
- What would close it: Call check_provenance in refusal and bake, and replay every bank question that has a recording

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
