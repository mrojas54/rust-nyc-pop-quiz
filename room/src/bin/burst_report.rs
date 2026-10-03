//! `burst`'s numbers and its verdict (T-21): percentiles that carry their own
//! uncertainty, the pass / miss / invalid decision, and the JSON report.
//!
//! Pure — samples in, a report and an exit code out, no sockets and no clock —
//! so `just test` can hold it to its decisions on recorded samples
//! (`tests/burst_report.rs`) while the client that gathers the samples stays
//! behind the `burst` feature. Not a binary (`autobins = false`); `burst.rs`
//! and that test include it with `#[path]`, so nothing here is in the room's
//! library or its image.
//!
//! Carried over from the T-03 spike's client (`spike-burst.rs`), whose rules
//! it keeps: every raw sample ships in the report; p95 travels with its rank,
//! the rank's 95% interval (rounded outward) and the max; a pass whose upper
//! bound crosses the threshold is `marginal` in words, never a clean pass; a
//! run that did not happen properly is *invalid* (exit 2) and none of its
//! criteria are judged, which is a different thing from a miss (exit 1).

#![allow(dead_code)]

use serde_json::{json, Value};

/// AC-53, AC-54: "under 500ms at p95". Strictly under.
pub const WRITE_P95_MS: f64 = 500.0;
/// AC-41: "within 2 seconds at p95". At most, so exactly 2000 passes.
pub const REVEAL_P95_MS: f64 = 2000.0;
/// The client's own scheduling delay. Above this, the harness rather than the
/// room is what got measured, and the run is invalid rather than failing.
pub const SEND_LAG_P95_INVALID_MS: f64 = 25.0;
/// Below this many open files, 200 participants cannot all connect.
pub const MIN_NOFILE: u64 = 1024;
pub const SCHEMA: &str = "rustnyc-popquiz/burst-report/room-1";

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

/// A percentile summary that carries its own uncertainty.
///
/// p95 at n=200 is a single order statistic — one slow client moves it — so
/// the rank, its confidence interval and the max all travel beside the number.
#[derive(Debug, Clone)]
pub struct Stats {
    pub n: usize,
    pub p50: f64,
    pub p95: f64,
    pub p95_rank: usize,
    pub p95_ci_rank: (usize, usize),
    pub p95_ci: (f64, f64),
    pub p99: f64,
    pub max: f64,
    pub samples: Vec<f64>,
}

/// Nearest-rank: `x_(ceil(q*n))`, ascending, 1-based.
pub fn nearest_rank(sorted: &[f64], q: f64) -> (usize, f64) {
    let n = sorted.len();
    if n == 0 {
        return (0, f64::NAN);
    }
    let rank = ((q * n as f64).ceil() as usize).clamp(1, n);
    (rank, sorted[rank - 1])
}

/// The rank's 95% interval, rounded **outward**: `rank ± 1.96·sqrt(n·q·(1-q))`,
/// the low end floored and the high end ceiled, so `marginal` fires more
/// often, never less.
pub fn ci_ranks(n: usize, q: f64, rank: usize) -> (usize, usize) {
    if n == 0 {
        return (0, 0);
    }
    let sigma = (n as f64 * q * (1.0 - q)).sqrt();
    let lo = ((rank as f64) - 1.96 * sigma).floor().max(1.0) as usize;
    let hi = (((rank as f64) + 1.96 * sigma).ceil() as usize).min(n);
    (lo, hi)
}

