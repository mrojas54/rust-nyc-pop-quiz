# Dimension 1 — The Problem and the People

**Scope:** Pressure-test the PRD's stated problem ("Existing Rust quizzes use a
finite, publicly available question bank. Repeat attendees can recognize
questions and answers, reducing both the challenge and the value of the
resulting technical discussion.") and map who has it, how they solve it today,
and what already exists in the market. Skeptic's brief, not an advocate's.

---

## Summary verdict

> **AMENDED 2026-08-11 — client testimony supersedes the original verdict on
> point 2.** In the Phase 0 interview the client (a Rust NYC organizer) stated:
> *"attendees had gotten all answers over and over, we stopped doing dtolnay's
> quiz at all,"* and confirmed Rust NYC previously ran a quiz using an existing
> public Rust bank.
>
> This is **primary evidence, and it is stronger than anything the public record
> could have supplied.** The desk research below could not find third-party
> documentation of bank staleness and therefore graded the problem "asserted,
> not evidenced." That grade was correct on the evidence then available and is
> now wrong. The problem is **observed, acute, and terminal**: the practice did
> not merely degrade, it *ended*. An abandoned recurring segment is the
> strongest form of the claim — a dead practice, not a hypothetical one.
>
> Two findings below survive the amendment intact and grow in importance:
> `rust-quiz`'s 37-questions-in-7-years finitude (which explains *why* it was
> exhaustible), and the finding that no mainstream live-quiz tool can display a
> 35-line Rust program. Read the rest of this dossier with point 2 corrected.

**Original verdict (superseded on point 2, retained for the record): the problem
is half true and mostly undocumented. Take it as unproven, not false — but also
not yet demonstrated to be acute enough to justify the build as scoped.**

Split the claim into its two parts:

1. *"Existing Rust quizzes use a finite, publicly available question bank."*
   **True and verifiable.** dtolnay's `rust-quiz` — the flagship, most-cited
   Rust quiz asset — has 37 questions accumulated over roughly seven years of
   community PRs (github.com/dtolnay/rust-quiz). It is finite by construction:
   contribution is manual, one PR at a time, reviewed by one maintainer.

2. *"Repeat attendees can recognize questions and answers, reducing both the
   challenge and the value of the resulting technical discussion."*
   **Asserted, not evidenced.** No search combination — spanning Reddit,
   Meetup.com groups, tech-trivia blogs, dev.to, and general web search —
   surfaced a documented account of a Rust meetup, a programming-language
   meetup, or even a generic tech pub quiz where an organizer or attendee
   complained that a static question bank had gone stale enough to hurt the
   event. There is no evidence Rust NYC itself has ever run a quiz segment at
   all (see "The people," below) — so the "existing Rust quizzes" the PRD's
   problem statement gestures at are not something Rust NYC's own attendees
   have experienced repeatedly, as far as the public record shows.

**Name the failure pattern — WITHDRAWN 2026-08-11.** The "solution in search of
a problem" charge below does not survive the client testimony above: the
organizer watched a real segment die from the exact cause the PRD names, and
stopped running it. The paragraph is retained because its *positive* finding
still holds — this is a domain engineers find intrinsically fun to build for,
so the risk of over-building past the actual need remains live even though the
need itself is now established. The demand is real; whether it demands *this
much machinery* is the open question, and it moves to Phase 2.

**Original charge (withdrawn):** this has the shape of a solution conceived from
the builder's side of the table, not the user's. The generation+verification
pipeline (LLM generation → pinned rustc/Miri verification → semantic-dedup
judge) is a genuinely hard, interesting engineering problem for someone who
already knows Rust deeply — dtolnay's own `rust-quiz` README says its best
questions come from "personal experience writing or reading Rust code and
being bewildered by its behavior," i.e., this is a domain engineers find
*intrinsically fun to generate content for*. That is **interest**, not
**demand**. Interest predicts that a hand-built quiz bank would be pleasant
and sustainable to maintain; demand would require evidence that attendees
actually disengage on a second pass through a static bank. This research
found the former in abundance (1.9k GitHub stars on `rust-quiz`, an active
Rustlings project, multiple hobbyist Rust-quiz apps) and none of the latter.

