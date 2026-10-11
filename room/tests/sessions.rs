//! T-04b: participant sessions and the answer store, driven through the router
//! in-process (AC-28, AC-29, AC-30, AC-34, AC-35, AC-36, AC-46, AC-56, AC-57),
//! plus an in-process reconciliation at 200 sessions that supplements — and
//! does not discharge — AC-52, which is `burst`'s against the deployed room.

mod common;

use std::sync::Arc;

use axum::http::{Method, StatusCode};
use axum::Router;
use common::*;
use room::copy;
use room::phase::HostAction;
use room::rooms::{AppState, Urls};
use room::sessions::{parse_code, JoinRefusal};
use serde_json::{json, Value};

struct Harness {
    app: Router,
    state: Arc<AppState>,
}

fn harness(capacity: Option<usize>) -> Harness {
    let mut state = AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default());
    if let Some(c) = capacity {
        state = state.with_capacity(c);
    }
    let state = Arc::new(state);
    Harness {
        app: room::router_with(state.clone()),
        state,
    }
}

async fn host(h: &Harness, room: &HostedRoom, action: HostAction) {
    let (status, v) = http(&h.app, Method::POST, &format!("/rooms/{}/{}", room.id, action.slug()), Some(&room.host), None).await;
    assert_eq!(status, StatusCode::OK, "{action:?}: {v}");
}

async fn join(h: &Harness, code: &str) -> (StatusCode, Value) {
    http(&h.app, Method::POST, "/join", None, Some(json!({ "code": code }))).await
}

async fn joined(h: &Harness, room: &HostedRoom) -> String {
    let (status, v) = join(h, &room.code).await;
    assert_eq!(status, StatusCode::CREATED, "{v}");
    assert_eq!(v["room_id"], room.id.as_str());
    v["token"].as_str().unwrap().to_string()
}

async fn answer(h: &Harness, room: &HostedRoom, token: &str, letter: &str) -> (StatusCode, Value) {
    http(&h.app, Method::PUT, &format!("/rooms/{}/answer", room.id), Some(token), Some(json!({ "letter": letter }))).await
}

/// The session's saved answer as the server holds it (`None` = no answer).
fn saved(h: &Harness, room: &HostedRoom, token: &str) -> Option<String> {
    let payload = h.state.buzzer_for(&room.id, token).expect("the session exists");
    serde_json::to_value(payload).unwrap()["yours"].as_str().map(String::from)
}

async fn host_view(h: &Harness, room: &HostedRoom) -> Value {
    let (status, v) = http(&h.app, Method::GET, &format!("/rooms/{}/host", room.id), Some(&room.host), None).await;
    assert_eq!(status, StatusCode::OK);
    v
}

async fn live_room(h: &Harness) -> HostedRoom {
    let room = host_room(&h.app).await;
    host(h, &room, HostAction::PutOnScreen).await;
    room
}

#[tokio::test]
async fn ac28_join_by_code_in_any_case_and_nothing_else() {
    let h = harness(None);
    let room = host_room(&h.app).await;
    let (status, v) = join(&h, &format!("  {}  ", room.code.to_lowercase())).await;
    assert_eq!(status, StatusCode::CREATED, "{v}");
    // The join hands back the token and the caller's buzzer view, in `idle`.
    assert_eq!(v["buzzer"]["phase"], "idle");
    assert_eq!(v["buzzer"]["lines"][0], copy::BUZZER_IDLE);
    assert_eq!(v["token"].as_str().unwrap().len(), 64);
    assert_eq!(v.as_object().unwrap().len(), 3, "room_id, token, buzzer: {v}");
}

