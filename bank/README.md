# bank

The question bank: one JSON file per question, reviewable in a pull request,
diffable, with no server in front of it (BUILDPLAN D-C).

| Path | What it is | Written by |
|---|---|---|
| `questions/<id>.json` | one question — `SPEC.md` §3.1, with its §3.2 `verified` record | the pipeline; **T-14** migrated the four that are here |
| `schema/question.schema.json` | the published shape of that file | by hand, kept in step with the types by a test — see below |
| `history.json` | dedupe's memory across runs: exact hashes, normalized-AST fingerprints, token-bigram sets (§3.3) | **T-17**. The three stores are empty; T-14 fixed the shape |
| `fixtures/receipts/*.json` | the receipt's cases, with the lines each must render | by hand from `SPEC.md` §7.5 |
| `audit/` | `bank-audit`'s run reports, organizer-only and gitignored — see [below](#what-bank-audit-checks-and-why-it-can-never-touch-a-real-night) | **T-19** |

## This directory is private, and that is load-bearing

Nobody in the room has seen the question. A public bank would end the segment as
a thing worth doing, so the repository stays private and **nothing here is ever
published or linked** — not the questions, not the fixtures, not the audit report.

## Two rules for whoever fills it

**The correct answer is written by the verifier, never by a person.** It comes
from a `verified` record that the verifier produced by actually running the
program. If you find yourself reasoning about what a program prints, stop and run
it.

There is no `correct` field to edit. `SPEC.md` §3.1 marks the correct answer
*derived* (G-2), so `bank.correct_index()` computes it — for a does-not-compile
question, the single `does_not_compile` option; for UB that Miri reported under
both borrow models, the single `ub` option; for a question that panicked, the
single `panic` option; otherwise the option whose text equals `verified.stdout`.
A record with nowhere to write an answer down is a record nobody can hand-edit
one into (AC-7).

**Answer position is a pure function of the date, and nothing else.** No
history, no ledger, no bank state, no "not the same letter as last time". Every
balancing rule hands an attendee a free elimination, and an enforced balance
makes every fifth meetup certain. Uneven counts and repeated letters are the
correct output, not a bug to fix — `mvp/answer-history.json` is a record, never
an input (AC-23, AC-23a, AC-23b, G-1).

That second rule has regressed four times, once while somebody was fixing the
previous regression. Read `PHILOSOPHY.md` section 2 before touching the slot
path, and expect the audit ticket to come with you.

### The order options sit in here is not the order a room sees

Letters and the arrangement on the wall are drawn from the meetup date
(`slot_for_day`). The order in these files exists so that two runs of a writer
produce the same file, and **nothing may read a tell out of it** — not the
position of the answer, not the position of *does not compile* (AC-26).

## What is in the bank, and what is not

`T-14` migrated the eight verified MVP questions as far as the built wall can
take them. Four are here; four are not. `SPEC.md` §5.2's fit table, worked at the
guessed 15 ft / 20 ft room, is why.

| Question | Here? | Why |
|---|---|---|
| **q3** Collections | **yes, as authored** | 5 source lines, and all five options are one line of ≤ 29 characters — the only one of the eight that passes D-15 unchanged |
| **q4** Integer arithmetic | **yes, needs work** | fits as a program (6 lines); 4 of 5 options are two lines |
| **q7** Sorting | **yes, needs work** | fits as a program (5 lines); 4 of 5 options are one line of 48 characters |
| **q8** Borrow checking | **yes, needs work** | fits as a program (6 lines); 1 of 5 options is two lines |
| **q1** Ownership and Drop | no | 16 source lines |
| **q2** Iterators and closures | no | 10 source lines |
| **q5** Strings and UTF-8 | no | 9 source lines |
| **q6** Bindings and shadowing | no | 9 source lines |

