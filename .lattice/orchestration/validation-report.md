# Validation Report

Source spec: [SPEC.md](../../SPEC.md)
Source build plan: [BUILDPLAN.md](../../BUILDPLAN.md)
Source validation plan: [validation-plan.md](./validation-plan.md)
Result Validator: agent:result-validator (c11 `tab:22`, fresh session; five in-process Sonnet sub-agents walked the rows, one report-writer)
Date: 2026-10-05
Run completed: 2026-10-05 (every unheld ticket done; T-16 and T-18 held)
Audit tree: `origin/main` @ `e239346f938896c91f14bc6a299fd3fce8d0bb7f` (merge of PR #49), detached checkout at `rust-nyc-pop-quiz-worktrees/validate-final`. Nothing committed, pushed or edited; no ticket touched. `git status --porcelain` at exit: `?? .claude/` only.

## Summary

- **Total criteria audited: 83** pre-merge rows (7 `pre-merge-static` + 76 `pre-merge-runtime`, counted by script; the plan's footer says 8/74/44). All 43 `post-merge-smoke` rows are copied into the checklist below.
- **Pass: 60**
- **Partial: 15**
- **Fail: 2** — row 39 (AC-26 option-position tell, as written; F-48) and row 100 (AC-84 contrast; F-46)
- **Blocked: 6** — every one a never-built ticket: T-16 (PQ-21; rows 4, 5, 7) and T-18 (PQ-23; rows 28, 31, 43)

**Run state at audit.** The board has 64 tickets: 38 done, 1 cancelled (PQ-33), 25 backlog, and every one of the 25 is held. Two BUILDPLAN tickets were never built: T-16 (PQ-21, held on H-4, the API key) and T-18 (PQ-23, held on F-15, with the affirmation path F-52 open). The other held tickets are follow-ups and the guardrail audit's gaps (PQ-30, PQ-41…PQ-43, PQ-46…PQ-64). Fly v11 runs `1263add`, twenty merges behind this tree; the deployed room is not this audit's subject.

**Shared harness runs on the audit tree** (logs under `/tmp/claude-501/validate/`):

| Command | Result |
|---|---|
| `just test` | **EXIT 0, 17.0 s warm.** The cold first run (inside the sandbox) compiled and ran for 50 s, then failed only on sandbox denials: the uv cache, and the Discord mock's loopback `bind` (`discord_mock.rs:110`, EPERM). It was rerun with the per-command bypass. Room: every test binary green. Pipeline: **710 passed**, 1 warning (q3 `hint` trope warning, signpost *The key word*). Web: **286 pass, 0 fail** |
| `just canary` | EXIT 0, 12 passed |
| `just bank-audit` | EXIT 0, `bank-audit: passed`, one **WARN** (option-position tell, index E, 4/4 = 5.00× chance, p=0.0016); report `bank/audit/2026-10-05.json` (gitignored) |
| `just a11y` | EXIT 0, 31 pass, matrix and every contrast ratio printed |
| `just secret-scan` | EXIT 0, 2 passed |

*(Counts updated 2026-10-05 after the client-requested browser addendum: rows 49, 98 and 102 moved Partial → Pass. See **Addendum** below.)*

**Overall verdict: 🟡 YELLOW.** Everything built meets the contract as its named tests prove it, and the tree is better than at the interim. #47 built the affirm gate, the date-drawn arrangement, the reserve and the warning. #48 put AC-62 and AC-63 in a runbook. The canary, the sealed answers module, the phase machine, auth, the admin channel, the verifier's decision logic, the receipt and the bank audit all pass. It is not green, for three reasons:
- **G-12 has a gate and no key.** Nothing writes an affirmation, so `popquiz schedule` refuses every committed question and the reserve reads 0 (F-52). The first meetup from the built app needs a human decision on how q3 gets affirmed.
- **Two guardrail holes are open on the tree:**
  - A second room on a scheduled question can be created while the first is live. Releasing one publishes the answer at `/last` while the other is pre-reveal (G-3, GAP-8, confirmed at `rooms.rs:846-860`).
  - The standalone fallback CLI bakes the bank's stored order: the answer is at E, with no date (G-1, GAP-2).
- **The two Fails are known contract questions awaiting rulings** (F-48, F-46).

## Changed since the interim report

Interim: tree `0a4b6fb`, 55 / 15 / 2 / 11. Now: 60 / 15 / 2 / 6. Eleven rows changed; every other row has the same result on re-run evidence.

| # | Criterion | Interim → now | Why |
|---|---|---|---|
| 9 | AC-6 | Pass → **Partial** | #47 built scheduling without the stale-pin refusal SPEC §7.2 requires. `verify.is_stale` has no production caller (GAP-5, PQ-58) |
| 42 | AC-72 | Blocked → **Partial** | #47: `schedule.refusal` is a hard-error gate. The affirmation writer (T-18) is still absent |
| 46 | AC-75 | Blocked → **Pass** | #47: reserve count and trend are the first lines of every `popquiz` command |
| 47 | AC-76 | Blocked → **Pass** | #47: low-reserve warning with default lead time 2 |
| 50 | AC-102 host sheet | Partial → **Pass** | #47: `popquiz schedule` writes the file and its sheet from one call on the arranged record |
| 76 | AC-49 | Pass → **Partial** | No code change. This is a stricter reading: AC-49 says each host screen *states* its phase, and the label is announced but deliberately not shown (SPEC §11:628, PQ-34 #27). The criterion outranks SPEC |
| 89 | AC-62 | Blocked → **Partial** | #48: the runbook carries the sentence verbatim. The host's-first-screen half is not met (removed by PQ-34 #27) |
| 90 | AC-63 | Blocked → **Pass** | #48: `docs/RUNBOOK.md:275` verbatim, tested |
| 49 | AC-102 fallback | Partial → **Pass** | No code change. The addendum ran it as written: network blocked, real keys, on the shipped file |
| 98 | AC-82 | Partial → **Pass** | No code change. The addendum used real Tab presses and saw a visible ring on every surface |
| 102 | AC-86 | Partial → **Pass** | No code change. The addendum switched the preference on and saw no motion |

## Per-criterion results

The log is `just-test.log` unless named. PRs cite the merge that carried each row's proof (map: boot prompt §2). Where a later PR changed what an earlier one proved, the tree was judged and both PRs are named.

| # | Criterion | Result | Notes |
|---|---|---|---|
| 1 | AC-1 room never calls the generator | ⚠️ Partial | `room/Cargo.toml:94-138` has no generator, verifier or LLM crate and no path dependency on `pipeline/`. `rg 'anthropic\|reqwest\|popquiz' room/Cargo.lock` finds nothing. **No static dependency check exists in `just test`** (`boundary.rs:93` is the unsafe scan only), so "the check exists and passes" is unmet and the criterion holds by convention. The check was T-16's, and PQ-21 was never built. |
| 4 | AC-3 count/topics/difficulty honoured | ⛔ Blocked | No PR. PQ-21 (T-16) is in backlog with an empty `artifact_info` and was never dispatched (held on H-4, the API key). `generate.py:1-4` is a scaffold docstring. |
| 5 | AC-4 talk mode tags a concept | ⛔ Blocked | No PR; PQ-21 never dispatched (H-4). |
| 7 | AC-5 cost and wall-clock in the report | ⛔ Blocked | No PR; PQ-21 never dispatched (H-4). No run-report code. |
| 9 | AC-6 record fields and the pin | ⚠️ Partial | Proven: `test_a_ran_record_holds_exactly_what_the_machine_observed` (`test_verify.py:166`; `-Vv`, edition, triple, flags vs the recording and the pin), `test_a_legacy_record_is_exempt_from_the_stale_check` (:385), and `test_a_toolchain_that_is_not_the_pin_is_refused_before_any_candidate_step[×4]` (:326). *Wrong-pin candidate rejected stale* is a tested predicate only (`test_is_stale_matches_the_pin_it_was_verified_under`, :367). `verify.is_stale` (`verify.py:507`) has no production caller, and `schedule.refusal` (`schedule.py:127`) never checks it, although SPEC §7.2 says scheduling refuses a stale record. GAP-5 / PQ-58. PRs #16, #47. |
| 11 | AC-7 answer never hand-written | ✅ Pass | Reran the PQ-39 method. The pin is rustc 1.98.1 `48a229ceaefd` with miri 0.1.0 (420ed2a0c3 2026-09-18). The verifier digest, recomputed two ways (`verifier_version()` and an independent sha256 over `verify.py`, `runner.py` and `sandbox.py`), is `popquiz-verify 7b1bc351e4dd`, equal to `verifier_version` on q4, q7 and q8. rustc, edition, flags, triple and Miri on those three equal `pipeline/sandbox/pin.toml`, and none is stale. A StubRunner replay at the stored `verified_at` reproduces each record on every field. Round-trip through the bank writer is byte-identical (`cmp`) for q3, q4, q7 and q8. Static half: the bank stores no `correct` field. It is derived only by `bank.correct_index` (`bank.py:389`; room twin `answers.rs:224`), and records are built only in `verify.py:311` and written at `:693`, pinned by `test_only_the_verifier_the_migration_and_the_reader_construct_a_record` (:547). `test_a_hand_edited_answer_is_refused` (:431), `…stored_correct_field…` (:441), `…hand_edited_option_kind…` (:448) and `test_every_bank_question_passes_the_provenance_check` (:425) all pass. #36's hand-typed blocks are gone. Caveats in Drift: F-55 reproduced (a self-consistent hand edit passes `check_provenance`) and GAP-4 (provenance is not on the schedule/bake path). PRs #16, #40, #45. |
| 12 | AC-8 varying output rejected | ⚠️ Partial | `hashmap-order` gives `output_varied` (rejected, not flagged), per `test_the_output_that_varied_is_counted_not_flagged` (:128). `RUNS==5`; q3, q4 and q7 are accepted at N=5. *Rejection count reported* holds per candidate only: the run-level tally is T-16's run report (`verify.py:91`), never built. PR #16. |
| 14 | AC-9 Miri UB handling | ✅ Pass | `ub-both`/ran gives `ub_not_declared`, `ub-both`/ub gives `accepted`, and `ub-sb-only`/ub gives `borrow_models_disagree`. `test_tree_borrows_is_asked_only_for_declared_ub` (:114) and `test_ub_under_one_borrow_model_is_refused` (:463) pass. PR #16. |
| 16 | AC-10 Miri stdout ≠ native | ✅ Pass | `miri-differs` gives `miri_output_differs`, using equality, not prefix (`verify.py:459-466`). PR #16. |
| 18 | AC-11 declared non-compile | ✅ Pass | q8 is accepted with E0502 recorded. q3 declared does-not-compile gives `compiled_but_declared_does_not_compile`, and `syntax-error` gives `failed_without_an_error_code`. Passing: `test_a_does_not_compile_record_records_every_code_the_compiler_gave` (:203), `test_the_receipt_renders_the_does_not_compile_list_from_a_record_verify_wrote` (:292), `test_a_does_not_compile_verdict_runs_nothing` (:122). PR #16. |
| 21 | AC-13 receipt from the record alone | ✅ Pass | `test_the_receipt_module_needs_no_toolchain_and_no_world` (`test_receipt.py:283`; `receipt.py` imports `dataclasses` and `popquiz.bank` only). `test_each_fixture_renders_the_lines_the_spec_gives_it` and `test_a_legacy_record_still_renders` (`test_verify.py:304`) pass. PRs #16, #10. |
| 22 | AC-87 receipt lists and take-it-home detail | ✅ Pass | `test_receipt.py` runs over the 9 fixtures in `bank/fixtures/receipts/`. Complete-ran and legacy-ran give four lines, the two does-not-compile fixtures give three, and `no-receipt` gives none (`test_a_record_that_is_none_of_the_three_renders_no_receipt`). Rust twin: `twins.rs::the_receipt_matches_every_shared_fixture`. Take-it-home: `home.test.js:165` (complete: every `-Vv` line, edition, triple, flags, Miri version and configs) and `:182` (legacy: target *not recorded*, Miri *run separately*, nothing back-filled). `take_home.rs::a_complete_record_carries_the_machine_verbatim` and `::a_legacy_record_holds_only_what_it_recorded` pass. The committed q4, q7 and q8 are complete records. PRs #16, #34. |
| 23 | AC-14 exact duplicate | ✅ Pass | `test_a_byte_identical_resubmission_of_any_bank_question_is_rejected` (`test_dedupe.py:303`, all four) and `test_an_exact_resubmission_is_rejected_by_a_run_and_written_nowhere` (:324). PR #14. |
| 24 | AC-15 normalized duplicate | ✅ Pass | `test_renamed_bindings_and_reformatting_are_normalized_duplicates[×3]` (:359) and `test_a_renamed_bank_question_is_rejected` (:367). PQ-30 (F-20, false normalized duplicates) is the converse defect and is still open (`dedupe.py:55-62`), but it does not break this condition. PR #14. |
| 25 | AC-16 near-duplicate to review | ✅ Pass | `test_a_near_duplicate_lands_in_the_review_queue_neither_accepted_nor_dropped` (:703; `near_duplicate_of=="q3"`, status None) and `test_a_near_duplicate_is_marked_with_the_question_it_is_near` (:692). The queue is bank files with review fields, since `review.py` is a scaffold. Because of PQ-30, some distinct programs never reach it. PR #14. |
| 26 | AC-17 history persists | ⚠️ Partial | `test_the_history_persists_and_grows_across_two_runs_in_separate_working_directories` (:779): 0→5, then 5→6 (growth 1), in the CLI text and the `--json` run report. *On the review surface* is Blocked: T-18 was never built. PRs #14, #10. |
| 27 | AC-18 uniqueness wording | ✅ Pass | `UNIQUENESS_STATEMENT` is at `dedupe.py:121`. `test_every_uniqueness_statement_in_the_corpus_is_the_exact_sentence` (:958) and `test_the_module_never_says_original` (:954) pass. PR #14. |
| 28 | AC-19 review round-trips | ⛔ Blocked | No PR. PQ-23 (T-18) is in backlog with no PR artifact and was never dispatched (F-15; F-52 open). `review.py:1-4` is a scaffold. |
| 31 | AC-22 reviewer-has-seen statement | ⛔ Blocked | No PR; PQ-23 never dispatched (F-15). There is no review surface. |
| 32 | AC-88 difficulty drift fails the run | ⚠️ Partial | T-19 half holds: `audit.difficulty_drift` fails the **run** and names no question. Reran 11 tests: `test_audit.py:688` (drift 4/3 fails, no qid), `:701` (exactly one level passes), `:706` (unjudged and unaccepted are ignored), `:712` (no sample gives n/a). On the real bank, `bank-audit.log:14` reports n/a. T-18 half (`difficulty_judged` recorded): nothing writes it. PR #13. |
| 34 | AC-23 slot is pure | ✅ Pass | `slot.py:55` is `slot_for_day(day: date, n_options: int = 5)`, importing hashlib and datetime only. bank-audit reports `ok slot path and ledger`. Passing: `test_the_real_slot_module_is_clean` (`test_audit.py:196`), `test_the_slot_lint_catches_what_it_claims_to` (:232), `test_slot_for_day_reads_no_file` (`test_slot.py:78`). Caveat: `schedule.arrange` (`schedule.py:189`) carries its own `_rng` copy (:166) outside the slot lint (GAP-1, PQ-51). PR #13. |
| 35 | AC-23a attendee simulation | ✅ Pass | bank-audit over 10,000 nights: most-used 19.9%, least-used 20.1%, not-last-night 20.0% (band 18–22%). `test_three_attendees_with_perfect_memory_stay_at_chance` (:148) passes, and the harness catches the balancer, never-repeat and skew generators. The simulation runs over `slot_for_day`, not `arrange`'s output (PQ-51). PR #13. |
| 36 | AC-23b generator both tails | ✅ Pass | χ²=3.46 (df=4, both tails) over 20,000 draws: A 4041, B 4000, C 4059, D 3993, E 3907. Passing: `test_a_skewed_generator_fails_the_upper_tail` (:120), `test_a_rebalanced_generator_fails_the_lower_tail` (:125), `test_a_generator_that_never_repeats_fails` (:132). PR #13. |
| 37 | AC-24 five options, one does-not-compile | ✅ Pass | bank-audit: all 4 questions. `test_every_real_question_has_five_options_and_one_does_not_compile` (:326) and `test_five_options_fails_on_a_malformed_record` (:345) pass. PR #13. |
| 38 | AC-25 no published distribution | ✅ Pass | bank-audit: 10 participant-facing files clean. `test_the_real_participant_facing_files_are_clean` (:355) and `test_a_published_distribution_is_caught[×6]` (:380) pass. An extra scan of `web/`, `docs/RUNBOOK.md`, `room/src` and the READMEs is clean. `PRD.md:346-353,773-774,974` carries answer-category priors outside the lint list; it is organizer-facing, and the list gap is GAP-22 / PQ-62. PR #13. |
| 39 | AC-26 enumerated tells | ❌ Fail (as written) | All five tells are measured (`test_six_lines_are_measured_and_five_are_ac26s`, :499). **The option-position tell is 5.00× chance** (index E, 4/4 vs 0.80, tail p=0.0016), and it **reports WARN with exit 0**. The criterion says *fails above 1.5× chance*. The code fails only if the ratio is > 1.5 **and** the tail is ≤ 0.01/m = 0.001 (`audit.py:1053-1059`), pinned by `test_the_answer_written_last_warns_at_four_and_fails_at_five` (:550): this is F-48, PQ-48. New since the interim: #47's `schedule.arrange` redraws the visible order, so this tell now measures the bank's stored order. `audit.py:803-806,1011` still calls that order "visible" (GAP-26). Stored order still reaches a screen through `python -m popquiz.fallback q3 --bake` (GAP-2). PR #13. |
| 40 | AC-27 unsafe parity | ✅ Pass | Trivial on the real bank (no UB answer). `test_a_ub_answer_with_unsafe_nowhere_else_fails` (:656), `test_a_non_ub_question_containing_unsafe_restores_parity` (:662) and `test_only_accepted_questions_count_toward_parity` (:667) pass. PR #13. |
| 41 | AC-71 take-it-home sentence, wall claims none | ⚠️ Partial | Take-it-home: `home.test.js:199` asserts the page ends with *The machine checked the answer only. An organizer approved the explanation.* (`copy.js:210`, `copy.rs:110`). Wall: `wall.test.js:156` asserts no explanation prose, the nine `receipt_*` strings name no explanation, and the fallback reveal in the c11 browser shows 4 receipt lines and no explanation. **No test asserts the clause "the wall receipt contains no sentence about the explanation."** PRs #34, #21. |
| 42 | AC-72 affirmation gate | ⚠️ Partial | Built half (PR #47): `schedule.refusal()` (`schedule.py:127-158`) refuses with a hard error any question that is unverified, used, unaffirmed (`affirmed_by` or `affirmed_at` missing) or not accepted. Passing: `test_ac72_an_unaffirmed_question_is_refused_and_nothing_is_sent_or_written`, the half-affirmed and not-accepted variants, `test_g12_the_gate_agrees_with_the_reserve`, `test_ac72_every_committed_record_is_refused_today`. Live: `popquiz.schedule schedule q3 --no-push` refused (*q3 is not affirmed … cannot be scheduled (AC-72)*) and wrote nothing. **Absent: the writer.** Nothing records who and when (`bank.py:196-206` only reads them), and the *< 2 trace steps cannot be affirmed* rule is left to T-18 (`schedule.py:134-136`). F-52, F-57, F-15. |
| 43 | AC-73 quoted output checked | ⛔ Blocked | No PR; PQ-23 never dispatched. Only the candidate lister `bank.quoted_outputs` (`bank.py:466`, PR #10) exists, and by its own docstring it *does not do that comparison*. Nothing blocks acceptance. |
| 44 | AC-74 provenance markup | ✅ Pass | Wall: `wall.test.js:266` (answer and receipt under `.by-machine`, no human marker). Host: `host.test.js:168,180` (beats under `by-human` with ✎, no machine marker). Home: `home.test.js:136`. The markers differ in border, glyph and colour (`components.css:135-140`). The review surface is not covered (T-18 unbuilt). PRs #21, #22, #34. |
| 46 | AC-75 reserve count and trend | ✅ Pass | `test_ac75_the_reserve_and_trend_are_the_first_lines_of_every_command` (`test_schedule.py:492`), `…count_is_what_in_reserve_admits` (:483) and `…trend_counts_what_comes_in_and_what_went_out` (:505) pass. Live `popquiz.schedule reserve`: *reserve: 0 ready … trend: +0 accepted awaiting affirmation, +4 not yet reviewed, -0 used*. The "first screen" is the CLI's first lines, since there is no organizer UI. The count is 0 because nothing is affirmed (F-52). PR #47. |
| 47 | AC-76 low-reserve warning | ✅ Pass | `test_ac76_the_warning_fires_below_the_threshold_and_not_at_it` (:515) and `test_ac76_the_default_lead_time_is_two_meetups_and_both_are_configurable` (:524) pass. Live: *warning: 0 ready is below the threshold of 2 … lead time is 2 meetups*. PR #47. |
| 49 | AC-102 static fallback file | ✅ Pass (addendum) | **Run as written in the addendum, on the shipped, unmodified file:** headless WebKit 26.5 through Playwright 1.62.1. Every http(s) and ws request was intercepted and aborted from the first byte, and offline mode was on for the whole walk. Requests seen: 1 (the document itself); network requests: 0; `performance` resource entries: 0; CSP errors: 0. Real key presses: `Space` went idle→live→closed→split→work 1/6. `→` stops at 5/6 (≤ M-2). `Space` enters reveal at 6/6 (M-1). `←` goes down to 1/6 and clamps. `Space` reaches released, where `→` is inert. `Esc` goes released→reveal 6/6→work. work: 0 ✓, no receipt, 0 colour spans. live/closed/split: 13 colour spans. reveal: 6 ✓. Static vs room parity holds for 15 of 16 states (from the c11 run). `released` differs only in its link and QR. Log: `/tmp/claude-501/validate/F/run.out`. PR #24. |
| 50 | AC-102 host sheet | ✅ Pass | `test_the_sheet_holds_every_beat_and_note_verbatim_in_phase_order_and_nothing_else` (`test_fallback.py:237`), `…one_block_per_phase_in_the_rooms_order` (:256), `test_write_fallback_writes_the_file_and_the_sheet_beside_it` (:273) and `test_a_refused_record_writes_neither_file` (:281) pass. The schedule side: `test_ac102_one_call_writes_the_file_and_its_sheet_from_the_arranged_record` (`test_schedule.py:358`) and `test_ac102_no_push_writes_the_files_and_sends_nothing` (:375). PRs #24, #47. |
| 52 | AC-28 join by link or code | ✅ Pass | `ac28_join_by_code_in_any_case_and_nothing_else`; `ac28_the_short_link_and_the_typed_code_both_join` (`GET /<code>` gives 303 to `/join?code=`, both give 201); `buzzer.test.js:132-159` (one input, one button). PRs #18, #23, #26, #34. |
| 53 | AC-29 six failure states | ✅ Pass | `ac29_each_failure_has_its_own_sentence` (6 messages, 6 slugs) and `buzzer.test.js:173` pass. `copy.rs:30-36` and `copy.js:72-78` match `SPEC.md:622` character for character, each with a next step. `NotYetOpen` is never emitted by the server (Drift). PRs #18, #23, #30, #42. |
| 54 | AC-30 capacity before session | ✅ Pass | `ac30_the_201st_join_is_refused_and_reserves_nothing`: 409 `full`, no token, `session_count`==200, host view identical. PR #18. |
| 56 | AC-32 no source on a phone | ✅ Pass | `just canary` 12/12. The participant rule (`canary_scan/mod.rs:714-746`) runs on every buzzer surface, page, rendered buzzer and traffic item in all 7 phases; `the_rules_catch_a_planted_leak` is the positive control. PRs #25, #31. |
| 57 | AC-33 wall fits, phone no h-scroll | ⚠️ Partial | Take-it-home: `web/home/measure.html?run=1` in the c11 browser (WebKit) reports PASS over 28 renders. At 375 px the widest bank question, q4 (78 chars), gives `scrollWidth 375 = clientWidth 375` and `pageScrolls false`, with code wells scrolling inside. The 375 px came from iframes, since viewport emulation is unsupported. **The review half is Blocked**: T-18 was never built. PR #34. |
| 59 | AC-34 last answer wins | ✅ Pass | `ac34_five_changes_then_close_then_refused_with_the_saved_answer` (A,C,B,E,D leaves D; 409 after close restates D) and `buzzer.test.js:364`. PRs #18, #23. |
| 60 | AC-35 exactly one of saving/saved/failed | ✅ Pass | `buzzer.test.js:331` (every 4×4×4 sequence: exactly one state after every event, >400 checks) and `ac35_response_contract`. PRs #23, #18. |
| 61 | AC-36 failed write is safe | ✅ Pass | `buzzer.test.js:354` and `ac36_a_failed_write_leaves_the_previous_answer_intact` (6 failure kinds). PRs #23, #18. |
| 64 | AC-39 reveal surfaces | ⚠️ Partial | Wall: `wall.test.js:156` (✓, totals, the named option with its count, receipt, no explanation) and canary `mod.rs:834-841`. Host: `host.test.js:168`, three beats. **The buzzer shows all five bars with counts from split on** (`buzzer.test.js:233-272`); AC-39 says *the correct letter and the participant's own count, nothing more*. F-49 (PQ-37 #31 amended SPEC §4, not AC-39). PRs #21, #22, #23, #31. |
| 65 | AC-40 ✓ as well as colour | ⚠️ Partial | Wall `wall.test.js:229`, buzzer `buzzer.test.js:249,296`, home `home.test.js:119`, `check.test.js`, `buzzer_page.rs:184`. The ✓ was also seen in the browser on the fallback reveal and the buzzer reveal. **The review half is Blocked** (T-18; `check.test.js:3-5`). PRs #9, #21, #23, #34. |
| 67 | AC-42 trope check | ✅ Pass | `copylint.test.js:131` (no copy-module entry matches), `:101` (every trope group fires on `bank/fixtures/copy-lint/retired.json`), `:96`, `:136`. Python twin: `test_every_trope_group_fires_on_the_retired_strings`. Prose produces warnings only: `test_check_prose_never_raises` (8 params), and a live q3 warning did not fail the run. Today the warning reaches only pytest's summary, since the review screen is T-18. PR #38. |
| 69 | AC-43 receipt exact lines | ✅ Pass | `test_each_fixture_renders_the_lines_the_spec_gives_it` (9 fixtures, order checked). UB-declared gives *✓ Miri flagged undefined behavior*; panic gives four lines; q8 E0502 gives three, tied to the bank by `test_the_q8_fixture_agrees_with_the_migration_and_the_bank`. Wall lines: `copy.test.js:138` and `twins.rs:46` (no terminal punctuation, none of the five claim words), plus `test_no_rendered_line_overstates`. PRs #21, #16. |
| 72 | AC-45 host actions, exactly these | ✅ Pass | `every_state_and_command_does_exactly_what_the_table_says`, `the_host_actions_are_exactly_the_eight_of_ac_45`, `every_host_route_exists_and_nothing_moves_without_the_host` (`/skip` gives 404), and `host.test.js:96`. The labels equal `SPEC.md:630` and `copy.js:131-138`. F-53: `EVALUATION.md:107` still names the pre-HC-0 strings. PRs #15, #22. |
| 73 | AC-46 live counts | ✅ Pass | `ac46_counts_move_on_the_host_phone_while_live` ((0,0)→(2,0)→(2,1)→(2,2)→(1,1)), `ac46_the_host_socket_counts_move_while_live`, `host.test.js:128`. PRs #18, #20, #22. |
| 74 | AC-47 host has no answer pre-reveal | ✅ Pass | The canary's host pre-reveal rule (`mod.rs:757-831`), `ac47_no_page_or_payload_the_host_sees_carries_the_answer_before_reveal` and `host.test.js:150`. PRs #25, #22. |
| 75 | AC-48 hint is free and private | ✅ Pass | The hint appears only in the live buzzer view (`mod.rs:748-755`). Taking it sends no request and leaves the wall and host unchanged (`mod.rs:1418-1427`). Session fields are `[token, answer]` (`ac57_a_session_is_a_token_and_an_answer`). `buzzer.test.js:449,457`. PRs #23, #25. |
| 76 | AC-49 host screen shape | ⚠️ Partial | One primary action and the code on every screen: `ac45_ac49_…`, `host.test.js:77,89`. **The phase label is announced, not shown.** It is in the payload and spoken through the live region (`a11y.test.js:681`, all 7), but SPEC §11 (`SPEC.md:628`) says *not shown*, and `host.test.js:81` asserts it is absent (PQ-34 #27, HC-0 trim). AC-49 says every host screen *states its phase*. PRs #22, #27. |
| 83 | AC-56 nothing survives release | ✅ Pass | `ac56_nothing_per_person_survives_release`, `ac56_the_totals_expire_with_the_released_room`, `ac56_a_released_room_serves_no_question_content_and_no_count`, `ac56_the_survivors_carry_no_count`. There is no schema: no DB crate, no `.sql`/`.db`, no `fs::write` in `room/src`. The released room shell keeps its anonymous totals until the 4 h expiry, which matches AC-56's text. PRs #30, #34. |
| 84 | AC-57 no identity anywhere | ✅ Pass | Static. `Session {token, answer}` (`sessions.rs:49-56`, exhaustive pattern); `UsedRecord`/`UsedEntry` (`used.rs:33-57`); client state is room-scoped `sessionStorage` only (`buzzer.js:729`, `host.js:205-209`). Passing: `ac57_no_participant_identity_in_any_record`, `ac57_nothing_crosses_rooms_but_the_question`. PRs #30, #23, #18. |
| 85 | AC-58 recap computed on the phone | ✅ Pass | `assert_nothing_personal_after_close` (`mod.rs:1601-1639`): after close there is no `/answer` fetch and no letter, only the ws re-attach. `buzzer.test.js:390`. PRs #25, #23. |
| 87 | AC-60 canary on every pre-reveal path | ✅ Pass | `just canary` 12/12 over Wall, Host, Buzzer, BuzzerOther, Page, RenderedWall, RenderedBuzzer, PageTraffic (including ws frames) and TakeHome. Positive controls: `assert_the_plants_arrived` and `the_rules_catch_a_planted_leak` (19 plants). JS state is covered through frames and rendered HTML; the real-socket `canary_full` belongs to test-full. PR #25. |
| 88 | AC-61 sealed answers module | ✅ Pass | Static. The secrets are private fields of `answers::vault::Vault` in a private module (`answers.rs:456-545`), read only via `open(&RevealWitness)`. `RevealWitness` is built in `Machine::revealed()` under `Phase::Reveal`, and pre-reveal builders take `PublicView` only. 6 `compile_fail` doctests and `boundary.rs` 6/6 pass, as does `ac61_a_pushed_answer_is_reachable_only_through_the_sealed_module`. Caveat: GAP-9, a forged `RevealWitness` survives `boundary.rs` under mutation. PRs #15, #19. |
| 89 | AC-62 option text is public | ⚠️ Partial | Runbook half holds: `docs/RUNBOOK.md:273` carries the sentence verbatim (equal to `SPEC.md:540`), and `runbook.test.js:59` passes in `just test`. **Host's first screen: not met as written.** The sentence is in no file under `web/host` and not in `COPY`; `copy.test.js:117` asserts it was removed (PQ-34 #27), and `runbook.test.js:4-7` names the runbook as the only carrier. PRs #48, #27. |
| 90 | AC-63 host can infer, not a guarantee | ✅ Pass | `docs/RUNBOOK.md:275` carries it verbatim (`SPEC.md:540-543`), and `runbook.test.js:64` passes. Its pass condition names the runbook only ("Sentence present verbatim"), which holds. The host-screen removal is under row 89. PR #48. |
| 92 | AC-99 colour scoped | ✅ Pass | `wall.test.js:214` and `wall-static.test.js:137`: spans > 0 in live, closed and split; 0 in work and reveal at every step; no source in idle or released. The `colour` field is at `view.rs:459/477/582`; also `room_record.rs:266` and canary `mod.rs:827-830`. PRs #9, #21, #24. |
| 93 | AC-79 wall is inert and clean | ✅ Pass | `assert_no_control` (`mod.rs:903`) on the served wall and the rendered wall in every phase, plus the wall plant rules and `wall.test.js:314`. PRs #21, #25. |
| 94 | AC-80 no dark mode | ✅ Pass | Static. `rg prefers-color-scheme\|color-scheme web room/src` finds nothing, and the runtime stylesheet scan on all four surfaces finds 0 such rules. Three dormant `[data-room="dim"]` selectors ship (`components.css:116,141,142`), but nothing sets `data-room`. No test in `just test` asserts the absence. PR #9. |
| 96 | AC-100 fit at the floor | ✅ Pass | `test_each_fit_fixture_raises_exactly_its_own_flag[passing/too-long-option/too-long-program]` (`test_fit.py:188`), `test_the_fixture_set_is_one_of_each` (:194), `test_option_length_is_one_line_of_at_most_29` (:165), `test_the_constants_match_typemodel_js` (:276). bank-audit: every question fits. The method names a `just bank-audit` fixture set, but the CLI has no fixture mode, so the fixtures run under `just test`. PRs #13, #9. |
| 98 | AC-82 | ✅ Pass (addendum) | **Real key presses in the addendum** (headless WebKit 26.5). Key presses were Option+Tab: Safari/WebKit on macOS tabs only to text fields unless Option is held or *Press Tab to highlight each item* is on (platform default, not a page defect). Every stop showed `:focus-visible` true with a 2–3 px solid amber ring. **Buzzer:** input, then *join*. Tab reached letter C, Space saved it ("Saved, C."), and focus stayed on C. **Host, keyboard only:** Enter on *Start* → *Close answers* → *Show the room its split* → *Trace* → *Reveal* → *End Pop Quiz*. After each press focus sat on the next primary action with its ring, and the region read each phase label. In work, `→` is reachable and Enter steps 1/6→2/6 with focus kept; `←` is disabled at step 1 and correctly skipped. **Take-it-home:** two source scrollers, *next step*, and the `-Vv` block. **Wall:** one stop, the reading region. Logs: `F/run2.out`, `run3.out`, `run4.out`. PRs #37, #41. |
| 99 | AC-83 live regions | ✅ Pass | A MutationObserver on the polite region in the c11 browser read every buzzer string (*The question is on the screen.*, *Saving.*, *Saved, C.*, *Couldn't save; your last answer is safe.*, *Hint shown, only to you.*, closed, split, walking, *Revealed: it was E.*, released) and the host's 7 labels, all verbatim against `SPEC.md:627-628`. Run against the a11y harness's scripted room, not a live room. PRs #37, #41, #23. |
| 100 | AC-84 AA contrast | ❌ Fail (as written; F-46, PQ-43 held) | Computed in WebKit over every visible text element in every state. Everything is ≥ 4.67:1 **except the dimmed trace line numbers, `span.rn-src-ln`, at 3.57:1** (needs 4.5:1), on the wall (22.1 px) and home (13 px) in work and reveal steps 2–5. The `.btn`, `.buzz` and step-button edges are 1.41:1. AC-84 also names a dim-room presentation, which does not exist. Remedy is on PQ-43 (`--text-primary` at `wall.css:112`, `home.css:92`). PRs #37, #41. |
| 101 | AC-85 44 px targets | ✅ Pass | `getBoundingClientRect` in the browser. Buzzer: letters 351×79–118, join 351×44, hint 351×44, retry 112×44. Host: primary 448×44, steps 44×44. Home steps 44×44. Nothing under 44 px on the buzzer or host in any state. PRs #37, #41, #23. |
| 102 | AC-86 reduced motion | ✅ Pass (addendum) | **The preference actually switched on in the addendum** (headless WebKit, `reducedMotion: reduce`; `matchMedia` true). On wall (after four phases), buzzer, host and home: `getAnimations()` 0 and 0 elements with any transition or animation over 1 ms. The reduce blocks set 0.01 ms, the standard "off" value. Text is legible on each. With no preference set, the result is also 0, so nothing animates either way. Log: `F/run2.out`. PRs #37, #41. |
| 103 | AC-89 one question, no next | ✅ Pass | `Command::all()` has 10 entries and none is a next question (`phase_table.rs:70`). Released allows only *Run it again*, which makes a new room (`released_never_re_enters_the_room`). `/skip` gives 404, and a room's question is fixed at construction. No test is named for AC-89. *Run it again* on a fresh question is in Drift. PRs #15, #30. |
| 107 | AC-92 `used` written at release only | ⚠️ Partial (one clause fails) | Room half passes: `ac92_used_is_written_at_release`, `ac92_nothing_is_written_before_release`, `ac92_an_expired_or_quiet_room_records_nothing`, `ac92_scheduling_questions_writes_nothing`, `g10_the_release_transition_is_the_only_writer` (`used.rs`). Pipeline half: `test_ac92_schedule_writes_nothing_under_the_bank` (`test_schedule.py:391`), and `used` is written only by `schedule.merge_used` from the room's ledger. **Fails: *`build_deck.py`'s ledger write is gone*.** `mvp/tools/build_deck.py:220-234` (`slot_for_meetup`) still writes `answer-history.json` (:233) on the `--only` path (:250), the path CLAUDE.md tells organizers to use. `ledger_violations` scans `pipeline/src` only. GAP-20, PQ-46. PRs #30, #13, #47. |
| 108 | AC-93 reveal only via the walk | ✅ Pass | `no_transition_skips_a_phase_and_reveal_needs_the_walk_through`. PR #15. |
| 109 | AC-94 no ✗ | ✅ Pass | `buzzer.test.js:306` (7 phases × 6 states: no ✗, *wrong*, *incorrect* or red), `home.test.js:119`, `check.test.js:187`. PRs #23, #34. |
| 110 | AC-95 most-chosen incorrect, same everywhere | ⚠️ Partial | Proven: `the_most_chosen_incorrect_option_is_named_the_same_on_wall_and_host`. Its fixtures: [2,0,0,0,40] names A while E is correct; ties go to the lower letter; the wall line equals the host heading. Also `nobody_read_it_another_way` and `the_why_tempting_text_is_the_named_options_own`. Take-it-home has no count (`ac56_the_survivors_carry_no_count`, `home.test.js:98`). **Blocked half: *Affirm refuses a missing `why_tempting`*.** T-18 was never built, and `schedule.refusal` does not check it. Downstream refusals exist in `fallback.bake` (`fallback.py:132`, which runs before any push) and the room loader (`answers.rs:619-629`). F-18. |
| 113 | AC-97 the walk-through phase | ✅ Pass | `walk_bounds` (work 0..M-2 and refuses the next step; reveal enters at M-1; checked at M=2, 5, 6) and `work_shows_steps_up_to_m_minus_2_and_reveal_the_rest`; `wall-static.test.js:97,121`. Canary `mod.rs:807-830` and `assert_the_plants_arrived` (a step past M-2 gives 409). PRs #15, #21, #25. |
| 114 | AC-98 forbidden-copy lint | ⚠️ Partial | Holds: the patterns equal `SPEC.md:640` verbatim and in order (`copylint.test.js:53`), are case-insensitive (:66), and give zero hits over every `COPY` entry including #42's `join_transport_failed` (:126). *No UI counts or waits on a contribution*: `copy-freeze.test.js:482` (the host's primary is enabled at 0/0). **Not every participant-facing string is linted:** `trace.steps[].note`, shown on the wall, host and take-it-home, is outside the lint's homes (`copylint.py:158-179`). F-47, PQ-42, GAP-12. Also F-51: AC-98's *no count displayed* vs AC-46's counts. PRs #38, #42. |
| 116 | AC-64 role check, stand-in, live | ✅ Pass | Against the Discord mock: `ac64_a_member_with_the_role_creates_a_room` (201) and `ac64_a_member_without_the_role_is_denied` (403 `wrong_role`). The feature-off test became `ac64_no_stand_in_survives_in_src_cargo_dockerfile_or_ci` (`auth.rs:109`); `[features]` is spike, smoke and burst only. Also `ac64_any_bearer_that_is_not_an_organizer_session_is_401`. *Stand-in build refuses* is moot, since the stand-in was deleted (PQ-13). PRs #26, #32. |
| 118 | AC-65 roles not permissions | ✅ Pass | `ac65_an_administrator_and_owner_without_the_role_id_is_denied`, `ac65_the_role_id_is_compared_whole`, `ac65_the_member_record_is_read_for_roles_alone`. PR #32. |
| 119 | AC-66 refresh rotation persisted | ✅ Pass | `ac66_each_refresh_persists_the_rotated_token_and_presents_it_next`, `ac66_two_creates_at_once_refresh_once`, `ac66_a_refused_refresh_signs_the_organizer_out_and_leaves_their_open_room`, `ac66_an_access_token_refused_early_is_refreshed_once`. PR #32. |
| 120 | AC-67 participants never authenticate | ✅ Pass | Static. `/join` (`routes.rs:147,227`), `PUT …/answer` (:183,228), the buzzer `show` (:124) and the ws attach (`ws.rs:333-336`) use the session map only. `ac67_no_participant_route_touches_the_auth_module` (`auth.rs:280`) and `ac67_participants_need_and_carry_no_credential` (:301) pass. PRs #17, #23, #32. |
| 121 | AC-68 only the creator controls | ✅ Pass | `ac68_organizer_b_cannot_read_or_control_as_room` (401 with a null body on GET and 6 actions), `ac68_an_organizer_session_is_not_a_host_session`, `ac68_organizer_b_cannot_run_it_again_on_as_room`. PR #32. |
| 123 | AC-70 denial wording | ✅ Pass | `ac70_a_non_member_and_another_guilds_member_get_the_same_wrong_server` (byte-identical) and `ac70_a_member_without_the_role_hears_wrong_role`. PR #32. |
| 124 | AC-101 admin channel | ✅ Pass | `admin.rs` 15/15, including `ac101_the_right_token_schedules_and_reads_the_used_ledger`, `ac101_a_missing_or_wrong_token_is_refused_saying_nothing` (9 credentials × 8 paths × 6 methods, all 401 with an empty body), `ac101_the_admin_prefix_is_served_to_the_check_alone`, `ac101_g9_no_other_module_reads_the_token` and `g9_one_constant_time_compare_and_no_log`. `just secret-scan` EXIT 0 (`ac101_the_repository_holds_no_admin_token`, `…scan_catches_a_plant`). GAP-17: the route-table test reads `routes.rs` only. PR #33. |
| 125 | AC-101 canary plant | ✅ Pass | The canary's admin rule (`mod.rs:681`) runs on every surface and phase. `the_admin_rule_catches_a_planted_token` and `the_binary_logs_no_admin_token` pass; the latter pushes to the real binary, then scans its stdout and stderr. PRs #33, #25. |

## Drift from BUILDPLAN.md

- **Two live rooms on one question leak the answer (G-3; no row covers it).** `create_for` (`room/src/rooms.rs:846-860`) refuses only a *used* question. The guard at `rooms.rs:807` ("A room is running {id}") protects `schedule` (a re-push), not room creation. So an organizer can create a second unreleased room on the night's question. Releasing either one writes `used` and rebuilds the global take-it-home (`/last`, `rooms.rs:925`, `routes.rs:639`) with the answer, the explanation and the receipt, while the other room is still pre-reveal. Read on this tree; GAP-8 in the guardrail audit (with a probe test there). A host is unlikely to do this by accident, and *Run it again* refuses the used question. Still, it is the one path on the tree where the answer is public before a room's reveal. **The most important item in this report.**
- **The standalone fallback bakes stored order (G-1).** `python -m popquiz.fallback q3 --bake` emits the answer at E with no date, no `arrange`, no affirmation and no used check (`fallback.py:306-333`, pinned by `test_fallback.py:288`, documented at `room/README.md:595-596`). `popquiz schedule` does the right thing (`schedule.arrange`, `schedule.py:189-208`). A fallback built the other way puts the answer at E every night. GAP-2, PQ-52.
- **G-12 is a gate with no key (F-52, F-15, F-57).** `schedule.refusal` refuses all four committed records today (`test_ac72_every_committed_record_is_refused_today` is a deliberate tripwire). Nothing writes `affirmed_by`/`affirmed_at`: T-18 is held on F-15. The reserve reads 0. `EVALUATION.md:185`'s fallback (*the verified MVP bank, already affirmed by use*) has no recorded form. Affirmation also records who and when, not what, so an edited text would still schedule as affirmed (F-57). And the room accepts a pushed question with no affirmation (GAP-25), so the gate is client-side only.
- **`build_deck.py` still writes the build-time ledger (G-10, row 107).** `mvp/tools/build_deck.py:233`, reached from `--only` at :250. CLAUDE.md tells organizers to build the segment's deck with `--only <qid>`, so the documented MVP path still retires a question at build time. That is exactly the 2026-08-12 incident. `mvp/answer-history.json` is empty today, so no false record exists yet. GAP-20 / PQ-46; GAP-3 (the multi-question `answer_slots` balancer, `build_deck.py:96-109`) rides with it.
- **The arrangement lives outside the slot lint and the attendee simulation (G-1).** `schedule.arrange` uses its own `_rng` copy (`schedule.py:166`) for the four incorrect options. The slot lint and the AC-23a simulation cover `slot.py`, not `arrange`'s output. GAP-1 / PQ-51. Relatedly, the option-position tell (row 39) now measures stored order, which `arrange` discards; `audit.py:803-806,1011` still says that order is visible (GAP-26 / PQ-48).
- **Provenance and the pin are tested but not enforced at scheduling (G-2).** `check_provenance` runs in CI over committed files only, not in `schedule.refusal` or `fallback.bake` (GAP-4 / PQ-53). `verify.is_stale` has no caller (row 9; GAP-5 / PQ-58). F-55 was reproduced: a self-consistent hand edit of `stdout` plus the option text passes `check_provenance`, and only the recording replay catches it.
- **AC-1 holds by convention.** No static dependency check exists (row 1). The interim asked for one; nothing changed.
- **External #35 (`1263add`, what Fly v11 runs).** *Welcome to the POP QUIZ* is still a `PROPOSED_` entry (`room/src/copy.rs:116`, `web/shared/copy.js:223`) with no §11 row (F-50). The client's own line, so the client rules.
- **External #36.** The hand-typed `verified` blocks for q4, q7 and q8 are fully replaced by machine writes: digest, pin, byte-identical round-trip and replay all match (row 11). No residue.
- **External #42 edited SPEC §11 directly** (the *Buzzer, join connection failure* row, `SPEC.md:621`). The string is byte-identical in `copy.rs`/`copy.js`, linted and twin-pinned. The owner's prerogative; consistent.
- **External #43** (WOFF2 fonts) and **#44** (Node 24 actions): no criterion regressed. **#7** is the contract this audit read.
- **AC-59 cut from the buzzer (F-41).** A deliberate non-meet of a criterion the client owns (row 86 is smoke-side).
- **F-49 (row 64):** the buzzer shows five bars from split on; AC-39 still says letter and count. **F-53 (row 72):** `EVALUATION.md:107` names stale action strings; the code follows §11. **F-54:** §11's *no incorrect votes* row vs §4.5; the shipped host page and the runbook follow §4.5. **AC-49 vs §11:628 (row 76):** the host phase label is announced, not shown. **§6 vs the host screen (row 89):** the §8.1 sentences live in the runbook only.
- **F-15** (a does-not-compile question can never be affirmed as written) is unresolved, and it is still why T-18 is held. **F-18** and **F-19** are implemented as ruled. **D-15:** the bank holds q3 as authored plus re-authored q4, q7 and q8; every option is ≤ 29 characters and every question fits. Stale comments name q7 as widest (`web/home/measure.html`, `test_fit.py:201`); q4 is (F-40).
- **F-46, F-47, F-48** stand as recorded (rows 100, 114, 39). **F-55…F-57** are routed upstream and are visible on this tree (rows 11, 38, 42).
- **The used ledger and organizer store are in memory only** (`used.rs:59`). A Fly restart erases AC-92's record unless `popquiz sync` ran first, and the justfile's *restart before the next run* (`justfile:249-251`) makes skipping `sync` lossy. **Run it again on an unused question** opens a second question the same night (`used.rs:236`), which is in tension with AC-89 and AC-92's one question per meetup. SPEC.md:179 allows it.
- **Smaller items.**
  - `JoinRefusal::NotYetOpen` is never emitted; joins are open from creation.
  - The released fixture frame uses the default URL, so static/room parity skips `released`.
  - The shipped fallback's CSP blocks c11's injected script, which is fine for the product.
  - Three dormant `[data-room="dim"]` rules ship.
  - `HOST_DEV_TOKEN` residue remains in `.env.example:39-40`, `fly.toml` and `justfile:204,240` (GAP-18 / PQ-50).
  - `PRD.md` carries answer-category percentages outside the AC-25 lint (GAP-22 / PQ-62).
  - `bank-audit` is still not its own CI step (GAP-23 / PQ-49).
  - The route-table test reads `routes.rs` only (GAP-17).
  - The deployed-burst workflow keeps an organizer session and the admin token as repo secrets (GAP-19).
  - `smoke`/`burst` default to the real q3 (GAP-21 / PQ-41).

## Gaps

- **AC-3, AC-4, AC-5** (rows 4, 5, 7), and AC-1's static check (row 1): T-16 never built (H-4).
- **AC-19, AC-22, AC-73** (rows 28, 31, 43), the writer half of AC-72 (row 42), and the review halves of AC-17, AC-33, AC-40, AC-74, AC-88 and AC-95 (rows 26, 57, 65, 44, 32, 110): T-18 never built (F-15). **G-12 is gated but has no affirmation writer.**
- **AC-26** (row 39): built, but the option-position tell cannot fail at the current bank size (F-48).
- **AC-84** (row 100): F-46, held on PQ-43.
- **AC-92** (row 107): `build_deck.py`'s ledger write was never retired (PQ-46).
- **AC-98** (row 114): trace notes unlinted (F-47, PQ-42).
- **AC-62's host-screen half** (row 89) and **AC-49's on-screen label** (row 76): removed by the HC-0 trim.
- **AC-8's run-level rejection tally** (row 12): T-16's run report.

## Recommendations

**Fix-back-in-flight** (small, held tickets already exist; the Orchestrator and client decide when):
- **GAP-8:** refuse `create_for` while another unreleased room holds the question, with a test. It is one condition at `rooms.rs:852`, and it closes the only pre-reveal answer leak found.
- **PQ-52 (GAP-2):** make the standalone fallback require `--date` and `arrange`, or remove the CLI in favour of `popquiz schedule --no-push`.
- **PQ-46:** retire `slot_for_meetup`'s write and `answer_slots` in `mvp/tools/build_deck.py`, or retire the MVP deck. CLAUDE.md's `--only` instruction currently writes the ledger.
- **PQ-58, PQ-53:** call `is_stale` and `check_provenance` in `schedule.refusal` and `fallback.bake`.
- **PQ-43 (F-46):** `--text-primary` at `wall.css:112` and `home.css:92` gives 4.78:1. The 1.41:1 edges still need the owner's word.
- **PQ-42 (F-47):** bring `trace.steps[].note` into the lint's homes.
- Add the missing assertion for AC-71's wall clause (row 41), and a static dependency check for AC-1 (row 1).
- **PQ-49:** `just bank-audit` as a CI step.

**New tickets / client decisions:**
- **Before the first real meetup from the built app: how q3 (or any question) gets affirmed.** Either T-18 dispatched on F-15's amendment, or a documented organizer edit of the record (F-52). Today nothing can be scheduled.
- **F-48's ruling** on AC-26 at small bank sizes, and whether the tell should measure stored or arranged order (PQ-48).
- **GAP-25:** whether the room should refuse an unaffirmed pushed question, or the gate stays client-side and the contract says so.
- **PQ-51:** lint `schedule.arrange` like the slot path and simulate attendees over its output.
- The T-18 chain on F-15, and T-16 on H-4.

**Accept-as-is** (the owner's call; recorded so the strict reading is visible):
- AC-59 cut (F-41).
- The buzzer's five bars (F-49, amend AC-39).
- The §8.1 sentences in the runbook only (row 89) and the host phase label announced, not shown (row 76). Amend AC-49/§6 or restore the copy.
- *Welcome to the POP QUIZ* once it has a §11 row (F-50).
- AC-98's *no count* literal vs AC-46 (F-51).
- EVALUATION's stale AC-45 strings (F-53).
- *Run it again* on a fresh question in one night.

## What I couldn't verify

- ~~**The fallback with the network actually blocked (row 49).**~~ Done in the addendum. WKWebView in the c11 browser has no offline mode or request interception. Zero requests rests on `file://` with every server killed, 0 `performance` entries, the CSP and a static scan. Keys were stepped on a copy that adds `'unsafe-eval'`.
- ~~**A real Tab walk with the visible ring (row 98), the reduced-motion preference (row 102)**~~ — done in the addendum. **VoiceOver** is still unverified. Not drivable from c11; they need a human (web/README.md, Accessibility).
- **That the recordings and the q4/q7/q8 records came from the Docker image** rather than typing. `sandbox-build`/`test-verify-full` are out of scope (smoke rows 10, 13, 15, 17, 19).
- **The real-time reaper for the 4 h and inactivity expiries.** Proven on `ManualClock` only. Also the real-socket `canary_full`, which belongs to test-full.
- **`build_deck.py`'s ledger write dynamically.** Read statically, since running it writes a tracked file.
- **The `popquiz schedule` refusal's exit code by my own capture.** Only the tests assert the non-zero exit; the live run printed the refusal and wrote nothing.
- **Contract feedback.**
  - The plan's footer miscounts its own tags (8/74/44 stated, 7/76/43 actual), and the boot prompt said 83/43 correctly this time.
  - Rows 26, 44, 57, 65 and 110 mix a built and an unbuilt surface in one pass condition, which forces Partial.
  - Row 107 puts the room's ledger and the MVP tool's ledger in one row.
  - Row 96 says `just bank-audit` fixture set, but the CLI has no fixture mode.
  - Rows 98–102 say *`just a11y` via the c11 browser*, but `just a11y` is a node suite, so the browser half was improvised.
  - Row 49's *network blocked* is not achievable in the approved browser.
  - Row 116's feature-off and stand-in clauses are moot since PQ-13.

## Addendum — real-browser checks for rows 49, 98, 102 (client request, 2026-10-05)

The client asked for the three checks the c11 browser could not do. They ran in **headless WebKit 26.5** (the c11 browser's engine), driven by Playwright 1.62.1 from cached binaries. Nothing was downloaded or installed, and no tracked file changed. The page server was `python3 -m http.server 8766` at the repo root on loopback, stopped afterwards. The harness was `web/test/a11y/browser.html` (the repo's own, per `web/README.md`). Scripts and logs are in `/tmp/claude-501/validate/F/`.

This deviates from the plan in one way: the plan names the c11 browser. The engine is the same, but the driver is not. The client's request is the authority for the substitution.

- **Row 49:** the shipped `q3-fallback.html`, with no `'unsafe-eval'` copy. The network was blocked by interception for the whole load and walk, plus offline mode after load. WebKit's offline flag refuses `file://` documents outright, so interception covers the load. Zero network requests. Every phase and step rule held.
- **Row 98:** real Option+Tab, Enter and Space presses with a visible ring on every stop. The host was run end to end by keyboard. Finding: Safari's default Tab behaviour skips buttons on macOS. That is a platform setting a keyboard user controls, not a page defect, but the runbook could mention Option+Tab.
- **Row 102:** the preference was engaged (`matchMedia` true). No motion anywhere.
- **Not done: VoiceOver.** It needs a person listening; no automation hears it. Row 99's strings were already verified verbatim from the live region.

Result: rows 49, 98 and 102 move Partial → Pass. The totals are now **60 Pass, 15 Partial, 2 Fail, 6 Blocked**. The verdict stays 🟡 YELLOW for the reasons in the Summary.

## Addendum — deployed burst for rows 66, 79, 80, 81 (client request, 2026-10-10; Orchestrator seat, not the Result Validator)

The four burst rows were post-merge-smoke rows open at the time of this report. PQ-41 (PR #54, merged `8819b67`) made the harness safe to point at the deployed room, and the client ran it. This addendum records the evidence; the validator did not re-run it and the Pass/Partial totals above are unchanged (smoke rows are not in them).

**Run.** GitHub Actions `deployed-burst` run `38036365964` (https://github.com/mrojas54/rust-nyc-pop-quiz/actions/runs/38036365964), 2026-10-10 08:00Z, commit `8819b67`, against `https://rustnyc-popquiz.fly.dev`, 200 of 200 participants, deployed substrate. Reports `burst.json` and `smoke.json` are the run's artifact. It was the fifth dispatch of that morning; the four before it failed before any room was created (an admin token that was not the app's, including one stored as a literal string by a mistyped command, then an organizer session wiped by the restart that rotating the token caused), which is how the setup order in `validation-plan.md` came about.

| Row | Criterion | Measure | Value | Threshold | Result |
|---|---|---|---|---|---|
| 66 | AC-41 reveal fan-out | `runs.segment.reveal.p95_ms` | 94.98 ms | ≤ 2000 ms | pass |
| 79 | AC-52 200 sessions counted once | `runs.segment.reconcile.match` and burst-only | exact | exact | pass |
| 80 | AC-53 write p95 | `runs.segment.writes.p95_ms` | 160.53 ms | < 500 ms | pass |
| 81 | AC-54 deadline burst | `runs.burst_only.headline_p95_ms` | 166.85 ms | < 500 ms | pass |

Verdict line: `burst: all four criteria pass as measured`, `n=200 of 200`, exit 0. Expected peak connections 403 against the cap of 500 (`fly.toml` `hard_limit`, raised from 400 by the client on 2026-10-10 and live after the deploy of `8819b67`); not over the cap. Smoke on `smoke-q3`: SMOKE PASS, answer writes n=267 p50 85.9 ms p95 167.4 ms, reveal reaching all 200 buzzers p95 89.4 ms.

**What this does and does not establish.**
- AC-53's deployed clause (the PQ-41 note above) is evidenced by one run at 200 participants from the Actions runner (4 CPUs, `ulimit -n` 4096). One run is not a distribution; the figures are well inside the thresholds, not marginal (`marginal: false` on all four).
- The client's own connection (phones, venue wifi) is not in these numbers: AC-31 and AC-55 (rows 55, 82) remain HC-1/HC-4.
- The harness holds about 403 connections of the 500 the proxy routes to one machine; 500 is a routing limit, not a measured capacity of the VM. The first real meetup is the next test.
- Superseded findings in *Drift* above: the repo-level secrets for the workflow (GAP-19, now environment secrets behind a required reviewer on `main`) and the real-q3 default (GAP-21, now harness ids; bank ids are refused off loopback) are closed by PR #54.

## Handoff and run state at exit

- **This validator is done and exits.** It is one-shot: it fixed nothing, opened and edited no tickets, merged and pushed nothing, and never changed HEAD. `run-state.md` is the Orchestrator's file and was not touched; this report is the handoff.
- **Run state at exit.** Unchanged from the Summary: main @ `e239346`; board 38 done, 1 cancelled, 25 backlog (all held: PQ-21, PQ-23, PQ-30, PQ-41…PQ-43, PQ-46…PQ-64); Fly v11 = `1263add`.
- **For the Orchestrator (`tab:7`) to route:**
  1. GAP-8 (two rooms, one question) and GAP-2 (fallback at E) are the two guardrail holes on the tree.
  2. F-52/F-15 (affirmation) blocks the first real meetup.
  3. PQ-46 (build_deck ledger) bites the documented MVP path.
  4. F-48 and F-46 are the two Fails, and both await rulings.
- **Evidence on disk** (scratch, not committed): `/tmp/claude-501/validate/` holds `just-test.log`, `just-test-cold.log`, `canary.log`, `bank-audit.log` and `a11y.log`, plus per-bucket logs and scripts in `A/`, `B/`, `C/` (including the built `q3-fallback.html`), `D/` and `E/`.
- **The sandbox needs for anyone re-running this audit:**
  - The room tests bind loopback ports, so `just test` and `just canary` need the sandbox bypass.
  - `pipeline/` needs one `uv sync --frozen`, which uses the network.
  - Use `/usr/bin/python3`: pyenv's 3.12.10 is an x86_64 build and won't run on this arm64 machine.

## Operator smoke-pass checklist (post-merge)

Every `post-merge-smoke` row in validation-plan.md, copied through verbatim (43 rows, counted by script) for the operator to walk. The Result Validator attempted none of these.

| # | Criterion (ID) | Verification method | Artifact | Pass condition | runnable_at |
|---|---|---|---|---|---|
| 2 | AC-1 runtime | Run `smoke` with the pipeline binary absent | T-09 | Room created and run to release | post-merge-smoke |
| 3 | AC-2 generator re-runnable | `just test-full` generation-kill fixture | T-16 | Killed mid-candidate, re-run completes, no partial record in `bank/` | post-merge-smoke |
| 6 | AC-4 the connection is real | HC-2 | — | Organizer judges the first talk-mode batch | post-merge-smoke |
| 8 | AC-5 real figures | HC-2 first paid run | — | Figures replace ECONOMICS guess bands | post-merge-smoke |
| 10 | AC-6 real toolchain | `just test-full` on the T-15a image | T-15b | Same cases pass on the real toolchain | post-merge-smoke |
| 13 | AC-8 real HashMap program | `just test-full` | T-15b | Rejected on the image | post-merge-smoke |
| 15 | AC-9 real Miri | `just test-full` | T-15b | Same on real Miri, both configs | post-merge-smoke |
| 17 | AC-10 real | `just test-full` | T-15b | Same on real Miri | post-merge-smoke |
| 19 | AC-11 real rustc | `just test-full` | T-15b | Same on the image | post-merge-smoke |
| 20 | AC-12 sandbox containment | `just test-full` fixture suite on the T-15a image | T-15a | Socket, `/etc/passwd`, env var, over-allocation, infinite loop each contained and reported | post-merge-smoke |
| 29 | AC-20 one-screen review | HC-2 | — | Organizer reviews a real batch on one screen | post-merge-smoke |
| 30 | AC-21 review time | HC-2 stopwatch | — | A measured number replaces the 15-minute hypothesis | post-merge-smoke |
| 33 | AC-88 first sample | HC-2 | — | First real judged difficulties recorded | post-merge-smoke |
| 45 | AC-74 reads as two things | HC-1 | — | Client confirms | post-merge-smoke |
| 48 | AC-77 runs from reserve offline | `just test-full` network-blocked segment | T-20 | Segment completes with outbound blocked except the room's host | post-merge-smoke |
| 51 | AC-102 felt | HC-0 | — | Client opens the file offline, steps it, reads the sheet against it | post-merge-smoke |
| 55 | AC-31 join time on venue wifi | HC-1, HC-4 | — | Client-side join-to-lobby logged and reported | post-merge-smoke |
| 58 | AC-33 wall overflow measured | `just test-full` layout measurement | T-05 | Zero overflow wherever the state says *fits* | post-merge-smoke |
| 62 | AC-37 reconnect | `just test-full` socket drop | T-04c, T-06 | Saved answer survives; *paused* until fresh state | post-merge-smoke |
| 63 | AC-38 legible from the back row | HC-1, HC-4 | — | Felt | post-merge-smoke |
| 66 | AC-41 reveal fan-out ≤ 2 s | `just burst` against the deployed room | T-03, T-21 | p95 ≤ 2 s to 200 clients | post-merge-smoke |
| 68 | AC-42 read aloud | HC-1, HC-2 | — | Felt | post-merge-smoke |
| 70 | AC-43 not overstated | HC-2 | — | Felt | post-merge-smoke |
| 71 | AC-44 beginner explains it | HC-4 | — | Field notes | post-merge-smoke |
| 77 | AC-50 host resume | `just test-full` two devices | T-07 | Refresh survives; resume link attaches a second device; session never rotates | post-merge-smoke |
| 78 | AC-51 host never apologises | HC-1, HC-4 | — | Logged | post-merge-smoke |
| 79 | AC-52 200 sessions counted once | `just burst` deployed | T-03, T-21 | Each final answer counted exactly once | post-merge-smoke |
| 80 | AC-53 write p95 < 500 ms | `just burst` deployed | T-03, T-21 | p95 under 500 ms including the burst | post-merge-smoke |
| 81 | AC-54 the deadline burst | `just burst` deployed, separate report | T-03 | 200 writes inside 2 s reported separately and passing; measured before HC-0 | post-merge-smoke |
| 82 | AC-55 failed-request rate | HC-4 logs | — | Client reports from the room's logs | post-merge-smoke |
| 86 | AC-59 *nothing about you was recorded* | HC-1, HC-3 | — | Felt | post-merge-smoke |
| 91 | AC-78 legible at the real projector | HC-1 | — | Felt | post-merge-smoke |
| 95 | AC-81 one phase everywhere | `just test-full` wall + 200 buzzers | T-04c | Same phase within one broadcast of each transition | post-merge-smoke |
| 97 | AC-100 measured overflow | `just test-full` every bank question | T-05 | Zero overflow where *fits*; clipped edge where not | post-merge-smoke |
| 104 | AC-89 timing | HC-1, HC-4 | — | Clocked | post-merge-smoke |
| 105 | AC-90 scheduled last | Runbook + client | T-24 | Host script says *last*; client confirms the run-of-show | post-merge-smoke |
| 106 | AC-91 first-look time | HC-4 | — | Watched | post-merge-smoke |
| 111 | AC-95 affirmation | HC-2 | — | Organizer affirms each text | post-merge-smoke |
| 112 | AC-96 second beginner | HC-4 | — | Field notes | post-merge-smoke |
| 115 | AC-98 host behaviour | HC-1 | — | Watched | post-merge-smoke |
| 117 | AC-64 live Discord | Client's account against the real guild | T-10 | Room created; T-10's exit criterion, before HC-1 | post-merge-smoke |
| 122 | AC-69 runs with Discord down | `just test-full` | T-11, T-10 | Open room runs to release; no new room; dies at 4 h | post-merge-smoke |
| 126 | AC-101 live push | Client's laptop to the deployed server | T-20 | One push succeeds before the first real batch is scheduled | post-merge-smoke |
