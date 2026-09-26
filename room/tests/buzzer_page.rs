//! The buzzer page (T-06), served and driven through the wired room.
//!
//! - The page and its two assets serve from `GET /join`, and `POST /join` still
//!   answers at the same path.
//! - AC-28: the short link the wall prints (`join_url`, `{base}/{code}`)
//!   redirects to `/join?code=`; a path that is not a code is not a page.
//! - A phone through the real room: join → attach → answer → close → drop →
//!   re-attach with the same token → reveal → release. Everything the phone
//!   received is replayed through the page's own state machine
//!   (`web/test/buzzer.test.js` in its replay mode, under `node`), and what the page would show
//!   is held to what the server says at each step (AC-34, AC-35, AC-37, AC-58).
//! - G-8, AC-32 (the canary's idea): the page, its assets, and every frame and
//!   body a phone received carry neither the source nor any trace step.
//!
//! `/shared/*` belongs to T-05's route; until that is on this branch, only the
//! `#[ignore]`d browser harness at the bottom serves it, from a helper here.

mod common;

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, HeaderMap, Method, Request, StatusCode};
use common::*;
use room::rooms::{AppState, Urls};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;

const WAIT: Duration = Duration::from_secs(2);

/// A GET through the router, as text: `(status, headers, body)`.
async fn get(app: &axum::Router, uri: &str) -> (StatusCode, HeaderMap, String) {
    let req = Request::builder().method(Method::GET).uri(uri).body(Body::empty()).unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let (status, headers) = (res.status(), res.headers().clone());
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    (status, headers, String::from_utf8(bytes.to_vec()).unwrap())
}

fn content_type(headers: &HeaderMap) -> &str {
    headers.get(header::CONTENT_TYPE).unwrap().to_str().unwrap()
}

fn app() -> (axum::Router, Arc<AppState>) {
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default()));
    (room::router_with(state.clone()), state)
}

#[tokio::test]
async fn the_page_and_its_assets_serve() {
    let (app, _) = app();
    let (status, headers, page) = get(&app, "/join").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type(&headers).starts_with("text/html"));
    for asset in ["/shared/tokens.css", "/shared/components.css", "/shared/dom.js", "/shared/copy.js", "/shared/check.js", "/join/buzzer.css", "/join/buzzer.js"] {
        assert!(page.contains(&format!("\"{asset}\"")), "the page loads {asset}");
    }
    assert!(page.contains("id=\"buzzer\""));

    let (status, headers, js) = get(&app, "/join/buzzer.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type(&headers).starts_with("text/javascript"));
    assert!(js.contains("PQ.Buzzer"));
    let (status, headers, _) = get(&app, "/join/buzzer.css").await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type(&headers).starts_with("text/css"));

    // The page shares its path with the join itself.
    let room = host_room(&app).await;
    let (status, joined) = http(&app, Method::POST, "/join", None, Some(json!({ "code": room.code }))).await;
    assert_eq!(status, StatusCode::CREATED, "{joined}");
}

#[tokio::test]
async fn ac28_the_short_link_carries_the_code_to_the_page() {
    let (app, _) = app();
    let (status, created) = http(&app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({ "question_id": "q3" }))).await;
    assert_eq!(status, StatusCode::CREATED);
    let code = created["code"].as_str().unwrap();
    let join_url = created["join_url"].as_str().unwrap();
    let path = join_url.strip_prefix(&Urls::default().base).expect("join_url is under the base");
    assert_eq!(path, format!("/{code}"));

    let (status, headers, _) = get(&app, path).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(headers[header::LOCATION], format!("/join?code={code}").as_str());
    // Typed in lower case, the link still lands on the code.
    let (status, headers, _) = get(&app, &path.to_ascii_lowercase()).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(headers[header::LOCATION], format!("/join?code={code}").as_str());

    // Not a code (too short; O, 0, I and 1 are not in the alphabet): no page.
    for path in ["/abc", "/NOPE10", "/ABCDEFG"] {
        assert_eq!(get(&app, path).await.0, StatusCode::NOT_FOUND, "{path}");
    }
}

/// The wired room on a loopback socket, like `tests/wiring.rs`.
struct Wired {
    app: axum::Router,
    addr: std::net::SocketAddr,
}

impl Wired {
    async fn start() -> Wired {
        let (app, _) = app();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(room::ws::serve(listener, app.clone()));
        Wired { app, addr }
    }

    async fn attach(&self, room: &HostedRoom, token: &str) -> Socket {
        let url = format!("ws://{}/rooms/{}/ws/buzzer", self.addr, room.id);
        let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();
        let attach = json!({ "t": "attach", "token": token }).to_string();
        futures_util::SinkExt::send(&mut socket, Message::text(attach)).await.unwrap();
        socket
    }

    async fn act(&self, room: &HostedRoom, slug: &str) {
        let (status, body) = http(&self.app, Method::POST, &format!("/rooms/{}/{slug}", room.id), Some(&room.host), None).await;
        assert_eq!(status, StatusCode::OK, "{slug}: {body}");
    }
}

