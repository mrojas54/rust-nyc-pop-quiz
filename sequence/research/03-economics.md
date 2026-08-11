# Dimension 3 — Economic Viability

**Prepared:** 2026-08-11. All prices below are date-stamped at the point of
citation; re-check before relying on this for a purchase decision — cloud and
SaaS pricing moves.

**Scope reminder:** target is up to 200 anonymous participants per room, ~3
questions per room by default (range 1–5), roughly one meetup per month (12
rooms/year). Stack: Val Town (participant-facing host + val-scoped SQLite),
Modal (generation orchestration + Rust compile/Miri verification), Pydantic AI
Gateway + Logfire (model calls + observability). The PRD assumes Modal and
Pydantic AI Gateway capacity are free via Rust NYC sponsor relationships, and
assumes Val Town Pro.

---

## Summary

**Headline: the sponsorship question turns out not to be the load-bearing
one.** At Rust NYC's actual scale — 200 people, once a month, 3 questions a
room — the Modal compute and LLM token spend this design generates is tiny
enough (low tens of dollars a year, worst case) that Modal's own free tier and
a personal LLM API key comfortably absorb it without any sponsor relationship
at all. The number that actually drives the annual bill is **Val Town Pro**
(~$21/mo, ~$252/yr) — which the PRD's own Assumptions section never claims is
sponsored. So:

| Scenario | Annual cost | What's in it |
|---|---:|---|
| **"Sponsored" as PRD assumes** (Modal + Pydantic Gateway donated) | **~$252–260/yr** | Val Town Pro only. Modal/Gateway/Logfire costs are $0 because someone else is paying, but that someone-else bill would itself be tiny (see below). |
| **Fully unsponsored** (Modal + Pydantic Gateway credits vanish) | **~$257–280/yr** (expected case) | Val Town Pro (~$252/yr, unchanged) + Modal compute paid directly (~$0.3–1/yr, inside Modal's own $30/mo free-tier credit) + LLM tokens paid directly to a provider (~$2–5/yr on a mid-tier model) + Logfire ($0, free tier covers this volume forever). |
| **Low band** | **~$122/yr** | Legacy Val Town Pro pricing ($10/mo, grandfathered accounts only) + cheapest plausible generation/verification cost. |
| **High band / tail risk** | **~$1,620–2,020/yr** | Not a sponsorship failure — a **Val Town capacity failure**. If the 200-participant polling design turns out to need Val Town's Business tier ($167/mo list) instead of Pro, that dominates every other line item by 6–8×. See §5. |

**Biggest cost driver:** LLM generation + judging tokens routed through
Pydantic AI Gateway, not Modal compute and not Miri verification time — Modal
compute for compiling and running Miri on a 35-line program is a fraction of
a cent per candidate even under pessimistic assumptions (§2.2). The dollars
are in the language model calls, and even those are small at this volume.

**Does a $0–20/mo off-the-shelf tool undercut this?** Partially, and it's a
material finding (§5): AhaSlides Pro (~$16/mo, unlimited audience) or Kahoot
Pro Standard ($25/mo, 200-participant cap) already deliver the *live-quiz
delivery mechanics* — timers, host controls, 200-person rooms — for less than
or comparable to the Val Town Pro line alone, with zero engineering effort.
What none of them do is generate fresh, verified, non-repeating Rust
program-output questions — that's a content problem, not a platform problem,
and it's the actual reason this PRD proposes a custom build. The "build vs.
borrow" fork already flagged in `run-state.md` is real, but resolving it is
about whether the anti-repetition problem justifies a build, not about
infrastructure cost — infrastructure cost is a wash either way.

---

## 1. Cost model — assumptions table

Every number below is derived from a formula shown in §2; change any
assumption and the totals recompute the same way.

| # | Assumption | Value used | Why / source |
|---|---|---|---|
| A1 | Meetups per year | 12 | Stated cadence ("roughly one meetup per month"). |
| A2 | Questions per room | 3 | PRD default (range 1–5). |
| A3 | Candidates attempted per **accepted** question | **Low 1.5 / Expected 2.5 / High 5** | PRD caps at 5 total attempts per slot (schema, verification, uniqueness, distribution, and quality rejections all consume an attempt). No measured acceptance rate exists yet — this is an explicit, labeled guess band, not a benchmark. The PRD's own Generation Latency Spike (100 default rooms) is the actual instrument to replace this assumption. |
| A4 | Generation model | Claude Sonnet 5 (expected case); Claude Haiku 4.5 (low); Claude Opus 5 (high) | Illustrative — the PRD leaves `PYDANTIC_AI_GENERATOR_MODEL` configurable and doesn't pin a model. Gateway also routes to OpenAI, Google, Groq, Bedrock. |
| A5 | Generation call size | ~2,000 input / ~1,000 output tokens | Estimate for a candidate `QuestionCandidate` (35-line/4KiB Rust source + choices + explanation + prompt/schema overhead). Unmeasured — flag for real token-count logging once built. |
| A6 | Quality-judge call size | ~1,500 input / ~300 output tokens | Judge sees source, choices, verified outcome, explanation, topic, difficulty, talk context; returns structured scores. |
| A7 | Semantic-judge trigger rate | 15% of candidates | Only candidates with cosine similarity ≥ 0.82 against the top-20 historical bucket go to the semantic judge; most candidates won't be near a duplicate at low room volume. Assumption, not measured. |
| A8 | Embedding cost | Negligible (~$0.00001/candidate) | Voyage-4-lite / OpenAI text-embedding-3-small class pricing (§3.3) — two orders of magnitude below the generation/judge cost, doesn't move the total. |
| A9 | Miri wall-clock per configuration | **Unknown — measure it.** Modeled here at the PRD's own p95 stage budget (60s) as a ceiling, and at 15s as an "expected" midpoint | See §2.2 and §3.2 for why this doesn't change the conclusion even at the ceiling. |
| A10 | Modal Sandbox resources | 1 physical CPU core, 2 GiB memory (PRD's stated starting limits) | Priced at Modal's Sandbox-tier rate, which is higher than standard Function pricing (§3.2). |
| A11 | Val Town plan | Pro, new-signup pricing ($21/mo billed annually) | Free tier can't hold private vals at all (0 private vals) and the app must keep OAuth secrets and internal logic private — Pro is a hard requirement, independent of Modal/Pydantic sponsorship. |

---

## 2. Show the arithmetic

### 2.1 LLM cost per candidate (the dominant line item)

Using Claude Sonnet 5 standard pricing ($3/$15 per MTok in/out — see §3.3 for
the pricing table and the intro-rate caveat):

| Call | Input tok | Output tok | Cost |
|---|---:|---:|---:|
| Generation | 2,000 | 1,000 | $0.006 + $0.015 = **$0.021** |
| Quality judge | 1,500 | 300 | $0.0045 + $0.0045 = **$0.009** |
| Semantic judge (× 15% trigger rate) | 1,500 | 200 | 0.15 × ($0.0045 + $0.003) = **$0.0011** |
| Embedding | ~500 | — | **~$0.00001** (rounds to zero) |
| **Total LLM/Gateway cost per candidate** | | | **≈ $0.031** |

### 2.2 Modal compute cost per candidate — the surprisingly small line

Sandbox-tier rate: 1 core × $0.00003942/core-sec + 2 GiB × $0.00000667/GiB-sec
= **$0.0000528/sec** combined.

| Scenario | Sandbox wall-clock | Cost |
|---|---:|---:|
| Deterministic-output candidate, **expected** case (start 5s + static 2s + compile 10s + 1 Miri config @15s + 3 native runs @6s) | 38s | $0.0020 |
| UB candidate, **expected** case (start 5s + static 2s + compile 10s + 2 Miri configs @15s each) | 47s | $0.0025 |
| Deterministic candidate at the **PRD's own p95 stage-budget ceiling** (5+2+10+60+6=83s) | 83s | $0.0044 |
| UB candidate at the **p95 ceiling**, both Miri configs (5+2+10+60+60=137s) | 137s | $0.0072 |
| Pessimistic stress test: 5× the p95 ceiling on both Miri configs | ~610s | $0.0322 |

Even the deliberately pessimistic stress-test row is a third of a cent above
the LLM cost of a single candidate. **Modal compute is not the cost driver at
any plausible Miri runtime**, including runtimes far worse than the PRD's own
60-second p95 stage budget already allows for. Modal Function orchestration
(the generation/orchestrator process, billed at the cheaper standard rate:
$0.0000131/core-sec + $0.00000222/GiB-sec) adds well under $0.001/candidate on
top of this and doesn't change the picture.

### 2.3 Roll-up: candidate → question → room → year

Total cost per candidate (LLM + Modal, expected case) ≈ $0.031 + $0.0023 ≈
**$0.033**.

| | Low (A3=1.5, Haiku-class) | Expected (A3=2.5, Sonnet-class) | High (A3=5, Opus-class) |
|---|---:|---:|---:|
| Cost/candidate | ~$0.014 | ~$0.033 | ~$0.069 |
| Cost/accepted question | ~$0.021 | ~$0.083 | ~$0.345 |
| Cost/room (3 questions) | ~$0.06 | ~$0.25 | ~$1.04 |
| **Cost/year (12 rooms)** | **~$0.75** | **~$3.00** | **~$12.42** |

(Haiku-class uses Claude Haiku 4.5 $1/$5 per MTok; Opus-class uses Claude Opus
5 $5/$25 per MTok — see §3.3.)

**Read:** across a genuinely pessimistic stress test (top-tier model, the
5-attempt ceiling on every question, 5× the PRD's own pessimistic Miri
budget), the entire generation-and-verification pipeline costs about **$12 a
year**. That number is what "Modal + Pydantic sponsorship" is actually
buying — not the $500+/year figure that would make the sponsorship
load-bearing.

---

## 3. Per-vendor pricing (dated, with links)

### 3.1 Val Town — pricing checked 2026-08-11

| Plan | Price | Runs/day | Private vals | SQLite storage | Custom domains | Log retention | API req/min |
|---|---|---:|---:|---:|---|---:|---:|
| Free | $0 | 100,000 | 0 | 10 MB | 0 | 3 days | 1,000 |
| **Pro** | **$21/mo** (annual; legacy accounts pay $10/mo forever) | 1,000,000 | 2,000 | **1 GB** | 10 | 10 days | 10,000 |
| Business | from $167/mo (annual, "$400 savings" stated) | 5,000,000 | 2,000 | 1 GB (implied, not separately stated) | Unlimited | 10 days | not separately raised |
| Enterprise | Custom | "No limits" | — | — | — | — | — |

Sources: [val.town/pricing](https://www.val.town/pricing), [val.town/limits](https://www.val.town/limits), [docs.val.town SQLite reference](https://docs.val.town/reference/std/sqlite/).

**The 200-concurrent-polling workload against these numbers, worked out
explicitly:**

- PRD polling design: 1 request/participant/second during active play
  (asking/closed/reveal), 1 request/3 sec during lobby/generation/completed.
- A default 3-question room (90s timer default) plausibly runs ~25 minutes of
  active-play polling and ~7 minutes of lobby polling before that (both are
  estimates — the PRD doesn't fix a total room duration, so treat this as
  illustrative, re-derive from real timings once measured).
- Participant-driven requests for **one room**: 200 × 1,500s × 1/sec ≈
  **300,000** (active play) + 200 × 400s × ⅓/sec ≈ **27,000** (lobby) ≈
  **~327,000 requests in a single room session.**
- That is **3.3× the Free plan's 100,000 runs/day cap**, in one event. It is
  comfortably under Pro's 1,000,000/day cap (327K is ~33% of it) — so Pro's
  daily run budget holds for a single room, but there is no headroom for a
  second room the same day or for a bad retry storm.
- Separately, Val Town also lists "API requests per minute: 10,000 (Pro/Teams)."
  This sits under the same "API & Rate Limits" heading as "Val creations per
  minute," which is a platform-management-API metric — so it most likely
  governs `api.val.town` calls (creating/editing vals), not inbound HTTP
  traffic to a *deployed* val. If that reading is wrong and it does apply to
  the deployed app's own traffic, the design's peak rate (200 req/sec = 12,000
  req/min during active play) would exceed it in the very first minute.
  **This ambiguity is exactly why the PRD's own "Val Town 200-client Capacity
  Spike" pre-implementation gate exists — treat the arithmetic above as the
  reason that gate matters economically, not just latency-wise.** If the spike
  fails or Pro's headroom proves too thin, the real-dollar consequence is
  Business tier at ~$167/mo (~$2,004/yr list, ~$1,604/yr with the stated
  annual discount) — an 6–8× jump over Pro, and the single largest swing
  factor in this whole model. This, not Modal/Pydantic sponsorship, is the
  tail-risk line item (§ Summary, high band).
- **SQLite storage is not a constraint at this scale.** Even with indefinite
  retention of source, fingerprints, descriptors, and embeddings per accepted
  question, 36 questions/year at an estimated ~15 KB/record ≈ 540 KB/year.
  Free tier's 10 MB would last ~18 years; Pro's 1 GB, well over a millennium.
  Pro is needed for private-vals and (per the run-count math above) daily
  request headroom — not storage.
- **Concurrent execution:** Val Town does not publish an explicit
  concurrent-connections cap; its worker model is per-request/serverless
  (10-second worker idle timeout, 6-hour absolute worker lifetime, 4 GiB
  worker memory). Throughput is bounded by the run-count and (possibly)
  request-rate ceilings above, not by a documented concurrency number. No
  metered overage pricing is published — Val Town's limits appear to be hard
  plan-tier caps requiring an upgrade, not pay-as-you-go overage.

### 3.2 Modal — pricing checked 2026-08-11

| Plan | Base | Included credit | Containers | GPU concurrency |
|---|---|---:|---:|---:|
| Starter | $0/mo | $30/mo | 100 | 10 |
| Team | $250/mo | $100/mo | 5,000 | 50 |
| Enterprise | Custom | Custom | Custom | Custom |

| Resource | Standard Function rate | Sandbox/Notebook rate (what Rust verification uses) |
|---|---:|---:|
| CPU | $0.0000131/core-sec | $0.00003942/core-sec |
| Memory | $0.00000222/GiB-sec | $0.00000667/GiB-sec |
| Volumes | $0.09/GiB-mo (1 TiB/mo free) | — |

Source: [modal.com/pricing](https://modal.com/pricing), checked 2026-08-11.

**Sponsorship/credit programs found:** a Startup Program (early-stage AI/ML
companies, reportedly up to $25–50K in credits depending on source — figures
disagree across secondary aggregators, treat as unverified until confirmed on
modal.com/startups) and an Academics program (~$10–25K for grad
students/researchers). **No dedicated open-source or community-meetup credit
program was found** in this search — Rust NYC's arrangement appears to be a
direct, informally-negotiated sponsor relationship rather than a published
self-serve program. That matters: if the named contact at Modal changes or
the relationship lapses without notice, there's no public fallback tier to
reapply to beyond the standard $30/mo Starter credit — which, per §2.3, is
already >30× what this workload needs at the high end.

**At Rust NYC's actual usage (§2.3: $0.75–$12.42/year in Modal+LLM spend,
of which Modal's own share is under $1/year), the free Starter tier's
$30/month credit alone — with zero sponsorship — covers roughly 30 to 400×
this workload's Modal compute.** Modal sponsorship is a convenience (no card
on file, one less thing to budget), not an economic dependency.

### 3.3 LLM generation cost, embeddings, Gateway markup, Logfire — checked 2026-08-11

| Model | Input $/MTok | Output $/MTok | Notes |
|---|---:|---:|---|
| Claude Haiku 4.5 | $1.00 | $5.00 | Cheapest capable tier |
| Claude Sonnet 5 | $3.00 ($2.00 intro through 2026-08-31) | $15.00 ($10.00 intro) | Illustrative "expected" model in this report; intro pricing expires ~3 weeks after this report's date — use $3/$15 for durable planning |
| Claude Opus 5 | $5.00 | $25.00 | Illustrative "high" model / possible quality-judge tier |

Embedding models (any is cheap enough to round to zero at this volume):
OpenAI `text-embedding-3-small` $0.02/MTok; Voyage `voyage-4-lite` $0.02/MTok,
`voyage-4` $0.06/MTok, `voyage-4-large` $0.12/MTok (input-only, no output
tokens for embeddings).

**Pydantic AI Gateway markup:** the Gateway's own documentation
([pydantic.dev/docs/ai/overview/gateway/](https://pydantic.dev/docs/ai/overview/gateway/))
states no explicit pricing beyond project/user/key-level spending limits — it
does not publish a per-token markup, and secondary sources report that
gateways in this category generally pass provider rates through unmodified in
2026. **Treat "no markup" as industry-pattern-plausible but not
vendor-confirmed** — worth a direct question to Pydantic before finalizing a
budget, but not large enough to change this report's conclusions even if a
modest markup (say 10–20%) applies, since the underlying spend is already
single-digit dollars a year.

**Logfire pricing** (effective 2026-01-01, per
[pydantic.dev/articles/logfire-pricing-change](https://pydantic.dev/articles/logfire-pricing-change)):

| Plan | Price | Seats | Projects | Included records/mo | Overage |
|---|---|---:|---:|---:|---|
| Personal | $0 | 1 | 3 | 10M | n/a |
| Team | $49/mo | 5 | 5 | 10M | $2/M |
| Growth | $249/mo | Unlimited | Unlimited | 10M | $2/M |

At the volumes this PRD's own telemetry design produces (metadata-only spans
— timing, tokens, cost, model, status, room/generation/attempt IDs, explicitly
excluding prompts/completions/source) across 12 rooms/year, span count will
be in the thousands, not millions. **The free Personal tier covers this
indefinitely — Logfire needs no sponsorship at this scale, regardless of what
the PRD's Assumptions section says.**

### 3.4 Miri verification cost — what's known, what to measure

No published benchmark exists for Miri's wall-clock cost on a workload
shaped like this one (≤35-line, ≤4 KiB Rust programs, `-Zmiri-many-seeds=0..4`
under Stacked Borrows and, for UB candidates, again under Tree Borrows). What
is documented: Miri is generally reported as **10–100× slower than native
execution**, with pathological cases reported far higher (one optimization
case study saw ~7,000×). The PRD's own Generation Latency Spike assigns an
**initial p95 stage budget of 60 seconds per Miri configuration** — that is a
target/ceiling the PRD sets for itself, not a measurement, and this report
uses it only as a pessimistic bound (§2.2).

**Unknown — measure by:** running the pinned verifier manifest (rustc + Miri
version, the exact `MIRIFLAGS`, the exact Sandbox CPU/memory limits) against a
representative corpus of accepted-shape candidates across difficulty levels
and topics, on Modal's actual Sandbox tier, and recording p50/p95/p99
wall-clock per Miri configuration. This is precisely what the PRD's own
**Generation Latency Spike** (≥100 default rooms, documented p50/p95/p99) is
designed to produce — this economics research should not substitute for that
spike, only bound its cost consequences in advance (§2.2 shows the cost stays
trivial even at 5× the PRD's own worst-case budget).

### 3.5 What it replaces — off-the-shelf live-quiz tools, checked 2026-08-11

| Tool | Free tier cap | Paid tier that covers 200 participants | Price |
|---|---:|---|---:|
| **Kahoot! 360** | 10 players (3 on a business signup) | Pro Standard — 200-participant cap (exact match to target, no headroom) | $25/mo ($300/yr), or $228/yr Pro Start caps at only 50 |
| **Slido** | ~100 participants/event (secondary-source figure — Slido's own pricing page did not render plan numbers in this fetch; verify directly) | Engage/Professional tier — exact price not confirmed | Unknown — verify at slido.com/pricing |
| **Mentimeter** | 50 participants/month | Pro — 1,000-participant audience cap | Price not confirmed in this pass (secondary source cited "$14/presenter/mo" for the unlimited-participant Basic tier, below Pro); verify at mentimeter.com/plans |
| **Quizizz / Wayground** | Free plan exists but live-session cap unclear | Standard ($19/mo, ~100 concurrent) or Premier (~$37/mo annual, 1,000 concurrent) | $19–37/mo ($228–444/yr) |
| **AhaSlides** | 50 participants | Pro — unlimited audience size | $15.95/mo (~$191/yr at that rate; likely cheaper billed annually — page states "save 67% yearly" but didn't render the exact annual figure) |

Sources: [kahoot360.com/pricing](https://kahoot360.com/pricing/) (fetched
directly), [ahaslides.com/pricing](https://ahaslides.com/pricing/) (fetched
directly, partial), plus secondary aggregators for Slido, Mentimeter, and
Quizizz — **flagged lower-confidence; verify on the vendor's own page before
using these numbers in a decision.**

**What this does and doesn't tell you:** AhaSlides Pro at ~$16/mo (~$191/yr)
or Kahoot Pro Standard at $25/mo ($300/yr) are each in the same order of
magnitude as the Val Town Pro line this PRD already needs, and deliver 200+
person live-quiz mechanics — timers, host controls, reveals, aggregate
results — with zero engineering. **They do not generate original,
compiler/Miri-verified Rust program-output questions.** That's the actual
problem statement in the PRD ("Existing Rust quizzes use a finite, publicly
available question bank. Repeat attendees can recognize questions.") — no
off-the-shelf polling tool solves it; it would still require an organizer to
hand-author a fresh question set every month and paste it into Kahoot/AhaSlides.
So the honest comparison is: **~$190–300/yr + monthly organizer authoring
time**, versus **~$252–280/yr + the one-time cost of building and maintaining
the generation/verification pipeline.** The infrastructure bill is close to a
wash; the real trade is engineering effort against organizer authoring time,
which is squarely the "build vs. borrow" fork already flagged as open in
`run-state.md` and not something this economics pass resolves.

---

## 4. Sponsorship-lapse scenario, stated plainly

**If Modal and Pydantic AI Gateway credits vanish tomorrow, the incremental
unsponsored bill is roughly $3–15/year** (§2.3), not $50/year and nowhere
close to $500/year. That number is small enough to run on a single
individual's personal Anthropic (or OpenAI/Google) API key with a few
dollars of prepaid balance, and Modal's own free Starter tier absorbs the
compute with 30–400× headroom to spare. Logfire needs no paid tier at this
volume regardless. **Sponsorship here is a nice-to-have — it saves an
organizer from putting a few dollars a year on a personal card and from
managing a Gateway spending-limit config — not a go/no-go economic
dependency.**

What *is* load-bearing, and isn't protected by any sponsorship relationship
in the PRD's own Assumptions section, is **Val Town Pro** (~$252/yr) and the
tail risk that Pro's request/run ceilings prove too thin for a 200-person
room (§3.1), which would jump the bill to Business tier (~$1,600–2,000/yr) —
an 8× swing that has nothing to do with Modal or Pydantic.

**Answering the client's framing directly: is it $5/yr, $50/yr, or
$500/yr?** For the Modal + Pydantic piece specifically — closer to the low
end of that range, roughly **$3–15/year**. It is not $500/year under any
assumption this report could construct, including a deliberately pessimistic
stress test (top-tier model, maximum candidate-retry ceiling, 5× the PRD's
own worst-case Miri budget).

---

## 5. Who pays — typical patterns for volunteer meetup tooling

Rust NYC is a volunteer-run community meetup, not a business, and the
all-in annual bill here (~$252–280/yr expected case) is modest by the
standards of what individual organizers or informal sponsor pools already
cover for meetup logistics (venue, food, swap swag, streaming). Typical
patterns for a bill this size:

- **Organizer's personal card.** Common for meetup-scale recurring SaaS
  (Meetup.com group fees, a Zoom/streaming plan, a domain) — a few hundred
  dollars a year is well within what an individual organizer typically
  absorbs without seeking a dedicated sponsor.
- **A sponsor's discretionary/marketing budget.** NYC tech meetups routinely
  have a rotating slate of company sponsors covering venue and food; asking
  one to also cover a ~$250/yr SaaS line is a small, easy ask relative to
  typical meetup sponsorship amounts (often $500–2,000/event for venue +
  catering alone).
- **Rust Foundation.** **Unknown — not confirmed in this research.** The
  Foundation's public grant programs have historically emphasized project
  infrastructure and maintainer support over individual local user-group
  tooling; whether a recurring ~$250–2,000/yr SaaS bill for one city's meetup
  qualifies for Foundation funding would need direct outreach to
  foundation.rust-lang.org's current community-grant program — do not assume
  this is available.
- **Dedicated infra sponsor (Modal/Pydantic).** Nice to have as documented in
  the PRD, but per §4 not decisive for the total bill, since the piece those
  two vendors cover is already the cheapest line item.

**Practical read:** given the total bill is dominated by Val Town Pro and is
in the low-hundreds-of-dollars range, a single sponsor or the organizer
personally can plausibly cover the entire unsponsored scenario without
needing to secure or maintain a formal Modal/Pydantic sponsor relationship at
all — which somewhat undercuts the premise that those two specific
sponsorships are a prerequisite for shipping this.

---

## 6. Open questions

1. **Miri wall-clock on this exact workload is unmeasured.** This report
   bounds the cost consequence (§2.2, §3.4) but the PRD's own Generation
   Latency Spike is the actual instrument to close this gap — run it before
   trusting any p50/p95 number here.
2. **Does Val Town's "API requests per minute" limit apply to inbound HTTP
   traffic on a deployed val, or only to the `api.val.town` management API?**
   Val Town's docs don't say explicitly (§3.1). This materially changes
   whether the described 1-second polling design is safe on Pro or needs
   Business tier — get a direct answer from Val Town support, or let the
   200-client capacity spike settle it empirically.
3. **Exact candidates-per-accepted-question ratio.** Assumed 1.5/2.5/5 in
   this report (A3) with no measured basis; the Generation Latency Spike will
   produce a real number.
4. **Pydantic AI Gateway markup — confirmed 0%, or something nonzero?** Not
   found on Pydantic's own pricing/docs pages; low-stakes given the tiny
   absolute spend, but worth a direct question before signing anything.
5. **Slido and Mentimeter's exact paid-tier prices for a 200-cap room**
   weren't confirmed against the vendor's own page in this pass (§3.5) —
   Kahoot and AhaSlides were; if the "build vs. borrow" fork gets serious
   consideration, re-verify these two directly.
6. **Rust Foundation funding eligibility** for meetup-tooling SaaS costs is
   unconfirmed (§5) — would need direct outreach, not desk research.
7. **Val Town Business-tier full pricing** wasn't fully captured (the pricing
   page showed a "from $167/mo" starting figure with an unspecified "$400
   savings" annual framing) — get the exact annual number before treating the
   high-band estimate (§ Summary) as more than a rough tail-risk bound.

---

## Sources

- [Val Town Pricing](https://www.val.town/pricing) — fetched 2026-08-11
- [Val Town Limits](https://www.val.town/limits) — fetched 2026-08-11
- [Val Town SQLite docs](https://docs.val.town/reference/std/sqlite/) — fetched 2026-08-11
- [Modal Pricing](https://modal.com/pricing) — fetched 2026-08-11
- [Pydantic AI Gateway docs](https://pydantic.dev/docs/ai/overview/gateway/) — fetched 2026-08-11
- [Pydantic Logfire pricing change announcement](https://pydantic.dev/articles/logfire-pricing-change) — fetched 2026-08-11 (effective 2026-01-01)
- Claude model pricing — Anthropic API pricing table, cached 2026-06-24 per the `claude-api` skill reference (Haiku 4.5 $1/$5, Sonnet 5 $3/$15 standard ($2/$10 intro through 2026-08-31), Opus 5 $5/$25, all per MTok in/out)
- OpenAI/Voyage embedding pricing — aggregated via [embeddingcost.com](https://embeddingcost.com/) and related trackers, 2026 — secondary source, directionally reliable given the near-zero absolute cost involved
- Miri performance — general characterization (10–100×, pathological cases to ~7,000×) via web search of Miri community discussion and benchmark writeups; no primary Anthropic/Modal/rust-lang benchmark for this specific workload shape was found — see Open Question 1
- [Kahoot! 360 Pricing](https://kahoot360.com/pricing/) — fetched 2026-08-11
- [AhaSlides Pricing](https://ahaslides.com/pricing/) — fetched 2026-08-11 (partial render)
- Slido, Mentimeter, Quizizz/Wayground pricing — secondary aggregator sources (wooclap.com, vendr.com, saasworthy.com, and similar comparison sites), 2026 — lower confidence, flagged for re-verification in §3.5 and Open Question 5
- Modal Startup/Academics credit programs — secondary sources (grantedai.com, creditforstartups.com, guptadeepak.com), 2026 — figures disagree across sources; not confirmed against modal.com/startups directly
- PRD source: `/Users/michellerojas/rust-nyc-pop-quiz/PRD.md` (975 lines) — all quiz-configuration, verification-budget, retry-ceiling, and polling-cadence figures used in the cost model are drawn from this document
