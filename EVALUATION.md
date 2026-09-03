# Evaluation contract — Rust NYC Pop Quiz

**Status: Stage 3 `tone-architect`, Phase 1. Written 2026-09-03 against
`sequence/USER_STORIES.md` AC-1…AC-100, `DESIGN.md`, and the loved prototype.**
This is what *done* means for the build, and exactly how an agent proves each
criterion without a human in the loop — or, where it cannot, who proves it,
with what, and when. `SPEC.md` says what to build; this file says how it is
judged. `lattice-orchestrator` re-expresses it as the validation plan and
runs the `autonomous` rows itself.

## How to read a row

| Tag | Meaning |
|---|---|
| `autonomous` | A build agent proves it alone, with the harness hook named. Green means proven. |
| `operator-assisted` | Needs a human-supplied input or a manual step. The row says **what** the human supplies and **when**, so terminal validation never arrives unable to prove a headline claim. |
| `external-oracle` | Judged against a real third party — Discord's live API, a venue's wifi, a real room. The row names the oracle. |
| `felt` | Only a human using the thing settles it. Each is scheduled as a **human-use checkpoint** (HC-n below) with the prototype as the reference. **No user-facing milestone is done on green tests alone.** |

Criteria marked `MVP` in the stories are already satisfied by `mvp/tools/`;
the build inherits that code (see brownfield in `BUILDPLAN.md`) and the row
says whether the existing proof carries over or must be re-established.

## The harness

Contract names, bound to real commands in `BUILDPLAN.md` once the stack is
decided in Phase 3. The orchestrator requires the first two.

| Hook | Contract | Budget |
|---|---|---|
| `test` | Hermetic, parallel, no network, no toolchain beyond the repo's own. Every `autonomous` row that says `test` runs here. | **≤ 60 s** wall-clock. A slower default suite is a defect. |
| `test-full` | Everything: `test` + browser-driven checks, the Miri fixture suite, the load and burst tests, the accessibility sweep. | Minutes; CI and pre-merge. |
| `verify <program>` | The pipeline's verifier on one candidate — pinned rustc, N native runs, Miri — emitting a `verified` record. | Per candidate; the Miri wall-clock is measured here (Q-E4). |
| `bank-audit` | The whole bank against AC-23b (generator uniformity, both tails), AC-24, AC-25, AC-26 (the enumerated tells), AC-27, AC-100 (fits the configured room). | Seconds; runs on every build and every bank change. |
| `canary` | The secrecy suite: plant a canary in the answer, explanation, hint, and receipt; assert none of it appears in any pre-reveal payload, on any surface, by any path (AC-47, AC-48, AC-60, AC-61, AC-79, AC-32, AC-58). | Part of `test`. |
| `burst` | 200 synthetic participants; the deadline write burst in isolation (AC-54); then a full segment (AC-52, AC-53, AC-41). | `test-full`; also run against the deployed room before each checkpoint. |
| `a11y` | Keyboard reachability, live-region events, contrast, target size, reduced motion, over every surface and phase (AC-82…AC-86). | `test-full`. |
| `smoke` | The deployed room driven end to end through all seven phases with mock participants — the pre-checkpoint sanity run. | Minutes. |

## Criteria

### A. The pipeline

