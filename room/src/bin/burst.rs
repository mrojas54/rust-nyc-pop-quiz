//! The T-03 burst spike's load client. **Throwaway, but honest.**
//!
//! 200 synthetic participants against a deployed Fly machine, measuring the four
//! criteria the ticket names:
//!
//! | Criterion | Measure | Pass |
//! |---|---|---|
//! | AC-54 | the deadline burst alone: 200 writes inside a 2 s window | p95 write < 500 ms |
//! | AC-53 | answer-write p95 over a full segment including the burst | p95 < 500 ms |
//! | AC-41 | reveal fan-out: one broadcast reaching all 200 clients | p95 <= 2 s |
//! | AC-52 | 200 sessions complete a segment, each final answer counted once | exact |
//!
//! The point of the spike is that it is allowed to fail: BUILDPLAN §5 says a p95
//! miss re-opens D-A option 2 before any room code is written. So every number
//! is reported as measured, every raw sample ships in the report so the
//! percentiles can be recomputed by someone who does not trust this file, and a
//! pass whose confidence interval crosses the threshold is flagged `marginal`
//! rather than rounded into a pass.
//!
//! **Run it from the laptop against the deployed URL.** The client-side network
//! here is the laptop's; venue wifi is AC-55's oracle at HC-4 and is out of
//! scope for this ticket. See README.md.
//!
//! T-21 turns this into the `burst` justfile recipe: `--url`, JSON on stdout and
//! a non-zero exit on a miss are all already here, so that is a small step.

#[path = "spike_shared.rs"]
mod shared;

