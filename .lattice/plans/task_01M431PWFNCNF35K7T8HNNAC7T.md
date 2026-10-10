# PQ-48: AC-26's option-position tell: fail as written, or carry the amended rule

From the Result Validator report, row 39 (AC-26, Fail as written), routed upstream as finding F-48. SPEC section 7.6 and the plan say the enumerated tells fail above 1.5x chance. The audit (pipeline/src/popquiz/audit.py:779-786 and :1053-1059, PQ-24) requires ratio above 1.5 AND an exact tail at or below 0.01/m with m=10 rules; at a bank of four that bound is 0.001 while the smallest possible tail is 0.2 to the fourth, 0.0016, so the tell cannot fail at n=4. On main @ 0a4b6fb the option-position tell is 5.00x chance (index E, 4 hits of 4, 0.80 expected, tail p=0.0016) and reports WARN. The rule is documented (bank/README.md:359 and :371-375; test_the_answer_written_last_warns_at_four_and_fails_at_five) and was accepted by the Orchestrator as a PQ-24 deviation, but no F-n finding or amendment records it and the client never ruled on it. Two branches, the client's: (A) amend SPEC section 7.6 to the multiple-comparison form the code uses and say what happens below five questions, then this ticket only records the ruling and adds the sentence to bank/README.md; or (B) make the code fail as written (ratio above 1.5x, no tail condition), in which case the bank of four fails the audit today because every record stores its answer at E, which is the arrangement hazard the companion ticket addresses. Read PHILOSOPHY.md section 2 first: the audit reads the generator, not the sequence, in both tails, and an audit that can change tonight's output is a rule. HELD on the ruling.

# Plan (delegator, 2026-10-10)

Worktree `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/position-tell-arranged`, branch `ai-c11-cc/position-tell-arranged`, base `origin/main` @ 8819b67. Scope: the client's ruling "A + arranged position" and the Orchestrator's nine rulings in the boot prompt. Cleared files only: `pipeline/src/popquiz/audit.py`, `pipeline/tests/test_audit.py`, `bank/README.md`.

## Sites, re-found on 8819b67

- `audit.py:1-31` module docstring ("What this module does with the slot is call it on *synthetic* nights") - extend: it also arranges questions on synthetic nights.
- `audit.py:761-791` the tells comment block (statistic, margin and tail rule, "why none of this can become an input").
- `audit.py:793-796` `TELL_MARGIN`, `TELL_ALPHA` - unchanged (ruling 1).
- `audit.py:799-815` `Face` and its docstring ("until the room's arrangement exists ... treats bank order as visible") - ruling 5.
- `audit.py:863-891` `face_of`, `tell_pool` - where the arranged faces are built.
- `audit.py:970-981` `_at_index`, `_dnc_at_index` - the position rules; unchanged in logic.
- `audit.py:1009-1016` the "option position" TELLS entry and its statistic string - ruling 5.
- `audit.py:1031-1067` `score_rule`, `measure_tell`, `measure_tells`.
- `audit.py:1364-1387` `run_audit`'s tell lines (summary format "N hit(s) vs E by chance").
- `schedule.py:192-211` `arrange(question, day)`; `schedule.py:56` imports `in_reserve, is_accepted` from audit (so a module-level import back is circular - see Import below).
- `test_audit.py:538-561` the position tests; `:550` `test_the_answer_written_last_warns_at_four_and_fails_at_five` (ruling 7); `:564` the real-bank test.
- `bank/README.md:393` tells row; `:405-413` the "option-position tell warns because every migrated record has its correct option last" passage (ruling 6).

Bank audit before (real bank, 4 questions): `WARN tell: option position [AC-26]: worst rule 'index E': 4 hit(s) vs 0.80 by chance = 5.00x over 4 question(s), tail p=0.0016`. Answer category: 1 of 4 answers is does-not-compile.

## Which faces change, and why (ruling 2)

