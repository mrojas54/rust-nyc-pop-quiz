# User stories — Rust NYC Pop Quiz

Acceptance criteria are minted here with stable IDs and carried **unchanged**
through prototype → evaluation → spec → tickets → validation. A dropped
criterion should be mechanically visible.

**Shape:** fork B (Phase 2) — an **offline question pipeline** and a **live room
app**, decoupled. Stories are grouped by which they belong to.

**`felt`** marks a criterion that can only be validated by a human using the
thing — how it looks, how it reads aloud, whether a seam makes sense in use.
`tone-prototype` validates these; `tone-architect` schedules their human-use
checkpoints. They are not automatable and must not be quietly converted into
unit tests.

**Status legend:** `MVP` = already satisfied by the hand-run deck in `mvp/`.

---

## A. The pipeline — an organizer prepares a quiz

### Story A1 — Generate a batch of candidate questions
*As an organizer, I want to generate a batch of fresh Rust questions ahead of the
meetup, so that I am never waiting on a machine while a room watches me.*

| ID | Criterion |
|---|---|
| **AC-1** | Generation runs entirely outside any live room. No room can be blocked on it. |
| **AC-2** | A generation run that fails, times out, or is interrupted can be re-run with no cleanup and no user-visible consequence. |
| **AC-3** | The organizer specifies question count, topics, and difficulty; the run honours all three or reports which it could not. |
| **AC-4** | Talk mode accepts a public talk title and abstract and produces questions connected to a standard-library or core-language concept from it. |
| **AC-5** | A run reports per-candidate cost and wall-clock, so the economics in `ECONOMICS.md` can be replaced with measurements. |

### Story A2 — Verify every question before a human sees it
*As an organizer, I want the machine to establish every answer, so that I never
ship a question whose answer I guessed.*

| ID | Criterion |
|---|---|
| **AC-6** | Every candidate is compiled with a **pinned** rustc and edition, both recorded. `MVP` |
| **AC-7** | The correct answer is read from the program's actual output. No code path allows a hand-written answer to reach a deck. `MVP` |
| **AC-8** | Each candidate is run at least 5 times; any candidate whose output is not byte-identical across all runs is **rejected**, not flagged. `MVP` — *5 is a starting hypothesis, not a derived figure; revisit once rejection data exists.* |
| **AC-9** | Each candidate is run under Miri; a Miri-reported UB finding rejects the candidate unless UB is the intended answer. `MVP` |
| **AC-10** | Miri's output is compared against native output; a mismatch rejects the candidate. `MVP` |
| **AC-11** | A candidate expected not to compile must actually fail, and its error code is recorded. `MVP` |
| **AC-12** | Candidate programs are compiled and executed in a sandbox with no network, no secrets, no host filesystem access, and enforced CPU/memory/time limits. |
| **AC-13** | The recorded verification facts are sufficient to render a receipt without re-running anything. `MVP` |
| **AC-87** | The determinism claim is **scoped to the target triple actually tested**, and the receipt says so. Five identical runs on one architecture is evidence about that architecture. |

### Story A3 — Never repeat a question
*As an organizer, I want confidence that tonight's questions have not been asked
before, because that is the entire problem this project exists to solve.*

| ID | Criterion |
|---|---|
| **AC-14** | An exact duplicate of any previously accepted question is rejected. |
| **AC-15** | An AST-normalized duplicate — same program, renamed bindings or reformatted — is rejected. |
| **AC-16** | Near-duplicates are surfaced to the organizer for a judgement call rather than silently accepted or silently dropped. |
| **AC-17** | The history persists across runs and machines, and its size and growth are visible. |
| **AC-18** | Uniqueness claims are stated as *"no exact or normalized duplicate found"* — never as *"this question is original."* |

### Story A4 — Review before the room
*As an organizer, I want to read the batch before anyone else does, so a
confusing or trivial question never reaches the projector.*

| ID | Criterion |
|---|---|
| **AC-19** | Every candidate can be accepted, rejected, or edited; rejection reasons are recorded and re-usable as pipeline signal. |
| **AC-20** | Review shows the source, all options, the verified answer, the explanation and the receipt on one screen. `felt` |
| **AC-21** | Reviewing a default batch takes under 15 minutes for someone who knows Rust. `felt` — *15 minutes is a target to measure at the first real batch, not a derived figure.* |
| **AC-22** | An organizer who reviews the batch has necessarily seen the answers; the product says so plainly rather than implying host ignorance. |
| **AC-88** | Difficulty is **calibrated, not just requested**: across a sample, organizer-judged difficulty is within one level of the level asked for. A run that drifts is a generation defect, not an acceptable variance. |

### Story A5 — The meta stays unlearnable
*As an organizer, I want no shortcut to exist that lets a regular attendee score
well without reading the code.*