/// What the page would show after each event, from `buzzer.test.js`'s
/// replay mode.
fn replay(events: &[Value]) -> Vec<Value> {
    let script = repo().join("web/test/buzzer.test.js");
    let mut child = Command::new("node")
        .arg(&script)
        .env("BUZZER_REPLAY", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("`node` runs the page's state machine; `just test` needs it for test-web too");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(Value::from(events.to_vec()).to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "replay failed: {}", String::from_utf8_lossy(&out.stderr));
    serde_json::from_slice::<Vec<Value>>(&out.stdout).unwrap()
}

/// G-8, AC-32: neither the source (any line of it with something on it) nor
/// any trace step's words, nor a key that could hold them.
fn assert_no_source_or_trace(what: &str, text: &str) {
    let q = q3_json();
    for line in q["source"].as_str().unwrap().lines().map(str::trim).filter(|l| l.len() > 3) {
        assert!(!text.contains(line), "{what} carries source: {line}");
    }
    for step in q["trace"]["steps"].as_array().unwrap() {
        let note = step["note"].as_str().unwrap();
        assert!(!text.contains(note), "{what} carries a trace note");
    }
    for key in ["\"source\"", "\"trace\"", "\"options\""] {
        assert!(!text.contains(key), "{what} carries {key}");
    }
}

#[tokio::test]
async fn a_phone_through_the_wired_room_shows_what_the_server_holds() {
    let w = Wired::start().await;
    let room = host_room(&w.app).await;
    let mut events: Vec<Value> = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new(); // everything the phone received

    // The link: the page reads ?code= and joins.
    events.push(json!({ "type": "boot", "search": format!("?code={}", room.code.to_ascii_lowercase()), "stored": null }));
    let (status, joined) = http(&w.app, Method::POST, "/join", None, Some(json!({ "code": room.code }))).await;
    seen.push(("join response".into(), joined.to_string()));
    events.push(json!({ "type": "joined", "status": status.as_u16(), "body": joined }));
    let token = joined["token"].as_str().unwrap().to_string();

    let mut socket = w.attach(&room, &token).await;
    let take = |f: Value, seen: &mut Vec<(String, String)>, events: &mut Vec<Value>| {
        seen.push((format!("frame {}", f["phase"]), f.to_string()));
        events.push(json!({ "type": "frame", "frame": f.clone() }));
        f
    };
    let f = frame(&mut socket, WAIT).await;
    take(f, &mut seen, &mut events); // idle, the attach frame

    w.act(&room, "put-on-screen").await;
    let live = take(frame(&mut socket, WAIT).await, &mut seen, &mut events);
    assert_eq!(live["phase"], "live");

    // Two taps, each answered by the real route: the last one wins (AC-34).
    let uri = format!("/rooms/{}/answer", room.id);
    for letter in ["A", "C"] {
        events.push(json!({ "type": "tap", "letter": letter }));
        let (status, body) = http(&w.app, Method::PUT, &uri, Some(&token), Some(json!({ "letter": letter }))).await;
        seen.push(("answer response".into(), body.to_string()));
        events.push(json!({ "type": "answered", "status": status.as_u16(), "body": body }));
        if letter == "A" {
            // The first answer moves answered_live, so a frame is pushed; a
            // change of answer moves no count and pushes nothing.
            take(frame(&mut socket, WAIT).await, &mut seen, &mut events);
        }
    }
    // The hint: from the frame the page holds.
    events.push(json!({ "type": "hint" }));

    w.act(&room, "close-answers").await;
    take(frame(&mut socket, WAIT).await, &mut seen, &mut events);
    // A tap after close: the page must not send it (AC-58).
    events.push(json!({ "type": "tap", "letter": "B" }));

    // The phone drops; the room moves on without it.
    socket.close(None).await.unwrap();
    events.push(json!({ "type": "closed", "code": 1006 }));
    w.act(&room, "show-split").await;

    // Back, with the same token (AC-37).
    let mut socket = w.attach(&room, &token).await;
    let back = take(frame(&mut socket, WAIT).await, &mut seen, &mut events);
    assert_eq!(back["session"]["saved"], "C");

    let mut phases = Vec::new();
    for slug in ["walk-it", "reveal", "release"] {
        w.act(&room, slug).await;
        phases.push(take(frame(&mut socket, WAIT).await, &mut seen, &mut events));
    }
    let reveal = phases[1].clone();
    assert_eq!(reveal["phase"], "reveal");

    let shown = replay(&events);
    assert_eq!(shown.len(), events.len());
    let at = |i: usize| &shown[i];
    let find = |kind: &str, phase: &str| -> usize {
        (0..events.len())
            .find(|&i| events[i]["type"] == kind && (phase.is_empty() || events[i]["frame"]["phase"] == phase))
            .unwrap_or_else(|| panic!("no {kind} {phase}"))
    };

    // Joined by the link, paused until the attach frame, then the room's phase.
    assert_eq!(at(0)["effects"], json!(["join"]));
    assert_eq!(at(1)["screen"], "room");
    assert_eq!(at(1)["conn"], "paused");
    assert_eq!(at(2)["phase"], "idle");
    assert_eq!(at(2)["conn"], "attached");

    // Each tap sent exactly its letter; each response left exactly the saved
    // state the server holds (AC-35).
    let taps: Vec<usize> = (0..events.len()).filter(|&i| events[i]["type"] == "tap").collect();
    assert_eq!(at(taps[0])["puts"], json!(["A"]));
    assert_eq!(at(taps[0])["submit"], "saving");
    assert_eq!(at(taps[1])["puts"], json!(["C"]));
    let answered: Vec<usize> = (0..events.len()).filter(|&i| events[i]["type"] == "answered").collect();
    assert_eq!(at(answered[1])["saved"], "C");
    assert_eq!(at(answered[1])["submit"], "saved");
    assert!(at(answered[1])["html"].as_str().unwrap().contains("saved — C"));

    // The hint came from the held frame: no request.
    let hint = find("hint", "");
    assert_eq!(at(hint)["effects"], json!(["announce"]));
    assert!(at(hint)["html"].as_str().unwrap().contains(live["hint"]["text"].as_str().unwrap()));

    // Closed: the saved answer restated; a later tap sends nothing (AC-58).
    let closed = find("frame", "closed");
    assert_eq!(at(closed)["phase"], "closed");
    assert!(at(closed)["html"].as_str().unwrap().contains("you said <b>C</b>"));
    assert_eq!(at(taps[2])["puts"], json!([]));
    assert!(shown.iter().skip(closed).all(|s| s["puts"] == json!([])), "nothing is sent after close");

    // Dropped: paused, the answer named safe (AC-37).
    let dropped = find("closed", "");
    assert_eq!(at(dropped)["conn"], "paused");
    assert!(at(dropped)["html"].as_str().unwrap().contains("paused — reconnecting… your answer C is safe"));

    // Back in `split`: attached, the saved answer from the server's map, and
    // the count the page computed is the server's own total for C (AC-58).
    let split = find("frame", "split");
    assert_eq!(at(split)["conn"], "attached");
    assert_eq!(at(split)["saved"], "C");
    let c_total = back["counts"]["totals"][2].as_u64().unwrap();
    assert_eq!(at(split)["count"], c_total);

    // Reveal: the server's correct letter, marked, and never a mark against C.
    let rv = find("frame", "reveal");
    let correct = reveal["correct"].as_str().unwrap();
    let html = at(rv)["html"].as_str().unwrap();
    assert!(html.contains(&format!("It was {correct}.")), "{html}");
    assert!(!html.contains('✗') && !html.to_lowercase().contains("wrong"));
    assert_eq!(at(find("frame", "released"))["phase"], "released");

    // G-8, AC-32: nothing a phone received, and nothing it renders, carries
    // the source or the trace.
    for (what, text) in &seen {
        assert_no_source_or_trace(what, text);
    }
    for s in &shown {
        assert_no_source_or_trace("the rendered page", s["html"].as_str().unwrap());
    }
    for uri in ["/join", "/join/buzzer.js", "/join/buzzer.css"] {
        assert_no_source_or_trace(uri, &get(&w.app, uri).await.2);
    }
}

// --------------------------------------------------------------------------
// For validation in a browser, never in `test`:
//
//     cd room && BUZZER_PORT=3106 cargo test --test buzzer_page -- --ignored --nocapture
//
// Serves the router with the test auth and a `/shared` helper (T-05 owns the
// real route), creates one room, prints the join link and the host's bearer,
// and stays up for `BUZZER_SECS` (default 1200) seconds.
// --------------------------------------------------------------------------

async fn shared(axum::extract::Path(file): axum::extract::Path<String>) -> axum::response::Response {
    use axum::response::IntoResponse;
    if file.split('/').any(|p| p == ".." || p.is_empty()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Ok(bytes) = std::fs::read(repo().join("web/shared").join(&file)) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let kind = match file.rsplit('.').next() {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("ttf") => "font/ttf",
        _ => "application/octet-stream",
    };
    ([(header::CONTENT_TYPE, kind)], bytes).into_response()
}

#[tokio::test]
#[ignore = "a browser harness for validation, not a test"]
async fn serve_for_browser() {
    let port: u16 = std::env::var("BUZZER_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3106);
    let secs: u64 = std::env::var("BUZZER_SECS").ok().and_then(|p| p.parse().ok()).unwrap_or(1200);
    let base = format!("http://127.0.0.1:{port}");
    let urls = Urls { base: base.clone(), home: format!("{base}/home") };
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], urls));
    let app = axum::Router::new()
        .route("/shared/{*file}", axum::routing::get(shared))
        .merge(room::router_with(state));
    let room = host_room(&app).await;
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await.unwrap();
    println!("BUZZER {base}/{}", room.code);
    println!("ROOM {}", room.id);
    println!("HOST {}", room.host);
    tokio::spawn(room::ws::serve(listener, app));
    tokio::time::sleep(Duration::from_secs(secs)).await;
}
