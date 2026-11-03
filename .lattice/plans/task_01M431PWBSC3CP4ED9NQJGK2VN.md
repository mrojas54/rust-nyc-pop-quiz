# PQ-47: Static dependency check: the room reaches no generator, verifier or LLM code

From the Result Validator report, row 1 (AC-1, Partial). room/Cargo.toml depends on axum, tokio, serde, serde_json, getrandom, subtle, rustls, tokio-rustls, webpki-roots, futures-util and optional tokio-tungstenite; no generator, verifier or LLM crate and no path dependency on pipeline/, but nothing in just test asserts it (room/tests/boundary.rs checks unsafe and the vault only). The check is T-16's deliverable and PQ-21 waits on H-4, so this splits the static half out. Shape: a test under room/tests/ (boundary.rs or a sibling) that reads room/Cargo.toml and Cargo.lock's room subgraph and fails if any crate or path matches a denylist (anthropic, openai, llm, generator, popquiz, pipeline, miri, rustc_*), plus a source scan of room/src for a use of any such module; cargo test picks it up, so the justfile is untouched. Acceptance: the test passes on main; a planted path dependency on ../pipeline in a throwaway copy fails it (name the mutation). HELD: minted from the audit; released on the client's word.

# Plan (delegator, 2026-11-03)

One new file, `popquiz/tests/dependencies.rs` (a sibling of `boundary.rs`, which is about the seal, not dependencies). No production change, no crate, no justfile change. Reads text only.

**Denylist (one table, crate names only, never paths).** Names are lowercased and `-`→`_` (Cargo's own identity). Denied when a `_`-separated word is `anthropic`, `openai`, `llm`, `generator`, `miri`, `pipeline` or `popquiz` (the Python package's import and distribution name, `pipeline/pyproject.toml`), or when the name starts with `rustc_`. Matching names, not paths, is how the folder `popquiz/` stays out of it: the crate is `room`, and no check looks at a directory name.

**Tests.**
1. Manifest: every dependency table (`[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[target.*.…]`, dotted `[dependencies.x]`; optional and feature-gated entries live in these). Each key and any `package = "…"` rename go through the denylist; any `path = "…"` resolving into the repo's `pipeline/` fails.
2. Lock (`popquiz/Cargo.lock`, the only lock): the closure from `room` over `dependencies`; every name through the denylist; any package with no `source` other than `room` (a path package) fails.
3. Source: `popquiz/src/**/*.rs`, comments stripped, every `use` / `extern crate` root (brace groups included). A root the same file declares with `mod` is local (`server/routes/mod.rs` declares `mod pipeline;`, the room's admin route), as Rust resolves it.
4. The reader is not vacuous: it finds axum (deps), tokio-tungstenite (optional), tower (dev), and `room → axum` in the lock.
5. The mutations as fixtures: M1, M2, M3 spliced into the real text are each refused, so they stay caught.

**Mutations on the real files.** M3 in place (`#[cfg(any())] use openai::Client;` compiles; test 3 fails). M1 and M2 cannot compile under `--locked`/offline (cargo refuses first), so they run against a throwaway copy: build the test binary once, copy `Cargo.toml`/`Cargo.lock`/`src` to `$TMPDIR`, mutate, run the binary with `CARGO_MANIFEST_DIR` pointing at the copy (the test reads it at runtime). Each named failing test is recorded.

**Check.** `just test` green; Codex review ≤2 passes; PR to main.

## Reset 2026-11-03 by agent:delegator-pq47

## Reset 2026-11-03 by agent:delegator-pq47

## Reset 2026-11-03 by agent:delegator-pq47
