# Buzzer join hardening verification

Verified on 2026-10-02 in the e8da worktree.

## Regression evidence

Before implementation, `node --test web/test/buzzer.test.js` passed 34 tests
and failed the three added regressions: overlapping submits changed the code,
transport failures had no recovery status, and pending joins had no deadline.
After implementation, all 37 buzzer tests passed. The deadline test also confirms
that an old successful response cannot replace a newer pending join.

## Default repository suite

`UV_PYTHON=3.12 just test` exited 0 after preparing this worktree's locked
Python environment with `UV_PYTHON=3.12 uv sync --offline --frozen` in `pipeline/`.

- Web: 274 passed.
- Pipeline: 634 passed; one existing q3 hint prose warning.
- Rust, including doctests: 235 passed; four deliberately ignored tests.
- Existing Rust warnings: three unused imports in `tests/host_page.rs`.
- `git diff --check`: clean.
- Impeccable detector, changed product JS: empty findings array.

Initial runs failed because sandbox permissions blocked loopback/cache access,
then because the shell selected Python 3.13, then because the new Python 3.12
virtual environment needed its locked test dependencies. The successful run
used local socket/cache permissions and the prepared Python 3.12 environment.
No Python source changed; this project does not configure ruff or mypy.

## Browser evidence

Used the Codex in-app browser and the existing fixture harness at
`/web/test/a11y/browser.html?surface=buzzer` served from this worktree.

- A rejected join visibly shows recovery guidance, retains ABC234, restores
  focus to the code field, and enables the join button.
- Desktop and a narrow phone viewport both wrap the error without horizontal
  page overflow (observed CSS viewport widths 964 and 291 pixels).
- A stalled join disables the submit button, returns to a retryable error
  after its deadline, and succeeds on the next attempt.
- The error is linked to the field with `aria-describedby` and has `role=status`;
  the pending form has `aria-busy`.

The harness injects transport failures; it does not contact the deployed room.
No physical phone, VoiceOver, Safari, real network throttling, container/Miri
suite, or deployed burst check was run for this change. No CSS or visual identity
changed. Timeout aborts fetch where AbortController exists and ignores late
responses in all supported injection paths.
