//! T-11: a room's whole life — four hours at most (AC-69), closed for
//! inactivity (§4.6), nothing per person after release (AC-56, G-4), no
//! participant identity anywhere (AC-57).
//!
//! Time is a `ManualClock`; nothing here waits.

mod common;

use std::time::Duration;

use axum::http::{Method, StatusCode};
use common::*;
use room::lifecycle::{Clock, Ended, IDLE_QUIET, LATER_QUIET};
use room::phase::{Command, HostAction, Phase, Step};
use room::question::Letter;
use room::rooms::{AnswerError, RoomError, ROOM_LIFETIME};
use room::sessions::JoinRefusal;
use room::view;
use room::ws::SessionTokens;
use serde_json::Value;

const SECOND: Duration = Duration::from_secs(1);
const MINUTE: Duration = Duration::from_secs(60);

fn join(c: &Clocked, code: &str) -> Result<String, JoinRefusal> {
    c.state.join(code).map(|j| j.token.as_str().to_string())
}

/// Keep the host busy in `work` until `until` from creation, one step every
/// fifteen minutes, so only the four-hour bound can end the room.
fn keep_busy(c: &Clocked, room: &room::rooms::Created, until: Duration) {
    let start = c.clock.now();
    let mut forward = true;
    while c.clock.now().duration_since(start).unwrap() + 15 * MINUTE < until {
        c.advance(15 * MINUTE);
        let step = if forward { Step::Forward } else { Step::Back };
        forward = !forward;
        c.state
            .act(&room.id, Some(&room.host_session), Command::Step(step), c.clock.now())
            .unwrap();
    }
}

// --------------------------------------------------------------------------
// AC-69: four hours from creation, then gone.
// --------------------------------------------------------------------------

#[test]
fn ac69_a_room_is_gone_at_four_hours() {
    let c = Clocked::new();
    let created = c.clock.now();
    let room = c.create("q3");
    let token = join(&c, &room.code).unwrap();
    c.walk(&room, &[HostAction::PutOnScreen]);
    c.state.answer(&room.id, &token, Letter::B).unwrap();
    c.walk(&room, &[HostAction::CloseAnswers, HostAction::ShowSplit, HostAction::WalkIt]);
    keep_busy(&c, &room, ROOM_LIFETIME - SECOND);
    c.clock.set(created + ROOM_LIFETIME - SECOND);
    assert!(join(&c, &room.code).is_ok(), "one second before four hours the room is open");

    c.clock.set(created + ROOM_LIFETIME);
    // Refused on every touch before any sweep has run.
    assert_eq!(join(&c, &room.code), Err(JoinRefusal::AlreadyEnded));
    assert_eq!(c.act(&room, HostAction::Reveal), Err(RoomError::Ended(Ended::Expired)));
    assert_eq!(c.state.answer(&room.id, &token, Letter::A), Err(AnswerError::Ended(Ended::Expired)));
    assert_eq!(c.state.buzzer_for(&room.id, &token).err(), Some(RoomError::Ended(Ended::Expired)));
    assert_eq!(c.state.room_count(), 1, "nothing deletes but the sweep");

    // The sweep deletes it whole: record, sessions, totals.
    assert_eq!(c.state.sweep(c.clock.now()), vec![room.id.clone()]);
    assert_eq!(c.state.room_count(), 0);
    assert_eq!(c.state.session_count(&room.id), Err(RoomError::NotFound));
    assert!(c.state.with_room(&room.id, |r| r.public().frozen).is_err(), "totals die with the room");
    assert_eq!(c.state.resolve(&room.id, &token), None);
    assert_eq!(c.state.ended_room(&room.id), Some(Ended::Expired));
    // A late join is told it ended, not that it never existed (AC-29).
    assert_eq!(join(&c, &room.code), Err(JoinRefusal::AlreadyEnded));
    // And it was never released, so it recorded nothing (AC-92).
    assert!(c.state.used().all().is_empty());
    assert!(c.state.take_home().is_none());
}

