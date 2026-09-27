//! The wall page and the shared assets (T-05), in-process — no socket.
//!
//! - `GET /wall/{room_id}` serves the page, and a room that does not exist is
//!   `404`.
//! - Every file in `web/shared/` (css, js) and `web/shared/fonts/` (ttf) is
//!   served at `/shared/…` with its content type and the exact bytes on disk;
//!   anything else under `/shared/` is `404`.
//! - The page is static: it is the same bytes in every phase, and it carries no
//!   ✓, no receipt line, no plant, and no `stdout` in any phase. The wall
//!   *payload* before `reveal` has no `values` entry named `stdout` (G-3, D-10).
//!   This extends the canary's idea to the page; `canary.rs` is not edited.
//! - `web/wall/fixtures/q3-phases.json` is what `view::wall` serves for q3 in
//!   every phase and trace step. `web/test/wall.test.js` renders from it, so
//!   the web suite runs on real payloads, never typed ones. Regenerate with
//!   `UPDATE_WALL_FIXTURES=1 cargo test --test wall_page`.

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

/// A response's status, content type and body bytes.
async fn get(app: &Router, uri: &str) -> (StatusCode, String, Vec<u8>) {
    let res = app
        .clone()
        .oneshot(Request::builder().method(Method::GET).uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let ty = res
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let bytes = axum::body::to_bytes(res.into_body(), 4 << 20).await.unwrap();
    (status, ty, bytes.to_vec())
}

async fn post(app: &Router, uri: &str, bearer: &str, body: Option<Value>) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {bearer}"));
    let req = match body {
        Some(v) => req
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(v.to_string())),
        None => {
            req = req.header(header::CONTENT_LENGTH, "0");
            req.body(Body::empty())
        }
    }
    .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let json = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() };
    (status, json)
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
            vec![load(&planted()), q3(), load(&planted_dnc())],
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

struct Hosted {
    id: String,
    code: String,
    host: String,
}

async fn create(h: &Harness, question: &str) -> Hosted {
    let (status, created) = post(&h.app, "/rooms", ORGANIZER, Some(json!({ "question_id": question }))).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    Hosted {
        id: created["id"].as_str().unwrap().into(),
        code: created["code"].as_str().unwrap().into(),
        host: created["host_session"].as_str().unwrap().into(),
    }
}

async fn command(h: &Harness, room: &Hosted, c: Command) {
    let (status, body) = post(&h.app, &format!("/rooms/{}/{}", room.id, c.slug()), &room.host, None).await;
    assert_eq!(status, StatusCode::OK, "{} refused: {body}", c.slug());
}

async fn wall_json(h: &Harness, room: &Hosted) -> Value {
    let (status, _, bytes) = get(&h.app, &format!("/rooms/{}/wall", room.id)).await;
    assert_eq!(status, StatusCode::OK);
    serde_json::from_slice(&bytes).unwrap()
}

async fn page(h: &Harness, room: &Hosted) -> String {
    let (status, ty, bytes) = get(&h.app, &format!("/wall/{}", room.id)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ty, "text/html; charset=utf-8");
    String::from_utf8(bytes).unwrap()
}

/// Present and answered before close, so the split reads "58 of 61".
fn counts(h: &Harness, room: &Hosted, totals: [u32; 5]) {
    *h.sessions.totals.lock().unwrap() = totals;
    let answered: u32 = totals.iter().sum();
    h.state
        .set_live_counts(&room.id, LiveCounts { present: answered + 3, answered_live: answered })
        .unwrap();
}

const WALK: [HostAction; 4] = [
    HostAction::PutOnScreen,
    HostAction::CloseAnswers,
    HostAction::ShowSplit,
    HostAction::WalkIt,
];

// --------------------------------------------------------------------------
// Serving
// --------------------------------------------------------------------------

#[tokio::test]
async fn the_wall_page_serves_for_a_room_and_404s_for_none() {
    let h = harness();
    let room = create(&h, "q3").await;
    let html = page(&h, &room).await;
    assert!(html.starts_with("<!DOCTYPE html>"));
    for asset in [
        "/shared/tokens.css",
        "/shared/components.css",
        "/wall/wall.css",
        "/shared/dom.js",
        "/shared/phase.js",
        "/shared/check.js",
        "/shared/well.js",
        "/shared/trace.js",
        "/shared/typemodel.js",
        "/shared/copy.js",
        "/wall/qr.js",
        "/wall/wall.js",
    ] {
        assert!(html.contains(&format!("\"{asset}\"")), "the page does not load {asset}");
        let (status, _, _) = get(&h.app, asset).await;
        assert_eq!(status, StatusCode::OK, "{asset} is linked but not served");
    }

    let (status, _, body) = get(&h.app, "/wall/0123456789abcdef0123456789abcdef").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.is_empty(), "a 404 says nothing about what exists (AC-70)");
}