// The spike server, compiled into this bin's *tests* only, so the loopback test
// can serve the real router on 127.0.0.1:0 in-process and drive a whole segment
// through it. Nothing ships in the `burst` binary from this. Its own `main` and
// its unit tests come along too and are harmless here — the tests run once more,
// which costs milliseconds.
#[cfg(test)]
#[path = "spike-server.rs"]
#[allow(dead_code)]
mod server;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use shared::{answer_contribution, index_letter, splitmix64};
use std::{
    sync::{Arc, LazyLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::Message;

// ---------------------------------------------------------------------------
// Thresholds — SPEC.md §9 and the EVALUATION.md row for each criterion.
// ---------------------------------------------------------------------------

/// AC-53, AC-54: "under 500ms at p95". Strictly under.
const WRITE_P95_MS: f64 = 500.0;
/// AC-41: "within 2 seconds at p95". At most, so exactly 2000 passes.
const REVEAL_P95_MS: f64 = 2000.0;
/// The laptop's own scheduling delay. Above this, the harness rather than the
/// server is what got measured, and the run is invalid rather than failing.
const SEND_LAG_P95_INVALID_MS: f64 = 25.0;
const ACK_TIMEOUT: Duration = Duration::from_secs(5);

/// One process-wide origin, so a client's receipt instant can travel to the
/// driver as a plain number and still be compared against the driver's issue
/// instant. Both are on this one clock, so no cross-machine skew can enter AC-41.
static EPOCH: LazyLock<Instant> = LazyLock::new(Instant::now);

fn since_epoch_ms(i: Instant) -> f64 {
    i.saturating_duration_since(*EPOCH).as_secs_f64() * 1000.0
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

/// A percentile summary that carries its own uncertainty.
///
/// p95 at n=200 is a single order statistic — one slow client moves it — so the
/// rank, its confidence interval and the max all travel beside the number.
#[derive(Debug, Clone)]
struct Stats {
    n: usize,
    p50: f64,
    p95: f64,
    p95_rank: usize,
    p95_ci_rank: (usize, usize),
    p95_ci: (f64, f64),
    p99: f64,
    max: f64,
    samples: Vec<f64>,
}

/// Nearest-rank: `x_(ceil(q*n))`, ascending, 1-based.
///
/// Exact, with no bucket quantization to explain, and the raw samples ship so
/// anyone can recompute it. At n <= 1000 an HDR histogram would buy nothing.
fn nearest_rank(sorted: &[f64], q: f64) -> (usize, f64) {
    let n = sorted.len();
    if n == 0 {
        return (0, f64::NAN);
    }
    let rank = ((q * n as f64).ceil() as usize).clamp(1, n);
    (rank, sorted[rank - 1])
}

/// The rank's 95% interval, rounded **outward**.
///
/// The rank of the q-th percentile has binomial sampling error
/// `sigma = sqrt(n*q*(1-q))`; the interval is `rank +/- 1.96*sigma`. Flooring the
/// low end and ceiling the high end widens it, so `marginal` fires more often,
/// never less. For a mechanism whose whole job is "do not soften a miss",
/// outward is the only defensible direction.
fn ci_ranks(n: usize, q: f64, rank: usize) -> (usize, usize) {
    if n == 0 {
        return (0, 0);
    }
    let sigma = (n as f64 * q * (1.0 - q)).sqrt();
    let lo = ((rank as f64) - 1.96 * sigma).floor().max(1.0) as usize;
    let hi = (((rank as f64) + 1.96 * sigma).ceil() as usize).min(n);
    (lo, hi)
}

impl Stats {
    fn new(mut samples: Vec<f64>) -> Self {
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let n = samples.len();
        let (p95_rank, p95) = nearest_rank(&samples, 0.95);
        let (_, p50) = nearest_rank(&samples, 0.50);
        let (_, p99) = nearest_rank(&samples, 0.99);
        let (lo_rank, hi_rank) = ci_ranks(n, 0.95, p95_rank);
        let p95_ci = if n == 0 {
            (f64::NAN, f64::NAN)
        } else {
            (samples[lo_rank - 1], samples[hi_rank - 1])
        };
        let max = samples.last().copied().unwrap_or(f64::NAN);
        Stats { n, p50, p95, p95_rank, p95_ci_rank: (lo_rank, hi_rank), p95_ci, p99, max, samples }
    }

    /// A pass whose upper confidence bound crosses the threshold. Not a failure,
    /// but never reported as a clean pass either.
    fn marginal(&self, threshold: f64, strictly_under: bool) -> bool {
        if self.n == 0 {
            return false;
        }
        let passes = if strictly_under { self.p95 < threshold } else { self.p95 <= threshold };
        let crosses =
            if strictly_under { self.p95_ci.1 >= threshold } else { self.p95_ci.1 > threshold };
        passes && crosses
    }

    fn to_json(&self, threshold: f64, strictly_under: bool) -> Value {
        let above_p99 =
            self.n.saturating_sub(((0.99 * self.n as f64).ceil() as usize).max(1));
        json!({
            "n": self.n,
            "p50_ms": r2(self.p50),
            "p95_ms": r2(self.p95),
            "p95_rank": self.p95_rank,
            "p95_ci_rank": [self.p95_ci_rank.0, self.p95_ci_rank.1],
            "p95_ci_ms": [r2(self.p95_ci.0), r2(self.p95_ci.1)],
            "p99_ms": r2(self.p99),
            "p99_note": format!("informational: {above_p99} samples above it at n={}", self.n),
            "max_ms": r2(self.max),
            "marginal": self.marginal(threshold, strictly_under),
            "samples_ms": self.samples.iter().map(|s| r2(*s)).collect::<Vec<_>>(),
        })
    }
}

fn r2(v: f64) -> f64 {
    if v.is_finite() {
        (v * 100.0).round() / 100.0
    } else {
        -1.0
    }
}

// ---------------------------------------------------------------------------
// AC-52's reconciliation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default)]
struct Tally {
    totals: [u32; 5],
    answered: usize,
    seq_sum: u64,
    fingerprint: u64,
}

#[derive(Debug, Clone)]
struct Recon {
    expected: Tally,
    server: Tally,
    per_letter_match: bool,
    answered_match: bool,
    sum_match: bool,
    seq_sum_match: bool,
    fingerprint_match: bool,
}

impl Recon {
    fn matches(&self) -> bool {
        self.per_letter_match
            && self.answered_match
            && self.sum_match
            && self.seq_sum_match
            && self.fingerprint_match
    }

    fn to_json(&self) -> Value {
        json!({
            "expected_totals": self.expected.totals,
            "server_totals": self.server.totals,
            "per_letter_match": self.per_letter_match,
            "answered": self.server.answered,
            "answered_match": self.answered_match,
            "sum": self.server.totals.iter().sum::<u32>(),
            "sum_match": self.sum_match,
            "expected_seq_sum": self.expected.seq_sum,
            "server_seq_sum": self.server.seq_sum,
            "applied_seq_sum_match": self.seq_sum_match,
            "applied_fingerprint_match": self.fingerprint_match,
            "match": self.matches(),
        })
    }
}

/// Four checks, not one.
///
/// Per-letter totals catch a lost or duplicated write. `answered` and the sum
/// catch a miscount. `seq_sum` catches a stale write winning over a fresh one.
/// The fingerprint catches the one case the other three cannot — two sessions
/// swapping answers, which leaves every aggregate unchanged.
fn reconcile(expected: &Tally, server: &Tally, clients: usize) -> Recon {
    Recon {
        expected: expected.clone(),
        server: server.clone(),
        per_letter_match: expected.totals == server.totals,
        answered_match: server.answered == clients,
        sum_match: server.totals.iter().sum::<u32>() as usize == clients,
        seq_sum_match: expected.seq_sum == server.seq_sum,
        fingerprint_match: expected.fingerprint == server.fingerprint,
    }
}

// ---------------------------------------------------------------------------
// The verdict
// ---------------------------------------------------------------------------

/// Splitting "a criterion missed" from "the run is invalid" is what stops a
/// broken harness being read as a failing server, or a failing server being
/// excused as a broken harness.
#[derive(Debug, PartialEq, Clone, Copy)]
enum Exit {
    Pass = 0,
    Missed = 1,
    Invalid = 2,
}

#[derive(Debug, Default, Clone)]
struct VerdictInput {
    burst_headline_p95: f64,
    segment_write_p95: f64,
    reveal_p95: f64,
    ac52_ok: bool,
    /// Every burst-only cycle reconciled too, not just the segment.
    burst_reconciled: bool,
    post_close_ok: bool,
    clients_connected: usize,
    clients_expected: usize,
    send_lag_p95: f64,
    ulimit_nofile: u64,
    distinct_instances: usize,
    errors: u64,
    missing_reveal_receipts: u64,
    segment_population_ok: bool,
    /// A pass whose p95 upper confidence bound crosses the threshold. Not a
    /// miss — the exit code stays 0 — but the verdict has to say so in words,
    /// or "marginal" is a flag nobody reads.
    burst_marginal: bool,
    segment_marginal: bool,
    reveal_marginal: bool,
}

fn verdict(v: &VerdictInput) -> (Exit, Vec<String>) {
    let mut notes = Vec::new();
    let mut invalid = false;

    // Invalidity is decided first: a run that did not happen properly has no
    // numbers worth reading, pass or fail.
    if v.clients_connected != v.clients_expected {
        notes.push(format!(
            "RUN INVALID: {} of {} clients connected",
            v.clients_connected, v.clients_expected
        ));
        invalid = true;
    }
    if v.ulimit_nofile < 1024 {
        notes.push(format!("RUN INVALID: ulimit -n is {}, under 1024", v.ulimit_nofile));
        invalid = true;
    }
    if v.distinct_instances > 1 {
        notes.push(format!(
            "RUN INVALID: {} distinct Fly machine ids answered — room state was split across machines, so AC-52 counted against a fiction",
            v.distinct_instances
        ));
        invalid = true;
    }
    if v.errors > 0 {
        notes.push(format!(
            "RUN INVALID: {} harness errors (a connect, send or ack timeout, or a client that never reported a stage)",
            v.errors
        ));
        invalid = true;
    }
    if v.missing_reveal_receipts > 0 {
        notes.push(format!(
            "RUN INVALID: {} reveal receipts never arrived — a missed reveal must not be read as a fast one",
            v.missing_reveal_receipts
        ));
        invalid = true;
    }
    if v.send_lag_p95 > SEND_LAG_P95_INVALID_MS {
        notes.push(format!(
            "RUN INVALID: client send lag p95 {:.1} ms exceeds {:.0} ms — the laptop, not the server, is what was measured",
            v.send_lag_p95, SEND_LAG_P95_INVALID_MS
        ));
        invalid = true;
    }
    if !v.segment_population_ok {
        notes.push(
            "RUN INVALID: the segment write population is not churn+burst — post-close refusals must never be latency samples"
                .into(),
        );
        invalid = true;
    }
    if invalid {
        return (Exit::Invalid, notes);
    }

    let mut missed = false;
    if !(v.burst_headline_p95 < WRITE_P95_MS) {
        notes.push(format!(
            "AC-54 MISS: deadline-burst write p95 {:.1} ms, threshold < {:.0} ms",
            v.burst_headline_p95, WRITE_P95_MS
        ));
        missed = true;
    }
    if !(v.segment_write_p95 < WRITE_P95_MS) {
        notes.push(format!(
            "AC-53 MISS: segment write p95 {:.1} ms, threshold < {:.0} ms",
            v.segment_write_p95, WRITE_P95_MS
        ));
        missed = true;
    }
    if v.reveal_p95 > REVEAL_P95_MS {
        notes.push(format!(
            "AC-41 MISS: reveal fan-out p95 {:.1} ms, threshold <= {:.0} ms",
            v.reveal_p95, REVEAL_P95_MS
        ));
        missed = true;
    }
    if !v.ac52_ok {
        notes.push("AC-52 MISS: the segment reconciliation did not match".into());
        missed = true;
    }
    if !v.burst_reconciled {
        notes.push(
            "AC-52 MISS: a burst-only cycle's reconciliation did not match — a write acked during the burst was lost or double-counted"
                .into(),
        );
        missed = true;
    }
    if !v.post_close_ok {
        notes.push(
            "AC-52 MISS: a post-close write was accepted, or the saved answer was not restated (SPEC.md §4.3)"
                .into(),
        );
        missed = true;
    }

    // Marginal is reported on a pass only: on a miss, the miss is the story —
    // including when the miss is a *different* criterion. Gating each line only
    // on its own criterion passing let a MARGINAL note sit beside an unrelated
    // AC-52 MISS under exit 1, which is the opposite of what this comment says.
    let mut marginal = 0;
    for (flag, passed, text) in [
        (v.burst_marginal, v.burst_headline_p95 < WRITE_P95_MS, "AC-54"),
        (v.segment_marginal, v.segment_write_p95 < WRITE_P95_MS, "AC-53"),
        (v.reveal_marginal, v.reveal_p95 <= REVEAL_P95_MS, "AC-41"),
    ] {
        if flag && passed && !missed {
            notes.push(format!(
                "{text} MARGINAL: the p95 passes, but the upper bound of its 95% confidence interval crosses the threshold — this is not a clean pass"
            ));
            marginal += 1;
        }
    }

    if missed {
        (Exit::Missed, notes)
    } else if marginal > 0 {
        notes.push(format!(
            "all four criteria pass as measured, {marginal} of them marginally — read the notes above before quoting this as a pass"
        ));
        (Exit::Pass, notes)
    } else {
        notes.push("all four criteria pass as measured".into());
        (Exit::Pass, notes)
    }
}

// ---------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum Shape {
    /// Offsets uniform over the whole window — SPEC.md §9's literal wording.
    Uniform,
    /// All of them inside the last 50 ms of the window: a room reacting to one
    /// countdown rather than drifting in over two seconds.
    Spike,
}

impl Shape {
    fn as_str(self) -> &'static str {
        match self {
            Shape::Uniform => "uniform",
            Shape::Spike => "spike",
        }
    }
}

#[derive(Clone, Debug)]
struct Args {
    url: String,
    clients: usize,
    seed: u64,
    out: Option<String>,
    warmup: bool,
    shapes: Vec<Shape>,
    cycles: usize,
    churn_secs: u64,
    window_ms: u64,
    reveals: usize,
    reveal_bytes: usize,
    /// Pause after the ramp. 3 s on a real run; the loopback test shortens it.
    settle_ms: u64,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            url: "http://127.0.0.1:8080".into(),
            clients: 200,
            seed: 1,
            out: None,
            warmup: false,
            // Both shapes run by default and the worst drives AC-54. AC-54 calls
            // the burst "the highest-risk moment in the system"; letting only the
            // gentler shape reach the exit code would mean the harder one could
            // never affect the D-A go/no-go.
            shapes: vec![Shape::Uniform, Shape::Spike],
            cycles: 3,
            churn_secs: 20,
            window_ms: 2000,
            reveals: 5,
            reveal_bytes: 2048,
            settle_ms: 3000,
        }
    }
}

const USAGE: &str = "\
burst - the T-03 spike's load client

  --url <base>          http(s):// base of the spike server (ws/wss derived from it)
  --clients <n>         default 200
  --seed <n>            makes a run reproducible
  --out <path>          also write the JSON report here
  --warmup              short run, always exit 0, numbers not for quoting
  --burst-shape <s>     uniform | spike | both   (default both; worst drives AC-54)
  --cycles <n>          burst cycles per shape (default 3)
  --churn-secs <n>      segment churn window (default 20)
  --window-ms <n>       burst window (default 2000)
  --reveals <n>         reveal broadcasts (default 5)
  --reveal-bytes <n>    reveal payload padding (default 2048)
  --settle-ms <n>       pause after the ramp (default 3000)

