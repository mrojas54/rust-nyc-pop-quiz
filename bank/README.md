# bank

The question bank: one JSON file per question, reviewable in a pull request,
diffable, with no server in front of it (BUILDPLAN D-C).

Empty today. **T-14** defines the format (SPEC.md 3.1–3.3) and migrates the
eight MVP questions, and **T-19** writes `bank-audit` over what lands here.

## This directory is private, and that is load-bearing

Nobody in the room has seen the question. A public bank would end the segment as
a thing worth doing, so the repository stays private and nothing here is ever
published or linked.

## Two rules for whoever fills it

**The correct answer is written by the verifier, never by a person.** It comes
from a `verified` record that the verifier produced by actually running the
program. A hand-edited answer field fails the build's provenance check (AC-7,
G-2). If you find yourself reasoning about what a program prints, stop and run
it.

**Answer position is a pure function of the date, and nothing else.** No
history, no ledger, no bank state, no "not the same letter as last time". Every
balancing rule hands an attendee a free elimination, and an enforced balance
makes every fifth meetup certain. Uneven counts and repeated letters are the
correct output, not a bug to fix — `mvp/answer-history.json` is a record, never
an input (AC-23, AC-23a, AC-23b, G-1).

That second rule has regressed four times, once while somebody was fixing the
previous regression. Read `PHILOSOPHY.md` section 2 before touching the slot
path, and expect the audit ticket to come with you.

## `bank/audit/`

`bank-audit` writes its report under `bank/audit/`. Nothing participant-facing
reads that directory — the tell audit necessarily names answer categories and
their frequencies, which is exactly what AC-25 keeps away from attendees.
