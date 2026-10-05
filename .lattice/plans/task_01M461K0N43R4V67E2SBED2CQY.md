# PQ-51: The arrangement is outside every slot lint and simulation

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-1** (G-1, Major, new): The arrangement is outside every slot lint and simulation
- Evidence: schedule.py:189-208, schedule.py:166 (own _rng); audit.py:357,501,537 read slot.py and slot_for_day calls only; M1c survived
- What would close it: Lint schedule.arrange like the slot path (no bank, used or ledger reads; no draw but slot_for_day) and run the attendee simulation over arrange's output positions

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
