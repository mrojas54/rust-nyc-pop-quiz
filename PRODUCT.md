# Product — Rust NYC Pop Quiz

This is a descriptive reading guide and dated source inventory, not an additional
contract or completion ledger. `PHILOSOPHY.md` governs purpose; `SPEC.md`,
`EVALUATION.md`, and `BUILDPLAN.md` define behavior, acceptance, and build work.

## Platform

web

The room display, participant buzzer, host controls, and take-home page are web
surfaces. The organizer's offline pipeline runs locally; mobile web remains web.

## Users

- **Attendees:** Rust NYC meetup participants, including newcomers, who read a
  program on the projector, vote privately on their phone, and work through the
  explanation together. Participation never obliges anyone to speak.
- **Hosts:** organizers who lead one question, control the room's pace, and read
  the explanation aloud. Hosts receive no answer preview.
- **Question organizers:** people who prepare and verify candidates, review
  explanations and traces, affirm questions, maintain reserve, and schedule the
  segment. These are jobs; one person may perform several of them.
- **Take-home readers:** attendees revisiting the last question at their own pace.

## Product Purpose

The quiz is a pretext. The understanding is the product, and the room gets there
by working through it together. Success means a beginner can explain the solution
to someone else (AC-44), with room for the conversation to continue afterward.

Run one fresh Rust program-output question in the last 3–5 minutes of the meetup.
The previous public question bank became familiar enough that the segment was
abandoned. New supply must preserve the need to read and reason, rather than
replace memorized questions with learnable answer shortcuts.

## Positioning

Two decoupled mechanisms support the segment:

1. **Offline supply:** generate candidates in batches, verify them with the pinned
   Rust toolchain and Miri, check duplicates and structural tells, obtain organizer
   affirmation, and keep a private reserve. Freshness is a supply problem.
2. **Live room:** join by link or code, read the projector, answer anonymously,
   see the room's split, follow a host-paced trace, reveal, and release the room.

The answer comes from executing the program. The anonymous split precedes the
answer so attendees see that others made the same reading before the reveal.
This is a volunteer-run community segment, with no revenue model, scoreboard,
certification, or participant learning-history product.

## Operating Context

The source and live trace are on the projector. Participant phones carry voting,
a private hint, and phase feedback. The host phone carries phase controls,
trace stepping, and the reveal script. Take-it-home supports independent reading
and trace stepping after the segment, without live room counts.

The phase order is title/idle → live → closed → split → work → reveal → released.
The host sets the pace. The reveal script explains what happened, why the room's
most-chosen incorrect option was a reasonable reading, and what to remember.

The live implementation uses Rust (`axum` and `tokio`) with Fly.io as the chosen
host. Frontends use plain HTML/CSS/JavaScript and vendored brand fonts. The local
pipeline uses Python and `uv`, JSON question records, and a Docker verification
sandbox. These describe repository implementation and architectural decisions;
they do not certify the currently deployed service.

A scheduled question must also have a local single-file wall fallback and a
plain-text host sheet so the segment can proceed without the room service.
Question generation is never required on meetup day.

## Capabilities and Constraints

### Durable contract

- One question per meetup, scheduled last. Building a deck does not mean the
  segment ran and must not retire a question.
- Correct answers derive from machine-written verification records, never a
  manually authored answer field. Preserve recorded provenance and missing
  legacy facts; do not back-fill observations that were never made.
- Miri checks executed paths. Repeated identical runs are evidence of
  determinism, not a universal guarantee.
- No published answer-category distribution or history-based answer balancing.
  Correct-answer position depends on the date alone. Audits examine the generator
  without changing tonight's position.
- No participant accounts, names, scores, leaderboard, or lasting per-person
  record. Ephemeral sessions support voting; anonymous totals expire with the
  room. Organizer authentication is a separate boundary.
- Source and trace are absent from live participant payloads. Answer, receipt,
  explanation, and resolving trace information obey the phase-specific secrecy
  boundaries in `SPEC.md`; the host receives no preview.
- Hints are requested privately; the host is not told who needed one. A person's
  choice receives a company count, never an incorrect marker.
- Organizer hosting is authorized by the configured Discord role ID for the
  selected club (D-26, `docs/adr/0001-several-clubs-on-one-room-server.md`, `room/src/club.rs`).
  Administrator status alone cannot bypass that club role check.
- Scheduling requires organizer affirmation. A successful verifier run alone
  does not approve explanation quality or make a question ready for the room.

### Implementation snapshot — rechecked 2026-10-11

This inventory is based on source inspected in this checkout. It is not a live
Lattice board, a fresh test result, or a deployment/readiness certification.
Refresh it against source and accepted work before treating it as current status.

