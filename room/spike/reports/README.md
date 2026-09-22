# Burst spike reports (T-03)

Measured 2026-09-22 (UTC) against `rustnyc-popquiz-spike.fly.dev` — one
`shared-cpu-1x` / 256 MB machine in `ewr` — from a laptop. The client-side
network is the laptop's. **Venue wifi is AC-55's oracle and is settled at HC-4,
not here.**

Every file is the untouched output of `burst --out`. Each one carries its own
verdict, its own raw `samples_ms`, and the exact invocation, so any figure below
can be recomputed from the file rather than taken from this page. The table was
generated from these files, not typed.

| File | Exit | Clients | Machines | AC-54 burst p95 (uniform / spike) | AC-53 write p95 [95% CI] | AC-41 reveal p95 [95% CI] | AC-52 | send-lag p95 |
|---|---|---|---|---|---|---|---|---|
| `2026-09-22.json` **INVALID** | 2 | 199/200 | 1 | 67.11 (66.75 / 67.11) | 57.38 [50.85, 64.01] | 176.36 [176.17, 176.88] | exact | 2.49 |
| `2026-09-22-run2.json` | 0 | 200/200 | 1 | 73.98 (67.16 / 73.98) | 51.66 [48.91, 61.32] | 117.47 [115.73, 131.95] | exact | 2.25 |
| `2026-09-22-run3.json` | 0 | 200/200 | 1 | 92.38 (85.2 / 92.38) | 63.49 [57.46, 73.38] | 76.47 [76.34, 76.88] | exact | 4.11 |
| `2026-09-22-run4.json` | 0 | 200/200 | 1 | 77.61 (55.53 / 77.61) | 50.35 [47.57, 54.75] | 98.99 [98.6, 99.66] | exact | 2.67 |

All times in milliseconds.

## Headline — the worst across the three valid runs

| Criterion | Threshold | Measured | |
|---|---|---|---|
| AC-54 · the deadline burst alone | p95 < 500 ms | **92.38 ms** (run 3, spike shape) | pass |
| AC-53 · writes over a full segment | p95 < 500 ms | **63.49 ms** (run 3) | pass |
| AC-41 · reveal fan-out to 200 | p95 ≤ 2000 ms | **117.47 ms** (run 2) | pass |
| AC-52 · each final answer counted once | exact | exact in all three | pass |

The headline is the **worst** valid run, never the median or the best. No
figure was flagged `marginal`: in every valid run the upper bound of the p95's
confidence interval is also far below its threshold.

## Why run 1 is kept, and why its numbers are not quoted

Run 1 connected **199 of 200** clients, so the harness marked it invalid (exit 2)
rather than report a 199-client run as a 200-client result. Its numbers would
have passed. They are still not quoted, because the rule is that an invalid run's
numbers are not quoted, and a rule that bends when the answer is convenient is
not one. It is committed rather than deleted, because deleting the run that did
not come out clean is exactly the kind of quiet curation this spike exists to
refuse.

Run 4 was taken afterwards, with the same binary, to give three valid runs.

**The cause of the missing client is not proven.** The harness records *that* a
join failed, not *why* — a diagnostic gap handed to T-21. The hypothesis the
evidence supports: session ids are deterministic (`s-0000`…`s-0199`) and reused by
every run, and the spike server decrements its `present` count only when it
notices a socket close, which behind the Fly proxy can lag. A run started
straight after another can then find capacity still held by the previous run's
closing sessions, and the last join is refused `full`. A 100-client diagnostic run
straight after a 200-client one connected **0 of 100**, which fits the same
mechanism.

That failure is **safe**: `present` gates only the join. It feeds nothing into
writes, totals or reveals, so it can make a run invalid but cannot make one pass
falsely. It is also a real lesson for **T-04b**, which owns join and capacity:
capacity has to be tied to session lifetime, not to when a close is noticed, or
two rooms run back to back will refuse joins.

## Diagnostics — not headlines, and not committed here

Kept out of this directory so that HC-0's re-run compares against the headline
runs only.

- **TLS off** (`http://`, so `ws://`): 200/200, exit 0; AC-54 65.5, AC-53 55.78,
  AC-41 133.69. TLS adds about 54 ms to *connecting* (p50 160 ms with it, 106 ms
  without) and nothing measurable to a write on an open socket. That is the
  point of starting the write clock after the handshake.
- **Half load** (100 clients): **invalid**, 0 of 100 connected, for the reason
  above. It was meant as a secondary check on whether the laptop was the
  bottleneck. The primary check, measured inside every run, covers it: send-lag
  p95 was 2.25–4.11 ms against a 25 ms invalid threshold.

## Re-running (HC-0)

```sh
cd room
cargo build --release --features spike --bin burst
./target/release/burst --url <deployed room> --clients 200 --seed 20260922 --out spike/reports/<date>.json
```

Against the walking skeleton, this client is HC-0's burst evidence. Leave a
minute between back-to-back runs, for the reason run 1 records.