The four that are absent exceed the reading layout's **7-line capacity at the
floor** — the wall cannot show them at a size the back row can read. They stay in
`mvp/2026-08-12/` and nowhere else, and they come back when they are re-authored
shorter, or when the rehearsal's screen and back-row measurements (H-5) raise the
floor and with it every number in §5.2.

**"Needs work" is not one thing.** An option's text has to equal what the program
printed, so a too-long *correct* option cannot be fixed by rewriting the option —
the program has to print something shorter, and then the machine decides the new
answer (T-15b). That is q4 and q7. q8's too-long option is a *distractor*, so its
answer stands, its program need not change, and only that one option needs
replacing. Each record's `review.reason` says which case it is, generated from the
measurement rather than typed.

**Nothing here is affirmed.** Every migrated record has `review.status` unset and
no `affirmed_by` / `affirmed_at`, so none of them can reach a deck (AC-72, G-12).
The beats and q3's trace were *drafted* — from the August explanation and the
prototype the client loved — and are waiting for an organizer to read them.

## How a record is verified

`just verify <candidate.json> --expect ran|does_not_compile|ub` runs
`pipeline/src/popquiz/verify.py` on the pinned image (`just sandbox-build` first)
and, if the candidate is accepted, writes its `verified` record into the file. The
declared answer is an argument, not a field: it is never stored, and a file that
already has a record recovers it from there. Exit 0 accepted, 1 rejected (file
untouched), 2 refused.

In order, each step only if the one before passed:

1. **The pin.** The image's `rustc -Vv` release and commit-hash and its
   `cargo miri --version` must equal `pipeline/sandbox/pin.toml`, and its host
   triple must be one the pin lists. Otherwise the verifier refuses to run at all.
2. **Compile** under the pin's edition and flags. Declared *does not compile*: it
   must fail with at least one `error[Exxxx]`, and every code is recorded;
   compiling is a rejection. Anything else that fails to compile is rejected.
3. **Five native runs**, each its own container. Stdout and exit code identical
   every time, or rejected with the count of distinct outputs (AC-8). Exit 0 or a
   panic (101) is an answer; any other exit is rejected.
4. **Miri, Stacked Borrows, strict provenance.** UB not declared: rejected. Miri's
   stdout or exit different from the native run: rejected (AC-10). Miri unable to
   run it (an operation isolation refuses): rejected.
5. **Tree Borrows**, for declared UB only; accepted only if both models report UB
   (AC-9), and Miri's output before the UB must equal the native output exactly —
   so a UB question prints only before its UB.

Every field of the record is read off a step or the pin, never typed. A
re-verified legacy record is replaced whole. `verify.is_stale(record, pin)` is
what scheduling asks: a record made under a different pin re-verifies first; a
legacy record is exempt. `verify.check_provenance(file)` is the build's AC-7
check: it refuses a stored `correct` field, a record the verifier did not write,
and options that no longer derive an answer from the record. It cannot see an
edit *inside* the record that leaves it self-consistent — nothing in §3.2 binds a
record to the run that produced it — and says so.

The four migrated questions below still carry their `legacy` records; they were
not re-verified on T-15b. That is an organizer step (T-20), and q4 and q7 need
their programs re-authored first.

Miri checks only the paths the program executed (AC-43).

## Where the migrated records came from

`pipeline/src/popquiz/migrate_mvp.py`, run once, output committed. Re-running it
reproduces these files byte for byte, and a test asserts that, so you can
regenerate them rather than take them on trust.

The records are **`legacy`** (`SPEC.md` §3.2, D-16): they hold only what the
August batch recorded — `rustc` as the one-line `--version` string, `edition`,
`runs`, `stdout`, and a Miri result with no version, no borrow models and no
seeds, because that pass ran outside the repo. `target_triple`, `flags`,
`exit_code`, `verified_at`, `verifier_version` and the full `-Vv` are **absent,
and never back-filled or inferred**. In the files that absence is a *missing key*,
never a key holding `null`: a null is a slot somebody could quietly fill, and the
absence is the whole proof that nothing was invented (G-2). Tests assert the
absence rather than the presence of a null.

