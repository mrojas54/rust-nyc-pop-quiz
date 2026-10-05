# PQ-52: The standalone fallback builder bakes stored order, answer at E for every bank question, with no date, no arrange, no affirmation or used check

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-2** (G-1, G-12, Major, new (not PQ-48 or PQ-45)): The standalone fallback builder bakes stored order, answer at E for every bank question, with no date, no arrange, no affirmation or used check
- Evidence: fallback.py:27-29,306-333; `python -m popquiz.fallback q3 --bake` exit 0, E = q3's answer; test_fallback.py:288 pins it; room/README.md:595-596 documents it; M12d
- What would close it: Require --date and arrange, apply schedule.refusal, or remove the CLI in favour of `popquiz schedule --no-push`

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.
