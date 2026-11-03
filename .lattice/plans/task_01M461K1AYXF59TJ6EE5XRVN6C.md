# PQ-57: A receipt can claim a Miri run the record does not hold

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-16** (G-7, Major, new): A receipt can claim a Miri run the record does not hold
- Evidence: answers.rs:347, receipt.py:120-121; answers.rs:174 accepts runs without miri; M7a, M7b survived
- What would close it: A receipt fixture with runs and no miri, asserting no Miri line, in both twins

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-11-02)

Tests and one fixture; no production code changes.

1. Add `bank/fixtures/receipts/ran-no-miri.json`: `complete-ran.json`'s synthetic record with `miri` removed. Its `expected_lines` come from running the real renderer, and I check by reading that they hold no Miri line.
2. Readers of the directory, confirmed by reading: `twins.rs` (room), `test_receipt.py` and `test_fallback.py` (pipeline) and `web/test/wall.test.js` (wall, prints the lines only). All glob it, and all their floors stay valid, so no count changes.
3. Add a named test in each twin, reading that fixture by name: `twins.rs` and `test_receipt.py` each assert the record has `runs` and no `miri`, renders a receipt, and has no line mentioning Miri.
4. Plant M7a (`answers.rs` :464, moved from the audit's :347) and M7b (`receipt.py` :120), confirm each fails its own twin's named test, and revert each.
5. `just test`, Codex review (two passes at most), PR. Stop at `review`.

Halt if a twin renders a Miri line for the fixture, if a loader refuses it, or if it would change a rendered string.

## Reset 2026-11-02 by agent:delegator-pq57
