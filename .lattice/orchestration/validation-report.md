# Validation Report

Source spec: [SPEC.md](../../SPEC.md)
Source build plan: [BUILDPLAN.md](../../BUILDPLAN.md)
Source validation plan: [validation-plan.md](./validation-plan.md)
Result Validator: agent:result-validator (c11 `tab:13`, fresh session; five in-process sub-agents for row verification, one report-writer)
Date: 2026-10-04
Run state at audit: interim — see Summary
Audit tree: `origin/main` @ `0a4b6fb4befdeddf6f000abc72586e8bb001db23` (merge of PR #45), detached checkout at `rust-nyc-pop-quiz-worktrees/validate-main`. Nothing committed, pushed or edited; no ticket touched.

## Summary

- **Total criteria audited: 83** pre-merge rows (7 `pre-merge-static` + 76 `pre-merge-runtime`). The plan's footer and the boot prompt say 82 and 44; the table itself carries 83 and 43 (counted by script). All 83 are walked here and all 43 smoke rows are in the checklist.
- **Pass: 55**
- **Partial: 15**
- **Fail: 2** (row 39, AC-26 option-position tell, as written; row 100, AC-84 contrast, F-46 open)
- **Blocked: 11** (every one: ticket never built — T-16, T-18, T-20, T-24)

**Run state at audit.** 33 tickets done, 1 cancelled (PQ-33), 10 open: five never built (T-16 PQ-21, T-18 PQ-23, T-20 PQ-25, T-23 PQ-28, T-24 PQ-29), one open bug (PQ-30), four follow-ups (PQ-41, PQ-42, PQ-43 held on owner decisions; PQ-44 in flight, not audited). The run is **not** complete; this is an interim terminal audit on the client's word ("run the validator", 2026-10-03). Deployed Fly v11 = `1263add`, ten PRs behind this tree, and not the subject of this audit.

**Shared harness runs on the audit tree** (logs under `/tmp/claude-501/validate/`):

| Command | Result |
|---|---|
| `just test` | EXIT 0, **16.3 s warm** (42.3 s with dependencies already compiled; the first, truly cold run took several minutes to compile and then failed only on sandbox denials — uv cache and loopback bind — and was rerun with the bypass). room: **236 passed, 4 ignored** (test-full halves); pipeline: **642 passed, 1 warning** (q3 `hint` trope warning, signpost "The key word"); web: **279 pass, 0 fail** |
| `just canary` | EXIT 0, 12 passed |
| `just bank-audit` | EXIT 0, `bank-audit: passed`; one **WARN** (option-position tell, 5.00× chance); report `bank/audit/2026-10-04.json` (gitignored) |
| `just a11y` | EXIT 0, 31 pass, matrix and every contrast ratio printed |
| `just secret-scan` | EXIT 0, 2 passed |

**Overall verdict: 🟡 YELLOW.** What was built largely meets the contract and is proven by its named tests. The canary, the sealed answer module, the phase machine, auth, the verifier's decision logic, the receipt and the bank audit all pass on the tree. The run is not done. Generation, review/affirm, scheduling/sync, the guardrail audit and the runbook were never built, so G-12 (affirm) and the scheduled path of G-1 have no code at all. The most important finding is outside any row. **Every bank question stores its correct answer at E, and nothing on this tree applies `slot_for_day` to arrange the options.** A room fed from this bank shows the answer at E every night (see Drift, first item).

## Per-criterion results

PRs cite the merge that carried each row's proof (map: boot prompt §2). "Log" = `just-test.log` unless named. Where a later PR changed what an earlier one proved, the tree was judged.

| # | Criterion | Result | Notes |
|---|---|---|---|
| 1 | AC-1 room never calls the generator | ⚠️ Partial | `room/Cargo.toml` deps are axum, tokio, serde, serde_json, getrandom, subtle, rustls, tokio-rustls, webpki-roots, futures-util, optional tokio-tungstenite (smoke/burst client). There's no generator, verifier or LLM crate, and no path dependency on `pipeline/`. **No static dependency check exists in `just test`** (`boundary.rs` checks unsafe and the vault only). That check is the T-16 deliverable, and PQ-21 was never built. |
| 4 | AC-3 count/topics/difficulty honoured | ⛔ Blocked | No PR; PQ-21 (T-16) never dispatched (H-4 API key). `pipeline/src/popquiz/generate.py` is a 4-line scaffold. |
| 5 | AC-4 talk mode tags a concept | ⛔ Blocked | No PR; PQ-21 never dispatched (H-4). |
| 7 | AC-5 cost and wall-clock in the report | ⛔ Blocked | No PR; PQ-21 never dispatched (H-4). No run-report code. |
| 9 | AC-6 record fields and the pin | ✅ Pass | `test_a_ran_record_holds_exactly_what_the_machine_observed` (`-Vv`, edition, triple, flags against recording and pin); `test_is_stale_matches_the_pin_it_was_verified_under`; `test_a_legacy_record_is_exempt_from_the_stale_check`; `test_a_toolchain_that_is_not_the_pin_is_refused_before_any_candidate_step[release/commit-hash/miri/host]`. `sandbox.py:279-286` compiles with the pin's edition and flags. PR #16. |
| 11 | AC-7 answer never hand-written | ✅ Pass | No bank record stores `correct`. It is derived by `bank.correct_index` (`bank.py:385`) and the room twin (`answers.rs:224`), and `fallback.py:142` and `build_deck.py:292` only emit the derived value. The digest recomputed over `verify.py`, `runner.py` and `sandbox.py` is `popquiz-verify 7b1bc351e4dd`, which equals `verifier_version` on q4, q7 and q8. rustc release/commit-hash, edition, flags and miri_version equal `pipeline/sandbox/pin.toml`. Round-trip through `bank.save_question` is byte-identical for q3, q4, q7 and q8. The PQ-39 replay (StubRunner over the recordings) was **rerun**: every field matches except `verified_at`. `test_a_hand_edited_answer_is_refused` passes. The #36 hand-typed blocks are gone. PRs #16; #40, #45 (records). Caveat in Drift: `check_provenance` accepts a *self-consistent* hand edit of `stdout` + option text, and only the recording replay catches it. |
| 12 | AC-8 varying output rejected | ⚠️ Partial | `hashmap-order` case gives `output_varied` (rejected, not flagged); `test_the_output_that_varied_is_counted_not_flagged`; q3, q4 and q7 accepted at RUNS=5. *Rejection count reported* holds per candidate only. The run-level tally is T-16's run report (`verify.py:91`), never built. PR #16. |
| 14 | AC-9 Miri UB handling | ✅ Pass | cases.toml: `ub-both`/ran gives `ub_not_declared`; `ub-both`/ub gives `accepted`; `ub-sb-only`/ub gives `borrow_models_disagree`. `test_tree_borrows_is_asked_only_for_declared_ub`. PR #16. |
| 16 | AC-10 Miri stdout ≠ native | ✅ Pass | `miri-differs` gives `miri_output_differs`; `verify.py:459-466` uses equality, not prefix. PR #16. |
| 18 | AC-11 declared non-compile | ✅ Pass | q8 E0502 accepted with codes recorded (`test_a_does_not_compile_record_records_every_code…`). A compiling candidate declared non-compile gives `compiled_but_declared_does_not_compile`. `test_the_receipt_renders_the_does_not_compile_list_from_a_record_verify_wrote`. PR #16. |
| 21 | AC-13 receipt from the record alone | ✅ Pass | `test_each_fixture_renders_the_lines_the_spec_gives_it` renders from `verified` only; `test_the_receipt_module_needs_no_toolchain_and_no_world` (receipt.py imports only `dataclasses` and `popquiz.bank`). PRs #16, #10. |
| 22 | AC-87 receipt lists and take-it-home detail | ✅ Pass | `test_receipt.py` over 9 fixtures in `bank/fixtures/receipts/`: four lines for complete and legacy ran, three for does-not-compile, none for `no-receipt`. `twins.rs::the_receipt_matches_every_shared_fixture`. `take_home.rs` legacy/complete/does-not-compile tests. `home.test.js:165-197`: complete shows `-Vv`, edition, triple, flags, Miri version/configs/seeds; legacy shows target *not recorded* and Miri *run separately*. PRs #16, #34. Note: *not recorded* / *run separately* are marked PROPOSED-§11 in `copy.js:213-232` (Drift). |
| 23 | AC-14 exact duplicate | ✅ Pass | `test_a_byte_identical_resubmission_of_any_bank_question_is_rejected` (all four give `exact_duplicate`); `test_an_exact_resubmission_is_rejected_by_a_run_and_written_nowhere`. PR #14. |
| 24 | AC-15 normalized duplicate | ✅ Pass | `test_renamed_bindings_and_reformatting_are_normalized_duplicates[×3]`; `test_a_renamed_bank_question_is_rejected`. Open bug PQ-30 (F-20) is the converse: distinct programs wrongly rejected. It is reproduced on this tree (C1, C6 give `normalized_duplicate`) but does not break this pass condition. PR #14. |
| 25 | AC-16 near-duplicate to review | ✅ Pass | `test_a_near_duplicate_lands_in_the_review_queue_neither_accepted_nor_dropped` (`near_duplicate_of=="q3"`, status None). Because of PQ-30, some distinct programs never reach this queue. PR #14. |
| 26 | AC-17 history persists | ⚠️ Partial | `test_the_history_persists_and_grows_across_two_runs_in_separate_working_directories` gives 5 → 6, growth 1, in the run report. *On the review surface* is unproven: T-18 was never built. PRs #14, #10. |
| 27 | AC-18 uniqueness wording | ✅ Pass | `test_every_uniqueness_statement_in_the_corpus_is_the_exact_sentence`; an independent `rg` for *original* over pipeline/, bank/, web/ and room/src found no uniqueness claim. PR #14. |
| 28 | AC-19 review round-trips | ⛔ Blocked | No PR; PQ-23 (T-18) never dispatched (F-15 ruling). `review.py` is a scaffold. |
| 31 | AC-22 reviewer-has-seen statement | ⛔ Blocked | No PR; PQ-23 never dispatched (F-15). |
| 32 | AC-88 difficulty drift fails the run | ⚠️ Partial | T-19 half: `audit.py:1129-1161` (`DRIFT_LIMIT=1.0`, fails the **run**, names no question). Rerun by name: `test_drift_above_one_level_fails_the_run_and_names_no_question`, `test_drift_of_exactly_one_level_passes` and `test_drift_ignores_unjudged_and_unaccepted_questions` PASSED. The check reaches `exit_status` via `report["failed"]` (`audit.py:1389`, `:1473`). Unproven: *`difficulty_judged` recorded*. Nothing writes it (T-18 never built), and `bank-audit` reports `n/a`. PR #13. |
| 34 | AC-23 slot is pure | ✅ Pass | `slot.py:55` `slot_for_day(day: date, n_options: int = 5)`, importing only hashlib and datetime. bank-audit `slot path and ledger … ok`; `test_the_real_slot_module_is_clean`; `test_the_slot_lint_catches_what_it_claims_to`. PR #13. (The function is pure. Nothing on the tree calls it to arrange options; see Drift.) |
| 35 | AC-23a attendee simulation | ✅ Pass | bank-audit over 10,000 nights: most-used 19.9%, least-used 20.1%, not-last-night 20.0% (band 18–22%). `test_three_attendees_with_perfect_memory_stay_at_chance`. PR #13. |
| 36 | AC-23b generator both tails | ✅ Pass | χ²=3.46, df=4, both tails, 20,000 draws (A 4041, B 4000, C 4059, D 3993, E 3907). `test_a_skewed_generator_fails_the_upper_tail`; `test_a_rebalanced_generator_fails_the_lower_tail` (lower bound the 1% point, 0.297). PR #13. |
| 37 | AC-24 five options, one does-not-compile | ✅ Pass | bank-audit: `all 4 question(s)`; `test_five_options_fails_on_a_malformed_record[…]`. PR #13. |
| 38 | AC-25 no published distribution | ✅ Pass | bank-audit: 10 participant-facing files clean; `test_a_published_distribution_is_caught`. The lint list (`audit.py:636-647`) omits `room/src/copy.rs` and the organizer READMEs. A manual `distribution_hits` over 32 more files found nothing. PR #13. |
| 39 | AC-26 enumerated tells | ❌ Fail (as written) | All five tells are measured (unsafe and topic come back n/a, no rule fired). **The option-position tell is 5.00× chance** (`index E`, 4 hits of 4 vs 0.80 expected, tail p=0.0016) **and reports WARN, not fail.** SPEC §7.6 and the row say *failing above 1.5× chance*. The code requires ratio > 1.5 **and** tail ≤ 0.01/m (`audit.py:779-786`, `:1053-1059`). With m=10 rules in the family, α=0.001 > 0.2⁴, so **at a bank of four this tell cannot fail.** It is documented as deliberate (`bank/README.md:359,371-375`; `test_the_answer_written_last_warns_at_four_and_fails_at_five`), but no F-n finding or amendment records it. Stored order is the visible order on this tree. PR #13. |
| 40 | AC-27 unsafe parity | ✅ Pass | `test_a_ub_answer_with_unsafe_nowhere_else_fails`; `test_a_non_ub_question_containing_unsafe_restores_parity`. Trivial on the real bank (no UB answer). PR #13. |
| 41 | AC-71 take-it-home sentence, wall claims none | ⚠️ Partial | Take-it-home: `ok 144 - AC-71: How we know ends with…` asserts both sentences (`copy.js:210`). Wall: `ok 241` asserts no explanation *prose*, but **no test asserts the receipt carries no sentence about the explanation**. The data agrees (q3's receipt is four step lines; no *explan* in `receipt.py` or the fixtures), but the clause is untested. PRs #34, #21. |
| 42 | AC-72 affirmation gate | ⛔ Blocked | No PR; PQ-23 (affirm) and PQ-25 (schedule) never dispatched (F-15; after T-18). No affirm or schedule code. **G-12 has no enforcement on this tree.** |
| 43 | AC-73 quoted output checked | ⛔ Blocked | No PR; PQ-23 never dispatched. |
| 44 | AC-74 provenance markup | ✅ Pass | Wall `ok 248` (answer and receipt under `.by-machine`, no human marker); host `ok 161/162` (beats under `by-human` + ✎, no machine marker); home `ok 139` (beats `data-provenance="human"`, receipt and options `machine`). Markers differ in border, glyph and colour (`components.css:135-140`). PRs #21, #22, #34. |
| 46 | AC-75 reserve count and trend | ⛔ Blocked | No PR; PQ-25 (T-20) never dispatched (after T-18). |
| 47 | AC-76 low-reserve warning | ⛔ Blocked | No PR; PQ-25 never dispatched. |
| 49 | AC-102 static fallback file | ⚠️ Partial | Built with `python -m popquiz.fallback q3`; bake equals `web/wall/fixtures/q3-static.json`. In the c11 browser: Space walks idle → … → released; `→` stops at M-2 in work; reveal enters at 6/6 (M-1); `←` and Esc behave. work shows 0 ✓, 0 receipt, 0 colour spans, no stdout. Static vs room render parity 30/30 frames in WebKit. **Network was never actually blocked**: WKWebView has no offline emulation or request interception. Established instead: CSP `default-src 'none'; connect-src 'none'`; the server log shows only the document GET; `performance` resource entries 0; no URLs in a static scan. Key stepping ran on a copy whose only change adds `'unsafe-eval'` (the real CSP refuses c11's injected script). PR #24. |
| 50 | AC-102 host sheet | ⚠️ Partial | T-26 half present: `fallback.py:248` `host_sheet()` gives plain text, phase order, every beat and note verbatim, no other prose. `fallback.py:291-303` writes HTML and `<stem>.host-sheet.txt` from one call, both or neither. `test_fallback.py:217,237,256,273,281`; `no_fallback_route.rs` (never served). Unproven: that the schedule step calls it (T-20, never built). PR #24. |
| 52 | AC-28 join by link or code | ✅ Pass | `ac28_join_by_code_in_any_case_and_nothing_else`, `ac28_the_short_link_and_the_typed_code_both_join`, `ac28_the_short_link_carries_the_code_to_the_page`; node test 32 (one input, one button), 33–34. PRs #18, #23, #26. |
| 53 | AC-29 six failure states | ✅ Pass | Byte-compared with SPEC.md:622: `room/src/copy.rs:30-36` and `web/shared/copy.js:72-78` match. #42 added only row :621 (`join_transport_failed`, also matching). `ac29_each_failure_has_its_own_sentence` (6 distinct messages, 6 slugs); node 36. The server never emits `NotYetOpen` (Drift). PRs #18, #23, #30, #42. |
| 54 | AC-30 capacity before session | ✅ Pass | `ac30_the_201st_join_is_refused_and_reserves_nothing` (409 `full`, no token, `session_count`==200, host view unchanged). PR #18. |
| 56 | AC-32 no source on a phone | ✅ Pass | `just canary` 12/12. The participant rule (`canary_scan/mod.rs:715-746`) forbids source, trace notes, option text and those keys on every buzzer surface, page and traffic, in every phase. PRs #25, #31. |
| 57 | AC-33 wall fits, phone no h-scroll | ⚠️ Partial | Take-it-home: `/web/home/measure.html?run=1` reports PASS over 28 renders. Widest bank question q4 (5 × 78) at 375 px: `scrollWidth 375 = clientWidth 375`, `pageScrolls:false`, code wells scroll internally. 375 px came from frames, since viewport emulation is unsupported. **Review half Blocked**: T-18 never built. PR #34. |
| 59 | AC-34 last answer wins | ✅ Pass | `ac34_five_changes_then_close_then_refused_with_the_saved_answer` (A, C, B, E, D gives D; after close 409 with D restated); node "taps in flight — last wins". PRs #18, #23. |
| 60 | AC-35 exactly one of saving/saved/failed | ✅ Pass | Node 49 (exactly one state after each of >400 events across every response class); `ac35_response_contract`. PRs #23, #18. |
| 61 | AC-36 failed write is safe | ✅ Pass | Node 50 (last answer safe, retry re-sends); `ac36_a_failed_write_leaves_the_previous_answer_intact` (6 failure kinds). PRs #23, #18. |
| 64 | AC-39 reveal surfaces | ⚠️ Partial | Wall (node 241; canary :834-841) and host three beats (host.test 168) hold. **The buzzer shows all five bars from split on** (node 259), more than AC-39's letter and count. SPEC §4 was amended by PQ-37 (#31); AC-39 was not. The client rules. PRs #21, #22, #23, #31. |
| 65 | AC-40 ✓ as well as colour | ⚠️ Partial | Wall `ok 246/247`, buzzer `ok 41/47`, home `ok 138` assert glyph and class, with no ✗. Seen in the browser on the fallback reveal and the buzzer reveal. **Review half Blocked** (T-18). PRs #9, #21, #23, #34. |
| 67 | AC-42 trope check | ✅ Pass | `copylint.test.js` copy-module trope test asserts zero hits across `COPY`; `retired.json` (7 strings): every group fires, every string caught; `test_copylint.py::test_check_prose_*` and `test_check_prose_never_raises` (prose gives warnings only). PR #38. |
| 69 | AC-43 receipt exact lines | ✅ Pass | `test_each_fixture_renders_the_lines_the_spec_gives_it` and `test_no_rendered_line_overstates` (9 fixtures); `test_undefined_behavior_as_the_answer_reads_as_a_flag_not_a_pass`; `test_a_panic_answer_renders_the_full_list`; q8 `E0502` three lines, tied to the bank. JS and Rust twins read the same fixtures; `copy.test.js:138` and `twins.rs:32` check punctuation and the five claim words. UB and panic fixtures are synthetic (labelled). PRs #21, #16. |
| 72 | AC-45 host actions, exactly these | ✅ Pass | `every_state_and_command_does_exactly_what_the_table_says`; `the_host_actions_are_exactly_the_eight_of_ac_45`; `every_host_route_exists_and_nothing_moves_without_the_host`; node 154–155. PRs #15, #22. |
| 73 | AC-46 live counts | ✅ Pass | `ac46_counts_move_on_the_host_phone_while_live`; `ac46_the_host_socket_counts_move_while_live`; node 157. PRs #18, #20, #22. |
| 74 | AC-47 host has no answer pre-reveal | ✅ Pass | Canary pre-reveal rule covers the Host surface (`mod.rs:757-831`: no ✓, `correct`, `receipt`, `read_aloud`, `why_tempting`); `ac47_no_page_or_payload_the_host_sees_carries_the_answer_before_reveal`. PRs #25, #22. |
| 75 | AC-48 hint is free and private | ✅ Pass | Hint only in the live Buzzer view (`mod.rs:750-755`); taking it sends no request and leaves the wall and host projections unchanged (`mod.rs:1418-1427`); no hint field in `sessions.rs`. PRs #23, #25. |
| 76 | AC-49 host screen shape | ✅ Pass | `ac45_ac49_each_phase_offers_one_action…`; `the_host_screen_names_its_phase_and_one_action`; node 152–153. PR #22. |
| 83 | AC-56 nothing survives release | ✅ Pass | No DB schema; state is in memory. `ac56_nothing_per_person_survives_release`; `ac56_the_totals_expire_with_the_released_room`; `ac56_the_survivors_carry_no_count`. PRs #30, #34. |
| 84 | AC-57 no identity anywhere | ✅ Pass | Static. `Session` is exactly `{token, answer}` (`sessions.rs:49-51`, exhaustive pattern); `UsedRecord` has 4 fields (`used.rs:56`); buzzer `sessionStorage` is room-scoped (`buzzer.js:729`); `ac57_*` ×3. PRs #30, #23, #18. |
| 85 | AC-58 recap computed on the phone | ✅ Pass | `assert_nothing_personal_after_close` in the canary walk; node 54. PRs #25, #23. |
| 87 | AC-60 canary on every pre-reveal path | ✅ Pass | Every projection, frame, page, rendered HTML and page traffic scanned at every revision; positive controls `assert_the_plants_arrived`, `the_rules_catch_a_planted_leak`. PR #25. |
| 88 | AC-61 sealed answers module | ✅ Pass | Static. Secrets are private fields of `answers::vault::Vault`, a private nested module (`answers.rs:456-545`). Its one read, `open`, needs a `RevealWitness` that only `Machine::revealed()` builds under `Phase::Reveal`. Pre-reveal builders (`view.rs mod sealed`) take only `PublicView`. 6 `compile_fail` doctests; `boundary.rs` 6/6. PRs #15, #19. |
| 89 | AC-62 option text is public | ⛔ Blocked | No PR; PQ-29 (T-24) never dispatched (after T-20). The sentence is on no runbook and no host screen. It survives only in `SPEC.md:540` and in `web/test/copy.test.js:117`'s `REMOVED` map (Drift). |
| 90 | AC-63 host can infer, not a guarantee | ⛔ Blocked | No PR; PQ-29 never dispatched. Absent from `web/host/host.js`; only in `SPEC.md:541-543` and `copy.test.js:118` `REMOVED` (Drift). |
| 92 | AC-99 colour scoped | ✅ Pass | Node 245 (spans > 0 in live/closed/split, 0 in work/reveal, no source idle/released), 228 (static); payload `colour` at `view.rs:459/477/582`; `the_web_fixtures_are_what_the_room_serves`. PRs #9, #21, #24. |
| 93 | AC-79 wall is inert and clean | ✅ Pass | `assert_no_control` on every rendered wall (`mod.rs:903`); node 250; pre-reveal plant rules on Wall and RenderedWall. PRs #21, #25. |
| 94 | AC-80 no dark mode | ✅ Pass | Static. No `prefers-color-scheme` or `color-scheme` in `web/`; nothing sets `data-room`. Three dormant `[data-room="dim"]` rules ship (`components.css:116,141,142`). No test in `just test` asserts the absence. PR #9. |
| 96 | AC-100 fit at the floor | ✅ Pass | `test_each_fit_fixture_raises_exactly_its_own_flag[passing/too-long-option/too-long-program]`; `test_the_fixture_set_is_one_of_each`; a 30-char option is flagged; `test_the_constants_match_typemodel_js`. The fixtures run under `just test`; the `bank-audit` CLI has no fixture mode (`audit.py:1485-1491`). bank-audit on the real bank: every question fits. PRs #13, #9. |
| 98 | AC-82 keyboard | ⚠️ Partial | Reachable: every control is native with tabIndex ≥ 0 (buzzer, host, home; the wall has one `role=region` and 0 controls). **Visible focus not observed in the browser**: c11 `press Tab` moves no focus, and programmatic `focus()` reports `:focus-visible` false. The hermetic `a11y.log` has a *ring ok* cell per screen. PRs #37, #41. |
| 99 | AC-83 live regions | ✅ Pass | A MutationObserver on the polite region in the c11 browser read every buzzer string verbatim against SPEC.md:627 (question on screen, Saving., Saved, ‹X›., Couldn't save…, Hint shown…, closed, split, walking, Revealed: it was ‹Y›., released) and the host's 7 labels in order. PRs #37, #41, #23. |
| 100 | AC-84 AA contrast | ❌ Fail (as written; F-46, PQ-43 held) | `a11y.log`: dimmed trace line numbers `#808895` on `#fff` = **3.57:1** at 13 px (needs 4.5) on wall and home work and reveal. `.buzz`, `.btn` and host step-button edges are **1.41:1**, printed as supplementary. Everything else is ≥ 4.67:1. AC-84 also names a dim-room presentation, which does not exist (`web/README.md:98`). Remedy and ruling on PQ-43 (`--text-primary` at `wall.css:112`, `home.css:92` gives 4.78:1). PRs #37, #41. |
| 101 | AC-85 44 px targets | ✅ Pass | Rendered boxes: buzzer letters 351×97–104, join 351×44, hint 351×44; host primary 448×44, step buttons 44×44; home steps 44×44. PRs #37, #41, #23. |
| 102 | AC-86 reduced motion | ⚠️ Partial | `getAnimations().length` = 0 on wall, buzzer, host and home, and every state is legible from static text. But `matchMedia('(prefers-reduced-motion: reduce)')` was false throughout: c11 cannot emulate it and the OS setting was not changed. *Under the preference* rests on the static `a11y` tests `ok 27-31`. PRs #37, #41. |
| 103 | AC-89 one question, no next | ✅ Pass | A room's `question: Arc<Scheduled>` is set only at construction (`rooms.rs:237/328`); the table allows only `released → new room`; `g10_run_it_again_refuses_the_used_question`. No test is named for AC-89. PRs #15, #30. |
| 107 | AC-92 `used` written at release only | ⚠️ Partial (one clause fails) | Room clauses pass: `ac92_used_is_written_at_release`, `ac92_nothing_is_written_before_release`, `ac92_scheduling_questions_writes_nothing`, `ac92_an_expired_or_quiet_room_records_nothing`, `g10_the_release_transition_is_the_only_writer`, `ac92_the_pipeline_makes_a_used_record_only_when_reading_the_bank`. **Fails: *`build_deck.py`'s ledger write is gone*.** `mvp/tools/build_deck.py:220-234` (`slot_for_meetup`) still writes `answer-history.json` (line 233) on every build. T-19 declined to port it but did not retire it (G-10, BUILDPLAN T-19). PRs #30, #13. |
| 108 | AC-93 reveal only via the walk | ✅ Pass | `no_transition_skips_a_phase_and_reveal_needs_the_walk_through` (tried at every trace position). PR #15. |
| 109 | AC-94 no ✗ | ✅ Pass | Node 48 (every phase × every saved letter: no ✗, *wrong* or *incorrect*, no red class); wall 247; home 138. PRs #23, #34. |
| 110 | AC-95 most-chosen incorrect, same everywhere | ⚠️ Partial | Proven: wall and host name the same option and count, including a popular-correct and a does-not-compile-wins fixture (`the_most_chosen_incorrect_option_is_named_the_same_on_wall_and_host`); nobody-else variant (`nobody_read_it_another_way`, node 243); take-home has no count. **Unproven: *affirm refuses a missing `why_tempting`*** (T-18 never built). Only the room loader refuses (`answers.rs:619-629`, `twins.rs:308`). PRs #15, #18, #21, #22, #34. |
| 113 | AC-97 the walk-through phase | ✅ Pass | Phase table plus `walk_bounds` (work 0..M-2; reveal enters M-1); canary in `work` finds no ✓, receipt, `stdout` row, step > M-2 or colour (`mod.rs:772-830`); `work_shows_steps_up_to_m_minus_2…`; node 227/229. PRs #15, #21, #25. |
| 114 | AC-98 forbidden-copy lint | ⚠️ Partial | Patterns byte-equal to SPEC §11's Forbidden row (`copylint.test.js:53` reads SPEC.md; 9 patterns; case-insensitive `:66`); zero hits over every `COPY` entry, including `join_transport_failed` (#42, pinned to `copy.rs:153` by `twins.rs:366`); `copy-freeze.test.js` "nothing waits for a contribution" set passes. **Not every participant-facing string is linted:** `trace.steps[].note` (wall, host, take-it-home) is in neither lint's homes (F-47, PQ-42 held). Host status `reason` strings are allowlisted (`copy-freeze.test.js:311`). A manual run over the four bank questions' notes found 0 hits today. PRs #38, #42. |
| 116 | AC-64 role check, stand-in, live | ✅ Pass | `ac64_a_member_with_the_role_creates_a_room` / `…without_the_role_is_denied`; `room/Cargo.toml` has no `dev-host-token` feature; `ac64_no_stand_in_survives_in_src_cargo_dockerfile_or_ci`; `ac64_any_bearer_that_is_not_an_organizer_session_is_401`. *Stand-in build refuses without it* is moot: the stand-in was deleted (#32). PRs #26, #32. |
| 118 | AC-65 roles not permissions | ✅ Pass | `ac65_an_administrator_and_owner_without_the_role_id_is_denied`; `ac65_the_member_record_is_read_for_roles_alone`; `ac65_the_role_id_is_compared_whole`. PR #32. |
| 119 | AC-66 refresh rotation persisted | ✅ Pass | `ac66_each_refresh_persists_the_rotated_token_and_presents_it_next`; `ac66_a_refused_refresh_signs_the_organizer_out_and_leaves_their_open_room`; `ac66_two_creates_at_once_refresh_once`. PR #32. |
| 120 | AC-67 participants never authenticate | ✅ Pass | Static. In `routes.rs`, `/join`, `PUT answer` and `show(Buzzer)` (`routes.rs:124`) plus the buzzer ws attach (`ws.rs:333-336`) resolve only the session map; `ac67_*` ×2. PRs #17, #23, #32. |
| 121 | AC-68 only the creator controls | ✅ Pass | `ac68_organizer_b_cannot_read_or_control_as_room` (401 on GET and every action); `…run_it_again…`. PR #32. |
| 123 | AC-70 denial wording | ✅ Pass | `ac70_a_non_member_and_another_guilds_member_get_the_same_wrong_server` (byte-identical); `ac70_a_member_without_the_role_hears_wrong_role`. PR #32. |
| 124 | AC-101 admin channel | ✅ Pass | `admin.rs` 15/15, including `ac101_the_admin_prefix_is_served_to_the_check_alone`, `…no_participant_wall_host_or_auth_route_accepts_the_admin_token`, `…missing_or_wrong_token_is_refused_saying_nothing`; `just secret-scan` EXIT 0. PR #33. |
| 125 | AC-101 canary plant | ✅ Pass | Canary admin rule on every surface and phase (`mod.rs:681`); `the_admin_rule_catches_a_planted_token`; `the_binary_logs_no_admin_token`. PRs #33, #25. |

## Drift from BUILDPLAN.md

- **The answer is at E for every question, and nothing arranges options by date (G-1 at runtime; no row covers it).** q3, q4, q7 and q8 all store the correct option at index 4 (q8's is `does_not_compile` at E). `slot_for_day` is pure and passes every audit, but nothing on the tree calls it to arrange a question. `room/src/question.rs:16-17`: *"The room never arranges options… the date-drawn arrangement is the pipeline's (AC-23, T-20)"*. `pipeline/src/popquiz/bank.py:28` makes arranging the caller's job. The static fallback (`fallback.py:27-28`) renders stored order. T-20 was never built. So a room or fallback built from this tree today puts the answer at E, the AC-23 failure the house rules guard hardest. bank-audit's option-position WARN (row 39) is this same fact, and it cannot fail at n=4. **Most important item in this report.** It is not a defect of any merged PR, since each says arranging is T-20's job. It is a hazard of the interim state if anyone runs a real meetup from this tree before T-20 exists.
- **External #35 (`1263add`, the commit Fly v11 runs).** Take-it-home's empty state changed from *No question has been released yet.* to **Welcome to the POP QUIZ** (`web/home/home.js:48`). It sits in a `PROPOSED` table, has no §11 row, and SPEC §11 says copy is *authored here and only here*. It is the client's own line, so the client rules; it should either gain a §11 row or go.
- **External #36.** The client hand-typed `verified` blocks for q4, q7 and q8. On the tree they are fully replaced by machine writes: digest, pin, byte-identical round-trip and recording replay all match (row 11). No residue.
- **External #42 edited SPEC §11 directly.** It added the *Buzzer, join connection failure* row. The string is byte-identical in `room/src/copy.rs:35`, `web/shared/copy.js:77` and SPEC.md:621; it is linted and twin-pinned. Consistent; the contract was changed outside tone-architect, which is the owner's prerogative.
- **External #43.** WOFF2 is primary with the vendored TTF as fallback; the fallback build strips the TTF fallback and inlines WOFF2 (`fallback.py:172-182`); `fonts.test.js` added. Bears on F-16 and AC-77; no criterion regressed.
- **External #44.** Node 24 actions in CI. No criterion impact.
- **External #7** (D-16…D-25 contract amendments) is the contract this audit read.
- **AC-59 cut from the buzzer (F-41).** The *nothing about you was recorded* line is not on the buzzer. It is a deliberate non-meet of a criterion the client owns (row 86 is smoke-side).
- **The §8.1 sentences were removed from the host's first screen.** SPEC §6 says the host's first screen carries §8.1's two sentences. PQ-34 (#27) and PQ-36 (#29) removed the keys, and `copy.test.js:117-118` asserts they are gone. With T-24 unbuilt, AC-62 and AC-63 have no surface anywhere (rows 89–90).
- **The buzzer shows the five bars from split on** (SPEC §4 amended by PQ-37 #31; AC-39 says letter and count). Row 64.
- **Run it again on an unused question** makes a second room on the same night (`g10_run_it_again_on_an_unused_question_makes_a_new_room`). That is in tension with AC-89 and AC-92's *one question per meetup*.
- **The used ledger and organizer store are in memory only** (`used.rs:59-65`, `discord.rs:47-50`). A Fly restart erases AC-92's record until T-20's `sync` pulls it. Combined with `justfile:249-251` (*restart before the next run*), the ledger is lost by the documented procedure if `sync` is skipped.
- **F-15** (a does-not-compile question can never be affirmed as the contract is written): unresolved. It is the reason T-18, and with it T-20 and T-24, was not built.
- **F-18** (`why_tempting` on each option) and **F-19** (correct index by option kind): implemented as ruled (`bank.py`, `answers.rs:224`, `twins.rs`).
- **D-15:** the bank holds q3 as authored plus re-authored q4, q7 and q8; every option ≤ 29 chars and every question fits (bank-audit). `bank/README.md:371` still says q4, q7 and q8 are flagged for option length (stale, PQ-39 review Minor 1). `web/home/measure.html:24` and `room/README.md:240` still name q7 as widest; q4 (5 × 78) is (F-40).
- **F-46 / F-47** stand as recorded (rows 100, 114).
- **Smaller items.** `JoinRefusal::NotYetOpen` is never emitted (joins are accepted in `idle`). The fallback file embeds the live wall's `fetch("/rooms/…/fit", PUT)` path, dead in static mode and blocked by CSP. The fallback live region keeps *The question is on the screen.* after Esc back to idle. `[data-room="dim"]` rules ship unused. The ✎ marker is `aria-hidden` on home but not on host. Host denial strings (`copy.rs:80-81`) are not in §11 (F-33 run ruling). Leftover `HOST_DEV_TOKEN` comments and empty values sit in `.env.example:39-40`, `fly.toml`, `justfile` (not code). CI runs `just test` but never `just bank-audit`, so the audit's checks reach CI through pytest only.

## Gaps

- **AC-3, AC-4, AC-5** (rows 4, 5, 7) and AC-1's static check (row 1): T-16 never built (H-4).
- **AC-19, AC-22, AC-72, AC-73** (rows 28, 31, 42, 43), and the review halves of AC-17, AC-33, AC-40, AC-88 and AC-95 (rows 26, 57, 65, 32, 110): T-18 never built (F-15). **G-12 is unenforced.**
- **AC-75, AC-76** (rows 46, 47), T-20's halves of AC-102 (row 50), and the date-drawn arrangement of options that makes AC-23 real at runtime: T-20 never built.
- **AC-62, AC-63** (rows 89, 90): T-24 never built, and the host-screen copy was removed.
- **G-1…G-12 adversarial audit:** T-23 never built. This report is not a substitute for it.
- **AC-26** (row 39): built, but the option-position tell cannot fail at the current bank size.
- **AC-92** (row 107): `build_deck.py`'s ledger write was never retired.

## Recommendations

**Fix-back-in-flight** (small, owned by finished tickets, routable now):
- Retire `slot_for_meetup`'s write in `mvp/tools/build_deck.py:220-234` (T-19 / PQ-24's audit demand, G-10). One function; the house rules already name it as a defect.
- PQ-43 (F-46): `--text-primary` at `wall.css:112` and `home.css:92` clears 3.57:1 → 4.78:1 without an amendment. It still needs the owner's word on the 1.41:1 edges.
- PQ-42 (F-47): bring `trace.steps[].note` into the lint's homes.
- Add the one missing assertion for AC-71's wall clause (row 41): the receipt holds no sentence about the explanation.
- Stale docs: `bank/README.md:371`, `web/home/measure.html:24`, `room/README.md:240`.

**New tickets:**
- **Before any real meetup from this tree: the date-drawn option arrangement** (T-20's push half, or a stop-gap that applies `slot_for_day` wherever a question is pushed or baked). Until then a room run from this bank shows the answer at E.
- A static dependency check for AC-1 in `just test` (`room/` reaches no pipeline/LLM crate). Today it holds by convention.
- Decide AC-26's small-bank rule in the contract: either amend SPEC §7.6 to the multiple-comparison form the code uses (and say what happens at n < 5), or make the code fail as written.
- `just bank-audit` in CI.
- The T-18 → T-20 → T-24 chain, gated on F-15's amendment; T-16 on H-4; T-23 last.

**Accept-as-is** (owner's call; recorded so the strict reading is visible):
- AC-59 cut (F-41); the buzzer's five bars (row 64); the removed §8.1 host sentences (rows 89–90) if the runbook is to be their only home; `Welcome to the POP QUIZ` (#35) once it has a §11 row; AC-98's *no count displayed* literal vs AC-46's Joined/Answered counts (the contract wants both; AC-98 needs rewording); *Run it again* on a fresh question in one night.

## What I couldn't verify

- **The fallback with the network blocked (row 49).** WKWebView in the c11 browser cannot emulate offline or intercept requests. Zero requests is established by CSP, the server log, `performance` entries and a static scan, not by a blocked network. Stepping ran on a copy with `'unsafe-eval'` added to `script-src`.
- **Visible focus under a real Tab key (row 98), the OS reduced-motion preference (row 102) and VoiceOver.** Not drivable from c11. They need a human (web/README.md, step 5).
- **That the recordings and the q4/q7/q8 records came from the Docker image** rather than typing. `sandbox-build` and `test-verify-full` are out of scope here (smoke rows 10, 13, 15, 17, 19).
- **The real-time reaper for the 4 h and inactivity expiries.** Proven on `ManualClock` only.
- **Whether the option-position WARN-not-FAIL rule was approved by the client.** Documented in `bank/README.md`; not found in run-state's F-n list.
- **Contract feedback.**
  - The plan's footer miscounts its own tags (8/74/44 stated, 7/76/43 actual), and the boot prompt inherited 82/44.
  - Rows 57 and 65 mix a built and an unbuilt surface in one pass condition, which forces Partial.
  - Row 107's pass condition puts the room's ledger and the MVP tool's ledger in one row.
  - Row 96's method says `just bank-audit` fixture set, but the fixtures run under `just test` (the CLI has no fixture mode).
  - Rows 98–102 say "`just a11y` via the c11 browser", but `just a11y` is a node suite, so the browser half had to be improvised.
- The canary, transport and harness `--full` halves are ignored in `just test` by design and are the client's (test-full).

## Operator smoke-pass checklist (post-merge)

Every `post-merge-smoke` row in validation-plan.md (43 rows; the plan's footer says 44), copied through verbatim for the client to walk on the assembled tree. The Result Validator did not attempt these.

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
