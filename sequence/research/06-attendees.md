# Dimension 6 — The attendees, from the group's own records

**Scope:** T-6. Establish who is actually in the room — size, turnover, show
rate, experience mix — and surface any attendee testimony that already exists,
from Rust NYC's own Meetup and Luma exports. Written *before* any meetup evidence
existed; the part only a room can answer is captured separately in
`mvp/FIELD-NOTES-TEMPLATE.md`, **first runnable in October** (Aug 12 skipped,
Aug 20 unavailable, September is RustConf).

**Method:** four CSV exports supplied by the client, aggregated locally. **No
personally identifying data was copied into this repo** — no names, emails,
phone numbers, member IDs, or profile URLs. Member identity was hashed before
cross-file matching. Free-text responses are quoted unattributed.

**Sources (local, not committed):** three Meetup event exports
(`Rust__Gametank…` ×2, `The_perks_and_pitfalls…`, `Unconf_2025…`) and one Luma
guest export (`Rust NYC Unconf 2025… - Guests - 2025-11-29`).

---

## Summary verdict

> The exports are **better evidence than expected on population and worse than
> expected on voice.** They establish the room's size, turnover, and show rate
> with real numbers. They contain **no survey responses at all** — the Luma
> feedback columns are empty — and no check-in data, so actual attendance is
> still unmeasured. The only attendee testimony is 41 unique free-text answers
> to a Meetup signup question about *event topics*.
>
> On room size this dimension **got it wrong and then corrected it.** Regular
> meetups cap at 110 RSVPs, and §6 originally read that as a demand ceiling
> arguing `AC-52`'s 200 was 2× too large. It is a **venue** cap: every event
> sold out with a waitlist, and on client testimony the group is moving to
> bigger rooms and has already put 200 people in one twice. **The proposed
> amendment to `AC-52` is withdrawn; 200 stands.**
>
> What survives is sharper than what was withdrawn: `AC-52` counts **concurrent
> participants**, and *attendance × participation rate* is the number that sizes
> the deadline write burst. **The participation rate has never been measured**,
> and a 40%-vs-90% spread swings it more than 2×. Tonight's field notes capture
> it for the first time.

---

## 1. Correction to two claims made earlier in this stage

Recorded because both were stated to the client before the data was read, and
one of them shaped how this dimension was scoped.

| Claim | Reality |
|---|---|
| *"`survey_response_rating` / `survey_response_feedback` on the Luma export are pre-existing attendee testimony."* | **False.** Both columns exist and are **entirely empty** — 0 responses of 294 rows. No survey was ever collected. |
| *"Four attendee exports."* | **Three events, four files.** The two `Rust__Gametank…` files are the *same event* exported twice, days apart (141 rows then 137; identical RSVP-window and `Last Attended` profile). Treating them as independent would double-count 110 people. |

A third artifact worth naming: **`The_perks_and_pitfalls…` is a post-event
export.** 110 of its 111 rows show `Last Attended = July 8, 2025` — that event
itself — so its "never attended before" count is **0% by construction**, not by
fact. Newcomer rates below use the pre-event exports only.

---

## 2. How big is the room, really

> **AMENDED 2026-08-12 — client testimony supersedes the inference below.** The
> client, a Rust NYC organizer, states that the group is **actively pursuing
> bigger venues**, and that the **Rust NYC Summer Social on June 26 drew 200 and
> sold out** — adding, unprompted, *"but that was a party not a meetup."*
>
> **The 110 figure is a venue supply cap, not a demand ceiling, and this dossier
> originally read it as the latter.** The disproof was already in the table
> below: every event carried a waitlist — 27–31 at Gametank, **76 at the Unconf
> against 200 approved**. Demand exceeded supply at *every event in the record*.
> Reasoning from "the cap is 110" to "the room is ~100" was invalid, and the
> conclusion drawn from it in §6 is withdrawn there.
>
> No export exists for the Summer Social; it is testimony, which per this
> dossier's standing practice is primary evidence and stronger than the desk
> data it corrects.
>
> **The client's own caveat is the right one and is adopted here.** A sold-out
> social is evidence about the group's *draw*, not about how many people take
> part in a technical segment at the end of a talk. It raises the ceiling on
> room size; it says nothing about participation.