| ID | Criterion |
|---|---|
| **AC-23** | An attendee who remembers the correct-answer position of **every** previous meetup gains **no advantage**. The position is drawn uniformly and independently of all history, so their best guess is 1 in 5 and stays 1 in 5 forever. `MVP` |
| **AC-23a** | No rule may narrow the next position. Balancing, quotas, and *"never the same letter as last time"* are **prohibited** — each one converts memory into a free elimination, and enforced balance makes every fifth meetup fully determined. Observed counts will therefore be uneven and letters will repeat; that is the correct behaviour and must not be "corrected". `MVP` |
| **AC-23b** | The **generator** is audited for uniformity on every build, in **both** tails — too skewed indicates a broken generator, too even indicates that someone reintroduced a constraint. The audit runs against synthetic draws and may never alter or reject a specific night's position; an audit that can change tonight's output is a rule, and rules leak. `MVP` |
| **AC-24** | "does not compile" appears as an option on every question, so its presence signals nothing. `MVP` |
| **AC-25** | No answer-category distribution is published, displayed, or documented in any artifact an attendee could read. |
| **AC-26** | No structural property of a question correlates with its answer. The tested set is **named and enumerated**: presence of `unsafe`, source length, option text length, option position, and topic. Each is measured against the accumulated history and must not exceed chance by a stated margin. The set itself is reviewed as the bank grows — an unlisted tell is a bug in this criterion, not an excuse. |
| **AC-27** | If UB-answer questions are introduced, `unsafe` must also appear in questions whose answer is not UB. |

### Story A6 — The explanation is trustworthy
*As an organizer, I want the explanation I read aloud to be correct, because the
room will believe it — and it sits under a verification badge that did not check
it.*

The machine establishes the **answer**. It does not establish the prose. A wrong
answer gets caught by the room working through it, which is the outcome we want anyway; a
wrong explanation gets believed and repeated. This story exists because the
verification receipt lends the explanation credibility it has not earned.

| ID | Criterion |
|---|---|
| **AC-71** | The explanation is treated as **machine-unverified** throughout. The receipt states that verification establishes the answer and **not** the explanation. |
| **AC-72** | **Blocking gate.** No question reaches a deck until an organizer has read its explanation and affirmed it correct. Who affirmed it and when are recorded. An unaffirmed question cannot be scheduled — this is enforced, not advisory. |
| **AC-73** | Any output the explanation quotes as printed is mechanically checked against the recorded verified output; a mismatch blocks acceptance. |
| **AC-74** | The reveal visually distinguishes machine-established fact (answer, receipt) from human-reviewed prose (explanation, hint), so the badge cannot be read as covering both. |

### Story A7 — Always have questions
*As an organizer, I want a meetup to never depend on a batch run succeeding that
day, because the segment dies the first time it does.*

| ID | Criterion |
|---|---|
| **AC-75** | The bank maintains a **reserve** of verified, reviewed, unused questions. The reserve size and its trend are visible at a glance. |
| **AC-76** | Falling below the reserve threshold warns the organizer with enough lead time to act — not on the day. |
| **AC-77** | A meetup can run entirely from reserve with **zero generation that day**, using no network beyond serving the room. |

---

## B. The live room — running the quiz

### Story B1 — Join without an account
*As an attendee, I want to join from my phone in seconds without signing up,
because the segment is ten minutes long.*

| ID | Criterion |
|---|---|
| **AC-28** | Joining requires only a short room code. No account, email, or nickname. |
| **AC-29** | Every room-code failure states what went wrong and what to do next, distinctly for: malformed, unknown, not yet open, already ended, closed for inactivity, and full. |
| **AC-30** | A room at capacity says so before creating any session, and reserves nothing. |
| **AC-31** | Joining, from tapping the field to seeing the lobby, feels immediate on venue wifi. `felt` |

### Story B2 — Read the code and answer
*As an attendee, I want to read a Rust program comfortably on my phone and commit
to an answer.*

| ID | Criterion |
|---|---|
| **AC-32** | **Neither the source nor its trace is rendered on any participant device.** Both live on the room display only, where AC-38 and AC-78 govern their legibility. A participant device carries the options to vote on. *Rewritten twice on 2026-08-14 — first the highlighting clause (Phase 1c), then the whole premise (A-4, client call). Was: "monospace, syntax-highlighted, legible without pinch-zoom on a 375px-wide phone." See Amendments.* |
| **AC-33** | Code that exceeds the viewport scrolls inside its own container; the page never scrolls horizontally. |
| **AC-34** | An answer can be changed freely until the server closes the question. |
| **AC-35** | Submission state is always visible and unambiguous: saving, saved, or failed. |
| **AC-36** | A failed submission states that the last saved answer is safe and offers a retry; it never silently discards a choice. |
| **AC-37** | A dropped connection re-attaches without losing the saved answer, and controls are visibly paused until state is fresh. |
| **AC-38** | The question is legible and answerable from the back of a room on a projector as well as on a phone. `felt` |

### Story B3 — The reveal
*As an attendee, I want to see the answer, why, and what the room thought — that
is the moment the room starts working it out together.*