| ID | Tag | How it is proven |
|---|---|---|
| AC-1 | `autonomous` | Static: the room app's dependency graph contains no generator, verifier, or LLM client module (`test`). Runtime: a room is created and run to release with the pipeline binary absent from the environment (`smoke`). |
| AC-2 | `autonomous` | `test-full`: a generation run is killed mid-candidate; re-run with no cleanup completes; the bank contains no partial record and the room app is unaffected. |
| AC-3 | `autonomous` | `test`: a run asked for *n* questions on topics T at difficulty d yields *n* candidates tagged T and d, or a report naming which of the three it could not honour. |
| AC-4 | `operator-assisted` | `test`: a fixture title + abstract yields candidates each tagged with a named std/core concept. **Whether the connection is real** is judged by the organizer at the first talk-mode batch (HC-2). |
| AC-5 | `autonomous` | `test`: the run report carries per-candidate cost and wall-clock; `ECONOMICS.md`'s guess bands (Q-E3, Q-E4) are replaced from the first real run. |
| AC-6 | `autonomous` `MVP` | `verify`: the record carries `rustc -Vv`, edition, target triple, and the flag set (overflow-checks, debug-assertions, opt-level — the flags the research found unpinned). `test`: a candidate verified under a different pin is rejected as stale. |
| AC-7 | `autonomous` `MVP` | Static: the only writer of the correct-answer field is the verifier's output reader; `test`: a hand-edited answer in a candidate file fails the build's provenance check. |
| AC-8 | `autonomous` `MVP` | `test` with fixtures: a program whose output varies (e.g. `HashMap` iteration) is **rejected**, not flagged; a deterministic one passes at N=5. N stays a configured hypothesis and the rejection count is reported (the data AC-8 asks for). |
| AC-9 | `autonomous` `MVP` | `test` with fixtures: a UB program is rejected; the same program with UB declared as the intended answer is accepted, and only if Stacked Borrows and Tree Borrows agree. |
| AC-10 | `autonomous` `MVP` | `test`: Miri stdout ≠ native stdout rejects. |
| AC-11 | `autonomous` `MVP` | `test`: a candidate declared non-compiling must fail with an error code, recorded (E0502 fixture); one that compiles is rejected. |
| AC-12 | `autonomous` | `test-full`: a fixture program that opens a socket, reads `/etc/passwd`, reads an env var, allocates past the limit, and loops forever — each is contained and reported. Proven on the sandbox actually used, not a stand-in. |
| AC-13 | `autonomous` `MVP` | `test`: the receipt renders from the `verified` record alone with the toolchain absent. |
| AC-87 | `autonomous` | `test`: the receipt text names the tested target triple and says the determinism claim is scoped to it. |
| AC-14 | `autonomous` | `test`: a byte-identical resubmission is rejected. |
| AC-15 | `autonomous` | `test`: fixtures with renamed bindings and reformatting are rejected as normalized duplicates. |
| AC-16 | `autonomous` | `test`: a near-duplicate (above the similarity threshold, below exact) lands in the review queue marked *near-duplicate of q*, neither accepted nor dropped. The threshold is a configured hypothesis. |
| AC-17 | `autonomous` | `test`: history persists across two runs on separate working directories; count and growth are in the run report and the review surface. |
| AC-18 | `autonomous` | `test`: every uniqueness string in the corpus matches *"no exact or normalized duplicate found"* and none says *original*. |
| AC-19 | `autonomous` | `test`: accept / reject-with-reason / edit each round-trip; a rejection reason is readable back by the generator's next run. |
| AC-20 | `felt` | **HC-2.** Source, options, verified answer, explanation (three beats), receipt on one screen — the organizer reviews a real batch. Reference: `prototypes/review-surface.html`. |
| AC-21 | `operator-assisted` | **HC-2.** The client times her own review of the first real batch; the number replaces the 15-minute hypothesis. Nothing else in the arc has ever measured it. |
| AC-22 | `autonomous` | `test`: the host and review surfaces carry the statement that the reviewer has seen the answers. |
| AC-88 | `operator-assisted` | The review surface records organizer-judged difficulty per question; `bank-audit` reports drift from requested. **HC-2** supplies the first sample; drift beyond one level fails the generation run, not the question. |
| AC-23 | `autonomous` `MVP` | `test`: `slot_for_day` (or its successor) is a pure function of the date; static: no history file, ledger, or bank state is an input to it. Carried over from `mvp/tools/build_deck.py`. |
| AC-23a | `autonomous` `MVP` | `test`: a 10,000-night simulation with a perfect-memory attendee scores within noise of 20%; static lint: no code path reads prior positions. **Any PR touching this path carries the audit ticket** (`SPEC.md` guardrail G-1). |
| AC-23b | `autonomous` `MVP` | `bank-audit`: 20,000 synthetic draws, chi-square df=4, both tails, verified to fail on a skewed generator and on a re-balanced one. Carried over. |
| AC-24 | `autonomous` `MVP` | `bank-audit`: every question has five options and one is *does not compile*. |
| AC-25 | `autonomous` | `bank-audit` + `test`: no participant-facing artifact (wall, buzzer, take-it-home, README) contains an answer-category distribution; the tell set is documented only in organizer-facing files. |
| AC-26 | `autonomous` | `bank-audit`: each enumerated tell — `unsafe`, source length, option text length, option position, topic — measured against the accumulated bank; fails above the stated margin. The margin is in `SPEC.md`. |
| AC-27 | `autonomous` | `bank-audit`: if any accepted question's answer is UB, at least one non-UB accepted question contains `unsafe`. |
| AC-71 | `autonomous` | `test`: the receipt's text states verification covers the answer and not the explanation, on every surface that renders it. |
| AC-72 | `autonomous` | `test`: scheduling an unaffirmed question is refused; affirmation records who and when; the refusal is a hard error, not a warning. |
| AC-73 | `autonomous` | `test`: any output quoted in the explanation is checked against the verified output; a mismatch blocks acceptance (fixture with a wrong quote). |
| AC-74 | `autonomous` + `felt` | `test`: machine-established and human-reviewed content carry distinct provenance markup on wall, host phone, take-it-home. **HC-1** confirms it reads as two things. |
| AC-75 | `autonomous` | `test`: the reserve (verified, affirmed, unused) count and its trend are on the organizer's first screen. |
| AC-76 | `autonomous` | `test`: dropping below threshold raises a warning with the configured lead time (default: two meetups). |
| AC-77 | `autonomous` | `test-full`: with outbound network blocked except the room's own host, a segment runs from the bank end to end. |

