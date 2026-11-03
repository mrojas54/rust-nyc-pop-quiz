# PQ-62: The distribution scan's file list misses docs/RUNBOOK.md, web/README.md, the page shells and room/src/copy.rs

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-22** (G-11, Minor, new (CONTRACT NOTE 2)): The distribution scan's file list misses docs/RUNBOOK.md, web/README.md, the page shells and room/src/copy.rs
- Evidence: audit.py:636-647; M11b survived
- What would close it: Scan by rule (every authored file outside bank/) and settle which organizer docs may carry tells

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

## Reset 2026-11-03 by agent:delegator-pq62

# Plan (delegator, 2026-11-03)

Client ruling (RESUME, "recommendation"): exempt by path `sequence/`, `docs/audit/`, `pipeline/tests/test_audit.py`, `mvp/2026-08-12/` and `PRD.md`. Exempt `BUILDPLAN.md:56` by exact line. Skip Python caches. The PRD fix goes under not-done.

**audit.py** (file list only; `RATIO`, `KIND_WORD` and `distribution_hits` are untouched):
1. Replace `PARTICIPANT_FACING` / `PARTICIPANT_FACING_DIRS` with a rule:
   - Walk the tree with `os.walk`, pruning `bank/` at the root.
   - Prune `target`, `node_modules`, `.git`, `.venv`, `__pycache__`, `.pytest_cache`, `.ruff_cache` and `.mypy_cache` anywhere.
   - Read every UTF-8 file.
   - Skip `DISTRIBUTION_EXEMPT_PATHS`, a commented dict of path to reason; a directory entry ends in `/`.
2. Add the `BUILDPLAN.md:56` line to `LINT_EXEMPTIONS`.
3. `distribution_lint` keeps its 3-tuple. The third element becomes the exempt paths missing from the tree (stale exemptions). The bank-audit report key changes from `not_present_yet` to `stale_exemptions`; nothing else reads that key.

**test_audit.py**:
1. The real-tree test now walks every file, and asserts that RUNBOOK, web/README, copy.rs, the five shells, copy.js and web/home are all scanned.
2. tmp_path tests:
   - A tell in `docs/RUNBOOK.md` (M11b), in `web/wall/index.html` and in `popquiz/src/copy.rs` is caught.
   - The same tell under `bank/`, `target/`, `node_modules/` or an exempt path is not.
   - `sequencex/` is not exempted by `sequence/`.
3. Every path exemption names something in the tree.

**Mutations, each reverted:**
- M11b: the M11a sentence in `docs/RUNBOOK.md`.
- A tell in `web/buzzer/index.html`.
- A tell in `popquiz/src/copy.rs`.
Each should fail `test_the_real_participant_facing_files_are_clean`.

**Check:** `just test` passes, and `distribution_lint` on this tree reports 0 hits.

## Reset 2026-11-03 by agent:delegator-pq62

## Reset 2026-11-03 by agent:delegator-pq62