#[test]
fn ac69_the_bound_is_from_creation_whatever_the_host_does() {
    let c = Clocked::new();
    let created = c.clock.now();
    let room = c.create("q3");
    c.walk(&room, &[HostAction::PutOnScreen, HostAction::CloseAnswers, HostAction::ShowSplit, HostAction::WalkIt]);
    keep_busy(&c, &room, ROOM_LIFETIME);
    c.clock.set(created + ROOM_LIFETIME);
    assert_eq!(c.state.with_room(&room.id, |r| r.ended(c.clock.now())).unwrap(), Some(Ended::Expired));
}

#[test]
fn a_released_room_keeps_its_shell_until_four_hours_and_never_goes_quiet() {
    let c = Clocked::new();
    let created = c.clock.now();
    let room = c.create("q3");
    c.walk(&room, &TO_REVEAL);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    c.clock.set(created + 3 * 60 * MINUTE);
    assert!(c.state.sweep(c.clock.now()).is_empty(), "released is over, not quiet");
    assert_eq!(c.state.room_count(), 1);
    assert_eq!(join(&c, &room.code), Err(JoinRefusal::AlreadyEnded));
    c.clock.set(created + ROOM_LIFETIME);
    assert_eq!(c.state.sweep(c.clock.now()), vec![room.id.clone()]);
    assert_eq!(c.state.room_count(), 0);
    assert_eq!(join(&c, &room.code), Err(JoinRefusal::AlreadyEnded));
    // What outlives it is untouched by its deletion.
    assert_eq!(c.state.used().all().len(), 1);
    assert!(c.state.take_home().is_some());
}

#[test]
fn an_ended_room_is_forgotten_after_its_memory() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.advance(IDLE_QUIET);
    c.state.sweep(c.clock.now());
    assert_eq!(join(&c, &room.code), Err(JoinRefusal::ClosedForInactivity));
    c.advance(room::lifecycle::ENDED_MEMORY);
    c.state.sweep(c.clock.now());
    assert_eq!(c.state.ended_room(&room.id), None);
    assert_eq!(join(&c, &room.code), Err(JoinRefusal::Unknown));
}

// --------------------------------------------------------------------------
// §4.6: closed for inactivity.
// --------------------------------------------------------------------------

#[test]
fn an_idle_room_closes_after_thirty_quiet_minutes() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.advance(IDLE_QUIET - SECOND);
    assert!(join(&c, &room.code).is_ok());
    assert!(c.state.sweep(c.clock.now()).is_empty());
    c.advance(SECOND);
    assert_eq!(join(&c, &room.code), Err(JoinRefusal::ClosedForInactivity));
    assert_eq!(c.act(&room, HostAction::PutOnScreen), Err(RoomError::Ended(Ended::Inactive)));
    assert_eq!(c.state.sweep(c.clock.now()), vec![room.id.clone()]);
    assert_eq!(c.state.room_count(), 0);
    assert_eq!(c.state.ended_room(&room.id), Some(Ended::Inactive));
    assert_eq!(join(&c, &room.code), Err(JoinRefusal::ClosedForInactivity));
    assert!(c.state.used().all().is_empty(), "a room that never reaches release records nothing");
}

#[test]
fn a_later_phase_closes_after_twenty_quiet_minutes() {
    for phase_actions in [
        &TO_REVEAL[..1],
        &TO_REVEAL[..2],
        &TO_REVEAL[..3],
        &TO_REVEAL[..4],
        &TO_REVEAL[..],
    ] {
        let c = Clocked::new();
        let room = c.create("q3");
        let token = join(&c, &room.code).unwrap();
        c.walk(&room, phase_actions);
        c.advance(LATER_QUIET - SECOND);
        assert!(c.state.buzzer_for(&room.id, &token).is_ok(), "{phase_actions:?}");
        c.advance(SECOND);
        assert_eq!(
            c.state.buzzer_for(&room.id, &token).err(),
            Some(RoomError::Ended(Ended::Inactive)),
            "{phase_actions:?}"
        );
        assert_eq!(c.state.sweep(c.clock.now()), vec![room.id.clone()], "{phase_actions:?}");
    }
}

