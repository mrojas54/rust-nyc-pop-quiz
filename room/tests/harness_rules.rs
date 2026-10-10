//! What `burst` and `smoke` refuse before they touch a room, and what they
//! say when a room refuses them (PQ-41, PQ-65). The rules are in
//! `src/bin/room_client.rs` and each binary's `guard`; the room's own refusals
//! come from the real router, in process, with no network.
//!
//! Behind the `burst` feature, like the binaries it includes (their TLS client
//! is compiled by nothing in `just test`); the fast CI job and `harness-full`
//! run it.

#![cfg(feature = "burst")]

mod common;

#[path = "../src/bin/burst.rs"]
#[allow(dead_code)]
mod burst;
#[path = "../src/bin/smoke.rs"]
#[allow(dead_code)]
mod smoke;
#[path = "../src/bin/room_client.rs"]
#[allow(dead_code)]
mod client;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, StatusCode};
use common::{http, q3, TestAuth, ORGANIZER};
use room::rooms::{AppState, Urls};
use serde_json::json;

fn argv(s: &str) -> Vec<String> {
    s.split_whitespace().map(String::from).collect()
}

// --------------------------------------------------------------------------
// Item B: a deployed-shaped URL cannot schedule or release a bank question by
// default.
// --------------------------------------------------------------------------

#[test]
fn both_harnesses_default_to_a_harness_id() {
    let b = burst::parse_args(&argv("--url https://rustnyc-popquiz.fly.dev --connection-cap 400")).unwrap();
    assert_eq!(b.question, "burst-q3");
    let s = smoke::parse_args(argv("--url https://rustnyc-popquiz.fly.dev")).unwrap();
    assert_eq!(s.question, "smoke-q3");
    assert!(client::is_harness_id(&b.question) && client::is_harness_id(&s.question));
}

#[test]
fn a_bank_id_off_loopback_is_refused_unless_spent_on_purpose() {
    for url in ["https://rustnyc-popquiz.fly.dev", "https://popquiz.rustnyc.org", "https://stand-in.invalid"] {
        for q in ["q3", "q7", "burst-", "anything"] {
            let e = burst::parse_args(&argv(&format!("--url {url} --connection-cap 400 --question {q}"))).unwrap_err();
            assert!(e.contains("retire a bank question") && e.contains("--spend-bank-question"), "{url} {q}: {e}");
            let e = smoke::parse_args(argv(&format!("--url {url} --question {q}"))).unwrap_err();
            assert!(e.contains("retire a bank question"), "{url} {q}: {e}");
        }
        let b = burst::parse_args(&argv(&format!("--url {url} --connection-cap 400 --question q3 --spend-bank-question"))).unwrap();
        assert!(b.spend_bank);
        assert!(smoke::parse_args(argv(&format!("--url {url} --question q3 --spend-bank-question"))).is_ok());
    }
    // On this machine a bank id is the harness's own business (test-full).
    for url in ["http://127.0.0.1:3000", "http://localhost:3000", "http://[::1]:3000"] {
        assert!(burst::parse_args(&argv(&format!("--url {url} --question q3"))).is_ok(), "{url}");
        assert!(smoke::parse_args(argv(&format!("--url {url} --question q3"))).is_ok(), "{url}");
    }
    // A host that merely starts with "localhost" is not this machine.
    assert!(smoke::parse_args(argv("--url https://localhost.example.com --question q3")).is_err());
}

#[test]
fn run_refuses_a_bank_id_whoever_built_the_arguments() {
    // `run` re-checks: Args built by hand, not through parse_args.
    let mut a = burst::parse_args(&argv("--url https://stand-in.invalid --connection-cap 400")).unwrap();
    a.question = "q3".into();
    assert!(burst::guard(&a).unwrap_err().contains("retire a bank question"));
    let rt = tokio::runtime::Runtime::new().unwrap();
    let e = rt.block_on(burst::run(&a, "not-a-session", "test".into())).unwrap_err();
    assert!(e.contains("retire a bank question"), "{e}");
    let mut s = smoke::parse_args(argv("--url https://stand-in.invalid")).unwrap();
    s.question = "q3".into();
    let e = rt.block_on(smoke::run(&s, "not-a-session")).err().expect("refused");
    assert!(e.contains("retire a bank question"), "{e}");
}

#[test]
fn no_bank_question_has_a_harness_id() {
    let dir = common::repo().join("bank/questions");
    let mut seen = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "json") {
            let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            let id = v["id"].as_str().unwrap();
            assert!(!client::is_harness_id(id), "{path:?}: bank id {id} looks like a harness id");
            seen += 1;
        }
    }
    assert!(seen > 0, "no bank questions read from {dir:?}");
}

// --------------------------------------------------------------------------
// Item A: off loopback the connection cap must be known.
// --------------------------------------------------------------------------

#[test]
fn burst_needs_the_connection_cap_off_loopback() {
    let e = burst::parse_args(&argv("--url https://rustnyc-popquiz.fly.dev")).unwrap_err();
    assert!(e.contains("--connection-cap") && e.contains("403"), "{e}");
    assert!(burst::parse_args(&argv("--url http://127.0.0.1:9 --question q3")).is_ok());
}

