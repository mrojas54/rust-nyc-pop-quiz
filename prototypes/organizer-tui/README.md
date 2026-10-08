# Organizer preview

Preview a Rust quiz question and rehearse its teaching trace in the terminal.
Read the code and choices, step through the explanation, then reveal the answer.

The preview reads questions from the bank without changing them.

![Question preview, teaching trace, and answer reveal](recordings/organizer-preview.gif)

The recording walks through `q3`, reveals the answer, and scrolls to the
verification receipt. [Download the terminal recording](recordings/organizer-preview.cast)
to replay it with `asciinema play recordings/organizer-preview.cast`.

## Run

From the repository root:

```sh
cd prototypes/organizer-tui
cargo run --locked -- --question q3
```

The first build downloads dependencies. Use a terminal at least 40 columns wide
and 14 rows tall; 100 × 36 gives the code and narration more room.

To load another bank, add `--bank /path/to/bank`. That directory must contain a
`questions/` subdirectory with the question JSON files.

## Controls

| Key | Action |
| --- | --- |
| `1` | Show the question and five choices |
| `t` or Tab | Switch between the question and teaching trace |
| Left / Right or `<` / `>` | Step backward or forward |
| `r` | Reveal the answer and final trace step |
| `[` / `]` | Open the previous or next question |
| Up / Down or `k` / `j` | Scroll the current view |
| Page Up / Page Down | Scroll ten rows |
| `q`, Escape, or Ctrl-C | Exit and restore the terminal |

Each trace step highlights the relevant code and shows the host's narration and
value changes. These steps are written as a teaching aid; the values are not
captured from a debugger.

In the teaching trace, the arrow keys stop before the final step. Press `r` to see that step, the answer,
the explanation, why the other choices are tempting, and the verification receipt.
Switching questions returns to the question view.

## Print a preview

Use `--snapshot` to print a screen without starting the interactive interface:

```sh
cargo run --locked -- --question q3 --snapshot question
cargo run --locked -- --question q3 --snapshot trace
cargo run --locked -- --question q3 --snapshot reveal
```

## Current limits

This prototype previews existing questions. Generation, editing, review decisions,
explanation approval, scheduling, and projector preview are not implemented.

Choices appear in bank order. Their letters can differ from the meetup's
date-based order. The preview does not check whether a question has already been
used; `q3` is included here as a demo.

Answers and receipts come from the saved verification records. Opening a question
does not run rustc or Miri. Older records identify Miri checks run outside the
verifier.

Long source lines wrap, and syntax highlighting is not implemented. Check
legibility in the actual projector view before using a question at a meetup.

## Development

The preview uses Ratatui 0.30.2 and Crossterm. It shares answer derivation and
receipt rendering with the `room` crate. Snapshots and rendering tests use
Ratatui's `TestBackend`.

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --check
```

See [VALIDATION.md](VALIDATION.md) for test results. This Cargo package is built
separately from the live room and is not included in its deployment.
