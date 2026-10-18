# Validation Report

Source spec: [SPEC.md](../../SPEC.md)
Source build plan: [BUILDPLAN.md](../../BUILDPLAN.md)
Source validation plan: [validation-plan.md](./validation-plan.md)
Result Validator: Codex /root with three temporary row auditors. The parent finished the interrupted checks.
Date: 2026-10-18
Run completed: 2026-10-18 11:32 EDT
Audit tree: `7573efa9cef9e91b3ea08a2f4844f5fea597b7d3` (main after PR #92), detached checkout `/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validate-2026-10-18`.
Evidence directory: [validation-evidence-2026-10-18-7573efa](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa)
Previous report preserved: [October 5 report](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/previous-report.md)

## Result

**RED against the written validation plan.** Dimmed line numbers have **3.57:1 contrast**, below the required **4.5:1** in row 100 / AC-84.
The accessibility suite passes because it explicitly accepts the lower contrast.

| Result | Rows |
|---|---:|
| Pass | **63** |
| Partial | **19** |
| Fail | **1** |
| Blocked | **0** |
| Operator checks not run | **43** |

The audit assessed **83 pre-merge rows**: 7 static and 76 runtime rows. These are not 83 unique acceptance criteria.
The 19 partial results cover outdated requirements, missing automated checks and incomplete browser observations.
This report does not establish event readiness.

## Test results — original audit at `7573efa`

**The full `just test` run failed.** The organizer PTY reveal check failed once. One isolated retry passed in both terminal modes.
The cause remains unknown. Timing sensitivity is a possible explanation.

| Check | Observed result | Evidence |
|---|---|---|
| Initial cold `just test` | Setup failed because it selected Python 3.13. project requires 3.12. The initial compilation completed. 422.48 s. | [initial log](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/test.log) |
| `just test` after Python 3.12 initialization | **Exit 2**: 953 pipeline tests pass (one q3 prose warning), 303 web pass, 325 combined Rust/organizer tests pass. Organizer PTY reveal assertion failed. 100.52 s. the full suite did not pass. | [run log](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/test-final.log) |
| One isolated PTY recheck | Legacy and enhanced modes pass. Original concurrent-run failure retained. timing sensitivity suspected, the cause remains unknown. | [recheck](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/terminal-repeat-recheck.log) |
| `just canary` | 12 pass, local only | [log](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/canary.log) |
| `just bank-audit` | Pass. no judged-difficulty sample. current arranged-position worst 1.25x | [log](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/bank-audit.log) |
| `just a11y` | 35 pass. **explicit 3.57 contrast exception remains** | [matrix/log](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/a11y.log) |
| `just secret-scan` | 2 pass | [log](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/secret-scan.log) |
| Focused receipt tests | 33 pass | [log](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/receipts.log) |
| Focused copy/runbook/wall tests | 76 pass | [log](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/copy-tests.log) |
| Chromium local rendering | 375px shipped a11y harness, plus offline date-arranged disposable fallback and follow-up keys | [browser evidence](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/browser-evidence.json), [follow-up](/Users/michellerojas/rust-nyc-pop-quiz-worktrees/validation-evidence-2026-10-18-7573efa/browser-followup.json) |


The warm run took **100.52 seconds**, above EVALUATION's **60-second** budget. Other work loaded the host during this run.
The audit did not measure a run on an otherwise idle host.

## PR #94 follow-up — 2026-10-18

[PR #94](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/94) merged at
`e4ed12f22712d1dc3f596f4d91206e3571388c15` (reviewed head `d4f1eee`).
The PTY harness now waits for expected terminal states with a three-second
deadline instead of assuming reveal output arrives within 120 ms. It drains
output during shutdown and rechecks success before reporting an exit/timeout
error, fixing the reviewed race where exit status 0 could be reported as failure.

Regression coverage includes a 350 ms child pause in both keyboard modes,
a dropped Enter that must time out with terminal evidence, and deterministic
exit-between-polls cases: exit 0 succeeds and exit 7 still fails. Existing
repeat-key and keyboard-restoration checks remain. The re-review found no
remaining blocker. [Verification record](https://github.com/mrojas54/rust-nyc-pop-quiz/blob/e4ed12f22712d1dc3f596f4d91206e3571388c15/prototypes/organizer-tui/docs/PTY_WAIT_VALIDATION.md).

Post-merge results were read back from GitHub; both workflow runs match the
merge commit above. Completion times are October 18, 2026, EDT.

| Post-merge check | Result | Completed | Evidence |
|---|---|---|---|
| CI: `just test` | **Success** | 13:04:42 | [Job](https://github.com/mrojas54/rust-nyc-pop-quiz/actions/runs/39147795120/job/117735074554) |
| CI: `just test-full` | **Success** | 13:07:14 | [Job](https://github.com/mrojas54/rust-nyc-pop-quiz/actions/runs/39147795120/job/117735074783) |
| Deployment: `fly deploy` | **Success** | 13:03:51 | [Job](https://github.com/mrojas54/rust-nyc-pop-quiz/actions/runs/39147795104/job/117735074660) |

This follow-up records the repair and successful post-merge workflows. It does
not replace the original audit evidence or establish the cause of its
uninstrumented PTY failure. F-46, all criterion results and counts, and every
remaining unchecked item below are unchanged. The CI results are not a new
row-by-row audit or venue/readiness assessment.

## Scope and limits

The audit used main at the revision shown above. It adapted the historical run instructions to current files and available tools.
It did not reuse the old commit, board counts or c11 identity commands.
The board snapshot was 52 Done, 23 Backlog, 1 In Progress and 2 Cancelled. The audit did not check or change those statuses.
PQ-77 work falls outside this revision.

Parallel reviewers reached an account usage limit. The parent kept their saved room results and browser evidence, then finished the pipeline and copy checks.
No result relies only on an interrupted agent's claim. Incomplete browser checks remain Partial.
The historical Sonnet runtime was unavailable, so the audit used available Codex agents.

The audit used a disposable bank copy for the offline fallback. It left the tracked audit checkout clean.
Setup used cached Python 3.12.13. With --no-sync, uv still printed an ambient Python selection warning, but pytest used the initialized 3.12 environment.

The audit did not run `test-full`, Docker, Fly, deployed smoke or burst checks, paid generation, live scheduling, bank review, affirmation or usage sync.
It made no source fixes, commits or tracker changes.

## Changes since October 5

The October 5 report recorded **60 Pass / 15 Partial / 2 Fail / 6 Blocked** at e239346.
Both reports cover the same 83 pre-merge rows. Changes reflect delivered code and stricter limits on what the evidence proves.
The counts do not measure effort or event readiness.

Previously blocked generator and review checks now have test evidence. The old stored-position audit failure is resolved.
Several former passes are now Partial because requirements conflict or the saved browser checks do not cover every case.
The original change table appears in the technical evidence section.

## Findings by criterion

Each Pass rests on its listed tests or observations, even though the full suite failed.
Criterion labels remain unchanged. The technical evidence section preserves exact test names, paths, links and detailed limits.

| # | Criterion | Result | Finding |
|---|---|---|---|
| 1 | AC-1 room never calls the generator | **Partial** | Source inspection found no generator dependency. The required automated dependency check is missing (PQ-47). |
| 4 | AC-3 count/topics/difficulty honoured | **Pass** | Tests check the requested count, topics and difficulty. The report lists unmet requests. |
| 5 | AC-4 talk mode tags a concept | **Pass** | Tests accept named Rust concepts and reject missing or invalid concepts. |
| 7 | AC-5 cost and wall-clock in the report | **Pass** | Each candidate has token, cost and time fields. These tests use fixtures, not a paid run. |
| 9 | AC-6 record fields and the pin | **Pass** | Tests check record fields and the toolchain pin. They also cover stale and legacy records. |
| 11 | AC-7 answer never hand-written | **Partial** | Tests reject hand-edited answers. Some build and schedule paths omit the provenance check. This audit did not run fresh rustc/Miri checks. |
| 12 | AC-8 varying output rejected | **Partial** | Tests reject output that changes across five runs. The audit did not establish a combined rejection count in the run report. |
| 14 | AC-9 Miri UB handling | **Pass** | StubRunner tests check UB handling under both borrow models. This audit did not run real Miri. |
| 16 | AC-10 Miri stdout ≠ native | **Pass** | Tests reject a mismatch between Miri and native stdout. |
| 18 | AC-11 declared non-compile | **Pass** | Tests check declared non-compiling programs, compiler codes and refusal receipts. They reject a declared non-compiling fixture that compiles. |
| 21 | AC-13 receipt from the record alone | **Pass** | All 33 receipt tests pass. Receipts need only the stored record, without a toolchain. |
| 22 | AC-87 receipt lists and take-it-home detail | **Pass** | Tests check receipt lists and take-home details for complete, legacy and non-compiling records. |
| 23 | AC-14 exact duplicate | **Pass** | Tests reject exact copies of bank programs without writing a candidate. |
| 24 | AC-15 normalized duplicate | **Pass** | Tests reject duplicates after binding names or formatting change. |
| 25 | AC-16 near-duplicate to review | **Pass** | Tests send near-duplicates to review with a reference. They neither accept nor discard them automatically. |
| 26 | AC-17 history persists | **Partial** | History persists and grows across runs. The review report and organizer screen omit the required history count and growth. |
| 27 | AC-18 uniqueness wording | **Pass** | Tests check the required uniqueness sentence and reject originality claims. |
| 28 | AC-19 review round-trips | **Pass** | Tests check accept, reject and edit flows. Decisions and rejection reasons survive round-trips. |
| 31 | AC-22 reviewer-has-seen statement | **Pass** | The review notice says that the reviewer will have seen the answer. Tests cover the notice-to-review flow. |
| 32 | AC-88 difficulty drift fails the run | **Pass** | Tests check judged difficulty and the drift threshold. The real bank has no judged sample. Operator check HC-2 remains open. |
| 34 | AC-23 slot is pure | **Pass** | The answer slot depends only on the date. Source checks and tests cover this path. |
| 35 | AC-23a attendee simulation | **Pass** | Tests run attendee strategies across 10,000 nights. Negative controls catch balancing, repeat avoidance and skew. |
| 36 | AC-23b generator both tails | **Pass** | The audit checks both overly even and overly uneven generators. Tests catch skew, balancing and repeat avoidance. |
| 37 | AC-24 five options, one does-not-compile | **Pass** | Tests and the bank audit check five options, including one does-not-compile option. |
| 38 | AC-25 no published distribution | **Pass** | The distribution scan passes for its defined files. PQ-62 covers expansion to more authored files. |
| 39 | AC-26 enumerated tells | **Partial** | The audit measures arranged positions and reports a worst result of 1.25x. The approved significance gate differs from the plan's unconditional 1.5x limit. |
| 40 | AC-27 unsafe parity | **Pass** | Tests check unsafe parity between correct and incorrect options. |
| 41 | AC-71 take-it-home sentence, wall claims none | **Pass** | Tests and browser evidence show the take-home machine/human statement. The wall makes no explanation claim. |
| 42 | AC-72 affirmation gate | **Pass** | Tests check the supported schedule affirmation gate. They do not cover pending PQ-77 server enforcement. |
| 43 | AC-73 quoted output checked | **Partial** | A quote mismatch blocks acceptance by default. Explicit --checked-quote acknowledgement permits a span judged not to claim output. The plan lacks this distinction. |
| 44 | AC-74 provenance markup | **Pass** | Tests and browser evidence show machine and human provenance markers. Operator row 45 covers human perception. |
| 46 | AC-75 reserve count and trend | **Pass** | Tests check reserve count and trend at the start of each command. |
| 47 | AC-76 low-reserve warning | **Pass** | Tests check warnings below the reserve threshold. They cover the default two-meetup threshold and a configurable lead time. |
| 49 | AC-102 static fallback file | **Pass** | The offline fallback made zero network requests. Space and Escape worked as tested. Work stopped at 5/6 and Reveal reached 6/6. |
| 50 | AC-102 host sheet | **Pass** | Tests check matching fallback and host-sheet content. Both use the same arranged record. |
| 52 | AC-28 join by link or code | **Pass** | Tests check joins through a link or a typed code. Codes accept either letter case. |
| 53 | AC-29 six failure states | **Pass** | Tests check six distinct failure messages and their recovery text. |
| 54 | AC-30 capacity before session | **Pass** | Tests reject the 201st join without a token or a reserved session. The host state stays unchanged. |
| 56 | AC-32 no source on a phone | **Pass** | Tests check that phones receive no source, trace or options. Canary checks cover pages and payloads before reveal. |
| 57 | AC-33 wall fits, phone no h-scroll | **Partial** | The sampled pages fit at 375px without horizontal page scroll. The audit used q3, not the widest question. Terminal review needs separate coverage. |
| 59 | AC-34 last answer wins | **Pass** | Tests check that the last answer wins, including changes during an unfinished write. |
| 60 | AC-35 exactly one of saving/saved/failed | **Pass** | Tests check that exactly one save state appears while live: saving, saved or failed. |
| 61 | AC-36 failed write is safe | **Pass** | Tests check that a failed write preserves the previous answer. The client names that answer and offers retry. |
| 64 | AC-39 reveal surfaces | **Pass** | Tests check reveal content on the wall, host and buzzer. The wall and host name the same incorrect option count. |
| 65 | AC-40 ✓ as well as colour | **Partial** | Tests and browser evidence show checkmarks as well as colour. This audit did not establish the exact glyph in terminal review. |
| 67 | AC-42 trope check | **Pass** | All 76 copy tests pass. They cover forbidden tropes, retired strings, prose warnings and trace notes. |
| 69 | AC-43 receipt exact lines | **Pass** | All 33 receipt tests pass. They check exact lines and wording. The wall uses the fixture lines unchanged. |
| 72 | AC-45 host actions, exactly these | **Pass** | Tests check the eight allowed host actions and the phase table. The host has no extra action buttons. |
| 73 | AC-46 live counts | **Partial** | The payload and wall carry both counts. The host intentionally shows only Answered while live. The older plan requires both counts there. |
| 74 | AC-47 host has no answer pre-reveal | **Pass** | Tests check that host pages and payloads hide the answer before reveal. |
| 75 | AC-48 hint is free and private | **Pass** | Tests check that opening a hint sends no request. Session data contains only a token and answer. |
| 76 | AC-49 host screen shape | **Partial** | The host intentionally omits a visible phase label. A polite live region announces the phase. The older plan still requires the visible label. |
| 83 | AC-56 nothing survives release | **Partial** | Tests check that release deletes all personal session data. Anonymous totals remain until four-hour expiry. USER_STORIES permits this, but the older plan does not. |
| 84 | AC-57 no identity anywhere | **Pass** | Tests and source checks find no participant identity beyond a temporary room token and answer. SPEC3.5 separately requires the organizer's Discord ID. |
| 85 | AC-58 recap computed on the phone | **Pass** | Tests check that recap taps after close send no personal data. The phone computes the recap. |
| 87 | AC-60 canary on every pre-reveal path | **Pass** | Local canary tests cover pre-reveal paths and detect planted leaks. These checks do not prove production or physical-browser behavior. |
| 88 | AC-61 sealed answers module | **Pass** | Tests check the sealed answer boundary. Reads require the reveal witness. Compile-fail tests check prohibited access. |
| 89 | AC-62 option text is public | **Partial** | The runbook contains the public-options statement. The host first screen intentionally omits it. The plan still requires it there. |
| 90 | AC-63 host can infer, not a guarantee | **Pass** | The runbook and tests state that host inference is possible. They do not promise a security guarantee. |
| 92 | AC-99 colour scoped | **Pass** | Tests check syntax colour only in the specified phases. Work and reveal have no coloured source tokens. |
| 93 | AC-79 wall is inert and clean | **Pass** | Tests check that the wall has no interactive controls. Canary checks cover the wall content. |
| 94 | AC-80 no dark mode | **Pass** | CSS and accessibility tests show only the light theme. T-16 defines no separate dim-room presentation. |
| 96 | AC-100 fit at the floor | **Partial** | Fit fixtures pass within just test, and the bank audit checks the real bank. The plan names a separate CLI fixture run that did not occur. |
| 98 | AC-82 keyboard | **Partial** | Sampled keyboard controls have visible focus outlines. Fallback keys work. The browser pass did not cover every control, failure or retry state. |
| 99 | AC-83 live regions | **Partial** | Tests check announcement text. Browser samples show host phases, Saved and wall reveal. Some transitions and actual screen-reader speech remain unchecked. |
| 100 | AC-84 AA contrast | **Fail** | Dimmed wall/home line numbers have 3.57:1 contrast, below 4.5:1. The suite explicitly accepts 3.57. F-46 remains open. PQ-78 covers home button edges. |
| 101 | AC-85 44 px targets | **Partial** | Measured controls meet the 44px minimum. The browser pass did not cover every failure, retry or responsive state. |
| 102 | AC-86 reduced motion | **Partial** | Most samples show zero active animations with reduced motion. The first host capture showed three transient animations. Their timing and effects remain unknown. |
| 103 | AC-89 one question, no next | **Pass** | Tests check one question per room and no next-question action. Run again creates a distinct room. |
| 107 | AC-92 `used` written at release only | **Partial** | Tests check that supported scheduling writes no used record and release writes it. The retired deck still writes at build time, contrary to the plan. |
| 108 | AC-93 reveal only via the walk | **Pass** | Tests check that reveal requires the walk-through. No transition skips a phase. |
| 109 | AC-94 no ✗ | **Pass** | Tests check that participant choices receive no cross, red mark or wrong label. |
| 110 | AC-95 most-chosen incorrect, same everywhere | **Pass** | Tests check the same most-chosen incorrect option on wall and host. Each incorrect option has its own explanation. |
| 113 | AC-97 the walk-through phase | **Pass** | Tests check walk-through bounds and required phase order. Work shows the trace without colour or stdout. |
| 114 | AC-98 forbidden-copy lint | **Pass** | Forbidden-copy tests pass, including trace notes. Question prose matches remain warnings. Source and option text are excluded. The no-count wording still needs reconciliation. |
| 116 | AC-64 role check, stand-in, live | **Partial** | Discord role tests pass. SPEC section8.2 requires deletion of the stand-in feature. The older plan still asks for a test with that feature enabled. |
| 118 | AC-65 roles not permissions | **Pass** | Tests require the exact role ID. Administrator or owner permissions alone do not grant access. |
| 119 | AC-66 refresh rotation persisted | **Pass** | Tests check rotated refresh tokens and sign-in recovery. Tokens persist in the organizer's memory store, not on disk. |
| 120 | AC-67 participants never authenticate | **Pass** | Tests check that participants need no Discord or organizer credentials. Temporary room tokens still authorize answer writes. |
| 121 | AC-68 only the creator controls | **Pass** | Tests check that another organizer cannot read, control or rerun the creator's room. |
| 123 | AC-70 denial wording | **Pass** | Tests check distinct wrong-server and wrong-role messages. |
| 124 | AC-101 admin channel | **Pass** | Tests check admin tokens, schedule access and used-ledger access. Scans detect planted tokens and check the repository for secrets. |
| 125 | AC-101 canary plant | **Pass** | Canary tests detect a planted admin token. They also check that the binary logs no admin token. |

## Open gaps

- **Contrast:** Row 100 fails. PQ-43 fixed host and buzzer edges, but dimmed line numbers remain below 4.5:1. PQ-78 separately covers home button edges.
- **Automated checks:** Row 1 lacks the required dependency check. Row 11 lacks provenance enforcement on every build and schedule path. Existing leads include PQ-47 and PQ-53.
- **Review:** Row 26 lacks visible history count and growth. Row 43 needs a clear distinction between output claims and other quoted spans.
- **Browser coverage:** Rows 57, 65 and 98–102 lack some required observations. The audit did not treat missing observations as passes.
- **Suite reliability:** PR #94 delivered the PTY synchronization and shutdown-race fixes; post-merge CI and deployment succeeded (see follow-up above). The original audit failure and 100.52 s warm-run observation remain historical evidence. The original failure's cause and timing on an otherwise idle host remain unverified.

## Requirements that differ from the implementation

- **Review interface:** PR #80 moved review from the browser to the organizer TUI. Some width, glyph and history requirements still assume browser review.
- **Host interface:** The host omits a visible phase label and public-options statement. It shows one count at a time. Older requirements differ in rows 73, 76 and 89.
- **Room release:** Anonymous totals remain until expiry. USER_STORIES permits this, but the older plan and EVALUATION require removal at release. Row 83 remains Partial.
- **Authentication:** SPEC §8.2 requires removal of the stand-in feature. Row 116 still asks for the removed feature's test case.
- **Answer clues:** PR #58 uses the approved significance gate and arranged positions. Row 39 still requires unconditional failure above 1.5x.
- **Retired deck:** Row 107 requires removal of the old build-time ledger writer. The deck is retired and PQ-46 is cancelled, but the writer remains.

## Recommended follow-up

These are recommendations only. The audit dispatched no fixes, created no tickets and granted no new waivers.

1. Decide the F-46 contrast remedy.
2. Complete the missing browser and terminal observations.
3. Reconcile approved implementation decisions with the written requirements.
4. PTY synchronization repair delivered in PR #94; retain the original failure's unknown cause and the unmeasured idle-host timing as explicit limits. The post-merge suite passed at the recorded merge commit.
5. Use existing PQ-47, PQ-53, PQ-78 and audit tickets where they cover the findings.

PQ-77 remains outside this report. Its future changes need a separate check.

## What remains unchecked

- All 43 operator rows, including real Miri, container containment, deployed bursts, Discord account actions and venue observations.
- Every keyboard, retry, hint and responsive state, actual screen-reader speech, widest-question terminal review and the transient animation behavior.
- Fresh execution of bank programs. This audit reran StubRunner and provenance tests only. Original Miri receipts cover only their executed paths.
- The PTY failure's cause and warm-run timing on an otherwise idle host.

Historical CI and rehearsal evidence do not count as fresh results in this report.

## Technical evidence

The following tables preserve the original audit notes verbatim. They retain all test identifiers, source paths, links and detailed limitations.
Paths refer to the pinned checkout. Shared results are in test-final.log unless a row names another log.
PR links identify the implementation source. They do not represent new PR reviews.

<details>
<summary>Original changes since October 5</summary>

| Row | Criterion | Old → new | Basis |
|---|---|---|---|
| 4 | AC-3 count/topics/difficulty honoured | Blocked → Pass | test_generate.py:365-398: ac3 clean run, short run, topics exceeding count, nothing emitted and CLI exit tests validate requested count/topics/difficulty or unmet report. |
| 5 | AC-4 talk mode tags a concept | Blocked → Pass | test_generate.py:601-622: talk fixture produces named std::mem::take and core-language concepts; missing/invalid concepts refused. |
| 7 | AC-5 cost and wall-clock in the report | Blocked → Pass | Fixture/API-equivalent numbers are not real paid-run economics. |
| 9 | AC-6 record fields and the pin | Partial → Pass | test_verify.py:166-215,461-526: machine record fields, wrong/missing pin refusals, stale/legacy cases; StubRunner parametrized cases executed. |
| 11 | AC-7 answer never hand-written | Pass → Partial | A provenance check is not called by every build/scheduling entry point; self-consistent edits are not cryptographic provenance. No fresh rustc/Miri execution in this audit. |
| 28 | AC-19 review round-trips | Blocked → Pass | test_review.py:98-116 accept/reject reason roundtrip; :270-373 edit/reverify/no-op preserving bank semantics; rejection_reasons API and reasons CLI readable by generator. |
| 31 | AC-22 reviewer-has-seen statement | Blocked → Pass | organizer-tui/src/review.rs:359 notice explicitly says reviewer will have seen answer; organizer review tests exercise notice-to-review flow. |
| 32 | AC-88 difficulty drift fails the run | Partial → Pass | Real bank has no judged sample; operator HC-2 remains separate. |
| 39 | AC-26 enumerated tells | Fail → Partial | Plan says unconditional fail above 1.5x; code applies significance gate. Owner approved A + arranged position (PQ-48, October 10), but plan wording remains stale. Historical stored-position failure resolved. |
| 41 | AC-71 take-it-home sentence, wall claims none | Partial → Pass | copy-tests.log includes wall reveal no explanation, receipt fixture rendering; home.test.js AC-71 closing pair and browser home evidence show machine/human statement. |
| 42 | AC-72 affirmation gate | Partial → Pass | This verifies supported scheduling gate, not pending PQ-77 server-side enforcement. |
| 43 | AC-73 quoted output checked | Blocked → Partial | Explicit --checked-quote acknowledgement can accept a mismatching span as not an output claim. Strict plan says mismatch blocks acceptance without qualification; TUI asks the organizer to confirm interpretation. |
| 64 | AC-39 reveal surfaces | Partial → Pass | web/test/wall.test.js reveal totals named option receipt and no explanation; web/test/host.test.js AC-74 three beats; web/test/buzzer.test.js reveal letter and bars; room_record same named incorrect count. |
| 73 | AC-46 live counts | Pass → Partial | Strict plan requires both present and answered while live. Current UI intentionally shows only Answered live; payload and wall still carry both. Updated SPEC phase table reflects simplified UI; validation plan/EVALUATION remain unreconciled. |
| 83 | AC-56 nothing survives release | Pass → Partial | Strict plan/EVALUATION require room gone at release. USER_STORIES AC-56 permits anonymous totals until room expiry; implementation follows latter. No per-person data survives release; not a privacy-test failure. |
| 96 | AC-100 fit at the floor | Pass → Partial | Named bank-audit CLI fixture-set execution is absent; equivalent fixtures run within just test, not through that CLI. Method discrepancy preserved. |
| 98 | AC-82 keyboard | Pass → Partial | Saved browser pass did not exhaust every enabled control and failure/retry state. Review moved to TUI. Full rendered keyboard acceptance not claimed. |
| 99 | AC-83 live regions | Pass → Partial | All failure/retry/hint transitions were not captured in the rendered pass; actual screen-reader speech not tested. |
| 101 | AC-85 44 px targets | Pass → Partial | Rendered failure/retry and all responsive variants not fully traversed; measured-state results only. |
| 102 | AC-86 reduced motion | Pass → Partial | First browser capture transiently saw three host animations after phase1; no timing/effect data saved for those. Could not establish literal zero-animation invariant across all transitions. |
| 110 | AC-95 most-chosen incorrect, same everywhere | Partial → Pass | room_record::the_most_chosen_incorrect_option_is_named_the_same_on_wall_and_host, nobody_read_it_another_way, the_why_tempting_text_is_the_named_options_own; take_home::every_incorrect_option_has_its_beat_and_none_carries_a_count; pipeline/tests/test_review.py::test_ac95_every_incorrect_option_needs_its_why_tempting. |
| 114 | AC-98 forbidden-copy lint | Partial → Pass | Question prose matches remain warnings under recorded lint policy; raw source/option text excluded. AC-98 literal no-count language conflicts with intended Joined/Answered counts; plan narrows this to contribution counts. |
| 116 | AC-64 role check, stand-in, live | Pass → Partial | SPEC section8.2 expressly requires T-10 to delete stand-in feature/code. Feature-on stand-in build case no longer exists; actual Discord role tests and no-stand-in scan pass. Partial against literal old plan, intentional retirement rather than auth regression. |

</details>

<details>
<summary>Original evidence for all 83 result rows</summary>

| # | Criterion | Result | Evidence and limitations |
|---|---|---|---|
| 1 | AC-1 room never calls the generator | **Partial** | Read room/Cargo.toml dependency list and source graph; no pipeline/generator/LLM dependency. **Limitation:** Required dedicated static dependency check is absent (PQ-47); source inspection does not establish a passing automated guard. [#61](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/61) |
| 4 | AC-3 count/topics/difficulty honoured | **Pass** | test_generate.py:365-398: ac3 clean run, short run, topics exceeding count, nothing emitted and CLI exit tests validate requested count/topics/difficulty or unmet report. [#61](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/61), [#62](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/62) |
| 5 | AC-4 talk mode tags a concept | **Pass** | test_generate.py:601-622: talk fixture produces named std::mem::take and core-language concepts; missing/invalid concepts refused. [#61](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/61), [#62](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/62) |
| 7 | AC-5 cost and wall-clock in the report | **Pass** | test_generate.py:629-652: each candidate has tokens, cost_usd and wall_clock_s; fixture cost formula and elapsed 7.5s pinned. **Limitation:** Fixture/API-equivalent numbers are not real paid-run economics. [#61](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/61), [#62](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/62) |
| 9 | AC-6 record fields and the pin | **Pass** | test_verify.py:166-215,461-526: machine record fields, wrong/missing pin refusals, stale/legacy cases; StubRunner parametrized cases executed. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16), [#40](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/40), [#45](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/45) |
| 11 | AC-7 answer never hand-written | **Partial** | test_verify.py:553-598 verifies bank provenance and refusal of hand-edited answer/kind/stored-correct; test_review.py:403 proves decisions preserve verified. **Limitation:** A provenance check is not called by every build/scheduling entry point; self-consistent edits are not cryptographic provenance. No fresh rustc/Miri execution in this audit. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16), [#40](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/40), [#45](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/45), [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 12 | AC-8 varying output rejected | **Partial** | test_verify.py:98-141: differing repeated output is rejected, not flagged; RUNS=5 and per-candidate rejection tested. **Limitation:** The plan also demands rejection count in a run report; generator drafts and verifier remain separate stages, and an aggregated varying-output rejection report was not established. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16), [#61](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/61) |
| 14 | AC-9 Miri UB handling | **Pass** | test_verify.py parametrized StubRunner cases and test_ub_under_one_borrow_model_is_refused; Tree Borrows only for declared UB, both models required. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16) |
| 16 | AC-10 Miri stdout ≠ native | **Pass** | test_verify.py::test_the_verifier_decides_each_case_as_recorded includes miri-differs rejection; verify.py compares Miri/native stdout. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16) |
| 18 | AC-11 declared non-compile | **Pass** | test_verify.py:122,203,427: does-not-compile runs nothing, captures compiler code order, renders refusal receipt; compiling declared-noncompile fixture rejected. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16) |
| 21 | AC-13 receipt from the record alone | **Pass** | 33 receipt tests passed, including test_the_receipt_module_needs_no_toolchain_and_no_world and complete/legacy/no-receipt cases. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16) |
| 22 | AC-87 receipt lists and take-it-home detail | **Pass** | 33 receipt fixtures/tests passed; receipt.py step gating, complete/legacy/noncompile detail; home.test.js:165-192 and Rust receipt twin exercised by shared suite. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16), [#34](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/34) |
| 23 | AC-14 exact duplicate | **Pass** | test_dedupe.py byte-identical resubmission tests reject each committed bank program and write no candidate. [#14](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/14) |
| 24 | AC-15 normalized duplicate | **Pass** | test_dedupe.py renamed-bindings/reformatted fixture cases and renamed bank-question rejection; current normalization regression fixtures exercised. [#14](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/14), [#51](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/51) |
| 25 | AC-16 near-duplicate to review | **Pass** | test_dedupe.py:874-909: near duplicate marked with reference, lands in review queue neither accepted nor dropped. [#14](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/14), [#51](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/51) |
| 26 | AC-17 history persists | **Partial** | test_dedupe.py:961 two-run/two-directory growth and :1025 status size tests exercise persistent history and report. **Limitation:** Review.show report and organizer review screen do not display history count/growth, required by plan. [#14](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/14), [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 27 | AC-18 uniqueness wording | **Pass** | test_dedupe.py:1136,1140 asserts exact uniqueness sentence across corpus and no originality claim. [#14](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/14) |
| 28 | AC-19 review round-trips | **Pass** | test_review.py:98-116 accept/reject reason roundtrip; :270-373 edit/reverify/no-op preserving bank semantics; rejection_reasons API and reasons CLI readable by generator. [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80), [#81](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/81) |
| 31 | AC-22 reviewer-has-seen statement | **Pass** | organizer-tui/src/review.rs:359 notice explicitly says reviewer will have seen answer; organizer review tests exercise notice-to-review flow. [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 32 | AC-88 difficulty drift fails the run | **Pass** | test_review.py:123 records judged difficulty 1..5; test_audit.py:827-851 fails run at mean 4/3, passes exactly 1, excludes unjudged and reports no sample. **Limitation:** Real bank has no judged sample; operator HC-2 remains separate. [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13), [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 34 | AC-23 slot is pure | **Pass** | slot.py:55 pure slot_for_day(day,n_options=5), imports hashlib/date only. test_audit.py slot/static-lint fixtures and test_schedule.py:190,206 cover slot path. [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13), [#47](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/47) |
| 35 | AC-23a attendee simulation | **Pass** | test_audit.py:150-176 executes 10,000-night attendee strategies and balancer/never-repeat/skew negative controls; bank-audit.log gives measured rates. [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13) |
| 36 | AC-23b generator both tails | **Pass** | bank-audit.log generator both-tail audit; test_audit.py:122,127,134 catches skew, rebalance and never-repeat generators. [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13) |
| 37 | AC-24 five options, one does-not-compile | **Pass** | bank-audit.log five-options check; test_audit.py:328,347 validates real bank and malformed option shapes. [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13) |
| 38 | AC-25 no published distribution | **Pass** | bank-audit.log authored-file distribution lint clean; test_audit.py:357-382 real corpus and planted distribution cases. **Limitation:** Scan scope is explicitly bounded; PQ-62 retains authored-file coverage expansion. [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13) |
| 39 | AC-26 enumerated tells | **Partial** | bank-audit.log now measures wall arrangement over 1000 synthetic nights (worst 1.25x), all five tells measured. test_audit.py:558 proves four-question planted tell warns, five fails. **Limitation:** Plan says unconditional fail above 1.5x; code applies significance gate. Owner approved A + arranged position (PQ-48, October 10), but plan wording remains stale. Historical stored-position failure resolved. [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13), [#58](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/58) |
| 40 | AC-27 unsafe parity | **Pass** | test_audit.py:795-813 planted UB-without-decoy fails, accepted unsafe non-UB restores parity, unaccepted excluded. Real bank has no UB answer (reported explicitly). [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13) |
| 41 | AC-71 take-it-home sentence, wall claims none | **Pass** | copy-tests.log includes wall reveal no explanation, receipt fixture rendering; home.test.js AC-71 closing pair and browser home evidence show machine/human statement. [#21](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/21), [#34](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/34) |
| 42 | AC-72 affirmation gate | **Pass** | test_review.py:168-209 identity, who/when/digest, <2 trace refusal; test_schedule.py:226-337 unaffirmed/unaccepted/stale/unbound refusal and no writes. **Limitation:** This verifies supported scheduling gate, not pending PQ-77 server-side enforcement. [#47](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/47), [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80), [#82](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/82) |
| 43 | AC-73 quoted output checked | **Partial** | test_review.py:141-153 mismatch blocks acceptance and leaves file unchanged; :228 blocks affirmation. **Limitation:** Explicit --checked-quote acknowledgement can accept a mismatching span as not an output claim. Strict plan says mismatch blocks acceptance without qualification; TUI asks the organizer to confirm interpretation. [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 44 | AC-74 provenance markup | **Pass** | browser-evidence.json home shows machine and human data-provenance markers; host/wall fixtures and tests assert by-human/by-machine markup. **Limitation:** Human perception remains operator row45. [#21](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/21), [#22](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/22), [#34](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/34) |
| 46 | AC-75 reserve count and trend | **Pass** | test_schedule.py:628 first lines of every command are reserve/trend; count and trend tests and current bank audit validate bound-affirmation reserve. [#47](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/47), [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 47 | AC-76 low-reserve warning | **Pass** | test_schedule.py:660-676 verifies warning below threshold, default two meetups and configurable lead time. [#47](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/47) |
| 49 | AC-102 static fallback file | **Pass** | Offline Chromium on supported date-arranged disposable q3 fallback: zero network requests; Space phases, work clamped at 5/6, Reveal at 6/6, Escape returns work1/6; no correctness/receipt in work. wall-static.test.js parity/driver tests passed in 303-test web suite. **Limitation:** Browser uses committed fixture rendering, not production. No fresh verification of question output claimed. [#24](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/24), [#47](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/47) |
| 50 | AC-102 host sheet | **Pass** | test_fallback.py:237-281 every beat/note verbatim, one block per phase, paired writes/refusal; test_schedule.py:428-445 same arranged record and no-push outputs. [#24](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/24), [#47](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/47) |
| 52 | AC-28 join by link or code | **Pass** | sessions::ac28_join_by_code_in_any_case_and_nothing_else; web/test/buzzer.test.js AC-28 one field, link joins without typing, typed code attaches. [#18](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/18), [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23), [#42](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/42) |
| 53 | AC-29 six failure states | **Pass** | sessions::ac29_each_failure_has_its_own_sentence, ac29_code_shape; web/test/buzzer.test.js each refusal shows its own section11 sentence; web/shared/copy.js six recovery messages. [#18](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/18), [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23) |
| 54 | AC-30 capacity before session | **Pass** | sessions::ac30_the_201st_join_is_refused_and_reserves_nothing asserts no token, exactly 200 sessions and unchanged host state. [#18](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/18) |
| 56 | AC-32 no source on a phone | **Pass** | canary::every_phase_every_surface_the_answer_stays_sealed_until_reveal; web/test/buzzer.test.js G-8 AC-32 reads no source trace or options; page and payload probes in canary_scan. [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25), [#31](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/31) |
| 57 | AC-33 wall fits, phone no h-scroll | **Partial** | 375px Chromium captures of home/host/buzzer/wall show no horizontal document scroll; home source scroller has bounded width. **Limitation:** Harness uses q3, not widest bank question. Browser review page no longer exists (review moved to TUI); widest-question 375px review criterion not fully executed. [#34](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/34), [#64](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/64), [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 59 | AC-34 last answer wins | **Pass** | sessions::ac34_five_changes_then_close_then_refused_with_the_saved_answer; web/test/buzzer.test.js taps while a write is in flight last wins. [#18](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/18), [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23) |
| 60 | AC-35 exactly one of saving/saved/failed | **Pass** | sessions::ac35_response_contract; web/test/buzzer.test.js AC-35 exactly one of saving/saved/failed at all times while live. [#18](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/18), [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23) |
| 61 | AC-36 failed write is safe | **Pass** | sessions::ac36_a_failed_write_leaves_the_previous_answer_intact; web/test/buzzer.test.js AC-36 failed write names last saved answer safe and offers retry. [#18](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/18), [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23) |
| 64 | AC-39 reveal surfaces | **Pass** | web/test/wall.test.js reveal totals named option receipt and no explanation; web/test/host.test.js AC-74 three beats; web/test/buzzer.test.js reveal letter and bars; room_record same named incorrect count. [#21](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/21), [#22](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/22), [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23), [#31](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/31) |
| 65 | AC-40 ✓ as well as colour | **Partial** | Wall, buzzer and home automated reveal tests assert checkmark; browser home/fallback confirms redundant correctness glyph. **Limitation:** Review is now TUI; exact glyph requirement across that surface not independently established in this pass. [#21](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/21), [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23), [#34](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/34), [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 67 | AC-42 trope check | **Pass** | copy-tests.log 76 passed: exact SPEC trope patterns, every retired-string group fires, COPY matches fail; prose warning and trace-note tests pass. [#38](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/38), [#85](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/85), [#86](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/86) |
| 69 | AC-43 receipt exact lines | **Pass** | receipts.log 33 passed: exact fixture lists, UB flag, panic four-line, q8 three-line, no terminal punctuation or claim words; wall G-7 prints fixture lines verbatim. [#16](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/16), [#21](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/21) |
| 72 | AC-45 host actions, exactly these | **Pass** | phase_table::every_state_and_command_does_exactly_what_the_table_says, the_host_actions_are_exactly_the_eight_of_ac_45; web/test/host.test.js eight actions plus arrows and no other buttons. [#15](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/15), [#22](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/22) |
| 73 | AC-46 live counts | **Partial** | sessions::ac46_counts_move_on_the_host_phone_while_live and host_page::ac46_the_host_socket_counts_move_while_live prove payload updates. web/test/host.test.js explicitly asserts Joined only idle and Answered only live onward. **Limitation:** Strict plan requires both present and answered while live. Current UI intentionally shows only Answered live; payload and wall still carry both. Updated SPEC phase table reflects simplified UI; validation plan/EVALUATION remain unreconciled. [#22](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/22), [#27](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/27), [#29](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/29) |
| 74 | AC-47 host has no answer pre-reveal | **Pass** | host_page::ac47_no_page_or_payload_the_host_sees_carries_the_answer_before_reveal; canary::every_phase_every_surface_the_answer_stays_sealed_until_reveal. [#22](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/22), [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25) |
| 75 | AC-48 hint is free and private | **Pass** | web/test/buzzer.test.js AC-48 hint read from held frame without request and through shell calls neither fetch nor socket; Session fields token/answer only; canary walks hints privately. [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23), [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25) |
| 76 | AC-49 host screen shape | **Partial** | host_page::ac45_ac49_each_phase_offers_one_action_and_the_room_accepts_only_that_one verifies payload label. web/test/host.test.js every phase headed wordmark and Host explicitly asserts no visible phase label; code and one action remain. **Limitation:** Strict visible phase-label clause is unmet deliberately since PQ-34; phase is announced through polite live region and carried in payload. Partial is contract drift, not failing test. [#22](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/22), [#29](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/29) |
| 83 | AC-56 nothing survives release | **Partial** | sessions::ac56_after_release_no_session_remains; lifecycle::ac56_nothing_per_person_survives_release, ac56_the_totals_expire_with_the_released_room, ac56_the_survivors_carry_no_count. Released shell and anonymous frozen totals intentionally survive until four-hour expiry. **Limitation:** Strict plan/EVALUATION require room gone at release. USER_STORIES AC-56 permits anonymous totals until room expiry; implementation follows latter. No per-person data survives release; not a privacy-test failure. [#30](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/30) |
| 84 | AC-57 no identity anywhere | **Pass** | sessions::ac57_a_session_is_a_token_and_an_answer checks exhaustive two-field Session; room/src/connections/sessions.rs and participant route/client state inspection show ephemeral per-room token/answer, no nickname score or cross-room identity. **Limitation:** No identity means participant data; organizer Discord ID is explicitly required by SPEC3.5. [#18](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/18), [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23), [#30](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/30) |
| 85 | AC-58 recap computed on the phone | **Pass** | web/test/buzzer.test.js AC-58 after close taps send nothing; canary::every_phase_every_surface_the_answer_stays_sealed_until_reveal invokes assert_nothing_personal_after_close scanning browser fetch/ws and server traffic. [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23), [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25) |
| 87 | AC-60 canary on every pre-reveal path | **Pass** | canary::every_phase_every_surface_the_answer_stays_sealed_until_reveal, the_receipt_plant_stays_sealed_until_reveal_does_not_compile, every_route_in_routes_rs_is_scanned_or_listed, the_rules_catch_a_planted_leak. **Limitation:** Local in-process HTTP/rendered-page canary evidence, not a production or physical-browser proof. [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25), [#72](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/72), [#79](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/79) |
| 88 | AC-61 sealed answers module | **Pass** | boundary::the_vault_holds_only_the_allowed_functions_and_every_read_takes_the_witness, the_pre_reveal_builders_never_see_the_opened_room, within_rooms_only_open_rs_names_the_sealed_answer_types; models/answers.rs private vault and phase RevealWitness compile-fail/twin doctests. [#15](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/15), [#72](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/72), [#79](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/79) |
| 89 | AC-62 option text is public | **Partial** | runbook.test.js:59 exact public-options sentence passes. **Limitation:** Host first screen intentionally omits sentence since PQ-34; plan still requires it there. EVALUATION requires organizer docs only. [#27](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/27), [#48](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/48) |
| 90 | AC-63 host can infer, not a guarantee | **Pass** | runbook.test.js:64 exact host-inference/not-security-guarantee sentence passes; docs/RUNBOOK.md carries it. [#48](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/48) |
| 92 | AC-99 colour scoped | **Pass** | web/test/wall.test.js AC-99 token spans live/closed/split zero work/reveal and no source idle/released. [#21](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/21) |
| 93 | AC-79 wall is inert and clean | **Pass** | web/test/wall.test.js AC-79 no interactive control every phase; canary::every_phase_every_surface_the_answer_stays_sealed_until_reveal and rendered-wall probes. [#21](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/21), [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25) |
| 94 | AC-80 no dark mode | **Pass** | Read shipped CSS and passing a11y no-dark-mode assertions: only light presentation, no prefers-color-scheme theme. **Limitation:** Dim-room criterion has no separate built presentation, per T-16. [#9](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/9), [#37](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/37) |
| 96 | AC-100 fit at the floor | **Partial** | test_fit.py:188-204 runs too-long program/option and passing fixtures with exact flags; bank-audit.log checks real bank. **Limitation:** Named bank-audit CLI fixture-set execution is absent; equivalent fixtures run within just test, not through that CLI. Method discrepancy preserved. [#13](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/13) |
| 98 | AC-82 keyboard | **Partial** | 35 a11y tests pass keyboard/focus invariants; Chromium Tab observations show visible 2px/3px outlines on sampled home/host/buzzer/wall controls; fallback keys executed. **Limitation:** Saved browser pass did not exhaust every enabled control and failure/retry state. Review moved to TUI. Full rendered keyboard acceptance not claimed. [#37](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/37), [#41](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/41), [#91](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/91) |
| 99 | AC-83 live regions | **Partial** | 35-test a11y matrix covers announced strings; browser captures polite host phase labels, buzzer Saved string and wall reveal. **Limitation:** All failure/retry/hint transitions were not captured in the rendered pass; actual screen-reader speech not tested. [#37](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/37), [#41](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/41), [#91](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/91) |
| 100 | AC-84 AA contrast | **Fail** | a11y.log prints wall/home dim line-number contrast 3.57:1 and intentionally holds floor at 3.57; normal text AA requires 4.5:1. Host/buzzer edges now pass 3:1 after PR91. **Limitation:** Known F-46 remains; green a11y suite explicitly tolerates it. Home trace-button edge coverage remains PQ-78. [#37](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/37), [#41](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/41), [#91](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/91) |
| 101 | AC-85 44 px targets | **Partial** | a11y assertions pass minimum44; Chromium measured host primary343x44, steps44x44, buzzer letter351x102+, hint351x44 and home44x44. **Limitation:** Rendered failure/retry and all responsive variants not fully traversed; measured-state results only. [#37](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/37), [#41](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/41), [#91](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/91) |
| 102 | AC-86 reduced motion | **Partial** | 35-test a11y suite checks reduced-motion CSS/static cues. Chromium reducedMotion true; sampled states zero active animations; follow-up host sampled at100ms and600ms reports zero. **Limitation:** First browser capture transiently saw three host animations after phase1; no timing/effect data saved for those. Could not establish literal zero-animation invariant across all transitions. [#37](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/37), [#41](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/41), [#91](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/91) |
| 103 | AC-89 one question, no next | **Pass** | phase_table::every_state_and_command_does_exactly_what_the_table_says, released_never_re_enters_the_room; canary::run_it_again_makes_a_new_room_and_never_reruns_the_question. Exhaustive commands lack next-question; run again creates distinct room. [#15](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/15), [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25), [#50](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/50) |
| 107 | AC-92 `used` written at release only | **Partial** | Room half only: used::ac92_used_is_written_at_release, ac92_nothing_is_written_before_release, ac92_an_expired_or_quiet_room_records_nothing, ac92_scheduling_questions_writes_nothing, g10_the_release_transition_is_the_only_writer. Room release-ledger tests and test_schedule.py:461 show supported scheduling does not write bank; unreleased room does not spend a question. **Limitation:** Retired mvp/tools/build_deck.py:221-233 still writes answer-history at build. Plan explicitly demands its removal; PQ-46 cancelled when deck retired. No deck was built. [#30](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/30), [#47](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/47) |
| 108 | AC-93 reveal only via the walk | **Pass** | phase_table::no_transition_skips_a_phase_and_reveal_needs_the_walk_through and exhaustive state-command table. [#15](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/15) |
| 109 | AC-94 no ✗ | **Pass** | web/test/buzzer.test.js AC-94 no cross/red/wrong against participant choice in any phase; web/test/home.test.js exactly one check and no cross plus forbidden strings scan. [#23](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/23), [#34](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/34) |
| 110 | AC-95 most-chosen incorrect, same everywhere | **Pass** | room_record::the_most_chosen_incorrect_option_is_named_the_same_on_wall_and_host, nobody_read_it_another_way, the_why_tempting_text_is_the_named_options_own; take_home::every_incorrect_option_has_its_beat_and_none_carries_a_count; pipeline/tests/test_review.py::test_ac95_every_incorrect_option_needs_its_why_tempting. [#15](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/15), [#34](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/34), [#80](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/80) |
| 113 | AC-97 the walk-through phase | **Pass** | phase_table::trace_step_bounds_on_a_five_step_trace, trace_step_bounds_on_the_shortest_trace, no_transition_skips_a_phase_and_reveal_needs_the_walk_through; web/test/wall.test.js work trace without colour or stdout; canary phase/surface scan. [#15](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/15), [#21](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/21), [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25) |
| 114 | AC-98 forbidden-copy lint | **Pass** | copy-tests.log: nine Forbidden patterns exactly match SPEC; trace-note probes cover every family; all COPY and frozen surface strings checked; no contribution prompt or wait at zero participants. **Limitation:** Question prose matches remain warnings under recorded lint policy; raw source/option text excluded. AC-98 literal no-count language conflicts with intended Joined/Answered counts; plan narrows this to contribution counts. [#38](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/38), [#85](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/85), [#86](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/86) |
| 116 | AC-64 role check, stand-in, live | **Partial** | auth::ac64_a_member_with_the_role_creates_a_room, ac64_a_member_without_the_role_is_denied, ac64_no_stand_in_survives_in_src_cargo_dockerfile_or_ci, ac64_any_bearer_that_is_not_an_organizer_session_is_401. **Limitation:** SPEC section8.2 expressly requires T-10 to delete stand-in feature/code. Feature-on stand-in build case no longer exists; actual Discord role tests and no-stand-in scan pass. Partial against literal old plan, intentional retirement rather than auth regression. [#32](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/32) |
| 118 | AC-65 roles not permissions | **Pass** | auth::ac65_an_administrator_and_owner_without_the_role_id_is_denied, ac65_the_role_id_is_compared_whole, ac65_the_member_record_is_read_for_roles_alone. [#32](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/32), [#79](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/79) |
| 119 | AC-66 refresh rotation persisted | **Pass** | auth::ac66_each_refresh_persists_the_rotated_token_and_presents_it_next, ac66_two_creates_at_once_refresh_once, ac66_a_refused_refresh_signs_the_organizer_out_and_leaves_their_open_room. Spent token signs out once; fresh sign-in succeeds. **Limitation:** Persisted within organizer in-memory store; fresh sign-in recovery verified, not durable disk persistence. [#32](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/32) |
| 120 | AC-67 participants never authenticate | **Pass** | auth::ac67_no_participant_route_touches_the_auth_module, ac67_participants_need_and_carry_no_credential; server/routes/connections.rs read confirms participant session bearer is ephemeral room token, no Discord auth. **Limitation:** Credential means organizer/Discord authentication; ephemeral room-session tokens intentionally authorize answer writes. [#32](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/32), [#72](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/72), [#79](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/79) |
| 121 | AC-68 only the creator controls | **Pass** | auth::ac68_organizer_b_cannot_read_or_control_as_room, ac68_an_organizer_session_is_not_a_host_session, ac68_organizer_b_cannot_run_it_again_on_as_room. [#32](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/32) |
| 123 | AC-70 denial wording | **Pass** | auth::ac70_a_non_member_and_another_guilds_member_get_the_same_wrong_server, ac70_a_member_without_the_role_hears_wrong_role. [#32](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/32) |
| 124 | AC-101 admin channel | **Pass** | admin::ac101_a_missing_or_wrong_token_is_refused_saying_nothing, ac101_the_right_token_schedules_and_reads_the_used_ledger, ac101_the_admin_prefix_is_served_to_the_check_alone, ac101_the_repository_holds_no_admin_token, ac101_the_repository_scan_catches_a_plant. [#33](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/33), [#52](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/52) |
| 125 | AC-101 canary plant | **Pass** | canary::the_admin_rule_catches_a_planted_token, the_binary_logs_no_admin_token; admin canary scan across every walked surface. [#25](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/25), [#33](https://github.com/mrojas54/rust-nyc-pop-quiz/pull/33) |

</details>

## Operator smoke-pass checklist (post-merge)

The following 43 plan rows remain verbatim source text. **This validator did not run them.**

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

## Report checks

- All **126** plan rows appear: **83** pre-merge results and **43** operator rows. Result IDs have no gaps or duplicates.
- The result counts remain **63 + 19 + 1 + 0 = 83**.
- The old plan footer says 8 static, 74 runtime and 44 smoke rows. Actual tags give 7, 76 and 43. The plan remains unchanged.
- The run included one audit and one targeted PTY retry. It created no self-wakeup or heartbeat.
- This plain-English edit changes wording and layout only. The original evidence tables and operator checklist remain unchanged.
