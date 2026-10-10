//! AC-55's log (T-21, `src/requestlog.rs`): which participant requests count,
//! which count as failed, the summary at release, and that no line carries a
//! secret. The real router with the real session map, sockets over loopback.
//!
//! A `5xx` cannot be provoked from outside a correct room, so its path is the
//! unit test in `requestlog.rs`; here are the drops, the non-failures and the
//! summary a client reads after a meetup.

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::http::{Method, StatusCode};
use common::{http, load, q3, q3_json, TestAuth, ORGANIZER};
use futures_util::{SinkExt, StreamExt};
use room::requestlog;
use room::rooms::{AppState, Urls};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

type Socket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn attach(addr: std::net::SocketAddr, room: &str, token: &str) -> Socket {
    let (mut s, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/rooms/{room}/ws/buzzer")).await.unwrap();
    s.send(Message::text(json!({ "t": "attach", "token": token }).to_string())).await.unwrap();
    // The first frame means it attached (and was counted).
    loop {
        match tokio::time::timeout(Duration::from_secs(5), s.next()).await.expect("a first frame") {
            Some(Ok(Message::Text(_))) => return s,
            Some(Ok(_)) => continue,
            other => panic!("attach: {other:?}"),
        }
    }
}

/// Wait for `pred` over this room's captured lines.
async fn lines_until(room: &str, pred: impl Fn(&[Value]) -> bool) -> Vec<Value> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let lines = requestlog::captured(room);
        if pred(&lines) || Instant::now() > deadline {
            return lines;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ac55_drops_are_counted_ordinary_refusals_are_not_and_release_summarizes() {
    requestlog::start_capture();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default()));
    let app = room::router_with(state);
    tokio::spawn(room::ws::serve(listener, app.clone()));

    let (status, created) = http(&app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({ "question_id": "q3" }))).await;
    assert_eq!(status, StatusCode::CREATED);
    let (room, code, host) = (
        created["id"].as_str().unwrap().to_string(),
        created["code"].as_str().unwrap().to_string(),
        created["host_session"].as_str().unwrap().to_string(),
    );

    // Three join (3), each attaches a buzzer socket (6).
    let mut tokens = Vec::new();
    for _ in 0..3 {
        let (status, joined) = http(&app, Method::POST, "/join", None, Some(json!({ "code": code }))).await;
        assert_eq!(status, StatusCode::CREATED);
        tokens.push(joined["token"].as_str().unwrap().to_string());
    }
    let mut sockets = Vec::new();
    for t in &tokens {
        sockets.push(attach(addr, &room, t).await);
    }
    // Not participant requests: a wrong code, a host action.
    let (status, _) = http(&app, Method::POST, "/join", None, Some(json!({ "code": "ZZZZZZ" }))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(http(&app, Method::POST, &format!("/rooms/{room}/put-on-screen"), Some(&host), None).await.0, StatusCode::OK);

    // An answer (7) and a state read (8); a write with a token the room does
    // not know is a 401 — counted, not failed.
    let (status, _) = http(&app, Method::PUT, &format!("/rooms/{room}/answer"), Some(&tokens[0]), Some(json!({ "letter": "B" }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(http(&app, Method::GET, &format!("/rooms/{room}/buzzer"), None, None).await.0, StatusCode::OK);
    let (status, _) = http(&app, Method::PUT, &format!("/rooms/{room}/answer"), Some("not-a-token"), Some(json!({ "letter": "C" }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Buzzer 1's connection breaks mid-`live`: no close frame. One failure.
    let broken = sockets.remove(1);
    drop(broken);
    let lines = lines_until(&room, |l| l.iter().any(|v| v["kind"] == "socket_dropped")).await;
    let dropped: Vec<&Value> = lines.iter().filter(|v| v["kind"] == "socket_dropped").collect();
    assert_eq!(dropped.len(), 1, "{lines:?}");
    assert_eq!(dropped[0]["phase"], "live");
    assert_eq!(dropped[0]["route"], "buzzer_socket");

    // Buzzer 2 closes properly (the phone left); buzzer 0 is replaced by the
    // same phone re-attaching (10). Neither is a failure.
    let mut left = sockets.remove(1);
    left.close(None).await.unwrap();
    let again = attach(addr, &room, &tokens[0]).await;
    // After close, a write is refused 409 (11): the room answering correctly.
    assert_eq!(http(&app, Method::POST, &format!("/rooms/{room}/close-answers"), Some(&host), None).await.0, StatusCode::OK);
    let (status, _) = http(&app, Method::PUT, &format!("/rooms/{room}/answer"), Some(&tokens[0]), Some(json!({ "letter": "D" }))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(requestlog::captured(&room).iter().filter(|v| v["event"] == "participant_request_failed").count(), 1);

    // Walk to release: the summary, once.
    for slug in ["show-split", "walk-it", "reveal", "release"] {
        assert_eq!(http(&app, Method::POST, &format!("/rooms/{room}/{slug}"), Some(&host), None).await.0, StatusCode::OK, "{slug}");
    }
    let lines = lines_until(&room, |l| l.iter().any(|v| v["event"] == "participant_requests")).await;
    let summary: Vec<&Value> = lines.iter().filter(|v| v["event"] == "participant_requests").collect();
    assert_eq!(summary.len(), 1, "{lines:?}");
    assert_eq!((summary[0]["failed"].as_u64(), summary[0]["total"].as_u64(), summary[0]["ended"].as_str()), (Some(1), Some(11), Some("released")), "{}", summary[0]);
    // A phone re-reading the released room does not reopen its counters.
    assert_eq!(http(&app, Method::GET, &format!("/rooms/{room}/buzzer"), None, None).await.0, StatusCode::OK);
    assert_eq!(requestlog::counts_for(&room), requestlog::Counts::default());
    // Sockets the room closes as it ends are not failures.
    drop(again);
    drop(sockets);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(requestlog::captured(&room).iter().filter(|v| v["event"] == "participant_request_failed").count(), 1);

    // Every line: exactly the stable fields, and no secret.
    for line in requestlog::captured(&room) {
        let keys: Vec<&str> = line.as_object().unwrap().keys().map(String::as_str).collect();
        let want: &[&str] = if line["event"] == "participant_requests" {
            &["ended", "event", "failed", "room", "total"]
        } else {
            &["event", "kind", "phase", "room", "route"]
        };
        assert_eq!(keys, want, "{line}");
        let text = line.to_string();
        for secret in tokens.iter().chain([&host, &code]) {
            assert!(!text.contains(secret.as_str()), "a secret in {text}");
        }
        for v in line.as_object().unwrap().values() {
            assert!(!["A", "B", "C", "D", "E"].contains(&v.as_str().unwrap_or("")), "an answer in {text}");
        }
    }
}

/// A server with the night's question and q3's record under a harness id, and
/// its address.
async fn two_question_server() -> (std::net::SocketAddr, axum::Router) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mut harness = q3_json();
    harness["id"] = "burst-q3".into();
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3(), load(&harness)], Urls::default()));
    let app = room::router_with(state);
    tokio::spawn(room::ws::serve(listener, app.clone()));
    (addr, app)
}

/// Create a room on `question`; its id, code and host session.
async fn create(app: &axum::Router, question: &str) -> (String, String, String) {
    let (status, created) = http(app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({ "question_id": question }))).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let f = |k: &str| created[k].as_str().unwrap().to_string();
    (f("id"), f("code"), f("host_session"))
}

async fn join(app: &axum::Router, code: &str) -> String {
    let (status, joined) = http(app, Method::POST, "/join", None, Some(json!({ "code": code }))).await;
    assert_eq!(status, StatusCode::CREATED, "{joined}");
    joined["token"].as_str().unwrap().to_string()
}

/// ws.rs's attach guard: a buzzer that attaches to a released room is not a
/// participant request. The real map drops sessions at release, so the room
/// refuses that socket anyway; a session map that still resolves the token
/// (`TestTokens`) is what reaches the guard — the race a release can win
/// between a phone's resolve and its count.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_buzzer_attaching_after_release_reopens_no_counter() {
    requestlog::start_capture();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default()));
    let tokens = Arc::new(common::TestTokens::default());
    tokens.add("a-phone", 1, None);
    let transport = room::ws::Transport::new(state.clone(), tokens);
    let app = room::router_with(state).layer(axum::Extension(transport));
    tokio::spawn(room::ws::serve(listener, app.clone()));
    let (room, _, host) = create(&app, "q3").await;
    for slug in ["put-on-screen", "close-answers", "show-split", "walk-it", "reveal", "release"] {
        assert_eq!(http(&app, Method::POST, &format!("/rooms/{room}/{slug}"), Some(&host), None).await.0, StatusCode::OK, "{slug}");
    }
    lines_until(&room, |l| l.iter().any(|v| v["event"] == "participant_requests")).await;
    let socket = attach(addr, &room, "a-phone").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(requestlog::counts_for(&room), requestlog::Counts::default(), "an attach after release reopened the counters");
    drop(socket);
}