#[test]
fn a_host_action_resets_the_quiet_clock() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.advance(IDLE_QUIET - MINUTE);
    c.walk(&room, &[HostAction::PutOnScreen]);
    c.advance(LATER_QUIET - MINUTE);
    c.walk(&room, &[HostAction::CloseAnswers]);
    c.advance(LATER_QUIET - MINUTE);
    assert!(c.state.sweep(c.clock.now()).is_empty());
    assert!(c.act(&room, HostAction::ShowSplit).is_ok());
}

#[test]
fn a_refused_command_is_not_activity() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.advance(10 * MINUTE);
    assert!(matches!(c.act(&room, HostAction::Reveal), Err(RoomError::Refused(_))));
    c.advance(IDLE_QUIET - 10 * MINUTE);
    assert_eq!(c.state.with_room(&room.id, |r| r.ended(c.clock.now())).unwrap(), Some(Ended::Inactive));
}

#[test]
fn a_wrong_credential_on_an_ended_room_learns_nothing() {
    let c = Clocked::new();
    let room = c.create("q3");
    c.advance(IDLE_QUIET);
    assert_eq!(
        c.state
            .act(&room.id, Some("not-the-host"), Command::Host(HostAction::PutOnScreen), c.clock.now()),
        Err(RoomError::Denied)
    );
}