| ID | Criterion |
|---|---|
| **AC-39** | Reveal shows the correct option, the explanation, anonymous per-option totals, and the verification receipt. |
| **AC-40** | Correct and incorrect are conveyed by shape or glyph as well as colour, never by colour alone. `MVP` |
| **AC-41** | Reveal reaches connected participants within 2 seconds at p95. |
| **AC-42** | The explanation reads well **aloud** — a host can read it to the room verbatim and be understood. `felt` |
| **AC-43** | The receipt states what verification proves and does not overstate it. `felt` |
| **AC-44** | After a reveal, an attendee who knows only **beginner** Rust can explain the solution to someone else. `felt` — sampled by asking one at a real meetup. If they cannot, the explanation failed, not the attendee. This is the criterion that matters most, and it tests the question and its explanation together. |

### Story B4 — Host the room
*As an organizer, I want to run the segment without fighting the tool in front of
fifty people.*

| ID | Criterion |
|---|---|
| **AC-45** | The host controls the room code, question start, early close, reveal, and advance. *Amended 2026-08-14 (T-13): "an optional published hint" removed — the hint is no longer a host action at all. See AC-48.* |
| **AC-46** | The host sees participant count and answers-in while a question is live. |
| **AC-47** | The host receives no answer preview before reveal. |
| **AC-48** | **The hint is available to every participant, on their own device, for as long as the question is open — and taking it tells nobody.** Not the host, not the room, not the totals. There is no host-published hint and no signal that anyone used one. *Rewritten 2026-08-14 (T-13). Was: "publishing a hint shows the same hint to everyone, at once."* |
| **AC-49** | Every host screen states its phase, one primary action, and the room code where returning is still possible. |
| **AC-50** | The host can recover control of their own room after a refresh, a crash, or a device change. |
| **AC-51** | Running a full segment requires no improvisation or apology to the room. `felt` |

### Story B5 — Capacity
*As an organizer, I want the room to hold everyone who came.*

| ID | Criterion |
|---|---|
| **AC-52** | 200 concurrent participants complete a full quiz with no lost or duplicated per-session response. |
| **AC-53** | Answer writes complete under 500ms at p95, including the burst when a deadline expires. |
| **AC-54** | The deadline write burst is load-tested explicitly and separately — it is the highest-risk moment in the system. |
| **AC-55** | Failed participant requests stay under 0.1% across a full session on venue wifi. |

### Story B6 — Privacy by construction
*As an attendee, I want to be wrong in a room full of colleagues without it being
recorded anywhere.*

| ID | Criterion |
|---|---|
| **AC-56** | No per-person answer record outlives the room. Only anonymous totals persist, and they expire with it. |
| **AC-57** | No accounts, nicknames, scores, leaderboards, or cross-session history exist for participants. |
| **AC-58** | Any personal recap is computed on-device from reveals and never transmitted. |
| **AC-59** | The wrap-up states plainly what was and was not recorded. `felt` |

### Story B7 — Answers stay secret until reveal
*As an organizer, I want the correct answer to be genuinely unavailable before
reveal, not merely hidden.*

| ID | Criterion |
|---|---|
| **AC-60** | No pre-reveal payload — HTML, JS state, API response, poll — contains the correct-option identity, the explanation, an unpublished hint, or verification detail. Proven by canary-secret tests, not by inspection. |
| **AC-61** | The public state-query path is structurally incapable of reading answer storage; enforced by tests, not convention. |
| **AC-62** | Option *text* is acknowledged as public — the correct answer is necessarily one of the visible options. |
| **AC-63** | A Rust-expert host can infer the answer by reading the source. Host ignorance is documented as **not** a security guarantee. |

### Story B8 — The room display
*As an attendee at the back of a dim room, I want to read the program off the
screen at the front, because that is where the room works through it together.*

The projector is not a big phone. It is a distinct surface with its own
constraints, and it is arguably the primary one — the phone is an input device,
the room display is where fifty people look at the same nine lines together.

| ID | Criterion |
|---|---|
| **AC-78** | The room display is a distinct surface with its own layout, legible at 6+ metres. `felt` |
| **AC-79** | It requires no interaction beyond host controls, and never shows the correct answer, the explanation, or an unpublished hint before reveal. |
| **AC-80** | It is readable with the room lights down. This needs either a dim-room mode or an amendment to the brand's *no dark mode in v1* non-goal — **the brand is what gets amended if a dim room wins.** `felt` |
| **AC-81** | The room display and the participant view never disagree about the current phase. |

### Story B9 — Usable by everyone in the room
*As an attendee using a keyboard, a screen reader, or just a phone in bright
light, I want to take part on the same terms as everyone else.*

Restores the standard the existing prototype already met — live regions, a
reduced-motion toggle, colour never carrying meaning alone — which the first
draft of these stories dropped.

| ID | Criterion |
|---|---|
| **AC-82** | Every interactive element is reachable and operable by keyboard, with a visible focus indicator. |
| **AC-83** | Every state change — question open, saving, saved, failed, closed, hint published, revealed — is announced through a polite live region. |
| **AC-84** | Text and essential UI meet WCAG AA contrast, in both the light and the dim-room presentations. |
| **AC-85** | Touch targets are at least 44px. |
| **AC-86** | Motion respects `prefers-reduced-motion`; no animation is required to understand state. |

