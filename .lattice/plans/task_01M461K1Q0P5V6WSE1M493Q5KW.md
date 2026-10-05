# PQ-60: Room-authored reason strings sit outside the copy module and are unlinted

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-14** (G-5, Minor, new): Room-authored reason strings sit outside the copy module and are unlinted
- Evidence: routes.rs:196,208; M5b3 survived
- What would close it: Move them into copy.rs/copy.js or lint room/src string literals

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
