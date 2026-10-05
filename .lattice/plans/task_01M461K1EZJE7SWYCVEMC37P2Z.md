# PQ-58: G-2 provenance follow-ups: stale pin, record digest, the MVP deck

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-5** (G-2, Minor, new): Scheduling does not refuse a stale pin
- Evidence: verify.is_stale (verify.py:507) has no caller; SPEC 7.2
- What would close it: Call is_stale in schedule.refusal for non-legacy records

**GAP-6** (G-2, Minor, new (CONTRACT NOTE 1)): A self-consistent hand edit to a verified record is undetectable
- Evidence: verify.py:592-596; M2c accepted by check_provenance
- What would close it: Contract: a digest or recording id binding the record to its run

**GAP-7** (G-2, Minor, new): The MVP deck trusts verified.json's answer with no provenance and no test
- Evidence: mvp/tools/build_deck.py:241,266; mvp/tools/verify.py:166-216
- What would close it: A provenance check in build_deck, or retire the MVP deck once the room runs

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