| Area | Implemented source | Remaining work or boundary |
|---|---|---|
| Live segment | Phase machine, sessions, answer updates, WebSockets, wall, buzzer, host controls | Human walkthrough and deployed/venue checks remain separate gates |
| Organizer access | Discord OAuth, per-club organizer-role checks, token refresh handling | Real account and environment validation cannot be inferred from source |
| Release and follow-up | Used-question ledger and take-home snapshot; take-home web surface | Verify real release, durable ledger consistency, and reader experience |
| Offline fallback | One-file wall builder, keyboard driver, host-sheet generation | Scheduling integration exists; offline operator walkthrough remains separate evidence |
| Verification | Bank schema, migration, pinned sandbox runner, verifier, receipts | Complete reviewed supply workflow; preserve legacy provenance |
| Freshness and fit | Dedupe, date-only slot mechanism, bank audit and fit logic | Dedupe documents unresolved false-match findings; do not claim the normalized-AST contract is satisfied |
| Generation | `pipeline/src/popquiz/generate.py` implements batch generation and candidate cost reporting | Real provider runs and human review throughput require separate evidence |
| Organizer review | `pipeline/src/popquiz/review.py` is an empty scaffold | T-18 accept/reject/edit/reverify and recorded affirmation |
| Scheduling and sync | `schedule.py` publishes affirmed questions, builds fallback and host sheet, syncs usage, and reports reserve through the authenticated admin channel | Real scheduling, sync, and offline operator validation require separate evidence |
| Operational finish | Test harnesses, copy checks, and `docs/RUNBOOK.md` exist | Current guardrail and human checkpoint evidence remains separate |

### Contract and remaining evidence

`BUILDPLAN.md`, `SPEC.md`, and `EVALUATION.md` define the build contract, not a
completion ledger. T-16 generation and T-20 scheduling/sync have implementations;
they are not unbuilt work. `review.py` remains the T-18 scaffold. Source presence
alone does not establish an accepted end-to-end organizer workflow. Check current
validation evidence for dedupe and guardrails, and use `docs/RUNBOOK.md` for the
implemented schedule/host/sync workflow.

Readiness includes rehearsal with actual projector dimensions and back-row
distance, a timed review of the first batch, take-home reading on a personal
phone, deployed burst/smoke checks, and a real meetup with field notes. Test
success, source presence, and a built fallback each establish narrower facts.

### Open operational evidence

The rehearsal measurements, review throughput, first-look reading time,
participation rate, live ledger consistency, available affirmed reserve, and
human checkpoint outcomes require current evidence. No numeric success claim or
completed checkpoint is established by this product record. The generator's
near-duplicate threshold needs calibration against real batches.

## Brand Commitments

Use the Rust NYC identity and the existing binding `DESIGN.md` and converged
projector-first prototypes. Existing visual decisions are preserved; this record
adds no palette, typography, or surface design.

Write copy to be read aloud by a host, in short concrete sentences. Speak to the
person who chose an incorrect option, never about them. Use counts rather than
personal verdicts. Preserve the established lines “Time for a pop quiz,” “Let's
walk it,” and “Let's go to the bar.” Never congratulate winners or require an
attendee to explain their choice publicly.

The organizer carries operating cost, as recorded in BUILDPLAN's T-21 client
answer. Historical economics estimates are not current prices or budgets.

## Evidence on Hand

- `PHILOSOPHY.md`: purpose, participant social contract, freshness, verification
  limits, and durable principles. Later client amendments supersede old wording.
- `sequence/USER_STORIES.md`: stable acceptance-criteria IDs and amendment history.
- `sequence/run-state.md`: Tone arc decisions and human touchpoints; historical
  entries are not a live completion board.
- `SPEC.md`, `BUILDPLAN.md`, `EVALUATION.md`: behavior, build work, and acceptance.
- `DESIGN.md`, `prototypes/C-projector-first.html`, and
  `prototypes/take-it-home.html`: confirmed visual direction and comparisons.
- `room/src/`, `web/`, and `pipeline/src/popquiz/`: implementation evidence.
- `bank/questions/`, `bank/schema/`, and `bank/fixtures/`: question records and
  verification/receipt fixtures. Record presence does not establish affirmation.
- `docs/demo/`: saved local-run screenshots referenced by README, not proof of
  current deployment or venue legibility.
- `mvp/PRACTICE-RUN.md` and `mvp/FIELD-NOTES-TEMPLATE.md`: rehearsal and field
  evidence instruments. The hand-run `mvp/` deck is retired; the scheduled single-file wall is the fallback.

Where `PRD.md` conflicts with the research synthesis and later client decisions,
use the later contract. The synthesis itself retains historical wording; the
amended purpose in `PHILOSOPHY.md` governs. Do not invent testimonials, attendance
results, question approvals, or successful deployments from these artifacts.

## Product Principles

1. **Understanding together is success.** One question leaves time and a place
   for the explanation to continue beyond the segment.
2. **Freshness includes the mechanism.** Avoid remembered questions and
   learnable structural shortcuts; never enforce historical answer balance.
3. **The machine establishes the answer; people affirm the teaching.** Separate
   executed evidence from explanation quality and preserve both provenances.
4. **Being wrong is ordinary.** Show the split before the answer, protect
   anonymity, make help private, and never oblige someone to speak.
5. **Prefer the smaller mechanism that survives the room.** Prepare supply
   offline, measure real use, and retain a runnable offline fallback.

## Accessibility & Inclusion

Newcomers are part of the intended audience. A quiet attendee can participate
fully without speaking, exposing their answer, or announcing a need for help.

The contract requires keyboard access, polite phase and hint announcements,
WCAG AA contrast for text and essential UI, touch targets of at least 44px,
reduced-motion support, and signals that do not depend on colour alone.

Code must not cause horizontal page scrolling. Personal-reading surfaces may
use an internal code scroll container; the projector must measure fit and
legibility against the room, visibly report clipping, and retain a readable
floor. Venue measurements are evidence to collect, not constants to invent.