fn content_type_for(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap() {
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "ttf" => "font/ttf",
        other => panic!("no content type for .{other}"),
    }
}

/// Every file on disk with one of `exts`, under `dir`.
fn files(dir: &str, exts: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(repo().join(dir))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| exts.iter().any(|x| n.ends_with(x)))
        .collect();
    names.sort();
    names
}

#[tokio::test]
async fn every_shared_file_and_font_serves_with_its_bytes_and_type() {
    let h = harness();
    let mut served = Vec::new();
    for (dir, prefix, exts) in [
        ("web/shared", "/shared", &[".css", ".js"][..]),
        ("web/shared/fonts", "/shared/fonts", &[".ttf"][..]),
        ("web/wall", "/wall", &[".css", ".js"][..]),
    ] {
        for name in files(dir, exts) {
            let uri = format!("{prefix}/{name}");
            let (status, ty, body) = get(&h.app, &uri).await;
            assert_eq!(status, StatusCode::OK, "{uri}");
            assert_eq!(ty, content_type_for(&name), "{uri}");
            let disk = std::fs::read(repo().join(dir).join(&name)).unwrap();
            assert!(body == disk, "{uri} is not the bytes on disk");
            served.push(uri);
        }
    }
    assert!(served.len() >= 16, "fewer assets than expected: {served:?}");

    for missing in [
        "/shared/README.md",
        "/shared/fonts/README.md",
        "/shared/nope.js",
        "/shared/..%2Froom%2FCargo.toml",
        "/shared/fonts/OFL-CascadiaMono.txt",
    ] {
        let (status, _, _) = get(&h.app, missing).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{missing}");
    }
}

// --------------------------------------------------------------------------
// The page carries nothing, in any phase (G-3, AC-79)
// --------------------------------------------------------------------------

fn scan_page(phase: Phase, html: &str) {
    let at = format!("the wall page in {phase:?}");
    assert!(!html.contains('✓'), "{at}: carries a ✓");
    assert!(!html.contains("stdout"), "{at}: names stdout");
    assert!(!html.contains(copy::RECEIPT_HEADING), "{at}: carries the receipt");
    for line in [copy::RECEIPT_COMPILED, copy::RECEIPT_NOTHING_RAN, copy::RECEIPT_COMPILER_REFUSED] {
        assert!(!html.contains(line.trim_start_matches("✓ ")), "{at}: receipt line {line}");
    }
    for plant in [
        PLANT_RESOLVING_NOTE,
        PLANT_WHAT,
        PLANT_TAKEAWAY,
        PLANT_HINT,
        PLANT_MIDDLE_STDOUT,
        PLANT_ERROR_CODE,
        "CANARY",
    ] {
        assert!(!html.contains(plant), "{at}: carries {plant}");
    }
    // AC-79: nothing on the wall takes input.
    for control in ["<button", "<input", "<select", "<textarea", "<a ", "onclick", "<form"] {
        assert!(!html.contains(control), "{at}: has an interactive control ({control})");
    }
}

