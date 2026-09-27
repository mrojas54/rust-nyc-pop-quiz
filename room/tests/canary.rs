//! `canary` — the secrecy suite, in `just test` (T-08).
//!
//! `EVALUATION.md`'s harness table splits the suite in two: this scan, which
//! runs inside `just test`, and `canary_full.rs`, which runs the same scan
//! through real TCP connections inside `just test-full`. The rules and the
//! walk live in `canary_scan/mod.rs`; the plants in `common::plants()`.
//!
//! The walk drives a room on the canary question through **every phase in
//! order** with a wall, a host and three buzzers attached over the loopback
//! listener and the real buzzer page (the served `buzzer.js`, under `node`)
//! joined by its code, and at every revision scans everything a client can
//! receive: each HTTP projection, each response a client provokes, each
//! socket frame per viewer, each served page and file, the wall's frames as
//! the served `wall.js` renders them, and the page's own rendered HTML and
//! traffic. `canary_scan::check` holds the phase-scoped rules (G-3, G-8,
//! AC-47, AC-48, AC-60, AC-97, AC-101's canary half); the positive controls
//! below prove the plants reached the surfaces that may show them, so a
//! scanner that saw nothing cannot pass.
//!
//! Also here, unchanged from T-04a: the host screen per phase, the host
//! routes and their denials, creation, and *Run it again*.

mod common;
#[path = "canary_scan/mod.rs"]
mod canary_scan;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use common::*;
use canary_scan::*;
use room::copy;
use room::phase::{Command, HostAction, Phase};
use room::rooms::{AppState, Urls};
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
}

fn harness() -> Harness {
    let sessions = FakeSessions::default();
    let state = Arc::new(
        AppState::new(
            Arc::new(TestAuth),
            vec![load(&planted()), load(&planted_dnc()), q3()],
            Urls::default(),
        )
        .with_sessions(move || Box::new(sessions.clone())),
    );
    Harness {
        app: room::router_with(state),
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

async fn command(app: &Router, room: &Hosted, c: Command) -> (StatusCode, Value) {
    call(app, Method::POST, &format!("/rooms/{}/{}", room.id, c.slug()), Some(&room.session), None).await
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

// --------------------------------------------------------------------------
// T-08: the scan.
// --------------------------------------------------------------------------

#[test]
fn every_route_in_routes_rs_is_scanned_or_listed() {
    assert_every_route_is_scanned();
    // The walk found something to walk: the three families and the pages.
    let table = route_table();
    assert!(table.len() >= 30, "{table:?}");
    assert!(served_files("shared").len() >= 10 && served_files("shared/fonts").len() >= 4);
}

#[test]
fn the_rules_catch_a_planted_leak() {
    // The scanner itself, checked: a rule that never fires would let every
    // scan below pass. Each case is a payload the room must never send.
    let p = plants();
    let cases: Vec<(Surface, Phase, Value)> = vec![
        (Surface::Buzzer, Phase::Live, json!({"phase": "live", "hint": {"text": p.hint}, "source": p.source})),
        (Surface::Host, Phase::Live, json!({"phase": "live", "note": p.resolving})),
        (Surface::Wall, Phase::Work, json!({"phase": "work", "colour": false, "why": p.why[1]})),
        (Surface::Wall, Phase::Split, json!({"phase": "split", "x": p.correct})),
        (Surface::Host, Phase::Idle, json!({"phase": "idle", "x": p.hint})),
        (Surface::Wall, Phase::Reveal, json!({"phase": "reveal", "x": p.admin})),
        (Surface::Wall, Phase::Work, json!({"phase": "work", "colour": false, "trace": {"at": 5}})),
        (Surface::Wall, Phase::Work, json!({"phase": "work", "colour": true})),
        (Surface::Buzzer, Phase::Reveal, json!({"phase": "reveal", "x": p.notes[0]})),
        (Surface::Page, Phase::Idle, json!({"x": p.what})),
        (Surface::Wall, Phase::Released, json!({"phase": "released", "x": p.source})),
        (Surface::Wall, Phase::Closed, json!({"phase": "closed", "values": [{"name": "stdout", "now": "-"}]})),
    ];
    for (surface, phase, v) in cases {
        let caught = std::panic::catch_unwind(|| {
            check(&mut Seen::default(), surface, phase, &v.to_string(), Some(&v), "self-test");
        });
        assert!(caught.is_err(), "the rules let {v} through as {surface:?} in {phase:?}");
    }
    // …and a clean live buzzer passes.
    let clean = json!({"phase": "live", "hint": {"text": p.hint}, "letters": ["A", "B", "C", "D", "E"]});
    check(&mut Seen::default(), Surface::Buzzer, Phase::Live, &clean.to_string(), Some(&clean), "self-test");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_phase_every_surface_the_answer_stays_sealed_until_reveal() {
    // AC-32, AC-47, AC-48, AC-58, AC-60, AC-79, AC-97, G-3, G-4, G-8.
    let server = Server::start(Http::InProcess).await;
    let e = walk_room(&server, Start::Question("canary"), Options { run_again_with: None, reconnect: false }).await;
    assert_the_plants_arrived(&e);
    let p = plants();
    // The correct option is E, and at reveal the wall's final step prints it.
    assert_eq!(e.reveal_wall["reveal"]["correct"], "E");
    assert!(e.seen.has(Surface::Wall, Phase::Reveal, &p.receipt), "the wall's receipt carries the planted run count");
    assert!(e.reveal_wall.to_string().contains(&p.correct));
    assert_eq!(e.reveal_buzzer["correct"], "E");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_receipt_plant_stays_sealed_until_reveal_does_not_compile() {
    // The same walk on the does-not-compile twin: the receipt carries the
    // planted error code, and only from reveal (G-3, AC-60).
    let server = Server::start(Http::InProcess).await;
    let e = walk_room(&server, Start::Question("canary-dnc"), Options { run_again_with: None, reconnect: false }).await;
    assert_the_plants_arrived(&e);
    let p = plants();
    assert_eq!(e.reveal_wall["reveal"]["correct"], "D");
    let lines = &e.reveal_wall["reveal"]["receipt"]["lines"];
    assert_eq!(lines, &json!(["✓ Compiler refused it", format!("✓ Error {}", p.error_code), "✓ Nothing ran"]));
    assert!(e.seen.has(Surface::RenderedWall, Phase::Reveal, &p.error_code));
}