**The PRD undercuts its own problem statement.** The PRD's closing
Assumptions section states outright: *"The 0.01% compiler-error target is an
intentional, learnable answer prior and weakens the original repeat-attendee
anti-gaming objective."* In plain terms: because the target-answer
distribution is a fixed, versioned product constant (79.99% deterministic
output / 0.01% compiler error / 20% undefined behavior), any attendee who
learns *that one fact* — "compiler error is almost never right, just guess
between the three deterministic outputs or UB" — has a strategy that
generalizes across *every future quiz*, session-to-session, without ever
seeing a repeated question. A single rule of thumb, learnable in one sitting,
defeats the freshness guarantee that motivated building fresh questions in
the first place. If repeat-attendee gaming is the problem, the built system
is self-admittedly imperfect against it — and arguably *easier* to game than
memorizing a 37-question static bank would be, because the exploit is a
one-line heuristic instead of dozens of memorized Q&As.

**What would change this verdict:** (a) a first-hand account from Michelle or
another Rust NYC organizer that they tried a quiz segment (rust-quiz,
homemade slides, or otherwise) and watched engagement drop on a repeat
pass — this is directly gatherable since attendee interviews are already in
scope (run-state.md D-5); (b) evidence that a peer meetup (another language's
NYC group, a general dev-trivia series) has hit this wall and said so
publicly; (c) a credible mechanism for why a monthly-cadence, ~5,000-member,
talk-first meetup would accumulate enough *repeat* quiz-takers, quickly
enough, for staleness to bite before a hand-curated bank could simply grow
past it (see "Existing Rust quiz assets," below, for how slow that growth
actually needs to be).

---

## Evidence

### Rust NYC's actual format has no quiz precedent
Rust NYC's website (rust.nyc) and Meetup.com listing describe a monthly,
talk-driven format: doors at 6:30pm for pizza/social, talks at 7:15pm,
typically two speaker slots per event. A 10-year retrospective
(lawrenceharvey.com) covering the group's history mentions technical talks,
an annual "Unconf" (30+ simultaneous speaker tracks), and a ticket-free
"Summer Social" — no mention anywhere of a quiz, trivia, or gamified
interactive segment. This means the PRD is not proposing to *fix* an existing
segment that's gone stale; it's proposing to *introduce* a segment type the
group has no visible track record of running. That's a materially different
(and riskier) claim than "our current thing is broken" — it's closer to
"we think this new thing would be good," which needs its own validation, not
inherited urgency from a staleness narrative.

### The one credible piece of "prior art" on AI-generated technical quiz risk cuts against ease of execution, not against demand
OpenCoderRank (arxiv.org/pdf/2509.06774), an AI-generated personalized
technical-assessment platform (Python/SQL, not Rust), documents exactly the
failure modes this PRD is trying to engineer around: **ambiguous problem
statements, incorrect generated answer keys, and confusion between rote
trivia recall and applied understanding.** This is useful corroboration that
the PRD's heavy verification stack (pinned rustc, dual Miri configurations,
uniqueness judge, quality judge) is aimed at a real risk *of the generation
approach* — but it says nothing about whether meetup attendees are
demanding fresh questions. It is evidence for "if you build this, verify
hard," not evidence for "you should build this."

### Searches that came back empty are informative
The following queries, run independently, returned no account of quiz-bank
staleness as an attendee or organizer pain point at any meetup (Rust or
otherwise): "meetup trivia quiz questions reused attendees complaint
organizer," "tech meetup same trivia questions recycled questions developer
quiz night complaint," "pub quiz software engineers seen these questions
before boring," "Rust NYC trivia quiz night event past." Absence of evidence
is not proof of absence, but across four independently phrased searches
spanning general web, Reddit-adjacent, and meetup-specific framings, the
signal was consistently zero. This is the point of "stop at saturation" —
further rephrasing was unlikely to change the picture.

---

## The people

**Meetup organizers running interactive segments — do they exist at
Rust NYC, and what do they reach for?**

