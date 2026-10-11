//! `burst`'s decisions on recorded samples (T-21): the percentile and its
//! interval, `marginal`, the pass / miss / invalid verdict, and the report's
//! shape. No sockets and no clock: `src/bin/burst_report.rs` is pure, so this
//! runs in `just test` while the client that gathers samples stays behind the
//! `burst` feature. What the client does over real sockets is
//! `tests/harness_full.rs` (test-full).

#[path = "../src/bin/burst_report.rs"]
mod burst_report;

use burst_report::*;
use serde_json::Value;

fn ms(v: impl IntoIterator<Item = f64>) -> Vec<f64> {
    v.into_iter().collect()
}

/// A run at `n` participants in which everything happened and every number is
/// fast: two isolated bursts, a churn, the deadline burst, the reveal.
fn clean(n: usize) -> Measured {
    let fast = |k: usize| ms((0..k).map(|i| 5.0 + (i % 17) as f64));
    let mut totals = [0u32; 5];
    for i in 0..n {
        totals[i % 5] += 1;
    }
    Measured {
        invocation: "burst --url http://127.0.0.1:1".into(),
        url: "http://127.0.0.1:1".into(),
        loopback: true,
        question: "q3".into(),
        seed: 7,
        participants: n,
        connected: n,
        window_ms: 2000,
        churn_secs: 3,
        shapes: 2,
        room_id: "a-room-id".into(),
        cycles: vec![
            Cycle { shape: "uniform", index: 0, writes_ms: fast(n), acked: true },
            Cycle { shape: "spike", index: 0, writes_ms: fast(n), acked: true },
        ],
        host_answered_after_bursts: Some(n as u64),
        churn_ms: fast(n / 3),
        deadline_ms: fast(n),
        scheduled_writes: 3 * n + n / 3,
        expected_totals: totals,
        split_totals: Some(totals),
        split_answered: Some(n as u64),
        split_disagreeing: 0,
        post_close_refused: n,
        post_close_restated: n,
        reveal_ms: fast(n),
        connect_ms: fast(n),
        join_ms: fast(n),
        send_lag_ms: ms((0..n).map(|_| 1.0)),
        ulimit_nofile: 4096,
        ..Measured::default()
    }
}

fn criterion<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["criteria"].as_array().unwrap().iter().find(|c| c["id"] == id).unwrap()
}

// --------------------------------------------------------------------------
// The numbers.
// --------------------------------------------------------------------------

#[test]
fn nearest_rank_and_its_interval_rounded_outward() {
    let v = ms((1..=200).map(f64::from));
    assert_eq!(nearest_rank(&v, 0.95), (190, 190.0));
    assert_eq!(nearest_rank(&v, 0.50), (100, 100.0));
    assert_eq!(nearest_rank(&[7.0], 0.95), (1, 7.0));
    assert!(nearest_rank(&[], 0.95).1.is_nan());
    // sqrt(200·0.95·0.05) = 3.08; ±1.96σ = ±6.04 around 190, floored and ceiled.
    assert_eq!(ci_ranks(200, 0.95, 190), (183, 197));
    let s = Stats::new(v);
    assert_eq!((s.p95_ci.0, s.p95_ci.1, s.max), (183.0, 197.0, 200.0));
}

#[test]
fn a_pass_whose_interval_crosses_the_threshold_is_marginal() {
    // p95 (rank 190) is 400, under 500; rank 197, the interval's top, is 600.
    let mut v = vec![1.0; 189];
    v.push(400.0);
    v.extend([600.0; 10]);
    let s = Stats::new(v);
    assert_eq!(s.p95, 400.0);
    assert!(s.marginal(WRITE_P95_MS, true));
    // Clean: the whole interval under.
    assert!(!Stats::new(vec![1.0; 200]).marginal(WRITE_P95_MS, true));
    // A miss is a miss, not marginal.
    assert!(!Stats::new(vec![700.0; 200]).marginal(WRITE_P95_MS, true));
    // AC-41 is "at most": exactly 2000 passes, and an interval ending at 2000 is clean.
    assert!(!Stats::new(vec![2000.0; 200]).marginal(REVEAL_P95_MS, false));
    // AC-53/54 are "strictly under": exactly 500 at the interval's top is marginal.
    let mut v = vec![1.0; 196];
    v.extend([500.0; 4]);
    assert!(Stats::new(v).marginal(WRITE_P95_MS, true));
}