| Event | RSVP cap | Waitlisted | Implied demand |
|---|---|---|---|
| Gametank / Optimization (Aug 5 2025) | **110** | 27–31 | ~137–141 |
| Perks & pitfalls of Rust for codegen (Jul 8 2025) | **110** | 1 | ~111 |
| Unconf 2025 — *"our biggest event yet"* (Oct 29 2025) | **200 approved** | 76 | **294** |
| Summer Social (Jun 26) — *client testimony, no export* | **200** | sold out | ≥200 |

The 110 figure is a **hard cap, not a coincidence** — three separate events each
land on exactly 110 "Yes" RSVPs with the remainder waitlisted. Regular monthly
meetups sell out and turn people away.

**Read that as a statement about venues, not about the audience.** Every event in
the record sold out, and the group has cleared 200 twice (Unconf, Summer Social)
while actively looking for larger rooms. The binding constraint has always been
seats, and it is being removed.

**Show rate — high, with a caveat.** Per-member `Events attended ÷ All time 'Yes'
RSVPs`, over 131 members with ≥3 lifetime yes-RSVPs:

| | |
|---|---|
| Median | **88%** |
| Mean | 85% |
| p25 / p75 | 75% / 98% |
| Aggregate | 1,160 attendances / 1,347 yes-RSVPs = **86%** |

That is far above the 50–60% rule of thumb for free tech meetups. **Caveat: the
two fields do not cleanly nest** — 32 of 131 members show *more* attendances than
yes-RSVPs (walk-ins, or RSVPs edited after the fact), so the denominator is soft.
Read it as *"materially higher than the free-meetup norm,"* not as 86% ± 1.

**At the current venue** a regular meetup plausibly puts **~90–100 people in the
room** (110 cap × ~86%). That is a fact about this year's room, not a forecast:
the cap is being lifted, the group has cleared 200 twice, and there has never
been an event where demand did not exceed seats. **Design for the room the group
is moving into, not the one it is leaving.**

> **Unknown — measure by counting.** No export records actual attendance. Meetup's
> `No shows` column is **all zeros across all four files** (unused), and Luma's
> `checked_in_at` is empty for **all 294** registrations — check-in was never
> run. The room count on `mvp/FIELD-NOTES-TEMPLATE.md` is the first real
> measurement this project will have.

---

## 3. Turnover: a quarter of the room is new every time

Pre-event exports, RSVP = Yes only:

| Event | n | Never attended before | Core (10+ events) |
|---|---|---|---|
| Gametank (snapshot 1) | 110 | **27 (25%)** | 17 (15%) |
| Gametank (snapshot 2) | 110 | **30 (27%)** | 17 (15%) |
| Unconf (Meetup listing) | 46 | 7 (15%) | 13 (28%) |

Across all four files: **226 unique members**, of whom 52 appear in two exports
and 12 in three. Median lifetime attendance is 2–5 events; the maximum is 45.

**This sharpens `PHILOSOPHY.md` §1 rather than weakening it.** Roughly one in
four people at a regular meetup has never been to a Rust NYC event at all and
therefore *cannot* have seen a prior question. The memorization problem that
killed the last quiz is concentrated in the **~15% core** — the people with 10+
events, who are also the people most likely to argue out loud and most likely to
be the room's centre of gravity. Freshness protects the core. It is not what the
newcomer quarter needs, and that is fine: a question fresh for the core is fresh
for everyone, and the converse is not true.

---

## 4. Voice: what attendees actually asked for

**41 unique free-text responses** to the Meetup signup question *"What kind of
events do you want to see more of? Or, would you like to co-organize a new kind
of event?"* (62 responses before deduplication, across three exports). This is
the only attendee testimony in the entire corpus.

Grouped by theme:

| Theme | Count | Representative (verbatim, unattributed) |
|---|---|---|
| More talks / deep dives / show-and-tell | ~12 | *"would love to see talks, show & tell, supporting open source crates, code reviews, future of the language"* |
| **Beginner-friendly / onboarding** | **5** | *"I'm a Rust newbie, so anything that is aimed at helping new folks onboard to Rust, or pairs up new people with more experienced engineers, would be very appealing to me."* |
| Networking | 4 | *"Networking, hackathons"* |
| AI / ML | 4 | *"AI focused"* |
| Applied domains (finance, crypto, hardware, devops, science, web/WASM) | ~10 | *"I'd love to see more events around using Rust in the web space… Rust + WASM"* |
| First-timers reserving judgement | 3 | *"This is my first one. Have to see how this one goes."* |

### 4a. The honest reading — nobody asked for a quiz

**Zero of the 41 responses mention a quiz, a game, a competition, or any
interactive segment.** Named plainly, per this dossier's standing rule about
*interest is not demand*: there is **no attendee pull for this format in the
record**.

Three things keep that from being fatal, and they should be weighed rather than
waved:

1. **The question asked about event *topics*, not segment format.** "What kind of
   events do you want to see more of" reliably elicits talk subjects. A quiz
   segment is not the sort of thing this question surfaces even when wanted.
2. **The pull is already established from stronger evidence.** The client's Phase
   0 testimony — *"attendees had gotten all answers over and over, we stopped
   doing dtolnay's quiz at all"* — is primary evidence that the segment existed,
   was run repeatedly, and died of a specific cause. A format that ran for years
   does not need signup-form demand to justify its replacement.
3. **`PHILOSOPHY.md` already frames the quiz as organizer-driven** — *"the quiz
   is a pretext, the argument is the product."* An organizer-chosen pretext does
   not require attendees to request it by name.

What this **does** mean: the segment has no independent mandate from attendees,
so its licence to exist rests entirely on the argument it produces. If the field
notes show it did not produce one, there is no attendee constituency to appeal
to. That raises the stakes on `AC-44` and on Q2 of the field notes.

### 4b. The unexpected support — the beginner ask

Five responses ask, unprompted, for beginner-level content, and one asks
specifically for *"pairs up new people with more experienced engineers."*

That is a close description of what the segment does when it works: a beginner
and a senior engineer disagree out loud about what nine lines print, and the
beginner finds out why. **`AC-44` — *an attendee who knows only beginner Rust can
explain the solution to someone else* — is therefore not merely a quality bar. It
is the criterion on which the segment answers a request attendees actually
made.** It deserves the weight `USER_STORIES.md` already gives it.

---

## 5. Experience mix — and why the titles cannot settle it

Job titles from the Luma export (206 of 294 registrations supplied one):

| Bucket | n | % |
|---|---|---|
| Engineer, seniority unmarked | 85 | 41% |
| Exec / manager / founder | 36 | 17% |
| Other / unclear | 33 | 16% |
| Staff / principal / architect | 17 | 8% |
| Senior / lead | 15 | 7% |
| Student / intern | 14 | 7% |
| Research / science | 5 | 2% |
| Junior | 1 | 0% |

59% (172 of 294) supplied a GitHub username — a technical, engaged audience.

> **This table cannot establish Rust experience and must not be used as if it
> could.** Title measures career seniority at whatever the person's day job is;
> a Staff Engineer at a Python shop is a Rust beginner. Rust NYC draws from a
> city where Rust is rarely the primary work language, so seniority and Rust
> fluency are close to uncorrelated here.
>
> **Unknown — measure by asking.** The `AC-44` beginner sample in
> `mvp/FIELD-NOTES-TEMPLATE.md` is the correct instrument: pick someone who
> calls *themselves* new to Rust, and see whether the explanation lands.

The free text corroborates that such people are present in numbers — five
self-identified newcomers to Rust or to the group in 41 responses, plus a 25%
never-attended-before rate at each meetup.

---

## 6. `AC-52`'s 200 concurrent participants — proposed amendment WITHDRAWN

> **WITHDRAWN 2026-08-12, same day it was proposed, on client testimony.**
> **`AC-52` stands at 200 as written. No amendment is recommended.**

