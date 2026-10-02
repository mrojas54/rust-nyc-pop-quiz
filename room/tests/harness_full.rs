//! `burst` and `smoke` against a room this test starts (T-21), in `test-full`.
//!
//! The Orchestrator's ruling on T-21 (F-39): nothing unattended may create a
//! room on the deployed app — only a signed-in organizer can — so the run CI
//! makes on every change is against **a room the job itself starts**: the real
//! router (`room::router_with`) served by `room::ws::serve` on `127.0.0.1:0`,
//! real sessions, real sockets, *Create a room* authorized by the tests'
//! `HostAuth` and q3 loaded the way the tests load it. Nothing here reaches the
//! binary, the Dockerfile or the image.
//!
//! What this proves: both harnesses drive the room's real protocol end to end;
//! AC-52's exact count, AC-54's isolated window and AC-41's fan-out mechanics
//! at 200 participants. What it does not: its latencies are this machine's
//! loopback, labelled so in the report, and **AC-53's "deployed substrate"
//! clause is met only by a run against the deployed room** — the client's
//! `deployed-burst` workflow or the laptop form (room/README.md, *Burst*).
//!
//! `#[ignore]`d and behind the `burst` feature, so `just test` never compiles
//! it; `just harness-full` (inside `test-full`) runs it with `--ignored`, after
//! raising the open-file limit.

mod common;

#[path = "../src/bin/burst.rs"]
#[allow(dead_code)]
mod burst;
#[path = "../src/bin/smoke.rs"]
#[allow(dead_code)]
mod smoke;

use std::sync::Arc;

use common::{q3, TestAuth, ORGANIZER};
use room::rooms::{AppState, Urls};
use serde_json::Value;

/// A fresh room server on loopback, one per run: a question runs once per
/// machine (§4.6), and each harness releases the room it makes.
async fn room_on_loopback() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let urls = Urls { base: base.clone(), home: format!("{base}/last") };
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], urls));
    tokio::spawn(room::ws::serve(listener, room::router_with(state)));
    base
}

fn criterion<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["criteria"].as_array().unwrap().iter().find(|c| c["id"] == id).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "test-full: `just harness-full`"]
async fn burst_on_loopback_at_200() {
    let base = room_on_loopback().await;
    let argv: Vec<String> = format!("--url {base} --participants 200 --cycles 1 --churn-secs 3 --gap-ms 500 --seed 20260930")
        .split_whitespace()
        .map(String::from)
        .collect();
    let args = burst::parse_args(&argv).unwrap();
    let (r, exit) = burst::run(&args, ORGANIZER, format!("harness_full {}", argv.join(" "))).await.expect("burst ran");

    let p95 = |id: &str| criterion(&r, id)["value"].as_f64().unwrap_or(-1.0);
    println!(
        "burst, LOOPBACK (this machine, not the deployed substrate): AC-54 burst p95 {:.1} ms · AC-53 segment write p95 {:.1} ms · AC-41 reveal p95 {:.1} ms · AC-52 exact {} · send-lag p95 {:.1} ms · exit {}",
        p95("AC-54"),
        p95("AC-53"),
        p95("AC-41"),
        criterion(&r, "AC-52")["value"],
        r["diagnostics"]["send_lag_ms"]["p95_ms"].as_f64().unwrap_or(-1.0),
        exit as i32
    );
    for note in r["verdict"]["notes"].as_array().unwrap() {
        println!("  {}", note.as_str().unwrap());
    }

    // The harness, not the machine: every participant joined, every write was
    // answered, every reveal arrived, the counts are exact.
    assert_eq!(r["substrate"], "loopback");
    assert_eq!(criterion(&r, "AC-53")["deployed_substrate"], false);
    assert!(r["client"]["ulimit_nofile"].as_u64().unwrap() >= 1024, "{}", r["client"]);
    assert_eq!(r["client"]["participants_connected"], 200, "{}", r["diagnostics"]["first_errors"]);
    assert_eq!(r["diagnostics"]["errors"], 0, "{}", r["diagnostics"]["first_errors"]);
    assert_eq!(r["diagnostics"]["missing_reveal_receipts"], 0);
    assert_eq!(r["runs"]["segment"]["reveal"]["n"], 200);
    for cycle in r["runs"]["burst_only"]["cycles"].as_array().unwrap() {
        assert_eq!(cycle["writes"]["n"], 200, "{cycle}");
        assert_eq!(cycle["acked"], true, "{}", cycle["shape"]);
    }
    assert_eq!(r["runs"]["burst_only"]["cycles"].as_array().unwrap().len(), 2, "one cycle per shape");
    assert_eq!(r["runs"]["segment"]["writes_population"]["ok"], true);
    assert_eq!(r["runs"]["segment"]["writes_population"]["deadline_burst"], 200);
    // AC-52, all four ways.
    assert_eq!(r["runs"]["burst_only"]["host_answered_after"], 200);
    assert_eq!(r["runs"]["segment"]["reconcile"]["match"], true, "{}", r["runs"]["segment"]["reconcile"]);
    assert_eq!(r["runs"]["segment"]["post_close"]["ok"], true, "{}", r["runs"]["segment"]["post_close"]);
    assert_eq!(criterion(&r, "AC-52")["value"], true);
    // The deadline burst's writes were all scheduled inside one window.
    assert_eq!(r["runs"]["segment"]["burst_window_ms"], 2000);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "test-full: `just harness-full`"]
async fn smoke_on_loopback() {
    let base = room_on_loopback().await;
    let args = smoke::parse_args(format!("--url {base} --participants 50").split_whitespace().map(String::from)).unwrap();
    let outcome = smoke::run(&args, ORGANIZER).await.expect("smoke ran");
    println!("smoke, LOOPBACK:");
    outcome.lines.iter().for_each(|l| println!("  {l}"));
    assert_eq!(outcome.substrate, "loopback");
    assert!(outcome.passed(), "smoke failed on loopback:\n{}", outcome.failures.join("\n"));
}
