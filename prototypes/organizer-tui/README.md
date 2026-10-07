# Organizer question and teaching-trace preview

A runnable, read-only Rust terminal prototype for reviewing existing bank candidates.
Ratatui 0.30.2 renders the interface; Crossterm handles terminal events. Answer
derivation and receipts reuse the live room's Rust implementation. This replaces
the earlier Python curses prototype. It is an exploratory interface, not the
generator or the complete organizer approval workflow.

## Run

From this checkout's `prototypes/organizer-tui/` directory:

```sh
cargo run --locked -- --question q3
```

Open in an interactive terminal of at least 40 columns and 14 rows;
100 columns and 36 rows is more comfortable. The first build fetches dependencies.
Use `--bank /absolute/path/to/bank` to inspect another bank directory
with a `questions/` subdirectory. Records are read, never modified.

| Key | Action |
| --- | --- |
| `1` | Question and five choices |
| `t` or Tab | Toggle question / teaching trace |
| Left / Right or `<` / `>` | Previous / next teaching step |
| `r` | Explicit reveal; enter the final resolving step |
| `[` / `]` | Previous / next candidate; return to question view |
| Up / Down or `k` / `j` | Scroll the current view |
| Page Up / Page Down | Scroll ten rows |
| `q`, Escape, or Ctrl-C | Exit and restore the terminal |

Trace mode highlights the authored source lines, respects the focus range, shows
the host's narration and authored value illustrations, and stops before the
resolving step. Reveal shows that last step plus the answer derived by the
existing bank function, explanation, distractor rationales, and recorded receipt.

For a non-interactive preview:

```sh
cargo run --locked -- --question q3 --snapshot question
cargo run --locked -- --question q3 --snapshot trace
```

## Scope

- Existing authored teaching traces, not instrumented variable captures.
- Existing recorded verification, with legacy provenance shown; no compiler runs.
- Canonical bank choice order, clearly labeled, not date-derived meetup lettering.
- Source lines wrap to remain reachable in narrow terminals; terminal appearance
  is not evidence of actual projector fit. Syntax coloring is not implemented.
- No generation, editing, saved review decisions, affirmation, scheduling, usage
  writes, or projector-browser integration in this prototype.
- q3 is demonstration content; its appearance here makes no claim of unused status.

Snapshots render the same Ratatui widgets through TestBackend. For local checks:

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

Validation evidence is in [VALIDATION.md](VALIDATION.md). The prototype is a
standalone Cargo package and is not part of the live room's build or deployment.
