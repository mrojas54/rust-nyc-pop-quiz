# PQ-51: The arrangement is outside every slot lint and simulation

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-1** (G-1, Major, new): The arrangement is outside every slot lint and simulation
- Evidence: schedule.py:189-208, schedule.py:166 (own _rng); audit.py:357,501,537 read slot.py and slot_for_day calls only; M1c survived
- What would close it: Lint schedule.arrange like the slot path (no bank, used or ledger reads; no draw but slot_for_day) and run the attendee simulation over arrange's output positions

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-11-02)

Outcome: `bank-audit`'s `slot path and ledger` check goes red when an ordinary edit to `schedule.arrange`, `_shuffled` or `_rng` lets anything but the date place the answer. `arrange` and every answer's position stay exactly as they are. Simulation half stays closed and untouched.

1. `audit.py`: add `arrange_violations(source)` beside the slot lints, and include it in `_lint_check`. Static only: `ast.parse`, never imports or runs `schedule`.
   - Fail closed: each of the three functions must exist once at the top level, with today's signature, undecorated. An unparseable or missing source is a problem.
   - Reads: per-function allow-list of globals and attributes, plus the slot lint's forbidden words on locals, keywords and strings. This refuses `question.used_by`, `load_bank`, a ledger read (M1c).
   - Position: in `arrange`, one `slot_for_day(day)` bound once; `options.insert` called once, unconditionally, with that name; no rebinding, no other subscript, no early return, no loops, so nothing moves the answer afterwards.
   - Shuffle: one `_shuffled(others, "distractors", day.isoformat(), question.id)` on the list of the four others; `_shuffled` calls `_rng(*seed_parts)` once; nothing else in the module uses either; `slot_for_day` is the import from `popquiz.slot`.
2. `test_audit.py`: real source clean; planted sources as text, in the existing slot-lint style: M1c, slot drawn from `_rng`, `_rng` seeded with the used count, each function renamed, plus the shapes above; `_lint_check` goes red on a plant.
3. Prove it: each named mutation made in the real `schedule.py`, `_lint_check` red, reverted.

Out of scope by ruling (goes in `not-done`): aliases, `getattr`-style indirection through allowed objects, a caller of `arrange` passing a different date or reordering the result, edits to `correct_index` in `bank.py`.

## Reset 2026-11-02 by agent:delegator-pq51
