//! The transport, at small scale, over a real loopback socket (T-04c).
//!
//! - AC-81: the wall, the host and three buzzers each get exactly one frame
//!   per revision, and every frame of a revision carries the same `phase`.
//! - AC-37: a buzzer that drops and re-attaches with the same token gets the
//!   room's current state as its first frame, with its saved answer.
//! - A second socket for a session replaces the first; bad credentials and
//!   late attaches are refused with a close code and no state.
//!
//! The same criteria at 200 buzzers are `tests/transport_full.rs`, which only
//! `just test-full` runs.

mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::http::{Method, StatusCode};
use common::*;
use futures_util::SinkExt;
use room::question::Letter;
use room::rooms::{AppState, Urls};
use room::ws::close;
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

const WAIT: Duration = Duration::from_secs(2);
/// Long enough that a stray frame would have arrived on loopback.
const QUIET: Duration = Duration::from_millis(300);

/// The frame less the transport's own fields: what the viewer's projection is.
fn payload(frame: &Value) -> Value {
    let mut v = frame.clone();
    let map = v.as_object_mut().unwrap();
    map.remove("t");
    map.remove("revision");
    map.remove("session");
    v
}

async fn projection(live: &Live, room: &LiveRoom, viewer: &str) -> Value {
    let bearer = (viewer == "host").then_some(room.host.as_str());
    let (status, v) = live
        .call(Method::GET, &format!("/rooms/{}/{viewer}", room.id), bearer, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    v
}

async fn room_revision(live: &Live, room: &LiveRoom) -> u64 {
    live.state.with_room(&room.id, |r| r.revision()).unwrap()
}

/// Wait (bounded) for a condition the server reaches asynchronously.
async fn eventually(what: &str, mut ok: impl FnMut() -> bool) {
    for _ in 0..100 {
        if ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("never happened: {what}");
}

#[tokio::test]
async fn every_viewer_gets_one_frame_per_revision_and_they_agree_on_the_phase() {
    let live = Live::start().await;
    let room = live.create().await;
    for (i, token) in ["b-1", "b-2", "b-3"].iter().enumerate() {
        live.tokens.add(token, i as u64, None);
    }
    let mut sockets = [
        ("wall", live.open(&room, "wall", None).await),
        ("host", live.open(&room, "host", Some(&room.host)).await),
        ("buzzer", live.open(&room, "buzzer", Some("b-1")).await),
        ("buzzer", live.open(&room, "buzzer", Some("b-2")).await),
        ("buzzer", live.open(&room, "buzzer", Some("b-3")).await),
    ];

    // The attach frames: current state, one revision, one phase.
    let mut revision = room_revision(&live, &room).await;
    let mut host = Value::Null;
    for (viewer, socket) in sockets.iter_mut() {
        let f = frame(socket, WAIT).await;
        assert_eq!(f["t"], "state");
        assert_eq!(f["revision"], revision, "{viewer}");
        assert_eq!(f["phase"], "idle", "{viewer}");
        assert_eq!(payload(&f), projection(&live, &room, viewer).await, "{viewer}");
        if *viewer == "host" {
            host = f;
        }
    }

    // Drive the room from idle to released by the host's own primary action,
    // stepping the trace once each way where the host screen allows it.
    let mut phases = Vec::new();
    let mut stepped = std::collections::HashSet::new();
    while host["phase"] != "released" {
        let phase = host["phase"].as_str().unwrap().to_string();
        let action = if host["step"]["can_forward"] == true && stepped.insert((phase.clone(), "fwd")) {
            "step-forward".to_string()
        } else if host["step"]["can_back"] == true && stepped.insert((phase.clone(), "back")) {
            "step-back".to_string()
        } else {
            host["primary"]["action"].as_str().unwrap().to_string()
        };
        assert_eq!(live.act(&room, &action).await, StatusCode::OK, "{action} in {phase}");

        let mut seen = Vec::new();
        for (viewer, socket) in sockets.iter_mut() {
            let f = frame(socket, WAIT).await;
            assert_eq!(f["revision"], revision + 1, "{viewer} after {action}: one frame per revision");
            assert_eq!(payload(&f), projection(&live, &room, viewer).await, "{viewer} after {action}");
            seen.push(f["phase"].clone());
            if *viewer == "host" {
                host = f;
            }
        }
        assert!(seen.iter().all(|p| *p == seen[0]), "after {action}: phases disagree: {seen:?}");
        if phases.last() != Some(&seen[0]) {
            phases.push(seen[0].clone());
        }
        revision += 1;
    }
    assert_eq!(
        phases,
        ["live", "closed", "split", "work", "reveal", "released"].map(Value::from).to_vec()
    );
    assert!(stepped.len() >= 3, "the trace was stepped in work and reveal: {stepped:?}");

    // A refused action changes nothing, so nothing is sent — to anyone.
    assert_eq!(live.act(&room, "reveal").await, StatusCode::CONFLICT);
    for (viewer, socket) in sockets.iter_mut() {
        assert!(matches!(next(socket, QUIET).await, Next::Nothing), "{viewer}: a frame without a revision");
    }
}

#[tokio::test]
async fn a_buzzer_that_reconnects_gets_current_state_first_with_its_saved_answer() {
    let live = Live::start().await;
    let room = live.create().await;
    live.tokens.add("tok", 7, Some(Letter::B));
    assert_eq!(live.act(&room, "put-on-screen").await, StatusCode::OK);

    let mut buzzer = live.open(&room, "buzzer", Some("tok")).await;
    let first = frame(&mut buzzer, WAIT).await;
    assert_eq!(first["phase"], "live");
    assert_eq!(first["session"]["saved"], "B");

    // The phone drops; the map is told once.
    buzzer.close(None).await.unwrap();
    eventually("gone reported", || live.tokens.gone_count(7) == 1).await;

    // The room moves on while it is away.
    assert_eq!(live.act(&room, "close-answers").await, StatusCode::OK);

    // Back with the same token: the first frame is the room as it is now.
    let mut buzzer = live.open(&room, "buzzer", Some("tok")).await;
    let first = frame(&mut buzzer, WAIT).await;
    assert_eq!(first["t"], "state");
    assert_eq!(first["phase"], "closed");
    assert_eq!(first["locked"], true);
    assert_eq!(first["revision"], room_revision(&live, &room).await);
    assert_eq!(first["session"]["saved"], "B", "the saved answer survives the reconnect");

    // And it is subscribed again.
    assert_eq!(live.act(&room, "show-split").await, StatusCode::OK);
    let f = frame(&mut buzzer, WAIT).await;
    assert_eq!(f["phase"], "split");
    assert!(f.get("session").is_none(), "only the attach frame is personal");
}

#[tokio::test]
async fn a_second_socket_for_the_same_session_replaces_the_first() {
    let live = Live::start().await;
    let room = live.create().await;
    live.tokens.add("tok", 3, None);

    let mut old = live.open(&room, "buzzer", Some("tok")).await;
    frame(&mut old, WAIT).await;
    let mut new = live.open(&room, "buzzer", Some("tok")).await;
    frame(&mut new, WAIT).await;

    assert_eq!(closed_with(&mut old, WAIT).await, Some(close::REPLACED));
    assert_eq!(live.act(&room, "put-on-screen").await, StatusCode::OK);
    assert_eq!(frame(&mut new, WAIT).await["phase"], "live");
    assert_eq!(live.tokens.gone_count(3), 0, "a replaced socket does not report the session gone");

    new.close(None).await.unwrap();
    eventually("the current socket reports gone", || live.tokens.gone_count(3) == 1).await;
}

#[tokio::test]
async fn several_host_devices_attach_at_once() {
    let live = Live::start().await;
    let room = live.create().await;
    let mut a = live.open(&room, "host", Some(&room.host)).await;
    let mut b = live.open(&room, "host", Some(&room.host)).await;
    frame(&mut a, WAIT).await;
    frame(&mut b, WAIT).await;
    assert_eq!(live.act(&room, "put-on-screen").await, StatusCode::OK);
    assert_eq!(frame(&mut a, WAIT).await["phase"], "live");
    assert_eq!(frame(&mut b, WAIT).await["phase"], "live");
}

#[tokio::test]
async fn bad_credentials_are_closed_with_no_state() {
    let live = Live::start().await;
    let room = live.create().await;
    live.tokens.add("tok", 1, None);

    let mut host = live.open(&room, "host", Some("not-the-host-session")).await;
    assert_eq!(closed_with(&mut host, WAIT).await, Some(close::UNAUTHORIZED));

    let mut buzzer = live.open(&room, "buzzer", Some("no-such-token")).await;
    assert_eq!(closed_with(&mut buzzer, WAIT).await, Some(close::UNAUTHORIZED));

    let mut garbled = live.open(&room, "buzzer", None).await;
    garbled.send(Message::text("tok")).await.unwrap();
    assert_eq!(closed_with(&mut garbled, WAIT).await, Some(close::UNAUTHORIZED));

    let missing = LiveRoom {
        id: "no-such-room".into(),
        host: String::new(),
    };
    let err = tokio_tungstenite::connect_async(live.url(&missing, "wall")).await.unwrap_err();
    assert!(err.to_string().contains("404"), "{err}");
}

#[tokio::test]
async fn a_socket_that_never_attaches_is_closed() {
    let live = Live::start_with(|t| t.with_attach_timeout(Duration::from_millis(100))).await;
    let room = live.create().await;
    let mut buzzer = live.open(&room, "buzzer", None).await;
    assert_eq!(closed_with(&mut buzzer, WAIT).await, Some(close::ATTACH_TIMEOUT));
    let mut host = live.open(&room, "host", None).await;
    assert_eq!(closed_with(&mut host, WAIT).await, Some(close::ATTACH_TIMEOUT));
}

#[tokio::test]
async fn a_room_nobody_watches_keeps_no_broadcast() {
    let live = Live::start().await;
    let room = live.create().await;
    live.tokens.add("tok", 1, None);
    let mut wall = live.open(&room, "wall", None).await;
    let mut buzzer = live.open(&room, "buzzer", Some("tok")).await;
    frame(&mut wall, WAIT).await;
    frame(&mut buzzer, WAIT).await;
    assert_eq!(live.transport.broadcasting(), 1);

    wall.close(None).await.unwrap();
    buzzer.close(None).await.unwrap();
    eventually("the broadcast is dropped", || live.transport.broadcasting() == 0).await;

    // A poke for a room nobody watches builds nothing.
    assert_eq!(live.act(&room, "put-on-screen").await, StatusCode::OK);
    assert_eq!(live.transport.broadcasting(), 0);
}

#[tokio::test]
/// The default router is wired to the real session map (PQ-32): walls are
/// served, and a token no join handed out is refused.
async fn the_default_router_serves_walls_and_refuses_unknown_tokens() {
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default()));
    let app = room::router_with(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(room::ws::serve(listener, app));
    let id = q3_json()["id"].as_str().unwrap().to_string();
    let created = state
        .create_room(Some(ORGANIZER), &id, std::time::SystemTime::now())
        .unwrap();

    let url = |viewer: &str| format!("ws://{addr}/rooms/{}/ws/{viewer}", created.id);
    let (mut wall, _) = tokio_tungstenite::connect_async(url("wall")).await.unwrap();
    assert_eq!(frame(&mut wall, WAIT).await["phase"], "idle");

    let (mut buzzer, _) = tokio_tungstenite::connect_async(url("buzzer")).await.unwrap();
    buzzer
        .send(Message::text(r#"{"t":"attach","token":"anything"}"#))
        .await
        .unwrap();
    assert_eq!(closed_with(&mut buzzer, WAIT).await, Some(close::UNAUTHORIZED));
}