**What was proposed.** That `AC-52`'s 200-concurrent target be labelled a
hypothesis at "roughly 2× the observed ceiling," on the grounds that regular
meetups cap at 110 RSVPs and therefore put ~90–100 people in the room.

**Why it was wrong.** It treated a **venue supply cap as a demand ceiling.** The
110 is how many seats the current room has, not how many people want in — and
the evidence against the inference was already in §2 of this same document:
every event carried a waitlist, including **76 turned away at the Unconf**.
Demand has exceeded supply at every event in the record. The client adds two
facts no export contained: the group is **actively pursuing bigger venues**, and
the **June 26 Summer Social drew 200 and sold out**. That is the second time
Rust NYC has put 200 people in a room.

Sizing a system to a constraint the organizers are in the middle of removing is
the wrong end of the telescope. **200 is a well-evidenced target for the room
this group is moving into.**

**Also withdrawn: the consequence.** This section previously argued that a
~100-participant ceiling weakens the case for the heavier realtime substrate.
It does not — that reasoning inherited the same error. Dimension 5's assessment
stands unchanged: **Durable Objects remains the textbook fit**, and Turso's
single-writer behaviour against a 200-write deadline burst **remains a live
concern**, not a hypothetical one.

### What survives the correction

The client's own caveat is the durable part: *"that was a party not a meetup."*
A sold-out social establishes **draw**, not **participation in a technical
segment**. And `AC-52` is not a criterion about attendance — it is about
**concurrent participants**.

> **The real open variable is the participation rate, and nothing in any record
> or testimony establishes it.** 200 in the room at 40% participation is 80
> concurrent sessions; at 90% it is 180. That is a >2× spread in exactly the
> quantity `AC-54` calls the highest-risk moment in the system, and no data
> anywhere in this dossier constrains it.

This is a genuine gap rather than a defect in `AC-52` — the criterion is right to
state a number the system must survive. But the number that actually sizes the
deadline write burst is *attendance × participation rate*, and only the first
factor has ever been measured.

> **Unknown — measure by counting twice.** `mvp/FIELD-NOTES-TEMPLATE.md`
> captures room size; it now also captures how many people actually took part,
> so the first meetup to run it yields the first participation ratio this
> project has ever had. One night at ~100 people does not settle the rate at
> 200, but it is the difference between an unmeasured variable and a measured
> one. **Earliest opportunity is October** unless Aug 20 is delegated to a
> co-organizer.

Field-notes **Q2** — *do phones help or hurt* — remains the consequential
question, and it is now the **only** live line of evidence bearing on whether
Stories B1/B5 shrink. The room-size line of evidence is retracted.

---

## 7. What this dimension could not establish

| Question | Status |
|---|---|
| Actual attendance at any event | **Unknown** — no-show fields unused, Luma check-in never run. Measure by counting the room. |
| What fraction of the room would use a phone | **Unknown** — no data exists. Field notes Q2. |
| **Participation rate** — of those present, how many take part | **Unknown, and it is the variable that sizes AC-52/AC-54.** No record or testimony constrains it. First capture is the next meetup the field notes are run at — **October** unless Aug 20 is delegated. |
| Ceiling on future room size | **Rising, unquantified.** Venues are being upgraded; 200 reached twice (Unconf, Jun 26 Summer Social). No basis to name an upper bound. |
| Rust experience distribution | **Unknown** — titles measure career seniority, not Rust fluency. Measure by asking. |
| Whether attendees want the segment | **No evidence either way.** 41 responses, zero mentions. See §4a. |
| Whether the prior quiz's death is visible in the records | **Not testable** — the exports post-date it and carry no segment-level data. Client testimony remains the only evidence, and it is sufficient. |

---

*Dimension 6, written 2026-08-12 in support of touchpoint T-6; timing amended
2026-08-13. Population and voice from the group's own records; the experiential
half is captured at the next meetup via `mvp/FIELD-NOTES-TEMPLATE.md`. Sources
are local CSV exports held outside the repo — figures here are reproducible from
them, and no personally identifying data was copied in.*
