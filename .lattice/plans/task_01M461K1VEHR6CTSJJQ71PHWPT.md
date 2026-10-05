# PQ-61: Deployed burst keeps an organizer session and the admin token as repo secrets

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-19** (G-9, Minor, new): Deployed burst keeps an organizer session and the admin token as repo secrets
- Evidence: .github/workflows/deployed-burst.yml:7-11,58-67,100
- What would close it: Name it in SPEC 8 as a credential path, or pass them as one-run inputs

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