impl Stats {
    pub fn new(mut samples: Vec<f64>) -> Self {
        samples.sort_by(f64::total_cmp);
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

    /// A pass whose upper confidence bound crosses the threshold. Not a
    /// failure, but never reported as a clean pass either.
    pub fn marginal(&self, threshold: f64, strictly_under: bool) -> bool {
        if self.n == 0 {
            return false;
        }
        let passes = if strictly_under { self.p95 < threshold } else { self.p95 <= threshold };
        let crosses = if strictly_under { self.p95_ci.1 >= threshold } else { self.p95_ci.1 > threshold };
        passes && crosses
    }

    pub fn to_json(&self, threshold: f64, strictly_under: bool) -> Value {
        json!({
            "n": self.n,
            "p50_ms": r2(self.p50),
            "p95_ms": r2(self.p95),
            "p95_rank": self.p95_rank,
            "p95_ci_rank": [self.p95_ci_rank.0, self.p95_ci_rank.1],
            "p95_ci_ms": [r2(self.p95_ci.0), r2(self.p95_ci.1)],
            "p99_ms": r2(self.p99),
            "max_ms": r2(self.max),
            "marginal": self.marginal(threshold, strictly_under),
            "samples_ms": self.samples.iter().map(|s| r2(*s)).collect::<Vec<_>>(),
        })
    }
}

/// Two decimals; a non-finite number (no samples) is `-1`, never `NaN` in JSON.
pub fn r2(v: f64) -> f64 {
    if v.is_finite() {
        (v * 100.0).round() / 100.0
    } else {
        -1.0
    }
}

// ---------------------------------------------------------------------------
// The verdict
// ---------------------------------------------------------------------------

/// Splitting "a criterion missed" from "the run is invalid" is what stops a
/// broken harness being read as a failing room, or a failing room being
/// excused as a broken harness.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Exit {
    Pass = 0,
    Missed = 1,
    Invalid = 2,
}

#[derive(Debug, Default, Clone)]
pub struct VerdictInput {
    pub burst_headline_p95: f64,
    pub segment_write_p95: f64,
    pub reveal_p95: f64,
    /// The split's totals equal the final answers exactly, `answered` and the
    /// sum are N.
    pub ac52_totals_ok: bool,
    /// Every write while live came back `200` naming its own letter, and the
    /// host's live count was N after the isolated bursts: each session counted
    /// once.
    pub bursts_acked: bool,
    /// After close, every session's write was refused with *its own* final
    /// letter restated — the per-session check, which a swap between two
    /// sessions fails although every total survives it.
    pub post_close_ok: bool,
    pub clients_connected: usize,
    pub clients_expected: usize,
    pub send_lag_p95: f64,
    pub ulimit_nofile: u64,
    pub errors: u64,
    pub missing_reveal_receipts: u64,
    pub segment_population_ok: bool,
    pub burst_marginal: bool,
    pub segment_marginal: bool,
    pub reveal_marginal: bool,
}

