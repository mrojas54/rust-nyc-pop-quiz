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
use common::{http, q3, TestAuth, ORGANIZER};
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
