# PQ-55: A RevealWitness can be forged without boundary.rs noticing

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-9** (G-3, Major, new): A RevealWitness can be forged without boundary.rs noticing
- Evidence: phase.rs:336; boundary.rs:98-113,148-156; M3e, M3e2 survived
- What would close it: Put RevealWitness in the no-derive, no-impl scan and count every constructor form in phase.rs

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
