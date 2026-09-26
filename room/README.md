# room

The live half. One room per meetup, one question, seven phases from `idle` to
`released`, driven from the host's phone and shown on the wall. Rust, `axum` and
`tokio`, one small machine on Fly.io.

## Running it

From the repository root:

    just setup
    just test       # the room's tests, along with the pipeline's and the web's
    just canary     # just the in-process secrecy seam

Or directly:

    cd room && cargo test
    cd room && cargo run     # serves on 127.0.0.1:3000

## What is here

`router()` and a binary that serves it. No routes yet — that is deliberate, and
it is what lets `tests/canary.rs` prove the harness without asserting anything
about room behaviour.

The modules that will hold the behaviour are named in `BUILDPLAN.md` section 2:

| Module | What it will do | Ticket |
|---|---|---|
| `phase` | `idle → live → closed → split → work → reveal → released`, no skipping | T-04a |
| `answers` | **The sealed one.** Correct option, receipt, explanation | T-04a |
| `rooms` | Sessions, capacity, totals frozen at close | T-04b |
| `ws` | One broadcast per room: wall, buzzers, host | T-04c |
| `auth` | Discord OAuth, role-**ID** check at room creation | T-10 |

`answers` is sealed in the structural sense: it must be unreachable from the
public state query by a module boundary or the type system, not by anyone
remembering to be careful (SPEC.md G-3, AC-61).

## The toolchain

This crate builds with whatever `cargo` the machine has, and is not pinned.

That is not an oversight, and it is worth being precise about because the two
things are easy to confuse: **the verification pin is a different pin.** The
full `rustc -Vv` release and commit-hash, plus the nightly's date for Miri, are
defined once by T-15a where the sandbox image is built, and they exist so that a
question's verified answer can be tied to an exact compiler. Nothing about
building this server needs that, and putting a `rust-toolchain.toml` here would
both invite the confusion and make `just test` reach for the network on any
machine that did not happen to have the pinned toolchain already.

## The canary seam

`tests/canary.rs` drives `router()` in-process, with no socket. Today it asserts
one thing — that the router answers at all — because the mechanism is the point.
T-08 lands the real scan on top of it: canaries planted in the resolving trace
step's `note`, the explanation, the receipt and the hint, asserted absent from
every pre-reveal payload. T-25 adds the admin token as a fifth plant.

The other half of that hook — scanning the deployed room's real frames and pages
— runs in `just test-full` once T-09 has something deployed.

## The burst spike (T-03)

The one real engineering risk in this project is the deadline write burst:
200 people answering inside the two seconds before the host closes answers.
`BUILDPLAN.md` §5 retires it at T-03, **before any room code is written**, and §3
is explicit about why — if p95 misses, D-A option 2 (Cloudflare Durable Objects)
re-opens and the substrate decision is taken again. So this spike exists to be
allowed to fail, and everything below is built so that a failure is impossible
to miss and a pass is impossible to overstate.

Two throwaway binaries, both behind the default-off `spike` feature:

| Bin | What it is |
|---|---|
| `spike-server` | One room, in memory, `idle → live → closed` plus a reveal broadcast. Not the phase machine — T-04a writes that on a blank page. |
| `burst` | The load client: 200 WebSockets, timing, a JSON report, a non-zero exit on a miss. |

### Running it

```sh
# Both bins are feature-gated, so the inner loop never builds them (see below).
cd room
cargo test  --features spike          # the hermetic tests: estimator, verdict, reconciliation
cargo build --release --features spike --bin spike-server --bin burst

# Against a loopback server — proves the harness, never a headline number.
PORT=8099 ./target/release/spike-server &
./target/release/burst --url http://127.0.0.1:8099 --clients 12 --cycles 2 \
  --churn-secs 3 --reveals 2 --window-ms 500

# Against the deployed machine — this is where the real numbers come from.
./target/release/burst --url https://rustnyc-popquiz-spike.fly.dev \
  --clients 200 --seed 20260920 --out spike/reports/$(date -u +%F).json
```

`--help` lists every flag. Exit codes carry meaning and are worth knowing:

| Exit | Means |
|---|---|
| `0` | all four criteria pass as measured |
| `1` | **a criterion missed** — the server did not meet the threshold |
| `2` | **the run is invalid; do not quote its numbers** — fewer than 200 clients connected, an ack timed out, a reveal receipt never arrived, more than one Fly machine answered, `ulimit -n` under 1024, or the laptop's own send lag exceeded 25 ms |

Separating 1 from 2 is the point. Without it, a broken harness reads as a failing
server and a failing server can be excused as a broken harness. The report says
the same thing in its own fields: an invalid run carries `"quotable": false` and
each criterion's `pass` is `null`, not judged. A pass whose confidence interval
crosses its threshold still exits 0, but the verdict notes call it `MARGINAL` in
words and drop the clean-pass sentence.

`cargo test --features spike` includes four loopback tests that serve the real
spike server in-process on `127.0.0.1:0` and drive whole segments through it:
last-write-wins, frozen totals, post-close refusal, every reveal received, exact
reconciliation, the join past capacity refused `full`, a deliberately invalid run
left unjudged, and the report surviving a round trip. They prove the harness. They
never produce a headline number.