Only the position tell's faces change. `Face` keeps `kinds`, `lengths`, `unsafe`, `lines`, `topic`, `correct` in stored order for the other five lines: unsafe, source length and topic read only question-level features and the answer's *kind*; option text length reads the set of lengths and which one is correct, which no permutation changes; answer category reads kinds. All are order-free. `Face` gains one field, `walls`: per synthetic night, the arranged option kinds and the arranged correct index, built from `schedule.arrange(question, night)`. The ten position rules read `walls` and nothing else.

## Days (ruling 3)

`POSITION_NIGHTS = 1_000` synthetic nights, `synthetic_nights(GENERATOR_START, POSITION_NIGHTS)` - the same consecutive-date construction as the generator audit's 20,000 and the attendee simulation's 10,000, fixed in the module, never a real meetup date, `used`, the ledger, `answer-history.json` or `bank/audit/`. Why 1,000: each question's per-night hit rate under a sound arrangement is a mean of 1,000 draws; at chance (0.2) its standard error is sqrt(0.16/1000) = 0.0126, i.e. 0.063 in ratio units, so the 1.5x margin is about 8 standard errors from a sound arrangement and no evening's luck enters the verdict. Cost is n x 1,000 arrangements (4,000 today; 100,000 at a 100-question bank, a second or two), which keeps `just test` well under its budget; 20,000 nights would make a large bank's audit slow for no gain in what it can decide.

## Face shape and its statistics (ruling 3): (b) per-question aggregation

Shape (a), one face per (question, night) with the tail over all n x D faces, is rejected: the faces are not independent. On one night every question's answer sits at the same `slot_for_day(night)`, and across nights a question's own features repeat (whether its answer is *does not compile*). Treating n x D correlated faces as independent trials makes the tail far too small: e.g. a bank where 2 of 4 answers are does-not-compile scores the "does-not-compile at j" rules at 2.5x with a tail near zero under (a), a FAIL; the honest count (2 of 4 questions, the same evidence the answer-category tell sees) is a WARN with tail 0.18.

Shape (b): the trial is the **question**. For question q and rule r, over the D nights:

- r fires on q if it names an option on at least one night; q abstains otherwise.
- r_q = (nights r hits) / (nights r fires) - its hit rate on q, in [0, 1].
- pi_q = mean over firing nights of |S|/5 - chance for q. Every position rule names one option, so pi_q = 0.2.
- hits = sum r_q (a real number), expected = sum pi_q, ratio = hits / expected - read exactly as before: a sound arrangement scores about 1x.
- tail = the exact Poisson-binomial tail over pi_q at floor(sum r_q). The Bernoulli model is the most spread-out a [0,1] trial with mean pi can be, and rounding down never makes a FAIL easier, so the tail is conservative.
- The family, margin and bound are unchanged: m = 10 rules, FAIL when ratio > 1.5 and tail <= 0.01/10 = 0.001, WARN above the margin otherwise.

What the bound means for this shape: the smallest possible tail on n questions is when every question's answer is found every night, prod pi_q = 0.2^n. 0.2^4 = 0.0016 > 0.001, 0.2^5 = 0.00032 < 0.001, so the position tell can only WARN below five questions - the same arithmetic the README records, by construction. A one-rule tell (unsafe, source length, topic) has bound 0.01 and 0.2^3 = 0.008, so it can fail at three.

What the measure now sees: under `arrange`, "index j" hits on night d iff slot_for_day(d) = j, for every question alike, so r_q = (nights at j)/D ~ 0.2 and the ratio is the generator's share at j over those 1,000 nights / 0.2 (~1). A bank that stores every answer at E scores ~1x at every index. The "does-not-compile at j" rules: arrange puts *does not compile* at the slot when it is the answer and shuffles it among the other four otherwise, so the rule fires on ~1 night in 5 for every question and hits on every firing night exactly when the question's answer is *does not compile*: r_q in {0, 1}, ratio = 5k/n with k the does-not-compile answers. Under the arrangement these five rules carry the same evidence as the answer-category tell's "pick does_not_compile" rule (identical hits, expected and tail), not a position signal; this is stated in the statistic string. Today's bank: 1 of 4, 1.25x, pass.

