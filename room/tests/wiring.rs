//! The wired room, end to end (PQ-32): the default router, T-04b's real
//! session map behind T-04c's transport, over a real loopback socket. No test
//! map is layered on; a token is whatever `POST /join` handed out.
//!
//! - AC-46: a join and an answer each move the host's counts, pushed as exactly
//!   one new frame to the wall and the host.
//! - AC-81: every attached viewer gets one frame per revision, and every frame
//!   of a revision carries the same `phase`.
//! - AC-37: a buzzer that drops and re-attaches with the same token gets the
//!   room's current state first, with its saved answer from the real map; the
//!   drop changes no count.
//! - After *End Pop Quiz* the token resolves to nothing.

mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, StatusCode};
use common::*;
use room::rooms::{AppState, Urls};
use room::ws::close;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

const WAIT: Duration = Duration::from_secs(2);
/// Long enough that a stray frame would have arrived on loopback.
const QUIET: Duration = Duration::from_millis(300);

struct Wired {
    app: axum::Router,
    addr: std::net::SocketAddr,
    state: Arc<AppState>,
}

impl Wired {
    /// The production router over a test state, served through `ws::serve`.
    async fn start() -> Wired {
        let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default()));
        let app = room::router_with(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(room::ws::serve(listener, app.clone()));
        Wired { app, addr, state }
    }

    async fn open(&self, room: &HostedRoom, viewer: &str, token: Option<&str>) -> Socket {
        let url = format!("ws://{}/rooms/{}/ws/{viewer}", self.addr, room.id);
        let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();
        if let Some(token) = token {
            let attach = serde_json::json!({ "t": "attach", "token": token }).to_string();
            futures_util::SinkExt::send(&mut socket, Message::text(attach)).await.unwrap();
        }
        socket
    }

    async fn act(&self, room: &HostedRoom, slug: &str) -> StatusCode {
        http(&self.app, Method::POST, &format!("/rooms/{}/{slug}", room.id), Some(&room.host), None)
            .await
            .0
    }

    async fn projection(&self, room: &HostedRoom, viewer: &str) -> Value {
        let bearer = (viewer == "host").then_some(room.host.as_str());
        let (status, v) = http(&self.app, Method::GET, &format!("/rooms/{}/{viewer}", room.id), bearer, None).await;
        assert_eq!(status, StatusCode::OK);
        v
    }

    fn revision(&self, room: &HostedRoom) -> u64 {
        self.state.with_room(&room.id, |r| r.revision()).unwrap()
    }
}

/// The frame less the transport's own fields: what the viewer's projection is.
fn payload(frame: &Value) -> Value {
    let mut v = frame.clone();
    let map = v.as_object_mut().unwrap();
    map.remove("t");
    map.remove("revision");
    map.remove("session");
    v
}

fn counts(host: &Value) -> (u64, Option<u64>) {
    (host["present"].as_u64().unwrap(), host["answered"].as_u64())
}

/// Every socket gets exactly one frame at `revision`, all with one `phase`,
/// and nothing after it. Returns the frames in socket order.
async fn one_frame_each(w: &Wired, room: &HostedRoom, sockets: &mut [(&str, &mut Socket)], revision: u64, what: &str) -> Vec<Value> {
    let mut frames = Vec::new();
    for (viewer, socket) in sockets.iter_mut() {
        let f = frame(socket, WAIT).await;
        assert_eq!(f["revision"], revision, "{viewer} after {what}: one frame per revision");
        assert_eq!(payload(&f), w.projection(room, viewer).await, "{viewer} after {what}");
        frames.push(f);
    }
    let phase = &frames[0]["phase"];
    assert!(frames.iter().all(|f| f["phase"] == *phase), "after {what}: phases disagree");
    // Quiet on every socket at once, so the wait is paid once per step.
    let quiet = futures_util::future::join_all(sockets.iter_mut().map(|(_, s)| next(s, QUIET))).await;
    for ((viewer, _), after) in sockets.iter().zip(quiet) {
        assert!(matches!(after, Next::Nothing), "{viewer} after {what}: a second frame");
    }
    frames
}