Exit: 0 all criteria pass - 1 a criterion missed - 2 run invalid, do not quote.";

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut i = 0;
    while i < argv.len() {
        let flag = argv[i].clone();
        let take = |i: &mut usize| -> Result<String, String> {
            *i += 1;
            argv.get(*i).cloned().ok_or_else(|| format!("{flag} needs a value"))
        };
        match argv[i].as_str() {
            "--url" => a.url = take(&mut i)?,
            "--clients" => {
                a.clients = take(&mut i)?.parse().map_err(|_| "--clients wants a number")?
            }
            "--seed" => a.seed = take(&mut i)?.parse().map_err(|_| "--seed wants a number")?,
            "--out" => a.out = Some(take(&mut i)?),
            "--warmup" => a.warmup = true,
            "--cycles" => {
                a.cycles = take(&mut i)?.parse().map_err(|_| "--cycles wants a number")?
            }
            "--churn-secs" => {
                a.churn_secs = take(&mut i)?.parse().map_err(|_| "--churn-secs wants a number")?
            }
            "--window-ms" => {
                a.window_ms = take(&mut i)?.parse().map_err(|_| "--window-ms wants a number")?
            }
            "--reveals" => {
                a.reveals = take(&mut i)?.parse().map_err(|_| "--reveals wants a number")?
            }
            "--reveal-bytes" => {
                a.reveal_bytes =
                    take(&mut i)?.parse().map_err(|_| "--reveal-bytes wants a number")?
            }
            "--settle-ms" => {
                a.settle_ms = take(&mut i)?.parse().map_err(|_| "--settle-ms wants a number")?
            }
            "--burst-shape" => {
                a.shapes = match take(&mut i)?.as_str() {
                    "uniform" => vec![Shape::Uniform],
                    "spike" => vec![Shape::Spike],
                    "both" => vec![Shape::Uniform, Shape::Spike],
                    other => return Err(format!("unknown --burst-shape {other}")),
                }
            }
            "--help" | "-h" => return Err("__help__".into()),
            other => return Err(format!("unknown flag {other}")),
        }
        i += 1;
    }
    if a.warmup {
        a.cycles = 1;
        a.churn_secs = 3;
        a.reveals = 1;
        a.shapes = vec![Shape::Uniform];
    }
    Ok(a)
}

// ---------------------------------------------------------------------------
// Stages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Stage {
    Burst { cycle: usize, shape: Shape, window_ms: u64, start: Instant },
    Churn { start: Instant, secs: u64 },
    PostClose,
    AwaitReveals { count: usize },
    Stop,
}

#[derive(Debug, Default, Clone, Copy)]
struct WriteSample {
    ms: f64,
    lag_ms: f64,
    accepted: bool,
}

/// What a client hands back at the end of each stage. Carrying the per-stage
/// final answer here is what lets the driver reconcile a cycle without the
/// server ever exposing a per-session answer.
#[derive(Debug, Default)]
struct StageDone {
    id: usize,
    writes: Vec<WriteSample>,
    final_answer: Option<(u8, u32)>,
    reveals: Vec<(u64, f64)>,
    post_close_refused: bool,
    post_close_restated: bool,
    errors: u64,
}

#[derive(Debug, Default)]
struct Joined {
    connect_ms: f64,
    join_rtt_ms: f64,
    instance: String,
    ok: bool,
}

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

fn session_id(id: usize) -> String {
    format!("s-{id:04}")
}

/// The per-client burst offset inside the window, derived from the seed so a run
/// reproduces exactly.
fn burst_offset_ms(seed: u64, cycle: usize, client: usize, shape: Shape, window_ms: u64) -> u64 {
    let r = splitmix64(seed ^ ((cycle as u64) << 40) ^ (client as u64));
    match shape {
        Shape::Uniform => r % window_ms.max(1),
        Shape::Spike => window_ms.saturating_sub(50) + (r % 50),
    }
}

/// The churn schedule for one client, ascending.
///
/// Sorted, and that is not cosmetic. A client sends its churn writes one at a
/// time, awaiting each ack, so the schedule has to be monotonic: drawing 2800 ms
/// and then 500 ms would mean sleeping past the second instant and booking the
/// difference as send lag. Unsorted, this alone pushed send-lag p95 to ~480 ms
/// on loopback and invalidated the run.
fn churn_offsets(seed: u64, id: usize, count: usize, secs: u64) -> Vec<u64> {
    let mut v: Vec<u64> = (0..count)
        .map(|k| {
            splitmix64(seed ^ 0xC0FFEE ^ ((k as u64) << 32) ^ id as u64) % (secs * 1000).max(1)
        })
        .collect();
    v.sort_unstable();
    v
}

/// `https://h` -> (`wss`, h, 443). `http://h` -> (`ws`, h, 80).
fn ws_target(base: &str) -> Result<(&'static str, String, u16), String> {
    let (scheme, rest) = if let Some(r) = base.strip_prefix("https://") {
        ("wss", r)
    } else if let Some(r) = base.strip_prefix("http://") {
        ("ws", r)
    } else {
        return Err(format!("--url must start with http:// or https:// (got {base})"));
    };
    let host_port = rest.trim_end_matches('/');
    let (host, port) = match host_port.rsplit_once(':') {
        Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
            (h.to_string(), p.parse().unwrap_or(0))
        }
        _ => (host_port.to_string(), if scheme == "wss" { 443 } else { 80 }),
    };
    Ok((scheme, host, port))
}

async fn connect(base: &str) -> Result<WsStream, String> {
    let (scheme, host, port) = ws_target(base)?;
    let tcp =
        tokio::net::TcpStream::connect((host.as_str(), port)).await.map_err(|e| e.to_string())?;
    // Nagle would put a 40 ms mode in the write histogram that belongs to the
    // kernel, not the server. Set on both ends; the server does it via tap_io.
    tcp.set_nodelay(true).map_err(|e| e.to_string())?;
    let url = format!("{scheme}://{host}:{port}/ws");
    let (ws, _) = tokio_tungstenite::client_async_tls_with_config(url, tcp, None, None)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ws)
}

/// `t_send` is taken on an already-open, already-handshaken connection and
/// `t_ack` when the matching ack is read off that same socket, in the same task
/// on the same clock. The interval holds local encrypt, kernel, network RTT, the
/// Fly proxy and the server's apply — and no connect, handshake or upgrade.
async fn send_answer(
    socket: &mut WsStream,
    session: &str,
    seq: u32,
    letter: char,
    scheduled: Instant,
) -> Result<WriteSample, ()> {
    let frame = json!({"t": "answer", "session": session, "seq": seq, "letter": letter.to_string()})
        .to_string();
    let t_send = Instant::now();
    // Recorded, reported, and never added to write latency: 200 real phones do
    // not queue behind each other, so charging the laptop's scheduler to the
    // server would be a lie in the wrong direction.
    let lag_ms = t_send.saturating_duration_since(scheduled).as_secs_f64() * 1000.0;
    socket.send(Message::Text(frame.into())).await.map_err(|_| ())?;

    loop {
        match tokio::time::timeout(ACK_TIMEOUT, socket.next()).await {
            Ok(Some(Ok(Message::Text(t)))) => {
                let t_ack = Instant::now();
                let Ok(v) = serde_json::from_str::<Value>(&t) else { continue };
                if v.get("t").and_then(Value::as_str) != Some("ack")
                    || v.get("seq").and_then(Value::as_u64) != Some(seq as u64)
                {
                    continue;
                }
                return Ok(WriteSample {
                    ms: t_ack.duration_since(t_send).as_secs_f64() * 1000.0,
                    lag_ms,
                    accepted: v.get("accepted").and_then(Value::as_bool).unwrap_or(false),
                });
            }
            Ok(Some(Ok(_))) => continue,
            _ => return Err(()),
        }
    }
}