#[tokio::test]
async fn the_page_is_the_same_bytes_in_every_phase_and_carries_no_answer() {
    for question in ["planted", "planted-dnc"] {
        let h = harness();
        let room = create(&h, question).await;
        let first = page(&h, &room).await;
        scan_page(Phase::Idle, &first);
        assert_eq!(stdout_rows(&wall_json(&h, &room).await), 0);

        let mut phases = vec![];
        for action in WALK {
            if action == HostAction::CloseAnswers {
                counts(&h, &room, [5, 4, 3, 2, 1]);
            }
            command(&h, &room, Command::Host(action)).await;
            phases.push(wall_json(&h, &room).await["phase"].as_str().unwrap().to_string());
            let html = page(&h, &room).await;
            scan_page(Phase::Live, &html);
            assert_eq!(html, first, "the page changed with the phase");
            let wall = wall_json(&h, &room).await;
            assert_eq!(stdout_rows(&wall), 0, "{question}: wall payload names stdout before reveal");
        }
        // Every step `work` allows: still no stdout, and still the same page.
        loop {
            let before = wall_json(&h, &room).await["trace"]["at"].clone();
            let (status, _) = post(
                &h.app,
                &format!("/rooms/{}/{}", room.id, Command::Step(Step::Forward).slug()),
                &room.host,
                None,
            )
            .await;
            let wall = wall_json(&h, &room).await;
            assert_eq!(stdout_rows(&wall), 0, "{question}: a work step names stdout");
            assert_eq!(page(&h, &room).await, first);
            if status != StatusCode::OK || wall["trace"]["at"] == before {
                break;
            }
        }
        assert_eq!(phases, ["live", "closed", "split", "work"]);

        command(&h, &room, Command::Host(HostAction::Reveal)).await;
        assert_eq!(page(&h, &room).await, first, "the page changed at reveal");
        // The scanner can see: at reveal the payload does carry what was sealed.
        let revealed = wall_json(&h, &room).await;
        assert!(revealed.to_string().contains(copy::RECEIPT_HEADING));

        command(&h, &room, Command::Host(HostAction::ReleaseRoom)).await;
        let html = page(&h, &room).await;
        scan_page(Phase::Released, &html);
        assert_eq!(html, first);
    }
}

// --------------------------------------------------------------------------
// The fixtures the web suite renders from
// --------------------------------------------------------------------------

/// The room code is random per run; the fixture carries a fixed stand-in so
/// the file is stable. It appears in `code` and in the join link.
const CODE_STANDIN: &str = "WALLQ3";

fn stable(mut v: Value, code: &str) -> Value {
    let s = v.to_string().replace(code, CODE_STANDIN);
    v = serde_json::from_str(&s).unwrap();
    v
}

/// One room through every phase and every trace step, and the wall payload at
/// each. `totals` are counts of votes the test casts, not program output.
async fn walk(h: &Harness, totals: [u32; 5]) -> Value {
    let room = create(h, "q3").await;
    let mut frames = serde_json::Map::new();
    frames.insert("idle".into(), stable(wall_json(h, &room).await, &room.code));
    for action in WALK {
        if action == HostAction::CloseAnswers {
            counts(h, &room, totals);
        }
        command(h, &room, Command::Host(action)).await;
        let wall = stable(wall_json(h, &room).await, &room.code);
        let phase = wall["phase"].as_str().unwrap().to_string();
        if phase != "work" {
            frames.insert(phase, wall);
        }
    }
    // work: step 0 up to the bound (M-2), one frame per step.
    let mut work = vec![stable(wall_json(h, &room).await, &room.code)];
    loop {
        let (status, _) = post(
            &h.app,
            &format!("/rooms/{}/{}", room.id, Command::Step(Step::Forward).slug()),
            &room.host,
            None,
        )
        .await;
        let wall = stable(wall_json(h, &room).await, &room.code);
        if status != StatusCode::OK || wall["trace"]["at"] == work.last().unwrap()["trace"]["at"] {
            break;
        }
        work.push(wall);
    }
    frames.insert("work".into(), Value::Array(work));
    // reveal: enters at M-1, then back to 0.
    command(h, &room, Command::Host(HostAction::Reveal)).await;
    let mut reveal = vec![stable(wall_json(h, &room).await, &room.code)];
    loop {
        let (status, _) = post(
            &h.app,
            &format!("/rooms/{}/{}", room.id, Command::Step(Step::Back).slug()),
            &room.host,
            None,
        )
        .await;
        let wall = stable(wall_json(h, &room).await, &room.code);
        if status != StatusCode::OK || wall["trace"]["at"] == reveal.last().unwrap()["trace"]["at"] {
            break;
        }
        reveal.push(wall);
    }
    frames.insert("reveal".into(), Value::Array(reveal));
    command(h, &room, Command::Host(HostAction::ReleaseRoom)).await;
    frames.insert("released".into(), stable(wall_json(h, &room).await, &room.code));
    Value::Object(frames)
}