#[tokio::test]
async fn ac29_each_failure_has_its_own_sentence() {
    let h = harness(Some(1));
    let room = host_room(&h.app).await;

    let expect = |v: &Value, r: JoinRefusal| {
        assert_eq!(v["refusal"], r.slug(), "{v}");
        assert_eq!(v["reason"], r.message(), "{v}");
    };

    for bad in ["", "ABC", "ABCDEFG", "ABCDE0", "ABCDEO", "ABCDE1", "ABCDEI", "ABC-EF"] {
        let (status, v) = join(&h, bad).await;
        assert_eq!(status, StatusCode::CONFLICT, "{bad:?}");
        expect(&v, JoinRefusal::Malformed);
    }

    let unknown = if room.code == "ZZZZZZ" { "YYYYYY" } else { "ZZZZZZ" };
    let (status, v) = join(&h, unknown).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    expect(&v, JoinRefusal::Unknown);

    joined(&h, &room).await;
    let (status, v) = join(&h, &room.code).await;
    assert_eq!(status, StatusCode::CONFLICT);
    expect(&v, JoinRefusal::Full);

    for action in [
        HostAction::PutOnScreen,
        HostAction::CloseAnswers,
        HostAction::ShowSplit,
        HostAction::WalkIt,
        HostAction::Reveal,
        HostAction::ReleaseRoom,
    ] {
        host(&h, &room, action).await;
    }
    let (status, v) = join(&h, &room.code).await;
    assert_eq!(status, StatusCode::CONFLICT);
    expect(&v, JoinRefusal::AlreadyEnded);

    // Six states, six sentences, six names — including the two no phase of
    // this machine reaches (not yet open; closed for inactivity, T-11's).
    let messages: std::collections::HashSet<_> = JoinRefusal::ALL.iter().map(|r| r.message()).collect();
    let slugs: std::collections::HashSet<_> = JoinRefusal::ALL.iter().map(|r| r.slug()).collect();
    assert_eq!((messages.len(), slugs.len()), (6, 6));
    assert_eq!(JoinRefusal::NotYetOpen.message(), copy::JOIN_FAIL_NOT_YET_OPEN);
    assert_eq!(JoinRefusal::ClosedForInactivity.message(), copy::JOIN_FAIL_CLOSED_INACTIVITY);
}

#[test]
fn ac29_code_shape() {
    assert_eq!(parse_code(" ab2c9z "), Some("AB2C9Z".into()));
    for bad in ["AB2C9", "AB2C9ZZ", "AB2C9O", "AB2C90", "AB2C9I", "AB2C91", "AB2C9é"] {
        assert_eq!(parse_code(bad), None, "{bad:?}");
    }
}

#[tokio::test]
async fn ac30_the_201st_join_is_refused_and_reserves_nothing() {
    let h = harness(None);
    let room = host_room(&h.app).await;
    for _ in 0..200 {
        joined(&h, &room).await;
    }
    let before = host_view(&h, &room).await;
    assert_eq!(before["present"], 200);

    let (status, v) = join(&h, &room.code).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(v["refusal"], "full");
    assert_eq!(v["reason"], copy::JOIN_FAIL_FULL);
    assert!(v.get("token").is_none());

    assert_eq!(h.state.session_count(&room.id).unwrap(), 200, "nothing reserved");
    assert_eq!(host_view(&h, &room).await, before, "nothing moved");
}

#[tokio::test]
async fn ac34_five_changes_then_close_then_refused_with_the_saved_answer() {
    let h = harness(None);
    let room = live_room(&h).await;
    let token = joined(&h, &room).await;
    for letter in ["A", "C", "B", "E", "D"] {
        let (status, v) = answer(&h, &room, &token, letter).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(v, json!({ "saved": letter }));
    }
    // Idempotent: the same letter again changes nothing.
    assert_eq!(answer(&h, &room, &token, "D").await, (StatusCode::OK, json!({ "saved": "D" })));
    assert_eq!(saved(&h, &room, &token).as_deref(), Some("D"));

    host(&h, &room, HostAction::CloseAnswers).await;
    let (status, v) = answer(&h, &room, &token, "A").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(v["saved"], "D", "the saved answer restated");
    assert_eq!(v["phase"], "closed");
    assert_eq!(v["reason"], copy::BUZZER_CLOSED);
    assert_eq!(saved(&h, &room, &token).as_deref(), Some("D"));

    host(&h, &room, HostAction::ShowSplit).await;
    let (_, wall) = http(&h.app, Method::GET, &format!("/rooms/{}/wall", room.id), None, None).await;
    let counts: Vec<u64> = wall["split"]["bars"].as_array().unwrap().iter().map(|b| b["count"].as_u64().unwrap()).collect();
    assert_eq!(counts, [0, 0, 0, 1, 0], "the last answer is the one counted");
}

#[tokio::test]
async fn answers_are_refused_before_live_too() {
    let h = harness(None);
    let room = host_room(&h.app).await;
    let token = joined(&h, &room).await;
    let (status, v) = answer(&h, &room, &token, "A").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!((&v["phase"], &v["saved"]), (&json!("idle"), &Value::Null));
    assert_eq!(saved(&h, &room, &token), None);
}