// --------------------------------------------------------------------------
// The verdict.
// --------------------------------------------------------------------------

fn passing() -> VerdictInput {
    VerdictInput {
        burst_headline_p95: 40.0,
        segment_write_p95: 40.0,
        reveal_p95: 120.0,
        ac52_totals_ok: true,
        bursts_acked: true,
        post_close_ok: true,
        clients_connected: 200,
        clients_expected: 200,
        send_lag_p95: 2.0,
        ulimit_nofile: 4096,
        segment_population_ok: true,
        ..VerdictInput::default()
    }
}

#[test]
fn every_criterion_can_miss_on_its_own() {
    assert_eq!(verdict(&passing()).0, Exit::Pass);
    let cases: [(&str, fn(&mut VerdictInput)); 7] = [
        ("AC-54 MISS", |v| v.burst_headline_p95 = 500.0),
        ("AC-53 MISS", |v| v.segment_write_p95 = 612.0),
        ("AC-41 MISS", |v| v.reveal_p95 = 2000.5),
        ("AC-52 MISS: the split", |v| v.ac52_totals_ok = false),
        ("AC-52 MISS: a write while live", |v| v.bursts_acked = false),
        ("AC-52 MISS: after close", |v| v.post_close_ok = false),
        ("AC-41 MISS", |v| v.reveal_p95 = f64::NAN),
    ];
    for (want, change) in cases {
        let mut v = passing();
        change(&mut v);
        let (exit, notes) = verdict(&v);
        assert_eq!(exit, Exit::Missed, "{want}");
        assert!(notes.iter().any(|n| n.starts_with(want)), "{want}: {notes:?}");
        assert!(!notes.iter().any(|n| n.contains("pass as measured")), "{want}: {notes:?}");
    }
    // Exactly 2000 ms passes AC-41 ("within 2 seconds").
    let mut v = passing();
    v.reveal_p95 = 2000.0;
    assert_eq!(verdict(&v).0, Exit::Pass);
}

#[test]
fn an_invalid_run_is_not_judged_even_when_a_criterion_also_missed() {
    let cases: [(&str, fn(&mut VerdictInput)); 6] = [
        ("of 200 participants", |v| v.clients_connected = 199),
        ("ulimit", |v| v.ulimit_nofile = 256),
        ("harness errors", |v| v.errors = 1),
        ("reveal receipts", |v| v.missing_reveal_receipts = 3),
        ("send lag", |v| v.send_lag_p95 = 25.5),
        ("write samples", |v| v.segment_population_ok = false),
    ];
    for (want, change) in cases {
        let mut v = passing();
        v.segment_write_p95 = 900.0;
        change(&mut v);
        let (exit, notes) = verdict(&v);
        assert_eq!(exit, Exit::Invalid, "{want}");
        assert!(notes.iter().all(|n| n.starts_with("RUN INVALID")), "{want}: {notes:?}");
        assert!(notes.iter().any(|n| n.contains(want)), "{want}: {notes:?}");
    }
}

#[test]
fn marginal_is_said_in_words_on_a_pass_and_never_beside_a_miss() {
    let mut v = passing();
    v.reveal_marginal = true;
    let (exit, notes) = verdict(&v);
    assert_eq!(exit, Exit::Pass);
    assert!(notes.iter().any(|n| n.starts_with("AC-41 MARGINAL")));
    assert!(notes.last().unwrap().contains("1 of them marginally"));
    assert!(!notes.iter().any(|n| n == "all four criteria pass as measured"));

    v.post_close_ok = false;
    let (exit, notes) = verdict(&v);
    assert_eq!(exit, Exit::Missed);
    assert!(!notes.iter().any(|n| n.contains("MARGINAL")), "{notes:?}");
}

// --------------------------------------------------------------------------
// The report.
// --------------------------------------------------------------------------