### Story B10 — One question, at the end, and then the room is released
*As an organizer, I want the segment to be one question in the last five minutes
of the night, so the working-through it starts is never the thing I have to interrupt.*

The segment was specified as a multi-question round because that is what a quiz
is. It does not need to be. The product is the disagreement, and a disagreement
needs somewhere to go — mid-meetup the only place it can go is *cut off*, so the
round was competing with its own output. Placed last, the working-through leaves the
room under its own power.

| ID | Criterion |
|---|---|
| **AC-89** | A segment is **one** question and completes within **five minutes** end to end, including the reveal and the explanation read aloud. A question that cannot be run in five minutes is a defect in the question, not a scheduling problem. |
| **AC-90** | The segment is scheduled **last**. Nothing the room must yield to is scheduled after it, and the wrap-up releases the room rather than moving it on. |
| **AC-91** | An attendee seeing the question for the first time can form an answer within **30 seconds** of it appearing — the whole budget the format allows for reading. `felt` — sampled by watching when hands start going up. Failure here is a question that is too long or a screen that is too small, and it is measurable either way. |
| **AC-92** | Running the segment consumes exactly one question from the bank per meetup, and which question each meetup used is recorded. `MVP` |

### Story B11 — Being wrong is the ordinary thing

*As an attendee who is new to Rust, I want to get it wrong without it costing me
anything, because the room I am in already feels like a place where you are not
allowed to.*

Minted 2026-08-14 during `tone-prototype`, on client testimony: the feedback
Rust NYC gets, surfaced through Women in Rust, is that **everything is very
intimidating and people aren't allowed to be wrong.** Story B6 already protects
the *record*; this story protects the *moment*, and they are not the same thing.
See `PHILOSOPHY.md` §9.

Dimension 6 corroborates the constituency without having named the cause: five
of 41 free-text responses ask unprompted for beginner-friendly content, one
specifically for *"pairs up new people with more experienced engineers"*, and
25–27% of every room has never attended before.

| ID | Criterion |
|---|---|
| **AC-93** | The room's split is shown **before** the correct answer, as its own phase, and the reveal cannot be reached without passing through it. Order is the mechanism, not a preference: a split shown *after* the answer is a scoreboard. |
| **AC-94** | No participant-facing surface marks a participant's **own** answer as incorrect — no ✗, no red, no "you were wrong". The correct option is marked (AC-40 still applies to the *answer*); the participant's own choice is marked only with the number of people who chose the same thing. |
| **AC-95** | The explanation **names the most-chosen incorrect option**, states how many chose it, and says why it is a reasonable reading. This makes the explanation a structured artifact — *what happened* / *why the popular wrong answer is tempting* / *the bit worth taking away* — rather than one paragraph, and the middle beat cannot be empty. Checked at review; blocking, alongside AC-72. |
| **AC-96** | The explanation's **first beat** is understandable to someone who knows only beginner Rust, without the other two. `felt` — same instrument as AC-44, sampled by asking one attendee at a real meetup. |

**The private hint lives at AC-48, not here.** It was raised as a conflict at
**T-13** and **resolved 2026-08-14 by rewriting AC-48** rather than by minting a
second criterion that would have said the same thing in different words. AC-48
now carries this story's requirement: the hint is pullable by anyone, at any
time while the question is open, and taking it tells nobody.

**Consequence for Story B4:** the host loses a control. AC-45 no longer lists a
published hint, because there is nothing left to publish — a hint that is always
available needs no announcement. `PHILOSOPHY.md` §8 (*prefer the smaller
mechanism*) paying out a third time, and this one came from an accessibility
argument rather than an engineering one.

**Consequence for Direction A:** a direction with no participant device cannot
satisfy AC-48 as amended. Its only possible hint is host-announced to the whole
room, which is the exact cost §9 names. Recorded at T-14.

---

## C. Authorization

### Story C1 — Only organizers host
*As the platform owner, I want hosting limited to actual Rust NYC organizers,
without maintaining a second list of people.*

| ID | Criterion |
|---|---|
| **AC-64** | Hosting requires current membership of the configured role in the configured guild. |
| **AC-65** | Authorization checks the role **ID** against `roles`, never a role name and never a computed permissions bitfield — Administrator and guild ownership must not grant access. |
| **AC-66** | Rotated refresh tokens are persisted on every refresh; a stale token must not silently lock an organizer out. |
| **AC-67** | Participants never authenticate. |
| **AC-68** | Only the organizer who created a room can view or control it; no room state leaks to another organizer. |
| **AC-69** | **Authorization is checked when a room is created, not continuously.** A room whose host was authorized at creation runs to completion regardless of provider availability, bounded by a maximum room lifetime of 4 hours. Creating a *new* room always requires a live check. This replaces the PRD's 15-minute grace window, which was shorter than most observed Discord incidents (20 minutes to 3+ hours) and required a degraded partial-service state to be designed, built and tested for no benefit. |
| **AC-70** | A denial says which condition failed — wrong server, wrong role — without leaking membership information. |

