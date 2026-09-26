//! Shared test fixtures: the test `HostAuth`, the questions, and the planted
//! canaries.
//!
//! Every question here is `bank/questions/q3.json` read from disk, sometimes
//! with authored prose swapped for a canary string. Nothing here writes down
//! what a program prints: q3's `stdout` is the verifier's, and the one
//! does-not-compile variant describes no program and says so.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use room::answers::{self, Scheduled};
use room::auth::{Denied, HostAuth, OrganizerId};
use room::rooms::{CloseSnapshot, Sessions, Totals};
use serde_json::Value;

pub const ORGANIZER: &str = "test-organizer-credential";
pub const OTHER_ORGANIZER: &str = "another-organizer-credential";

/// The test implementation of the auth seam. It exists only here: the crate
/// ships `DenyAll` and nothing that accepts a credential (G-9, §8.2).
pub struct TestAuth;

impl HostAuth for TestAuth {
    fn authorize_create(&self, bearer: Option<&str>) -> Result<OrganizerId, Denied> {
        match bearer {
            Some(ORGANIZER) => Ok(OrganizerId("organizer-1".into())),
            Some(OTHER_ORGANIZER) => Ok(OrganizerId("organizer-2".into())),
            _ => Err(Denied),
        }
    }
}

/// Sessions whose close snapshot is whatever the test set.
#[derive(Clone, Default)]
pub struct FakeSessions {
    pub totals: Arc<Mutex<Totals>>,
    pub closes: Arc<Mutex<u32>>,
}

impl Sessions for FakeSessions {
    fn close_snapshot(&self) -> CloseSnapshot {
        *self.closes.lock().unwrap() += 1;
        CloseSnapshot::from_totals(*self.totals.lock().unwrap())
    }
}

pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

pub fn q3_json() -> Value {
    let text = std::fs::read_to_string(repo().join("bank/questions/q3.json")).expect("q3.json");
    serde_json::from_str(&text).expect("q3.json parses")
}

pub fn load(v: &Value) -> Scheduled {
    answers::load(&v.to_string()).unwrap_or_else(|e| panic!("load: {e}"))
}

pub fn q3() -> Scheduled {
    load(&q3_json())
}

/// q3 with its second step dropped: M = 5, the final step still the one that
/// names `stdout`.
pub fn five_steps() -> Value {
    let mut v = q3_json();
    v["trace"]["steps"].as_array_mut().unwrap().remove(1);
    assert_eq!(v["trace"]["steps"].as_array().unwrap().len(), 5);
    v
}

// The canaries. T-08 owns the real plants and the page and frame scans; these
// are the seam it extends — a planted question, and `scan_pre_reveal` below.
pub const PLANT_RESOLVING_NOTE: &str = "CANARY-RESOLVING-NOTE";
pub const PLANT_WHAT: &str = "CANARY-EXPLAINS-WHAT";
pub const PLANT_TAKEAWAY: &str = "CANARY-EXPLAINS-TAKEAWAY";
pub const PLANT_HINT: &str = "CANARY-HINT";
pub const PLANT_MIDDLE_STDOUT: &str = "CANARY-MIDDLE-STDOUT";
pub const PLANT_ERROR_CODE: &str = "ECANARY0";

pub fn plant_why(letter: &str) -> String {
    format!("CANARY-WHY-TEMPTING-{letter}")
}

/// q3, id `planted`, with canaries in the resolving step's note, both beats,
/// every `why_tempting`, the hint, and a `stdout` row slipped into a middle
/// step (G-3 withholds *any* values entry named `stdout`, not only the last).
pub fn planted() -> Value {
    let mut v = q3_json();
    v["id"] = "planted".into();
    v["hint"] = PLANT_HINT.into();
    v["explains"]["what"] = PLANT_WHAT.into();
    v["explains"]["takeaway"] = PLANT_TAKEAWAY.into();
    for (i, letter) in ["A", "B", "C", "D", "E"].iter().enumerate() {
        if v["options"][i].get("why_tempting").is_some() {
            v["options"][i]["why_tempting"] = plant_why(letter).into();
        }
    }
    let steps = v["trace"]["steps"].as_array_mut().unwrap();
    let last = steps.len() - 1;
    steps[last]["note"] = PLANT_RESOLVING_NOTE.into();
    steps[2]["values"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"name": "stdout", "was": "—", "now": PLANT_MIDDLE_STDOUT}));
    v
}

