# PQ-57: A receipt can claim a Miri run the record does not hold

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-16** (G-7, Major, new): A receipt can claim a Miri run the record does not hold
- Evidence: answers.rs:347, receipt.py:120-121; answers.rs:174 accepts runs without miri; M7a, M7b survived
- What would close it: A receipt fixture with runs and no miri, asserting no Miri line, in both twins

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
