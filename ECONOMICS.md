# Economics — Rust NYC Pop Quiz

**Status:** Phase 2 economic plan. Client call pending on the model question in §5.
**Prepared:** 2026-08-11. Full workings, sources and date-stamps in
[`sequence/research/03-economics.md`](sequence/research/03-economics.md).
**Amended 2026-08-12** for D-7 (one question a night, ~5 minutes, scheduled
last). Load figures below marked *scaled* are the Dimension 3 research numbers
divided by the reduction in room-minutes — they are rescaled, **not re-derived**,
and the underlying research still describes a 3-question, 15-minute room.

Rust NYC Pop Quiz is a volunteer-run community tool, not a business. There is no
revenue line. "Economics" here means one thing: **what does it cost to run, and
who absorbs that cost, for as long as we want the segment to exist.** A tool
that dies because nobody renews a subscription has failed exactly as badly as
one that dies because its question bank was memorized.

## The headline

**The sponsorship risk we opened this research to investigate is not the real
risk.** The PRD assumes donated Modal and Pydantic AI Gateway capacity via Rust
NYC sponsor relationships. At this scale — 200 people, once a month, **1 question
a room** — that donated capacity is worth **under $5 a year**. It is a courtesy,
not a subsidy. If both sponsorships vanished tomorrow the annual bill would move
by roughly the price of a coffee.

The number that actually drives the bill is **Val Town Pro, ~$252/yr** — which
the PRD never claimed was sponsored, and which no one flagged as a risk.

| Scenario | Annual | What's in it |
|---|---:|---|
| Sponsored, as the PRD assumes | **~$252–260** | Val Town Pro only |
| Fully unsponsored | **~$257–280** | Val Town Pro + Modal (~$0.3–1) + LLM tokens (~$2–5) + Logfire ($0, free tier) |
| Low band | ~$122 | Grandfathered Val Town Pro at $10/mo |
| **Tail risk** | **~$1,620–2,020** | Val Town **Business** tier at $167/mo, if the 200-participant polling design exceeds Pro |

## The finding that matters

Read the tail-risk row against Dimension 5. The most expensive component in this
architecture is also the one least suited to the job:

- A 200-participant room polling at 1 Hz for ~15 minutes generates roughly
  **327,000 requests** (D3 §5). At one question and a ~5-minute room that is
  roughly **109,000** — *scaled, not re-derived.*
- Val Town's SQLite runs on Turso, whose classic engine is **single-writer**.
  200 near-simultaneous answer writes at the question deadline queue behind one
  lock — the exact shape most likely to blow the PRD's own 500ms p95 write
  target (D5). **D-7 does not fix this, it de-repeats it:** the burst is the
  same size, it just happens once a night instead of three times. AC-54 still
  names it the highest-risk moment in the system, and a burst you survive twice
  by luck is not a burst you have tested.
- Val Town publishes **no ceiling** for concurrent inbound traffic to a deployed
  val. The capacity gate is unpassed and, on current documentation, unpassable
  by desk research alone.

So the platform we would pay the most for is the platform we have the least
evidence can do the work. **Cloudflare Durable Objects** — one authoritative
object per room, SQLite-backed, WebSocket hibernation — is a textbook fit for
"one room, synchronized state," and lands inside the free tier at this scale
(100k DO requests/day, 13,000 GB-s/day). That is not a cost optimization. It is
a correctness argument that happens to also be cheaper.

## Cost is not the deciding variable

Off-the-shelf comparison, for honesty:

| Option | Annual | Does it solve the observed problem? |
|---|---:|---|
| AhaSlides Pro | ~$192 | No — no code display, no question freshness |
| Kahoot Pro Standard | ~$300 | No — 120-char question limit makes a 35-line Rust program impossible |
| Wooclap free | $0 | No — unlimited participants, but 5 questions/month and no code display |
| This build, on Durable Objects | **~$5–20** | Yes, if the pipeline works |
| This build, on Val Town Pro | ~$252–280 | Yes, if the capacity gate passes |

Infrastructure cost is close to a wash between building and borrowing, and both
are inside what a meetup can absorb. **Cost therefore does not decide the
build-vs-borrow fork** — it only decides *which substrate* if we build. The fork
turns on whether fresh, verified, non-repeating Rust questions are worth the
engineering, and that is a product question answered in the Phase 2 synthesis,
not here.

## Who pays

**Resolved 2026-09-03 (T-21, `tone-architect`): option 2 — the client, personally.**
*"rust on fly, i'll pay."* At the decided substrate (Rust on Fly.io, `BUILDPLAN.md`
D-A) that is single-digit dollars a month at most, inside the design target
below. Recorded rather than rewritten: the options as they were weighed.

Options, in order of durability:

1. **Rust NYC's own funds / existing sponsor budget** — most durable, survives
   any one person leaving. Requires the segment to be worth a line item.
2. **An organizer personally** — how most community meetup tooling actually gets
   paid for. Fine at $5–20/yr; a quiet liability at $252/yr, and a real one at
   $1,600/yr.
3. **Continued Modal + Pydantic sponsorship** — worth keeping, worth nothing to
   depend on. Design so that losing it costs $15/yr, not the product.

The design target that follows: **keep the unsponsored annual cost low enough
that one organizer can absorb it without thinking about it.** On Durable Objects
with batch generation, that is achievable — single-digit dollars a year. On Val
Town Pro with live generation, it is not.

## Open questions

- ~~**Q-E1**~~ — *Answered 2026-09-03: moot.* The substrate is Rust on Fly.io at
  single-digit dollars a month and the client carries it; Val Town was never
  chosen, so its ~$250/yr was never asked for.
- **Q-E2** — Are the Modal and Pydantic sponsor relationships committed for a
  defined term, or informal? Informal is fine given the $3–15/yr exposure; worth
  knowing so we never architect around them. *Client.*
- **Q-E3** — What is the real candidates-per-accepted-question ratio? Every LLM
  cost figure here rests on an explicit guess band (1.5 / 2.5 / 5) with no
  measurement behind it. *Measure — instrument the first batch run.*
- **Q-E4** — What does Miri actually cost in wall-clock per candidate? Modeled at
  the PRD's own 60s p95 ceiling; unmeasured. Doesn't change the conclusion at
  either end of the band, but it does change the batch-run duration. *Measure.*
