# PTY state waits — verification, 2026-10-18

Base: `7573efa`. GREEN implementation: `616ec99`.

The PTY harness previously checked reveal output after a fixed 120 ms wait.
A stopped child resumed after 350 ms reproduces the failure in both legacy
and enhanced keyboard modes. The harness now waits up to three seconds for
each expected state, draining output while it waits. Failure messages include
the mode, expected state, elapsed time, child exit status and captured output.

## RED

Commit `7395426` adds the regressions without replacing the fixed wait.
`python3 -m unittest discover -s prototypes/organizer-tui/tests -p 'test_terminal_repeat.py'`
failed in both delayed-child subtests. The dropped-Enter case also failed because
the old assertion did not provide the required timeout evidence.

The delayed-child fixture confirms SIGSTOP before starting its 350 ms resume
timer. The dropped-Enter fixture removes only the Enter write; it leaves the
real binary, PTY and remaining input handling intact.

## GREEN and VERIFY

Run from the isolated checkout root unless a different directory is named:

- The regression command above passes both test methods, including both
  keyboard-mode subtests. The missing reveal still fails within the deadline
  and reports terminal evidence, as expected by its test.
- `python3 prototypes/organizer-tui/tests/terminal_repeat.py` passes both modes.
  Repeated `r` does not reveal; Enter reveals; the child exits cleanly; enhanced
  keyboard reporting is restored.
- `just test` passes on the GREEN commit in **29.475 seconds**: 953 pipeline
  tests, 303 web tests, the Rust suites, and both PTY commands. The existing
  q3 prose warning remains. Clippy and Rust formatting checks pass in this run.
- From `pipeline/`, Ruff 0.17.1 `check` and mypy 2.4.0 `--strict` pass for
  `../prototypes/organizer-tui/tests/terminal_repeat.py` and
  `../prototypes/organizer-tui/tests/test_terminal_repeat.py`.
- `git diff --check` passes.

Environment: macOS, Node 22.22.3, Python 3.15.0 for standalone PTY checks,
Python 3.12.13 for the pipeline, cargo 1.96.1.

One exact-diff self-review checked deadline handling, output draining during
shutdown, timer cleanup, preservation of the negative reveal assertions, and
recipe integration. No outstanding finding. No product or Rust source changed.

This proves the controlled timing regression and the local suite. It does not
establish the cause of the original uninstrumented audit failure. F-46 and its
contrast ruling are outside this change. Docker/Miri checks are left to the
PR's `just test-full` CI job; they were not run locally.
