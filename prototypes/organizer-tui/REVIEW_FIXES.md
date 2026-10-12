# PR 64 review follow-up

Review target: `8303c2e`. Follow-up: 2026-10-11.

| Finding | Change or disposition |
| --- | --- |
| Arrows type into scheduling fields | The entry point passes a `KeyEvent` to the app. Scheduling receives `KeyCode`: Up/Down select fields, Left/Right are ignored, and page keys scroll. Printable text stays distinct. |
| Ctrl/Alt chords trigger plain-letter actions | Modified events are filtered before dispatch, except explicit Ctrl-C. Tests cover Ctrl/Alt-R/S/Q and Ctrl/Alt-Y at confirmation. |
| Reveal while stepping backward | Left from reveal returns to the teaching view and hides the answer and receipt. |
| Follow-up: safeguard reveal | One `r` arms reveal and Enter confirms; Escape or another key cancels. Repeating `r` leaves it armed without revealing. Help and controls document the distinct confirmation key. |
| q3-specific labels | All candidates now derive labels from the first sentence of the authored note. Editing a note changes its label; the bank itself is unchanged. |
| Failure heading after successful PUT | Nonzero scheduler exits say “Scheduler reported a problem.” Details retain the scheduler's partial-success or refusal message. |
| Confirmation summary | Date, room, and output path already appear before the conditional confirmation text. A new screen test asserts all three remain visible after Enter. |
| In-repository fallback path | Resolve existing path prefixes, symlinks, and parent components before confirmation and reject paths within the repository. The scheduler independently rechecks when submitted. |
| Result scroll past content | Clamp to the wrapped content and current viewport, including after resizing. Page keys also make long confirmation text accessible. |
| Legacy terminal auto-repeat bypass | Use `r` then Enter on every terminal. Request event-type reporting where supported and restore the previous protocol on exit. CI runs PTY regressions with legacy repeated bytes and enhanced repeat events; neither can reveal without Enter. |
| Local username in design QA | Removed the absolute local path from the reference description. |
| Crate omitted from CI | Added `test-organizer` to `just test`, dependency fetching to `just setup`, and the crate target/lockfile to CI caching. |

PR 66 was still open when checked. Current main continues to export
`room::answers`; this follow-up does not copy or preempt its proposed reorganization.
The new CI coverage builds the TUI against the room source in the tested checkout,
so a future incompatible API move will fail that check.