---

## Open — needs attendee evidence

Not yet stories. These want the Aug 20 dry run or real attendee interviews
before anyone writes criteria for them.

- Is a timer wanted at all, or does it cut arguments short? The MVP makes it
  optional deliberately, to find out. *Narrowed by Story B10 — inside a
  five-minute closer the timer competes with the working-through for the same 30
  seconds, so the MVP run should try it without.*
- Do phones help or hurt? A show of hands is louder and more talkative, and
  it needs no infrastructure. If the room prefers hands, stories B1/B5 lose most
  of their weight and the project gets much smaller. *One question a night makes
  this cheaper to answer and much more consequential — see the note under B5.*
- Does the verification receipt land as interesting, or as noise? *Sharper now:
  in a three-minute segment the receipt costs 30 of 180 seconds. It has to earn
  them.*

**Closed** — *Right number of questions per night.* Was: "3 is a PRD default, not
a finding." Resolved by the client, 2026-08-12: **one**. See Story B10.

---

## Amendments

Criteria are minted here and **finalized at handoff** to `tone-prototype`; after
that, an ID never changes meaning. Changes made during the Phase 4 review are
recorded below rather than applied silently, because a criterion that quietly
becomes something else is exactly the failure the AC-lineage rule exists to
prevent.

### 2026-08-11 — Phase 4 stories review (T-5)

| ID | Change | Why |
|---|---|---|
| **AC-44** | **Rewritten.** Was *"the reveal makes people talk to each other."* Now: a beginner-Rust attendee can explain the solution to someone else. | The original was unfalsifiable — nobody could definitively fail it, so every downstream stage would carry a criterion it could not discharge. The client's reformulation is observable (ask one attendee), falsifiable, and tests the question and its explanation together. |
| **AC-69** | **Rewritten.** Was a 15-minute grace window during auth-provider outages. Now: authorization is checked at room creation only, with a 4-hour maximum room lifetime. | Dimension 2 found real Discord incidents run 20 minutes to 3+ hours, so 15 minutes bought a degraded partial-service state that had to be designed, built and tested and would not have covered a real outage. Checking at the door preserves the security property exactly and deletes the failure mode. |
| **AC-8** | Clarified — 5 runs labelled a hypothesis. | The figure was asserted, not derived. |
| **AC-21** | Clarified — 15 minutes labelled a target to measure. | Same. |
| **AC-26** | Tightened — the tested set of tells is now named and enumerated. | *"No structural property correlates with its answer"* is unbounded and therefore untestable. Option length was measured after the fact and came back clean (2/8 vs. a 1.6/8 chance baseline) — by luck, not by any check. |

**Added:** Story A6 (AC-71…AC-74, explanation trustworthiness with a blocking
organizer gate), Story A7 (AC-75…AC-77, question reserve), Story B8
(AC-78…AC-81, the room display), Story B9 (AC-82…AC-86, accessibility), plus
AC-87 (determinism scoped to the tested target triple) and AC-88 (difficulty
calibration, restored from the PRD).

**Reviewed and left standing:** Story A2 discharges most of its criteria already
via the MVP. Stories B6 and B7 were found properly hard-edged. Story C1 keeps
Discord as the authorization source per the client's call, with only AC-69
changed.

**Still open, deliberately not yet criteria:** whether the MVP projector deck is
a supported mode or disposable — it currently satisfies criteria without being
specified itself. Decide once the Aug 20 run says whether phones are wanted at
all.

### 2026-08-12 — Segment shape: one question, five minutes, last

Client call: the segment becomes **one question, 3–5 minutes, at the very end of
the meetup**. Recorded here because it changed a criterion that was already
minted, and because it exposed a live defect.

| ID | Change | Why |
|---|---|---|
| **AC-23** | **Rewritten twice, same day.** Was *"balanced across a deck."* Briefly became *"balanced across the meetup series — least-used letter next, never twice consecutively."* Now: **drawn uniformly, independent of all history**, with the property stated in terms of what an attendee can infer. Split into AC-23/23a/23b. | See below — the first rewrite was wrong in a way worth recording. |

**The rewrite that had to be rewritten.** Moving to one question a night stranded
the old within-deck balancer on slot 0, which would have put the answer at **A at
every meetup, forever**. The obvious repair — balance across the series instead,
take the least-used letter, never repeat last month — was **worse**, and it is
the repair anyone would reach for second, so it is recorded here rather than
quietly replaced.

Balance and unpredictability are in direct conflict. Every rule that reads
history constrains the next answer, and a constraint is information. Enforcing
*"counts stay within one of each other"* means that after four meetups of a cycle
the fifth is **fully determined**:

| meetup in cycle | letters attendee can eliminate | their odds |
|---|---|---|
| 1 | 0 of 5 | 1 in 5 |
| 2 | 1 of 5 | 1 in 4 |
| 3 | 2 of 5 | 1 in 3 |
| 4 | 3 of 5 | 1 in 2 |
| **5** | **4 of 5** | **certain** |

Measured over 20 meetups: a guaranteed answer every fifth night and a **45.7%**
average hit rate against a 20% baseline — strictly worse than the always-A bug it
replaced, and it looked responsible, which is why it survived review long enough
to be written down. Caught by the client reading the sentence describing it.

The fix is to stop being clever: draw uniformly from the date and let the counts
fall where they fall. A perfect-memory attendee then scores at chance (measured:
17.5% across three strategies, baseline 20%) and can eliminate nothing, ever.
`mvp/answer-history.json` was demoted from an input to a write-only record.

The general form, worth carrying forward: **a fairness mechanism that shapes
output is an oracle.** If a check can change what tonight's answer is, an
attendee can run the same check.

**Added:** Story B10 (AC-89…AC-92) — one question, ≤5 minutes, scheduled last,
30-second time-to-opinion, one question consumed per meetup and recorded. Plus
AC-23a and AC-23b, which exist to stop the balanced-ledger repair being
reinvented by someone who notices the letters look lopsided.

**Reviewed and left standing, with their meaning changed by arithmetic rather
than by edit:**

- **Story A7 (AC-75…AC-77), the reserve.** Unchanged as written, but one
  question a night turns the existing 8 verified questions from *one night's
  deck* into *eight meetups of supply* — roughly eight months. The reserve
  criteria get much easier to satisfy and the generation pipeline stops being
  on any critical path. This is `PHILOSOPHY.md` §8 paying out a second time.
- **Story B5 (AC-52…AC-55), capacity.** The 200-participant target is unchanged,
  but the deadline write burst — named in AC-54 as the highest-risk moment in
  the system — now happens **once a night instead of three times**, and the room
  is open for about five minutes rather than fifteen. The risk does not go away;
  it stops being repeated.
- **Story A4 (AC-21), review under 15 minutes.** Still the target for a batch.
  Reviewing what a single meetup needs is now trivially inside it.

---

### 2026-08-14 — Stage 2, `tone-prototype`: being wrong is the ordinary thing

Client testimony, unprompted: *"i want newbies to be able to be wrong. the
feedback we get and it's surfaced in women in rust is: everything is very
intimidating. people aren't allowed to be wrong."*

Primary evidence about the room, of the same class as the T-2 quote that
withdrew Dimension 1's verdict at amendment A-1. **Nothing anywhere in the
corpus mentioned intimidation before this** — Dimension 6 could observe the
beginner *want* (§4b) but had no account of the *cause*.

**Added:** Story B11 (AC-93…AC-96) and `PHILOSOPHY.md` §9.

**Found by building, and fixed:** the first cut of the prototypes did the
opposite of this on every surface that mattered.

| Where | What it did | Now |
|---|---|---|
| Directions B and C | Marked the participant's own choice with a red **✗** | No ✗ exists in the product. The answer is marked; the person is not. |
| Direction B | Went from *answers closed* straight to *reveal* | A split phase sits between them — **AC-93** |
| All three | One intermediate-pitched explanation paragraph | Three beats, the middle one about the popular wrong answer — **AC-95**, **AC-96** |
| Direction A | Is the mechanism, not a variant of it | Recorded honestly in the take; see T-14 |

**Reviewed and left standing, with its weight changed:** **AC-44** (a beginner
can explain the solution to someone else) was already called *the criterion that
matters most*. It is now also the criterion this story is measured by, and
Dimension 6 §4b had already independently arrived at that.

### 2026-08-14 — T-13: AC-48 rewritten, and it takes a host control with it

| ID | Change | Why |
|---|---|---|
| **AC-48** | **Rewritten.** Was *"publishing a hint shows the same hint to everyone, at once."* Now: the hint is available to every participant on their own device while the question is open, and taking it tells nobody. | The old criterion protected the fairness of a competition that does not exist — no score, no leaderboard, nothing recorded. What it cost was real and one-sided: a newcomer who wanted help had to get the host to announce that the room needed help. Against `PHILOSOPHY.md` §9 that trade is indefensible. |
| **AC-45** | **Amended.** *"An optional published hint"* removed from the host's controls. | Not a separate decision — a consequence. A hint that is always available needs no publishing, so the control has nothing left to do. §8 (*prefer the smaller mechanism*) a third time, arrived at from an accessibility argument rather than an engineering one. |

**What this rules out.** A direction with no participant device cannot satisfy
AC-48 as amended — its only possible hint is announced to the whole room. That
is a second, independent strike against Direction A, and it was not visible
until the hint question was asked. Recorded at T-14, where the client's call was
to **keep A in the fan-out as the honest loser** rather than drop it: it costs
nothing to keep, and in October it is a built comparison for *do phones help or
hurt* instead of a hypothetical.

### 2026-08-14 — Phase 1c: the house design system reaches the criteria