### B. The live room

| ID | Tag | How it is proven |
|---|---|---|
| AC-28 | `autonomous` | `test`: join succeeds via the short link (code carried) and via the six-character code typed; the join form has no other field. |
| AC-29 | `autonomous` | `test`: six distinct failure states each show their own message and next step: malformed, unknown, not yet open, already ended, closed for inactivity, full. |
| AC-30 | `autonomous` | `test`: the 201st join is refused before any session is created; storage shows nothing reserved. |
| AC-31 | `felt` | **HC-1 and HC-4** on venue wifi. Instrumented: join-to-lobby time is logged client-side and reported. |
| AC-32 | `autonomous` | `canary`: the source text and the trace never appear in any participant payload, HTML, or state, in any phase. |
| AC-33 | `autonomous` | Wall: `test-full` measures the well after layout — zero overflow wherever the state says *fits* (AC-100). Review and take-it-home: browser test at 375px asserts no horizontal document scroll with the widest bank question. |
| AC-34 | `autonomous` | `test`: an answer changed five times before close records the last; a change after close is refused. |
| AC-35 | `autonomous` | `test`: the buzzer renders exactly one of saving / saved / failed at all times while a question is live. |
| AC-36 | `autonomous` | `test`: a failed write shows the last saved answer as safe and a retry; the previous answer is intact on the server. |
| AC-37 | `autonomous` | `test-full`: the socket is dropped and re-attached; the saved answer survives; controls show paused until fresh state arrives. |
| AC-38 | `felt` | **HC-1** from the measured back row; **HC-4** on the buzzer in a full room. |
| AC-39 | `autonomous` | `test`: at reveal the wall carries the ✓, the totals, the named most-chosen incorrect option with its count, the receipt, and **no explanation text**; the host phone carries the three-beat script; the buzzer carries the correct letter and the participant's count. |
| AC-40 | `autonomous` `MVP` | `test`: the correct option carries a ✓ glyph as well as its colour, on every surface. |
| AC-41 | `autonomous` + `external-oracle` | `burst`: reveal reaches 200 synthetic clients ≤ 2 s at p95. Venue wifi is the oracle at **HC-4**. |
| AC-42 | `felt` | **HC-2**: the client reads each new explanation aloud at review. **HC-1**: the host reads q3's aloud to the room. |
| AC-43 | `felt` + `autonomous` | `test`: receipt strings match the approved wording (*on the executed paths*, *N byte-identical runs on ‹triple›*). **HC-2** judges it as not overstated. |
| AC-44 | `felt` | **HC-4.** One beginner attendee, asked after the reveal. Instrument: `mvp/FIELD-NOTES-TEMPLATE.md`. The criterion that matters most, and it cannot be run before October. |
| AC-45 | `autonomous` | `test`: the host control exposes exactly the seven transitions; no transition fires without a host action; a phase cannot be skipped. |
| AC-46 | `autonomous` | `test`: present and answered counts on the host phone update while live. |
| AC-47 | `autonomous` | `canary`: the host's pre-reveal payloads carry no answer. |
| AC-48 | `autonomous` | `test`: the hint is fetchable by any participant while live; `canary`: no per-participant hint record exists server-side, and no host or wall payload changes when a hint is taken. |
| AC-49 | `autonomous` | `test`: every host screen has a phase label, one primary action, and the code where returning is possible. |
| AC-50 | `autonomous` | `test-full`: the host refreshes, closes the tab, and opens a second device; control resumes on the same room via the host session. |
| AC-51 | `felt` | **HC-1** (first time anyone watches the host), **HC-4**. Anything apologised for is logged. |
| AC-52 | `autonomous` | `burst`: 200 synthetic participants complete a segment; each session's final answer is counted exactly once. |
| AC-53 | `autonomous` | `burst`: answer-write p95 < 500 ms including the deadline burst. Run against the deployed substrate, not a local mock. |
| AC-54 | `autonomous` | `burst`: the deadline burst is its own test — 200 writes inside a 2 s window — reported separately from the full run. **This is the one real engineering risk and it is proven before the walking-skeleton checkpoint** (spike ticket in `BUILDPLAN.md`). |
| AC-55 | `external-oracle` | Venue wifi at **HC-4**; the client reports failed-request rate from the room's own logs. Synthetic runs cannot prove it. |
| AC-56 | `autonomous` | `test`: after release, storage holds only per-option totals with an expiry; static: the schema has no per-participant answer table. |
| AC-57 | `autonomous` | Static + `test`: no participant identity, nickname, score, or cross-room key exists in schema, API, or client state. |
| AC-58 | `autonomous` | `canary`: the buzzer's *n people said A, including you* is computed client-side; no request carries the participant's answer after close. |
| AC-59 | `felt` | **HC-1, HC-3.** The released buzzer's *nothing about you was recorded* read by a participant. |
| AC-60 | `autonomous` | `canary` over every pre-reveal path: HTML, JS state, API, socket frames. |
| AC-61 | `autonomous` | `test`: the public state query is typed such that answer storage is unreachable from it — a compile-time or module-boundary proof, not a convention. |
| AC-62 | `autonomous` | Documented in `SPEC.md`; `test` asserts the statement is present in the organizer docs. |
| AC-63 | `autonomous` | Same as AC-62. |
| AC-78 | `felt` | **HC-1** from the measured back row, at the measured screen size, with the real projector. |
| AC-99 | `autonomous` | `test`: coloured token spans are present on the wall in phases 1–4 and 7 and **zero** in 5 and 6. |
| AC-79 | `autonomous` | `canary` on wall payloads; `test`: the wall has no interactive control. |
| AC-80 | `felt` (answered) | Light is the default and the only presentation; `test` asserts no dark mode ships. Re-opened only if a real room contradicts T-16. |
| AC-81 | `autonomous` | `test-full`: the wall and 200 buzzers report the same phase within one broadcast of every transition. |
| AC-100 | `autonomous` | `test-full`: for every bank question and the configured room, the wall's measured overflow is zero where it reports *fits*, and the clipped edge renders where it does not; `bank-audit` flags questions that do not fit at the floor. |
| AC-82 | `autonomous` | `a11y`: every control reachable and operable by keyboard with a visible focus ring. |
| AC-83 | `autonomous` | `a11y`: each listed state change emits a polite live-region announcement. |
| AC-84 | `autonomous` | `a11y`: AA contrast over every surface and phase. |
| AC-85 | `autonomous` | `a11y`: touch targets ≥ 44 px on the buzzer and host phone. |
| AC-86 | `autonomous` | `a11y`: under `prefers-reduced-motion` no animation runs and every state is still legible. |
| AC-89 | `autonomous` + `operator-assisted` | `test`: a room holds exactly one question and has no *next question* path. Timing: **HC-1** clocks it; **HC-4** clocks it in a full room. |
| AC-90 | `operator-assisted` | The host script says *last*; the client confirms the October run-of-show. |
| AC-91 | `felt` | **HC-4**: watched when hands start going up. HC-1 gives a lower bound only. |
| AC-92 | `autonomous` | `test`: the used-question record is written at **release**, not at build (the AC-92 defect); building a deck writes nothing; a room that never reaches release records nothing. |
| AC-93 | `autonomous` | `test`: the phase machine has no transition to reveal except from the walk-through, which is reachable only from the split. |
| AC-94 | `autonomous` | `test`: no participant surface renders ✗, a red mark, or *wrong* against the participant's own choice, in any phase. |
| AC-95 | `autonomous` + `operator-assisted` | `test`: acceptance refuses an explanation whose middle beat is empty or names no option. The wall names the room's **actual** most-chosen incorrect option with its count. **HC-2**: the organizer affirms the beat. |
| AC-96 | `felt` | **HC-4**, same instrument as AC-44. |
| AC-97 | `autonomous` | `test`: the walk-through phase exists between split and reveal; `canary`: during it the wall carries no ✓, no receipt, no colour, and the trace is host-stepped. |
| AC-98 | `autonomous` + `felt` | `test`: a forbidden-copy lint over every participant-facing string (*turn to*, *ask someone*, *volunteer*, *who said*, *why?* as an address to the room); no UI counts or waits for a contribution. **HC-1** watches the host's mouth. |