/// AC-35's server half: every write gets exactly one of four answers, and
/// whatever came back, the server holds the letter of the last `200`. The
/// rendering half — exactly one of saving / saved / failed on the phone — is
/// T-06's.
#[tokio::test]
async fn ac35_response_contract() {
    let h = harness(None);
    let room = live_room(&h).await;
    let token = joined(&h, &room).await;
    let mut last_saved: Option<String> = None;

    let script: [(&str, &str); 7] = [
        ("B", "token"),
        ("C", "token"),
        ("F", "token"),
        ("A", "stranger"),
        ("C", "token"),
        ("close", ""),
        ("E", "token"),
    ];
    for (letter, who) in script {
        if letter == "close" {
            host(&h, &room, HostAction::CloseAnswers).await;
            continue;
        }
        let bearer = if who == "token" { token.clone() } else { "0".repeat(64) };
        let (status, v) = answer(&h, &room, &bearer, letter).await;
        match status {
            StatusCode::OK => {
                assert_eq!(v.as_object().unwrap().len(), 1, "{v}");
                last_saved = v["saved"].as_str().map(String::from);
            }
            StatusCode::CONFLICT => assert_eq!(v["saved"].as_str().map(String::from), last_saved, "409 restates"),
            StatusCode::UNAUTHORIZED => assert_eq!(v, Value::Null, "401 says nothing"),
            StatusCode::BAD_REQUEST => assert!(v["reason"].is_string()),
            other => panic!("{letter}/{who}: unexpected {other}"),
        }
        assert_eq!(saved(&h, &room, &token), last_saved, "after {letter}/{who}");
    }
    assert_eq!(last_saved.as_deref(), Some("C"));
}

#[tokio::test]
async fn ac36_a_failed_write_leaves_the_previous_answer_intact() {
    let h = harness(None);
    let room = live_room(&h).await;
    let token = joined(&h, &room).await;
    assert_eq!(answer(&h, &room, &token, "B").await.0, StatusCode::OK);

    let uri = format!("/rooms/{}/answer", room.id);
    let failures = [
        http(&h.app, Method::PUT, &uri, Some(&token), Some(json!({ "letter": "F" }))).await.0,
        http(&h.app, Method::PUT, &uri, Some(&token), Some(json!({ "letter": "b" }))).await.0,
        http(&h.app, Method::PUT, &uri, Some(&token), Some(json!({ "choice": "C" }))).await.0,
        http(&h.app, Method::PUT, &uri, Some(&token), None).await.0,
        http(&h.app, Method::PUT, &uri, None, Some(json!({ "letter": "C" }))).await.0,
        http(&h.app, Method::PUT, "/rooms/nope/answer", Some(&token), Some(json!({ "letter": "C" }))).await.0,
    ];
    for status in failures {
        assert!(status.is_client_error(), "{status}");
    }
    assert_eq!(saved(&h, &room, &token).as_deref(), Some("B"));
}

#[tokio::test]
async fn ac46_counts_move_on_the_host_phone_while_live() {
    let h = harness(None);
    let room = live_room(&h).await;
    let counts = |v: &Value| (v["present"].as_u64().unwrap(), v["answered"].as_u64().unwrap());
    assert_eq!(counts(&host_view(&h, &room).await), (0, 0));

    let a = joined(&h, &room).await;
    let b = joined(&h, &room).await;
    assert_eq!(counts(&host_view(&h, &room).await), (2, 0));
    answer(&h, &room, &a, "A").await;
    assert_eq!(counts(&host_view(&h, &room).await), (2, 1));
    answer(&h, &room, &a, "C").await;
    answer(&h, &room, &b, "C").await;
    assert_eq!(counts(&host_view(&h, &room).await), (2, 2));
    assert!(h.state.leave(&room.id, &b).unwrap());
    assert_eq!(counts(&host_view(&h, &room).await), (1, 1));
    assert!(!h.state.leave(&room.id, &b).unwrap(), "a second leave finds nothing");

    // After close `present` keeps counting; `answered` is frozen.
    host(&h, &room, HostAction::CloseAnswers).await;
    joined(&h, &room).await;
    assert_eq!(counts(&host_view(&h, &room).await), (2, 1));
}

