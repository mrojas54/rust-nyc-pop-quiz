# PQ-63: The room accepts a pushed question with no affirmation

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-25** (G-12, Minor, new): The room accepts a pushed question with no affirmation
- Evidence: answers.rs:100 (review ignored); admin.rs
- What would close it: Decide whether the room refuses an unaffirmed bank question or the gate stays client-side and is said so

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
