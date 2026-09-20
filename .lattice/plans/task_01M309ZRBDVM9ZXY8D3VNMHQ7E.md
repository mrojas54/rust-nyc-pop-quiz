# PQ-18: Bank format and MVP migration

BUILDPLAN.md T-14 (M3).

Bank format (`SPEC.md` §3.1–3.3) and migration of the eight MVP questions: verified facts carried over **as `legacy` records — only the fields the MVP wrote, nothing back-filled** (`SPEC.md` §3.2, D-16), the MVP's `error_codes` renamed to `compile_error_code` on q8's record, `explains` three beats and `trace` drafted for organizer affirmation, `explanation` kept as `legacy`

Criteria: AC-13, AC-17, AC-73, AC-87
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-01
BUILDPLAN notes: **Only q3 migrates as authored** (touchpoint T-22, D-15). q4, q7 and q8 fit as programs but their options must be re-authored to one line of at most 29 characters and re-verified (the machine decides the new answer). q1, q2, q5 and q6 exceed the wall's 7-line capacity at the guessed room and stay in the MVP bank only, until re-authored or until the rehearsal's measurements raise the capacity.

Orchestrator notes: Only q3 migrates as authored (D-15). Never write down what a program prints: legacy records carry only what `mvp/2026-08-12/verified.json` holds, nothing back-filled.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

---

# Plan (delegator, 2026-09-20)