async fn run_client(
    id: usize,
    args: Arc<Args>,
    mut stages: broadcast::Receiver<Stage>,
    joined_tx: mpsc::Sender<Joined>,
    done_tx: mpsc::Sender<StageDone>,
) {
    let session = session_id(id);
    let connect_started = Instant::now();
    let mut socket = match connect(&args.url).await {
        Ok(s) => s,
        Err(_) => {
            let _ = joined_tx.send(Joined { ok: false, ..Default::default() }).await;
            return;
        }
    };
    let connect_ms = connect_started.elapsed().as_secs_f64() * 1000.0;

    let mut instance = String::from("?");
    if let Some(Ok(Message::Text(t))) = socket.next().await {
        if let Ok(v) = serde_json::from_str::<Value>(&t) {
            instance = v.get("instance").and_then(Value::as_str).unwrap_or("?").to_string();
        }
    }

    let join_started = Instant::now();
    let join = json!({"t": "join", "session": session}).to_string();
    let mut ok = socket.send(Message::Text(join.into())).await.is_ok();
    let mut join_rtt_ms = 0.0;
    if ok {
        match socket.next().await {
            Some(Ok(Message::Text(t))) => {
                join_rtt_ms = join_started.elapsed().as_secs_f64() * 1000.0;
                if let Ok(v) = serde_json::from_str::<Value>(&t) {
                    // A `refused` here means capacity was hit: the run is not
                    // the run that was asked for.
                    ok = v.get("t").and_then(Value::as_str) == Some("joined");
                }
            }
            _ => ok = false,
        }
    }
    let _ = joined_tx.send(Joined { connect_ms, join_rtt_ms, instance, ok }).await;
    if !ok {
        return;
    }

    let mut seq: u32 = 0;

    loop {
        let stage = match stages.recv().await {
            Ok(s) => s,
            Err(_) => break,
        };
        let mut out = StageDone { id, ..Default::default() };

        match stage {
            Stage::Burst { cycle, shape, window_ms, start } => {
                let off = burst_offset_ms(args.seed, cycle, id, shape, window_ms);
                let scheduled = start + Duration::from_millis(off);
                tokio::time::sleep_until(scheduled.into()).await;
                seq += 1;
                let li = ((id + cycle) % 5) as u8;
                match send_answer(&mut socket, &session, seq, index_letter(li), scheduled).await {
                    Ok(s) => {
                        if s.accepted {
                            out.final_answer = Some((li, seq));
                        }
                        out.writes.push(s);
                    }
                    Err(_) => out.errors += 1,
                }
            }

            Stage::Churn { start, secs } => {
                // 1 + (id mod 3) writes: 67 clients change their mind once, 67
                // twice, 66 three times. Last write wins (SPEC.md §4.3, AC-34).
                let count = 1 + (id % 3);
                for (k, off) in
                    churn_offsets(args.seed, id, count, secs).into_iter().enumerate()
                {
                    let scheduled = start + Duration::from_millis(off);
                    tokio::time::sleep_until(scheduled.into()).await;
                    seq += 1;
                    let li = ((id + k) % 5) as u8;
                    match send_answer(&mut socket, &session, seq, index_letter(li), scheduled).await
                    {
                        Ok(s) => {
                            if s.accepted {
                                out.final_answer = Some((li, seq));
                            }
                            out.writes.push(s);
                        }
                        Err(_) => out.errors += 1,
                    }
                }
            }

            Stage::PostClose => {
                // SPEC.md §4.3: a write after close is refused with the saved
                // answer restated. A correctness probe, never a latency sample —
                // it must not enter AC-53's population.
                seq += 1;
                let now = Instant::now();
                match send_answer(&mut socket, &session, seq, 'E', now).await {
                    Ok(s) => out.post_close_refused = !s.accepted,
                    Err(_) => out.errors += 1,
                }
                out.post_close_restated = out.post_close_refused;
            }

            Stage::AwaitReveals { count } => {
                let mut seen = 0;
                while seen < count {
                    match tokio::time::timeout(Duration::from_secs(15), socket.next()).await {
                        Ok(Some(Ok(Message::Text(t)))) => {
                            let at = Instant::now();
                            let Ok(v) = serde_json::from_str::<Value>(&t) else { continue };
                            if v.get("t").and_then(Value::as_str) == Some("reveal") {
                                let rid = v.get("reveal_id").and_then(Value::as_u64).unwrap_or(0);
                                out.reveals.push((rid, since_epoch_ms(at)));
                                seen += 1;
                            }
                        }
                        Ok(Some(Ok(_))) => continue,
                        _ => {
                            out.errors += 1;
                            break;
                        }
                    }
                }
            }

            Stage::Stop => break,
        }

        if done_tx.send(out).await.is_err() {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// The control socket
// ---------------------------------------------------------------------------

/// Sends one control frame and reads until `control_ok`, skipping the reveal
/// broadcasts the control socket also receives.
async fn control(socket: &mut WsStream, cmd: &str, extra: Value) -> Result<Value, String> {
    let mut frame = json!({"t": "control", "cmd": cmd});
    if let (Some(f), Some(e)) = (frame.as_object_mut(), extra.as_object()) {
        for (k, v) in e {
            f.insert(k.clone(), v.clone());
        }
    }
    socket.send(Message::Text(frame.to_string().into())).await.map_err(|e| e.to_string())?;
    loop {
        match tokio::time::timeout(Duration::from_secs(15), socket.next()).await {
            Ok(Some(Ok(Message::Text(t)))) => {
                let Ok(v) = serde_json::from_str::<Value>(&t) else { continue };
                if v.get("t").and_then(Value::as_str) == Some("control_ok") {
                    return Ok(v);
                }
            }
            Ok(Some(Ok(_))) => continue,
            _ => return Err(format!("control {cmd}: no control_ok")),
        }
    }
}

fn tally_from_control(v: &Value) -> Tally {
    let mut totals = [0u32; 5];
    if let Some(arr) = v.get("totals").and_then(Value::as_array) {
        for (i, x) in arr.iter().take(5).enumerate() {
            totals[i] = x.as_u64().unwrap_or(0) as u32;
        }
    }
    Tally {
        totals,
        answered: v.get("answered").and_then(Value::as_u64).unwrap_or(0) as usize,
        seq_sum: v.get("applied_seq_sum").and_then(Value::as_u64).unwrap_or(0),
        fingerprint: v
            .get("applied_fingerprint")
            .and_then(Value::as_str)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
    }
}

fn expected_from(finals: &[(usize, u8, u32)]) -> Tally {
    let mut t = Tally::default();
    for (id, letter, seq) in finals {
        t.totals[*letter as usize] += 1;
        t.seq_sum += *seq as u64;
        t.fingerprint ^= answer_contribution(&session_id(*id), *letter, *seq);
        t.answered += 1;
    }
    t
}

/// Waits for every client to report the stage, and counts any that never do as
/// a harness error.
///
/// That count is what keeps the exit codes honest. A client whose stage report
/// went missing may already have had its write applied server-side, so without
/// this the run would come back as an AC-52 *miss* — exit 1, blaming the server —
/// when what actually failed was the harness, which is exit 2. Taking `errors`
/// by reference means no call site can forget to add the shortfall in.
async fn collect(
    done_rx: &mut mpsc::Receiver<StageDone>,
    n: usize,
    errors: &mut u64,
) -> Vec<StageDone> {
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        match tokio::time::timeout(Duration::from_secs(120), done_rx.recv()).await {
            Ok(Some(d)) => v.push(d),
            _ => break,
        }
    }
    *errors += n.saturating_sub(v.len()) as u64;
    v
}

fn ulimit_nofile() -> u64 {
    std::process::Command::new("sh")
        .arg("-c")
        .arg("ulimit -n")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn now_unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    // tokio-tungstenite declares rustls with default features off, so no crypto
    // provider is guaranteed. Installing ring here makes TLS deterministic
    // rather than dependent on which feature some other crate happened to turn
    // on. Ignoring the result is correct: it only errors if one is already set.
    let _ = rustls::crypto::ring::default_provider().install_default();
    LazyLock::force(&EPOCH);

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => Arc::new(a),
        Err(e) if e == "__help__" => {
            eprintln!("{USAGE}");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("burst: {e}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    let invocation = std::env::args().collect::<Vec<_>>().join(" ");
    match run(args.clone(), invocation).await {
        Ok((report, code)) => {
            let text = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into());
            println!("{text}");
            // The report is written before the exit code is acted on, always, so
            // a failing run still leaves a full committed artifact behind.
            if let Some(path) = &args.out {
                if let Some(dir) = std::path::Path::new(path).parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                if let Err(e) = std::fs::write(path, &text) {
                    eprintln!("burst: could not write {path}: {e}");
                }
            }
            let code = if args.warmup { Exit::Pass } else { code };
            eprintln!("burst: exit {}", code as i32);
            std::process::exit(code as i32);
        }
        Err(e) => {
            eprintln!("burst: {e}");
            std::process::exit(2);
        }
    }
}

async fn run(args: Arc<Args>, invocation: String) -> Result<(Value, Exit), String> {
    // Fix the clock origin before any instant is taken. `main` does this too,
    // but `run` is also driven directly by the loopback test, and a lazily
    // initialised origin would otherwise be set *after* the first reveal's issue
    // instant — saturating it to zero and inflating every fan-out.
    LazyLock::force(&EPOCH);
    let started_at = now_unix_ms();
    eprintln!("burst: {} clients against {}", args.clients, args.url);

    let mut ctl = connect(&args.url).await.map_err(|e| format!("control connect: {e}"))?;
    let _ = ctl.next().await; // hello

    let (stage_tx, _) = broadcast::channel::<Stage>(64);
    let (joined_tx, mut joined_rx) = mpsc::channel::<Joined>(args.clients + 8);
    let (done_tx, mut done_rx) = mpsc::channel::<StageDone>(args.clients + 8);

    // Ramp: 20 clients per 250 ms. Handshakes all happen here, so their cost can
    // never land inside a write latency.
    for id in 0..args.clients {
        tokio::spawn(run_client(
            id,
            args.clone(),
            stage_tx.subscribe(),
            joined_tx.clone(),
            done_tx.clone(),
        ));
        if (id + 1) % 20 == 0 {
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }
    drop(joined_tx);
    drop(done_tx);

    let mut joins = Vec::new();
    for _ in 0..args.clients {
        match tokio::time::timeout(Duration::from_secs(120), joined_rx.recv()).await {
            Ok(Some(j)) => joins.push(j),
            _ => break,
        }
    }
    let connected = joins.iter().filter(|j| j.ok).count();
    let mut instances: Vec<String> =
        joins.iter().filter(|j| j.ok).map(|j| j.instance.clone()).collect();
    instances.sort();
    instances.dedup();
    let connect_stats = Stats::new(joins.iter().filter(|j| j.ok).map(|j| j.connect_ms).collect());
    let join_stats = Stats::new(joins.iter().filter(|j| j.ok).map(|j| j.join_rtt_ms).collect());
    eprintln!("burst: {connected}/{} connected, instances {:?}", args.clients, instances);

    // Settle, so the ramp's own scheduling noise is not in the first cycle.
    tokio::time::sleep(Duration::from_millis(args.settle_ms)).await;

    let n = connected;
    let mut errors: u64 = 0;
    let mut all_send_lags: Vec<f64> = Vec::new();

    // --- Run A: the deadline burst alone (AC-54) ---------------------------
    let mut shape_blocks = Vec::new();
    let mut burst_headline = f64::NAN;
    // The marginal flag of whichever cycle set the headline. AC-54 is the
    // criterion this spike exists for; its headline must carry the flag itself
    // rather than leave it nested three levels down in the cycle that set it.
    let mut burst_headline_marginal = false;
    // Every burst cycle is reconciled, and every one has to match. A write that
    // was acked during the burst and then lost is exactly what AC-52 forbids.
    let mut burst_cycles_reconciled = true;
    let mut cycle_counter = 0usize;

    for shape in args.shapes.clone() {
        let mut cycles_json = Vec::new();
        let mut pooled: Vec<f64> = Vec::new();
        let mut worst = f64::NAN;
        let mut worst_marginal = false;

        for c in 0..args.cycles {
            control(&mut ctl, "reset", json!({})).await?;
            control(&mut ctl, "live", json!({})).await?;
            let start = Instant::now() + Duration::from_millis(300);
            let _ = stage_tx.send(Stage::Burst {
                cycle: cycle_counter,
                shape,
                window_ms: args.window_ms,
                start,
            });
            cycle_counter += 1;

            let dones = collect(&mut done_rx, n, &mut errors).await;
            let mut writes = Vec::new();
            let mut lags = Vec::new();
            let mut finals = Vec::new();
            for d in &dones {
                errors += d.errors;
                for w in &d.writes {
                    writes.push(w.ms);
                    lags.push(w.lag_ms);
                }
                if let Some((l, s)) = d.final_answer {
                    finals.push((d.id, l, s));
                }
            }
            all_send_lags.extend(lags.iter().copied());

            let ok = control(&mut ctl, "close", json!({})).await?;
            let server = tally_from_control(&ok);
            let recon = reconcile(&expected_from(&finals), &server, n);

            let st = Stats::new(writes.clone());
            pooled.extend(writes);
            if st.n > 0 && (worst.is_nan() || st.p95 > worst) {
                worst = st.p95;
                worst_marginal = st.marginal(WRITE_P95_MS, true);
            }
            burst_cycles_reconciled &= recon.matches();
            cycles_json.push(json!({
                "cycle": c,
                "writes": st.to_json(WRITE_P95_MS, true),
                "send_lag": Stats::new(lags).to_json(SEND_LAG_P95_INVALID_MS, true),
                "reconcile": recon.to_json(),
            }));
            eprintln!(
                "burst: {} cycle {c}: write p95 {:.1} ms (n={}), reconcile {}",
                shape.as_str(),
                st.p95,
                st.n,
                recon.matches()
            );
        }

        if burst_headline.is_nan() || worst > burst_headline {
            burst_headline = worst;
            burst_headline_marginal = worst_marginal;
        }
        shape_blocks.push(json!({
            "shape": shape.as_str(),
            "window_ms": args.window_ms,
            "cycles": cycles_json,
            "pooled": Stats::new(pooled).to_json(WRITE_P95_MS, true),
            "worst_cycle_p95_ms": r2(worst),
        }));
    }

    // --- Run B: the full segment (AC-53, AC-52, AC-41) ---------------------
    control(&mut ctl, "reset", json!({})).await?;
    control(&mut ctl, "live", json!({})).await?;

    let churn_start = Instant::now() + Duration::from_millis(300);
    let _ = stage_tx.send(Stage::Churn { start: churn_start, secs: args.churn_secs });
    let churn_dones = collect(&mut done_rx, n, &mut errors).await;
    let mut churn_writes = Vec::new();
    let mut churn_lags = Vec::new();
    let mut finals: Vec<(usize, u8, u32)> = Vec::new();
    for d in &churn_dones {
        errors += d.errors;
        for w in &d.writes {
            churn_writes.push(w.ms);
            churn_lags.push(w.lag_ms);
        }
        if let Some((l, s)) = d.final_answer {
            finals.retain(|(i, _, _)| *i != d.id);
            finals.push((d.id, l, s));
        }
    }

    let seg_burst_start = Instant::now() + Duration::from_millis(300);
    let seg_shape = args.shapes.first().copied().unwrap_or(Shape::Uniform);
    let _ = stage_tx.send(Stage::Burst {
        cycle: cycle_counter,
        shape: seg_shape,
        window_ms: args.window_ms,
        start: seg_burst_start,
    });
    let burst_dones = collect(&mut done_rx, n, &mut errors).await;
    let mut seg_burst_writes = Vec::new();
    let mut seg_burst_lags = Vec::new();
    for d in &burst_dones {
        errors += d.errors;
        for w in &d.writes {
            seg_burst_writes.push(w.ms);
            seg_burst_lags.push(w.lag_ms);
        }
        if let Some((l, s)) = d.final_answer {
            finals.retain(|(i, _, _)| *i != d.id);
            finals.push((d.id, l, s));
        }
    }
    all_send_lags.extend(churn_lags.iter().copied());
    all_send_lags.extend(seg_burst_lags.iter().copied());

    let ok = control(&mut ctl, "close", json!({})).await?;
    let frozen = tally_from_control(&ok);
    let segment_recon = reconcile(&expected_from(&finals), &frozen, n);

    // AC-53's population is accepted answer writes only: churn plus burst.
    // Post-close refusals are a correctness probe and never a latency sample.
    let mut segment_writes = churn_writes.clone();
    segment_writes.extend(seg_burst_writes.iter().copied());
    let expected_population = churn_writes.len() + n;
    let segment_population_ok = segment_writes.len() == expected_population;

    // SPEC.md §4.3, then §4.4: refusal restates the saved answer, and the frozen
    // totals do not move afterwards.
    let _ = stage_tx.send(Stage::PostClose);
    let pc = collect(&mut done_rx, n, &mut errors).await;
    let refused = pc.iter().filter(|d| d.post_close_refused).count();
    let restated = pc.iter().filter(|d| d.post_close_restated).count();
    for d in &pc {
        errors += d.errors;
    }
    let after = tally_from_control(&control(&mut ctl, "totals", json!({})).await?);
    let totals_unchanged = after.totals == frozen.totals && after.answered == frozen.answered;
    let post_close_ok = refused == n && restated == n && totals_unchanged;

    // --- Reveal fan-out (AC-41) --------------------------------------------
    let _ = stage_tx.send(Stage::AwaitReveals { count: args.reveals });
    tokio::time::sleep(Duration::from_millis(300)).await;
    let mut issued: Vec<f64> = Vec::new();
    for r in 0..args.reveals {
        let at = Instant::now();
        issued.push(since_epoch_ms(at));
        control(&mut ctl, "reveal", json!({"reveal_id": r, "pad_bytes": args.reveal_bytes}))
            .await?;
        if r + 1 < args.reveals {
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }
    let reveal_dones = collect(&mut done_rx, n, &mut errors).await;

    let mut per_reveal: Vec<Vec<f64>> = vec![Vec::new(); args.reveals];
    let mut receipts: u64 = 0;
    for d in &reveal_dones {
        errors += d.errors;
        for (rid, at_ms) in &d.reveals {
            let idx = *rid as usize;
            if idx < per_reveal.len() {
                per_reveal[idx].push(at_ms - issued[idx]);
                receipts += 1;
            }
        }
    }
    let expected_receipts = (n * args.reveals) as u64;
    let missing = expected_receipts.saturating_sub(receipts);

    let reveals_json: Vec<Value> = per_reveal
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let st = Stats::new(v.clone());
            let spread = if st.n > 0 { st.max - st.samples[0] } else { f64::NAN };
            json!({
                "reveal_id": i,
                "n": st.n,
                "p50_ms": r2(st.p50),
                "p95_ms": r2(st.p95),
                "max_ms": r2(st.max),
                // Skew-free and path-free: last receipt minus first. Isolates the
                // server's broadcast loop from the network between us and it.
                "spread_ms": r2(spread),
                "missing": n.saturating_sub(st.n),
            })
        })
        .collect();
    let reveal_pooled = Stats::new(per_reveal.concat());

    let _ = stage_tx.send(Stage::Stop);

    // --- Verdict ------------------------------------------------------------
    let segment_stats = Stats::new(segment_writes);
    let send_lag_stats = Stats::new(all_send_lags);
    let nofile = ulimit_nofile();

    let vin = VerdictInput {
        burst_headline_p95: burst_headline,
        segment_write_p95: segment_stats.p95,
        reveal_p95: reveal_pooled.p95,
        ac52_ok: segment_recon.matches(),
        burst_reconciled: burst_cycles_reconciled,
        post_close_ok,
        clients_connected: connected,
        clients_expected: args.clients,
        send_lag_p95: send_lag_stats.p95,
        ulimit_nofile: nofile,
        distinct_instances: instances.len(),
        errors,
        missing_reveal_receipts: missing,
        segment_population_ok,
        burst_marginal: burst_headline_marginal,
        segment_marginal: segment_stats.marginal(WRITE_P95_MS, true),
        reveal_marginal: reveal_pooled.marginal(REVEAL_P95_MS, false),
    };
    let (code, notes) = verdict(&vin);

    // An invalid run's criteria are not judged at all. Printing `pass: true`
    // beside an exit code of 2 invites exactly the misreading the exit codes
    // exist to prevent, so each criterion's verdict is null and the report says
    // at the top level that none of its numbers may be quoted.
    let quotable = code != Exit::Invalid;
    let judged = |b: bool| if quotable { Value::Bool(b) } else { Value::Null };
    let ac52_pass = segment_recon.matches() && burst_cycles_reconciled && post_close_ok;

    let report = json!({
        "schema": "rustnyc-popquiz/burst-report/2",
        "ticket": "T-03",
        "started_at_unix_ms": started_at,
        "invocation": invocation,
        "warmup": args.warmup,
        "client": {
            "cpus": std::thread::available_parallelism().map(|p| p.get()).unwrap_or(0),
            "seed": args.seed,
            "clients_requested": args.clients,
            "clients_connected": connected,
            "tls": "rustls-ring",
            "nodelay": true,
            "ulimit_nofile": nofile,
            "reveal_bytes": args.reveal_bytes,
            "note": "run from the laptop; the client-side network is the laptop's. AC-55's oracle is venue wifi at HC-4 and is out of scope here.",
        },
        "server": {
            "url": args.url,
            "instances_seen": instances,
        },
        "runs": {
            "burst_only": {
                "shapes": shape_blocks,
                "headline_p95_ms": r2(burst_headline),
                "headline_marginal": burst_headline_marginal,
                "all_cycles_reconciled": burst_cycles_reconciled,
                "headline_note": "the worst cycle p95 across every shape run. AC-54 calls the burst the highest-risk moment; the gentler shape alone must not decide it.",
            },
            "segment": {
                "churn_s": args.churn_secs,
                "burst_window_ms": args.window_ms,
                "writes": segment_stats.to_json(WRITE_P95_MS, true),
                "writes_population": {
                    "churn": churn_writes.len(),
                    "burst": seg_burst_writes.len(),
                    "expected": expected_population,
                    "ok": segment_population_ok,
                    "note": "accepted answer writes only. Post-close refusals are a correctness probe, never a latency sample.",
                },
                "burst_subset": Stats::new(seg_burst_writes).to_json(WRITE_P95_MS, true),
                "churn_subset": Stats::new(churn_writes).to_json(WRITE_P95_MS, true),
                "reconcile": segment_recon.to_json(),
                "post_close": {
                    "n": pc.len(),
                    "all_refused": refused == n,
                    "saved_answer_restated": restated == n,
                    "totals_unchanged": totals_unchanged,
                },
                "reveals": reveals_json,
                "reveal_pooled": reveal_pooled.to_json(REVEAL_P95_MS, false),
            },
        },
        "diagnostics": {
            "connect_ms": connect_stats.to_json(f64::INFINITY, true),
            "join_rtt_ms": join_stats.to_json(f64::INFINITY, true),
            "send_lag_ms": send_lag_stats.to_json(SEND_LAG_P95_INVALID_MS, true),
            "errors": errors,
            "missing_reveal_receipts": missing,
        },
        "quotable": quotable,
        "criteria": [
            {"id": "AC-54", "measure": "runs.burst_only.headline_p95_ms", "value": r2(burst_headline),
             "threshold_ms": WRITE_P95_MS, "comparison": "<",
             "pass": judged(burst_headline < WRITE_P95_MS),
             "marginal": burst_headline_marginal},
            {"id": "AC-53", "measure": "runs.segment.writes.p95_ms", "value": r2(segment_stats.p95),
             "threshold_ms": WRITE_P95_MS, "comparison": "<",
             "pass": judged(segment_stats.p95 < WRITE_P95_MS),
             "marginal": segment_stats.marginal(WRITE_P95_MS, true)},
            {"id": "AC-41", "measure": "runs.segment.reveal_pooled.p95_ms", "value": r2(reveal_pooled.p95),
             "threshold_ms": REVEAL_P95_MS, "comparison": "<=",
             "pass": judged(reveal_pooled.p95 <= REVEAL_P95_MS),
             "marginal": reveal_pooled.marginal(REVEAL_P95_MS, false)},
            // AC-52 is three things, all of which must hold: the segment's
            // reconciliation, every burst cycle's, and the post-close refusal
            // (SPEC.md §4.3). Naming one field here while judging on three would
            // let a reader recompute a pass that the run did not earn.
            {"id": "AC-52",
             "measure": ["runs.segment.reconcile.match",
                         "runs.burst_only.all_cycles_reconciled",
                         "runs.segment.post_close.{all_refused,saved_answer_restated,totals_unchanged}"],
             "value": ac52_pass,
             "threshold_ms": Value::Null, "comparison": "exact",
             "pass": judged(ac52_pass),
             "marginal": false},
        ],
        "verdict": {
            "pass": code == Exit::Pass,
            "exit_code": code as i32,
            "notes": notes,
        },
    });

    Ok((report, code))
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn passing() -> VerdictInput {
        VerdictInput {
            burst_headline_p95: 120.0,
            segment_write_p95: 100.0,
            reveal_p95: 300.0,
            ac52_ok: true,
            post_close_ok: true,
            clients_connected: 200,
            clients_expected: 200,
            send_lag_p95: 1.0,
            ulimit_nofile: 1_048_576,
            distinct_instances: 1,
            errors: 0,
            missing_reveal_receipts: 0,
            segment_population_ok: true,
            burst_reconciled: true,
            burst_marginal: false,
            segment_marginal: false,
            reveal_marginal: false,
        }
    }

    #[test]
    fn p95_of_1_to_200_is_the_190th() {
        let s = Stats::new((1..=200).map(|v| v as f64).collect());
        assert_eq!(s.n, 200);
        assert_eq!(s.p95_rank, 190);
        assert_eq!(s.p95, 190.0);
        // 10 samples strictly above it — ranks 191..=200. 11 sit at or above.
        assert_eq!(s.samples.iter().filter(|v| **v > s.p95).count(), 10);
        assert_eq!(s.samples.iter().filter(|v| **v >= s.p95).count(), 11);
    }

    #[test]
    fn p95_ci_ranks_at_n200_round_outward_to_183_and_197() {
        // sigma = sqrt(200*0.95*0.05) = sqrt(9.5) = 3.0822
        // 190 +/- 1.96*3.0822 = [183.96, 196.04]; outward -> [183, 197].
        assert_eq!(ci_ranks(200, 0.95, 190), (183, 197));
    }

    #[test]
    fn the_p95_rank_comes_from_the_actual_n_and_is_never_hardcoded() {
        // The segment population is churn + burst. With 1+(i mod 3) churn writes
        // over 200 clients that is 399, plus 200 burst writes = 599 — not the
        // ~800 an earlier draft assumed, and so rank 570, not 760.
        let churn: usize = (0..200).map(|i| 1 + (i % 3)).sum();
        assert_eq!(churn, 399);
        assert_eq!(Stats::new((1..=599).map(|v| v as f64).collect()).p95_rank, 570);
        // And the function tracks n rather than any constant.
        assert_eq!(Stats::new((1..=1000).map(|v| v as f64).collect()).p95_rank, 950);
        assert_eq!(Stats::new((1..=600).map(|v| v as f64).collect()).p95_rank, 570);
    }

    #[test]
    fn marginal_fires_when_the_upper_ci_crosses_the_threshold() {
        // 190 fast, 10 slow: p95 lands under the line while the upper CI bound
        // (rank 197) is one of the slow ones.
        let mut s: Vec<f64> = vec![100.0; 190];
        s.extend(vec![900.0; 10]);
        let st = Stats::new(s);
        assert!(st.p95 < WRITE_P95_MS, "p95 should pass");
        assert!(st.marginal(WRITE_P95_MS, true), "but it must not be called a clean pass");

        let clean = Stats::new(vec![10.0; 200]);
        assert!(!clean.marginal(WRITE_P95_MS, true));
    }

    #[test]
    fn verdict_thresholds_are_the_ones_the_criteria_state() {
        assert_eq!(verdict(&passing()).0, Exit::Pass);

        // AC-53/54 say "under 500ms", so 500 exactly is a miss.
        assert_eq!(
            verdict(&VerdictInput { burst_headline_p95: 500.0, ..passing() }).0,
            Exit::Missed
        );
        assert_eq!(
            verdict(&VerdictInput { burst_headline_p95: 499.9, ..passing() }).0,
            Exit::Pass
        );
        assert_eq!(
            verdict(&VerdictInput { segment_write_p95: 501.0, ..passing() }).0,
            Exit::Missed
        );

        // AC-41 says "within 2 seconds", so 2000 exactly passes.
        assert_eq!(verdict(&VerdictInput { reveal_p95: 2000.0, ..passing() }).0, Exit::Pass);
        assert_eq!(verdict(&VerdictInput { reveal_p95: 2000.1, ..passing() }).0, Exit::Missed);

        assert_eq!(verdict(&VerdictInput { ac52_ok: false, ..passing() }).0, Exit::Missed);
        assert_eq!(verdict(&VerdictInput { post_close_ok: false, ..passing() }).0, Exit::Missed);
    }

    #[test]
    fn an_invalid_run_outranks_a_miss() {
        // A run whose harness misbehaved has no numbers worth reading either
        // way — it must not be reported as a failing server.
        let both =
            VerdictInput { send_lag_p95: 26.0, burst_headline_p95: 900.0, ..passing() };
        assert_eq!(verdict(&both).0, Exit::Invalid);

        for bad in [
            VerdictInput { send_lag_p95: 26.0, ..passing() },
            VerdictInput { distinct_instances: 2, ..passing() },
            VerdictInput { missing_reveal_receipts: 1, ..passing() },
            VerdictInput { errors: 1, ..passing() },
            VerdictInput { clients_connected: 199, ..passing() },
            VerdictInput { ulimit_nofile: 256, ..passing() },
            VerdictInput { segment_population_ok: false, ..passing() },
        ] {
            assert_eq!(verdict(&bad).0, Exit::Invalid);
        }
    }

    #[test]
    fn reconcile_detects_a_lost_write_a_stale_write_and_a_swap() {
        let finals = [(0usize, 0u8, 3u32), (1, 1, 4), (2, 1, 5)];
        let expected = expected_from(&finals);
        assert!(reconcile(&expected, &expected.clone(), 3).matches());

        let mut lost = expected.clone();
        lost.totals[1] -= 1;
        assert!(!reconcile(&expected, &lost, 3).per_letter_match);

        // A stale write winning: totals identical, seq sum wrong.
        let mut stale = expected.clone();
        stale.seq_sum -= 1;
        assert!(!reconcile(&expected, &stale, 3).seq_sum_match);

        // The case only the fingerprint sees. Sessions 1 and 2 both hold letter
        // B, so exchanging their seqs leaves totals, answered, sum and even the
        // pooled seq sum untouched.
        let swapped_finals = [(0usize, 0u8, 3u32), (1, 1, 5), (2, 1, 4)];
        let swapped = expected_from(&swapped_finals);
        let r = reconcile(&expected, &swapped, 3);
        assert!(r.per_letter_match && r.answered_match && r.sum_match && r.seq_sum_match);
        assert!(!r.fingerprint_match, "the fingerprint is the only check that sees a swap");
        assert!(!r.matches());
    }

    #[test]
    fn ws_target_derives_the_scheme_and_port() {
        assert_eq!(ws_target("https://a.fly.dev").unwrap(), ("wss", "a.fly.dev".into(), 443));
        assert_eq!(ws_target("http://127.0.0.1:8080").unwrap(), ("ws", "127.0.0.1".into(), 8080));
        assert_eq!(ws_target("https://a.fly.dev/").unwrap(), ("wss", "a.fly.dev".into(), 443));
        assert!(ws_target("a.fly.dev").is_err());
    }

    #[test]
    fn the_spike_shape_stays_inside_the_window_and_uniform_spreads() {
        let mut uniform_seen = std::collections::HashSet::new();
        for id in 0..200 {
            let spike = burst_offset_ms(7, 0, id, Shape::Spike, 2000);
            assert!((1950..2000).contains(&spike), "spike offset {spike} left the last 50 ms");
            let uni = burst_offset_ms(7, 0, id, Shape::Uniform, 2000);
            assert!(uni < 2000);
            uniform_seen.insert(uni / 200); // which tenth of the window
        }
        // The uniform shape should touch most tenths of the window; a generator
        // that bunched them would quietly turn AC-54 into a different test.
        assert!(uniform_seen.len() >= 8, "uniform offsets covered only {} tenths", uniform_seen.len());
    }

    #[test]
    fn churn_offsets_are_monotonic_and_inside_the_window() {
        // Regression. Unsorted, a client sleeping to 2800 ms and then to 500 ms
        // booked 2300 ms of "send lag" against the server — loopback send-lag
        // p95 hit 481.9 ms and every run came back invalid.
        for id in 0..200 {
            let count = 1 + (id % 3);
            let offs = churn_offsets(11, id, count, 20);
            assert_eq!(offs.len(), count);
            assert!(offs.windows(2).all(|w| w[0] <= w[1]), "not monotonic for id {id}: {offs:?}");
            assert!(offs.iter().all(|o| *o < 20_000));
        }
        // The counts the segment population arithmetic depends on.
        let total: usize = (0..200).map(|i| churn_offsets(11, i, 1 + (i % 3), 20).len()).sum();
        assert_eq!(total, 399);
    }

    #[test]
    fn parse_args_reads_the_flags_t21_will_pass() {
        let a = parse_args(&[
            "--url".into(),
            "https://x.fly.dev".into(),
            "--clients".into(),
            "200".into(),
            "--seed".into(),
            "42".into(),
            "--out".into(),
            "r.json".into(),
        ])
        .unwrap();
        assert_eq!(a.url, "https://x.fly.dev");
        assert_eq!(a.clients, 200);
        assert_eq!(a.seed, 42);
        assert_eq!(a.out.as_deref(), Some("r.json"));
        assert_eq!(a.shapes, vec![Shape::Uniform, Shape::Spike]);

        assert!(parse_args(&["--burst-shape".into(), "sideways".into()]).is_err());
        assert!(parse_args(&["--nope".into()]).is_err());

        let w = parse_args(&["--warmup".into()]).unwrap();
        assert_eq!(w.cycles, 1);
        assert_eq!(w.shapes, vec![Shape::Uniform]);
    }

    #[test]
    fn a_marginal_pass_is_said_in_words_and_never_as_a_clean_pass() {
        let (code, notes) = verdict(&VerdictInput { burst_marginal: true, ..passing() });
        // Marginal is not a miss: the exit code stays 0.
        assert_eq!(code, Exit::Pass);
        assert!(notes.iter().any(|n| n.starts_with("AC-54 MARGINAL")), "{notes:?}");
        assert!(
            !notes.iter().any(|n| n == "all four criteria pass as measured"),
            "a marginal run must not carry the clean-pass sentence: {notes:?}"
        );

        // On a miss the miss is the story; marginal is not reported beside it.
        let (code, notes) = verdict(&VerdictInput {
            burst_marginal: true,
            burst_headline_p95: 600.0,
            ..passing()
        });
        assert_eq!(code, Exit::Missed);
        assert!(!notes.iter().any(|n| n.contains("MARGINAL")), "{notes:?}");
    }

    #[test]
    fn a_miss_anywhere_silences_marginal_notes_everywhere() {
        // The cross-criterion case: AC-52 misses while AC-53 passes marginally.
        // The run is a miss, and the notes must say only that.
        let (code, notes) = verdict(&VerdictInput {
            ac52_ok: false,
            segment_marginal: true,
            ..passing()
        });
        assert_eq!(code, Exit::Missed);
        assert!(notes.iter().any(|n| n.starts_with("AC-52 MISS")), "{notes:?}");
        assert!(!notes.iter().any(|n| n.contains("MARGINAL")), "{notes:?}");
    }

    #[test]
    fn a_lost_write_in_any_burst_cycle_is_an_ac52_miss() {
        let (code, notes) = verdict(&VerdictInput { burst_reconciled: false, ..passing() });
        assert_eq!(code, Exit::Missed);
        assert!(notes.iter().any(|n| n.contains("burst-only cycle")), "{notes:?}");
    }

    // --- Loopback: the real server, in-process, on 127.0.0.1:0 ----------------
    //
    // These prove the harness and the server's semantics end to end. They never
    // produce a headline number: loopback has no network, and EVALUATION.md's
    // AC-53 row requires the deployed substrate for that.

    async fn serve_loopback(capacity: usize) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = server::app("loopback".into(), "local".into(), "spike-test".into(), capacity);
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        format!("http://{addr}")
    }

    fn small_run(url: String, clients: usize) -> Arc<Args> {
        Arc::new(Args {
            url,
            clients,
            seed: 7,
            cycles: 1,
            churn_secs: 1,
            window_ms: 300,
            reveals: 2,
            settle_ms: 100,
            ..Args::default()
        })
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn loopback_segment_end_to_end() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let clients = 12;
        let url = serve_loopback(clients).await;
        let (report, code) = run(small_run(url, clients), "test".into()).await.unwrap();
        let seg = &report["runs"]["segment"];

        assert_eq!(code, Exit::Pass, "verdict: {}", report["verdict"]);
        assert_eq!(report["quotable"], true);

        // Every client connected, and all of them met one server.
        assert_eq!(report["client"]["clients_connected"], clients);
        assert_eq!(report["server"]["instances_seen"], json!(["loopback"]));

        // AC-52: last write wins across churn and burst, the frozen totals
        // reconcile exactly — per letter, count, seq sum and fingerprint — and so
        // does every burst-only cycle.
        assert_eq!(seg["reconcile"]["match"], true, "{}", seg["reconcile"]);
        assert_eq!(seg["reconcile"]["answered"], clients);
        assert_eq!(report["runs"]["burst_only"]["all_cycles_reconciled"], true);

        // SPEC.md §4.3 and §4.4: every post-close write is refused with the
        // saved answer restated, and the frozen totals do not move.
        assert_eq!(seg["post_close"]["all_refused"], true);
        assert_eq!(seg["post_close"]["saved_answer_restated"], true);
        assert_eq!(seg["post_close"]["totals_unchanged"], true);

        // AC-53's population is churn plus burst, and refusals are not in it.
        let churn: usize = (0..clients).map(|i| 1 + (i % 3)).sum();
        assert_eq!(seg["writes"]["n"], churn + clients);
        assert_eq!(seg["writes_population"]["ok"], true);

        // AC-41: every reveal reached every client.
        assert_eq!(report["diagnostics"]["missing_reveal_receipts"], 0);
        assert_eq!(seg["reveal_pooled"]["n"], clients * 2);

        // Both burst shapes ran, and the headline is the worse of the two.
        let shapes = report["runs"]["burst_only"]["shapes"].as_array().unwrap();
        assert_eq!(shapes.len(), 2);
        let worst = shapes
            .iter()
            .map(|s| s["worst_cycle_p95_ms"].as_f64().unwrap())
            .fold(f64::MIN, f64::max);
        assert_eq!(report["runs"]["burst_only"]["headline_p95_ms"].as_f64().unwrap(), worst);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn the_join_past_capacity_is_refused_full() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let capacity = 12;
        let url = serve_loopback(capacity).await;

        // Hold `capacity` joined sessions open, then try one more.
        let mut held = Vec::new();
        for i in 0..=capacity {
            let mut ws = connect(&url).await.unwrap();
            let _ = ws.next().await; // hello
            let join = json!({"t": "join", "session": session_id(i)}).to_string();
            ws.send(Message::Text(join.into())).await.unwrap();
            let Some(Ok(Message::Text(t))) = ws.next().await else { panic!("no reply to join {i}") };
            let v: Value = serde_json::from_str(&t).unwrap();
            if i < capacity {
                assert_eq!(v["t"], "joined", "join {i} of {capacity}: {v}");
            } else {
                // SPEC.md §4.1 / AC-30: refused, not queued, and nothing reserved.
                assert_eq!(v["t"], "refused", "join past capacity: {v}");
                assert_eq!(v["reason"], "full");
            }
            held.push(ws);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn report_round_trips_and_carries_schema_v1() {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let clients = 12;
        let url = serve_loopback(clients).await;
        let (report, _) = run(small_run(url, clients), "test".into()).await.unwrap();

        // What HC-0 will read back: serialised and parsed again, nothing lost.
        let text = serde_json::to_string_pretty(&report).unwrap();
        let back: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(back, report);
        assert_eq!(back["schema"], "rustnyc-popquiz/burst-report/2");
        assert_eq!(back["ticket"], "T-03");

        // Four criteria, in a fixed order, each judged, each carrying `marginal`.
        let criteria = back["criteria"].as_array().unwrap();
        let ids: Vec<&str> = criteria.iter().map(|c| c["id"].as_str().unwrap()).collect();
        assert_eq!(ids, ["AC-54", "AC-53", "AC-41", "AC-52"]);
        for c in criteria {
            assert!(c.get("marginal").is_some(), "{} has no marginal field", c["id"]);
            assert!(c["pass"].is_boolean(), "{} is not judged on a valid run", c["id"]);
        }

        // The three latency criteria name a field, and that field holds the value
        // they report — so a reader can find every headline where it says it is.
        for c in &criteria[..3] {
            let mut node = &back;
            for key in c["measure"].as_str().unwrap().split('.') {
                node = &node[key];
            }
            assert_eq!(node, &c["value"], "{} measure does not resolve to its value", c["id"]);
        }

        // Raw samples ship, one per measured write, so a percentile can be
        // recomputed without trusting this code.
        let seg = &back["runs"]["segment"]["writes"];
        assert_eq!(seg["samples_ms"].as_array().unwrap().len(), seg["n"].as_u64().unwrap() as usize);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn an_invalid_run_leaves_its_criteria_unjudged() {
        // A genuinely invalid run, through the real code path: 13 clients asked
        // for against a room that holds 12, so one join is refused and the run
        // is not the run that was requested. This is run 1's failure, on purpose.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let url = serve_loopback(12).await;
        let (report, code) = run(small_run(url, 13), "test".into()).await.unwrap();

        assert_eq!(code, Exit::Invalid, "verdict: {}", report["verdict"]);
        assert_eq!(report["client"]["clients_connected"], 12);
        assert_eq!(report["quotable"], false);

        // Its numbers exist and may well be good, but none is judged. A report
        // that printed `pass: true` beside exit 2 would invite exactly the
        // misreading the exit codes are there to stop.
        for c in report["criteria"].as_array().unwrap() {
            assert!(c["pass"].is_null(), "{} was judged on an invalid run: {c}", c["id"]);
        }
        let notes = report["verdict"]["notes"].as_array().unwrap();
        assert!(notes.iter().any(|n| n.as_str().unwrap().contains("12 of 13")), "{notes:?}");
    }
}
