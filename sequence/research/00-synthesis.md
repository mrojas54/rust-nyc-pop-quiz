# Phase 2 — Refine: what the research changed

**Date:** 2026-08-11. Reads on top of dimensions 01–05 and the Phase 0 interview.
This is the fold-back: what we now believe, what the PRD got wrong, and the hard
fork that goes to the client.

---

## 1. The problem is real, and it is sharper than the PRD states

The PRD's problem statement was generic — *"existing Rust quizzes use a finite,
publicly available question bank."* The client's testimony makes it specific and
much stronger:

> *"attendees had gotten all answers over and over, we stopped doing dtolnay's
> quiz at all."*

That is not a degraded segment. That is **a dead one**. Rust NYC ran a quiz,
attendees exhausted it, and the practice was abandoned. Desk research
(Dimension 1) had graded this claim "asserted, not evidenced" and named a
solution-in-search-of-a-problem risk; that grade is now withdrawn. The mechanism
is confirmed by the numbers: dtolnay's `rust-quiz` holds **37 questions
accumulated over ~7 years** — at 3 questions a meetup, twelve meetups a year, a
room burns through it in roughly a year.

**Root cause, stated precisely:** question supply is finite relative to
cumulative attendee memory. That is the disease. Everything else is treatment.

The client also named four purposes for the segment, all four selected: spark
technical discussion, energize the room, teach specific concepts, tie into
tonight's talk. Note what that list implies — **the quiz is a pretext for the
argument afterward.** Originality is necessary because a remembered question
produces no argument. It is not sufficient, and it is not the point.

## 2. The proposed cure reintroduces the disease

This is the finding that reorganizes everything. Dimension 4 established that
the PRD's own generation design creates new memorizable structure:

| Learnable tell | What it gives away | Cost to learn |
|---|---|---|
| The published **0.01% compiler-error prior** | "Never pick *does not compile*" — a **33% relative EV lift over random guessing, with zero code reading** | One sentence |
| **`unsafe` appears in the source** | The PRD permits `unsafe` *only* in UB-targeted questions. Safe Rust cannot cause UB, so `unsafe` in the displayed program is a near-perfect tell for "the answer is *exhibits undefined behavior*" | One sentence |

The PRD half-noticed the first — its closing assumption concedes the prior
"weakens the original repeat-attendee anti-gaming objective." It did not notice
the second, which is worse.

**Put plainly: a repeat attendee needs to memorize two heuristics instead of 37
questions.** The cure is more learnable than the disease it treats. An
infinite supply of fresh questions drawn from a fixed, published distribution is
not fresh in the way that matters — the *questions* rotate, the *meta* does not,
and the meta is what a returning attendee actually learns.

This does not kill the idea. It kills one specific design: a fixed, published,
mechanically-enforced answer-category distribution. Fixing it is cheap — vary
the distribution, stop publishing it, and never let `unsafe` be a category
signal. But it must be fixed deliberately, because the PRD currently specifies
the broken version as a success metric.

## 3. The decisive reframe: freshness is a supply problem, not a runtime problem

The PRD generates and verifies questions **live, at room-open, inside a
five-minute budget.** Nothing about the observed problem requires that.

Attendees need questions they have not seen. They do not need questions that did
not exist twenty minutes ago. Those are different requirements, and the PRD
conflates them — paying the full cost of the second to satisfy the first.

Decouple generation from the live room, and the top risks dissolve:

| Risk | Source | Survives decoupling? |
|---|---|---|
| 5-minute room-ready gate, unpassed | D4 / PRD gate | **Gone** — batch generation has no deadline |
| UB-path budget compounding: ~187s = 62% of the deadline on first attempt; ~48.8% of 3-question rooms contain at least one UB slot, with one attempt of slack | D4 | **Gone** — retries are free when nobody is waiting |
| LLM question quality unmeasured; no published yield evidence for *constructing* confirmed UB | D4 | **Largely gone** — an organizer reviews the batch before it ever reaches a room |
| Modal + Pydantic as live runtime dependencies | D3 / PRD | **Gone** — an offline batch job that fails costs nothing and reruns |
| Discord outage during a live room; 15-min grace window undersized vs. real incidents (20 min – 3 h) | D2 | **Reduced** — fewer live moving parts, and the allowlist alternative removes the coupling entirely |
| 200-participant live capacity; single-writer Turso; unpassed gate | D5 | **Survives** — the live room still has to carry 200 phones. This remains the one real engineering risk. |