/// A SYNTHETIC does-not-compile record, id `planted-dnc`: q3's prose with a
/// verified record that names no program — its one error code is a canary and
/// no compiler produced it. The receipt it renders is `✓ Error ECANARY0`,
/// which is how "the receipt" is planted when the receipt is derived.
pub fn planted_dnc() -> Value {
    let mut v = planted();
    v["id"] = "planted-dnc".into();
    v["verified"] = serde_json::json!({
        "rustc": "SYNTHETIC - no compiler ran",
        "edition": "2021",
        "compile_error_code": [PLANT_ERROR_CODE],
    });
    // The does-not-compile option (D) is now the correct one, so the output
    // option that was (E) needs its own middle beat.
    v["options"][4]["why_tempting"] = plant_why("E").into();
    v["options"][3].as_object_mut().unwrap().remove("why_tempting");
    v
}

/// Every string at any depth, and every key.
pub fn walk<'a>(v: &'a Value, out: &mut Vec<(&'a str, &'a Value)>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map {
                out.push((k.as_str(), child));
                walk(child, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|i| walk(i, out)),
        _ => {}
    }
}

pub fn keys_named<'a>(v: &'a Value, name: &str) -> Vec<&'a Value> {
    let mut all = Vec::new();
    walk(v, &mut all);
    all.into_iter().filter(|(k, _)| *k == name).map(|(_, v)| v).collect()
}

/// Every object anywhere in `v` that has a `name` equal to `stdout`.
pub fn stdout_rows(v: &Value) -> usize {
    keys_named(v, "name").iter().filter(|n| n.as_str() == Some("stdout")).count()
}

// T-04b: an in-process driver for the participant routes.