// --------------------------------------------------------------------------
// Item D: no organizer session in cleartext; no call waits forever.
// --------------------------------------------------------------------------

#[test]
fn plain_http_is_refused_for_a_host_that_is_not_this_machine() {
    for url in ["http://rustnyc-popquiz.fly.dev", "http://10.0.0.5:3000", "http://stand-in.invalid"] {
        let e = client::Target::parse(url).unwrap_err();
        assert!(e.contains("cleartext"), "{url}: {e}");
        assert!(burst::parse_args(&argv(&format!("--url {url} --connection-cap 400"))).is_err(), "{url}");
        assert!(smoke::parse_args(argv(&format!("--url {url}"))).is_err(), "{url}");
    }
    for url in ["http://127.0.0.1:3000", "http://localhost:8080", "http://[::1]:8080", "https://rustnyc-popquiz.fly.dev"] {
        assert!(client::Target::parse(url).is_ok(), "{url}");
    }
}

#[tokio::test]
async fn a_room_that_never_answers_ends_the_call_not_the_run() {
    // Accepts, reads, never answers.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let mut held = Vec::new();
        loop {
            let (s, _) = listener.accept().await.unwrap();
            held.push(s);
        }
    });
    let target = Arc::new(client::Target::parse(&format!("http://{addr}")).unwrap());
    let mut http = client::Http::new(&target, &None).with_read_timeout(Duration::from_millis(200));
    let r = tokio::time::timeout(Duration::from_secs(5), http.call("PUT", "/rooms/x/answer", Some("t"), Some(&json!({"letter": "A"}))))
        .await
        .expect("the call has its own deadline");
    let e = r.err().expect("no answer is an error");
    assert!(e.contains("no response within"), "{e}");
    assert!(!e.contains(": connect "), "a silent room is not a harness-side failure: {e}");
    assert_eq!(client::READ_TIMEOUT, Duration::from_secs(30));
    assert_eq!(client::CONNECT_TIMEOUT, Duration::from_secs(10));
}

// --------------------------------------------------------------------------
// PQ-65: a 409 on create, by the room's reason. Never a restart.
// --------------------------------------------------------------------------

async fn refusal(state: &Arc<AppState>, question: &str) -> String {
    let app = room::router_with(state.clone());
    let (status, body) = http(&app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({ "question_id": question }))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    body["reason"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn each_create_refusal_gets_its_own_hint_and_none_says_restart() {
    // Not scheduled.
    let empty = Arc::new(AppState::new(Arc::new(TestAuth), vec![], Urls::default()));
    let reason = refusal(&empty, "burst-q3").await;
    let said = client::create_refused(&reason, "burst-q3");
    assert!(said.contains(&format!("{reason:?}")), "the reason, verbatim: {said}");
    assert!(said.contains("schedule q3's record as burst-q3"), "{said}");

    // Open in another room: the held case.
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default()));
    let app = room::router_with(state.clone());
    let (status, created) = http(&app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({ "question_id": "q3" }))).await;
    assert_eq!(status, StatusCode::CREATED);
    let reason = refusal(&state, "q3").await;
    let held = client::create_refused(&reason, "q3");
    assert!(held.contains(&format!("{reason:?}")), "{held}");
    assert!(held.contains("go quiet") && held.contains("30 minutes") && held.contains("20 once started") && held.contains("4 hours"), "{held}");
    assert!(held.contains("another harness id"), "{held}");

    // Already run: walk the room to release, then ask again.
    let (room, host) = (created["id"].as_str().unwrap(), created["host_session"].as_str().unwrap());
    for slug in ["put-on-screen", "close-answers", "show-split", "walk-it", "reveal", "release"] {
        assert_eq!(http(&app, Method::POST, &format!("/rooms/{room}/{slug}"), Some(host), None).await.0, StatusCode::OK, "{slug}");
    }
    let reason = refusal(&state, "q3").await;
    let run = client::create_refused(&reason, "q3");
    assert!(run.contains(&format!("{reason:?}")) && run.contains("has been run on this machine") && run.contains("another harness id"), "{run}");

    // A reason this harness has never seen: verbatim, no guess.
    let unknown = client::create_refused("The room is on fire.", "q3");
    assert!(unknown.contains("\"The room is on fire.\"") && unknown.contains("does not know"), "{unknown}");

    for said in [&said, &held, &run, &unknown] {
        assert!(!said.to_lowercase().contains("restart"), "a 409 hint suggests a restart: {said}");
    }
}

#[test]
fn explain_blames_a_vanished_room_only_for_host_actions_not_create() {
    let create = client::explain("create: 404 {}");
    assert_eq!(create, "create: 404 {}", "a create's 404 is the question or the session, and says so itself");
    let action = client::explain("reveal: 404 {}");
    assert!(action.contains("the room may be gone"), "{action}");
}