#[test]
fn a_clean_run_reports_every_field_and_every_sample() {
    let (r, exit) = report(&clean(200));
    assert_eq!(exit, Exit::Pass, "{}", r["verdict"]);
    assert_eq!(r["verdict"]["notes"].as_array().unwrap().last().unwrap(), "all four criteria pass as measured");
    assert_eq!(r["conditions"]["at_criteria_conditions"], true);
    assert_eq!(r["schema"], SCHEMA);
    assert_eq!(r["quotable"], true);
    for key in ["substrate", "substrate_note", "invocation", "room", "connections", "conditions", "client", "runs", "diagnostics", "criteria", "verdict"] {
        assert!(!r[key].is_null(), "{key}");
    }
    for id in ["AC-54", "AC-53", "AC-41", "AC-52"] {
        assert_eq!(criterion(&r, id)["pass"], true, "{id}");
    }
    // Raw samples ship, so any percentile can be recomputed from the file.
    let seg = &r["runs"]["segment"]["writes"];
    assert_eq!(seg["n"], 200 * 3 + 200 / 3);
    assert_eq!(seg["samples_ms"].as_array().unwrap().len(), 200 * 3 + 200 / 3);
    assert_eq!(r["runs"]["segment"]["reveal"]["samples_ms"].as_array().unwrap().len(), 200);
    assert_eq!(r["runs"]["burst_only"]["cycles"].as_array().unwrap().len(), 2);
    // The report survives a round trip through text.
    let again: Value = serde_json::from_str(&r.to_string()).unwrap();
    assert_eq!(again, r);
}

#[test]
fn the_substrate_is_labelled_and_loopback_never_claims_the_deployed_clause() {
    let (r, _) = report(&clean(10));
    assert_eq!(r["substrate"], "loopback");
    assert!(r["substrate_note"].as_str().unwrap().starts_with("LOOPBACK"));
    assert_eq!(criterion(&r, "AC-53")["deployed_substrate"], false);

    let mut m = clean(10);
    m.loopback = false;
    m.url = "https://rustnyc-popquiz.fly.dev".into();
    let (r, _) = report(&m);
    assert_eq!(r["substrate"], "deployed");
    assert_eq!(criterion(&r, "AC-53")["deployed_substrate"], true);
}

#[test]
fn an_invalid_report_says_so_and_judges_nothing() {
    let mut m = clean(10);
    m.connected = 9;
    m.deadline_ms.pop();
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Invalid);
    assert_eq!(r["quotable"], false);
    assert_eq!(r["verdict"]["exit_code"], 2);
    for id in ["AC-54", "AC-53", "AC-41", "AC-52"] {
        assert!(criterion(&r, id)["pass"].is_null(), "{id} judged on an invalid run");
    }
}

#[test]
fn a_refusal_counted_as_a_write_invalidates_the_run() {
    let mut m = clean(10);
    m.deadline_ms.push(3.0); // a post-close 409 slipped into the samples
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Invalid);
    assert_eq!(r["runs"]["segment"]["writes_population"]["ok"], false);
}

#[test]
fn the_worst_cycle_drives_ac54() {
    let mut m = clean(20);
    m.cycles[1].writes_ms = vec![700.0; 20];
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Missed);
    assert_eq!(r["runs"]["burst_only"]["headline_p95_ms"], 700.0);
    assert_eq!(criterion(&r, "AC-54")["pass"], false);
}

#[test]
fn two_sessions_swapping_answers_fail_ac52_although_every_total_survives() {
    // Totals, answered and the sum all match; two sessions' refusals after close
    // restated each other's letter rather than their own.
    let mut m = clean(20);
    m.post_close_restated = 18;
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Missed);
    assert_eq!(r["runs"]["segment"]["reconcile"]["match"], true);
    assert_eq!(criterion(&r, "AC-52")["pass"], false);

    // A session counted twice shows in the host's live count.
    let mut m = clean(20);
    m.host_answered_after_bursts = Some(21);
    assert_eq!(report(&m).1, Exit::Missed);

    // A churn write the room did not save as sent.
    let mut m = clean(20);
    m.unacked_writes = 1;
    assert_eq!(report(&m).1, Exit::Missed);

    // A lost write shows in the totals.
    let mut m = clean(20);
    m.split_totals.as_mut().unwrap()[0] -= 1;
    m.split_totals.as_mut().unwrap()[1] += 1;
    assert_eq!(report(&m).1, Exit::Missed);
}