/// One request through the router, no socket: `(status, JSON body or Null)`.
pub async fn http(
    app: &axum::Router,
    method: axum::http::Method,
    uri: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> (axum::http::StatusCode, Value) {
    use axum::body::Body;
    use axum::http::{header, Request};
    use tower::ServiceExt;
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
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// A room as its host and its participants know it.
pub struct HostedRoom {
    pub id: String,
    pub code: String,
    pub host: String,
}

/// *Create a room* on q3 as [`ORGANIZER`].
pub async fn host_room(app: &axum::Router) -> HostedRoom {
    let (status, created) = http(
        app,
        axum::http::Method::POST,
        "/rooms",
        Some(ORGANIZER),
        Some(serde_json::json!({"question_id": "q3"})),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED, "{created}");
    HostedRoom {
        id: created["id"].as_str().unwrap().into(),
        code: created["code"].as_str().unwrap().into(),
        host: created["host_session"].as_str().unwrap().into(),
    }
}

// --------------------------------------------------------------------------
// T-04c: the transport's test session map and a loopback socket harness.
// --------------------------------------------------------------------------

use std::collections::HashMap;
use std::time::Duration;

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use futures_util::{SinkExt, StreamExt};
use room::question::Letter;
use room::rooms::{AppState, Urls};
use room::ws::{SessionId, SessionTokens, Transport};
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;

/// The test implementation of the transport's session seam: a token →
/// session map with each session's saved answer, and a log of `gone` calls.
/// T-04b's real map replaces it.
#[derive(Default)]
pub struct TestTokens {
    pub sessions: Mutex<HashMap<String, (SessionId, Option<Letter>)>>,
    pub gone: Mutex<Vec<SessionId>>,
}

impl TestTokens {
    pub fn add(&self, token: &str, session: u64, saved: Option<Letter>) {
        self.sessions
            .lock()
            .unwrap()
            .insert(token.to_string(), (SessionId(session), saved));
    }

    pub fn gone_count(&self, session: u64) -> usize {
        self.gone.lock().unwrap().iter().filter(|s| s.0 == session).count()
    }
}

impl SessionTokens for TestTokens {
    fn resolve(&self, _room_id: &str, token: &str) -> Option<SessionId> {
        self.sessions.lock().unwrap().get(token).map(|(s, _)| *s)
    }
    fn saved(&self, _room_id: &str, session: SessionId) -> Option<Letter> {
        self.sessions
            .lock()
            .unwrap()
            .values()
            .find(|(s, _)| *s == session)
            .and_then(|(_, saved)| *saved)
    }
    fn gone(&self, _room_id: &str, session: SessionId) {
        self.gone.lock().unwrap().push(session);
    }
}

pub type Socket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// The room served on `127.0.0.1:0` through `room::ws::serve`, with the
/// transport layered on. HTTP goes in-process through the same router (the
/// same layers run), sockets go over the loopback listener.
pub struct Live {
    pub app: axum::Router,
    pub addr: std::net::SocketAddr,
    pub state: Arc<AppState>,
    pub transport: Transport,
    pub tokens: Arc<TestTokens>,
}

pub struct LiveRoom {
    pub id: String,
    pub host: String,
}

impl Live {
    pub async fn start() -> Live {
        Live::start_with(|t| t).await
    }

    pub async fn start_with(configure: impl FnOnce(Transport) -> Transport) -> Live {
        let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default()));
        let tokens = Arc::new(TestTokens::default());
        let transport = configure(Transport::new(state.clone(), tokens.clone()));
        let app = room::router_with(state.clone()).layer(axum::Extension(transport.clone()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(room::ws::serve(listener, app.clone()));
        Live {
            app,
            addr,
            state,
            transport,
            tokens,
        }
    }

    pub async fn call(&self, method: Method, uri: &str, bearer: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
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
        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
        let json = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() };
        (status, json)
    }

    pub async fn create(&self) -> LiveRoom {
        let id = q3_json()["id"].as_str().unwrap().to_string();
        let (status, created) = self
            .call(Method::POST, "/rooms", Some(ORGANIZER), Some(serde_json::json!({ "question_id": id })))
            .await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
        LiveRoom {
            id: created["id"].as_str().unwrap().into(),
            host: created["host_session"].as_str().unwrap().into(),
        }
    }

    /// A host action through its HTTP route.
    pub async fn act(&self, room: &LiveRoom, slug: &str) -> StatusCode {
        self.call(Method::POST, &format!("/rooms/{}/{slug}", room.id), Some(&room.host), None)
            .await
            .0
    }

    pub fn url(&self, room: &LiveRoom, viewer: &str) -> String {
        format!("ws://{}/rooms/{}/ws/{viewer}", self.addr, room.id)
    }

    /// Open a socket; send `{"t":"attach","token"}` if a token is given.
    pub async fn open(&self, room: &LiveRoom, viewer: &str, token: Option<&str>) -> Socket {
        let (mut socket, _) = tokio_tungstenite::connect_async(self.url(room, viewer)).await.unwrap();
        if let Some(token) = token {
            let attach = serde_json::json!({ "t": "attach", "token": token }).to_string();
            socket.send(Message::text(attach)).await.unwrap();
        }
        socket
    }
}

/// What the next thing off a socket was.
#[derive(Debug)]
pub enum Next {
    Frame(Value),
    Closed(Option<u16>),
    Nothing,
}

/// The next frame or close within `wait`; `Nothing` if neither arrived.
pub async fn next(socket: &mut Socket, wait: Duration) -> Next {
    loop {
        match tokio::time::timeout(wait, socket.next()).await {
            Err(_) => return Next::Nothing,
            Ok(None) | Ok(Some(Err(_))) => return Next::Closed(None),
            Ok(Some(Ok(Message::Text(text)))) => return Next::Frame(serde_json::from_str(&text).unwrap()),
            Ok(Some(Ok(Message::Close(frame)))) => return Next::Closed(frame.map(|f| u16::from(f.code))),
            Ok(Some(Ok(_))) => continue,
        }
    }
}

/// The next frame, which must arrive within `wait`.
pub async fn frame(socket: &mut Socket, wait: Duration) -> Value {
    match next(socket, wait).await {
        Next::Frame(v) => v,
        other => panic!("expected a state frame, got {other:?}"),
    }
}

/// The close code the socket ends with, skipping nothing: a state frame
/// before the close is a failure.
pub async fn closed_with(socket: &mut Socket, wait: Duration) -> Option<u16> {
    match next(socket, wait).await {
        Next::Closed(code) => code,
        other => panic!("expected the socket to close, got {other:?}"),
    }
}
