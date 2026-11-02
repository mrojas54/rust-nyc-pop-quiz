# PQ-75: ADR-0005 item 7: rename the crate folder room/ to popquiz/

ADR-0005 Action Item 7. Rename room/ to popquiz/ and update justfile, CI, Fly config, docs and scans that name the path.

# Plan (delegator, 2026-11-01)

Move `room/` to `popquiz/`, folder only. Package, lib and `room` binary names, Fly apps, recipes and env vars stay. Nothing changes on the night.

## Commit order
1. `git mv room popquiz`, nothing else (every file a 100% rename; proof: `git diff -M --summary HEAD~1 HEAD`, `git diff --find-renames=100% --stat`).
2. Build, deploy and CI paths: Dockerfile, .dockerignore, fly.toml, justfile, workflows, organizer Cargo.toml.
3. Scans and tests that read the crate by path (Rust, pipeline).
4. Comments and docs: crate source comments, pipeline and web/test comments, READMEs, RUNBOOK, REHEARSAL-GUIDE, PRODUCT, bank/README, CLAUDE + AGENTS, .claude/agents.
5. ADR-0005 item 7 ticked, nothing else in the file.

## Every hit
Changed (live paths):
- Dockerfile: COPY lines, `cd room`, `/src/room/target/release/room` (the binary path `/usr/local/bin/room` and `--bin room` stay), comments.
- .dockerignore: the three allowlist entries and the comment.
- fly.toml comments; popquiz/fly.spike.toml, popquiz/spike/{Dockerfile,.dockerignore,reports/README.md} comments and `cd`.
- justfile: every `cd room` (setup, test-room, test-full, harness-full, test-transport-full, canary, secret-scan, smoke, burst) and comments. Recipe names stay.
- .github/workflows/ci.yml: cache paths, hashFiles keys, `cd room`. deployed-burst.yml: same, plus `room/target/release/{burst,smoke}` and the error text pointing at the README. deploy.yml has no path hits.
- prototypes/organizer-tui/Cargo.toml path dependency. Cargo.lock names the package, not the path, so it should not change; `--locked` proves it.
- popquiz/README.md, room_client.rs, the Rust scans and tests below.
- pipeline: audit.py tuple and docstring, test_kill_rerun.py, comments in schedule.py, bank.py, test_affirmation_twin.py, test_club_twin.py, test_fallback.py.
- web/test comments only: buzzer, host, home, wall, wall-static, copy-freeze, copylint.
- README.md, docs/RUNBOOK.md, REHEARSAL-GUIDE.md, PRODUCT.md, bank/README.md, CLAUDE.md + AGENTS.md, .claude/agents/{rust,refactoring}-specialist.md.

Deliberately left, listed in DONE:
- Never-edit list: BUILDPLAN.md, docs/audit/**, sequence/** (their `room/` paths are history).
- Dated ADRs 0001-0004; ADR-0005 keeps its own paths, only item 7 is ticked.
- Not cleared: web/ outside web/test (comments in buzzer.js, host.js, wall.js, measure.html, home/index.html, shared/phase.js, shared/README.md, fallback/static.js, plus the two generated fixture `_note`s), bank/audit/README.md, bank/fixtures/clubs/aliases.json. Leaving them keeps every served web file byte-identical, which is a useful proof.
- The two generator strings that must equal those fixtures: `_note` in take_home.rs (web/home/fixtures/take-home.json) and in wall_page.rs (web/wall/fixtures/q3-phases.json). Changing them would force a fixture edit outside the cleared files.
- .claude/hooks/install-flyctl.sh: write-protected, one comment.
- Not paths: `/usr/local/bin/room`, `room::` imports, `name = "room"`, `room = {path}` key, the `"room"` JSON fields, `.fly.dev/room` in smoke_config.rs, `--room`, `https://room.test`, `/host/room-1`, `rooms/room.rs`, `/room-reorg/`.

## Paths built at run time
- pipeline/src/popquiz/audit.py `for top in ("web", "room")`: **silently skips a missing folder** (`if not root.is_dir(): continue`).
- test_kill_rerun.py `REPO / "room"` twice: `rglob` on a missing folder yields nothing, so the digest would quietly stop covering the crate.
- Relative `include_str!`/`#[path]`/CARGO_MANIFEST_DIR: unchanged, same folder depth. `repo().join("room/...")` in common/mod.rs, auth.rs, one_room.rs, sessions.rs, canary_scan/mod.rs, take_home.rs, used.rs change.

## Scans and how each is shown detecting
Already fail on a missing tree: common::room_sources (`read_dir().unwrap()`, asserts >10 files), auth.rs (`read_dir().unwrap()`), one_room.rs and sessions.rs (`read_to_string().unwrap()`), canary_scan (node exits on a missing file), used.rs and twins.rs through room_sources, boundary.rs through CARGO_MANIFEST_DIR.
Pass on an empty walk today, so a test-only guard is added (no logic in audit.py changes, deviation recorded in DONE):
- bank_audit_references: test_audit.py asserts `popquiz/` exists in the repo and that a plant under `popquiz/` in a tmp repo is found. audit.py keeps its skip because the existing tmp-repo test relies on it.
- test_kill_rerun.py: assert the crate walk found files.
Planted violations, one per suite, each reverted: room (a forbidden pattern in `popquiz/src/` through a room_sources scan and through auth.rs), pipeline (a `bank/audit` mention in `popquiz/src/`), web (a scan in web/test). Bounded at 15 minutes each. The organizer suite reads no crate source; it proves the path dependency by compiling.

## Image and validation
Docker is running (29.8.2). `docker build` from the worktree root through the approval prompt, then confirm `/usr/local/bin/room` in the image. CI does not build the image on a PR, so this local build is the only pre-merge proof. No `fly deploy`.
Then `just setup`, `just test` warm under 60 s, `just test-full` if Docker holds, Codex review of `origin/main...HEAD`, a final grep with each hit explained, a local room booted through its existing harness.