pub fn verdict(v: &VerdictInput) -> (Exit, Vec<String>) {
    let mut notes = Vec::new();
    let mut invalid = false;

    // Invalidity is decided first: a run that did not happen properly has no
    // numbers worth reading, pass or fail.
    if v.clients_connected != v.clients_expected {
        notes.push(format!("RUN INVALID: {} of {} participants joined and attached", v.clients_connected, v.clients_expected));
        invalid = true;
    }
    if v.ulimit_nofile < MIN_NOFILE {
        notes.push(format!("RUN INVALID: ulimit -n is {}, under {MIN_NOFILE}", v.ulimit_nofile));
        invalid = true;
    }
    if v.errors > 0 {
        notes.push(format!(
            "RUN INVALID: {} harness errors (a connect, a write that got no answer, a socket that never saw a phase)",
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
            "RUN INVALID: client send lag p95 {:.1} ms exceeds {:.0} ms — the client machine, not the room, is what was measured",
            v.send_lag_p95, SEND_LAG_P95_INVALID_MS
        ));
        invalid = true;
    }
    if !v.segment_population_ok {
        notes.push(
            "RUN INVALID: the segment's write samples are not the writes it scheduled — post-close refusals must never be latency samples"
                .into(),
        );
        invalid = true;
    }
    if invalid {
        return (Exit::Invalid, notes);
    }

    let mut missed = false;
    if !(v.burst_headline_p95 < WRITE_P95_MS) {
        notes.push(format!("AC-54 MISS: deadline-burst write p95 {:.1} ms, threshold < {:.0} ms", v.burst_headline_p95, WRITE_P95_MS));
        missed = true;
    }
    if !(v.segment_write_p95 < WRITE_P95_MS) {
        notes.push(format!("AC-53 MISS: segment write p95 {:.1} ms, threshold < {:.0} ms", v.segment_write_p95, WRITE_P95_MS));
        missed = true;
    }
    if v.reveal_p95.is_nan() || v.reveal_p95 > REVEAL_P95_MS {
        notes.push(format!("AC-41 MISS: reveal fan-out p95 {:.1} ms, threshold <= {:.0} ms", v.reveal_p95, REVEAL_P95_MS));
        missed = true;
    }
    if !v.ac52_totals_ok {
        notes.push("AC-52 MISS: the split's totals are not the final answers exactly".into());
        missed = true;
    }
    if !v.bursts_acked {
        notes.push("AC-52 MISS: a write while live was not saved as sent, or the host's live count after the isolated bursts was not one per session".into());
        missed = true;
    }
    if !v.post_close_ok {
        notes.push("AC-52 MISS: after close, a session's write was accepted or did not restate that session's own final answer (SPEC §4.3)".into());
        missed = true;
    }

    // Marginal is reported on a pass only: on a miss, the miss is the story —
    // including when the miss is a *different* criterion.
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
// The report
// ---------------------------------------------------------------------------

/// One isolated deadline burst: every participant wrote once inside the window.
#[derive(Debug, Clone, Default)]
pub struct Cycle {
    pub shape: &'static str,
    pub index: usize,
    pub writes_ms: Vec<f64>,
    /// Every write came back `200` with `saved` equal to the letter sent.
    pub acked: bool,
}

/// Everything a run measured. Samples are milliseconds, raw.
#[derive(Debug, Clone, Default)]
pub struct Measured {
    pub invocation: String,
    pub started_at_unix_ms: u64,
    pub url: String,
    pub loopback: bool,
    pub question: String,
    pub seed: u64,
    pub participants: usize,
    pub connected: usize,
    pub window_ms: u64,
    pub churn_secs: u64,
    pub cycles: Vec<Cycle>,
    /// The host's live `answered` after the isolated bursts (N if each session
    /// was counted once).
    pub host_answered_after_bursts: Option<u64>,
    pub churn_ms: Vec<f64>,
    pub deadline_ms: Vec<f64>,
    /// Churn and deadline writes the room answered but did not save as sent
    /// (anything but `200 {saved: <the letter>}`).
    pub unacked_writes: usize,
    /// How many writes the live phase scheduled; every one must be a sample.
    pub scheduled_writes: usize,
    pub expected_totals: [u32; 5],
    pub split_totals: Option<[u32; 5]>,
    pub split_answered: Option<u64>,
    /// Buzzers whose split frame disagreed with the expected totals.
    pub split_disagreeing: usize,
    pub post_close_refused: usize,
    pub post_close_restated: usize,
    pub reveal_ms: Vec<f64>,
    pub connect_ms: Vec<f64>,
    pub join_ms: Vec<f64>,
    pub send_lag_ms: Vec<f64>,
    pub ulimit_nofile: u64,
    pub errors: Vec<String>,
}

pub fn substrate_note(loopback: bool) -> &'static str {
    if loopback {
        "LOOPBACK: the room was on this machine. This proves the harness and AC-52's exact count, AC-54's window and AC-41's fan-out mechanics; the latencies are this machine's, and a loopback run never meets AC-53's 'run against the deployed substrate' clause."
    } else {
        "DEPLOYED: run against the room at the URL above, over the public internet from this client. The client-side network is this client's link, not venue wifi: AC-55's oracle is venue wifi at HC-4."
    }
}