Base: `origin/ai-c11-cc/repo-scaffold` @ `2ab95e31` (PR #8). Worktree
`/Users/michellerojas/rust-nyc-pop-quiz-worktrees/bank-format`, branch
`ai-c11-cc/bank-format`, currently equal to the base with no commits of its own.

## 1. What the brownfield actually holds

`mvp/2026-08-12/verified.json` does not use SPEC §3.2's field names, so the
migration is a rename plus a reshape, not a copy. The mapping, per question:

| SPEC §3.2 | MVP source | Note |
|---|---|---|
| `rustc` | top-level `rustc` = `"rustc 1.96.1 (31fca3adb 2026-06-26)"` | file-level, not per question; the `--version` string, never a `-Vv` |
| `edition` | top-level `edition` = `"2021"` | file-level |
| `runs` | per-question `runs` (5) + `byte_identical` (true) | §3.2 makes `runs` a compound `{count, byte_identical}` |
| `stdout` | per-question `answer` | machine-written; copied, never typed |
| `miri.clean` | per-question `miri` = `"clean"` | a free-text string translated to a bool, not invented |
| `miri.output_matched` | per-question `miri_output_matches` | |
| `compile_error_code` | q8's `error_codes` = `["E0502"]` | renamed, values unchanged |

Dropped on purpose, with reasons:

- `compiled`, `verified` (bools) — MVP bookkeeping. §7.5 derives *Compiled* from
  `runs` being present, so a second writer of the same fact is a field with no
  reader (§3, opening rule).
- `stderr_head` (q8) — not in §3.2's field list. It is a real machine artifact, so
  discarding it is a small loss; §3.2 is the contract and `compile_error_code` is
  what the receipt reads. Noted for the Orchestrator rather than smuggled in.
- `runs_per_question`, `generated_for`, top-level `miri` prose, `expect`,
  `difficulty` (→ `difficulty_requested`), `hint`, `topic`, `source` — the last
  five are §3.1 question fields, not `verified` fields, and move there.

q8 has no `runs`, no `byte_identical`, no `miri_output_matches`, and its `miri`
reads `"n/a (does not compile)"` — not a result, so no `miri` object is written.

## 2. Files

**New:**

- `pipeline/src/popquiz/bank.py` — the §3.1 question and §3.2 `verified` record as
  frozen dataclasses; `review`, `used`; load / save / append for
  `bank/questions/<id>.json`; the `history` shape with load/save and a size
  helper; `receipt_class(verified)`; `correct_index(question)`;
  `quoted_outputs(explains)`.
- `pipeline/src/popquiz/receipt.py` — `receipt_lines(verified) -> list[str] | None`
  and `receipt_detail(verified)`. Stdlib only, reads nothing but its argument.
- `pipeline/src/popquiz/migrate_mvp.py` — reads the two MVP files, writes the bank
  records; run once, output committed.
- `pipeline/tests/test_bank.py`, `test_receipt.py`, `test_migration.py`.
- `bank/schema/question.schema.json` — the published shape.
- `bank/questions/q3.json`, `q4.json`, `q7.json`, `q8.json` — migration output.
- `bank/history.json` — the shape, entries empty.
- `bank/fixtures/receipts/*.json` — seven cases, see §5.

**Changed:** `bank/README.md`, `pipeline/README.md` (the module table's `bank`,
`receipt`, `migrate_mvp` rows). Nothing else. Not `justfile`, not
`pyproject.toml`, not `runner.py`.

## 3. Design decisions, and the side taken

**D1 · `correct` is a derived function, not a stored field.** §3.1 lists
`correct` and marks it **derived (G-2)**. A stored field is a field someone can
hand-edit, which is exactly what AC-7 exists to catch. So there is no `correct`
key in the record and no way to write one: `correct_index(question)` computes it
— for a record that ran, the option whose text equals `verified.stdout`; for a
does-not-compile record, the single `does_not_compile` option.

**D2 · The stdout↔option comparison normalizes one trailing newline.** §3.1 says
the correct option's text "equals the verified output", but every MVP `stdout`
ends in `\n` and no option text does — `build_deck.py:266` strips it. Comparing
raw would make `correct_index` return `None` for all seven output questions. Side
taken: compare on `rstrip("\n")`, with the precedent cited in the docstring and a
test that pins it. Flagged as a SPEC imprecision, not a SPEC defect.

**D3 · Bank order is canonical and meaningless, and says so.** Letters and the
wall's option order are drawn from the date by `slot_for_day` (AC-23), so the
stored order must carry no information. Stored order = authored distractors, then
`does not compile` if it is not the answer, then the derived correct text last.
`bank/README.md` and the dataclass docstring both state that bank order is not
wall order, so no later ticket reads a position tell into it (AC-26).

**D4 · `why_tempting` keys on the option, never on a letter.** The prototype's
`explains.whyWrong` is `{option: "A", text: …}`. A letter is date-derived, so a
letter-keyed beat dangles the moment the date changes — the same Stage-2 leak
§3.1 flags for `argue`. See the open choice in §7 for which non-letter key.

**D5 · The trace's resolving step is derived, not typed.** `data.js:127` writes
the final step's `stdout` value by hand. The migration builds that entry from
`verified.stdout` and a test asserts byte equality; the intermediate `values`
(program state, not printed output) stay authored, which is what the organizer
affirms. A second test asserts `migrate_mvp.py`'s own source text contains no
literal equal to any MVP `answer` — the house rule, mechanized.

**D6 · `receipt_detail` returns facts, not take-it-home copy.** AC-87's *not
recorded* and *run separately* are participant-facing strings on a surface T-12
builds and T-22 freezes; §11 does not hold them, so this ticket does not author
them. `receipt_detail` returns a dataclass whose `target_triple` is `None` and
whose `miri_ran_outside_verifier` is `True` for a legacy record — the facts are
**derivable**, which is what the ticket asks, and the wording stays with the
surface that renders it.

**D7 · The re-authoring note goes in `review.reason`, with `status` unset.** No
new field is minted. `status` unset is honest: no organizer has judged q4/q7/q8.
`reason`'s reader per §3.1 is the generator's next run, which is exactly who
needs to know the options must be re-authored to one line of ≤ 29 characters.

**D8 · q4, q7 and q8 migrate with an empty trace.** The MVP holds no step
structure for them, their options are about to be re-authored, and a trace drafted
against options that will change is worse than none. An empty trace also blocks
affirmation by §7.4, which is the correct state for a question awaiting
re-verification. Validation therefore requires the "last step names `stdout`"
invariant only when the trace is non-empty and the record ran.

**D9 · The schema is checked against the dataclasses, not enforced by a
validator.** There is no JSON-Schema validator in the standard library and
`pyproject.toml` is serialized to another ticket, so adding one is not this
ticket's call. `test_bank.py` walks both directions — every dataclass field is a
schema property with agreeing optionality, and every schema property exists on the
dataclass — so drift fails the build. Migrated records are validated by being
loaded through `load_question`. The README says plainly that the schema is the
published contract and the drift test is its enforcement, so nobody reads it as
record validation. Noted for T-19.

**D10 · `✓ Ran ‹N› times` renders literally.** At N=1 that reads "Ran 1 times".
SPEC §7.5 gives the template verbatim and the copy module is T-22's, so inventing
a singular here would be authoring participant-facing copy this ticket is not
cleared for. Implemented as given, noted for the Orchestrator.

## 4. The receipt, gate by gate (§7.5)

`receipt_lines(verified)`, precedence does-not-compile first:

- `compile_error_code` non-empty → `✓ Compiler refused it`, `✓ Error <codes
  comma-separated>`, `✓ Nothing ran`. Three lines, whether the record is complete
  or `legacy`.
- else `runs` present → `✓ Compiled`; `✓ Ran <count> times` when `runs` has a
  count; `✓ Output never varied` when `runs.byte_identical`; the Miri line when
  `miri` is present **with `output_matched` true**, reading `✓ Miri ran clean` on
  `miri.clean` and `✓ Miri flagged undefined behavior` otherwise.
- else → `None`.

Every line is gated on the record holding its step, so the list cannot claim more
than was done (G-2). `RECEIPT_HEADING = "How we know"` is a module constant rather
than a returned line: the heading carries AC-74's provenance marker, which is the
renderer's business, and the AC-43 assertions are about the step lines.

## 5. Fixtures — `bank/fixtures/receipts/<case>.json`

`{"_note": …, "verified": {…}, "expected_lines": [… | null]}`. The note sits
outside `verified` so the record stays clean. T-05's JavaScript twin reads the
same files; `bank/README.md` says so.

| Case | Record | Expected |
|---|---|---|
| `q8-does-not-compile-legacy` | q8's real legacy record | the three lines |
| `does-not-compile-two-codes` | synthetic, two codes | `✓ Error E0502, E0499` |
| `legacy-ran` | q3's real legacy record | the four lines |
| `complete-ran` | complete, all §3.2 fields | the same four lines |
| `panic-answer` | empty `stdout`, non-zero `exit_code` | the four lines |
| `ub-declared` | `miri.clean` false, `output_matched` true | `✓ Miri flagged undefined behavior` |
| `no-receipt` | neither complete, `legacy`, nor does-not-compile | `null` |

**Provenance of the synthetic four.** `q8-does-not-compile-legacy` and
`legacy-ran` are copied from the machine-written `verified.json`. The other four
describe **no Rust program** — they exist to exercise the receipt's gates, which
never read `stdout`'s content, only whether a step is present. Each carries a
`_note` saying so, following `tests/fixtures/scaffold-placeholder.json`'s
precedent exactly. `panic-answer`'s `stdout` is empty, so nothing is typed there
at all. This is a judgment call against the house rule and is flagged as one.

## 6. Tests, by criterion

**AC-13** — `receipt_lines` renders from a `Verified` alone; `receipt.py` imports
only the standard library; the scaffold's existing
`test_nothing_just_test_imports_can_start_a_process` already holds the suite to no
toolchain and the new modules are inside the directories it scans.

**AC-87** — complete-ran and legacy-ran render byte-identical lines; complete and
legacy does-not-compile both render the three-line list with no run count and no
Miri line; a record that is none of the three renders `None`; `receipt_detail`
returns `-Vv`, edition, triple, flags and Miri config for a complete record, and
for a legacy one returns `target_triple=None`,
`miri_ran_outside_verifier=True` and nothing back-filled.

**AC-43** — all seven fixtures, driven from the published files rather than
inline copies; across every rendered line, none ends in `.`, `!` or `?` and none
contains *verified*, *established*, *proves*, *always* or *guaranteed*
(case-insensitive).

**AC-73** — q3's `explains.legacy` is byte-equal to `content.json`'s
`explanation`; `quoted_outputs` returns the quoted spans in q3's beats. The
docstring is exact that it lists **candidates** and that deciding which are claims
about printed output, and comparing them to `verified.stdout`, is T-18's.

**AC-17** — `bank/history.json` round-trips through save/load from two separate
directories in one test; the size helper reports per-store counts for the run
report and review surface. T-17 proves the rest on this shape.

**G-2 / AC-7** — no legacy record holds `target_triple`, `flags`, `exit_code`,
`verified_at`, `verifier_version`, or a `rustc` string that looks like `-Vv` (no
newline, no `host:` / `release:` / `commit-hash:` token); every migrated
question's correct option text is byte-equal to `verified.stdout.rstrip("\n")`;
`migrate_mvp.py`'s source holds no literal equal to any MVP `answer`; the final
trace step's `stdout` value equals the verified stdout.

**Shape** — each migrated question has five options with exactly one
`does_not_compile` (AC-24's bank-wide audit stays T-19's); the schema/dataclass
drift check in both directions.

`just test` run warm, time recorded, kept under 60 s.

## 7. Open choice for the client

**How `explains.why_tempting` identifies its option** (D4 settles that it is not a
letter; this settles what it is). Option index in bank order is stable within the
file and survives re-lettering, but silently mis-associates if a later edit
reorders options. Matching on the option's own text is self-describing and breaks
loudly instead, but goes stale the moment T-15b re-authors an option — which is
scheduled for q4, q7 and q8. I will put a `TODO(human)` at the decision during
implementation rather than choose for her.

## 8. Contract tension and one defect

**Tension, side taken.** The plan preamble above says *PR base: origin/main*; the
boot prompt says base `ai-c11-cc/repo-scaffold`, stacked on #8, because this
ticket needs the scaffold's harness. Following the boot prompt — it is the later
and more specific instruction, and it explains itself. The Orchestrator retargets
after #8 merges.

**Defect, reported not fixed.** A does-not-compile question can never be
affirmed, so it can never be scheduled. §7.4 requires a trace of at least two
steps before affirm; §3.1 and D-10 require the trace's last step to be the one
whose `values` names `stdout`. A does-not-compile record has no `stdout` — §3.2
says so explicitly. So the two rules together have no satisfiable form for such a
question. This is not hypothetical: q8 is one, AC-11 has the generator produce
them, AC-24 puts *does not compile* on every question, D-22 gives the class its
own receipt, and BUILDPLAN's minimum viable cut expects q8 in the October bank.
Going to the Orchestrator as a `lattice comment`. This ticket implements the
narrowest honest reading (the invariant binds only a non-empty trace on a record
that ran) and does not amend the contract.

## 9. Order of work

1. `bank.py` types, then the schema and its drift test.
2. `receipt.py` with `receipt_lines` and `receipt_detail`; fixtures; the AC-43 and
   AC-87 tests.
3. `migrate_mvp.py`; the `TODO(human)` on `why_tempting`'s key; run it; commit the
   four records and `history.json`.
4. The provenance tests (G-2, AC-7), AC-73, AC-17.
5. `bank/README.md`, `pipeline/README.md`.
6. `just test`, code review, validation, PR against `ai-c11-cc/repo-scaffold`.
