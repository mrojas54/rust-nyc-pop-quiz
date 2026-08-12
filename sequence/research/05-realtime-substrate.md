# Dimension 5 — The live-room realtime substrate and its capacity

Research for the Rust NYC Pop Quiz Tone arc, Phase 1. All web sources accessed
2026-08-11 unless otherwise noted. Val Town's own limits page carries no
per-page timestamp, so treat every Val Town number below as "true as of
2026-08-11" and re-verify before the PRD's readiness gate is actually run —
these limits are explicitly subject to change.

## Summary verdict

**Needs measurement — lean skeptical.** Val Town can plausibly serve the
*read* side of 200 clients at 1 Hz on the Pro plan (180,000 requests in a
15-minute session is ~18% of the documented 1,000,000 runs/day cap). It
cannot currently be validated against a *documented* concurrency or
requests-per-second ceiling, because Val Town does not publish one for
inbound HTTP traffic to a deployed val — the only published rate limit
(1,000–10,000 requests/minute) is explicitly for calls to Val Town's own
management API, not for traffic served by your val. The higher-risk unknown
is not sustained polling; it's the **write burst** at each question deadline,
where up to 200 clients submit or change an answer within a one-to-two-second
window against a SQLite database whose documented backend (Turso/libSQL)
defaults to a single-writer model. Neither Val Town's docs nor Turso's public
docs confirm which write engine (classic single-writer WAL vs. the newer
concurrent-write/MVCC engine) backs `std/sqlite` specifically. This is the
single most important thing to measure before trusting the PRD's capacity
gate.

The strongest alternative for this exact shape of workload — one
authoritative room, small durable state, bursty fan-out, ~12 events/year,
volunteer-run — is **Cloudflare Workers + Durable Objects**, specifically a
single SQLite-backed Durable Object per room using the WebSocket Hibernation
API. It is a close-to-textbook fit and is very likely free at this scale. A
single small Fly.io VPS running a plain Axum/WS server is the strongest
*non-managed* fallback if the team wants to stay in Rust end-to-end and avoid
any platform-specific unknowns entirely.

## Val Town limits (as published, accessed 2026-08-11)