The reconcile onto the Rust NYC Design System was applied to the prototypes on
2026-08-14 and **the criteria were not updated with it**, so for a day the corpus
required something the takes deliberately did not do. Caught on resume and
recorded here rather than fixed quietly, because a criterion that disagrees with
the artifact it governs is the failure the AC-lineage rule exists to catch.

| ID | Change | Why |
|---|---|---|
| **AC-32** | **Rewritten.** Was *"monospace, syntax-highlighted, legible without pinch-zoom on a 375px phone."* Now: monospace, **line-numbered**, rendered **without** syntax highlighting, same legibility bar. | The design system's `SourceCode` renders unhighlighted on purpose. `PROJECTOR_SPEC.md` §4.1 is the reason: the trace signals by highlighting the executing lines and dimming outside the focus region, so a syntax palette underneath competes with the only signal the room is meant to follow. Legibility was always the point; highlighting was one implementation of it, and it works against the trace. `PHILOSOPHY.md` §5 amended in the same pass. |

**Not the same problem, still open.** Phase 1 already recorded that **AC-32 has
no surface at all in Direction C** — *legible on a 375px phone* governs nothing
when the phone carries no code. That is a T-12 consequence and is deliberately
left for the direction call; this amendment only fixes the highlighting clause,
which was wrong in every direction.

**Room code, applied at the same time.** The design system's room code is six
characters (`KMT4XW`); the takes still showed the pre-corpus `RUST-4417`. Now six
characters everywhere, from an alphabet with no `O`/`0` or `I`/`1` — a code read
off a projector from six metres back cannot afford an ambiguous glyph. No
criterion changes: **AC-28** says only *"a short room code"* and **AC-49** only
requires it be on screen; both were already satisfied and remain so.

### 2026-08-14 — A-4: the source never appears on a participant device

Client call, during the T-12 drive-through: *"AC-32, should not put it on the
phone, only options to vote on, code and tracing need to be on screen, maybe
explanation and summary."*

| ID | Change | Why |
|---|---|---|
| **AC-32** | **Rewritten — the premise, not the wording.** Was *"source is monospace, legible without pinch-zoom on a 375px-wide phone."* Now: neither the source nor its trace is rendered on any participant device; both live on the room display only, and the device carries the options to vote on. | The criterion asked *how well* source reads on a phone. The client's answer is that it should not be there at all. Precedent for keeping the ID through a reversal is **AC-48** at T-13 — same subject, opposite answer. |

**This was already half-known and could not be resolved by reasoning.** Phase 1
recorded, from building the takes, that *AC-32 has no surface in Direction C* —
a criterion about phone legibility governs nothing when the phone carries no
code. That was logged as a consequence *if* C won. The client has now made it a
requirement independent of which take wins, which is the stronger claim.

**What it does to the fan-out.** The call is incompatible with two of the three
directions, and neither is a close call:

- **Direction B is eliminated.** Its core assumption is that *the phone carries
  the code, the answer and the reveal*. The first clause is what was just
  refused.
- **Direction A is eliminated.** *"Only options to vote on"* presumes a device
  with options on it. A has no participant device at all — which was already
  carrying two recorded strikes (public commitment against §9, and it cannot
  satisfy AC-48 as amended).
- **Direction C is the surviving assumption**, stated by the client in her own
  words rather than picked off a menu.

**Consequences to settle, not yet minted:**

1. **AC-33 loses its phone framing — and this has now been measured.**
   *"Code that exceeds the viewport scrolls inside its own container"* was
   written for a phone, where the reader's thumb is the escape hatch. AC-79 says
   the wall carries no interaction beyond host controls, so on the wall overflow
   is not scrollable — it is **invisible**, and a horizontal scrollbar is the
   prototype reporting success at the exact thing it is failing.
   **Not one question in the verified bank fits the wall at 27px**: `q3` loses
   168px off the right edge, `q7` loses 605px, and `q1` also loses 212px off the
   bottom. Direction C therefore needs a sibling criterion that the source
   **fits** — a bound on question dimensions, not a CSS fix, and so a constraint
   on the generation pipeline. Kin to **AC-88**. Measured 2026-08-14 with the
   *source under test* switcher built on the client's observation that the
   example on screen was not complex enough.
2. **Explanation and summary on the device — open.** The client said *maybe*.
   Direction C as built refuses it outright: the phone reads *"the why is being
   read out now — listen, don't read."* Reversing that line is the open fork.
3. **§9's private pace had nowhere left to live** once the trace went to the wall
   host-stepped. T-15 gave the per-participant stepper to Direction B for a §9
   reason: a newcomer should work through it *at their own pace, with nobody
   watching how many times they went back*. With the trace on the wall only and
   the explanation spoken only, nothing on the participant's side is re-readable
   and that property is gone from the product.

### The take-it-home link — client proposal, 2026-08-14

Raised by the client in response to (3), and it is a better answer than putting
the explanation on the phone: *"can't we also give a link they can go to after,
to trace on their full monitor."*

