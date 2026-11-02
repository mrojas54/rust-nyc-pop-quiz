# PQ-75 checkpoint (uncommitted)

**Phase done:** plan. Plan is at `/Users/michellerojas/rust-nyc-pop-quiz/.lattice/plans/task_01M4SJWPXYG3FQPA60NV7M50Z8.md` (every hit, left list, scans, commit order). Ticket `planned`.

**Next:** implement in five commits (rename; build/deploy/CI paths; scans; comments and docs; ADR tick), then `just setup`, `just test`, checkpoint, Codex review, validate, PR.

**Decisions:**
- web/ outside web/test stays untouched (not cleared), so served bytes stay identical. The `_note` strings in take_home.rs (web fixture) and wall_page.rs stay `room/tests/...` because they must equal fixtures outside the cleared files.
- audit.py keeps its skip of a missing folder; guards are test-only (test_audit.py, test_kill_rerun.py). Recorded as a deviation for DONE.
- room_client.rs CLI refusals get the new README path. Not copy.rs, copy.js or a rendered page, so not a clause 10 halt. Mention in DONE.

**Files touched so far:** none in git. `.claude/settings.json` and `.claude/boot-prompt.md` are the Orchestrator's: leave uncommitted.

## After implement + just test (04:52Z)
HEAD 39df857, six commits on e073ec7, unpushed. `just test` green: web 303, pipeline 1105 passed 1 skipped, cargo 345 passed 4 ignored across 35 results. Warm 77 s at load average 80-95 from sibling builds; pipeline alone 24 s under that load. Retime when quiet.
Checked: `python -m popquiz.*` from the repo root resolves to pipeline/src/popquiz (regular package beats the root folder).
Next: Codex review, then mutations, docker build, boot a local room, final grep, PR. One heavy gate at a time (Orchestrator comment): siblings PQ-53, PQ-56 are live. PQ-56 touches copylint and web/shared/README.md.

## PR open (05:42Z)
PR #116, HEAD fc95f61, CI green, ticket in review, DONE posted. Nothing left but the client's merge. Do not merge here.
