BURST MEASUREMENTS - PQ-3 / BUILDPLAN T-03. Measured 2026-09-22 UTC.
Target: rustnyc-popquiz-spike.fly.dev, one shared-cpu-1x / 256MB machine, ewr. Load client run FROM THE LAPTOP: the client-side network is the laptop's. Venue wifi is AC-55's oracle at HC-4, out of scope here.
Every figure below was generated from the committed JSON (room/spike/reports/), not typed. Each file carries its own raw samples_ms, verdict and invocation.

| run | exit | clients | machines | AC-54 burst p95 (uniform / spike) | AC-53 write p95 [95% CI] | AC-41 reveal p95 [95% CI] | AC-52 | send-lag p95 |
|---|---|---|---|---|---|---|---|---|
| run 1 **INVALID** | 2 | 199/200 | 1 | 67.11 (66.75 / 67.11) | 57.38 [50.85, 64.01] | 176.36 [176.17, 176.88] | exact | 2.49 |
| run 2 | 0 | 200/200 | 1 | 73.98 (67.16 / 73.98) | 51.66 [48.91, 61.32] | 117.47 [115.73, 131.95] | exact | 2.25 |
| run 3 | 0 | 200/200 | 1 | 92.38 (85.2 / 92.38) | 63.49 [57.46, 73.38] | 76.47 [76.34, 76.88] | exact | 4.11 |
| run 4 | 0 | 200/200 | 1 | 77.61 (55.53 / 77.61) | 50.35 [47.57, 54.75] | 98.99 [98.6, 99.66] | exact | 2.67 |

HEADLINE - the WORST across the 3 valid runs (run 2, 3, 4). Never the median or best.
  AC-54 deadline burst alone   92.38 ms   threshold < 500 ms    PASS  (run 3, spike shape)
  AC-53 full-segment writes    63.49 ms   threshold < 500 ms    PASS  (run 3)
  AC-41 reveal fan-out to 200 117.47 ms   threshold <= 2000 ms  PASS  (run 2)
  AC-52 each final once        exact in all three               PASS
No figure flagged marginal: the upper bound of every p95's 95% CI is also far below its threshold.
Headroom: AC-54 ~5.4x, AC-53 ~7.9x, AC-41 ~17x.

The spike shape (all 200 writes in the last 50ms of the window) set the AC-54 headline in EVERY valid run, and in run 4 it was 40% worse than uniform (77.61 vs 55.53). This is why both shapes are headline (plan-review M-3): the uniform spread alone would have flattered AC-54.

RUN 1 IS INVALID AND ITS NUMBERS ARE NOT QUOTED. It connected 199/200, so the harness exited 2 instead of reporting a 199-client run as 200. Its numbers would have passed; they are still not quoted. It is committed, not deleted. Run 4 was taken afterwards with the same binary for a third valid run.
Cause NOT proven (the harness records that a join failed, not why - diagnostic gap for T-21). Hypothesis: session ids are deterministic (s-0000..s-0199) and reused per run, and the spike server decrements 'present' only when it notices a socket close, which behind the Fly proxy lags; a back-to-back run finds capacity held by the previous run's closing sessions. A 100-client diagnostic straight after a 200-client run got 0/100 - same mechanism. FAILS SAFE: 'present' gates join only, feeds nothing into writes/totals/reveals, so it can invalidate a run but never make one falsely pass. Lesson for T-04b (owns join/capacity): tie capacity to session lifetime, not to a noticed close.

DIAGNOSTICS (not headlines, not committed):
  TLS-off (ws://): 200/200 exit 0; AC-54 65.5, AC-53 55.78, AC-41 133.69. TLS costs ~54ms on CONNECT (p50 160 vs 106ms) and nothing measurable per write on an open socket - which is why the write clock starts after the handshake.
  Half-load (100): INVALID, 0/100, reason above. Its purpose - is the laptop the bottleneck? - is covered by the primary in-run check: send-lag p95 2.25-4.11ms vs 25ms invalid threshold.

DEPLOY NOTES: The Fly org is on a trial with no payment method, so Fly stops every machine after 5 minutes. Runs were fit inside two 5-minute windows (a full run is ~55s). A mid-run trial kill would drop clients and exit 2, never a silently wrong number. fly deploy also created a 2nd machine 'for high availability' despite min_machines_running=1; only one was ever started (auto_start_machines=false keeps the other asleep) and every valid run's hello frames name exactly one machine id (683e401a273958). fly launch also rewrote fly.spike.toml, stripping every comment; the committed version was restored and passes 'fly config validate'.