**Why it beats the phone.** Every option on the table traded §9's private pace
against the room's attention — prose on fifty phones during the reveal is fifty
people not looking at the wall or at each other, in the exact ninety seconds the
room is supposed to be working through it together. The link **moves the private pace off the phone
and out of the segment entirely**: the room stays a room, and the newcomer gets
to step the trace afterwards, on a real screen, as many times as they like, with
nobody watching. It is also the literal form of the one thing — *placed last,
nobody has to stop it; it walks out with them.*

**It is nearly free, for a reason already in the corpus.** `PROJECTOR_SPEC.md` §6
requires the projector **view** be built separately from its **data source** so
the static fallback is `mode: "static"` plus a fixture rather than a second
design. A take-it-home trace page is that same view, with a different data source
and a different hand on the stepper. The separation the project already owes
itself is what makes this cheap.

**Three things it forces a decision on, none of them blocking T-12:**

| # | Question | Why it is not obvious |
|---|---|---|
| 1 | **Does the page carry the room's split?** | *"24 of 58 chose `[1, 2, 3]`"* is the single most §9 fact the segment produces — and **AC-56** says no per-person record outlives the room and totals expire *with* it. A page carrying the split is an aggregate that outlives the room. Either AC-56 is amended deliberately or the page ships question + trace + explanation and no room data. |
| 2 | **Is it public?** | It would be the first public surface this project has. *What this is not: a question bank to publish.* The freshness objection mostly dissolves — **Story A3** never repeats a question, so a spent question costs nothing — but the last quiz died **because its bank was public**, and that deserves a decision rather than a default. |
| 3 | **How is it handed over?** | The phone is already in their hand at release; showing the link there is the cheap answer, and a QR on the wall covers whoever never joined. Participants never authenticate (**AC-67**), so the page is unauthenticated by construction. |

**Client calls, 2026-08-14, on all three:**

1. **No room state.** *"The page is just informational, no room state."* **AC-56
   holds unamended** — the split stays in the room that produced it.
2. **Public, and it rotates.** *"Public yes, it's okay if people go back, it's
   not set for all time, maybe it expires until the next meetup."*
3. **QR code or link.**

**Prototyped:** `prototypes/take-it-home.html`, framed as a desktop browser.

**Two things the build surfaced, one of them a real conflict:**

- **AC-95 cannot be satisfied on this page as written.** It requires the
  explanation to *state how many chose* the popular wrong answer — which is room
  state. `proto.js`'s `explainHtml()` renders *"Why 24 of us said A"* and *"41%
  of the room read it that way"*, so the page needed its own renderer. The beat
  survives the loss because what makes the wrong reading reasonable was never the
  count: it is a fact about the language. **The reading is that AC-95 is scoped to
  the live reveal**, where the count *is* the mechanism — *you are not alone* —
  and that a page read alone days later needs a different form of the same beat.
  **Not yet a criterion.** It needs an amendment to AC-95 or a sibling criterion
  for this surface, and it is `tone-architect`'s to settle.
- **Rotation is the expiry, and it needs no mechanism.** A URL that always means
  *the last meetup's question* — `popquiz.rustnyc.org/last` — gets the client's
  *"expires until the next meetup"* by construction: no expiry job, no archive,
  no dated URLs accumulating, and **only ever one question public at a time**.
  Story A3 never repeats a question, so a spent one costs nothing to show. The
  link never changes, so it can be printed on anything; the page behind it
  rotates. `PHILOSOPHY.md` §8 — *prefer the smaller mechanism* — a fourth time.

### 2026-08-14 — A-5: the one thing changes, and Story B11 gains a phase

Client testimony: *"no, work through it. not argument, understanding through
repetition and knowledge sharing."*

`PHILOSOPHY.md` said **the argument is the product** from Stage 1. It now says
the **understanding** is, reached by working through it together. **An argument
has sides and a winner** — the thing §9 exists to prevent. Full reasoning in
`PHILOSOPHY.md`'s one thing and at **A-5** in `run-state.md`.

**No AC changes meaning**, and that is worth stating plainly: not one criterion
was written in terms of winning an argument, which is some evidence the criteria
were closer to the truth than the thesis sentence above them. **AC-44** —
*a beginner can explain the solution to someone else* — reads as the most
load-bearing criterion in the file under the new wording, not the old one.

**Owed, and deliberately not minted here:** *working through it* is now a built
phase of the product — the room shares readings, then walks the trace together,
before any answer is shown — and nothing in this file requires it. It belongs
under **Story B11** beside AC-93's ordering rule, and it needs the same care
AC-93 got: the order **share → repeat → resolve** is the mechanism, not a
preference. Left for the round that mints it rather than slipped in as a
consequence.

---

*Minted 2026-08-11, Stage 1 Phase 3; reviewed at Phase 4 the same day.
`tone-prototype` is licensed to reopen and extend this file; new criteria take
fresh IDs and existing IDs never change meaning after handoff. Range is now
AC-1 … AC-96 across 19 stories.*
