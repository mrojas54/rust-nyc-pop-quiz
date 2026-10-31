# PQ-72: PQ-23 follow-ups: re-verify-path tests, quotes_checked docs, review.rs nits

Nits 3–5 from the orchestrator's exact-head review of PR #80 (comment on PQ-23). Nits 1–2 were fixed in PR #81 (26cccc0).
3. Tests: rejected-stays-rejected is covered only on the no-program-change edit path. Add the re-verify path (_verify_into_bank) and the unreviewed case; add an identical-content edit of an affirmed question.
4. review.py: quotes_checked survives edits while the span text survives. Probably intended (AC-73 keys on the span) but undocumented: document it.
5. review.rs:235 passes the id positionally with no `--` (real ids are q<N>); review.rs:321 sets `changed` on "Not recorded" (harmless reload).
Also deferred by the PQ-23 delegator (triage, not required): multi-blocker sentences, newline in --by, predictable editor temp path, panic-unsafe env cleanup, absolute git-status guard, Esc can't cancel re-verify.

# Plan (delegator, 2026-10-30)

Base 9c718c3. No visible string moves. Nothing outside the cleared files.

## Item 3: tests, `pipeline/tests/test_review.py` (edit tests, near line 337-366)

Each new test uses an option edit through `StubRunner`, and asserts the output says `re-verified` and `verified_at` moved, so it cannot pass on the no-program-change path.

| Test | Pins | Mutation |
|---|---|---|
| rejected through re-verify | status and reason kept, "stays rejected", not accepted, affirm refused | `edit`: status becomes `edited` for a rejected record (review.py:452) |
| unreviewed through re-verify | status stays unset, "not accepted yet", affirm refused | `edit`: unset status becomes `edited` (review.py:452) |
| identical edit of an affirmed question | bytes, status `accepted`, affirmation `bound`, still schedulable, verifier never called | drop the `edited == question` return (review.py:445) |

Line 360's existing test already covers the bytes and the message for the identical edit; the new one adds the affirmation, the status and the no-verifier check. A real bug exposed here gets reported by mail, not fixed.

## Item 4: `quotes_checked`, `pipeline/src/popquiz/review.py` (comment only)

Confirmed by reading: `edit` carries `prior` forward with only the status and the affirmation changed (review.py:453), and `quotes` compares the exact span text (review.py:101-103). AC-73 says a mismatch blocks acceptance; the attestation is the established exception and covers one exact span, so it does not contradict AC-73.

The comment goes at the `edit` line that carries the field. `pipeline/README.md` does not name the field, so it stays unchanged.

One test backs the comment: a checked span stays checked through a prose edit and through a re-verify, and a reworded span blocks. Mutations: `edit` clears `quotes_checked`; `quotes` treats any recorded check as covering every span.

Contract note for DONE: the attestation does not look at the sentence around the span or at the program, so an edit can turn a checked span into a claim about output without unchecking it. Any edit still clears the affirmation, and the screen labels the span "not a printed line, checked" for whoever re-affirms.

## Item 5: `prototypes/organizer-tui/src/review.rs`

**`--` (`command`, line 235).** Tried on the real parser (Python 3.12.13) against a temp copy of the fixtures: `show --bank B -- q4`, `difficulty --bank B -- q4 3`, and the export, accept, reject, affirm and edit forms all parse, and `show --bank B -- -x` reaches the bank as the id. `show -- q4 --bank B` fails, because everything after `--` is positional. So every option moves before `--`, and the id then the difficulty level go last.

- Unit test pins the argv of every job. Update the two existing argv assertions, which pin the old order. Mutations: drop `--`; put `--bank` after it.
- `tests/review.rs` walk gains difficulty and reject, the two forms it does not drive, so the real parser sees every form. Mutation: the level before `--`.

**`changed` (`poll`, line 321).** Set only when the CLI succeeded, and never cleared: an accept followed by a refused affirm must still reload on close.

- Unit tests drive `poll` through a channel: "Not recorded" alone leaves `changed` false; "Recorded" then "Not recorded" leaves it true. Mutations: the old `self.changed = true`; `self.changed = out.ok`.

## Not done (rulings, out of scope)

Multi-blocker sentences, newline in `--by`, predictable editor temp path, panic-unsafe env cleanup, absolute git-status guard, Esc cannot cancel re-verify.
