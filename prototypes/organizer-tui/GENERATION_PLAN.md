# Generation plan

Status: proposed implementation, 2026-10-10. No generation controls or provider
calls are implemented by this document.

An organizer requests a small batch, sees which candidates passed the machine
checks, and opens each question in the existing preview. The teaching trace is
the main review tool. The answer stays behind an explicit reveal.

The first complete slice ends with one newly generated, machine-verified
candidate open in this TUI. It does not schedule a question or open a live room.

## What exists

| Part | Current source | Work needed |
| --- | --- | --- |
| Question preview and teaching trace | `src/lib.rs` | Load staged candidates and refresh when a run finishes |
| Generation | `../../pipeline/src/popquiz/generate.py` | Empty stub; implement bounded batch jobs |
| Verification | `../../pipeline/src/popquiz/verify.py` | Reuse pinned toolchain checks and recorded evidence |
| Program execution | `../../pipeline/src/popquiz/runner.py` | Use the existing sandbox runner, never the TUI process |
| Duplicate checks | `../../pipeline/src/popquiz/dedupe.py` | Reuse checks against bank history and the current batch |
| Review decisions | `../../pipeline/src/popquiz/review.py` | Empty stub; implement separately after preview works |
| Record format | `../../pipeline/src/popquiz/bank.py` | Reuse the schema; keep job state outside question records |

