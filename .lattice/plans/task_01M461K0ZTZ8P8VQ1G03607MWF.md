# PQ-54: Two live rooms on one question: releasing one publishes the answer at /last while the other is pre-reveal

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-8** (G-3, Major, new): Two live rooms on one question: releasing one publishes the answer at /last while the other is pre-reveal
- Evidence: rooms.rs:846-886 (create_for checks used only), rooms.rs:925 (global take_home), routes.rs:639; probe test
- What would close it: Refuse create_for while another unreleased room holds the question, with a test

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