Expected after on the real bank: option position PASS, worst 'index X' about 1.0x-1.1x (exact value quoted from the run).

## Import (ruling 4)

`schedule.py:56` imports `in_reserve, is_accepted` from `audit`, so `audit` importing `schedule` at module level is a cycle (ImportError when audit loads first). No lint or import-boundary test trips. Asked by comment (QUESTION PQ-48, 2026-10-10). Planned resolution unless the Orchestrator rules otherwise: `tell_pool(questions, arrange=None)` with `arrange` injectable (as `audit_generator`'s `generator`); `None` resolves to `popquiz.schedule.arrange` by a function-local import. Nothing flows back: arrange's output becomes `Face.walls`, read only by `score_across_nights`, reported only in the check detail. `slot.py` and `schedule.py` are not touched; the slot-path, call-site and ledger lints over `pipeline/src` must stay clean (`test_the_real_pipeline_is_clean`). A question `arrange` refuses (BankError) has no walls; it abstains from the position tell and its id is named in `left_out` with the reason, never guessed.

## Code (rulings 1, 2, 5)

1. `Face.walls: tuple[tuple[tuple[str, ...], int], ...] = ()`; docstring says stored order for the order-free tells, `walls` for the position tell.
2. `POSITION_NIGHTS`, `position_nights()`.
3. `walls_of(question, arrange, nights)`; `face_of(question, arrange, nights)`; `tell_pool(questions, arrange=None)`.
4. `score_across_nights(name, rule, faces)` per (b), returning `RuleScore` (hits becomes `float`, the count of questions an arranged rule found, summed as rates; the stored-order rules still produce whole numbers).
5. `measure_tell(..., score=score_rule)`: one keyword, default unchanged, so the verdict logic (margin, alpha / m, worst) is the same code path for every tell. TELLS rows gain the scorer for the position row only. This is the only touch to `measure_tell`; flagged as a deviation-with-reason since ruling 1 says "do not change measure_tell": its rule is unchanged, only where the scores come from.
6. Statistic string: "pick index j; pick 'does not compile' when it sits at index j - in the wall's order, schedule.arrange over 1,000 synthetic nights, one trial per question ...".
7. `run_audit` summary: hits printed with two decimals for the position line ("question(s)").

## Tests (ruling 7) and mutations

- Replace `test_the_answer_written_last_warns_at_four_and_fails_at_five` with `test_a_bank_storing_every_answer_last_is_not_a_position_tell`: five and six real `Question`s with the answer stored at E, real `schedule.arrange`, `tell_pool` -> position verdict `pass`, every rule's ratio < 1.5.
- `test_an_arrangement_that_ignores_the_date_warns_at_four_and_fails_at_five`: a stand-in arrangement injected in the test only (returns the stored order) over several nights -> warn at 4, fail at 5 - the small-bank arithmetic, and that nights do not multiply trials.
- `test_an_arrangement_that_pins_the_answer_is_a_tell`: stand-in that puts the correct option at C on every night, 10 questions -> fail, worst 'index C'.
- Keep `test_the_answer_always_in_one_place_is_a_tell` and `test_a_small_bank_does_not_fail_on_luck` via a one-night stored-order wall built by the test's `face()` helper (walls default to the stored order as a stand-in).
- `test_the_position_nights_are_synthetic`: `position_nights()` equals `synthetic_nights(GENERATOR_START, POSITION_NIGHTS)` and `tell_pool` takes no date.
- `test_the_dnc_rules_carry_the_answer_category_evidence`: under real arrange, a bank with 2 of 4 does-not-compile answers -> each 'does-not-compile at j' rule's (hits, expected, tail) equals 'pick does_not_compile' (approx), verdict warn not fail.
- Mutations (each run, bounded, reverted): M1 walls from stored order instead of arrange -> the stored-at-E test fails. M2 pool (question, night) faces as independent trials -> the ignores-the-date warn-at-four test fails (and the DNC-evidence test). M3 floor -> ceil, or dropping the tail condition -> the warn-at-four test fails. M4 position nights taken from a different start/count -> the synthetic-nights test fails.

## Docs (ruling 6)

`bank/README.md:393` tells row: the position rules read the wall's order (`popquiz schedule`'s arrangement over 1,000 made-up nights, one trial per question). `:405-413`: replace the "warns because every migrated record has its correct option last" passage with the arranged measure, the stored-at-E bank passing, and the rule: m = 10, bound 0.001, smallest tail 0.2^n, so the position tell can only warn below five questions; a one-rule tell can fail at three. A fact of the rule.