/// The report and the exit code. Written in full whatever the verdict, so a
/// failing or invalid run still leaves a complete artifact behind.
pub fn report(m: &Measured) -> (Value, Exit) {
    let n = m.participants;

    // AC-54's headline: the worst cycle's p95 across every shape run, so the
    // harder shape can never be outvoted by the gentler one.
    let cycle_stats: Vec<Stats> = m.cycles.iter().map(|c| Stats::new(c.writes_ms.clone())).collect();
    let worst = cycle_stats
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.p95.total_cmp(&b.1.p95))
        .map(|(i, _)| i);
    let (burst_headline, burst_marginal) = match worst {
        Some(i) => (cycle_stats[i].p95, cycle_stats[i].marginal(WRITE_P95_MS, true)),
        None => (f64::NAN, false),
    };

    let mut segment: Vec<f64> = m.cycles.iter().flat_map(|c| c.writes_ms.iter().copied()).collect();
    segment.extend(&m.churn_ms);
    segment.extend(&m.deadline_ms);
    let population_ok = segment.len() == m.scheduled_writes && m.deadline_ms.len() == m.connected;
    let segment_stats = Stats::new(segment);
    let reveal = Stats::new(m.reveal_ms.clone());
    let send_lag = Stats::new(m.send_lag_ms.clone());

    let bursts_acked = m.cycles.iter().all(|c| c.acked) && m.host_answered_after_bursts == Some(n as u64) && m.unacked_writes == 0;
    let sum: u32 = m.split_totals.map(|t| t.iter().sum()).unwrap_or(0);
    let totals_ok = m.split_totals == Some(m.expected_totals)
        && m.split_answered == Some(n as u64)
        && sum as usize == n
        && m.split_disagreeing == 0;
    let post_close_ok = m.post_close_refused == n && m.post_close_restated == n;

    let vin = VerdictInput {
        burst_headline_p95: burst_headline,
        segment_write_p95: segment_stats.p95,
        reveal_p95: reveal.p95,
        ac52_totals_ok: totals_ok,
        bursts_acked,
        post_close_ok,
        clients_connected: m.connected,
        clients_expected: n,
        send_lag_p95: if send_lag.n == 0 { 0.0 } else { send_lag.p95 },
        ulimit_nofile: m.ulimit_nofile,
        errors: m.errors.len() as u64,
        missing_reveal_receipts: m.connected.saturating_sub(reveal.n) as u64,
        segment_population_ok: population_ok,
        burst_marginal,
        segment_marginal: segment_stats.marginal(WRITE_P95_MS, true),
        reveal_marginal: reveal.marginal(REVEAL_P95_MS, false),
    };
    let (code, notes) = verdict(&vin);

    // An invalid run's criteria are not judged at all: `pass` is null, and
    // the report says at the top level that none of its numbers may be quoted.
    let quotable = code != Exit::Invalid;
    let judged = |b: bool| if quotable { Value::Bool(b) } else { Value::Null };
    let ac52_pass = totals_ok && bursts_acked && post_close_ok;
    let substrate = if m.loopback { "loopback" } else { "deployed" };

    let cycles_json: Vec<Value> = m
        .cycles
        .iter()
        .zip(&cycle_stats)
        .map(|(c, s)| {
            json!({
                "shape": c.shape,
                "cycle": c.index,
                "acked": c.acked,
                "writes": s.to_json(WRITE_P95_MS, true),
            })
        })
        .collect();

    let report = json!({
        "schema": SCHEMA,
        "ticket": "T-21",
        "substrate": substrate,
        "substrate_note": substrate_note(m.loopback),
        "started_at_unix_ms": m.started_at_unix_ms,
        "invocation": m.invocation,
        "room": {
            "url": m.url,
            "question": m.question,
        },
        "client": {
            "cpus": std::thread::available_parallelism().map(|p| p.get()).unwrap_or(0),
            "seed": m.seed,
            "participants_requested": n,
            "participants_connected": m.connected,
            "ulimit_nofile": m.ulimit_nofile,
        },
        "runs": {
            "burst_only": {
                "window_ms": m.window_ms,
                "cycles": cycles_json,
                "headline_p95_ms": r2(burst_headline),
                "headline_marginal": burst_marginal,
                "host_answered_after": m.host_answered_after_bursts,
                "headline_note": "the worst cycle's p95 across every shape run; each cycle is every participant writing once inside the window with nothing else in flight.",
            },
            "segment": {
                "churn_s": m.churn_secs,
                "burst_window_ms": m.window_ms,
                "writes": segment_stats.to_json(WRITE_P95_MS, true),
                "writes_population": {
                    "isolated_bursts": m.cycles.iter().map(|c| c.writes_ms.len()).sum::<usize>(),
                    "churn": m.churn_ms.len(),
                    "deadline_burst": m.deadline_ms.len(),
                    "not_saved_as_sent": m.unacked_writes,
                    "scheduled": m.scheduled_writes,
                    "ok": population_ok,
                    "note": "every accepted write while live. Post-close refusals are a correctness probe, never a latency sample.",
                },
                "deadline_subset": Stats::new(m.deadline_ms.clone()).to_json(WRITE_P95_MS, true),
                "churn_subset": Stats::new(m.churn_ms.clone()).to_json(WRITE_P95_MS, true),
                "reconcile": {
                    "expected_totals": m.expected_totals,
                    "split_totals": m.split_totals,
                    "answered": m.split_answered,
                    "sum": sum,
                    "buzzers_disagreeing": m.split_disagreeing,
                    "match": totals_ok,
                },
                "post_close": {
                    "n": n,
                    "refused": m.post_close_refused,
                    "own_answer_restated": m.post_close_restated,
                    "ok": post_close_ok,
                },
                "reveal": reveal.to_json(REVEAL_P95_MS, false),
            },
        },
        "diagnostics": {
            "connect_ms": Stats::new(m.connect_ms.clone()).to_json(f64::INFINITY, true),
            "join_ms": Stats::new(m.join_ms.clone()).to_json(f64::INFINITY, true),
            "send_lag_ms": send_lag.to_json(SEND_LAG_P95_INVALID_MS, true),
            "errors": m.errors.len(),
            "first_errors": m.errors.iter().take(10).collect::<Vec<_>>(),
            "missing_reveal_receipts": vin.missing_reveal_receipts,
        },
        "quotable": quotable,
        "criteria": [
            {"id": "AC-54", "measure": "runs.burst_only.headline_p95_ms", "value": r2(burst_headline),
             "threshold_ms": WRITE_P95_MS, "comparison": "<",
             "pass": judged(burst_headline < WRITE_P95_MS), "marginal": burst_marginal},
            {"id": "AC-53", "measure": "runs.segment.writes.p95_ms", "value": r2(segment_stats.p95),
             "threshold_ms": WRITE_P95_MS, "comparison": "<",
             "pass": judged(segment_stats.p95 < WRITE_P95_MS), "marginal": vin.segment_marginal,
             "deployed_substrate": !m.loopback},
            {"id": "AC-41", "measure": "runs.segment.reveal.p95_ms", "value": r2(reveal.p95),
             "threshold_ms": REVEAL_P95_MS, "comparison": "<=",
             "pass": judged(reveal.p95 <= REVEAL_P95_MS), "marginal": vin.reveal_marginal},
            {"id": "AC-52",
             "measure": ["runs.segment.reconcile.match", "runs.burst_only.cycles[].acked", "runs.burst_only.host_answered_after", "runs.segment.post_close.ok"],
             "value": ac52_pass, "threshold_ms": Value::Null, "comparison": "exact",
             "pass": judged(ac52_pass), "marginal": false},
        ],
        "verdict": {
            "pass": code == Exit::Pass,
            "exit_code": code as i32,
            "notes": notes,
        },
    });
    (report, code)
}
