# PQ-86: Delete JoinRefusal::NotYetOpen and amend the join-failure count to five

Ruling (Michelle, 2026-11-05 19:19Z): "delete it and amend the spec, since joining early and waiting is how the room works and nobody has asked to refuse early arrivals." The variant has never been emitted: joins are accepted while the room is idle, and room/README.md:472 records that no phase of the seven reaches it. PQ-50's delegator raised the question on 2026-11-02 (plan task_01M431PWQ4QF5KBCENBTM9YW02); it stayed open until this ruling.

Scope, code:
- room/src/connections/sessions.rs:59-99: drop NotYetOpen from the JoinRefusal enum, ALL, slug() and message().
- room/src/copy.rs:33-34 and :159: drop JOIN_FAIL_NOT_YET_OPEN and its table entry.
- room/tests/sessions.rs:170-179: six states become five; drop the NotYetOpen assertion and fix the comment.
- web/shared/copy.js:74 and :273: drop join_fail_not_yet_open and its entry in the section map.
- web/buzzer/buzzer.js:41: drop the not_yet_open slug mapping.
- web/test/buzzer.test.js:174-175: five slugs and five keys.
- room/README.md:472: drop the row.
No other sentence changes (AC-98 copy freeze). The room/web copy-agreement test must stay green with one fewer entry.

Scope, contract (allowed by this ruling only; the build never edits these files otherwise):
- SPEC.md:185 (section 4.1): six failure states become five; drop "not yet open".
- SPEC.md:629 (section 11, "Buzzer, join failures (AC-29)"): drop the "not yet open" sentence.
- EVALUATION.md:91 (AC-29 row): "six distinct failure states" becomes five.
- sequence/USER_STORIES.md:130 (AC-29): drop "not yet open" from the list.
AC-29 keeps its ID and its meaning (every room-code failure states what went wrong and what to do next); only the enumerated list shrinks. PRD.md:246 (the superseded ready/lobby lifecycle) stays as it is.

Acceptance (AC-29, AC-98; harness: just test):
- rg -n "NotYetOpen|not_yet_open|NOT_YET_OPEN|not yet open" over the repo, excluding .lattice/, .git/, target/ and PRD.md, returns nothing.
- just test green; the section 11 table, copy.rs and copy.js agree on the remaining five join failures.
- The commit message or PR body cites this ruling by time.

Held: not released. Michelle names it for a batch.

# Plan (delegator, 2026-11-05)

Paths are `main`'s (`popquiz/`), verified at 7dda474. Ruling: Michelle, 2026-11-05 19:19Z.

## Edits

Room (commit 1):
- popquiz/src/connections/sessions.rs:59 doc "six failure states" -> "five"; drop variant :64, ALL entry :74 and `[JoinRefusal; 6]` -> `5` :71, slug arm :85, message arm :97.
- popquiz/src/copy.rs:37-38 drop JOIN_FAIL_NOT_YET_OPEN; :164 drop its ALL table entry. ANSWER_NOT_YET_OPEN (:281) stays.
- popquiz/tests/sessions.rs:170-179 comment -> five states, the one no phase reaches (closed for inactivity, T-11's); `(6, 6)` -> `(5, 5)`; drop the NotYetOpen assertion.
- popquiz/tests/reason_copy.rs:49 drop "not_yet_open" from REFUSAL_CODES (every_refusal_code_is_still_sent fails otherwise, since the literal leaves src/connections).

Web (commit 2):
- web/shared/copy.js:71 comment six -> five; :74 drop key; :274 drop it from the section map.
- web/buzzer/buzzer.js:36 comment "six refusals" -> five; :41 drop slug mapping.
- web/test/buzzer.test.js:2, :171 comments six -> five; :174-175 five slugs, five keys; :189 size 6 -> 5, "five distinct messages".
- web/test/copy.test.js:56 join-failure row count 6 -> 5 (a pinned count the boot's scan did not list; count only).

README (commit 3): popquiz/README.md:472 drop the row.

Contract (commit 4, the four lines the ruling clears):
- SPEC.md §4.1: drop **not yet open** from the list (the word sits on line 184 of the sentence that runs 183-185).
- SPEC.md:629 §11 row: drop the "**not yet open:** ..." sentence and its " · " separator; every other sentence unchanged.
- EVALUATION.md:91 "six distinct" -> "five distinct"; drop "not yet open".
- sequence/USER_STORIES.md:130 drop "not yet open" from the list. AC-29 keeps ID and meaning.

## What pins each layer
- Rust enum/slugs: popquiz/tests/sessions.rs (5,5); reason_copy.rs every_refusal_code_is_still_sent.
- Rust <-> JS copy: popquiz/tests/twins.rs the_copy_module_mirrors_copy_js_in_both_directions.
- JS rows: web/test/copy.test.js (row count; every entry in a row; every row key resolves).
- Buzzer map: web/test/copy-freeze.test.js (drives every REFUSAL_KEY slug and expects its join_fail_ key in the render); buzzer.test.js AC-29/30.
- SPEC §11 text vs copy: no test reads the §11 join sentences (copylint reads only the Forbidden row and §11.1); held by review.

## Mutations (each reverted)
1. not_yet_open slug left in buzzer.js -> expect copy-freeze.test.js (REFUSAL_KEY loop) or copy.test.js to fail; else add the smallest test.
2. join_fail_not_yet_open left in copy.js while copy.rs lost it -> twins.rs the_copy_module_mirrors_copy_js_in_both_directions (and copy.test.js row count if left in the map).
3. ALL left at six entries -> compiler (variant gone, so the ALL entry cannot name it; with `[_; 5]` and 6 entries, a type error).
