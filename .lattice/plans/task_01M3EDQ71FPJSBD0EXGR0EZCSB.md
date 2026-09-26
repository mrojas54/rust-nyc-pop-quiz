# PQ-31: Room correct-index twin follows F-19: panic and ub by option kind

Follow-up minted by the Orchestrator 2026-09-26. PQ-4 built room/src/answers.rs as an exact twin of pipeline bank.correct_index (by text for panic and ub). PQ-20 was cleared under F-19 to derive panic and ub answers by option kind (panic <-> non-zero exit_code; ub <-> Miri UB with both borrow models agreeing), never by text equality. Once both PRs are on main, change the Rust twin to the same rule so the two cannot drift, with the same test cases as bank.py's. Criteria: AC-7 (derived, never stored), G-2. Depends on PQ-4 and PQ-20. Mode: fast-track. Terminal pre-merge status: review. PR base: origin/main.

# Plan (delegator, 2026-09-26)

Branch ai-c11-cc/correct-index-twin, rebased onto origin/main @ cac818b (#17 merged since dispatch).

## Files
- `room/src/answers.rs` — `Record::correct_index` mirrors `bank.correct_index` exactly:
  1. does-not-compile record -> the `does_not_compile` options;
  2. ran + `ub_confirmed` (miri present, `clean == false`, `configs` present and holding both
     `stacked_borrows` and `tree_borrows`) -> the `ub` options;
  3. ran + `exit_code` present and != 0 -> the `panic` options;
  4. ran + `stdout` absent -> Ok(None); else options whose text == stdout less one `\n`.
  Each rule is final (no fall-through once one fires). >1 match -> LoadError (bank raises
  BankError with the same message); 0 -> Ok(None). Private `BORROW_MODELS` + `ub_confirmed`
  twins; `#[allow(dead_code)]` dropped from `exit_code`, `configs`, `OptionKind`. Module doc:
  one sentence naming the precedence and bank.py as the reference (G-2). No new pub surface;
  vault untouched.
- `room/tests/twins.rs` — every `correct_index` case in `test_bank.py`, same names in spirit.

## Tests (AC-7, AC-9, G-2)
Fixtures: options authored in twins.rs; `verified` blocks are the verifier's own output,
produced by replaying `verify.verify` on StubRunner over
`pipeline/tests/fixtures/verify/recordings/{q3,q8,panics,ub-both}.json` (accepted verdicts),
pasted verbatim as raw strings, each commented with its recording. No stdout, exit code or
Miri verdict typed.
- output text (q3 as today), exactly-one-newline, no verified record, no match -> None,
  two text matches -> Err
- dnc (q8 replay block) -> dnc option
- panic (panics replay) -> panic option; panic whose stdout equals an output option -> still panic;
  panic with no panic option -> None (no fall-through to text)
- ub both models (ub-both replay) -> ub option; ub outranks non-zero exit (ub-both block with
  exit_code taken from the panics block, not typed)
- ub under one model (ub-both block, `tree_borrows` removed from configs; the verifier never
  accepts that shape — ub-sb-only is rejected `borrow_models_disagree` — so no machine record
  of it exists): falls through to rules 3/4 -> exit 0 -> the output-text rule (matches an
  "before"-less option set -> None, as bank's test; and with the stdout as an option -> that option)
- legacy unclean Miri (no configs) -> not UB -> falls to text
- two options of the derived kind -> Err
- cross-language pin: each embedded block is checked against its recording file in the
  pipeline's fixture set (stdout and exit code equal the recording's runs, configs equal the
  recording's `miri:<model>:*` keys), and every `bank/questions/*.json` derives one option.
  A re-record on the pipeline side fails this suite.

## Open choices / tensions
- (a) "Keep every fixture inside twins.rs" vs "cross-language pin loading the same fixture set":
  no `bank/fixtures/` correct-index set exists and `test_bank.py` builds its cases in memory, so a
  two-sided shared file would need a pipeline test change (out of scope). Side taken: fixtures stay
  in twins.rs; the pin reads the pipeline's recordings + bank questions. Two-sided set proposed as
  a follow-up.
- (b) one-model UB fixture is a mutation of a machine record (config list only), stated in the test.
