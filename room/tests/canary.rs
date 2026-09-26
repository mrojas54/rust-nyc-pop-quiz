//! The `canary` seam — in-process, no socket.
//!
//! `EVALUATION.md`'s harness table splits the secrecy suite in two: the
//! in-process scan, which drives the router directly and runs inside `just
//! test`, and the scan of the deployed room's real frames and pages, which runs
//! inside `just test-full`. This file is the first half.
//!
//! It drives `router_with()` through **every phase in order**, for all three
//! viewers, and asserts per phase what G-3 and AC-47/AC-97 forbid before
//! `reveal`: no ✓, no receipt line, no explanation, no `why_tempting`, no
//! resolving step, no `values` entry named `stdout`, no trace step beyond
//! `M-2`, no hint before `live`; every option object exactly `{letter, text}`
//! in arrival order; the buzzer never carries source, trace or option text
//! (G-8). At `reveal` it asserts the plants **do** appear — without that, a
//! scanner that saw nothing would pass by default.
//!
//! The plants here are T-04a's seam, not T-08's scan. **T-08** replaces them
//! with its canary set (the resolving step's `note`, the explanation, the
//! receipt, the hint — see `tests/common/mod.rs`, `planted()`), and extends
//! `scan_pre_reveal` to the pages and socket frames. **T-25** adds
//! `POPQUIZ_ADMIN_TOKEN` as a fifth plant, asserted absent everywhere.
//!
//! Nothing here runs `rustc`, Miri or Docker, and nothing here opens a port.

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use common::*;
use room::copy;
use room::phase::{Command, HostAction, Phase, Step};
use room::rooms::{AppState, LiveCounts, Urls};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn the_router_can_be_driven_without_a_socket() {
    let response = room::router()
        .oneshot(
            Request::builder()
                .uri("/this-route-does-not-exist")
                .body(Body::empty())
                .expect("the request builder should accept a bare GET"),
        )
        .await
        .expect("driving the router in-process should not error");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

struct Harness {
    app: Router,
    state: Arc<AppState>,
    sessions: FakeSessions,
}

fn harness() -> Harness {
    let sessions = FakeSessions::default();
    let for_rooms = sessions.clone();
    let state = Arc::new(
        AppState::new(
            Arc::new(TestAuth),
            vec![load(&planted()), load(&planted_dnc()), q3()],
            Urls::default(),
        )
        .with_sessions(move || Box::new(for_rooms.clone())),
    );
    Harness {
        app: room::router_with(state.clone()),
        state,
        sessions,
    }
}

async fn call(app: &Router, method: Method, uri: &str, bearer: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(b) = bearer {
        req = req.header(header::AUTHORIZATION, format!("Bearer {b}"));
    }
    let req = match body {
        Some(v) => req
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let json = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() };
    (status, json)
}

struct Hosted {
    id: String,
    session: String,
}

async fn create(app: &Router, question: &str) -> Hosted {
    let (status, created) = call(app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": question}))).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    Hosted {
        id: created["id"].as_str().unwrap().into(),
        session: created["host_session"].as_str().unwrap().into(),
    }
}

async fn views(app: &Router, room: &Hosted) -> [(&'static str, Value); 3] {
    let mut out = Vec::new();
    for (viewer, bearer) in [("wall", None), ("buzzer", None), ("host", Some(room.session.as_str()))] {
        let (status, v) = call(app, Method::GET, &format!("/rooms/{}/{viewer}", room.id), bearer, None).await;
        assert_eq!(status, StatusCode::OK, "{viewer}");
        out.push((viewer, v));
    }
    out.try_into().unwrap()
}

async fn command(app: &Router, room: &Hosted, c: Command) -> (StatusCode, Value) {
    call(app, Method::POST, &format!("/rooms/{}/{}", room.id, c.slug()), Some(&room.session), None).await
}

/// The phase-scoped rules for one pre-reveal payload (G-3, G-8, AC-47, AC-97).
/// T-08 extends this to pages and socket frames.
fn scan_pre_reveal(viewer: &str, phase: Phase, v: &Value, m: u64, extra_plants: &[&str]) {
    let s = v.to_string();
    let at = format!("{viewer} in {phase:?}");

    assert!(!s.contains('✓'), "{at}: carries a ✓");
    assert!(!s.contains(copy::RECEIPT_HEADING), "{at}: carries the receipt");
    for line in [copy::RECEIPT_COMPILED, copy::RECEIPT_NOTHING_RAN, copy::RECEIPT_COMPILER_REFUSED] {
        assert!(!s.contains(line.trim_start_matches("✓ ")), "{at}: receipt line {line}");
    }
    for plant in [PLANT_RESOLVING_NOTE, PLANT_WHAT, PLANT_TAKEAWAY, PLANT_MIDDLE_STDOUT, PLANT_ERROR_CODE]
        .into_iter()
        .chain(extra_plants.iter().copied())
    {
        assert!(!s.contains(plant), "{at}: carries {plant}");
    }
    for letter in ["A", "B", "C", "D", "E"] {
        assert!(!s.contains(&plant_why(letter)), "{at}: carries why_tempting {letter}");
    }
    assert_eq!(stdout_rows(v), 0, "{at}: a values entry named stdout");
    for key in ["correct", "kind", "why_tempting", "explains", "receipt", "reveal", "read_aloud", "mark"] {
        assert!(keys_named(v, key).is_empty(), "{at}: has a {key:?} key");
    }

    // No trace position beyond M-2 (D-10).
    for key in ["at"] {
        for pos in keys_named(v, key) {
            assert!(pos.as_u64().unwrap() <= m - 2, "{at}: trace at {pos}");
        }
    }

    // The hint: in every buzzer's live payload, nowhere else (§4.2, AC-48, D-8).
    let hint_expected = viewer == "buzzer" && phase == Phase::Live;
    assert_eq!(s.contains(PLANT_HINT), hint_expected, "{at}: hint");

    // Every option object is exactly {letter, text}, A–E, in arrival order.
    let q = planted();
    for options in keys_named(v, "options") {
        let options = options.as_array().unwrap();
        assert_eq!(options.len(), 5, "{at}");
        for (i, o) in options.iter().enumerate() {
            let keys: Vec<&String> = o.as_object().unwrap().keys().collect();
            assert_eq!(keys, ["letter", "text"], "{at}: option {i}");
            assert_eq!(o["letter"], ["A", "B", "C", "D", "E"][i]);
            assert_eq!(o["text"], q["options"][i]["text"], "{at}: option order");
        }
    }
}

/// Everywhere, every phase (G-8, AC-50, AC-81).
fn scan_every_phase(viewer: &str, phase: Phase, v: &Value, session: &str) {
    let s = v.to_string();
    // One phase field, the room's (AC-81: the three projections read one value).
    assert_eq!(keys_named(v, "phase").len(), 1, "{viewer}: phase fields");
    assert_eq!(v["phase"], json!(phase_name(phase)), "{viewer}");
    if viewer != "host" {
        assert!(!s.contains(session), "{viewer} carries the host session");
        assert!(keys_named(v, "first_screen").is_empty());
    }
    if viewer == "buzzer" {
        let src = q3_json()["source"].as_str().unwrap().to_string();
        assert!(!s.contains(src.lines().nth(1).unwrap().trim()), "buzzer carries source");
        for key in ["source", "trace", "options", "step"] {
            assert!(keys_named(v, key).is_empty(), "buzzer has {key:?}");
        }
        for o in q3_json()["options"].as_array().unwrap() {
            let text = o["text"].as_str().unwrap();
            assert!(!s.contains(text), "buzzer carries option text {text}");
        }
    }
}

fn phase_name(p: Phase) -> &'static str {
    ["idle", "live", "closed", "split", "work", "reveal", "released"][p.index()]
}

/// Drive one room through all seven phases, scanning at every stop.
async fn drive_and_scan(question: &str, totals: [u32; 5]) -> Value {
    let h = harness();
    *h.sessions.totals.lock().unwrap() = totals;
    let room = create(&h.app, question).await;
    let m: u64 = 6;
    let mut reveal_views = Value::Null;

    for phase in Phase::ALL {
        let mut positions = vec![views(&h.app, &room).await];
        if phase == Phase::Live {
            h.state.set_live_counts(&room.id, LiveCounts { present: 20, answered_live: 12 }).unwrap();
        }
        if phase == Phase::Work {
            // Walk to the bound; one more is refused.
            loop {
                let (status, _) = command(&h.app, &room, Command::Step(Step::Forward)).await;
                if status == StatusCode::CONFLICT {
                    break;
                }
                assert_eq!(status, StatusCode::OK);
                positions.push(views(&h.app, &room).await);
            }
            assert_eq!(positions.len() as u64, m - 1, "work shows steps 0..=M-2");
        }
        for views in &positions {
            let phases: Vec<&Value> = views.iter().map(|(_, v)| &v["phase"]).collect();
            assert!(phases.iter().all(|p| *p == phases[0]), "the three projections disagree: {phases:?}");
            for (viewer, v) in views {
                scan_every_phase(viewer, phase, v, &room.session);
                if phase.index() < Phase::Reveal.index() {
                    scan_pre_reveal(viewer, phase, v, m, &[]);
                }
            }
        }
        if phase == Phase::Reveal {
            reveal_views = json!({
                "wall": positions[0][0].1, "buzzer": positions[0][1].1, "host": positions[0][2].1,
            });
            // Step back through the whole trace: the middle stdout row is there.
            let mut saw_middle_stdout = false;
            while command(&h.app, &room, Command::Step(Step::Back)).await.0 == StatusCode::OK {
                let wall = &views(&h.app, &room).await[0].1;
                saw_middle_stdout |= wall.to_string().contains(PLANT_MIDDLE_STDOUT);
            }
            assert!(saw_middle_stdout, "reveal may step the whole trace");
        }
        if phase == Phase::Released {
            let released = views(&h.app, &room).await;
            for (viewer, v) in &released {
                let s = v.to_string();
                for plant in [PLANT_RESOLVING_NOTE, PLANT_WHAT, PLANT_TAKEAWAY, PLANT_HINT] {
                    assert!(!s.contains(plant), "released {viewer} carries {plant}");
                }
            }
            break;
        }
        let (status, host) = command(&h.app, &room, Command::Host(phase.next_action())).await;
        assert_eq!(status, StatusCode::OK, "{phase:?}: {host}");
    }
    assert_eq!(*h.sessions.closes.lock().unwrap(), 1, "totals frozen once, at closed");
    reveal_views
}

#[tokio::test]
async fn every_pre_reveal_payload_is_clean_and_reveal_shows_the_plants() {
    // B is the room's most-chosen incorrect option.
    let reveal = drive_and_scan("planted", [2, 6, 1, 0, 3]).await;
    let (wall, buzzer, host) = (reveal["wall"].to_string(), reveal["buzzer"].to_string(), reveal["host"].to_string());

    // Positive controls: the scan above would pass vacuously if the plants
    // never reached a payload at all.
    assert!(wall.contains(PLANT_RESOLVING_NOTE), "reveal enters at the resolving step");
    assert_eq!(reveal["wall"]["trace"]["at"], 5);
    assert_eq!(reveal["wall"]["reveal"]["correct"], "E");
    assert_eq!(reveal["wall"]["reveal"]["mark"], "✓");
    assert_eq!(reveal["wall"]["reveal"]["receipt"]["heading"], "How we know");
    assert_eq!(reveal["wall"]["reveal"]["middle"]["line"], "6 of us said B");
    assert!(host.contains(PLANT_WHAT) && host.contains(PLANT_TAKEAWAY));
    assert!(host.contains(&plant_why("B")));
    assert_eq!(reveal["host"]["read_aloud"]["middle"]["heading"], "Why 6 of us said B");
    assert_eq!(reveal["buzzer"]["correct"], "E");
    assert!(buzzer.contains("✓ It was E."));
    // The explanation is the host's to read aloud; the wall and buzzers never carry it.
    for s in [&wall, &buzzer] {
        assert!(!s.contains(PLANT_WHAT) && !s.contains(PLANT_TAKEAWAY));
    }
}

#[tokio::test]
async fn the_receipt_plant_stays_sealed_until_reveal() {
    // A does-not-compile record: its receipt carries the planted error code.
    let reveal = drive_and_scan("planted-dnc", [0, 0, 0, 5, 0]).await;
    let lines = &reveal["wall"]["reveal"]["receipt"]["lines"];
    assert_eq!(lines, &json!(["✓ Compiler refused it", "✓ Error ECANARY0", "✓ Nothing ran"]));
    assert_eq!(reveal["wall"]["reveal"]["correct"], "D");
    // Everyone right: the nobody-read-it-another-way variant (§4.5).
    assert_eq!(reveal["wall"]["reveal"]["middle"]["line"], "Nobody read it another way.");
    assert_eq!(reveal["host"]["read_aloud"]["middle"]["heading"], "Why nobody said anything else");
}

#[tokio::test]
async fn the_host_screen_names_its_phase_and_one_action() {
    let h = harness();
    let room = create(&h.app, "q3").await;
    for phase in Phase::ALL {
        let (_, host) = call(&h.app, Method::GET, &format!("/rooms/{}/host", room.id), Some(&room.session), None).await;
        assert_eq!(host["label"], phase.host_label());
        assert_eq!(host["primary"]["label"], phase.next_action().label());
        assert_eq!(host["primary"]["action"], phase.next_action().slug());
        assert_eq!(host["code"].as_str().unwrap().len(), 6);
        assert_eq!(host.get("first_screen").is_some(), phase == Phase::Idle);
        if phase == Phase::Idle {
            let first = host["first_screen"].to_string();
            assert!(first.contains(copy::NOT_A_GUARANTEE_OPTIONS_PUBLIC));
            assert!(first.contains(copy::NOT_A_GUARANTEE_HOST_HONEST));
            assert!(first.contains(&room.session), "the resume link carries the session");
        }
        if phase == Phase::Released {
            break;
        }
        command(&h.app, &room, Command::Host(phase.next_action())).await;
    }
}

#[tokio::test]
async fn every_host_route_exists_and_nothing_moves_without_the_host() {
    let h = harness();
    let room = create(&h.app, "q3").await;

    // One route per host action and step: the eight minus Create (which is
    // POST /rooms), plus ← and →.
    let routes = room::host_routes();
    assert_eq!(routes.len(), 9);
    for (c, path) in &routes {
        assert_eq!(*path, format!("/rooms/{{id}}/{}", c.slug()));
    }

    // No route for anything else — there is nothing to skip with.
    let (status, _) = call(&h.app, Method::POST, &format!("/rooms/{}/skip", room.id), Some(&room.session), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // No credential, a wrong one, or the organizer's create credential: 401,
    // and the room has not moved.
    for bearer in [None, Some("not-the-session"), Some(ORGANIZER)] {
        let (status, body) = call(&h.app, Method::POST, &format!("/rooms/{}/put-on-screen", room.id), bearer, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body, Value::Null, "a denial says nothing");
        let (status, _) = call(&h.app, Method::GET, &format!("/rooms/{}/host", room.id), bearer, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (_, wall) = call(&h.app, Method::GET, &format!("/rooms/{}/wall", room.id), None, None).await;
    assert_eq!(wall["phase"], "idle");

    // Out of order: refused with a reason, still idle.
    let (status, body) = command(&h.app, &room, Command::Host(HostAction::Reveal)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["reason"], "Reveal isn't next. Put it on the screen comes next.");
    let (_, wall) = call(&h.app, Method::GET, &format!("/rooms/{}/wall", room.id), None, None).await;
    assert_eq!(wall["phase"], "idle");
}

#[tokio::test]
async fn creation_needs_the_create_credential_and_a_scheduled_question() {
    // What the binary ships: DenyAll, nothing scheduled.
    let (status, _) = call(&room::router(), Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": "q3"}))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let h = harness();
    let (status, _) = call(&h.app, Method::POST, "/rooms", None, Some(json!({"question_id": "q3"}))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, body) = call(&h.app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": "q99"}))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["reason"], "No question is scheduled with that id.");
}

#[tokio::test]
async fn run_it_again_makes_a_new_room_and_never_reruns_the_question() {
    let h = harness();
    let room = create(&h.app, "q3").await;
    let again = format!("/rooms/{}/run-it-again", room.id);

    // Before release it is not the next action.
    let (status, _) = call(&h.app, Method::POST, &again, Some(ORGANIZER), Some(json!({"question_id": "planted"}))).await;
    assert_eq!(status, StatusCode::CONFLICT);

    for phase in &Phase::ALL[..6] {
        command(&h.app, &room, Command::Host(phase.next_action())).await;
    }
    let (_, wall) = call(&h.app, Method::GET, &format!("/rooms/{}/wall", room.id), None, None).await;
    assert_eq!(wall["phase"], "released");

    // Only the organizer who created the room; never the same question.
    let (status, _) = call(&h.app, Method::POST, &again, Some(OTHER_ORGANIZER), Some(json!({"question_id": "planted"}))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = call(&h.app, Method::POST, &again, Some(&room.session), Some(json!({"question_id": "planted"}))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "the host session is not a create credential");
    let (status, body) = call(&h.app, Method::POST, &again, Some(ORGANIZER), Some(json!({"question_id": "q3"}))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["reason"], "That question has already been run. Pick another.");
    let (status, _) = call(&h.app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": "q3"}))).await;
    assert_eq!(status, StatusCode::CONFLICT, "nor through plain creation");

    let (status, created) = call(&h.app, Method::POST, &again, Some(ORGANIZER), Some(json!({"question_id": "planted"}))).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_ne!(created["id"], json!(room.id));
    assert_ne!(created["host_session"], json!(room.session));
    let new_room = created["id"].as_str().unwrap();
    let (_, wall) = call(&h.app, Method::GET, &format!("/rooms/{new_room}/wall"), None, None).await;
    assert_eq!(wall["phase"], "idle");
    // The old room is done: still released.
    let (_, wall) = call(&h.app, Method::GET, &format!("/rooms/{}/wall", room.id), None, None).await;
    assert_eq!(wall["phase"], "released");
}
