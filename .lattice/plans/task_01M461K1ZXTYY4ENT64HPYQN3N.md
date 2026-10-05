# PQ-62: The distribution scan's file list misses docs/RUNBOOK.md, web/README.md, the page shells and room/src/copy.rs

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-22** (G-11, Minor, new (CONTRACT NOTE 2)): The distribution scan's file list misses docs/RUNBOOK.md, web/README.md, the page shells and room/src/copy.rs
- Evidence: audit.py:636-647; M11b survived
- What would close it: Scan by rule (every authored file outside bank/) and settle which organizer docs may carry tells

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