// --------------------------------------------------------------------------
// The routes: ended rooms answer with PQ-8's refusal states.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ended_rooms_answer_through_the_routes() {
    let c = Clocked::new();
    let app = room::router_with(c.state.clone());
    let room = c.create("q3");
    let token = join(&c, &room.code).unwrap();
    c.advance(IDLE_QUIET);

    let (status, v) = http(&app, Method::POST, "/join", None, Some(serde_json::json!({"code": room.code}))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(v["refusal"], "closed_for_inactivity");
    assert_eq!(v["reason"], JoinRefusal::ClosedForInactivity.message());

    let (status, v) = http(&app, Method::POST, &format!("/rooms/{}/put-on-screen", room.id), Some(&room.host_session), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{v}");
    assert_eq!(v["refusal"], "closed_for_inactivity");
    assert_eq!(v["reason"], Ended::Inactive.host_reason());

    let (status, v) = http(
        &app,
        Method::PUT,
        &format!("/rooms/{}/answer", room.id),
        Some(&token),
        Some(serde_json::json!({"letter": "A"})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(v["refusal"], "closed_for_inactivity");

    c.state.sweep(c.clock.now());
    let (status, v) = http(&app, Method::POST, "/join", None, Some(serde_json::json!({"code": room.code}))).await;
    assert_eq!((status, v["refusal"].as_str()), (StatusCode::CONFLICT, Some("closed_for_inactivity")));
    // After the sweep the room is gone: its id is unknown, like any other.
    let (status, _) = http(&app, Method::POST, &format!("/rooms/{}/put-on-screen", room.id), Some(&room.host_session), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_routes_read_the_one_clock() {
    let c = Clocked::new();
    let app = room::router_with(c.state.clone());
    let (status, created) = http(&app, Method::POST, "/rooms", Some(ORGANIZER), Some(serde_json::json!({"question_id": "q3"}))).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap();
    let expires = c.state.with_room(id, |r| r.expires_at()).unwrap();
    assert_eq!(expires, c.clock.now() + ROOM_LIFETIME, "created_at is the clock's, not the OS's");
}

/// The sweep hands its ids to the transport, which closes the room's sockets
/// with `4404`; the wall keeps its last frame (wall.js).
#[tokio::test]
async fn a_swept_room_closes_its_sockets() {
    use room::ws::Transport;
    let c = Clocked::new();
    let transport = Transport::new(c.state.clone(), c.state.clone());
    let app = room::router_with(c.state.clone()).layer(axum::Extension(transport.clone()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(room::ws::serve(listener, app));
    let room = c.create("q3");
    let (mut wall, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/rooms/{}/ws/wall", room.id))
        .await
        .unwrap();
    assert_eq!(frame(&mut wall, Duration::from_secs(2)).await["phase"], "idle");

    c.advance(ROOM_LIFETIME);
    for id in c.state.sweep(c.clock.now()) {
        transport.changed(&id);
    }
    assert_eq!(closed_with(&mut wall, Duration::from_secs(2)).await, Some(room::ws::close::ROOM_GONE));
}

// --------------------------------------------------------------------------
// AC-56 / G-4: nothing per person survives release.
// --------------------------------------------------------------------------

#[test]
fn ac56_nothing_per_person_survives_release() {
    let c = Clocked::new();
    let room = c.create("q3");
    let tokens: Vec<String> = (0..3).map(|_| join(&c, &room.code).unwrap()).collect();
    c.walk(&room, &[HostAction::PutOnScreen]);
    for (t, l) in tokens.iter().zip([Letter::A, Letter::B, Letter::B]) {
        c.state.answer(&room.id, t, l).unwrap();
    }
    c.walk(&room, &TO_REVEAL[1..]);
    assert!(c.state.with_room(&room.id, |r| r.public().frozen).unwrap().is_some());

    c.walk(&room, &[HostAction::ReleaseRoom]);

    // By inspection of the state, not by the absence of a route.
    assert_eq!(c.state.session_count(&room.id).unwrap(), 0, "every session is gone");
    let (frozen, present, answered_live) = c
        .state
        .with_room(&room.id, |r| {
            let v = r.public();
            (v.frozen, v.present, v.answered_live)
        })
        .unwrap();
    // The anonymous per-option totals stay with the shell until it expires
    // (AC-56: "only anonymous totals persist, and they expire with it").
    assert_eq!(frozen, Some((3, [1, 2, 0, 0, 0])), "the anonymous totals stay");
    assert_eq!((present, answered_live), (0, 0), "the live counts were the sessions'");
    for t in &tokens {
        assert_eq!(c.state.resolve(&room.id, t), None, "a token names nothing");
        assert_eq!(c.state.buzzer_for(&room.id, t).err(), Some(RoomError::Denied));
    }
    let host = c.state.with_room(&room.id, view::host).unwrap();
    assert_eq!((host.present, host.answered), (0, Some(3)), "the host still reads the frozen count");
}

#[test]
fn ac56_the_totals_expire_with_the_released_room() {
    let c = Clocked::new();
    let created = c.clock.now();
    let room = c.create("q3");
    let token = join(&c, &room.code).unwrap();
    c.walk(&room, &[HostAction::PutOnScreen]);
    c.state.answer(&room.id, &token, Letter::E).unwrap();
    c.walk(&room, &TO_REVEAL[1..]);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    assert_eq!(c.state.with_room(&room.id, |r| r.public().frozen).unwrap(), Some((1, [0, 0, 0, 0, 1])));
    assert_eq!(c.state.with_room(&room.id, |r| r.expires_at()).unwrap(), created + ROOM_LIFETIME);
    c.clock.set(created + ROOM_LIFETIME);
    assert_eq!(c.state.sweep(c.clock.now()), vec![room.id.clone()]);
    assert!(c.state.with_room(&room.id, |r| r.public().frozen).is_err(), "gone with the room");
}

#[test]
fn ac56_a_released_room_serves_no_question_content_and_no_count() {
    let c = Clocked::new();
    let room = c.create("q3");
    let token = join(&c, &room.code).unwrap();
    c.walk(&room, &[HostAction::PutOnScreen]);
    c.state.answer(&room.id, &token, Letter::C).unwrap();
    c.walk(&room, &TO_REVEAL[1..]);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    let urls = c.state.urls().clone();
    for viewer in view::Viewer::ALL {
        let v = c.state.with_room(&room.id, |r| view::project(r, viewer, &urls)).unwrap();
        assert_eq!(v["phase"], "released");
        for key in ["source", "options", "hint", "trace", "split", "counts", "read_aloud", "reveal", "correct", "step", "fit"] {
            assert!(keys_named(&v, key).is_empty(), "{viewer:?} carries {key}: {v}");
        }
    }
}

#[test]
fn ac56_the_survivors_carry_no_count() {
    let c = Clocked::new();
    let room = c.create("q3");
    let tokens: Vec<String> = (0..4).map(|_| join(&c, &room.code).unwrap()).collect();
    c.walk(&room, &[HostAction::PutOnScreen]);
    for t in &tokens {
        c.state.answer(&room.id, t, Letter::D).unwrap();
    }
    c.walk(&room, &TO_REVEAL[1..]);
    c.walk(&room, &[HostAction::ReleaseRoom]);

    let used = serde_json::to_value(c.state.used().all()).unwrap();
    let mut take_home = serde_json::to_value(c.state.take_home().unwrap()).unwrap();
    // The trace's line numbers are the program's, not the room's.
    take_home.as_object_mut().unwrap().remove("trace");
    for survivor in [&used, &take_home] {
        let mut all = Vec::new();
        walk(survivor, &mut all);
        for (k, v) in all {
            assert!(
                !["totals", "count", "counts", "answered", "present", "middle", "split", "percent", "most_chosen"].contains(&k),
                "a survivor carries {k}: {survivor}"
            );
            assert!(!v.is_number(), "a survivor carries a number under {k}: {survivor}");
        }
    }
}

// --------------------------------------------------------------------------
// AC-57: no participant identity anywhere.
// --------------------------------------------------------------------------

#[test]
fn ac57_no_participant_identity_in_any_record() {
    let words = [
        "nickname", "username", "user_name", "display_name", "email", "participant_id", "device", "device_id",
        "user_agent", "ip_addr", "remote_addr", "score", "scores", "leaderboard", "history", "streak",
    ];
    let dir = common::repo().join("room/src");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for (n, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap().trim();
            // A field or a binding: `name:` at the start of a declaration.
            let Some((head, _)) = code.split_once(':') else { continue };
            let field = head.trim().trim_start_matches("pub ").trim_start_matches("pub(crate) ").trim();
            assert!(
                !words.contains(&field),
                "{}:{}: a field named {field:?}",
                path.display(),
                n + 1
            );
        }
    }
}

#[test]
fn ac57_nothing_crosses_rooms_but_the_question() {
    // A token from one room names nothing in another.
    let c = Clocked::new();
    let a = c.create("q3");
    let b = c.create("q3-again");
    let token = join(&c, &a.code).unwrap();
    assert_eq!(c.state.resolve(&b.id, &token), None);
    assert!(c.state.buzzer_for(&b.id, &token).is_err());
    // The ledger's record names a room and a date, never a participant.
    c.walk(&a, &TO_REVEAL);
    c.walk(&a, &[HostAction::ReleaseRoom]);
    let used: Value = serde_json::to_value(c.state.used().all()).unwrap();
    let mut keys = Vec::new();
    walk(&used, &mut keys);
    let mut names: Vec<&str> = keys.into_iter().map(|(k, _)| k).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names, ["fit", "meetup_date", "question_id", "released_at", "room_id", "used"]);
}

// --------------------------------------------------------------------------
// `test-full`'s AC-69 row, shaped for T-10: with the auth provider down an
// open room runs to release; a new room cannot be created; a room dies at 4 h.
// --------------------------------------------------------------------------

#[test]
#[ignore = "test-full, T-10: needs the Discord mock"]
fn test_full_ac69_open_room_runs_to_release_with_auth_down() {
    // T-10: build the state on the Discord-backed HostAuth over a mock that
    // answers until `down()`, on a ManualClock.
    let c = Clocked::new();
    let room = c.create("q3");
    // T-10: mock.down();
    c.walk(&room, &TO_REVEAL);
    c.walk(&room, &[HostAction::ReleaseRoom]);
    assert_eq!(c.state.with_room(&room.id, |r| r.phase()).unwrap(), Phase::Released);
    // T-10: assert creation is refused while the mock is down.
    let created = c.state.create_room(Some(ORGANIZER), "q3-again", c.clock.now());
    assert!(created.is_err(), "T-10: a new room needs a live check");
    c.advance(ROOM_LIFETIME);
    c.state.sweep(c.clock.now());
    assert_eq!(c.state.room_count(), 0);
}