Rust NYC is a real, large, active group: ~5,090 Meetup members, in its 10th
year (founded ~2016), described in its own retrospective as "the world's
largest Rust user group." It has a defined organizer structure — Cole
Lawrence as listed "Super Organizer" plus roughly 10 co-organizers — and an
associated Discord (via rusteastcoast.com) that the PRD's own `nyc-organizers`
role authorization scheme is built on. This part of the PRD's user model
(Organizer = Discord role holder) is grounded in something real and
checkable, unlike the problem statement.

What they reach for **today**, per every public source checked, is: talks +
pizza/social, monthly, at borrowed corporate venues (Datadog, BlackRock,
Materialize, Nominal). The one outlier is the annual "Unconf" — a
one-day, 30-track, 200+-attendee unconference format, structurally
unrelated to a pop-quiz segment appended to a normal monthly talk night.

**How acute is their pain?** Unmeasured by this research pass, and — per the
absence of any public record of Rust NYC running a quiz before — plausibly
zero to date, simply because there is no existing quiz practice to have gone
stale. This reframes the client-facing question: the ask isn't "should we fix
our stale quiz," it's "should we add a quiz segment at all, and if so, does
freshness-of-questions matter enough on day one to justify this
architecture." That is a founder-conviction question, not a
pain-relief-for-an-existing-practice question, and it should be named as such
to the client.

**Typical attendance size — Unknown, measure by X.** Meetup.com's own event
pages (checked directly, e.g. the Aug 11, 2026 and Aug 5, 2025 events) do not
expose RSVP or attendee counts to a non-member/non-organizer view; this
research could not determine typical monthly attendance from public sources.
The one hard number available is the Unconf's 200+ attendees — the PRD's
"support up to 200 anonymous participants" ceiling looks calibrated to that
outlier annual event, not to a normal monthly meetup. **Measure by:** pull
actual RSVP/attendance history from the Rust NYC Meetup organizer dashboard
(Michelle has access) for the last 6–12 monthly events, and separately ask
how many of a typical night's attendees are first-timers vs. repeats — the
repeat-attendee fraction is the actual population at risk of the "stale
bank" problem, and until it's measured, the acuity of the problem cannot be
sized.

---

## Existing Rust quiz assets