#[tokio::test]
async fn ghost_sessions_keep_their_answer_and_slot_until_they_leave() {
    let h = harness(Some(2));
    let room = live_room(&h).await;
    let ghost = joined(&h, &room).await;
    answer(&h, &room, &ghost, "E").await;
    joined(&h, &room).await;
    // The ghost's socket dropping is T-04c's business; until `leave`, the
    // session holds its slot and its answer.
    assert_eq!(join(&h, &room.code).await.1["refusal"], "full");
    assert_eq!(saved(&h, &room, &ghost).as_deref(), Some("E"));
    h.state.leave(&room.id, &ghost).unwrap();
    assert!(h.state.buzzer_for(&room.id, &ghost).is_err(), "gone for good");
    joined(&h, &room).await;
}

/// Supplements AC-52 in-process; AC-52 itself is `burst`'s, run against the
/// deployed room.
#[tokio::test]
async fn ac52_in_process_reconciliation() {
    let h = harness(None);
    let room = live_room(&h).await;
    let letters = ["A", "B", "C", "D", "E"];
    let mut finals = [0u64; 5];
    for i in 0..200usize {
        let token = joined(&h, &room).await;
        if i % 7 == 0 {
            continue; // some never answer
        }
        let mut last = i % 5;
        answer(&h, &room, &token, letters[last]).await;
        if i % 3 == 0 {
            last = (i / 3) % 5; // changed their mind
            answer(&h, &room, &token, letters[last]).await;
        }
        finals[last] += 1;
    }
    host(&h, &room, HostAction::CloseAnswers).await;
    host(&h, &room, HostAction::ShowSplit).await;
    let (_, wall) = http(&h.app, Method::GET, &format!("/rooms/{}/wall", room.id), None, None).await;
    let split = &wall["split"];
    let counts: Vec<u64> = split["bars"].as_array().unwrap().iter().map(|b| b["count"].as_u64().unwrap()).collect();
    assert_eq!(counts, finals, "each final answer counted exactly once");
    assert_eq!(split["answered"].as_u64().unwrap(), finals.iter().sum::<u64>());
    assert_eq!(split["present"], 200);
}

#[tokio::test]
async fn ac56_after_release_no_session_remains() {
    let h = harness(None);
    let room = live_room(&h).await;
    let token = joined(&h, &room).await;
    answer(&h, &room, &token, "A").await;
    for action in [
        HostAction::CloseAnswers,
        HostAction::ShowSplit,
        HostAction::WalkIt,
        HostAction::Reveal,
        HostAction::ReleaseRoom,
    ] {
        host(&h, &room, action).await;
    }
    assert_eq!(h.state.session_count(&room.id).unwrap(), 0);
    assert_eq!(answer(&h, &room, &token, "B").await, (StatusCode::UNAUTHORIZED, Value::Null));
    assert!(h.state.buzzer_for(&room.id, &token).is_err(), "no token resolves");
    assert_eq!(join(&h, &room.code).await.1["refusal"], "already_ended");
}

#[test]
fn ac57_a_session_is_a_token_and_an_answer() {
    // The compile-time half is the exhaustive pattern in `sessions.rs`; this
    // reads the source so the claim is visible in the test output too.
    let src = std::fs::read_to_string(repo().join("room/src/sessions.rs")).unwrap();
    let body = src.split("pub struct Session {").nth(1).expect("struct Session").split('}').next().unwrap();
    let fields: Vec<&str> = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .map(|l| l.trim_start_matches("pub ").split(':').next().unwrap().trim())
        .collect();
    assert_eq!(fields, ["token", "answer"]);
    assert!(src.contains("const _: fn(Session) = |Session { token: _, answer: _ }| {};"));
}

#[tokio::test]
async fn yours_is_the_callers_own_answer_and_only_on_its_own_paths() {
    let h = harness(None);
    let room = live_room(&h).await;
    let a = joined(&h, &room).await;
    let b = joined(&h, &room).await;
    answer(&h, &room, &a, "A").await;
    answer(&h, &room, &b, "B").await;
    assert_eq!(saved(&h, &room, &a).as_deref(), Some("A"));
    assert_eq!(saved(&h, &room, &b).as_deref(), Some("B"));
    let (_, public) = http(&h.app, Method::GET, &format!("/rooms/{}/buzzer", room.id), None, None).await;
    assert!(public.get("yours").is_none(), "{public}");
    // Nothing of b's reaches a's payload beyond what every buzzer has.
    let mine = serde_json::to_value(h.state.buzzer_for(&room.id, &a).unwrap()).unwrap();
    let mut without = mine.clone();
    without.as_object_mut().unwrap().remove("yours");
    assert_eq!(without, public);
}
