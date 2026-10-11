//! AC-81 and AC-37 as `EVALUATION.md` writes them: the wall and 200 buzzers,
//! over real loopback sockets (T-04c).
//!
//! `#[ignore]`d so that `just test` never runs it; `just test-transport-full`
//! (a dependency of `test-full`) runs it with `--ignored`, after raising the
//! open-file limit — 201 sockets are two file descriptors each in one process.
//!
//! - AC-81: after every transition, the wall and all 200 buzzers each report
//!   the new phase as their very next frame, at the next revision — within
//!   one broadcast of the transition.
//! - AC-37: twenty buzzers drop while the room is `closed` and re-attach after
//!   it has moved to `split`; each one's first frame is the current state and
//!   carries its own saved answer.

mod common;

use std::time::Duration;

use axum::http::StatusCode;
use common::*;
use futures_util::future::join_all;
use room::question::Letter;
use serde_json::Value;

const BUZZERS: usize = 200;
const DROPPED: usize = 20;
const WAIT: Duration = Duration::from_secs(5);

fn token(i: usize) -> String {
    format!("session-{i:03}")
}

fn saved(i: usize) -> Letter {
    Letter::ALL[i % 5]
}

async fn revision(live: &Live, room: &LiveRoom) -> u64 {
    live.state.with_room(&room.id, |r| r.revision()).unwrap()
}

/// Every socket's next frame, read concurrently; asserts one phase and one
/// revision across all of them and returns the phase.
async fn one_broadcast(sockets: &mut [Socket], expect_revision: u64, what: &str) -> Value {
    let frames = join_all(sockets.iter_mut().map(|s| frame(s, WAIT))).await;
    for (i, f) in frames.iter().enumerate() {
        assert_eq!(f["revision"], expect_revision, "socket {i} after {what}");
        assert_eq!(f["phase"], frames[0]["phase"], "socket {i} after {what}: phases disagree");
    }
    frames[0]["phase"].clone()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "200 sockets: run by `just test-transport-full` inside `test-full`"]
async fn the_wall_and_200_buzzers_agree_within_one_broadcast_and_reconnects_resume() {
    let live = Live::start().await;
    let room = live.create().await;
    for i in 0..BUZZERS {
        live.tokens.add(&token(i), i as u64, Some(saved(i)));
    }

    // Index 0 is the wall; 1..=200 are buzzers.
    let mut sockets = vec![live.open(&room, "wall", None).await];
    let tokens: Vec<String> = (0..BUZZERS).map(token).collect();
    sockets.extend(join_all(tokens.iter().map(|t| live.open(&room, "buzzer", Some(t)))).await);
    let mut rev = revision(&live, &room).await;
    assert_eq!(one_broadcast(&mut sockets, rev, "attach").await, "idle");

    let mut transitions = vec![
        ("put-on-screen", "live"),
        ("close-answers", "closed"),
        // AC-37 happens here: twenty drop in `closed`, return in `split`.
        ("show-split", "split"),
        ("walk-it", "work"),
        ("step-forward", "work"),
        ("reveal", "reveal"),
        ("step-back", "reveal"),
        ("release", "released"),
    ]
    .into_iter();

    for (action, phase) in transitions.by_ref() {
        if action == "show-split" {
            break;
        }
        assert_eq!(live.act(&room, action).await, StatusCode::OK, "{action}");
        rev += 1;
        assert_eq!(one_broadcast(&mut sockets, rev, action).await, phase, "{action}");
    }

    // Twenty phones drop in `closed`.
    let dropped: Vec<Socket> = sockets.drain(1..=DROPPED).collect();
    join_all(dropped.into_iter().map(|mut s| async move { s.close(None).await.ok() })).await;
    for _ in 0..200 {
        if (0..DROPPED).all(|i| live.tokens.gone_count(i as u64) == 1) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!((0..DROPPED).all(|i| live.tokens.gone_count(i as u64) == 1), "every drop reported once");

    // The room moves on without them.
    assert_eq!(live.act(&room, "show-split").await, StatusCode::OK);
    rev += 1;
    assert_eq!(one_broadcast(&mut sockets, rev, "show-split").await, "split");

    // They come back with the same tokens: current state first, saved answer intact.
    let mut back = join_all(tokens[..DROPPED].iter().map(|t| live.open(&room, "buzzer", Some(t)))).await;
    let firsts = join_all(back.iter_mut().map(|s| frame(s, WAIT))).await;
    for (i, f) in firsts.iter().enumerate() {
        assert_eq!(f["phase"], "split", "re-attached {i}");
        assert_eq!(f["revision"], rev, "re-attached {i}");
        assert_eq!(f["session"]["saved"], saved(i).as_str(), "re-attached {i}: saved answer");
    }
    sockets.extend(back);
    assert_eq!(sockets.len(), 1 + BUZZERS);

    for (action, phase) in transitions {
        assert_eq!(live.act(&room, action).await, StatusCode::OK, "{action}");
        rev += 1;
        assert_eq!(one_broadcast(&mut sockets, rev, action).await, phase, "{action}");
    }
}