A legacy record renders the same step list as a complete one (§7.5, D-25). Its
take-it-home page is where it says what it lacks — the target reads *not
recorded* and the Miri row says the check was run separately (§13, AC-87). When
T-15b re-verifies one of these questions the record is **replaced whole, never
merged**, so the bank never holds half of each.

q8's record carries `compile_error_code`, renamed from the MVP's `error_codes`
with the codes unchanged, and no `runs`, `stdout` or `miri` — nothing ran (D-22).
The MVP's `miri` field for q8 reads `n/a (does not compile)`, which is not a
result, so no Miri record was written rather than one implying it was fine.

### Two deliberate departures from `SPEC.md` §3.1

**`why_tempting` rides on the option it describes**, rather than under `explains`.
A beat keyed by letter dangles the moment the date draws a different slot — the
prototype keyed one to `"A"` — and a beat keyed by position silently attaches to
the wrong option the first time somebody reorders them. On the option there is no
key to go stale, and AC-95's *every incorrect option has a non-empty
`why_tempting`* becomes a structural property instead of a join that can miss.

**The MVP's `stderr_head` is not carried.** It is a real machine artifact, but
§3.2 does not list it and `compile_error_code` is what the receipt reads. Dropped
rather than smuggled in under an invented key; noted here so the loss is on the
record.

## The receipt fixtures, and why they live here rather than in the tests

`SPEC.md` §7.5's receipt is needed in **two languages** — Python for the pipeline
and review surface, JavaScript for the wall and take-it-home (T-05). The contract
says one function; two languages cannot share one. So the cases are data:

    fixtures/receipts/<case>.json   { "_note": …, "verified": …, "expected_lines": [ … ] }

Both implementations test against **these same files**, which is how the build
keeps them one function rather than two that agree today and drift next month.
`expected_lines` is `null` for a record that renders no receipt at all — not `[]`,
which would mean a receipt with no steps, a different and wrong answer.

The `expected_lines` were written **from `SPEC.md` §7.5 by hand**, not recorded
from the implementation. A fixture whose expectations came out of the code it tests
proves only that the code equals itself.

**Two of the nine cases are real; seven are synthetic, and each says which.**
`legacy-ran` (q3) and `q8-does-not-compile-legacy` are copied from
`mvp/2026-08-12/verified.json`, which a machine wrote. The rest describe **no Rust
program** — they exist to exercise the receipt's gates, which never read
`stdout`'s content, only whether a step is present — and each carries a `_note`
saying so in its own text, following the precedent of
`pipeline/tests/fixtures/scaffold-placeholder.json`. That is a judgment call
against the house rule, made openly: a placeholder in a record that claims to be
no program's output is not a program's output written down.

## The schema is a contract plus a test, not a validator

`schema/question.schema.json` is the published shape. Each `$defs` name is the
dataclass of the same name in `pipeline/src/popquiz/bank.py`, and
`pipeline/tests/test_bank.py` walks the two against each other **in both
directions** — a field missing from the schema publishes an incomplete contract, a
property left behind documents something the code now refuses — plus the required
sets and the enums.

**Nothing validates records against it at runtime.** There is no JSON Schema
validator in the Python standard library and `pipeline/pyproject.toml` belongs to
another ticket, so adding a dependency was not T-14's call. Records are validated
by being loaded through `bank.question_from_dict`, which refuses unknown fields
and enforces the structural rules (five options, exactly one *does not compile*,
no duplicate option text, and a trace that ends on `stdout` when the question
ran). T-19 did not add one either: `bank-audit` checks AC-24's shape on each raw
file, so a malformed record is reported by name, and loads the rest through
`question_from_dict`.

## What `bank-audit` checks, and why it can never touch a real night

