# PQ-64: Persisted fields with no writer or no production reader

From the guardrail audit (PQ-28, PR #49, docs/audit/2026-10-guardrail-audit.md), audit tree 0c7d60a.

**GAP-27** (Cross-cut., Minor, new): Persisted fields with no writer or no production reader
- Evidence: see the one-writer table
- What would close it: Delete or wire each, per SPEC 3

Held: an audit's findings are the client's to release. Read the gap's section in the audit report before planning; re-verify on the current main.

# Plan (delegator, 2026-11-08)

Traced on worktree HEAD `ae4d8a0` (= origin/main), then fast-forwarded to `00c7753` after PQ-87 merged; PQ-87 touched only `web/host/host.js`, `web/test/a11y.test.js` and `web/test/host-pick.test.js`, none of them in this trace, so every file:line below holds on `00c7753`. Ruling: verify-and-close. Every row is closed: each field has a writer and a production reader, and an existing test fails when that reader stops reading it (bounded mutations below, each run against only the named test, then restored with `git checkout`; the same tests pass unmutated). **No guard test is needed and no code changes: no PR; close report.**

## Per-row trace

| Audit row | Field | Writer | Production reader (user path) | Covering test | Verdict |
|---|---|---|---|---|---|
| 551 | `review.affirmed_by`, `affirmed_at` | `review.affirm` review.py:289-306 (`just review affirm`) | `schedule.refusal` schedule.py:327 refuses an unaffirmed record (`just schedule`); also `Review.affirmed` bank.py:216 (reserve), dedupe.py:2922, ledger report schedule.py:763 | test_schedule.py:256, :263 (M3) | closed |
| 552 | `verified.verified_at`, `verifier_version` | verify.py:328-329 (`just verify`) | `verify.check_question` verify.py:685, called by `schedule.record_refusal` schedule.py:378 (`just schedule`), `fallback.bake` fallback.py:138, and via `check_provenance` verify.py:671 from `_check_file` verify.py:770 (`verify --check`, `just demo` justfile:367) | test_schedule.py:361 `[M2a-unstamped]` (M4); test_schedule.py:398 pins `just demo` running `--check` before any network call | closed |
| 553 | `verified.miri.seeds` | verify.py:442, :490 (`MIRI_SEEDS = (0,)`, :93) | every hop carries it: bank load bank.py:874 → dump `_dump` bank.py:752 → PUT body `schedule.push` schedule.py:564-568 (`just schedule --room`), or `just demo` justfile:391-392 (stored file verbatim plus `meetup_date`) → room `Miri` answers.rs:216 (`deny_unknown_fields`) → `HowWeKnow` answers.rs:557 → `TakeHome.machine.miri.seeds` used.rs:280-283 (as strings, AC-56) → `/last` pages.rs:63 (`script_json`) → home.js:266 parse → home.js:180 render (take-home page) | test_verify.py:405 (M2, load/dump); test_schedule.py:769 (M1, PUT body); take_home.rs:240 (M8, M9, room); home.test.js:178 "AC-87, a complete record" (M7, page) | closed, no hop drops it |
| 554 | `review.reason` | `review.reject` review.py:272-277 (`just review reject`) | `rejection_reasons` review.py:233-236 → `popquiz review reasons` review.py:569,577 (AC-19's generator signal); `report` review.py:219 → `review show` → organizer TUI review.rs | test_review.py:111-116 (M5) | closed |
| 555 | `explains.legacy` | migrate_mvp.py:499 (T-14, one-time; review.py:381 refuses rewriting it) | `bank.quoted_outputs` bank.py:567-568, called by `review.quotes` review.py:106 (`accept_blockers`, AC-73, `just review accept/show`) and generate.py:1046 (quoted spans for review) | test_migration.py:826 (M6) | closed |

## Bounded mutations (break one reader, run only the covering test, restore)

| # | Mutation | Test | Result |
|---|---|---|---|
| M1 | `push` pops `verified.miri.seeds` from the body | test_schedule.py::test_ac101_the_push_carries_the_bearer_and_exactly_the_arranged_record | fails (bites) |
| M2 | bank loader sets `seeds=None` | test_verify.py::test_the_record_round_trips_the_bank_format[q3-ran] | fails (bites) |
| M3 | `refusal` ignores missing `affirmed_*` | test_schedule.py::test_ac72_affirmed_by_without_affirmed_at_is_refused | fails (bites) |
| M4 | `check_question` ignores missing `verified_at`/`verifier_version` | test_schedule.py::test_ac7_a_record_the_verifier_did_not_write_is_refused[room-M2a-unstamped] | fails (bites) |
| M5 | `rejection_reasons` returns "" for the reason | test_review.py::test_ac19_reject_needs_a_reason_and_the_generator_can_read_it_back | fails (bites) |
| M6 | `quoted_outputs` skips `explains.legacy` | test_migration.py::test_the_legacy_string_is_searched_too | fails (bites) |
| M7 | home.js stops rendering seeds | web/test/home.test.js "AC-87, a complete record" (pass 17, fail 1) | fails (bites) |
| M8 | room `HowWeKnow` sets `seeds: None` (answers.rs:557) | popquiz/tests/take_home.rs::a_complete_record_carries_the_machine_verbatim | fails (bites): panics at the seeds assertion take_home.rs:257 |
| M9 | `take_home` sets `seeds: None` (used.rs:280) | same | fails (bites): panics at take_home.rs:257 |

Unmutated baseline: the 14 pipeline tests above pass; home.test.js pass 18 fail 0; take_home.rs passes (1 passed). The popquiz tree was clean after each restore.

## SPEC §3 one-writer table, fields beyond the audit's list

None without a writer or a production reader. Checked: `used.{meetup_date, room_id, released_at, fit}` (release writes; scheduling, `popquiz sync` and the take-home read), the push's `meetup_date` (PQ-80; room requires it, SPEC §8.5), the recording's `_source_sha256`/`_record_sha256`/`_recorded` (verify writes; `schedule.record_refusal` reads), `review.difficulty_judged`, `near_duplicate_of`, `topic`/`difficulty_requested` (review, audit, dedupe, TUI read). Contract note, not fixed here (SPEC is never-edit): `review.affirmed_digest` and `review.quotes_checked` are persisted (review.py writes; room affirmation.rs and review.py:103 read) but absent from SPEC §3.1's `review` row.

## Guard tests

None. Every row already has a test that bites (table above).

## Question

None.

## Validation (2026-11-08, HEAD 00c7753)

`just test`, sandbox off for loopback binds: green twice. Cold (after the fast-forward) 247.6 s; warm 53.8 s. Counts, the same on both runs: cargo 406 passed, 0 failed, 4 ignored across 39 test binaries; web 330 pass, 0 fail; pipeline 1286 passed, 1 skipped (49.7 s warm); unittest 3 ran, ok. Not run: `just test-full` (no code changed), CI (no PR), Codex review (no code to review).
