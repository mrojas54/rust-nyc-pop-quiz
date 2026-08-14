# Run state — Rust NYC Pop Quiz

The resume anchor for the whole Tone arc. Every stage reads this first on invoke.

| | |
|---|---|
| **Stage** | 2 — `tone-prototype`, **opened 2026-08-14**. Stage 1 (`tone-initiation`) **complete 2026-08-13**. |
| **Phase** | **1b — takes reworked on client testimony, awaiting the drive-through.** T-11 answered (all three directions + review surface as one take). Five clickable prototypes in `prototypes/`, all on the real verified batch. **A-3 amendment: `PHILOSOPHY.md` §9 and Story B11 / AC-93…AC-96** — being wrong has to be the ordinary thing. T-12 (pick a direction), T-13 (private hint vs AC-48) and T-14 (does A survive) all open. Nothing converges until the client has *used* them. Carried in from Stage 1, none blocking: T-6 room half (October), AC-92 mechanism defect (→ architect), participation rate unmeasured. |
| **Repo** | `~/rust-nyc-pop-quiz` (canonical) |
| **Branch** | `ai-c11-cc/tone-prototype`, off `main` @ `9d3fff7`. *(Stage 1 ran on `ai-c11-cc/tone-initiation` and three follow-on branches, all merged to `main` via PRs #1–#4.)* |
| **Remote** | `git@github.com:mrojas54/rust-nyc-pop-quiz.git` (private) |
| **Client** | Michelle Rojas (Rust NYC organizer) |
| **Opened** | 2026-08-11 |

## What this run is

Not a fresh commission. A 975-line `PRD.md` and a validated HTML state-explorer
prototype already exist, both produced **outside** the Tone arc. Stage 1's
artifacts did not exist. The client's Phase-0 call was **full re-open**: the
concept is treated as genuinely undecided, including whether to build it at all.
The PRD is one input among several and may be substantially rewritten.

## Prior artifacts inherited

| Artifact | Location | Standing |
|---|---|---|
| `PRD.md` (975 lines) | repo root | **Draft — blocked on feasibility evidence.** Input, not contract. Phase 2 recommends substantial revision. |
| State-explorer prototype | `~/Downloads/Pop Quiz - Rust NYC (offline).html` | 20 named states, both lanes, `RustNYCDesignSystem` component set. **Design work survives the fork** — see synthesis §4. Not yet in-repo. |
| Superseded copy | `~/Documents/rust-nyc-quiz` | Marked `SUPERSEDED.md` 2026-07-25. Do not work there. |
| Riptide workspace copy | `~/.humanlayer/workspaces/set-up-initial-github/rust-nyc-pop-quiz` | Byte-identical PRD. Branch `ai-humanlayer-cc/set-up-initial-github`. |

## Phase 1 — Research: complete

Five dimensions, 1,811 lines, 101 unique cited sources. Dossier in
`sequence/research/`. Fold-back in `sequence/research/00-synthesis.md`.

| # | Dimension | Headline |
|---|---|---|
| 01 | Problem & people *(mandatory)* | `rust-quiz` = 37 questions in ~7 years. No mainstream live-quiz tool can display a 35-line Rust program. **Verdict amended by client testimony** — see below. |
| 02 | Discord seam *(mandatory)* | Simpler than feared: role IDs immutable, no bot needed, Administrator cannot bypass by construction. 15-min grace window undersized vs. real outages (20 min–3 h). Refresh tokens rotate (undocumented). |
| 03 | Economics *(mandatory)* | Sponsorship worth $3–15/yr — **not load-bearing**. Real cost is Val Town Pro ~$252/yr, tail to ~$2,020/yr. |
| 04 | Generation & verification | Feasible but unproven. UB path ≈187s = 62% of the 5-min deadline; ~48.8% of rooms hit it. **`unsafe` in source is a near-perfect tell for UB-targeted questions.** |
| 05 | Realtime substrate | Val Town: needs measurement, lean skeptical. Turso single-writer vs. 200-write deadline burst. Durable Objects is a textbook fit and free at this scale. |
| 06 | Attendees *(added 2026-08-12, T-6)* | Current venue caps at **110 RSVPs**, ~86% show rate — but every event sold out with a waitlist and the group has hit **200 twice**, so that is a venue cap, not a demand ceiling (amendment A-2). **25–27% newcomers every meetup**, 15% core. 41 free-text responses, **none asking for a quiz**; five asking for beginner-friendly content. **Participation rate never measured** — the real unknown behind AC-52/AC-54. |

## Living-artifact amendments

| # | Artifact | Change |
|---|---|---|
| A-1 | `research/01-problem-and-people.md` | Verdict "problem unproven; solution in search of a problem" **withdrawn**. Client testimony — *"attendees had gotten all answers over and over, we stopped doing dtolnay's quiz at all"* — is primary evidence that the problem is observed, acute, and terminal. Amendment recorded in-file; original retained for the record. |
| A-3 | `PHILOSOPHY.md` (new §9), `sequence/USER_STORIES.md` (new Story B11 / AC-93…AC-96) | **Client testimony 2026-08-14, unprompted:** *"i want newbies to be able to be wrong. the feedback we get and it's surfaced in women in rust is: everything is very intimidating. people aren't allowed to be wrong."* Principle 6 protected the **record**; nothing protected the **moment**, and they are not the same thing. Nothing anywhere in the corpus had mentioned intimidation — Dimension 6 §4b could see the beginner *want* but had no account of the *cause*. Same evidence class as A-1. Details in Stage 2 Phase 1b below. |
| A-2 | `research/06-attendees.md` §2, §6 | Finding *"AC-52's 200 is ~2× the observed ceiling"* **withdrawn**. The 110-RSVP figure is a **venue supply cap, not a demand ceiling** — every event sold out with a waitlist. Client testimony: bigger venues in progress, **Jun 26 Summer Social 200 and sold out**. AC-52 stands at 200. Replaced by a sharper open variable: the **participation rate**, never measured. Original reasoning retained in-file. |

## Keystone decisions

| Decision | Resolution | Why |
|---|---|---|
| Build vs. borrow | **Resolved — build.** Borrowing rejected. | Every off-the-shelf tool fails on code display, which is disqualifying for a program-output quiz. |
| Live vs. batch generation | **Resolved — batch.** Client call, T-4. | Freshness is a supply problem, not a runtime problem. Decoupling dissolves 4 of 5 top risks. |
| Participant-facing host | **Leaning Durable Objects** over Val Town | Better fit for one-authoritative-room state, and free at this scale vs. ~$252/yr with an unpassed capacity gate. |
| Generation substrate | Deferred — matters far less once batch | An offline job that fails costs nothing and reruns. |
| Does this get built at all | **Resolved — yes**, as two decoupled components | Problem confirmed by primary evidence. |

## Decisions logged

| # | Decision | Rationale |
|---|---|---|
| D-1 | Home is `~/rust-nyc-pop-quiz` | `SUPERSEDED.md` is explicit; only this copy has the remote. |
| D-2 | Work on `ai-c11-cc/tone-initiation` | Initiation may rewrite the PRD; the existing branch shouldn't absorb that silently. |
| D-3 | Initiation scope = **full re-open** | Client call, Phase 0. |
| D-4 | `ECONOMICS.md` in scope | Client call. Written; conclusion inverted the client's stated risk. |
| D-5 | Attendee interviews in scope | Client has access. Not yet conducted — see T-6. |
| D-6 | Borrow option (C) rejected | Code display is disqualifying across all candidates. Recorded rather than escalated because the evidence is one-sided. |
| D-7 | **Segment = one question, 3–5 min, scheduled last.** Client call, 2026-08-12. | The argument is the product and an argument needs somewhere to go; anywhere but last, finishing on time means interrupting it. Shrinks the build (no navigation, one write-burst, ~5 min room life) and turns the 8 verified questions into ~8 months of supply. Minted Story B10 / AC-89…AC-92 and forced a rewrite of AC-23 — see below. |

## Conflicts

| # | Conflict | Status |
|---|---|---|
| C-1 | PRD's anti-gaming objective vs. its own published answer prior | **Resolved in synthesis §2** — and found to be worse than the PRD knew: `unsafe`-implies-UB is a second, unnoticed tell. Fix is cheap but must be deliberate. |
| C-2 | PRD Draft-blocked on two gates, yet a full spec and prototype were built atop it | **Resolved in synthesis §3** — decoupling removes one gate entirely; the other (live capacity) survives and is the project's one real engineering risk. |

## Touchpoints

| # | Phase | Subject | Status |
|---|---|---|---|
| T-1 | 0 | Initiation scope · client & user access · economics | **Answered** 2026-08-11 |
| T-2 | 0 | Opening interview — origin, history, purpose, timing | **Answered** 2026-08-11 |
| T-3 | 2 | Economic-model dialogue | Folded into T-4 — cost does not decide the fork |
| T-4 | 2 | Hard fork — build as written / build something else / don't build | **Answered** — decouple generation (option B) |
| T-5 | 4 | Stories review — AC-1…AC-88 | **Answered** 2026-08-11 |
| T-6 | 1–2 | Attendee interviews (client has access) | **Records half answered** 2026-08-12 → `research/06-attendees.md`. **Room half slipped to October** — Aug 12 skipped, Aug 20 unavailable, Sept is RustConf → `mvp/FIELD-NOTES-TEMPLATE.md` |
| T-7 | 1b | AC-52's 200-concurrent target — label as hypothesis? | **Answered** 2026-08-12 — **proposal withdrawn**, AC-52 stands at 200. Client testimony: bigger venues being pursued; Jun 26 Summer Social hit 200, sold out. |
| T-8 | 1b | Calendar fork — delegate Aug 20, slip to Oct, or build ahead of evidence | **Answered** 2026-08-13 — **October, client runs it herself.** Delegation declined: *"it's kind of my project and i don't want it taken from me."* |
| T-9 | 1b | Did the Aug 12 segment run without the client? (q3 rollback) | **Answered** 2026-08-13 — **nobody ran it.** q3 rolled back, bank restored to 8. |
| T-10 | 4 | Handoff to `tone-prototype` now vs. hold for October evidence | **Answered** 2026-08-13 — **hand off now.** |
| T-11 | 2·0 | Exploration directions — which core-assumption forks get built, plus review-surface scope and the Discord seam mock | **Answered** 2026-08-14 — **all three directions**; review surface **in scope as one take**, not fanned out. |
| T-12 | 2·1 | Drive the five takes and pick a direction to converge on | **Open** 2026-08-14 |
| T-13 | 2·1 | Private pull-your-own hint vs. **AC-48** (one hint, everyone, at once) | **Open** 2026-08-14 |
| T-14 | 2·1 | Does Direction A stay in the fan-out now that public commitment is the named problem? | **Open** 2026-08-14 |

## Client interview record (T-2)

- **Origin:** *"attendees had gotten all answers over and over, we stopped doing
  dtolnay's quiz at all."* Problem observed, not predicted. Segment abandoned.
- **History:** previously ran a quiz from an existing public Rust bank.
- **Purpose:** all four selected — spark technical discussion, energize the room,
  teach specific concepts, tie into tonight's talk.
- **Timing:** meetups Aug 12 (tomorrow) and Aug 20; September off for RustConf;
  *"you tell me, idc if it has to be october."* → Recommendation put to client at
  T-4: **October** for the built product, with an optional **Aug 20** manual dry
  run to generate real user evidence for the prototype stage.

## Stats

| Phase | Agents | Touchpoints | Wall-clock |
|---|---|---|---|
| 0 | 0 | 2 answered | ~25 min |
| 1 | 5 | — | ~10 min (parallel) |
| 2 | 0 | 1 answered | ~20 min |
| 3 | 0 | — | ~35 min |
| MVP | 0 | — | ~40 min |
| 1b (T-6) | 0 | 1 open (T-7) | ~20 min |

## Phase 3 output

- `PHILOSOPHY.md` — the one thing (*the quiz is a pretext; the argument is the
  product*), 8 principles, taste. Referenced by `CLAUDE.md`.
- `sequence/USER_STORIES.md` — **18 stories, AC-1 … AC-92, 13 marked `felt`** after Phase 4b (17 / AC-1…AC-88 / 12 at Phase 4; 13 / AC-1…AC-70 / 10 as first minted).
- `CLAUDE.md` — references every root artifact and records that `PRD.md` is a
  prior artifact superseded in part by the Phase 2 synthesis.

## MVP shipped alongside (client call, T-2 timing)

`mvp/2026-08-12/pop-quiz-2026-08-12.html` — self-contained projector deck for
the Aug 12 meetup. 8 questions, all machine-verified: pinned rustc 1.96.1,
5 byte-identical native runs each, Miri clean under strict provenance with
output matching native, one confirmed E0502 rejection. No answer written by
hand. Satisfies AC-6…AC-11, AC-13, AC-23, AC-24, AC-40 today.

Two bugs found and fixed by audit during the build, both recorded because they
are exactly the failure mode `PHILOSOPHY.md` §2 exists to prevent:
1. Answer positions shipped a visible B,D,B,D pattern (badly seeded LCG on
   near-identical question ids). Fixed with a hashed, balanced shuffle.
2. Each question shipped 4 options instead of 5 — "does not compile" replaced
   an authored distractor instead of being appended. Fixed, plus three build
   assertions.

## Phase 4 — Stories review (T-5)

Self-review found ten findings; the client resolved all of them. Applied:

| Finding | Resolution |
|---|---|
| **Explanations were never verified** — the machine established the answer, but the LLM-written prose a host reads aloud sat unchecked under a verification badge | New **Story A6** (AC-71…AC-74) with a **blocking** organizer sign-off, a mechanical check that quoted output matches recorded output, and a receipt that states it covers the answer and not the prose |
| **AC-44 was unfalsifiable** (*"the reveal makes people talk to each other"*) | Rewritten to the client's formulation: a **beginner-Rust attendee can explain the solution to someone else** — observable by asking one, and it tests question and explanation together |
| **AC-69's 15-minute grace window** was shorter than real Discord outages (20 min – 3 h) | Rewritten: **authorize at room creation, not continuously**; a room runs to completion once opened, bounded by a 4-hour max lifetime. Deletes the degraded partial-service state entirely. Discord kept as the auth source per client call. |
| No story for the bank running dry | New **Story A7** (AC-75…AC-77) — a visible reserve, early warning, and the ability to run a meetup with zero generation that day |
| Zero accessibility criteria | New **Story B9** (AC-82…AC-86) — keyboard, live regions, AA contrast, 44px targets, reduced motion. Restores what the prototype already did. |
| Room display smuggled in as one `felt` on a phone story | New **Story B8** (AC-78…AC-81). AC-80 notes the brand's *no dark mode in v1* is what gets amended if a dim room wins. |
| Asserted figures and unbounded claims | AC-8 and AC-21 labelled hypotheses; AC-26's tested tells named and enumerated; AC-87 scopes determinism to the tested target triple; AC-88 restores difficulty calibration |

Bookkeeping corrected: run-state had claimed 14 `felt` criteria; the true count
at minting was 10. Two audits came back clean — AC IDs are continuous with no
gaps or duplicates, and the option-length tell measured 2/8 against a 1.6/8
chance baseline (clean, but by luck — nothing checked it, which is why AC-26 now
names it).

## Phase 4b — Segment shape (D-7), 2026-08-12

Client call: **one question, 3–5 minutes, at the very end of the meetup.**

Applying it surfaced a live defect. `answer_slots()` balanced correct-answer
positions *within a deck*; asked for one question it returned slot 0
unconditionally, so **every meetup would have shipped its answer at position A,
forever.** Third answer-position bug in this codebase, and the quietest — the
format change broke a mechanism nobody touched.

**The first fix was worse than the bug — recorded because it is the fix anyone
would write.** Balancing was moved to the meetup series: least-used letter next,
never repeat last month's. Enforced balance means that once four letters in a
cycle are spent the fifth is arithmetic, so an attendee with a memory got a
**certain** answer every fifth meetup and **45.7%** across a cycle against a 20%
baseline. Client caught it from the sentence describing it — *"idk man like
that's gonna be obvious that it's never last time's answer"* — before any test
did.

| | |
|---|---|
| **Fix** | Slot is drawn **uniformly from the date and nothing else**. `slot_for_day()` is pure and takes no history; `mvp/answer-history.json` demoted from input to write-only record. Perfect-memory attendee measured at **17.5%** across three strategies (chance = 20%), eliminating nothing. Counts are deliberately uneven and letters repeat. |
| **Audit** | Now tests the **generator**, not the sequence: 20,000 synthetic draws, chi-square df=4, **both tails** — upper catches a skewed generator, lower catches someone reintroducing balancing. Structurally cannot alter a real night's output. Both failure modes verified to fail the build. |
| **Principle** | *A fairness mechanism that shapes output is an oracle.* Added to `PHILOSOPHY.md` §2. |
| **AC-23** | Rewritten twice the same day; now split **AC-23 / 23a / 23b** and stated in terms of what an attendee can infer. Both rewrites recorded in `USER_STORIES.md` rather than replaced. |
| **Minted** | Story B10 / **AC-89…AC-92** — one question, ≤5 min end to end, scheduled last, 30-second time-to-opinion (`felt`), one question consumed and recorded per meetup. |
| **Closed** | Open question *"right number of questions per night"* — answered: one. |
| **Build** | `build_deck.py --only <qid>` writes `pop-quiz-<day>-<qid>.html`, so a single-question build never clobbers a batch build. Batch build retained for review. |
| **Aug 12 deck** | `q3` (Collections — `vec.dedup()` on non-adjacent duplicates), answer at **E**. Four lines, difficulty 2, and the `[1, 2, 3]` distractor catches everyone who reads `dedup` as `unique`. |

## Phase 1b — T-6 attendee evidence, 2026-08-12

T-6 was the last open item in Stage 1. It splits in two: what the group's own
records can answer without a room, and what only a room can answer.

**Records half — done.** `sequence/research/06-attendees.md`, a sixth dimension.
Four CSV exports aggregated locally; **no PII copied into the repo** (identities
hashed before cross-file matching, free text quoted unattributed).

| Finding | Detail |
|---|---|
| **Room size** | Regular meetups cap at **110 RSVPs** (three events, exactly 110 each, 1–31 waitlisted). Unconf 2025 approved 200 against 294 registrations. |
| **Show rate** | Median **88%**, aggregate **86%** (1,160 attendances / 1,347 yes-RSVPs, n=131 with ≥3 RSVPs). Far above the 50–60% free-meetup norm. Soft: 32 of 131 show attendances > yes-RSVPs, so the fields don't cleanly nest. |
| **Turnover** | **25–27% of each room has never attended a Rust NYC event.** Core (10+ events) is 15%. 226 unique members across the exports. |
| **Voice** | 41 unique free-text responses — the only attendee testimony in the corpus. **Zero mention a quiz, game, or interactive segment.** Five ask unprompted for beginner-friendly content, one for *"pairs up new people with more experienced engineers."* |
| **Experience mix** | Not establishable. Titles measure career seniority, not Rust fluency; 41% are unmarked "engineer". |

**Two corrections to claims made earlier this stage**, recorded because both were
stated before the data was read:

1. The Luma `survey_response_rating` / `survey_response_feedback` columns were
   described as pre-existing testimony. They are **entirely empty** — 0 of 294.
   No survey was ever collected.
2. "Four attendee exports" is **three events, four files** — the two Gametank
   files are the same event exported days apart. Also, `The_perks_and_pitfalls…`
   is a *post-event* export, so its 0% newcomer rate is an artifact of export
   timing, not a fact about the room.

**Room half — instrument built, capture SLIPPED to October.**
`mvp/FIELD-NOTES-TEMPLATE.md` is instrument and capture form in one, targeting
AC-44, AC-89, AC-91, AC-88, AC-38/78/80, AC-51, the participation ratio, and the
three open questions. Written so a **co-organizer can run it cold** — it does not
require the client. A hand-run projector deck with no phones **is** the
hands-only condition Story B5's fate depends on, so the natural experiment costs
nothing to run whenever a meetup happens.

*(Originally `mvp/2026-08-12/field-notes.md`, dated to a meetup that did not
happen for the client. Generalized and undated 2026-08-13.)*

## T-7 — AC-52 amendment proposed and WITHDRAWN, same day

**`AC-52` stands at 200 as written. No amendment.** Raised and retracted
2026-08-12 on client testimony; recorded rather than deleted because the error
is instructive.

**The proposal** was to label `AC-52`'s 200-concurrent target a hypothesis at
"2× the observed ceiling," reasoning from the 110-RSVP cap to a ~90–100-person
room.

**The error** was reading a **venue supply cap as a demand ceiling** — and the
disproof was already inside the same dossier: every event sold out with a
waitlist, 76 turned away at the Unconf alone. Client testimony supplied two
facts no export held: the group is **actively pursuing bigger venues**, and the
**Jun 26 Summer Social drew 200, sold out**. Second time Rust NYC has filled a
200-person room. Sizing the system to a constraint the organizers are actively
removing is backwards.

**Also retracted:** the claim that a ~100-person ceiling weakens the case for the
heavier realtime substrate. Dimension 5 stands unchanged — Durable Objects
remains the fit, and Turso's single-writer behaviour against a 200-write burst
remains a live concern.

**What survives, and it is sharper.** The client's own caveat — *"that was a
party not a meetup"* — is adopted: a sold-out social establishes **draw**, not
participation in a technical segment. `AC-52` counts **concurrent
participants**, and the number that sizes the deadline write burst is
*attendance × participation rate*. **Only the first factor has ever been
measured.** 200 present at 40% participation is 80 concurrent; at 90% it is 180
— a >2× spread in precisely the quantity AC-54 calls the highest-risk moment in
the system.

Field-notes Q2 (*do phones help or hurt*) is now the **only** live line of
evidence bearing on whether Stories B1/B5 shrink; the room-size line is
retracted. The field notes were amended to capture a participation count, making
tonight the first measurement of that ratio.

## Calendar reality, 2026-08-13 — attendee evidence slips to October

Client: **Aug 12 skipped** (did not attend); **Aug 20 unavailable** (Alloy
company boat party); **September is RustConf**, no meetup. **Next meetup is
October.**

**The consequence is a collision, and it is the reason this is recorded rather
than noted.** The T-2/T-4 plan was *October for the built product, with an
optional Aug 20 manual dry run to generate real user evidence for the prototype
stage*. The dry run is gone. **Evidence and ship date now land in the same
month** — the meetup that was supposed to de-risk the build is the meetup the
build was aiming at.

| # | Option | Cost |
|---|---|---|
| 1 | **Delegate Aug 20 to a co-organizer.** The exports show 3–4 organizer-role accounts besides the client, and `FIELD-NOTES-TEMPLATE.md` is written to be run cold by someone else. | Cheapest by far. Saves two months. Needs one ask. |
| 2 | **October meetup becomes the dry run**; built product targets Nov/Dec. | Honest, but slips the build a full quarter. |
| 3 | **Build ahead of evidence**, ship October. | Fastest. Designs Stories B1/B5 without ever testing whether phones are wanted — the exact risk field-notes Q2 exists to retire. |

**Resolved 2026-08-13 (T-8): option 2 — October, and the client runs it
herself.** Delegation was recommended and **declined for a reason that is
accepted without argument**: *"it's kind of my project and i don't want it taken
from me."* Authorship of the segment is not a cost to be optimised away, and the
run records it as a legitimate client call rather than an efficiency loss.

**The collision largely dissolves anyway**, because T-10 sent the arc forward:
`tone-prototype` validates `felt` criteria with the *client* driving clickable
prototypes, not with attendees; only AC-44, AC-91, the participation ratio and
Q2 need a room. Design discovery occupies the two months that would otherwise
have idled, and the October meetup lands as evidence feeding back into a design
that already exists — rather than as the first and last chance to learn anything.

**Carry phones-vs-hands as a prototype direction.** It is a core-assumption fork,
which is exactly what `tone-prototype` fans out on. Making it one of the
exploration directions turns the unanswered question into design input instead
of a blocker, and means October's answer selects between designs already built
rather than sending anyone back to the drawing board.

## Defect — AC-92 records build time, not run time

`build_deck.py:220-234` (`slot_for_meetup`) writes `answer-history.json` from
`main()` **when a deck is built**. AC-92 requires recording which question each
meetup **used**.

**Live consequence — found and repaired 2026-08-13.** The Aug 12 deck was built
and the meetup was skipped, so the ledger claimed **q3 was consumed when nobody
had seen it**. One of only 8 verified questions was wrongly retired — with one
question per meetup, a month of supply. The already-run check at
`build_deck.py:253-256` would also have fired a **false** *"q3 was already run
on 2026-08-12"* warning at the next build, pressuring whoever built it to burn a
second question to avoid a repeat that never happened.

Client confirmed at T-9 that the segment did not run. **The 2026-08-12 entry was
removed by hand and `meetups` is now empty; the bank is back to 8 and q3 is
fresh.** The rollback and its reason are recorded inside
`mvp/answer-history.json` itself. Note that rebuilding a deck for 2026-08-12
would recreate the entry — the defect below is unfixed.

**Not an oracle bug** — the ledger is correctly write-only and does not feed slot
selection; that part works as designed. This is a **provenance** bug: build time
masquerading as run time.

**Mechanism not fixed — carried to `tone-architect`.** Whether consumption is
confirmed at build, at run, or by an explicit "this ran" step is a design
decision, not a patch. It is a live candidate for a new acceptance criterion
under Story B10, since `AC-92` as written is not satisfied by any code that
exists. Data rolled back; mechanism open.

## Stage 1 — COMPLETE 2026-08-13. Handed off to `tone-prototype`.

All touchpoints T-1…T-10 answered. Carried forward, none of them blocking:

- **T-6 room half — October.** The client runs `mvp/FIELD-NOTES-TEMPLATE.md` at
  the October meetup and folds the answers back. `tone-prototype` is licensed to
  reopen `USER_STORIES.md` when they land, which is the expected path for AC-44,
  AC-91, the participation ratio and Q2.
- **Phones-vs-hands → a prototype direction**, not a blocker. See T-8 above.
- **AC-92 mechanism defect** → `tone-architect`. Likely a new criterion under
  Story B10; `AC-92` as written is satisfied by no code that exists.
- **Participation rate** remains the unmeasured variable behind AC-52/AC-54.

*Stage 1 opened 2026-08-11, closed 2026-08-13. Superseded along the way: an
earlier "Remaining" section listed T-5 as open and scoped it to AC-1…AC-70; T-5
was answered 2026-08-11 and Phase 4b took the range to AC-92.*

*(Superseded: an earlier version of this section listed T-5 as remaining and
scoped it to AC-1…AC-70. T-5 was answered 2026-08-11, and Phase 4b took the
range to AC-92.)*

---

# Stage 2 — `tone-prototype`

## Phase 0 — Intake, 2026-08-14

Corpus read cold: `run-state.md`, `PHILOSOPHY.md`, `sequence/USER_STORIES.md`
(18 stories, AC-1…AC-92, 13 `felt`), `CLAUDE.md`, `ECONOMICS.md`, the Phase-2
synthesis, and `PRD.md` §Brand And Accessibility. Inputs are complete; nothing
upstream is missing.

**Prior design asset re-assessed.** `~/Downloads/Pop Quiz - Rust NYC
(offline).html` (869 KB) carries two lanes — Organizer and Participant — across
~20 named states, including the ones nobody remembers to design: *Discord is
unreachable*, *Reconnecting*, *Room closed*, *Hint published to everyone*, *How
the room voted*, plus a reduced-motion toggle and screen-reader announcements.
Its `RustNYCDesignSystem` tokens match the `PRD.md` brand contract exactly
(`--radius:4px`, `--accent-primary:#d69e2e`, 44px targets, 480px container).

Two things it is **not**, both load-bearing for this stage:

1. **It is pre-D-7.** Its vocabulary is a multi-question round — *Next question*,
   *End quiz*, *Quiz complete*, a question count, and *Generate quiz* as a live
   action. The one-question-in-five-minutes segment invalidates that spine.
2. **It has no room-display lane.** Story B8 (AC-78…AC-81) — the surface
   `USER_STORIES.md` calls *arguably the primary one* — was never designed.
   Under D-7 the projector carries the whole segment.

So it is a strong **component and edge-state library**, not a converged take.

**Prototype scope note.** Story A4/A6/A7 (the organizer review surface, AC-20 and
AC-21 both `felt`) is a real design surface with no fan-out axis — it is a tool
for one person, and that person is the client. Carried to T-11 as a scope
question, not a direction.

**Seam mock owed.** Discord is the external system the user also lives in
(Story C1, AC-64…AC-70). Per the stage contract this needs a stand-in of
Discord's own surface plus a one-page seam diagram — what maps to what, what
crosses, what each side sees — not a black box.

## T-11 — exploration directions (open)

Put to the client 2026-08-14. Three candidate directions, each resting on a
different core assumption about **where the segment actually happens**:

| | Direction | Core assumption | What it costs if wrong |
|---|---|---|---|
| **A** | **Hands** — no participant app at all. Projector + a host control. | The room's voice beats its phones; the infrastructure was never the point. | Stories B1/B5/B6 lose most of their weight; the build shrinks to a fraction. |
| **B** | **Phone-first** — the phone carries code, answer, and reveal. | Commitment is private, and anonymity is what makes people willing to be wrong out loud. | Closest to the PRD and the prior prototype — but that prototype is pre-D-7. |
| **C** | **Projector-first, phone as buzzer** — code lives on the big screen, the phone is five letters. | Fifty people reading the *same* nine lines is the event; the phone is an input device, not a reading surface. | Makes AC-91's 30-second budget a projector-typography problem, not a phone-layout one. |

The phones-vs-hands fork was carried here deliberately from T-8: it is a
core-assumption fork, which is what this stage fans out on, and building it as a
direction means October's field notes **select between designs that already
exist** rather than sending the design back to the start.

**Answered 2026-08-14: all three, and the review surface as one take.**

## Phase 1 — the takes, 2026-08-14

`prototypes/`, five clickable HTML takes plus an index. Every one runs on the
**real verified batch** from `mvp/2026-08-12/` — real sources, real distractors,
the real long explanations, real receipt facts, and `q3`'s answer at **E**
exactly as `slot_for_day()` drew it. Room figures come from
`research/06-attendees.md` (110-RSVP cap, 88% median show → 95 present). Nothing
is shortened to flatter a layout; that is the point of prototyping on real data.

| File | Take | Frames |
|---|---|---|
| `A-hands.html` | Direction A — Hands | projector + host phone |
| `B-phone-first.html` | Direction B — Phone-first | participant phone + host phone + projector mirror |
| `C-projector-first.html` | Direction C — Projector-first, phone as buzzer | projector + buzzer phone + host phone |
| `review-surface.html` | Review surface (one take) | desktop browser |
| `seam-discord.html` | The Discord seam | Discord stand-in + our side + SVG seam diagram |
| `index.html` | The takes, side by side | — |

Shared: `_shared/tokens.css` (the `PRD.md` brand contract, inherited not
reinvented), `_shared/data.js` (the real batch), `_shared/proto.js` (Rust
highlighting, the persistent `PROTOTYPE` badge, the AC-83 live region).

**Instruments built into the takes**, so `felt` criteria can be judged rather
than asserted:

- **C carries a back-of-the-room test** — front row / middle / six metres back,
  scaling the whole frame. AC-78 is the criterion this direction lives or dies
  on, and a mock only ever seen at full size passes a test the room never runs.
- **A and C carry a lights-down toggle.** AC-80 says the brand's *no dark mode
  in v1* is what gets amended if a dim room wins, so the amendment is built and
  offered for judgement instead of argued about.
- **The review surface times itself.** AC-21 targets 15 minutes for a batch and
  nobody has ever measured it; the clock in the corner is the measurement.
- **Every host surface shows the participation ratio** — the one variable behind
  AC-52/AC-54 that has never been measured.

**Findings already surfaced by building, ahead of the client's drive-through:**

1. **The prior state-explorer is not a Direction-B head start.** Its spine is a
   multi-question round; D-7 deletes that vocabulary. Its edge-state catalogue
   survives and is reused.
2. **AC-39 does not survive Direction A as written.** *Anonymous per-option
   totals* exist there only because the host types them, and a raised hand was
   never anonymous to the room. If A wins, AC-39 needs amending — recorded in
   the take itself rather than hidden.
3. **AC-32 has no surface in Direction C.** *Legible without pinch-zoom on a
   375px phone* applies to nothing when the phone carries no code. If C wins,
   AC-32 is withdrawn or rewritten against the projector.
4. **The explanation's delivery is a design fork, not a detail.** B puts the
   prose on every phone; C puts it only in the host's hand. Under C, AC-42
   (*reads well aloud*) stops being aspirational — speech is the only delivery.
5. **`q8`'s answer *is* "does not compile",** so appending AC-24's universal
   option would have shipped a five-option question with four real choices.
   Caught in the review surface and fixed there.

## Phase 1b — client testimony reopens the philosophy, 2026-08-14

**The input.** Unprompted, on first sight of the takes: *"i want newbies to be
able to be wrong. the feedback we get and it's surfaced in women in rust is:
everything is very intimidating. people aren't allowed to be wrong."*

**Why it is recorded as an amendment and not a preference.** This is primary
evidence about the room, the same class as the T-2 quote that withdrew Dimension
1's verdict. **Nothing in 1,811 lines of dossier mentions intimidation.**
Dimension 6 §4b found the beginner *want* — five unprompted asks for
beginner-friendly content, one for *"pairs up new people with more experienced
engineers"*, against a 25–27% never-attended-before rate — and correctly tied it
to AC-44. It had no account of the cause. This is the cause.

**What it changed in the philosophy.** Principle 6 (*nothing per-person is
recorded*) protects the **record**. It turns out to be necessary and nowhere
near sufficient: a segment can record nothing at all and still be the most
exposing five minutes of someone's evening. New **§9** protects the **moment**.
The one thing is unchanged and is what the new principle serves — an argument is
the product, and an argument needs people willing to be wrong out loud.

**The asset the format has and nothing else does.** *Majority wrongness,
anonymous, in public.* On `q3`, **24 of 58 chose `[1, 2, 3]`** — 41% of the
room, wrong, together. No host saying it is fine to be wrong does what a room
watching 41% of itself be wrong does. The failure mode to avoid is the one that
looks like kindness: softening questions, or reassuring people.

**The prototypes were doing the opposite, and it took building them to see it.**

| Where | What the first cut did | Now |
|---|---|---|
| B and C | A red **✗** on the participant's own choice | No ✗ exists anywhere in the product |
| B | *answers closed* → *reveal*, no split beat | Split phase between them — **AC-93** |
| A, B, C | One intermediate-pitched paragraph | Three beats; the middle one is the popular wrong answer — **AC-95**, **AC-96** |
| B, C | Hint only publishable by the host to the whole room | A quiet pull-your-own hint, **badged as a proposal** (T-13) |

**Direction A takes a direct hit.** *Hands up for A*, one letter at a time, is
public commitment with your face attached. The newcomer who is unsure raises
nothing, visibly, five times. Every available mitigation — eyes closed, heads
down — removes the public commitment, and the public commitment **is** the
direction. Recorded inside the take itself rather than quietly de-emphasised;
whether it stays in the fan-out is **T-14**, the client's call.

**Minted:** Story B11 / **AC-93…AC-96**. `PHILOSOPHY.md` gains §9 and one taste
line (*the person who got it wrong is being talked to, not about*).

**Not minted, deliberately:** the private hint. It contradicts **AC-48** head-on,
and a new criterion that contradicts a standing one is a fork in the truth rather
than an addition to it. **T-13** resolves it first.

**Spec consequence for `tone-architect`.** AC-95 turns `explanation` from one
string into three fields, and the middle one cannot be empty. That is a
content-model change and a new blocking review gate alongside AC-72, not a copy
change.