#[tokio::test]
async fn the_web_fixtures_are_what_the_room_serves() {
    // Each walk releases q3, and a released question is never run twice (G-10),
    // so each gets its own room service.
    // Most of the room on E, a real crowd on A: the named option is A.
    let split = [24, 9, 6, 3, 16];
    // Everybody on the correct option: the "nobody read it another way" wall.
    let unanimous = [0, 0, 0, 0, 12];
    let fixture = json!({
        "_note": "GENERATED by room/tests/wall_page.rs from view::wall for bank/questions/q3.json — never edit by hand. Regenerate: UPDATE_WALL_FIXTURES=1 cargo test --test wall_page. The totals are votes the test casts. The room code is replaced by WALLQ3.",
        "question": "q3",
        "totals": split,
        "frames": walk(&harness(), split).await,
        "unanimous": { "totals": unanimous, "frames": walk(&harness(), unanimous).await },
    });

    let path = repo().join("web/wall/fixtures/q3-phases.json");
    let text = serde_json::to_string_pretty(&fixture).unwrap() + "\n";
    if std::env::var_os("UPDATE_WALL_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
    }
    let on_disk = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        on_disk == text,
        "web/wall/fixtures/q3-phases.json is not what the room serves; regenerate with \
         UPDATE_WALL_FIXTURES=1 cargo test --test wall_page"
    );

    // What the web suite leans on, stated here so a regenerated file that lost
    // it fails in the room's suite too.
    let f = &fixture["frames"];
    let m = f["reveal"][0]["trace"]["m"].as_u64().unwrap();
    assert_eq!(f["work"].as_array().unwrap().len() as u64, m - 1, "work steps 0..=M-2");
    assert_eq!(f["reveal"][0]["trace"]["at"].as_u64().unwrap(), m - 1, "reveal enters at M-1");
    assert_eq!(f["reveal"].as_array().unwrap().len() as u64, m, "reveal steps back through all of it");
    assert_eq!(f["reveal"][0]["reveal"]["middle"]["letter"], "A");
    assert!(fixture["unanimous"]["frames"]["reveal"][0]["reveal"]["middle"]["letter"].is_null());
}

// --------------------------------------------------------------------------
// The wall's fit verdict reaches the host (§3.4, §5.2, AC-100)
// --------------------------------------------------------------------------

async fn put_fit(app: &Router, room: &str, body: &str) -> StatusCode {
    let req = Request::builder()
        .method(Method::PUT)
        .uri(format!("/rooms/{room}/fit"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    app.clone().oneshot(req).await.unwrap().status()
}

async fn host_fit(h: &Harness, room: &Hosted) -> Value {
    let res = h
        .app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/rooms/{}/host", room.id))
                .header(header::AUTHORIZATION, format!("Bearer {}", room.host))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    serde_json::from_slice::<Value>(&bytes).unwrap()["fit"].clone()
}

#[tokio::test]
async fn the_walls_verdict_becomes_the_hosts_fit_line() {
    let h = harness();
    let room = create(&h, "q3").await;
    command(&h, &room, Command::Host(HostAction::PutOnScreen)).await;
    assert!(host_fit(&h, &room).await.is_null(), "no verdict before the wall has measured");

    for (verdict, line) in [
        ("clipped_y", copy::HOST_FIT_CLIPPED_Y),
        ("clipped_x", copy::HOST_FIT_CLIPPED_X),
        ("clipped_xy", copy::HOST_FIT_CLIPPED_XY),
        ("fits", copy::HOST_FIT_FITS),
    ] {
        // The wall has no credential, and needs none (§3.4: the wall writes it).
        assert_eq!(put_fit(&h.app, &room.id, &json!({ "fit": verdict }).to_string()).await, StatusCode::NO_CONTENT);
        assert_eq!(host_fit(&h, &room).await, line, "{verdict}");
    }

    let before = h.state.with_room(&room.id, |r| r.revision()).unwrap();
    assert_eq!(put_fit(&h.app, &room.id, r#"{"fit":"fits"}"#).await, StatusCode::NO_CONTENT);
    assert_eq!(h.state.with_room(&room.id, |r| r.revision()).unwrap(), before, "a repeat pushes nothing");

    for bad in [r#"{"fit":"fits the room"}"#, r#"{"fit":"null"}"#, r#"{"fit":"FITS"}"#] {
        assert_eq!(put_fit(&h.app, &room.id, bad).await, StatusCode::BAD_REQUEST, "{bad}");
    }
    assert_eq!(host_fit(&h, &room).await, copy::HOST_FIT_FITS, "a refused verdict changed nothing");
    assert_eq!(
        put_fit(&h.app, "0123456789abcdef0123456789abcdef", r#"{"fit":"fits"}"#).await,
        StatusCode::NOT_FOUND
    );
}