## Commits

1. audit: the position tell reads the arranged order over synthetic nights (code + docstrings + statistic).
2. tests: arranged-order position tests replace the answer-written-last test.
3. bank/README: the tells row and the small-bank rule.

## Contract tension, and the side taken

- SPEC 7.6 "failing above 1.5x chance": known, routed upstream as F-48; not edited, not a reason to change the rule.
- Ruling 1 "do not change measure_tell" vs ruling 2's per-question scorer: add a defaulted `score` keyword; the rule is unchanged. Deviation 1.
- EVALUATION canary row "in bank order": the room renders the pushed record's order (`room/src/question.rs:14-17`), and `popquiz schedule` pushes the arranged record (`schedule.py:557-565`), so on the wall "bank order" is the arranged order. Contract-wording note for upstream; not edited.

## Orchestrator RULING on the import (ev_01M4K909JA85JCKWCR60JK9X9C), folded in

Approved as proposed, with: (a) the default is the real `schedule.arrange`, resolved at call time; the bank-audit CLI path (`run_audit` -> `tell_pool(questions)`) never passes a stand-in; test: a stored-at-E bank written as JSON records under a tmp repo reads at chance through `run_audit` (the CLI-level entry point), not only via an injected arrange. (b) No top-level import of `popquiz.schedule`; a comment at the local import says why (schedule imports audit at schedule.py:56). (c) `arrange`, `_rng`, `_shuffled` stay in schedule.py; a pure arrangement module is named in DONE contract-notes for PQ-51, not built here.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer (sonnet, fresh eyes): no Critical, 3 Major, 4 Minor.

- **M1 accepted (wording).** Under the real arrangement every question's index-rule rate is the same shared number (the nights' share at j), so the ratio says nothing about the stored order and the tail over those rates is *not* a calibrated p-value; "conservative" holds only for question-specific leakage. The margin is what holds for the date-driven part. Measured: over the 1,000 synthetic nights from 2026-08-12 the slots are A=214 B=212 C=177 D=228 E=169, so the largest index-rule ratio a sound arrangement can show is 1.14x. Code comment and statistic string say what the index rules can and cannot see. The earlier "8 standard errors" sentence stays only as the reason for the night count; the quoted counts are the evidence.
- **M2 accepted (wording), rule family unchanged.** AC-24 (enforced by `check_five_options`) gives every question exactly one does-not-compile option, so the identity is 5k/n. The DNC-at-j rules duplicate 'pick does_not_compile' under the arrangement, against bound 0.001 instead of 0.0025, so they can never FAIL where the answer-category tell passes its bound. Kept in the family (dropping them changes m, which ruling 1 forbids); documented.
- **M3 accepted.** Added: CLI-level tests through `run_audit` on a tmp repo of JSON records (ruling (a)): a stored-at-E bank of five reads the position line PASS under the default arrangement; a DNC-heavy bank (3 of 5 answers DNC) through `run_audit` - position not FAIL beyond what answer category says.
- **Minor floor accepted, documented:** rates of 0.95 warn up to six questions and fail at seven (computed: n=6 tail 0.0016, n=7 0.00037).
- **Minor left_out accepted:** position-only exclusions are not in the shared `left_out`; the position check's detail carries `unarranged`.
- **Minor hits-float accepted:** summary formats hits `:.2f` for the position line, `:g` elsewhere (unchanged output for integers); `asdict`/JSON take floats.
- **Deviation 1 stands:** `measure_tell` gains a defaulted `score` keyword; its verdict code is unchanged.