#[tokio::test]
async fn join_answer_transitions_and_reconnect_through_the_real_session_map() {
    let w = Wired::start().await;
    let room = host_room(&w.app).await;
    let mut wall = w.open(&room, "wall", None).await;
    let mut host = w.open(&room, "host", Some(&room.host)).await;
    let r0 = w.revision(&room);
    assert_eq!(frame(&mut wall, WAIT).await["revision"], r0);
    assert_eq!(counts(&frame(&mut host, WAIT).await).0, 0);

    // Join: the token comes from the real map, and the new `present` is pushed
    // to the wall and the host as one frame (AC-46).
    let (status, joined) = http(&w.app, Method::POST, "/join", None, Some(serde_json::json!({ "code": room.code }))).await;
    assert_eq!(status, StatusCode::CREATED, "{joined}");
    let token = joined["token"].as_str().unwrap().to_string();
    let f = one_frame_each(&w, &room, &mut [("wall", &mut wall), ("host", &mut host)], r0 + 1, "join").await;
    assert_eq!(counts(&f[1]).0, 1, "present moved on the host");

    // Attach with that token: the full current state first (AC-37), nothing
    // saved yet. Attaching changes no count, so nobody else hears of it.
    let mut buzzer = w.open(&room, "buzzer", Some(&token)).await;
    let first = frame(&mut buzzer, WAIT).await;
    assert_eq!(first["t"], "state");
    assert_eq!(first["revision"], r0 + 1);
    assert_eq!(first["phase"], "idle");
    assert_eq!(payload(&first), w.projection(&room, "buzzer").await);
    assert!(first["session"]["saved"].is_null());
    for s in [&mut wall, &mut host] {
        assert!(matches!(next(s, QUIET).await, Next::Nothing), "an attach is not a revision");
    }

    let mut all: [(&str, &mut Socket); 3] = [("wall", &mut wall), ("host", &mut host), ("buzzer", &mut buzzer)];

    // Live, then an answer: one frame each, one phase, and the host's
    // answers-in moves (AC-46, AC-81).
    assert_eq!(w.act(&room, "put-on-screen").await, StatusCode::OK);
    let f = one_frame_each(&w, &room, &mut all, r0 + 2, "put-on-screen").await;
    assert_eq!(f[0]["phase"], "live");
    assert_eq!(counts(&f[1]), (1, Some(0)));

    let uri = format!("/rooms/{}/answer", room.id);
    let (status, saved) = http(&w.app, Method::PUT, &uri, Some(&token), Some(serde_json::json!({ "letter": "C" }))).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["saved"], "C");
    let f = one_frame_each(&w, &room, &mut all, r0 + 3, "answer").await;
    assert_eq!(counts(&f[1]), (1, Some(1)), "answers-in moved on the host");
    assert!(f[2].get("session").is_none(), "broadcasts are never personal");

    // Close answers: one frame each, one phase.
    assert_eq!(w.act(&room, "close-answers").await, StatusCode::OK);
    let f = one_frame_each(&w, &room, &mut all, r0 + 4, "close-answers").await;
    assert_eq!(f[0]["phase"], "closed");

    // The phone drops. The session, its answer and its slot stay: nothing is
    // pushed, and the counts are as they were.
    drop(all);
    buzzer.close(None).await.unwrap();
    for s in [&mut wall, &mut host] {
        assert!(matches!(next(s, QUIET).await, Next::Nothing), "a drop is not a revision");
    }
    assert_eq!(w.revision(&room), r0 + 4);
    assert_eq!(counts(&w.projection(&room, "host").await).0, 1, "the ghost keeps its place in present");
    assert_eq!(w.state.session_count(&room.id).unwrap(), 1);

    // The room moves on while it is away.
    assert_eq!(w.act(&room, "show-split").await, StatusCode::OK);
    let mut two: [(&str, &mut Socket); 2] = [("wall", &mut wall), ("host", &mut host)];
    one_frame_each(&w, &room, &mut two, r0 + 5, "show-split").await;

    // Back with the same token: the room as it is now, first, with the saved
    // answer from the real map (AC-37).
    let mut buzzer = w.open(&room, "buzzer", Some(&token)).await;
    let first = frame(&mut buzzer, WAIT).await;
    assert_eq!(first["t"], "state");
    assert_eq!(first["revision"], r0 + 5);
    assert_eq!(first["phase"], "split");
    assert_eq!(payload(&first), w.projection(&room, "buzzer").await);
    assert_eq!(first["session"]["saved"], "C", "the saved answer survives the reconnect");

    // Drive on to released by the host's own primary action; every step is
    // one frame each, one phase across all three (AC-81).
    let mut revision = r0 + 5;
    let mut host_view = w.projection(&room, "host").await;
    let mut all: [(&str, &mut Socket); 3] = [("wall", &mut wall), ("host", &mut host), ("buzzer", &mut buzzer)];
    while host_view["phase"] != "released" {
        let action = host_view["primary"]["action"].as_str().unwrap().to_string();
        assert_eq!(w.act(&room, &action).await, StatusCode::OK, "{action}");
        revision += 1;
        host_view = one_frame_each(&w, &room, &mut all, revision, &action).await.swap_remove(1);
    }
    drop(all);

    // Released: the sessions are gone with the room's answers, so the token
    // resolves to nothing — on the socket and on the answer route.
    let mut again = w.open(&room, "buzzer", Some(&token)).await;
    assert_eq!(closed_with(&mut again, WAIT).await, Some(close::UNAUTHORIZED));
    let (status, _) = http(&w.app, Method::PUT, &uri, Some(&token), Some(serde_json::json!({ "letter": "A" }))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_second_socket_for_a_real_session_replaces_the_first() {
    let w = Wired::start().await;
    let room = host_room(&w.app).await;
    let (_, joined) = http(&w.app, Method::POST, "/join", None, Some(serde_json::json!({ "code": room.code }))).await;
    let token = joined["token"].as_str().unwrap();
    let mut old = w.open(&room, "buzzer", Some(token)).await;
    frame(&mut old, WAIT).await;
    let mut new = w.open(&room, "buzzer", Some(token)).await;
    frame(&mut new, WAIT).await;
    assert_eq!(closed_with(&mut old, WAIT).await, Some(close::REPLACED));

    // Two joins are two sessions: each resolves to its own socket.
    let (_, other) = http(&w.app, Method::POST, "/join", None, Some(serde_json::json!({ "code": room.code }))).await;
    let mut third = w.open(&room, "buzzer", Some(other["token"].as_str().unwrap())).await;
    frame(&mut third, WAIT).await;
    // `new` hears the second join's `present` and nothing else: not replaced.
    assert_eq!(frame(&mut new, WAIT).await["phase"], "idle");
    assert!(matches!(next(&mut new, QUIET).await, Next::Nothing), "another session does not replace this one");
}
