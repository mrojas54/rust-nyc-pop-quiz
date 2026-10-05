# PQ-56: The Forbidden row is not enforced on question prose

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-11** (G-5, Major, new): The Forbidden row is not enforced on question prose
- Evidence: copylint.py:183-194 warns only; test_copylint.py:191-199; M5d2 survived
- What would close it: Fail `just test` on a Forbidden match in explains, why_tempting, hint (tropes stay warnings, SPEC 11.1)

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