### C. Authorization

| ID | Tag | How it is proven |
|---|---|---|
| AC-64 | `autonomous` + `external-oracle` | `test` against a Discord mock: member with the role hosts, member without is denied. **Live**: the client's own account against the real guild before the first deployed checkpoint. |
| AC-65 | `autonomous` | `test`: an Administrator whose `roles` lacks the ID is denied; the check reads `roles`, never `permissions`. |
| AC-66 | `autonomous` | `test`: each refresh persists the rotated token; a replayed old token is detected and does not lock the organizer out of an open room. |
| AC-67 | `autonomous` | Static: no participant route touches the auth module; `test`: no participant request carries a credential. |
| AC-68 | `autonomous` | `test`: organizer B cannot read or control organizer A's room. |
| AC-69 | `autonomous` | `test-full`: with the Discord mock down, an open room runs to release; a new room cannot be created; a room dies at 4 h. |
| AC-70 | `autonomous` | `test`: denial names wrong server or wrong role, and never says whether the user is in the guild. |

## Human-use checkpoints

Each has a trigger, a driver, an instrument, and the criteria it settles. The
prototype is the reference at every one.

| HC | Trigger | Who | Instrument | Settles |
|---|---|---|---|---|
| **HC-0 · Walking skeleton** | Wall + buzzer + host phone deployed on the chosen substrate, mock data, all seven phases, `burst` green | The client, at a desk, against `prototypes/C-projector-first.html` side by side | The T-20 guide, re-run | Fidelity to the loved take; AC-45, AC-49, AC-74, AC-93/94/97 as *felt*; anything that drifted from the prototype is a defect |
| **HC-1 · The rehearsal (T-18)** | Two regulars, the client, a projector — the venue if at all possible. The built room if HC-0 has passed, else the prototype | The client | `mvp/PRACTICE-RUN.md` | The two room measurements → AC-100's configuration; AC-78, AC-38, AC-31, AC-51, AC-59, AC-89 timing, AC-91 lower bound, AC-98 host behaviour, AC-42 aloud |
| **HC-2 · First real batch** | The pipeline produces its first LLM-generated batch | The client, alone | The review surface; a stopwatch | AC-20, AC-21, AC-88, AC-4, AC-42, AC-43, AC-95 affirmation, AC-72's first affirmations |
| **HC-3 · Take it home** | The released wall's link resolves to the real page | The client, on a phone, not at a desk | The page | AC-33 on a phone, AC-59, the second repetition's pace |
| **HC-4 · October** | The meetup | The client hosting; a co-organizer holding the notes | `mvp/FIELD-NOTES-TEMPLATE.md` | AC-44, AC-96, AC-91, AC-31, AC-51, AC-55, AC-89, the participation rate, AC-92's first real record |

