# PQ-64: Persisted fields with no writer or no production reader

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-27** (Cross-cut., Minor, new): Persisted fields with no writer or no production reader
- Evidence: see the one-writer table
- What would close it: Delete or wire each, per SPEC 3

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