`just bank-audit` runs the whole bank against `SPEC.md` §7.6 and writes
`bank/audit/<date>.json`. The same checks run inside `just test`. The code is
`pipeline/src/popquiz/audit.py`.

**It cannot move tonight's answer.** Where the correct answer sits is decided by
`slot_for_day(date)` in `pipeline/src/popquiz/slot.py`. That file imports nothing
but `hashlib` and `datetime`, keeps no state, and takes the date and nothing else.
The audit checks all of that on every run. The audit calls `slot_for_day` only on
made-up nights — 20,000 of them starting from 2026-08-12 — and looks at what comes
out. It never reads a real meetup, never reads which questions were used, and never
hands anything back. It measures the dice; it does not roll them. That matters
because this rule has broken four times, and the worst break was a "fairness"
rule: once a check can change tonight's letter, an attendee can run the same check
(`PHILOSOPHY.md` §2).

What it checks:

| Check | Criteria | What it measures |
|---|---|---|
| The generator | AC-23b | 20,000 made-up nights must look random in **both** directions. Too lopsided means the generator is broken; too even means somebody is balancing it, which is the exploitable failure. It must also repeat a letter on consecutive nights sometimes. |
| Attendees with perfect memory | AC-23a | Three attendees who remember every past night each guess over 10,000 nights: the most-used letter, the least-used letter, and anything but last night's. Each must land between 18 % and 22 %. Chance is 20 %. |
| The slot path | AC-23, G-1, G-10 | `slot.py` takes the date and nothing else, and every caller passes the literal 5. Nothing in the pipeline reads, writes or imports `mvp/answer-history.json`, and no function both draws a slot and writes a file — the shape of the MVP's old ledger-writer, under any name. |
| Five options | AC-24 | Every question has five options, and exactly one is *does not compile*. |
| No published distribution | AC-25, G-11 | No percentage or ratio sits within a line of *compile*, *UB*, *undefined*, *panic* or *output* in anything an attendee could read: the copy table, take-it-home text, the public README, and the organizer docs. Nothing under `web/` or `room/` may mention `bank/audit`. |
| The tells | AC-26 | For each tell there is a short list of rules an attendee might learn — "`unsafe` means undefined behaviour", "long programs don't compile", "pick the longest option", "it's always E", "this topic never compiles". A sixth line checks the answer categories themselves ("always pick *does not compile*"). A rule fails when it beats chance by more than 1.5× **and** luck cannot explain it. With four questions, beating chance by luck is easy, so a rule that is merely lucky is shown as a warning. |
| `unsafe` parity | AC-27 | If any accepted question's answer is undefined behaviour, some accepted question whose answer is not UB must contain `unsafe` too. |
| Fits the room | AC-100, D-15 | Each program fits the wall's reading area at the smallest size the back row can read, using the same arithmetic as `web/shared/typemodel.js`. Each option is one line of at most 29 characters. |
| Difficulty drift | AC-88 | Across accepted questions, the organizer's judged difficulty averages within one level of what was asked for. If not, the **run** fails, not any one question. |

**What fails the run, and what is only reported.** The run fails on any failing
check in the table, and on a question that is flagged for fit or option length
*while it is in the reserve* — accepted, affirmed and unused, so tonight's
schedule could pick it. A flagged question still waiting for review is listed but
does not fail the run. Otherwise every too-long draft would keep the audit red
until someone reviewed it. `--strict` fails on any flag or warning.

As of this ticket, q4, q7 and q8 are flagged for option length, as the table
above says. The option-position tell warns because every migrated record has its
correct option last in the file. The file order is not meant to be the order a
room sees. But until the room's arrangement exists and is shown to reshuffle it,
the audit treats it as visible, and a fifth record written answer-last will fail.

**The report is organizer-only and not committed.** It names answer categories and
how often a rule would have won, which is exactly what AC-25 keeps from attendees.
It is gitignored and regenerated on demand; see `bank/audit/README.md`.