// --------------------------------------------------------------------------
// PQ-41: what a verdict may not claim.
// --------------------------------------------------------------------------

fn notes(r: &Value) -> Vec<String> {
    r["verdict"]["notes"].as_array().unwrap().iter().map(|n| n.as_str().unwrap().to_string()).collect()
}

#[test]
fn a_run_below_the_criteria_conditions_can_miss_but_never_pass() {
    let mut window = clean(200);
    window.window_ms = 3000;
    let mut one_shape = clean(200);
    one_shape.shapes = 1;
    one_shape.cycles.pop();
    one_shape.scheduled_writes -= 200;
    for (want, m) in [("n=12", clean(12)), ("3000 ms window", window), ("1 burst shape", one_shape)] {
        let (r, exit) = report(&m);
        assert_eq!(exit, Exit::NotAPass, "{want}: {:?}", notes(&r));
        assert_eq!(r["verdict"]["exit_code"], 3);
        assert_eq!(r["verdict"]["pass"], false);
        let n = notes(&r);
        assert!(n.iter().any(|t| t.starts_with("NOT A PASS") && t.contains(want)), "{want}: {n:?}");
        assert!(!n.iter().any(|t| t.contains("pass as measured")), "{want}: {n:?}");
        for id in ["AC-54", "AC-53", "AC-41", "AC-52"] {
            assert!(criterion(&r, id)["pass"].is_null(), "{want}: {id} judged a pass below the conditions");
        }
        assert_eq!(r["conditions"]["at_criteria_conditions"], false);
    }
    // A miss below the conditions is still a miss, and says so.
    let mut m = clean(12);
    m.deadline_ms = vec![900.0; 12];
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Missed);
    assert_eq!(criterion(&r, "AC-53")["pass"], false);
    assert!(criterion(&r, "AC-41")["pass"].is_null());
    assert_eq!(r["conditions"]["n"], 12);
}

#[test]
fn a_run_over_the_connection_cap_is_invalid_never_a_pass() {
    assert_eq!(expected_peak_connections(200), 403);
    assert_eq!(expected_peak_connections(1), 5);
    let mut m = clean(200);
    m.connection_cap = Some(400);
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Invalid);
    assert_eq!(r["quotable"], false);
    assert_eq!((r["connections"]["expected_peak"].as_u64(), r["connections"]["cap"].as_u64(), r["connections"]["over_cap"].as_bool()), (Some(403), Some(400), Some(true)));
    assert!(notes(&r).iter().any(|t| t.contains("connection cap") && t.contains("403") && t.contains("400") && t.contains("AC-53")), "{:?}", notes(&r));
    // At the cap exactly, the run stands.
    m.connection_cap = Some(403);
    assert_eq!(report(&m).1, Exit::Pass);
}

#[test]
fn a_write_the_room_never_answered_is_an_ac52_miss() {
    let mut m = clean(200);
    m.deadline_ms.pop();
    m.unanswered_writes = 1;
    m.unanswered_deadline = 1;
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Missed, "{:?}", notes(&r));
    assert_eq!(r["runs"]["segment"]["writes_population"]["ok"], true, "an unanswered write is in the population");
    assert!(notes(&r).iter().any(|t| t.starts_with("AC-52 MISS: 1 writes sent while live were never answered")), "{:?}", notes(&r));
    assert_eq!(criterion(&r, "AC-52")["pass"], false);
    // Said even when the run is invalid for another reason.
    m.errors.push("connect: refused".into());
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Invalid);
    assert!(notes(&r).iter().any(|t| t.starts_with("AC-52 MISS")), "{:?}", notes(&r));
}

#[test]
fn no_send_lag_samples_is_invalid_not_zero() {
    let mut m = clean(200);
    m.send_lag_ms.clear();
    let (r, exit) = report(&m);
    assert_eq!(exit, Exit::Invalid);
    assert!(notes(&r).iter().any(|t| t.contains("no send-lag samples")), "{:?}", notes(&r));
}
