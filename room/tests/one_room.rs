//! GAP-8 (G-3): one room per question. The client's ruling of 2026-10-05:
//! *"no second room. Refuse creating a room on a question while another
//! unreleased room holds it."*
//!
//! Two rooms on one question leak: releasing either puts its answer on `/last`
//! — one take-home for the whole machine — while the other is still before its
//! reveal (the audit's Probe P1). A room holds its question while it is not
//! released and has not ended; a released room's question is refused by the
//! used ledger instead (G-10), and a room that went quiet or expired can never
//! release, so it holds nothing.

mod common;

use std::sync::{Arc, Barrier};
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use common::*;
use room::lifecycle::{Clock, IDLE_QUIET, LATER_QUIET};
use room::phase::{Command, HostAction, Phase};
use room::rooms::{RoomError, ROOM_LIFETIME};
use serde_json::{json, Value};
use tower::ServiceExt;

const HELD_REFUSAL: &str = "That question is open in another room. Pick another.";
const USED_REFUSAL: &str = "That question has already been run. Pick another.";

fn held() -> Option<RoomError> {
    Some(RoomError::Refused(HELD_REFUSAL.into()))
}

fn used() -> Option<RoomError> {
    Some(RoomError::Refused(USED_REFUSAL.into()))
}

/// `create_room` on `c`'s clock, the error only.
fn try_create(c: &Clocked, question_id: &str) -> Option<RoomError> {
    c.state.create_room(Some(ORGANIZER), question_id, c.clock.now()).err()
}

// --------------------------------------------------------------------------
// Probe P1, kept: the second room is refused, and /last stays empty.
// --------------------------------------------------------------------------

#[test]
fn gap8_a_second_room_on_a_live_question_is_refused_and_last_stays_empty() {
    let c = Clocked::new();
    let first = c.create("q3");
    c.walk(&first, &[HostAction::PutOnScreen]);

    assert_eq!(try_create(&c, "q3"), held());
    assert_eq!(c.state.room_count(), 1, "no room was made");
    assert!(c.state.take_home().is_none(), "nothing is on /last while the room is live");
    assert!(c.state.used().all().is_empty());

    // The one room runs to release; only then is there a take-home.
    c.walk(&first, &TO_REVEAL[1..]);
    assert!(c.state.take_home().is_none(), "nor at its reveal");
    c.walk(&first, &[HostAction::ReleaseRoom]);
    assert_eq!(c.state.take_home().unwrap().question_id, "q3");
    assert_eq!(c.state.used().all().len(), 1);
}

#[test]
fn gap8_a_refused_second_room_leaves_the_earlier_take_home_in_place() {
    let c = Clocked::new();
    let earlier = c.create("q3-again");
    c.walk(&earlier, &TO_REVEAL);
    c.walk(&earlier, &[HostAction::ReleaseRoom]);
    let first = c.create("q3");
    c.walk(&first, &[HostAction::PutOnScreen]);

    assert_eq!(try_create(&c, "q3"), held());
    assert_eq!(c.state.take_home().unwrap().question_id, "q3-again");
}

#[test]
fn gap8_every_phase_before_release_holds_the_question() {
    let c = Clocked::new();
    let first = c.create("q3");
    assert_eq!(try_create(&c, "q3"), held(), "idle");
    for action in TO_REVEAL {
        c.walk(&first, &[action]);
        let phase = c.state.with_room(&first.id, |r| r.phase()).unwrap();
        assert_eq!(try_create(&c, "q3"), held(), "{phase:?}");
    }
    assert_eq!(c.state.with_room(&first.id, |r| r.phase()).unwrap(), Phase::Reveal);
    assert_eq!(c.state.room_count(), 1);
}

// --------------------------------------------------------------------------
// What does not hold: a released room (the used rule answers), an ended room.
// --------------------------------------------------------------------------

#[test]
fn gap8_after_release_the_used_rule_refuses_not_the_hold() {
    let c = Clocked::new();
    let first = c.create("q3");
    c.walk(&first, &TO_REVEAL);
    c.walk(&first, &[HostAction::ReleaseRoom]);
    assert_eq!(try_create(&c, "q3"), used());
    assert_eq!(c.state.run_again(&first.id, Some(ORGANIZER), "q3", c.clock.now()).err(), used());
}

