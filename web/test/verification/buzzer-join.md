# Buzzer join hardening verification

Initial verification on 2026-10-02 in worktree
`/Users/michellerojas/.codex/worktrees/e8da/rust-nyc-pop-quiz`, branch
`codex/harden-buzzer-join`. The initial reviewed head was `01de92d`.

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

## Review-fix verification

The initial PR head was `01de92dc166f5ce702e4d51d7951275599a9bcda` on
`codex/harden-buzzer-join`; `e8da` above identifies the worktree directory, not
its Git revision. GitHub CI on that initial head passed `just test` and
`just test-full` before these review changes.

The added recovery-feedback regression failed before the review fix and passed
afterward. The updated local `UV_PYTHON=3.12 just test` exited 0: 275 web tests,
634 pipeline tests, and 235 Rust tests passed, with four deliberately ignored
Rust tests and the same existing warnings. The Impeccable detector returned no
findings; `git diff --check` was clean.

A fresh browser pass confirmed the revised message, retained code and focus,
`data-error="join-connection"` with no server-refusal marker, and no horizontal
overflow on desktop or a 320px phone viewport. The message recommends waiting
before retry but does not enforce a Retry-After delay.


## Known limits and deferred server work

**Unconfirmed joins can consume sessions.** `AppState::join` creates a session
under the room lock before the response reaches the browser; `SessionMap::join`
checks capacity against the session map and issues a new token on every join.
The browser deadline and abort do not roll back that server operation. If the
body is slow, the connection fails after allocation, or the abort arrives after
allocation, the browser can discard a successful response without ever storing
the token. A retry can allocate a second session. An unattached orphan can occupy
capacity and inflate the room's present count without adding an answer. Repeated
failures may cause a full-room refusal. Room release clears the session map;
there is no idempotent join recovery in this change. The late-response guard
protects browser state only.

This is the review's permitted documentation path, not a claim that the risk
has been fixed or that an organizer has accepted it for a live event. Proper
recovery would need a per-attempt client nonce with server-side deduplication
and an explicit lifetime, or cleanup of sessions that never attach. Deduplication
by room code alone would conflate different attendees joining the same room.
That protocol work is deferred from this UI hardening PR.

**The deadline is a timer target, not a wall-clock guarantee.** Background mobile
tabs can throttle timers or suspend JavaScript, keeping the join button disabled
beyond ten seconds until execution resumes. Foreground deadline behavior was
verified; physical-device background suspension was not. The settled guard still
prevents a late response from replacing a newer attempt.

**Error wording deliberately groups unconfirmed outcomes.** Network failure,
429, 5xx and malformed success bodies share "Couldn't finish joining". This
makes no claim that the request failed to reach the server. "Wait a moment"
is guidance only: the browser does not enforce server Retry-After or automatic
backoff for joining.

**Copy provenance has a manual step.** The new wording is authored in SPEC.md
section 11 and manually transcribed into the JS and Rust copy modules. The
`twins.rs` test enforces equality between those two modules; no test parses the
SPEC table or proves that transcription against the prose.
