# PQ-73: PQ-71 follow-ups: stale discord.rs mentions, host_page.rs imports, rustdoc warnings

Items PQ-71's DONE deferred (PR #79, merged a006424):
- Stale `discord.rs` mentions after the split into host/discord/: room/Cargo.toml:115,119; connections/requestlog.rs:8; ADR-0001; ADR-0002:35; the guardrail audit.
- host_page.rs unused imports.
- 4 rustdoc private-link warnings.
Behaviour-preserving. Line numbers are as of a006424; re-check on main.

# Plan (delegator, 2026-10-18)

Base `origin/main` 8098174. Line numbers re-checked on this worktree.

## 1. Stale `discord.rs` mentions (`rg -n 'discord\.rs'`)

| Where | Fix |
|---|---|
| `room/Cargo.toml:115` | HTTPS client is `host/discord/http.rs` |
| `room/Cargo.toml:119` | `host/discord/http.rs` passes ring (`ClientConfig::builder_with_provider`, http.rs:113) |
| `room/src/connections/requestlog.rs:8` | "the same sink the Discord backend (`host/discord/`) writes to" (`Stderr` in config.rs) |
| ADR-0001:24 (problem table, history) | keep; add "(now `host/discord/mod.rs`)" |
| ADR-0001:197 (*Where it lives*, a pointer) | `room/src/host/discord/mod.rs` |
| ADR-0002:35 (decision diagram) | keep; add "(now discord/mod.rs)" in the label |
| ADR-0005:17,302,313,327 | kept: they describe the split itself |
| `docs/audit/2026-10-guardrail-audit.md:422,423,690` | kept as a dated record (ruling 3) |

## 2. `room/tests/host_page.rs` unused imports

`axum::extract::Path` (28), `axum::response::IntoResponse` (30), `axum::routing::get` (31): remove. Deviation: the file is under `room/tests/`, not `room/src/`; the ticket names it, so it is in scope.

## 3. Rustdoc private-link warnings (`cargo doc --no-deps --document-private-items`; no justfile recipe)

- `rooms/create.rs:176`, `rooms/room.rs:307`, `rooms/state.rs:226` -> `AppState::create_for` (private)
- `server/mod.rs:3` -> `routes` (`pub(crate)`)

No public item stands for either target (`create_for` is the private room maker; `routes` is crate-private by design), so retargeting would change the sentence's meaning. Fix: plain code spans, which is the convention `server/mod.rs` already uses for the private `reaper` (line 7). Text unchanged.

## Verify

`cargo build --tests` warn-free for host_page; rustdoc 4 -> 0; `just test`; diff filter of ruling 6.