These paths and stub states were checked in this worktree. This plan follows
[SPEC §7](../../SPEC.md#7-the-pipeline), T-16/T-18 in
[BUILDPLAN.md](../../BUILDPLAN.md), and [PHILOSOPHY.md](../../PHILOSOPHY.md).

## Organizer flow

1. Press `g` from the question view to open Generate. This is a proposed shortcut.
   Tab moves through count, topics, difficulty, and optional public talk title
   and abstract. Default count: three candidates for review, not three questions
   for a meetup. Room capacity comes from configuration.
2. Review the request, configured provider/model, and spend limit. Enter on
   “Generate batch” submits it; Escape returns without starting a job. Missing
   credentials or a spend limit produces a specific setup message.
3. Watch the batch list: queued, generating, verifying, duplicate check,
   preparing walkthrough, ready for review, or failed. Show elapsed time and
   completed counts, not a guessed percentage. The TUI remains navigable.
4. Select a ready candidate with arrows or `j/k`; Enter opens the unspoiled
   question preview. List rows show topic, requested difficulty, and status.
   They never show the answer category, output, resolving note, or receipt.
5. Use `t` for the teaching trace, Tab for pane focus, and arrows to rehearse.
   The final resolving step remains unavailable through normal navigation.
   Press `r` to reveal the answer, explanation, and verification evidence.
6. Return to the batch to inspect another candidate. A failed row opens its
   failure details rather than pretending it is a reviewable question.

Failure details can contain compiler output and spoilers; label that view before
opening it. Provider request logs are diagnostics, separate from the authored
teaching walkthrough. Neither is presented as a model's private reasoning.

## Pipeline and ownership

Keep generation in Python beside verification and dedupe. Ratatui starts or
attaches to a worker and renders its saved job state. Use a versioned JSON event
stream for progress and an on-disk manifest for recovery; do not parse prose logs.
The worker owns writes. Each event carries run ID, candidate ID, stage, and time.

Proposed private staging directory: `bank/runs/<run-id>/`, excluded from Git by
default when implemented. Store the request, manifest, drafts, verification
evidence, diagnostics, and ready records there. Only ready records appear under
the staging bank's `questions/` directory so the existing `--bank` loader can be
reused. Never put incomplete drafts in the main review queue.

For each candidate:

1. Generate source and a structured draft without `verified`, `review`, or a
   stored correct-answer field. Validate requested topics, difficulty, room
   limits, and schema. Report unmet constraints; do not silently trim code.
2. Verify with the existing pinned rustc/Miri procedure and sandbox. A provider's
   predicted result is not evidence. Infrastructure errors and timeouts remain
   failures with a retry action, never a “does not compile” answer.
3. Run duplicate checks against the bank, its history, and this batch. Reject
   exact/normalized duplicates; retain near-duplicate details for human judgment
   under the existing policy. Say “no exact or normalized duplicate found.”
4. Prepare the five options, explanations, and teaching trace from the verified
   result. The correct output text comes from the verifier; model-written
   distractors are proposals. Require exactly one matching answer and the
   existing option-kind and length rules. Source or option edits trigger
   re-verification before the candidate becomes ready.
5. Validate trace line references, focus ranges, minimum step count, and final
   resolving-step boundary. Compare quoted output and the final stdout value
   with the saved evidence where applicable. Run copy checks. Intermediate
   values and narration remain authored explanations requiring organizer review;
   a clean program execution does not prove the walkthrough is correct.
6. Save a ready record and emit its path. Preserve failures and diagnostics in
   staging. Report the actual yield even if fewer than the requested count pass.

The existing `dedupe.run` writes admitted records into a bank by default. Use its
dry-run mode while staging. Promotion must recheck current bank history and
fresh IDs before using the existing admission path; preview alone never promotes.

## Review after the first slice

Add accept, reject with a reason, and edit as a separate implementation step.
Affirmation remains a distinct action with reviewer identity and timestamp.
Acceptance alone cannot satisfy the affirmation gate. Record judged difficulty
separately from requested difficulty, and apply the existing explanation/trace
requirements before affirmation. State that the reviewer has seen the answer.

Source or option changes invalidate verification and prior approval. Explanation
or trace changes require renewed explanation review and affirmation. Preserve the
previous revision and its evidence so a reviewer can see what changed.

Admission adds an unreviewed candidate to the private bank. Acceptance and
affirmation record human judgments. Scheduling and recording actual meetup use
remain separate operations. Generation must never update the usage ledger or
alter date-based answer positioning.

## Limits and recovery

- Persist provider job IDs before polling. Restart attaches to the same job;
  it must not submit a second paid request automatically.
- Set attempt and spend limits per run. Retries consume the same budget. Report
  tokens, elapsed time, and actual or estimated cost with its basis; unknown cost
  remains unknown. Do not automatically refill a batch until it reaches count.
- Cancel stops new local work and requests provider cancellation where supported.
  Keep completed candidates and explain if submitted work may still be charged.
- Record source hashes, prompt/schema version, provider/model, and toolchain
  provenance. Write records atomically and keep credentials out of artifacts.
- Keep generation offline from the room. Do not balance answer categories or
  answer positions, and do not use prior answer letters to choose candidates.

T-16 names an Anthropic batch adapter. Confirm the configured model, credentials,
and spend cap before a paid trial; this document does not claim that the model
named in the older contract is currently available. Provider selection belongs
behind the worker interface, not in Ratatui widgets.

## Build order and acceptance

| Step | Deliverable | Evidence before moving on |
| --- | --- | --- |
| 1 | Worker request, manifest, progress events, and fixture adapter | Restart resumes a saved run; malformed responses and budget exhaustion stay explicit |
| 2 | Staged generation → verification → dedupe → walkthrough | Recorded fixtures cover success, verification failure, duplicate, near-duplicate, and partial batch; no main-bank writes |
| 3 | Generate form and batch list in Ratatui | Keyboard tests cover submit/back/cancel/reopen; responsive UI while a worker runs; ready candidate opens the existing preview |
| 4 | Configured provider adapter and bounded real trial | One fresh candidate verified with the real sandbox and opened for question/trace/reveal review; cost and receipt saved |
| 5 | Review decisions and deliberate bank admission | Required reasons and affirmation enforced; edits invalidate stale evidence; concurrent duplicate admission is refused |

Use RED → GREEN → VERIFY → DOCS for each implementation slice. Run Rust checks
from this crate and Python checks from `pipeline/`, where its `pyproject.toml`
lives. Fixture tests establish control flow; they do not count as fresh Rust/Miri
verification. Update the dark terminal recording after the generation flow works.

The completion demo is concrete: request three candidates, inspect the actual
yield, open one verified candidate, rehearse its teaching steps, and deliberately
reveal its answer. No live-room dependency is introduced.