/// What an aborted harness run leaves behind (room/README.md, *After a
/// meetup*): its room open, and one `socket_dropped` line per buzzer whose
/// socket it abandoned — every one carrying the harness room's id, none of
/// them on the night's room's lines or counters.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_aborted_harness_run_leaves_its_drops_on_its_own_room() {
    requestlog::start_capture();
    let (addr, app) = two_question_server().await;
    let (night, night_code, _) = create(&app, "q3").await;
    let phone = join(&app, &night_code).await;
    let _phone_socket = attach(addr, &night, &phone).await;

    let (harness, code, host) = create(&app, "burst-q3").await;
    let mut sockets = Vec::new();
    for _ in 0..4 {
        let token = join(&app, &code).await;
        sockets.push(attach(addr, &harness, &token).await);
    }
    assert_eq!(http(&app, Method::POST, &format!("/rooms/{harness}/put-on-screen"), Some(&host), None).await.0, StatusCode::OK);
    // The process dies: every socket goes without a close frame.
    drop(sockets);
    let lines = lines_until(&harness, |l| l.iter().filter(|v| v["kind"] == "socket_dropped").count() == 4).await;
    assert_eq!(lines.iter().filter(|v| v["kind"] == "socket_dropped").count(), 4, "{lines:?}");
    assert_eq!(requestlog::counts_for(&harness).failed, 4);
    // The night's room: one join, one attach, nothing failed.
    assert!(requestlog::captured(&night).is_empty(), "{:?}", requestlog::captured(&night));
    assert_eq!(requestlog::counts_for(&night), requestlog::Counts { failed: 0, total: 2 });
    // And the harness room is still open, holding only its harness id.
    let (status, again) = http(&app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({ "question_id": "burst-q3" }))).await;
    assert_eq!((status, again["reason"].as_str()), (StatusCode::CONFLICT, Some("That question is open in another room. Pick another.")));
}
