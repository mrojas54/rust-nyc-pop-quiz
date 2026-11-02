# PQ-81: An edit can turn a checked quote into a claim about output without unchecking it

Found by the PQ-72 delegator (PR #109, merged 0deed94); client ruled a ticket on 2026-10-31. Today a quotes_checked attestation lasts as long as the span's text does, and it ignores the surrounding sentence and the program. So an edit can change the context so a previously checked span becomes a real claim about what a program prints, while the span stays checked. Mitigation today: any edit clears the affirmation and the review screen labels the span for whoever re-affirms. Not a contradiction of AC-73 as written: a mismatch still blocks unless that exact span was attested. Pinned behaviour: test_ac73_a_checked_quote_lasts_exactly_as_long_as_its_text in pipeline/tests/test_review.py, and the comment in pipeline/src/popquiz/review.py; both need changing if behaviour does. Decide first: what should invalidate an attestation (sentence changed, program changed, or any edit), keeping House rule AC-7 (never write down what a program prints). Acceptance: the chosen trigger unchecks the span, shown failing under its mutation; just test green; if AC-73's wording needs amending, that goes to the client.

# Plan (delegator, 2026-10-31)

Every path that writes a question, and whether it clears `quotes_checked`:

| Path | Changes content? | Clears? |
|---|---|---|
| `review.edit`, no program change (prose, hint, why_tempting, trace notes) | yes | **yes** (new) |
| `review.edit`, re-verify path (`_verify_into_bank`) | yes | **yes** (new; `verify.with_record` keeps `review`, so the clear survives) |
| `review.edit`, nothing changed (`edited == question`) | no | no; returns before writing |
| `review.edit`, verifier refuses or edit invalid | nothing written | no |
| `review.reverify`, what it records differs (trace `stdout` row, `verified.stdout`, compile code) | yes | **yes** (new; ruling 2) |
| `review.reverify`, same output | no (only timestamps) | no |
| `accept`, `reject`, `judge_difficulty`, `affirm` (`_save` callers) | no | no |
| `export` | never writes the bank | n/a |
| `schedule.py:541` sync | adds a `used` record only | no |
| `migrate_mvp` | never overwrites a held file | n/a |
| `generate`, `dedupe` | no `save_question` call in either | n/a |
| Organizer TUI | shells out to the CLI above | inherits |

`_cleared()` is the affirmation clear and is also used by `reject`, so it stays as it is. Both edit paths flow from one line (`review.py:459`), so the clear goes there, ahead of the branch.

Change: `quotes_checked=None` at that line; replace the comment; `reverify` clears when what it recorded differs. No string changes. `bank.py` docstring and `pipeline/README.md` edit row state the rule.

Tests, each with its mutation:
1. `test_ac73_any_edit_unchecks_a_questions_checked_quotes`, parametrized: hint, explanation sentence, why_tempting, trace note. Mutation: restore carry-over on the no-program-change path.
2. Same rule, parametrized through the re-verifier: option text, program. Mutation: restore carry-over on the re-verify path.
3. `test_ac73_a_no_change_edit_keeps_its_checks`. Mutation: clear before the no-change return and write.
4. Edit, affirm refused, accept with the check, affirm succeeds, on both paths. Mutations 1 and 2.
5. `test_ac73_a_removed_quote_does_not_come_back_checked`. Mutation 1.
6. Reverify: changed output clears, same output keeps. Mutations: never clear, always clear.
7. Decisions keep the checks (reject, difficulty, accept). Mutation: `_save` clears.
