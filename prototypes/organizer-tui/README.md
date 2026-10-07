# Organizer question and teaching-trace preview

A runnable, read-only terminal prototype for reviewing existing bank candidates.
It uses Python's standard-library curses interface and the existing bank models;
it adds no runtime dependencies. It is an exploratory interface, not the generator
or the complete organizer approval workflow.

## Run

From this checkout's `pipeline/` directory:

```sh
UV_PYTHON=3.12 uv run python -m popquiz.preview_tui --question q3
```

Open in an interactive macOS/Linux terminal of at least 36 columns and 12 rows;
100 columns and 36 rows is more comfortable. Windows requires a curses-compatible
environment. Use `--bank /absolute/path/to/bank` to inspect another bank directory
with a `questions/` subdirectory. Records are read, never modified.

| Key | Action |
| --- | --- |
| `1` | Question and five choices |
| `t` or Tab | Toggle question / teaching trace |
| Left / Right | Previous / next teaching step |
| `r` | Explicit reveal; enter the final resolving step |
| `[` / `]` | Previous / next candidate; return to question view |
| Up / Down or `k` / `j` | Scroll the current view |
| Page Up / Page Down | Scroll a page |
| `q` or Escape | Exit |

Trace mode highlights the authored source lines, respects the focus range, shows
the host's narration and authored value illustrations, and stops before the
resolving step. Reveal shows that last step plus the answer derived by the
existing bank function, explanation, distractor rationales, and recorded receipt.

For a non-interactive preview:

```sh
UV_PYTHON=3.12 uv run python -m popquiz.preview_tui --question q3 --snapshot question
UV_PYTHON=3.12 uv run python -m popquiz.preview_tui --question q3 --snapshot trace --step 3
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

Validation evidence is in [VALIDATION.md](VALIDATION.md).
