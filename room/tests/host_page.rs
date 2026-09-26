//! The host phone's pages (T-07): AC-45, AC-46, AC-47, AC-49, AC-50.
//!
//! The page itself is `web/host/`, served by the `// T-07 pages` block in
//! `routes.rs`. What this file proves from the room's side:
//!
//! - the pages and their assets serve, and every asset the page references is
//!   one the room serves (`/shared/*` through a test-only helper until PQ-7's
//!   route is on this branch);
//! - AC-47, the canary's idea for the host: driving a planted question through
//!   every phase as the host, neither the page nor any payload the page fetches
//!   carries a plant, a `correct` key, the script or a resolving step before
//!   `reveal` — and at `reveal` the script does arrive, so the scan is not
//!   vacuous. (The page is static, so its half guards against anyone inlining
//!   room data into it later; the payload half overlaps `canary.rs` by design.)
//! - AC-45/AC-49: in each phase the payload's one primary action is the one
//!   route that succeeds, and every other action is refused;
//! - AC-46: the host socket's counts move while `live`;
//! - AC-50: the resume link is `/host/{id}#{session}` — this page's own address
//!   — and two devices attached with it both see and both control the room;
//!   the session never rotates.

mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::Path;
use axum::http::{header, Method, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use common::*;
use room::phase::{Command, HostAction, Phase};
use room::rooms::{AppState, Urls};
use serde_json::{json, Value};
use tower::ServiceExt;

const WAIT: Duration = Duration::from_secs(5);

/// `GET /shared/{file}` from `web/shared/`, for these tests only. PQ-7 owns the
/// real route; this stands in so the page's references can be followed.
fn shared() -> Router {
    Router::new().route(
        "/shared/{file}",
        get(|Path(file): Path<String>| async move {
            let dir = repo().join("web/shared");
            let ty = match file.rsplit('.').next() {
                Some("css") => "text/css; charset=utf-8",
                Some("js") => "text/javascript; charset=utf-8",
                _ => return StatusCode::NOT_FOUND.into_response(),
            };
            match (file.contains('/') || file.contains(".."), std::fs::read_to_string(dir.join(&file))) {
                (false, Ok(body)) => ([(header::CONTENT_TYPE, ty)], body).into_response(),
                _ => StatusCode::NOT_FOUND.into_response(),
            }
        }),
    )
}

fn app() -> Router {
    let state = Arc::new(AppState::new(
        Arc::new(TestAuth),
        vec![load(&planted()), q3()],
        Urls::default(),
    ));
    room::router_with(state).merge(shared())
}

/// One GET: status, content type, body text.
async fn get_text(app: &Router, uri: &str) -> (StatusCode, String, String) {
    let res = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let ty = res
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    (status, ty, String::from_utf8(bytes.to_vec()).unwrap())
}

async fn create(app: &Router, question: &str) -> (String, String, String) {
    let (status, created) = http(app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": question}))).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    (
        created["id"].as_str().unwrap().into(),
        created["code"].as_str().unwrap().into(),
        created["host_session"].as_str().unwrap().into(),
    )
}

async fn host_view(app: &Router, id: &str, session: &str) -> Value {
    let (status, v) = http(app, Method::GET, &format!("/rooms/{id}/host"), Some(session), None).await;
    assert_eq!(status, StatusCode::OK, "{v}");
    v
}

fn phase_of(v: &Value) -> Phase {
    let name = v["phase"].as_str().unwrap();
    let i = ["idle", "live", "closed", "split", "work", "reveal", "released"]
        .iter()
        .position(|p| *p == name)
        .unwrap();
    Phase::ALL[i]
}

#[tokio::test]
async fn the_pages_serve_and_everything_they_reference_serves() {
    let app = app();
    let (id, _, _) = create(&app, "q3").await;
    for uri in ["/host".to_string(), format!("/host/{id}")] {
        let (status, ty, html) = get_text(&app, &uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert!(ty.starts_with("text/html"), "{uri}: {ty}");
        let mut refs = 0;
        for attr in ["src=\"", "href=\""] {
            for piece in html.split(attr).skip(1) {
                let target = piece.split('"').next().unwrap();
                assert!(
                    target.starts_with("/shared/") || target.starts_with("/host/"),
                    "{uri} references {target}"
                );
                let (status, ty, _) = get_text(&app, target).await;
                assert_eq!(status, StatusCode::OK, "{target}");
                let want = if target.ends_with(".css") { "text/css" } else { "text/javascript" };
                assert!(ty.starts_with(want), "{target}: {ty}");
                refs += 1;
            }
        }
        assert_eq!(refs, 8, "{uri}: four stylesheets and four scripts");
    }
}

#[tokio::test]
async fn the_page_that_reads_a_credential_is_never_cached_or_referred() {
    let res = app()
        .oneshot(Request::builder().uri("/host").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(res.headers()[header::REFERRER_POLICY], "no-referrer");
}

/// The host's pre-reveal rules for one page or payload (G-3, AC-47).
fn scan_pre_reveal(at: &str, text: &str) {
    for plant in [PLANT_RESOLVING_NOTE, PLANT_WHAT, PLANT_TAKEAWAY, PLANT_MIDDLE_STDOUT, PLANT_HINT] {
        assert!(!text.contains(plant), "{at}: carries {plant}");
    }
    for letter in ["A", "B", "C", "D", "E"] {
        assert!(!text.contains(&plant_why(letter)), "{at}: carries why_tempting {letter}");
    }
    assert!(!text.contains('✓'), "{at}: carries a ✓");
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        assert_eq!(stdout_rows(&v), 0, "{at}: a stdout row");
        for key in ["correct", "read_aloud", "receipt", "reveal", "why_tempting", "explains", "mark"] {
            assert!(keys_named(&v, key).is_empty(), "{at}: has a {key:?} key");
        }
    }
}

#[tokio::test]
async fn ac47_no_page_or_payload_the_host_sees_carries_the_answer_before_reveal() {
    let app = app();
    let (id, code, session) = create(&app, "planted").await;
    let page = format!("/host/{id}");

    // Three participants, so the close has votes to freeze and the middle
    // beat has an incorrect option to name.
    let mut tokens = Vec::new();
    for _ in 0..3 {
        let (status, joined) = http(&app, Method::POST, "/join", None, Some(json!({"code": code}))).await;
        assert_eq!(status, StatusCode::CREATED, "{joined}");
        tokens.push(joined["token"].as_str().unwrap().to_string());
    }

    let mut acted: Option<Value> = None;
    loop {
        let v = host_view(&app, &id, &session).await;
        let phase = phase_of(&v);
        let at = format!("host in {phase:?}");
        let (_, _, html) = get_text(&app, &page).await;
        let (_, _, js) = get_text(&app, "/host/host.js").await;
        if phase == Phase::Reveal {
            let s = v.to_string();
            assert!(s.contains(PLANT_WHAT) && s.contains(PLANT_TAKEAWAY), "reveal carries the script: {s}");
            assert!(!keys_named(&v, "read_aloud").is_empty());
        } else {
            scan_pre_reveal(&at, &v.to_string());
            if let Some(a) = acted.take() {
                scan_pre_reveal(&format!("{at}, the action's response"), &a.to_string());
            }
        }
        // The page never carries room data, in any phase.
        scan_pre_reveal(&format!("{at}: page"), &html);
        scan_pre_reveal(&format!("{at}: host.js"), &js);

        if phase == Phase::Live {
            for (t, letter) in tokens.iter().zip(["A", "B", "C"]) {
                let (status, _) =
                    http(&app, Method::PUT, &format!("/rooms/{id}/answer"), Some(t), Some(json!({"letter": letter}))).await;
                assert_eq!(status, StatusCode::OK);
            }
        }
        if phase == Phase::Released {
            break;
        }
        let slug = v["primary"]["action"].as_str().unwrap().to_string();
        let (status, body) = http(&app, Method::POST, &format!("/rooms/{id}/{slug}"), Some(&session), None).await;
        assert_eq!(status, StatusCode::OK, "{slug}: {body}");
        acted = Some(body);
    }
}

#[tokio::test]
async fn ac45_ac49_each_phase_offers_one_action_and_the_room_accepts_only_that_one() {
    let app = app();
    let (id, _, session) = create(&app, "planted").await;
    let mut seen = Vec::new();
    loop {
        let v = host_view(&app, &id, &session).await;
        let phase = phase_of(&v);
        assert_eq!(v["label"], phase.host_label(), "AC-49: the phase label");
        assert!(v["code"].as_str().is_some_and(|c| c.len() == 6), "AC-49: the code");
        let primary = v["primary"]["action"].as_str().unwrap().to_string();
        assert_eq!(primary, phase.next_action().slug());
        seen.push(primary.clone());

        // Every other action is refused here, and refusing it moves nothing.
        for c in Command::all() {
            let Command::Host(a) = c else { continue };
            if a.slug() == primary || matches!(a, HostAction::CreateRoom | HostAction::RunItAgain) {
                continue;
            }
            let (status, _) = http(&app, Method::POST, &format!("/rooms/{id}/{}", a.slug()), Some(&session), None).await;
            assert_eq!(status, StatusCode::CONFLICT, "{} in {phase:?}", a.slug());
        }
        assert_eq!(phase_of(&host_view(&app, &id, &session).await), phase, "a refusal moved the room");

        if phase == Phase::Released {
            // *Run it again* is a new room, on the organizer's credential and
            // a question not yet run; the host session alone cannot do it.
            let (status, _) = http(&app, Method::POST, &format!("/rooms/{id}/run-it-again"), Some(&session), Some(json!({"question_id": "q3"}))).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            let (status, again) = http(&app, Method::POST, &format!("/rooms/{id}/run-it-again"), Some(ORGANIZER), Some(json!({"question_id": "q3"}))).await;
            assert_eq!(status, StatusCode::CREATED, "{again}");
            assert_ne!(again["id"], json!(id));
            break;
        }
        let (status, _) = http(&app, Method::POST, &format!("/rooms/{id}/{primary}"), Some(&session), None).await;
        assert_eq!(status, StatusCode::OK, "{primary} in {phase:?}");
    }
    assert_eq!(
        seen,
        ["put-on-screen", "close-answers", "show-split", "walk-it", "reveal", "release", "run-it-again"]
    );
}

/// Frames off a host socket until one satisfies `want`.
async fn until(socket: &mut Socket, want: impl Fn(&Value) -> bool) -> Value {
    let deadline = tokio::time::Instant::now() + WAIT;
    loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        assert!(!left.is_zero(), "no frame matched in time");
        let f = frame(socket, left).await;
        if want(&f) {
            return f;
        }
    }
}

#[tokio::test]
async fn ac50_the_resume_link_attaches_a_second_device_and_both_control_the_room() {
    let live = Live::start().await;
    let room = live.create().await;

    // The resume link the first screen shows is this page's own address.
    let (_, v) = live.call(Method::GET, &format!("/rooms/{}/host", room.id), Some(&room.host), None).await;
    let resume = v["first_screen"]["resume"].as_str().unwrap();
    let link = format!("{}/host/{}#{}", Urls::default().base, room.id, room.host);
    assert!(resume.ends_with(&link), "{resume}");
    let (_, _, html) = get_text(&live.app, &format!("/host/{}", room.id)).await;
    assert!(html.contains("/host/host.js"));

    // Device one, then a "refresh" (a new socket, same fragment), then device two.
    let mut one = live.open(&room, "host", Some(&room.host)).await;
    assert_eq!(frame(&mut one, WAIT).await["phase"], "idle");
    drop(one);
    let mut one = live.open(&room, "host", Some(&room.host)).await;
    let mut two = live.open(&room, "host", Some(&room.host)).await;
    assert_eq!(frame(&mut one, WAIT).await["phase"], "idle");
    assert_eq!(frame(&mut two, WAIT).await["phase"], "idle");

    // Either device acts; both see it.
    assert_eq!(live.act(&room, "put-on-screen").await, StatusCode::OK);
    until(&mut one, |f| f["phase"] == "live").await;
    until(&mut two, |f| f["phase"] == "live").await;
    assert_eq!(live.act(&room, "close-answers").await, StatusCode::OK);
    until(&mut one, |f| f["phase"] == "closed").await;
    until(&mut two, |f| f["phase"] == "closed").await;

    // The session never rotates: the same fragment still works to the end.
    for slug in ["show-split", "walk-it", "reveal", "release"] {
        assert_eq!(live.act(&room, slug).await, StatusCode::OK, "{slug}");
    }
    until(&mut two, |f| f["phase"] == "released").await;
    let (status, _) = live.call(Method::GET, &format!("/rooms/{}/host", room.id), Some(&room.host), None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn ac46_the_host_socket_counts_move_while_live() {
    let live = Live::start().await;
    let room = live.create().await;
    let (_, created) = live.call(Method::GET, &format!("/rooms/{}/host", room.id), Some(&room.host), None).await;
    let code = created["code"].as_str().unwrap().to_string();
    let mut host = live.open(&room, "host", Some(&room.host)).await;
    frame(&mut host, WAIT).await;
    assert_eq!(live.act(&room, "put-on-screen").await, StatusCode::OK);
    let f = until(&mut host, |f| f["phase"] == "live").await;
    assert_eq!(f["answered"], 0);

    let mut tokens = Vec::new();
    for _ in 0..2 {
        let (status, joined) = live.call(Method::POST, "/join", None, Some(json!({"code": code}))).await;
        assert_eq!(status, StatusCode::CREATED, "{joined}");
        tokens.push(joined["token"].as_str().unwrap().to_string());
    }
    let (status, _) = live
        .call(Method::PUT, &format!("/rooms/{}/answer", room.id), Some(&tokens[0]), Some(json!({"letter": "A"})))
        .await;
    assert_eq!(status, StatusCode::OK);
    until(&mut host, |f| f["present"] == 2 && f["answered"] == 1).await;
    let (status, _) = live
        .call(Method::PUT, &format!("/rooms/{}/answer", room.id), Some(&tokens[1]), Some(json!({"letter": "B"})))
        .await;
    assert_eq!(status, StatusCode::OK);
    until(&mut host, |f| f["present"] == 2 && f["answered"] == 2).await;
}
