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
//!
//! A run below the criteria's own conditions — fewer than 200 participants, a
//! window wider than 2 s, one burst shape — can show a miss but cannot earn a
//! pass: when nothing missed it exits `3`, *not a pass*, and judges nothing.

#![allow(dead_code)]

// Explicit paths also work when integration tests include this module via #[path].
#[path = "burst_report/statistics.rs"]
mod statistics;
#[path = "burst_report/verdict.rs"]
mod verdict;

pub use statistics::{ci_ranks, nearest_rank, r2, Stats};
pub use verdict::{
    verdict, Exit, VerdictInput, MIN_NOFILE, REVEAL_P95_MS, SEND_LAG_P95_INVALID_MS,
    WRITE_P95_MS,
};

use serde_json::{json, Value};

/// The criteria's own conditions: AC-52's 200 participants, SPEC §9's 2 s
/// deadline window, and both burst shapes (the worst drives AC-54).
pub const CRITERIA_PARTICIPANTS: usize = 200;
pub const CRITERIA_WINDOW_MS: u64 = 2000;
pub const CRITERIA_SHAPES: usize = 2;

/// Connections `burst` holds at its peak with `n` participants: each one's
/// kept-alive HTTP connection and its buzzer socket, then the wall's socket,
/// the host's socket and the host's HTTP connection (`burst.rs`, `run`).
pub fn expected_peak_connections(n: usize) -> usize {
    2 * n + 3
}
pub const SCHEMA: &str = "rustnyc-popquiz/burst-report/room-1";

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
    /// How many burst shapes ran.
    pub shapes: usize,
    /// The room this run created (an id, never a credential).
    pub room_id: String,
    /// The proxy's connection cap, when given.
    pub connection_cap: Option<u64>,
    pub cycles: Vec<Cycle>,
    /// The host's live `answered` after the isolated bursts (N if each session
    /// was counted once).
    pub host_answered_after_bursts: Option<u64>,
    pub churn_ms: Vec<f64>,
    pub deadline_ms: Vec<f64>,
    /// Churn and deadline writes the room answered but did not save as sent
    /// (anything but `200 {saved: <the letter>}`).
    pub unacked_writes: usize,
    /// Writes sent while live that the room never answered, and how many of
    /// them were deadline writes.
    pub unanswered_writes: usize,
    pub unanswered_deadline: usize,
    pub unanswered_first: Vec<String>,
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
    // Every scheduled write is a sample or a write the room never answered;
    // a post-close refusal slipped in shows as one too many.
    let population_ok = segment.len() + m.unanswered_writes == m.scheduled_writes
        && m.deadline_ms.len() + m.unanswered_deadline == m.connected;
    let segment_stats = Stats::new(segment);
    let reveal = Stats::new(m.reveal_ms.clone());
    let send_lag = Stats::new(m.send_lag_ms.clone());

    let bursts_acked = m.cycles.iter().all(|c| c.acked)
        && m.host_answered_after_bursts == Some(n as u64)
        && m.unacked_writes == 0
        && m.unanswered_writes == 0;
    let peak = expected_peak_connections(n) as u64;
    let mut reduced = Vec::new();
    if n < CRITERIA_PARTICIPANTS {
        reduced.push(format!("n={n}, under {CRITERIA_PARTICIPANTS} participants"));
    }
    if m.window_ms > CRITERIA_WINDOW_MS {
        reduced.push(format!("a {} ms window, wider than {CRITERIA_WINDOW_MS} ms", m.window_ms));
    }
    if m.shapes < CRITERIA_SHAPES {
        reduced.push(format!("{} burst shape, not both", m.shapes));
    }
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
        send_lag_p95: send_lag.p95,
        ulimit_nofile: m.ulimit_nofile,
        errors: m.errors.len() as u64,
        missing_reveal_receipts: m.connected.saturating_sub(reveal.n) as u64,
        segment_population_ok: population_ok,
        unanswered_writes: m.unanswered_writes as u64,
        connections: m.connection_cap.map(|cap| (peak, cap)),
        reduced: reduced.clone(),
        burst_marginal,
        segment_marginal: segment_stats.marginal(WRITE_P95_MS, true),
        reveal_marginal: reveal.marginal(REVEAL_P95_MS, false),
    };
    let (code, notes) = verdict(&vin);

    // An invalid run's criteria are not judged at all: `pass` is null, and
    // the report says at the top level that none of its numbers may be quoted.
    // Below the criteria's conditions a criterion can be judged a miss, never
    // a pass.
    let quotable = code != Exit::Invalid;
    let judged = |b: bool| {
        if !quotable || (b && !reduced.is_empty()) {
            Value::Null
        } else {
            Value::Bool(b)
        }
    };
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
            "id": m.room_id,
            "question": m.question,
        },
        "connections": {
            "expected_peak": peak,
            "cap": m.connection_cap,
            "over_cap": m.connection_cap.is_some_and(|cap| peak > cap),
            "note": "2n+3: each participant's HTTP connection and buzzer socket, the wall's and host's sockets, the host's HTTP connection",
        },
        "conditions": {
            "n": n,
            "window_ms": m.window_ms,
            "shapes": m.shapes,
            "at_criteria_conditions": reduced.is_empty(),
            "below_because": reduced.clone(),
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
                    "never_answered": m.unanswered_writes,
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
            "first_unanswered": m.unanswered_first.iter().take(10).collect::<Vec<_>>(),
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