Source: [val.town/limits](https://www.val.town/limits), [val.town/pricing](https://www.val.town/pricing).

| Limit | Free | Pro ($21/mo, $252/yr) | Teams |
|---|---|---|---|
| Execution timeout (wall clock per run) | 1 min | 10 min | 10 min |
| Worker max lifetime (absolute ceiling) | 6 hrs | 6 hrs | 6 hrs |
| Worker idle timeout (before process is recycled) | 10 sec | 10 sec | 10 sec |
| Worker memory | 4 GiB | 4 GiB | 4 GiB |
| Runs per day | 100,000 | 1,000,000 | 1,000,000 |
| **API requests per minute** (Val Town *management* API — creating/reading vals, not inbound HTTP traffic to a deployed val) | 1,000 | 10,000 | 10,000 |
| Val creations per minute | 10 | 60 | 60 |
| Minimum cron interval | 15 min | 1 min | 1 min |
| Request depth (val → val → val chain) | 15 | 15 | 15 |
| Log retention | 3 days | 10 days | 10 days |
| Log lines per day | 100,000 | 200,000 | 200,000 |
| Val source file size | 80,000 characters | same | same |
| Incoming email size | 30 MiB | same | same |
| Request body size | up to 100 MB (raised from 2 MB) | same | same |
| Response size cap | none (removed via streaming architecture) | same | same |
| SQLite storage (val-scoped, via `std/sqlite`) | 10 MB | 1 GB | 1 GB (contact support for more) |

**Not documented anywhere I could find:** a per-second or concurrent-request
ceiling for HTTP traffic actually served by a val acting as a web endpoint;
how many simultaneous requests one val can process at once (single worker
process vs. a pool); behavior under a sudden burst of hundreds of
simultaneous connections. This is the load-bearing gap for the PRD's
capacity gate.

**Historical performance data point** (2024, likely stale but the only public
signal on warm-path latency): Val Town's own engineering blog reported early
HTTP-val cold starts of 500 ms–1 s, and after shipping Deno process reuse in
mid-2024, warm requests dropped to ~5–50 ms. [Source](https://github.com/val-town/val-town-product/discussions/113).
This shows the *architecture* — a process is kept warm and reused between
requests, then recycled after the 10-second idle timeout — but says nothing
about how many *concurrent* requests one warm process can serve, which is
exactly what 200 phones polling once a second requires.

**Reliability signal:** third-party uptime tracking recorded upward of 175
distinct incidents affecting Val Town over the trailing year, including
website and "HTTP (Preview)" outages in April and May 2026.
[Source](https://statusgator.com/services/val-town), [incidents](https://status.val.town/incidents).
Treat this as a caution about single-vendor dependency for a live, one-shot,
un-repeatable event (a quiz can't be rescheduled mid-outage), not as a hard
capacity number.

## SQLite concurrency assessment

Val Town's `std/sqlite` gives every val (or organization/user, legacy) its
own private database "powered by [Turso](https://turso.tech/)."
[Source](https://docs.val.town/std/sqlite/), [usage](https://docs.val.town/std/sqlite/usage/).
Neither page documents concurrency behavior, locking, write throughput, or
guidance for high-frequency or bursty writes — this is a genuine
documentation gap, not something I'm summarizing away.

What we know from the underlying technology (Turso/libSQL), which Val Town's
docs don't contradict but also don't confirm applies to `std/sqlite`
specifically:

- **Classic SQLite/libSQL WAL mode** (the long-standing default): one writer
  at a time, globally. A write transaction holds a lock for its full
  duration; concurrent writers queue behind it; readers don't block writers
  and vice versa. `SQLITE_BUSY` / "database is locked" errors happen when a
  writer can't acquire the lock inside its busy-timeout window. A busy
  timeout of 5 seconds or more is reported to eliminate lock errors in
  benchmarks of concurrent write load; short timeouts don't.
  [Source](https://tenthousandmeters.com/blog/sqlite-concurrent-writes-and-database-is-locked-errors/).
- **Turso has since built a newer concurrent-write engine** (`BEGIN
  CONCURRENT` + MVCC) that removes the single-writer bottleneck entirely and
  claims up to 4x SQLite's write throughput with no `SQLITE_BUSY` errors.
  [Source](https://turso.tech/blog/beyond-the-single-writer-limitation-with-tursos-concurrent-writes),
  [source](https://turso.tech/blog/concurrent-writes-on-turso-cloud). **It is
  not documented whether Val Town's val-scoped `std/sqlite` databases run on
  this newer engine or the classic single-writer path.** This is a direct,
  answerable question to Val Town support before relying on the platform.

**Does the substrate survive a 200-client write burst?** Assume the
pessimistic case (classic single-writer WAL, which is what Val Town's own
docs implicitly describe by saying nothing about concurrent writes). Each
answer write is a small, single-row upsert — cheap, plausibly 1–10 ms of
lock-held time including WAL/fsync overhead. Serialized, 200 such writes
arriving within the same 1–2 second deadline window is 200–2,000 ms of
total queued write time; the *last* client in that queue could see write
latency well past the 500 ms p95 target even if each individual write is
fast, purely from queuing. This is exactly the "thundering-herd write burst"
named in the prompt, and it is the most plausible single point of failure
for the stated latency target — not because SQLite can't handle 200 small
writes total, but because they're not spread out, they arrive in a spike.
Mitigations if staying on Val Town: keep write transactions to a single
statement, set an adequate busy timeout, batch/queue writes at the
application layer if `std/sqlite`'s concurrent-write engine isn't confirmed,
and consider deliberately jittering client-side submit timing so 200 writes
don't land in the same 100 ms window even when the deadline itself is exact.

## Transport options on Val Town

- **HTTP polling** — fully supported, the default, and what the PRD assumes
  (versioned conditional GET with `304 Not Modified`).
- **Server-Sent Events** — supported since June 2024 ("HTTP Streaming,"
  described by Val Town as their most-requested feature at the time; request
  body cap raised to 100 MB and the old 10 MB response cap removed as part of
  the same streaming-architecture change).
  [Source](https://blog.val.town/http-streaming).
- **WebSockets** — **not supported**, as far as this research could confirm
  for August 2026. The 2024 streaming blog post says HTTP streaming support
  "paves the way for future support for websockets," implying it wasn't
  shipped at that time, and no later Val Town blog post, changelog, or GitHub
  discussion turned up in this research announces WebSocket support shipping
  since. Treat as unsupported and verify directly (Val Town Discord or
  support) before designing around it.

Given that, if 200 concurrent long-lived SSE connections held open by one
val for a 15-minute event is itself untested (worker idle timeout is 10
seconds when a connection is *idle*, but an open SSE stream is not idle in
the same sense — this interaction is undocumented), polling remains the only
transport on Val Town with a track record at this platform.

## The 200-client arithmetic

- 1 Hz polling × 200 clients = 200 requests/second sustained during active
  play. Over a 15-minute session with most of that time in active play, that
  is on the order of 150,000–180,000 requests for the session — which
  matches the PRD's own spike design (200 clients, 15+ minutes).
- Against Val Town's **daily** run budget: Free plan (100,000 runs/day) is
  blown through by this single session alone in under 9 minutes at 200
  req/s. Pro plan (1,000,000 runs/day) absorbs one full 15-minute session
  comfortably on a daily-quota basis (~18%) — Pro is a hard requirement, not
  an optional upgrade, for this design.
- Against Val Town's **per-second/concurrency** budget: there isn't a
  published one to check against. The only published rate limit (1,000 or
  10,000 requests/minute) is for the Val Town management API, confirmed by
  Val Town's own limits page copy ("Authenticated Val Town API calls") —
  it is not the ceiling for HTTP requests hitting your val's own endpoint.
  So the 200 req/s figure is *plausible* against daily quotas but
  **unverifiable** against any documented per-second ceiling. That gap is
  the crux of why this is a "needs measurement" verdict rather than a "yes."
- The read-polling load (200 req/s, small conditional-GET payloads, mostly
  answered by cheap version checks) is very likely the *easier* half of the
  problem. The write burst at each deadline (described above) is the harder
  half, and it's the one the PRD's own gate criteria (write latency, no
  persistent lock failures, no lost/duplicated responses) are actually
  designed to catch.

## Alternatives comparison

Evaluated for this specific workload: 200 anonymous clients, one
authoritative room, ~15-minute bursts, ~12 events/year, volunteer-run, low
tolerance for ops burden.

| Option | Natively solves fan-out? | Cost at this scale | Ops burden | Notes |
|---|---|---|---|---|
| **Val Town (current plan)** | No — polling only, no push | Pro required, $21/mo (or pause between events) | Very low — already the PRD's chosen stack | Undocumented concurrency ceiling for HTTP traffic; SQLite write-serialization risk under burst; ~175 tracked incidents/year is a real single-event-risk signal |
| **Cloudflare Workers + Durable Objects** | **Yes** — a DO is one authoritative, single-threaded, globally-coordinated instance per room; WebSocket Hibernation API fans out pushes near-free | Free tier likely covers it outright (100k DO requests/day, 13,000 GB-s/day free compute, 5 GB SQLite storage free); paid tier is $5/mo base if exceeded | Low-to-moderate — one Worker + one Durable Object class, `wrangler deploy`; more code than Val Town's polling model but a textbook match for "one room, synchronized state" | Closest fit to the actual problem shape; SQLite-backed DO storage is per-DO-instance, sidestepping the shared-database write-contention question entirely since each room is its own storage |
| **PartyKit** | Yes, for WebSocket rooms | Effectively free at this scale | Low | PartyKit now runs each "party" as a Cloudflare Durable Object under the hood — it's a developer-experience layer over the same substrate as the row above, not a separate infrastructure provider. Consider it "Durable Objects with a friendlier SDK" rather than a distinct option |
| **Supabase Realtime** | Yes — broadcast/presence channels | Free tier: 200 concurrent connections (**exactly** at our ceiling, no headroom), 2M msgs/mo; Pro $25/mo → 500 connections | Moderate — full Postgres project, RLS policies, Edge Functions for write validation; heavier than the problem needs | Free tier's 200-connection cap means zero margin for reconnects/dupes at a live event; would want Pro for headroom, which is fine cost-wise but adds Postgres-level ops the workload doesn't otherwise need |
| **Deno Deploy** | No native single-room primitive | Free tier: 1M req/mo, 1 GiB KV, generous | Low-moderate | Supports WebSocket and a global KV store, but has no Durable-Object-style single-instance-per-room guarantee — broadcasting to all connections on a stateless/multi-isolate deploy needs your own pub/sub layer, which reintroduces the coordination problem DOs solve for free |
| **Fly.io single small VPS** | No — you write it (trivial at 200 conns) | ~$2–10/mo if always-on, near-zero with scale-to-zero/auto-stop machines billing per-second, ideal for 12 events/year | Moderate — you own the box: deploys, TLS, a Dockerfile/fly.toml, SQLite file (or Litestream), updates | 200 concurrent WebSocket connections on a single small process is a non-event at this scale — this is the "no platform-specific unknowns" option and keeps the whole stack in Rust if desired (e.g., Axum + tokio-tungstenite) |
| **Firebase / Firestore** | Yes — realtime listeners are the product | Spark (free) tier plausibly covers one 15-min event (well under 50k reads/day, 20k writes/day); Blaze pay-as-you-go is pennies beyond that | Low-moderate — mature SDKs, but a different toolchain/vendor (security rules, console) from the rest of the stack | Realtime Database (sibling product) is documented to handle up to 200,000 simultaneous connections on Blaze — enormous headroom versus our 200; well-trodden path for exactly this kind of live-quiz app |
| **Ably / Pusher** | Yes, for the push leg only — not a database | Ably free: 200 concurrent connections + 200 channels, 6M msgs/mo (exactly at our ceiling, no headroom); Pusher Sandbox free tier caps at **100** concurrent connections — below our 200-client requirement outright, would need a paid Pusher plan | Low for the transport; still need your own system of record | Interesting *hybrid* option: keep Val Town/SQLite as system of record for writes, use Ably (paid tier, for headroom) purely to push "state changed" events to clients instead of 200×1Hz polling — this converts the read side to push while leaving the harder write-burst problem where it already is |

## Venue network reality

200 phones on a single meetup/coworking-space wifi network is genuinely
risky infrastructure, independent of anything Val Town does:

- **What typically fails first is association/DHCP at the start of the
  event**, not steady-state throughput. Consumer/prosumer APs common at
  meetup venues often have small default DHCP pools and are not tuned for
  high-density join storms; 200 phones associating within a few minutes of
  "please join the wifi" can exhaust a pool or overload the AP's
  association handling well before any polling traffic even starts.
- **Airtime contention, not raw bandwidth, is the steady-state constraint.**
  Wifi is a shared, half-duplex medium; every device's request/ACK cycle
  consumes airtime regardless of payload size. 200 clients each issuing a
  new-or-reused HTTPS request once per second creates a real minimum
  airtime floor from 802.11 overhead (management frames, ACKs, backoff),
  even though each individual poll payload is tiny. This is worse on a
  2.4 GHz-heavy venue AP than on a modern 5/6 GHz-capable one, and there's
  no way to know which the venue has without asking.
- **SSE/WebSocket vs. polling on flaky wifi:** a single persistent
  connection per client is *gentler* in steady state (no repeated
  connection setup, minimal idle airtime) but *more fragile* to any network
  blip — venue wifi routinely drops long-lived TCP connections on AP
  roaming, DHCP lease renewal, or phones backgrounding the browser tab to
  save battery. The failure mode isn't graceful degradation, it's a
  **reconnect storm**: a shared network hiccup drops most of the 200
  connections simultaneously, and naive reconnect logic re-establishes them
  all in the same instant, recreating the exact "200 simultaneous requests"
  spike polling was supposed to avoid — just less often. Any push-based
  transport needs jittered, exponential-backoff reconnection to actually be
  gentler than polling in practice; without it, it isn't.
- **Practical implication for this event:** whatever transport is chosen,
  plan for the wifi itself to be the weakest link — test on venue-similar
  gear if possible, have a fallback narrower than "everyone refresh," and
  don't assume push transports are automatically safer than polling on
  networks this uncontrolled.

## Simplification check

1 Hz polling is not required by the "reveal within 2 seconds at p95" target
— it's one implementation choice, and probably not the cheapest one that
still meets the bar.

- **2-second jittered conditional polling** (each client picks its own
  ~1.5–2.5s interval instead of a synchronized exact-1s tick) roughly halves
  sustained request volume versus 1 Hz, still comfortably clears the 2s
  reveal target, and — critically — avoids creating a synchronized
  200-requests-in-the-same-instant spike, which softens both the Val Town
  load question and the venue-wifi airtime question at once. This is a
  small change to the PRD's already-built `304`-based conditional polling
  design, not a redesign.
- **Long-polling** (server holds the request open until state changes or a
  timeout) would deliver near-immediate updates with less request volume
  than fixed-interval polling, but its interaction with Val Town's
  10-second worker idle timeout and undocumented per-connection concurrency
  behavior is unknown — I would not choose this on Val Town specifically
  without first getting a straight answer from Val Town about how long a
  request can be held open and how many concurrently held requests one val
  can serve.
- **SSE** is the architecturally "correct" choice for push-based reveal —
  supported by Val Town since 2024 — but 200 concurrently open SSE streams
  held by a single val for 15 minutes is untested territory on this
  platform, and reconnection-storm behavior on venue wifi (above) still
  needs jitter/backoff regardless of transport.

**What I'd actually do:** ship 2-second jittered conditional polling as the
default — it's the smallest change from what's already designed, is the
transport most forgiving of both Val Town's unknowns and flaky venue wifi,
and clears the 2-second reveal target with margin. Treat SSE as a stretch
upgrade only after the capacity spike (below) proves the platform out; do
not reach for 1 Hz synchronized polling or WebSockets (unsupported) for an
event at this scale.

## Recommended smallest experiment to settle the gate

The PRD already specifies a well-formed gate (200 simulated clients, 15+
minutes, versioned polling, answer changes near deadlines, all host
transitions, measuring write latency / reveal latency / poll failure rate /
lock failures / lost-or-duplicated responses at p50/p95/p99). That's the
right end-to-end test and should still be run. But it's a half-day-or-more
effort to build, and it bundles together several independently-answerable
questions. Two things are worth doing *first*, both answerable in under an
hour, that would let the team abort early if the substrate is a clear no
rather than discovering that after building the full harness:

1. **Ask Val Town directly** (Discord or hi@val.town) two concrete
   questions before writing any load-test code: (a) does `std/sqlite` run
   on Turso's classic single-writer engine or the newer concurrent-write
   (MVCC) engine, and (b) is there a documented or practical ceiling on
   concurrent/per-second HTTP requests to one val's serving endpoint. Either
   answer is useful — a confirmed concurrent-write engine substantially
   de-risks the write-burst concern; a confirmed hard concurrency ceiling
   below ~200 ends the investigation immediately.
2. **A synthetic SQLite write-burst micro-benchmark**, isolated from the
   rest of the app: deploy a minimal Val Town HTTP val with the intended
   `responses` table schema and one endpoint that performs a single-row
   upsert. Fire 200 concurrent write requests at it in a tight burst (a
   simple k6 or Node script, no real devices needed) and record commit
   latency p50/p95/p99 and any `SQLITE_BUSY`/lock errors. Repeat 3–5 times
   to see variance. This isolates the single highest-risk unknown named in
   this report — the thundering-herd write burst — without needing 200
   devices, a venue, or the full room state machine. If this fails badly
   (p95 far past 500 ms, or any lock failures), stop: revise the transport
   or host per the PRD's own instruction before investing in the full
   200-client, 15-minute end-to-end spike, since that spike would almost
   certainly fail for the same reason.

Only after (1) and (2) look survivable should the team run the PRD's full
200-participant, 15-minute Val Town Capacity Spike as already specified —
at that point it's confirming an end-to-end result, not fishing for a
root cause.

## Open questions

- Does Val Town route concurrent HTTP requests to one val across multiple
  worker processes, or serialize them through a single warm process? Not
  documented; directly determines whether 200 simultaneous polls are served
  in parallel or queued.
- Which write engine backs val-scoped `std/sqlite` — classic single-writer
  libSQL/WAL, or Turso's newer `BEGIN CONCURRENT`/MVCC engine? Not
  documented by Val Town; this is the single highest-leverage fact to get
  confirmed.
- What happens to an in-flight request or write if the serving worker hits
  its 10-second idle timeout or 6-hour max lifetime mid-event? Not
  documented.
- Has Val Town shipped WebSocket support since the 2024 "paves the way for
  future support" note? Not confirmed either way in this research as of
  August 2026 — verify directly before ruling it out or in.
- What AP hardware and DHCP pool size does the actual venue have? Unknown
  and unresearchable remotely — this needs a direct question to the venue,
  not a web search, since it materially changes the venue-wifi risk
  assessment above.

## Sources

- [Val Town — Limits](https://www.val.town/limits)
- [Val Town — Pricing](https://www.val.town/pricing)
- [Val Town Docs — SQLite](https://docs.val.town/std/sqlite/)
- [Val Town Docs — SQLite Usage](https://docs.val.town/std/sqlite/usage/)
- [Val Town Docs — full reference text](https://docs.val.town/llms-full.txt)
- [Val Town Docs — Cron](https://docs.val.town/vals/cron/)
- [Val Town Blog — HTTP Streaming (June 2024)](https://blog.val.town/http-streaming)
- [GitHub — val-town-product discussion #113, "HTTP vals are far too slow"](https://github.com/val-town/val-town-product/discussions/113)
- [GitHub — val-town-product discussion #19, "Usage limit alerts"](https://github.com/val-town/val-town-product/discussions/19)
- [StatusGator — Val Town status history](https://statusgator.com/services/val-town)
- [Val Town status page — incidents](https://status.val.town/incidents)
- [macwright.com — Val Town 2023–2025 retrospective](https://macwright.com/2025/11/11/val-town)
- [Turso — Beyond the single-writer limitation](https://turso.tech/blog/beyond-the-single-writer-limitation-with-tursos-concurrent-writes)
- [Turso — SQLite concurrent writes early preview](https://turso.tech/blog/concurrent-writes-on-turso-cloud)
- [SQLite concurrent writes and "database is locked" errors — tenthousandmeters.com](https://tenthousandmeters.com/blog/sqlite-concurrent-writes-and-database-is-locked-errors/)
- [Turso Docs — Multi-Process Access](https://docs.turso.tech/sql-reference/multiprocess-access)
- [Cloudflare Workers — Pricing](https://developers.cloudflare.com/workers/platform/pricing/)
- [Cloudflare Durable Objects — Pricing](https://developers.cloudflare.com/durable-objects/platform/pricing/)
- [Cloudflare Durable Objects — What are Durable Objects](https://developers.cloudflare.com/durable-objects/concepts/what-are-durable-objects/)
- [PartyKit overview — Zaira Labs guide](https://zairalabs.ai/guide/tools/partykit-cloud/)
- [Supabase Docs — Realtime Limits](https://supabase.com/docs/guides/realtime/limits)
- [Supabase Docs — Realtime concurrent peak connections quota](https://supabase.com/docs/guides/troubleshooting/realtime-concurrent-peak-connections-quota-jdDqcp)
- [Deno Docs — Subhosting pricing and limits](https://docs.deno.com/subhosting/manual/pricing_and_limits/)
- [Fly.io — Resource pricing](https://fly.io/docs/about/pricing/)
- [Ably vs Pusher pricing comparison](https://ably.com/compare/ably-vs-pusher/pricing)
- [Firebase — Firestore billing example](https://firebase.google.com/docs/firestore/billing-example)
- Project context: `/Users/michellerojas/rust-nyc-pop-quiz/PRD.md` (Val Town
  Application section, Success Metrics, Pre-Implementation Readiness Gates —
  the existing "Val Town Capacity Spike" gate this report evaluates)