### Where the numbers come from

**The headline figures for AC-41, AC-53 and AC-54, and AC-52 at 200 sessions,
come from a run against the deployed Fly machine, from a laptop.** No loopback
number is ever quoted as any of them — `EVALUATION.md`'s AC-53 row says it
outright: *run against the deployed substrate, not a local mock.* Committed runs
live in `spike/reports/`.

The client-side network in those runs is the laptop's. **Venue wifi is AC-55's
oracle and it is settled at HC-4 in October, not here**; the report records a
failure rate but labels it as the laptop's link.

### How it tries not to lie

A load test that flatters its subject is worse than none, so the specific ways
this one could have been wrong are each defended against, and the defenses are
visible in the report rather than asserted here:

- **The write clock** starts on an already-open, already-handshaken connection
  and stops when that socket reads the matching `ack` — one clock, one task. TLS
  and connection setup land in `diagnostics.connect_ms` and can never leak into a
  write latency.
- **The ack means the write is in the authoritative map**, not that a frame
  arrived: the server drops the room lock and then acks.
- **`send_lag_ms`** — the laptop's own scheduling delay — is measured, reported,
  and never added to write latency. 200 real phones do not queue behind each
  other. Above 25 ms at p95 the run is invalid, because at that point the laptop
  is what was measured.
- **Both burst shapes run, and the worst drives AC-54.** `uniform` spreads the
  200 writes over the window; `spike` puts all of them in its last 50 ms. Both
  satisfy `SPEC.md` §9's *200 writes inside 2 s*, but AC-54 calls the burst *the
  highest-risk moment in the system*, and letting only the gentler shape reach
  the exit code would mean the harder one could never affect the go/no-go.
- **`p95` travels with its own uncertainty.** At n=200 it is the 190th sample —
  one slow client moves it. The report prints the rank, its 95% interval (ranks
  183–197, rounded outward), and the max; a pass whose upper bound crosses the
  threshold is flagged `marginal` rather than rounded into a clean pass.
- **Every raw sample ships** in `samples_ms`, so any percentile can be recomputed
  by someone who does not trust this code.
- **AC-52 is checked four ways** — per-letter totals, `answered`, the applied
  `seq` sum, and an XOR fingerprint over `(session, letter, seq)`. The first
  three are aggregates and two sessions swapping answers would slip past all of
  them; the fingerprint is what makes it a per-session claim. The server still
  exposes no per-session answer.
- **`TCP_NODELAY` on both ends.** axum does not set it (`tap_io` is the seam);
  without it a 40 ms delayed-ACK mode appears in the histogram and reads as the
  server being slow.
- **The reveal frame is padded to 2048 bytes**, because a fan-out measured on a
  60-byte frame is a measurement that lies by being easy.
- **`auto_stop_machines` is off** in `fly.spike.toml`, and every `hello` frame
  carries `FLY_MACHINE_ID`. If more than one machine ever answered, room state
  would be split across two maps and AC-52 would be reconciling against a
  fiction — so the run is invalidated rather than reported.
- **The report is written before the exit code is computed**, so a failing run
  still leaves a complete artifact behind.

### Deploying the spike

`fly.spike.toml` and `spike/Dockerfile` describe a **throwaway** app,
`rustnyc-popquiz-spike` in `ewr` on the smallest shared-cpu machine. This is not
the room's deploy — T-09 owns that, along with `room/fly.toml`,
`popquiz.rustnyc.org` and `smoke`.

The single most important line in `fly.spike.toml` is
`http_service.concurrency.hard_limit`. Fly's default is **25 connections**; at
200 WebSockets the proxy would shed connections before the server was under any
strain, and the spike would report a p95 miss belonging to the config rather
than to Rust on Fly — a false no-go on the substrate decision.

```sh
cd room
fly launch --no-deploy --copy-config --config fly.spike.toml \
  --name rustnyc-popquiz-spike --region ewr --dockerfile spike/Dockerfile
fly deploy --remote-only --config fly.spike.toml --app rustnyc-popquiz-spike \
  --dockerfile spike/Dockerfile --ignorefile spike/.dockerignore .
fly machine stop <ID> -a rustnyc-popquiz-spike   # leave it scaled to zero
```

`fly launch` may drop a `fly.toml`, `Dockerfile` or `.dockerignore` into `room/`.
None of those belong to this ticket; check `git status` afterwards and delete
them.

### What is deliberately not here

`just burst` is still the `_pending` stub. **T-21** wires this client into the
recipe and into `test-full`; `--url`, JSON on stdout and a non-zero exit on a
miss are already in place so that is a small change.

The cost of that, stated plainly rather than left to be discovered: because both
bins sit behind `required-features = ["spike"]` — which is exactly what keeps
`just test` inside its 60 s budget — **neither `test` nor `test-full`
type-checks them until T-21 lands.** Until then the gate is by hand, and it is
the two `--features spike` commands at the top of this section.