**The honest counter-argument**, and it deserves a straight answer: talk mode.
Doesn't tying questions to tonight's talk require generating *tonight*? No — the
PRD's own host-configuration screen asks the organizer to type the talk's
**public title and abstract**, both of which exist days before the meetup. Batch
generation the morning of, or the moment the talk is scheduled, satisfies talk
mode completely. Live generation buys nothing here that a few hours' lead time
does not.

**What decoupling costs:** an organizer has to run a batch job and skim the
results before the meetup. That is a real chore — call it 15 minutes a month —
and it must be designed for, not wished away. It also introduces a human who
could, in principle, remember an answer. Given hosts are told they "play along
too," that is a small honesty cost worth stating out loud.

## 4. The other thing nobody is building

Dimension 1 surfaced a finding independent of everything above: **no mainstream
live-quiz tool can display a 35-line Rust program.** Kahoot caps questions at
120 characters and answers at 75. Slido, Mentimeter, AhaSlides and Quizizz offer
no monospace or code presentation. Code degrades to a screenshot — unreadable on
a phone, unusable for the argument the segment exists to start.

So there are two unmet needs, not one:

1. **Question freshness** — the observed problem, solved by supply.
2. **Code legibility on a phone** — an unsolved gap in every off-the-shelf tool,
   and the reason "just use Kahoot with hand-written questions" is not actually
   available as a fallback.

The existing prototype already solves (2) well — the `SourceCode` component,
Cascadia Mono, five choices sized for thumbs. That work is not wasted by
anything below.

## 5. Economics, briefly

Full plan in [`../../ECONOMICS.md`](../../ECONOMICS.md). Three sentences: the
sponsorship risk the client flagged is worth $3–15/yr and is not load-bearing;
the real cost is Val Town Pro at ~$252/yr with a ~$1,620–2,020/yr tail if the
capacity gate forces the Business tier; and Cloudflare Durable Objects is both a
better fit for one-authoritative-room state and free at this scale. Cost does not
decide build-vs-borrow — it decides substrate.

## 6. The hard fork

Three defensible outcomes. Recommendation is **B**.

### A. Build the PRD as written
Live generation, five-minute gate, Val Town, fixed answer prior. **Not
recommended.** Two gates unpassed, the UB budget analysis says the deadline is
tight-to-infeasible on half of rooms, and the answer-prior design actively
recreates the memorization failure the project exists to fix.

### B. Build something else — decouple generation from the room ✅
Two components, neither of which has an unpassed five-minute gate:

- **Offline question pipeline.** LLM generation → pinned rustc + Miri
  verification → semantic-dedup → **organizer review** → a growing private bank.
  Runs on a laptop or a scheduled job. No deadline. Cheap retries. Human
  judgment on ambiguity and difficulty, which is the quality risk D4 could find
  no evidence to bound. Varied, unpublished answer distribution; `unsafe` never a
  category tell.
- **Live room app.** Join by code, read code, answer, reveal, aggregate. Roughly
  what the prototype already designs. On Durable Objects, plausibly free.

Preserves all four of the client's stated purposes, solves the observed problem
at its root, keeps the existing prototype's design work, and reduces the project
to one genuine engineering risk (live capacity) instead of four.

### C. Don't build — borrow
Wooclap free tier or AhaSlides Pro plus a hand-curated growing bank. **Rejected
on the evidence, and worth saying why explicitly:** every candidate tool fails on
code display, which for a Rust program-output quiz is not a cosmetic limitation
but a disqualifying one. Borrowing does not produce a working segment here.

## 7. What goes to Phase 3 regardless of the fork

- The philosophy candidate — *the questions have to be ones nobody in the room
  has seen, and the code has to be readable on a phone; everything else is
  negotiable.*
- **The meta must not be learnable.** Promote from a PRD footnote to a
  first-class principle with a test behind it.
- Verification honesty: Miri proves absence of UB **only on executions it
  sampled** (the design samples 4 seeds against Miri's own default of 64, on a
  topic list that includes concurrency). The prototype's "Verification receipt"
  is a good idea that currently overstates its own guarantee — the wording is a
  Phase 3 taste decision.
- Anonymity and the local-only recap. The prototype's *"Nothing per-person was
  recorded"* is a genuine stance and should be philosophy, not a footer.