| Asset | What it is | Bank size / growth | Live-group fit |
|---|---|---|---|
| [dtolnay/rust-quiz](https://github.com/dtolnay/rust-quiz) | "Medium to hard Rust questions with explanations," inspired by cppquiz.org; solo self-check web quiz | **37 questions** (74 files = 37 `.rs`/`.md` pairs), accumulated over ~7 years via community PR review; 1.9k GitHub stars, 69 forks | Not built for live/group use — it's a self-paced single-player web quiz with immediate reveal per question, no timer/host/room concept |
| [matklad/rust-quiz](https://github.com/matklad/rust-quiz) | A fork/derivative, smaller | Smaller than upstream | Same limitation |
| [Rustlings](https://github.com/rust-lang/rustlings) (rust-lang org) | Small compiler-error-fixing exercises, official Rust project; embeds periodic quiz files (`exercises/quizN.rs`) between topic sections | Actively maintained, exercise count not confirmed in this pass | Explicitly self-paced, individual, CLI-based — not a live-room format at all |
| Assorted mobile "Rust Quiz" apps (e.g. an iOS app by Denis Denisov) | Bite-sized solo quiz apps for learning ownership/lifetimes/traits | Size unstated in listing | Solo study tool, no live/group mechanism |
| 100-question MCQ collections (CodeForGeek, ProProfs, Brainscape flashcards) | Generic trivia-style Rust MCQ content, largely beginner-level definitional trivia, not "what does this program print" reasoning | Fixed, static | Not designed for live hosting; content skews toward recall trivia rather than the reasoning-heavy format the PRD wants |

**Could a hand-curated, growing bank solve the same problem?** Almost
certainly, for a meetup-scale cadence. dtolnay's bank grew by 37 questions
over ~7 years through casual, unpaid community contribution with a single
reviewer — a far lower bar than the PRD's five-question-average room. If
Rust NYC organizers (or motivated attendees) wrote and reviewed even 3–5
original "what does this print" questions per month by hand — using rustc
and Miri locally, the same verification substrate the PRD proposes to
automate — a hand-curated bank would out-accumulate any plausible
*repeat-attendee* exposure rate for a monthly meetup, without any of the
LLM/Modal/Pydantic AI infrastructure. This is the crux of the "build vs.
borrow" keystone decision already flagged as open in `run-state.md`, and it
is this research's strongest reason to treat that fork as genuinely live,
not a formality.

---

## Competitive landscape

| Tool | Free-tier participant cap | Monospace / code-block support | Content limits relevant to code questions | Fit for a 50–200-person live technical room |
|---|---|---|---|---|
| **Kahoot!** | **10 participants** (3 if signed up via a business account) [multiple sources converge on this figure] | None native; code must be pasted as a low-res screenshot image | **120-character question limit, 75-character answer limit** — a 35-line Rust program cannot be entered as text at all | Poor. Free tier can't seat the room; paid tiers still can't natively hold a multi-line, monospaced code block as the question stem |
| **Slido** | 100 participants (Basic/free) | No dedicated code formatting | Free plan: 3 polls total and a 1-quiz limit per event | Participant cap is workable for a 50-person night, not 200; poll/quiz-count ceiling makes a 3–5 question quiz workable but tight, and there's no code-friendly question type |
| **Mentimeter** | 50 participants **per month**, cumulative across all presentations that month (help.mentimeter.com) | No dedicated code formatting | Once the 50/month cap is hit, no further live presenting until reset | Free tier structurally cannot support a 200-person room, or even one 50+ person night without burning the whole month's quota |
| **Quizizz** | ~100 participants free | No confirmed code/syntax support | Not fully characterized in this pass | Participant cap workable for a small-to-mid room; code-question fit unconfirmed and likely weak (it's a K-12/corporate-trivia product) |
| **AhaSlides** | ~50 participants free | No dedicated code formatting | Unlimited presentations/month at that cap | Doesn't reach 200; no code-specific affordance found |
| **Wooclap** | **Unlimited participants** on the free "Starter" plan, but **capped at 5 questions per month** total (wooclap.com/en/pricing) | 21+ question types listed; no explicit code/monospace format found | 5 questions/month is workable for one 3-question room but leaves almost no room for a second session or iteration in the same month | Participant cap is actually generous enough for 200 — the real constraint is the *questions/month* budget, not headcount |
| **Poll Everywhere** | ~25–40 participants free, per third-party aggregators (Poll Everywhere's own pricing page did not resolve cleanly in this research pass — treat this figure as provisional) | No dedicated code formatting | Not fully characterized | Weakest of the group on capacity; unconfirmed on code fit |
| **Hand-curated bank + any of the above** | Inherits the chosen tool's cap | Inherits the chosen tool's formatting limits (i.e., still hits the same code-block wall) | N/A — question supply is whatever organizers write | Solves the "supply of fresh, verified questions" problem at near-zero infrastructure cost, but does **not** solve the code-formatting/monospace/mobile-readability wall any of these generic tools presents |
| **The PRD's proposed build** (Val Town + Discord auth + Modal/Pydantic AI generation + rustc/Miri verification) | Designed for 200, unvalidated (PRD's own Val Town capacity spike is an unpassed gate) | Full control — built for exactly this | Full control — 35-line/4KiB source limit is a self-imposed design choice, not a platform ceiling | The only option in this table actually designed around monospace, multi-line Rust source as the primary question content — this is the landscape's one clear, defensible gap the generic tools leave open |

**The one competitive fact that most changes the picture:** none of the
mainstream live-quiz tools can natively hold a 35-line, monospaced Rust
program as a question stem — Kahoot's 120-character cap makes this
structurally impossible without degrading to a pasted screenshot, and no
other tool in this table advertises code-block support either. If code-heavy
question *presentation* (not question *freshness*) is the real unmet need,
the PRD is solving the wrong layer of the problem: the pain isn't "the bank
runs out," it's "no generic audience-response tool can display a Rust program
properly on a phone." That's a UI/format problem solvable by a much smaller
build (a purpose-built display layer wrapping a hand-curated or slowly-grown
question bank) than the full LLM-generation-plus-Miri-verification pipeline
in the PRD.

---

## What the strongest counter-argument to building this is

**A volunteer organizer with rustc and Miri installed locally can hand-write
and verify a 3-question quiz in under an hour, once a month, indefinitely —
and a hand-curated bank growing at that pace will always be several years
ahead of any plausible repeat-attendee memorization rate for a monthly,
~200-cap meetup.** dtolnay's `rust-quiz` reached only 37 questions after
seven years of low-effort community contribution and still functions as a
respected, actively-cited resource (1.9k stars) — nobody has publicly
complained it's "used up." Pair that hand-curated (or even semi-automated,
single-pass-LLM-then-human-reviewed) bank with Wooclap's free tier — which
already handles unlimited participants, comfortably clearing the PRD's
200-person target, with a 5-questions/month ceiling that roughly matches a
single room's question count — and the organizer gets 90% of the stated
goal (fresh-feeling, verified, live, phone-friendly quiz nights) for
approximately zero infrastructure, zero Discord OAuth integration, zero
Modal/Pydantic AI sponsorship dependency, and zero maintenance burden, versus
the PRD's ~975-line specification covering a five-service trust boundary,
two unpassed feasibility spikes, and two donated-infrastructure economic
dependencies (flagged as open in `run-state.md`, D-4 and the keystone-decision
table). The counter-argument is not "don't ever build this" — it's "the
default should be 'don't build this' until someone demonstrates the
hand-curated-bank-plus-existing-tool path is insufficient in practice," and
right now nothing in the public record does that.

---

## Open questions for the client

These can only be answered by Michelle or another Rust NYC organizer,
ideally via the attendee interviews already in scope (`run-state.md`, D-5):

1. **Has Rust NYC (or a Rust NYC organizer, in any capacity) ever run a quiz
   or trivia segment before?** If yes: what happened, what was reused, and
   did anyone visibly disengage on a repeat pass? If no: what specifically
   made "quiz" the chosen new-segment format over other options (lightning
   talks, live-coding challenges, panel Q&A)?
2. **What fraction of a typical monthly meetup's attendance is repeat vs.
   first-time?** This is the actual population exposed to a stale bank, and
   it is currently unmeasured. Without it, "acuteness" cannot be sized.
3. **Is the real target event the monthly ~talk-and-pizza night, or the
   annual 200+-person Unconf?** The PRD's 200-participant ceiling reads as
   calibrated to the latter; if the actual near-term use case is a monthly
   night with materially fewer attendees, several of the build's hardest
   requirements (Val Town's unpassed 200-client capacity spike, in
   particular) may be solving for a scale that won't be exercised for a long
   time, if ever.
4. **Would a hand-curated or lightly-AI-assisted-but-human-reviewed
   question set, delivered through an off-the-shelf tool with a
   purpose-built code-display wrapper, be acceptable — or is "fully
   original, zero-human-authored, freshly generated every single session"
   a hard product requirement in itself** (e.g., because it's also meant to
   demonstrate Rust NYC's own technical sophistication to sponsors/attendees,
   independent of whether attendees would notice the difference)? This
   reframes "build vs. borrow" as partly a marketing/identity decision, not
   purely a user-pain decision — worth surfacing explicitly rather than
   assuming.
5. **Is there a sponsor-relationship or community-signaling motive** (Modal
   and Pydantic AI sponsorships are already assumed as free infrastructure
   per the PRD's Assumptions section) that makes *building* the elaborate
   pipeline valuable to Rust NYC independent of whether it solves an
   attendee-side pain — i.e., is this partly a "flagship community project"
   goal rather than a "fix attendee experience" goal? If so, that is a
   legitimate reason to build, but it is a different justification than the
   PRD's stated problem, and should be named as such rather than smuggled in
   under a staleness narrative.

---

## Sources

- PRD.md (repo root) — problem statement, users, assumptions, keystone
  contradiction (0.01% compiler-error prior weakening the anti-gaming
  objective)
- `sequence/run-state.md` — prior conflict log (C-1, C-2), keystone decisions,
  touchpoints
- [Rust NYC — Meetup.com group page](https://www.meetup.com/rust-nyc/) —
  member count (~5,090), organizer list, format description
- [Rust NYC events — Meetup.com](https://www.meetup.com/rust-nyc/events/)
- [Rust NYC: "An intro to wgpu" / "Let's Talk Generics!" — Aug 11, 2026 event](https://www.meetup.com/rust-nyc/events/315963710/)
- [Rust NYC: Validating/Optimizing DB Queries — Aug 5, 2025 event](https://www.meetup.com/rust-nyc/events/310107945/)
- [Rust NYC Unconf 2025 — Meetup.com](https://www.meetup.com/rust-nyc/events/311757146/)
- [rust.nyc](https://rust.nyc/) — mission, Discord link, format
- [rusteastcoast.com](https://rusteastcoast.com) — Discord/live-stream/job hub referenced by both rust.nyc and the PRD's auth model
- [10 Years of Rust NYC — Lawrence Harvey](https://www.lawrenceharvey.com/newsroom/10-years-of-rust-nyc-the-community-behind-the-worlds-largest-rust-user-group) — history, "world's largest Rust user group," format description
- [github.com/rust-nyc](https://github.com/rust-nyc) and [rust-nyc/meetups](https://github.com/rust-nyc/meetups) — "try to host once a month"
- [dtolnay/rust-quiz — GitHub](https://github.com/dtolnay/rust-quiz) — 37 questions, cppquiz.org inspiration, 1.9k stars, 69 forks, contribution model
- [matklad/rust-quiz — GitHub](https://github.com/matklad/rust-quiz)
- [rust-lang/rustlings — GitHub](https://github.com/rust-lang/rustlings) — self-paced exercises, embedded `quizN.rs` files
- [Rust Quiz iOS app — App Store](https://apps.apple.com/us/app/rust-quiz/id6754566593) — solo self-study app, unrelated to live-group format
- [OpenCoderRank paper (arXiv 2509.06774)](https://arxiv.org/pdf/2509.06774) — documented AI-generated technical-assessment failure modes: ambiguity, incorrect answer keys, trivia-vs-competency confusion
- Kahoot free-tier participant cap (10, or 3 for business signups) and
  question/answer character limits (120/75), cross-corroborated across:
  [TriviaAnywhere](https://www.triviaanywhere.com/blog/kahoot-free-player-limit),
  [TriviaEverywhere](https://triviaeverywhere.com/blog/kahoot-free-plan-guide/),
  [Brighterly](https://brighterly.com/blog/kahoot-pricing/),
  [Wooclap's Kahoot pricing writeup](https://www.wooclap.com/en/blog/kahoot-pricing/)
  (competitor-authored, read with appropriate skepticism),
  [Kahoot community forum thread on character limits](https://support.kahoot.com/hc/en-us/community/posts/360027192453-Increase-character-limits-for-questions-and-responses)
- [Slido Community — participant limits on free Basic plan (100)](https://community.slido.com/community-q-a-7/how-many-participants-for-a-free-plan-user-7842)
- [Mentimeter Help Center — participant limits (50/month)](https://help.mentimeter.com/en/articles/465589-how-many-people-can-participate-in-a-mentimeter-presentation)
- [AhaSlides — free-tier comparison blog (50 participants)](https://ahaslides.com/blog/free-alternative-to-mentimeter/)
- [Wooclap Pricing (official) — unlimited participants, 5 questions/month free](https://www.wooclap.com/en/pricing/)
- [Poll Everywhere / PollEv pricing](https://www.polleverywhere.com/pricing) — figure not independently confirmed in this pass; treat the commonly cited 25–40 participant free cap as provisional, sourced only from third-party aggregators (e.g. [Wooclap's Poll Everywhere pricing writeup](https://www.wooclap.com/en/blog/poll-everywhere-pricing/))
- Quizizz free participant cap (~100) — [aboutchromebooks.com Quizizz pricing summary](https://www.aboutchromebooks.com/quizizz-pricing/); code/syntax-highlighting support unconfirmed
