# PQ-73: PQ-71 follow-ups: stale discord.rs mentions, host_page.rs imports, rustdoc warnings

Items PQ-71's DONE deferred (PR #79, merged a006424):
- Stale `discord.rs` mentions after the split into host/discord/: room/Cargo.toml:115,119; connections/requestlog.rs:8; ADR-0001; ADR-0002:35; the guardrail audit.
- host_page.rs unused imports.
- 4 rustdoc private-link warnings.
Behaviour-preserving. Line numbers are as of a006424; re-check on main.
