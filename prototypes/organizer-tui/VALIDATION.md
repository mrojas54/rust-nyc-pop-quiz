# Organizer TUI prototype validation

## Ratatui replacement — 2026-10-07

Implementation commit `326b49e`, RED commit `2549b1e`.

From `prototypes/organizer-tui/`:

- `cargo test --locked`: 11 passed (4 unit, 7 integration), no failures.
- `cargo clippy --locked --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean.

The tests render real Ratatui widgets through TestBackend. They cover source and
choices, highlighted teaching steps and narration, the explicit resolving-step
boundary, return from reveal, candidate switching, narrow terminals, incomplete
traces, suppressing stdout rows before reveal, missing evidence, scroll/resize,
unchanged bank files, and unknown-candidate CLI errors.

From `pipeline/`, `.venv/bin/python -m pytest --ignore=tests/sandbox`: 710 passed.
The six obsolete Python preview tests were removed with the curses implementation.
Warnings: existing q3 copy warning and sandbox-denied pytest cache write; neither
affected test execution. Rust replaces Python in the preview, so Clippy and the
compiler are its lint/type checks.

Interactive smoke check in a 100 x 36 pseudo-terminal: opened q3, entered trace,
used real arrow-key escape sequences to reach step 5 of 6, attempted another
step without revealing, explicitly revealed step 6, returned to teaching mode,
switched candidate, and quit with terminal restoration.

Existing answer derivation and receipt rendering are reused from the `room`
crate. No source in that crate or bank data changed. This is a read-only
prototype; no new compiler/Miri verification or venue-fit claim is made.

## Earlier Python prototype — superseded

Validated on 2026-10-06 (America/New_York), implementation commit `1516e8e`.

From `pipeline/`, using Python 3.12:

- `uv run pytest --ignore=tests/sandbox`: 716 passed, one existing q3 copy warning.
- `uv run --with ruff ruff check src/popquiz/preview_tui.py tests/test_preview_tui.py`: clean.
- `uv run --with mypy mypy --strict --follow-imports=silent src/popquiz/preview_tui.py`: clean.

The initial full pytest invocation refused to collect the real-container sandbox
suite. The successful invocation follows the repository's `test-pipeline` recipe;
container verification was not run, and no new question verification is claimed.

The six new tests cover pre-reveal content isolation, the resolving-step boundary,
returning from reveal and switching candidates, missing/short traces, narrow
rendering, and unchanged bank files. Tests first failed because the module did
not exist (RED commit `b776f84`).

Interactive smoke check in a 100-column, 36-row pseudo-terminal:

- Opened q3 with all five choices and no marked answer.
- Entered the teaching trace and stepped through source highlights and narration.
- Attempted to step beyond teaching step 5 of 6; stayed at step 5.
- Pressed `r`; entered resolving step 6 and displayed the recorded answer.
- Pressed `q`; exited cleanly and restored the terminal.

This establishes prototype behavior, not projector fit, event readiness, or fresh
Rust/Miri verification. Native Terminal launch was unavailable in the execution
environment; the launch command remains directly runnable in an interactive terminal.
