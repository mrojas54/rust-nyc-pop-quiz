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