#[test]
fn gap8_a_room_that_went_quiet_in_idle_holds_nothing() {
    let c = Clocked::new();
    let quiet = c.create("q3");
    c.advance(IDLE_QUIET - Duration::from_secs(1));
    assert_eq!(try_create(&c, "q3"), held(), "a second short of the bound");
    c.advance(Duration::from_secs(1));
    // Not swept yet: the record is still there, and still does not hold.
    assert_eq!(c.state.with_room(&quiet.id, |r| r.phase()).unwrap(), Phase::Idle);
    c.create("q3");
}

#[test]
fn gap8_a_room_that_went_quiet_after_start_holds_nothing() {
    let c = Clocked::new();
    let quiet = c.create("q3");
    c.walk(&quiet, &[HostAction::PutOnScreen, HostAction::CloseAnswers]);
    c.advance(LATER_QUIET);
    assert_eq!(c.act(&quiet, HostAction::ShowSplit), Err(RoomError::Ended(room::lifecycle::Ended::Inactive)));
    c.create("q3");
}

#[test]
fn gap8_an_ended_room_stays_ended_once_a_new_room_is_let_in() {
    // The new room is let in because the quiet one has ended. An action on
    // the quiet one carrying an earlier clock reading — a wall-clock step
    // back, or a request that read the clock just before the lock — must not
    // revive it, or there would be two live rooms on q3.
    let c = Clocked::new();
    let quiet = c.create("q3");
    let before = c.clock.now() + IDLE_QUIET - Duration::from_secs(1);
    c.advance(IDLE_QUIET);
    c.create("q3");
    let revive = Command::Host(HostAction::PutOnScreen);
    assert_eq!(
        c.state.act(&quiet.id, Some(&quiet.host_session), revive, before),
        Err(RoomError::Ended(room::lifecycle::Ended::Inactive))
    );
    assert_eq!(c.state.with_room(&quiet.id, |r| r.phase()).unwrap(), Phase::Idle);
}

#[test]
fn gap8_an_expired_room_holds_nothing() {
    let c = Clocked::new();
    let old = c.create("q3");
    c.advance(ROOM_LIFETIME);
    assert_eq!(
        c.state.with_room(&old.id, |r| r.ended(c.clock.now())).unwrap(),
        Some(room::lifecycle::Ended::Expired)
    );
    c.create("q3");
}

// --------------------------------------------------------------------------
// Run it again, and other questions.
// --------------------------------------------------------------------------

#[test]
fn gap8_run_it_again_onto_a_held_question_is_refused() {
    let c = Clocked::new();
    let source = c.create("q3-again");
    c.walk(&source, &TO_REVEAL);
    c.walk(&source, &[HostAction::ReleaseRoom]);
    let live = c.create("q3");
    c.walk(&live, &[HostAction::PutOnScreen]);

    assert_eq!(c.state.run_again(&source.id, Some(ORGANIZER), "q3", c.clock.now()).err(), held());
    assert_eq!(c.state.room_count(), 2, "no room was made");
    assert_eq!(c.state.with_room(&source.id, |r| r.phase()).unwrap(), Phase::Released);
}

#[test]
fn gap8_a_different_question_is_never_blocked() {
    let c = Clocked::new();
    let first = c.create("q3");
    c.walk(&first, &[HostAction::PutOnScreen]);
    c.create("q3-again");
    assert_eq!(c.state.room_count(), 2);
}

// --------------------------------------------------------------------------
// One critical section: the check and the insert under one lock.
// --------------------------------------------------------------------------