## What the operator supplies, and when

| Input | Needed before | Default if absent |
|---|---|---|
| Discord application id, client secret, guild id, organizer role id | Story C1 tickets dispatch | None — C1 cannot be built against guesses. |
| The three room measurements (screen width, screen height, back-row distance) | HC-1 sets them; AC-100 needs them configured before HC-4 | 15 ft / 16:9 / 20 ft, labelled hypothesis |
| The short-link domain behind *join @* | AC-28's link path; before HC-0 | A placeholder link that carries the code |
| An LLM API key and a spend cap | Story A1 tickets dispatch | None; the pipeline can be built and tested on fixtures |
| Q-E1 — is ~$250/yr on the table | Phase 3's substrate decision | Assume no; single-digit dollars is the target |
| The first batch's affirmations (AC-72) | Any deployed room with a real question | The verified MVP bank, already affirmed by use |
| Cole's view on colour (AC-99) | Never blocking | AC-99 stands |

## External oracles

- **Discord.** Member-lookup shape, role-removal latency, and whether
  `guilds.members.read` needs approval are unverified; the first live check is
  against the client's real account and guild.
- **The toolchain.** rustc and Miri at the pinned versions; the sandbox actually
  chosen. AC-12 is proven on it, not on a stand-in.
- **The room.** Venue wifi, the projector, and 200 phones — HC-4 only.

## The `felt` list, in one place

AC-20, AC-21, AC-31, AC-38, AC-42, AC-43, AC-44, AC-51, AC-59, AC-74, AC-78,
AC-80 (answered), AC-91, AC-96, AC-98. Fifteen. Every one is scheduled above.
None may be converted into a unit test and closed.
