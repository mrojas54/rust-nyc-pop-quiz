# Organizer TUI prototype validation

## Review fixes — 2026-10-11

RED commit `237c863`; GREEN commit `7e7cef4`.

- `just test` passed, including the newly added `test-organizer` dependency.
- TUI: 26 tests (11 unit, 15 integration), Clippy with warnings denied,
  and formatting checks passed. The path test initially assumed macOS's
  per-user temporary directory was `/tmp`; its expected canonical path was fixed.
- The full recipe also passed the room and web suites and 710 pipeline tests
  (the existing q3 copy warning remains). No container or deployed tests ran.
- Real PTY smoke against a loopback server exercised arrow escape sequences,
  Ctrl-Y, and Alt-Y before plain `y`. No request was sent until plain `y`;
  exactly one PUT followed, fallback files were created, and the temporary bank
  remained unchanged. The test token did not appear in terminal output.
- Refreshed the teaching navigator and scheduling recordings. Inspected the
  note-derived navigator labels in the rendered terminal capture.

See [REVIEW_FIXES.md](REVIEW_FIXES.md) for each finding and its disposition.

## Scheduling — 2026-10-11

RED commit `44387fd`; GREEN commit `4f3b6e0`.

- Rust: 20 tests passed (7 unit, 13 integration); Clippy with warnings denied
  and formatting checks passed.
- Python, from `pipeline/`: all 66 scheduling tests passed; the full
  non-container suite passed 710 tests with the existing q3 copy warning.
- Real binary smoke check: entered the schedule form over a pseudo-terminal,
  filled its fields, reviewed the confirmation, and pressed `y`. The existing
  Python scheduler sent exactly one PUT to a loopback test server and wrote
  the fallback. It sent no request before confirmation. The temporary bank
  remained byte-identical, the test token did not appear in terminal output,
  and the TUI exited cleanly.
- Inspected rendered confirmation and success screens at 110 × 36. The first
  smoke assertion incorrectly searched the incremental terminal byte stream
  for a whole rendered line; status was instead checked in the rendered capture,
  with HTTP requests and fallback files checked directly. The final smoke passed.

The smoke used a temporary copy of q3 with fixture review metadata and a local
test server. No production request, real affirmation, or new compiler/Miri
verification occurred. Python implementation files were unchanged. The TUI now
has an explicit write action, so the earlier read-only description applies only
to browsing. Scheduling does not alter bank records or sync the used ledger.

## Step navigator — 2026-10-10

RED commit `4d5387a`; GREEN commit `6d8579e`.

From `prototypes/organizer-tui/`, after the final source changes:

- `cargo test --locked`: 15 passed (4 unit, 11 integration).
- `cargo clippy --locked --all-targets -- -D warnings`: clean.
- `cargo fmt --check`: clean.

The four new tests failed before implementation. They cover selecting versus
opening a step, focus and Escape behavior, blocking reveal through list navigation,
and help without changing the underlying state. Existing isolation and rendering
tests continue to pass.

A 136 × 41 pseudo-terminal recording exercised selection, Enter, Tab through all
panes, stepping, explicit reveal, scrolling, help, and clean exit. The recording
is about 35 seconds. See [design-qa.md](design-qa.md) for the visual comparison.
Only the prototype checks were rerun; no new bank verification is claimed.

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