#[test]
fn gap8_concurrent_creates_on_one_question_make_one_room() {
    const THREADS: usize = 8;
    for round in 0..50 {
        let c = Clocked::new();
        let now = c.clock.now();
        let gate = Arc::new(Barrier::new(THREADS));
        let results: Vec<_> = (0..THREADS)
            .map(|_| {
                let state = c.state.clone();
                let gate = gate.clone();
                std::thread::spawn(move || {
                    gate.wait();
                    state.create_room(Some(ORGANIZER), "q3", now).err()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|t| t.join().unwrap())
            .collect();
        assert_eq!(results.iter().filter(|r| r.is_none()).count(), 1, "round {round}: {results:?}");
        assert!(results.iter().all(|r| r.is_none() || *r == held()), "round {round}: {results:?}");
        assert_eq!(c.state.room_count(), 1, "round {round}");
    }
}

/// The lock discipline, by inspection — a tripwire on the source text, not a
/// proof; a reshaped `create_for` must keep these properties and update it —
/// for the windows no test can schedule:
/// a release landing between the used check and the insert, and `schedule`
/// replacing the record between its lookup and the insert. `create_for` takes
/// `questions` then `rooms`, checks the hold and the ledger under both, and
/// lets neither go before the room is in.
#[test]
fn gap8_create_for_checks_and_inserts_in_one_critical_section() {
    let src = std::fs::read_to_string(repo().join("room/src/rooms.rs")).unwrap();
    let start = src.find("    fn create_for(").expect("create_for");
    let body = &src[start..start + src[start..].find("\n    }\n").expect("end of create_for")];
    let at = |needle: &str| body.find(needle).unwrap_or_else(|| panic!("create_for lost `{needle}`"));
    let questions = at("lock(&self.questions)");
    let rooms = at("lock(&self.rooms)");
    let hold = at(".holds(question_id, now)");
    let ledger = at("self.used.contains(question_id)");
    let insert = at("rooms.insert(");
    assert!(questions < rooms, "questions, then rooms (the order schedule takes)");
    assert!(rooms < hold && rooms < ledger, "both checks under the rooms lock");
    assert!(hold < insert && ledger < insert);
    assert_eq!(body.matches("lock(&self.rooms)").count(), 1, "one rooms lock, held from check to insert");
    assert_eq!(body.matches("lock(&self.questions)").count(), 1);
    let before_insert = &body[..insert];
    assert!(!before_insert.contains("drop(rooms)"), "rooms is held through the insert");
    assert!(!before_insert.contains("drop(questions)"), "questions is held through the insert");
    assert_eq!(body.matches("self.used.contains").count(), 1, "the ledger is checked once, under the lock");
}

// --------------------------------------------------------------------------
// Over HTTP: the refusal reaches the organizer as `reason`.
// --------------------------------------------------------------------------

async fn call(app: &Router, method: Method, uri: &str, bearer: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(b) = bearer {
        req = req.header(header::AUTHORIZATION, format!("Bearer {b}"));
    }
    let req = match body {
        Some(v) => req.header(header::CONTENT_TYPE, "application/json").body(Body::from(v.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into()));
    (status, json)
}

async fn post_create(app: &Router, question: &str) -> (StatusCode, Value) {
    call(app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": question}))).await
}

#[tokio::test]
async fn gap8_a_second_create_answers_409_with_the_reason_and_last_stays_empty() {
    let c = Clocked::new();
    let app = room::router_with(c.state.clone());
    let (status, first) = post_create(&app, "q3").await;
    assert_eq!(status, StatusCode::CREATED, "{first}");
    let start = format!("/rooms/{}/{}", first["id"].as_str().unwrap(), Command::Host(HostAction::PutOnScreen).slug());
    let (status, _) = call(&app, Method::POST, &start, first["host_session"].as_str(), None).await;
    assert!(status.is_success(), "{status}");

    let (status, body) = post_create(&app, "q3").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["reason"], HELD_REFUSAL);

    let (status, last) = call(&app, Method::GET, "/last", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let last = last.as_str().unwrap();
    let empty = "<script type=\"application/json\" id=\"take-home\">null</script>";
    assert!(last.contains(empty), "/last carries nothing");

    // And the slot does fill at release, so the check above can tell.
    let host = first["host_session"].as_str();
    for action in TO_REVEAL[1..].iter().chain([HostAction::ReleaseRoom].iter()) {
        let uri = format!("/rooms/{}/{}", first["id"].as_str().unwrap(), Command::Host(*action).slug());
        let (status, body) = call(&app, Method::POST, &uri, host, None).await;
        assert!(status.is_success(), "{action:?}: {status} {body}");
    }
    let (_, last) = call(&app, Method::GET, "/last", None, None).await;
    assert!(!last.as_str().unwrap().contains(empty), "released: /last carries the question");
}

#[tokio::test]
async fn gap8_run_it_again_onto_a_held_question_answers_409_with_the_reason() {
    let c = Clocked::new();
    let source = c.create("q3-again");
    c.walk(&source, &TO_REVEAL);
    c.walk(&source, &[HostAction::ReleaseRoom]);
    c.create("q3");
    let app = room::router_with(c.state.clone());
    let (status, body) = call(
        &app,
        Method::POST,
        &format!("/rooms/{}/run-it-again", source.id),
        Some(ORGANIZER),
        Some(json!({"question_id": "q3"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["reason"], HELD_REFUSAL);
}
