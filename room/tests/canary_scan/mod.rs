//! T-08's scan core, shared by `canary.rs` (`just test`) and `canary_full.rs`
//! (`just test-full`). Not a test target: cargo compiles only the top-level
//! files in `tests/`, and each suite includes this with `#[path]`.
//!
//! **What a client can receive**, and so what is scanned at every revision:
//!
//! - every HTTP projection (`GET /rooms/{id}/wall|buzzer|host`);
//! - every response a client provokes: the host action's own, `POST /join`,
//!   `PUT …/answer`, `PUT …/fit`, the denials (`401`, `404`);
//! - every socket frame each attached socket received — one wall, one host,
//!   three buzzers — drained until it equals the viewer's projection, so no
//!   stop is scanned before its frame arrived;
//! - every page and every file the router serves, found by walking
//!   `src/routes.rs` itself ([`route_table`]);
//! - the wall's frames as the served `wall.js` renders them, and the real
//!   buzzer page, mounted under `node` against the running room (`page.js`).
//!
//! The rules are [`check`]'s, phase-scoped per SPEC G-3 and keyed on the
//! phase the payload itself names. Every plant a rule forbids is a
//! `CANARY-…` string from `common::plants()`.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write as _};
use std::net::SocketAddr;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{header, Method};
use axum::middleware::Next;
use axum::response::Response;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use room::copy;
use room::phase::{HostAction, Phase, Step};
use room::rooms::{AppState, Urls};
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite::Message;
use tower::ServiceExt;

use crate::common::*;

pub const WAIT: Duration = Duration::from_secs(5);

// --------------------------------------------------------------------------
// The server: the canary questions, the real session map, the real sockets.
// --------------------------------------------------------------------------

/// One request as the server saw it (the server's request log, AC-58).
#[derive(Clone, Debug)]
pub struct Logged {
    pub method: String,
    pub path: String,
    pub bearer: Option<String>,
    pub body: String,
}

pub type Log = Arc<Mutex<Vec<Logged>>>;

async fn record(State(log): State<Log>, request: Request, next: Next) -> Response {
    let (parts, body) = request.into_parts();
    // A GET carries no body; leaving it untouched keeps the socket upgrade's
    // request exactly as hyper made it.
    let bytes = if parts.method == Method::GET {
        axum::body::Bytes::new()
    } else {
        axum::body::to_bytes(body, 1 << 20).await.unwrap_or_default()
    };
    log.lock().unwrap().push(Logged {
        method: parts.method.to_string(),
        path: parts.uri.to_string(),
        bearer: parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(str::to_string),
        body: String::from_utf8_lossy(&bytes).into_owned(),
    });
    let request = if parts.method == Method::GET {
        Request::from_parts(parts, Body::empty())
    } else {
        Request::from_parts(parts, Body::from(bytes))
    };
    next.run(request).await
}

/// How the Rust side reaches the server's HTTP routes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Http {
    /// Through the router with `oneshot` — the same router, the same layers.
    InProcess,
    /// Over a real TCP connection to the loopback listener.
    Tcp,
}

pub struct Server {
    pub app: Router,
    pub addr: SocketAddr,
    pub state: Arc<AppState>,
    pub log: Log,
    pub http: Http,
}

/// A response, whole: what a client could read from it.
#[derive(Clone, Debug)]
pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Resp {
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or(Value::Null)
    }
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
    /// Everything a client receives, as one string: status, headers, body.
    pub fn everything(&self) -> String {
        let mut s = format!("{}\n", self.status);
        for (k, v) in &self.headers {
            s.push_str(&format!("{k}: {v}\n"));
        }
        s.push_str(&self.text());
        s
    }
}

impl Server {
    pub async fn start(http: Http) -> Server {
        plants(); // sets POPQUIZ_ADMIN_TOKEN before anything is served
        // T-25 — AC-101's plant, live: the admin channel is opened by the planted
        // token, read from the environment as the binary reads it, and the
        // canary questions arrive through it — nothing is seeded.
        let token = room::admin::AdminToken::from_var(std::env::var(room::admin::VAR).ok()).expect("the plant is set");
        let state = Arc::new(AppState::new(Arc::new(TestAuth), Vec::new(), Urls::default()));
        let log: Log = Arc::default();
        let app = room::router_with_admin(state.clone(), token)
            .layer(axum::middleware::from_fn_with_state(log.clone(), record));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(room::ws::serve(listener, app.clone()));
        let server = Server {
            app,
            addr,
            state,
            log,
            http,
        };
        for q in [canary_question(), canary_question_dnc()] {
            let id = q["id"].as_str().unwrap().to_string();
            let r = server
                .request(Method::PUT, &format!("/admin/questions/{id}"), Some(&plants().admin), Some(q))
                .await;
            assert_eq!(r.status, 201, "PUT /admin/questions/{id}: {}", r.text());
            check_admin(&r.everything(), &format!("PUT /admin/questions/{id}"));
        }
        server
    }

    pub fn base(&self) -> String {
        format!("http://{}", self.addr)
    }

    pub async fn request(&self, method: Method, uri: &str, bearer: Option<&str>, body: Option<Value>) -> Resp {
        match self.http {
            Http::InProcess => {
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
                let status = res.status().as_u16();
                let headers = res
                    .headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).into_owned()))
                    .collect();
                let body = axum::body::to_bytes(res.into_body(), 64 << 20).await.unwrap().to_vec();
                Resp { status, headers, body }
            }
            Http::Tcp => tcp_request(self.addr, method.as_str(), uri, bearer, body).await,
        }
    }

    pub async fn get(&self, uri: &str) -> Resp {
        self.request(Method::GET, uri, None, None).await
    }

    pub fn log_since(&self, mark: usize) -> Vec<Logged> {
        self.log.lock().unwrap()[mark..].to_vec()
    }

    pub fn log_len(&self) -> usize {
        self.log.lock().unwrap().len()
    }
}

/// HTTP/1.1 over a fresh loopback connection, `Connection: close`. No new
/// crate: the room's responses are small and whole, so a status line, headers,
/// and a `Content-Length` (or chunked) body are all there is to read.
pub async fn tcp_request(addr: SocketAddr, method: &str, uri: &str, bearer: Option<&str>, body: Option<Value>) -> Resp {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let body = body.map(|v| v.to_string()).unwrap_or_default();
    let mut head = format!("{method} {uri} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n");
    if let Some(b) = bearer {
        head.push_str(&format!("Authorization: Bearer {b}\r\n"));
    }
    if !body.is_empty() {
        head.push_str("Content-Type: application/json\r\n");
    }
    head.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
    stream.write_all(head.as_bytes()).await.unwrap();
    stream.write_all(body.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    tokio::time::timeout(WAIT, stream.read_to_end(&mut raw))
        .await
        .expect("the room answered within the wait")
        .unwrap();
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("a response head");
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let mut rest = raw[split + 4..].to_vec();
    let mut lines = head.split("\r\n");
    let status: u16 = lines.next().unwrap().split(' ').nth(1).unwrap().parse().unwrap();
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    if headers.iter().any(|(k, v)| k == "transfer-encoding" && v.contains("chunked")) {
        let mut out = Vec::new();
        loop {
            let at = rest.windows(2).position(|w| w == b"\r\n").unwrap();
            let size = usize::from_str_radix(String::from_utf8_lossy(&rest[..at]).trim(), 16).unwrap();
            if size == 0 {
                break;
            }
            out.extend_from_slice(&rest[at + 2..at + 2 + size]);
            rest.drain(..at + 2 + size + 2);
        }
        rest = out;
    }
    Resp {
        status,
        headers,
        body: rest,
    }
}

// --------------------------------------------------------------------------
// The sockets.
// --------------------------------------------------------------------------

pub type Socket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// One attached socket and every frame it has received.
pub struct Watcher {
    pub label: String,
    /// `wall`, `host` or `buzzer`.
    pub viewer: &'static str,
    pub socket: Socket,
    /// Frames not yet scanned.
    pub fresh: Vec<Value>,
    pub all: Vec<Value>,
    pub closed: Option<Option<u16>>,
}

impl Watcher {
    pub async fn attach(server: &Server, room: &str, viewer: &'static str, token: Option<&str>, label: &str) -> Watcher {
        let url = format!("ws://{}/rooms/{room}/ws/{viewer}", server.addr);
        let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();
        if let Some(token) = token {
            let attach = json!({ "t": "attach", "token": token }).to_string();
            socket.send(Message::text(attach)).await.unwrap();
        }
        let mut w = Watcher {
            label: label.to_string(),
            viewer,
            socket,
            fresh: Vec::new(),
            all: Vec::new(),
            closed: None,
        };
        w.read_one(WAIT).await.expect("the attach frame arrives first (AC-37)");
        w
    }

    /// One frame, or `None` on timeout or close.
    pub async fn read_one(&mut self, wait: Duration) -> Option<Value> {
        loop {
            match tokio::time::timeout(wait, self.socket.next()).await {
                Err(_) => return None,
                Ok(None) | Ok(Some(Err(_))) => {
                    self.closed = Some(None);
                    return None;
                }
                Ok(Some(Ok(Message::Text(text)))) => {
                    let v: Value = serde_json::from_str(&text).unwrap();
                    self.fresh.push(v.clone());
                    self.all.push(v.clone());
                    return Some(v);
                }
                Ok(Some(Ok(Message::Close(frame)))) => {
                    self.closed = Some(frame.map(|f| u16::from(f.code)));
                    return None;
                }
                Ok(Some(Ok(_))) => continue,
            }
        }
    }

    /// Read until the newest frame, as a payload, equals `projection`.
    pub async fn catch_up(&mut self, projection: &Value) {
        if self.fresh.last().or(self.all.last()).is_some_and(|f| as_payload(f) == *projection) {
            return;
        }
        loop {
            let Some(f) = self.read_one(WAIT).await else {
                panic!(
                    "{}: no frame matching the {} projection arrived (last: {:?})",
                    self.label,
                    self.viewer,
                    self.all.last()
                );
            };
            if as_payload(&f) == *projection {
                return;
            }
        }
    }

    /// Anything more, within `wait`: frames that arrive when nothing should.
    pub async fn quiet_for(&mut self, wait: Duration) -> Vec<Value> {
        let mut extra = Vec::new();
        while let Some(f) = self.read_one(wait).await {
            extra.push(f);
        }
        extra
    }

    pub fn take_fresh(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.fresh)
    }
}

/// A frame as a payload: the envelope (`t`, `revision`) and the buzzer
/// attach frame's own `session` removed.
pub fn as_payload(frame: &Value) -> Value {
    let mut v = frame.clone();
    if let Some(o) = v.as_object_mut() {
        o.remove("t");
        o.remove("revision");
        o.remove("session");
    }
    v
}

// --------------------------------------------------------------------------
// The route walk: every path `src/routes.rs` registers.
// --------------------------------------------------------------------------

/// One registered route: its path pattern and the methods it serves.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Route {
    pub method: &'static str,
    pub path: String,
}

// T-25 admin plant ---------------------------------------------------------
//
// AC-101's canary half, live: the helpers the walk calls at every stop and
// after release. The other T-25 lines are marked in place: `Server::start`
// (the channel opened by the plant, the questions pushed), `route_table`'s
// `any(`, three `DRIVEN` rows, one call in `stop` and the release check in
// `walk_room`.

impl Server {
    /// AC-101 at every stop: the admin prefix refuses a missing or wrong
    /// token saying nothing, and what the right token reads carries no plant.
    pub async fn admin_probe(&self, question: &str, at: &str) -> Resp {
        for bearer in [None, Some("not-the-admin-token"), Some(self.probe_bearer())] {
            for (method, uri) in [(Method::GET, "/admin/used".to_string()), (Method::PUT, format!("/admin/questions/{question}"))] {
                let body = (method == Method::PUT).then(canary_question);
                let r = self.request(method.clone(), &uri, bearer, body).await;
                assert_eq!((r.status, r.body.len()), (401, 0), "{at}: {method} {uri} refused, saying nothing");
                assert_eq!(r.header("www-authenticate"), None, "{at}: {method} {uri}");
                check_admin(&r.everything(), &format!("{at}: {method} {uri}, refused"));
            }
        }
        let r = self.request(Method::GET, "/admin/used", Some(&plants().admin), None).await;
        assert_eq!(r.status, 200, "{at}: GET /admin/used");
        check_admin(&r.everything(), &format!("{at}: GET /admin/used"));
        r
    }

    /// A bearer the room did issue (the organizer's), which is not the admin
    /// token: no other credential opens the admin prefix.
    fn probe_bearer(&self) -> &'static str {
        ORGANIZER
    }
}

/// The rule for the admin channel's own responses: no plant of any kind,
/// the admin token's included, in any phase.
pub fn check_admin(text: &str, at: &str) {
    for (name, plant) in plants().all() {
        assert!(!text.contains(plant), "{at}: an admin response carries the {name} plant");
    }
}

/// The canary record with the given id (the two canary questions share a
/// body shape; the admin push needs the path and the record to agree).
pub fn canary_question_named(id: &str) -> Value {
    if id == canary_question_dnc()["id"] {
        canary_question_dnc()
    } else {
        canary_question()
    }
}

// end T-25 admin plant -----------------------------------------------------

fn routes_rs() -> String {
    std::fs::read_to_string(repo().join("room/src/routes.rs")).expect("room/src/routes.rs")
}

/// Every route `routes.rs` registers, read from the file itself: each
/// `.route(` literal (or `format!` over the three viewers, or the host-action
/// family `room::host_routes()` returns) with the method handler it names.
/// A route added to the file joins this list whether or not the canary knows
/// it, and [`assert_every_route_is_scanned`] then fails. Textual, and scoped to
/// this one file: a router `merge`d or `nest`ed from elsewhere would escape it,
/// so a split router must extend this walk.
pub fn route_table() -> BTreeSet<Route> {
    let src = routes_rs();
    let mut out = BTreeSet::new();
    let mut rest = src.as_str();
    while let Some(at) = rest.find(".route(") {
        rest = &rest[at + ".route(".len()..];
        let call_end = rest.find(".route(").unwrap_or(rest.len());
        let call = &rest[..call_end];
        let trimmed = call.trim_start();
        // T-25: `any(` is the admin prefix's one entry point.
        let method = ["get(", "post(", "put(", "axum::routing::put(", "axum::routing::get(", "axum::routing::any("]
            .iter()
            .filter_map(|m| call.find(m).map(|i| (i, *m)))
            .min()
            .map(|(_, m)| {
                if m.contains("any") {
                    "ANY"
                } else if m.contains("put") {
                    "PUT"
                } else if m.contains("post") {
                    "POST"
                } else {
                    "GET"
                }
            })
            .unwrap_or("?");
        let paths: Vec<String> = if let Some(lit) = trimmed.strip_prefix('"') {
            vec![lit[..lit.find('"').unwrap()].to_string()]
        } else if let Some(fmt) = trimmed.strip_prefix("&format!(\"") {
            let pat = fmt[..fmt.find('"').unwrap()].replace("{{", "{").replace("}}", "}");
            ["wall", "buzzer", "host"].iter().map(|v| pat.replace("{name}", v)).collect()
        } else if trimmed.starts_with("&path") {
            room::host_routes().into_iter().map(|(_, p)| p).collect()
        } else {
            panic!("routes.rs registers a route the canary cannot read: {}", &trimmed[..trimmed.len().min(80)]);
        };
        // `&path` is registered twice (run-it-again, and every other action);
        // both are POST.
        let method = if trimmed.starts_with("&path") { "POST" } else { method };
        for path in paths {
            out.insert(Route { method, path });
        }
    }
    out
}

/// The files `/shared/{file}` and `/shared/fonts/{file}` serve, read from the
/// `include_*!` lines in `routes.rs`.
pub fn served_files(dir: &str) -> Vec<String> {
    let src = routes_rs();
    let needle = format!("../../web/{dir}/");
    let mut out = Vec::new();
    let mut rest = src.as_str();
    while let Some(at) = rest.find(&needle) {
        rest = &rest[at + needle.len()..];
        let name = &rest[..rest.find('"').unwrap()];
        if !name.contains('/') {
            out.push(name.to_string());
        }
    }
    out
}

/// What the scan drives, route by route. A route in [`route_table`] that is in
/// neither this nor [`UNSCANNED`] fails [`assert_every_route_is_scanned`].
pub const DRIVEN: &[(&str, &str)] = &[
    ("POST", "/rooms"),
    ("POST", "/join"),
    ("PUT", "/rooms/{id}/answer"),
    ("PUT", "/rooms/{id}/fit"),
    ("GET", "/rooms/{id}/wall"),
    ("GET", "/rooms/{id}/buzzer"),
    ("GET", "/rooms/{id}/host"),
    ("GET", "/rooms/{id}/ws/wall"),
    ("GET", "/rooms/{id}/ws/buzzer"),
    ("GET", "/rooms/{id}/ws/host"),
    ("GET", "/wall/{room_id}"),
    ("GET", "/wall/wall.js"),
    ("GET", "/wall/wall.css"),
    ("GET", "/wall/qr.js"),
    ("GET", "/shared/{file}"),
    ("GET", "/shared/fonts/{file}"),
    ("GET", "/host"),
    ("GET", "/host/{room_id}"),
    ("GET", "/host/host.js"),
    ("GET", "/host/host.css"),
    ("GET", "/join"),
    ("GET", "/join/buzzer.js"),
    ("GET", "/join/buzzer.css"),
    ("GET", "/{code}"),
    // T-12: take it home (SPEC §13).
    ("GET", "/last"),
    ("GET", "/last/{club}"),
    ("GET", "/home/home.js"),
    ("GET", "/home/home.css"),
    // T-25: the admin channel, every method, at every stop (`admin_probe`).
    ("ANY", "/admin"),
    ("ANY", "/admin/"),
    ("ANY", "/admin/{*rest}"),
];

/// Routes the scan does not drive, and why. Empty: every route is driven.
/// The host actions (`POST /rooms/{id}/<action>`) are driven by the walk
/// itself and checked against `room::host_routes()` separately.
pub const UNSCANNED: &[(&str, &str, &str)] = &[
    // T-10 ------------------------------------------------------------------
    ("GET", "/auth/discord", "T-10 sign-in start: a redirect to Discord; served only with the Discord backend, which the canary's TestAuth state has not; tests/auth.rs secrecy_* scans it with Discord canaries"),
    ("GET", "/auth/discord/callback", "T-10 sign-in callback: served only with the Discord backend, which the canary's TestAuth state has not; tests/auth.rs secrecy_* scans it with Discord canaries"),
    // end T-10 --------------------------------------------------------------
];

pub fn assert_every_route_is_scanned() {
    let table = route_table();
    let actions: BTreeSet<String> = room::host_routes().into_iter().map(|(_, p)| p).collect();
    let known: BTreeSet<(&str, &str)> = DRIVEN
        .iter()
        .copied()
        .chain(UNSCANNED.iter().map(|(m, p, _)| (*m, *p)))
        .collect();
    for r in &table {
        if r.method == "POST" && actions.contains(&r.path) {
            continue;
        }
        assert!(
            known.contains(&(r.method, r.path.as_str())),
            "routes.rs serves {} {} and the canary neither scans it nor lists it as unscanned",
            r.method,
            r.path
        );
    }
    for (m, p) in DRIVEN {
        assert!(
            table.iter().any(|r| r.method == *m && r.path == *p),
            "the canary drives {m} {p}, which routes.rs no longer serves"
        );
    }
    assert!(actions.len() >= 9, "the host actions: {actions:?}");
}

// --------------------------------------------------------------------------
// The rules.
// --------------------------------------------------------------------------

/// Where a thing was received. The rules differ by who can read it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Surface {
    /// The wall's payload: projection or frame.
    Wall,
    /// The host's payload: projection, frame, or a host action's response.
    Host,
    /// A buzzer payload that carries the room's buzzer view: projection,
    /// frame, the join response.
    Buzzer,
    /// Any other response a participant provokes: the answer route's.
    BuzzerOther,
    /// A served page or file: the same bytes in every phase.
    Page,
    /// The wall's rendered HTML (served `wall.js` over a real frame).
    RenderedWall,
    /// The real buzzer page's rendered HTML.
    RenderedBuzzer,
    /// A request the buzzer page made, or a message it sent.
    PageTraffic,
    /// T-12: `GET /last`, the last **released** question (SPEC §13). Nothing
    /// of the canary question before its own release; all of it after.
    TakeHome,
    /// T-12: `GET /last` while it still holds an *earlier* room's release —
    /// the second room of `canary_full`, whose question shares every plant
    /// with the first. A plant scan cannot tell that page from a leak, so the
    /// walk holds it to more: byte-identical to what it served before this
    /// room could release (`Walk::stop`). Only the admin-token rule applies.
    TakeHomeEarlier,
}

impl Surface {
    fn participant(self) -> bool {
        matches!(
            self,
            Surface::Buzzer | Surface::BuzzerOther | Surface::RenderedBuzzer | Surface::PageTraffic | Surface::Page
        )
    }
}

/// Which plants were seen where: the positive controls read this.
#[derive(Default)]
pub struct Seen(pub BTreeMap<(Surface, &'static str), BTreeSet<String>>);

impl Seen {
    pub fn has(&self, surface: Surface, phase: Phase, plant: &str) -> bool {
        self.0
            .get(&(surface, phase_name(phase)))
            .is_some_and(|s| s.contains(plant))
    }
}

pub fn phase_name(p: Phase) -> &'static str {
    ["idle", "live", "closed", "split", "work", "reveal", "released"][p.index()]
}

pub fn phase_of(name: &str) -> Phase {
    Phase::ALL[["idle", "live", "closed", "split", "work", "reveal", "released"]
        .iter()
        .position(|n| *n == name)
        .unwrap_or_else(|| panic!("no such phase: {name}"))]
}

fn count(hay: &str, needle: &str) -> usize {
    hay.matches(needle).count()
}

/// The rules for one received thing. `json` is the parsed payload where there
/// is one; `text` is everything the client received. Panics with `at` and
/// the offending plant on a violation.
pub fn check(seen: &mut Seen, surface: Surface, phase: Phase, text: &str, json: Option<&Value>, at: &str) {
    let p = plants();
    let at = format!("{at} [{surface:?} in {}]", phase_name(phase));
    let pre = phase.index() < Phase::Reveal.index();

    for (name, plant) in p.all() {
        if text.contains(plant) {
            seen.0.entry((surface, phase_name(phase))).or_default().insert(plant.to_string());
        }
        let _ = name;
    }

    // AC-101's canary half: the admin token, nowhere, ever.
    assert!(!text.contains(&p.admin), "{at}: carries POPQUIZ_ADMIN_TOKEN");

    // T-12: /last holds the last *released* question (§13). Before the canary
    // question's release it is the no-release page or an earlier, different
    // question, so it carries no plant; from `released` on it is this one, and
    // carries it — answer, beats, every why_tempting, the receipt — by design.
    if surface == Surface::TakeHomeEarlier {
        return;
    }
    if surface == Surface::TakeHome {
        if phase != Phase::Released {
            for (name, plant) in p.all() {
                assert!(!text.contains(plant), "{at}: /last carries the {name} plant before its release");
            }
        }
        return;
    }

    // Pages are compile-time bytes: nothing of any question, in any phase.
    if surface == Surface::Page {
        for (name, plant) in p.all() {
            assert!(!text.contains(plant), "{at}: a served page carries the {name} plant");
        }
        return;
    }

    // `released` closes the answers again: every plant is gone (§4 table).
    if phase == Phase::Released {
        for (name, plant) in p.all() {
            assert!(!text.contains(plant), "{at}: released, and still carries the {name} plant");
        }
    }

    // G-8 / AC-32: no source, no trace, no option text on a participant surface.
    if surface.participant() {
        assert!(!text.contains(&p.source), "{at}: a participant surface carries the source (G-8)");
        for (i, note) in p.notes.iter().enumerate() {
            assert!(!text.contains(note.as_str()), "{at}: a participant surface carries trace step {i}'s note (G-8)");
        }
        for (name, plant) in [
            ("resolving note", &p.resolving),
            ("trace value", &p.trace_value),
            ("middle stdout", &p.middle_stdout),
            ("correct option text", &p.correct),
            // The explanation is the host's to read aloud; the receipt is the wall's.
            ("explains.what", &p.what),
            ("explains.takeaway", &p.takeaway),
            ("receipt", &p.receipt),
            ("error code", &p.error_code),
        ] {
            assert!(!text.contains(plant.as_str()), "{at}: a participant surface carries the {name} (G-8, G-3)");
        }
        for why in &p.why {
            assert!(!text.contains(why.as_str()), "{at}: a participant surface carries a why_tempting");
        }
        let q = canary_question();
        for o in q["options"].as_array().unwrap() {
            let t = o["text"].as_str().unwrap();
            assert!(!text.contains(t), "{at}: a participant surface carries option text {t:?} (G-8)");
        }
        if let Some(v) = json {
            for key in ["source", "trace", "options", "step"] {
                assert!(keys_named(v, key).is_empty(), "{at}: a participant payload has a {key:?} key (G-8)");
            }
        }
    }

    // AC-48: the hint, before `live` nowhere; in `live` in every buzzer view
    // and nowhere else; from `closed` on nowhere.
    let hint_here = text.contains(&p.hint);
    match (phase, surface) {
        (Phase::Live, Surface::Buzzer) => assert!(hint_here, "{at}: the live buzzer view lacks the hint (AC-48)"),
        (Phase::Live, Surface::RenderedBuzzer) => {}
        _ => assert!(!hint_here, "{at}: carries the hint outside the live buzzer (AC-48, AC-60)"),
    }

    if pre {
        // G-3 / AC-47 / AC-60: nothing that joins an option to the output.
        for (name, plant) in [
            ("resolving note", &p.resolving),
            ("explains.what", &p.what),
            ("explains.takeaway", &p.takeaway),
            ("receipt", &p.receipt),
            ("error code", &p.error_code),
            ("middle stdout", &p.middle_stdout),
        ] {
            assert!(!text.contains(plant.as_str()), "{at}: carries the {name} before reveal (G-3)");
        }
        for (letter, why) in ["A", "B", "C", "D", "E"].iter().zip(&p.why) {
            assert!(!text.contains(why.as_str()), "{at}: carries why_tempting {letter} before reveal (G-3)");
        }
        assert!(!text.contains('✓'), "{at}: carries a ✓ before reveal (G-3, AC-97)");
        assert!(!text.contains(copy::RECEIPT_HEADING), "{at}: carries the receipt heading before reveal");
        for line in [
            copy::RECEIPT_COMPILED,
            copy::RECEIPT_NOTHING_RAN,
            copy::RECEIPT_COMPILER_REFUSED,
            copy::RECEIPT_OUTPUT_NEVER_VARIED,
            copy::RECEIPT_MIRI_CLEAN,
        ] {
            assert!(!text.contains(line.trim_start_matches("✓ ")), "{at}: carries receipt line {line:?}");
        }

        // The correct option's text is public — as option E, exactly once,
        // on the wall, from `live` to `work` — and nowhere else (the join).
        // (The rendered wall may show bars instead of options: at most once.)
        let expected = match surface {
            Surface::Wall if phase != Phase::Idle => 1,
            _ => 0,
        };
        let seen_here = count(text, &p.correct);
        if surface == Surface::RenderedWall {
            assert!(seen_here <= 1, "{at}: the rendered wall shows the correct option's text twice (G-3)");
        } else {
            assert_eq!(seen_here, expected, "{at}: the correct option's text appears outside option E's slot (G-3)");
        }

        if let Some(v) = json {
            if expected == 1 {
                assert_eq!(
                    v["options"][4]["text"].as_str(),
                    Some(p.correct.as_str()),
                    "{at}: the correct option's text is somewhere other than option E's slot (G-3)"
                );
            }
            assert_eq!(stdout_rows(v), 0, "{at}: a values entry named stdout before reveal (G-3)");
            for key in ["correct", "kind", "why_tempting", "explains", "receipt", "reveal", "read_aloud", "mark", "exit_code", "stdout", "verified"] {
                assert!(keys_named(v, key).is_empty(), "{at}: has a {key:?} key before reveal");
            }
            // D-10: no trace position beyond M-2.
            let m = canary_question()["trace"]["steps"].as_array().unwrap().len() as u64;
            for pos in keys_named(v, "at") {
                assert!(pos.as_u64().unwrap() <= m - 2, "{at}: trace at {pos}, beyond M-2 (D-10, AC-97)");
            }
            // Every option object exactly {letter, text}, A–E, in bank order.
            let q = canary_question();
            for options in keys_named(v, "options") {
                let options = options.as_array().unwrap();
                assert_eq!(options.len(), 5, "{at}");
                for (i, o) in options.iter().enumerate() {
                    let keys: Vec<&String> = o.as_object().unwrap().keys().collect();
                    assert_eq!(keys, ["letter", "text"], "{at}: option {i} carries more than letter and text");
                    assert_eq!(o["letter"], ["A", "B", "C", "D", "E"][i], "{at}");
                    assert_eq!(o["text"], q["options"][i]["text"], "{at}: option order");
                }
            }
            if surface == Surface::Wall && phase == Phase::Work {
                // AC-97: no colour in `work`.
                assert_eq!(v["colour"], json!(false), "{at}: the work wall is coloured (AC-97)");
            }
        }
    }

    if phase == Phase::Reveal && matches!(surface, Surface::Wall | Surface::RenderedWall) {
        for (name, plant) in [("explains.what", &p.what), ("explains.takeaway", &p.takeaway)] {
            assert!(!text.contains(plant.as_str()), "{at}: the wall carries the host's {name}");
        }
        for why in &p.why {
            assert!(!text.contains(why.as_str()), "{at}: the wall carries a why_tempting");
        }
    }

    // PQ-37: the phone's split — the wall's five bars — only from `split` on
    // (split, work, reveal), and nothing in it but a letter, a count and a
    // percent per option, A–E in order. Before the split it would publish the
    // room's votes while answers are open; after release nothing of the room
    // is left (G-3, G-4, AC-58).
    if let (Some(v), Surface::Buzzer | Surface::BuzzerOther) = (json, surface) {
        let splits = keys_named(v, "split");
        if matches!(phase, Phase::Split | Phase::Work | Phase::Reveal) {
            for split in splits {
                assert_phone_split(split, &at);
            }
        } else {
            assert!(splits.is_empty(), "{at}: a buzzer payload carries the split outside split, work and reveal (PQ-37)");
        }
    }

    // AC-81: a payload names one phase, the room's.
    if let (Some(v), Surface::Wall | Surface::Host | Surface::Buzzer) = (json, surface) {
        let phases = keys_named(v, "phase");
        if !phases.is_empty() {
            assert_eq!(phases.len(), 1, "{at}: phase fields");
            assert_eq!(phases[0], &json!(phase_name(phase)), "{at}: the payload names another phase");
        }
    }
}

/// PQ-37: the phone's `split` is exactly five `{letter, count, percent}`
/// entries, A–E in order, whole numbers, a percent no more than 100.
pub fn assert_phone_split(split: &Value, at: &str) {
    let bars = split.as_array().unwrap_or_else(|| panic!("{at}: the buzzer's split is not an array (PQ-37)"));
    assert_eq!(bars.len(), 5, "{at}: the buzzer's split has {} entries (PQ-37)", bars.len());
    for (i, b) in bars.iter().enumerate() {
        let mut keys: Vec<&String> = b.as_object().unwrap_or_else(|| panic!("{at}: split entry {i}")).keys().collect();
        keys.sort();
        assert_eq!(keys, ["count", "letter", "percent"], "{at}: split entry {i} carries more than letter, count and percent (PQ-37)");
        assert_eq!(b["letter"], ["A", "B", "C", "D", "E"][i], "{at}: split order (PQ-37)");
        assert!(b["count"].is_u64(), "{at}: split entry {i}'s count is not a whole number");
        assert!(b["percent"].as_u64().is_some_and(|p| p <= 100), "{at}: split entry {i}'s percent");
    }
}

/// PQ-37: the real buzzer page's bars — five rows, A–E, the page's own letter
/// the one marked row; the ✓ on the correct letter's row at reveal and on no
/// row before it; no data-count (the phone draws the room's counts, it
/// computes none).
pub fn assert_page_bars(html: &str, yours: &str, correct: Option<&str>, at: &str) {
    let rows: Vec<&str> = html.split("<div class=\"bar-row").skip(1).collect();
    assert_eq!(rows.len(), 5, "{at}: the page draws five bars: {html}");
    for (i, row) in rows.iter().enumerate() {
        let letter = ["A", "B", "C", "D", "E"][i];
        assert!(row.contains(&format!("data-bar=\"{letter}\"")), "{at}: bar {i} is {letter}");
        assert_eq!(row.starts_with(" is-yours"), letter == yours, "{at}: only {yours}'s bar is marked as the page's own");
        let row = &row[..row.find("</span></div>").unwrap_or(row.len())];
        assert_eq!(row.contains("rn-correct"), Some(letter) == correct, "{at}: the ✓ on bar {letter}");
    }
    assert!(!html.contains("data-count"), "{at}: the page counts nothing itself");
    assert!(!html.contains('✗'), "{at}: no mark against anyone (AC-94)");
}

/// AC-79: nothing on the wall takes input.
pub fn assert_no_control(html: &str, at: &str) {
    for control in ["<button", "<input", "<select", "<textarea", "<a ", "<form", "onclick", "onkeydown", "contenteditable"] {
        assert!(!html.contains(control), "{at}: the wall has an interactive control ({control}) (AC-79)");
    }
}

// --------------------------------------------------------------------------
// node: the served pages' own scripts.
// --------------------------------------------------------------------------

/// The `<script src>` of a served page, in order.
pub fn script_srcs(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(at) = rest.find("<script src=\"") {
        rest = &rest[at + "<script src=\"".len()..];
        out.push(rest[..rest.find('"').unwrap()].to_string());
    }
    out
}

/// Every `src=` and `href=` a page loads.
pub fn page_refs(html: &str) -> Vec<String> {
    let mut out = Vec::new();
    for attr in ["src=\"", "href=\""] {
        let mut rest = html;
        while let Some(at) = rest.find(attr) {
            rest = &rest[at + attr.len()..];
            out.push(rest[..rest.find('"').unwrap()].to_string());
        }
    }
    out
}

fn node() -> Command {
    let mut c = Command::new("node");
    c.arg(repo().join("room/tests/canary_scan/page.js"))
        .env("POPQUIZ_ADMIN_TOKEN", &plants().admin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    c
}

/// Every wall frame as the served `wall.js` renders it (`PQ.Wall.html`).
/// `scripts` are the page's scripts as the room served them.
pub fn render_walls(scripts: &[(String, String)], frames: &[Value]) -> Vec<String> {
    let mut child = node()
        .arg("render")
        .spawn()
        .expect("`node` renders the served wall; `just test` needs it for test-web too");
    let input = json!({ "scripts": scripts.iter().map(|(n, t)| json!({"name": n, "text": t})).collect::<Vec<_>>(), "frames": frames });
    child.stdin.take().unwrap().write_all(input.to_string().as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "the wall renderer failed");
    serde_json::from_slice::<Vec<String>>(&out.stdout).expect("one HTML string per frame")
}

/// The real buzzer page, mounted under node against the running room.
pub struct Page {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Page {
    pub fn mount(base: &str, search: &str, scripts: &[(String, String)]) -> Page {
        let mut child = node()
            .arg("buzzer")
            .spawn()
            .expect("`node` runs the served buzzer page; `just test` needs it for test-web too");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut page = Page { child, stdin, stdout };
        let scripts: Vec<Value> = scripts.iter().map(|(n, t)| json!({"name": n, "text": t})).collect();
        page.send(json!({"cmd": "mount", "base": base, "search": search, "scripts": scripts}));
        page
    }

    /// One command, one JSON reply. Blocking: call it from `spawn_blocking`
    /// or while the server runs on other worker threads.
    pub fn send(&mut self, cmd: Value) -> Value {
        writeln!(self.stdin, "{cmd}").unwrap();
        self.stdin.flush().unwrap();
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        let v: Value = serde_json::from_str(&line).unwrap_or_else(|e| panic!("page.js said {line:?}: {e}"));
        assert!(v.get("error").is_none(), "page.js: {v}");
        v
    }
}

impl Drop for Page {
    fn drop(&mut self) {
        let _ = writeln!(self.stdin, "{}", json!({"cmd": "quit"}));
        let _ = self.child.wait();
    }
}

// --------------------------------------------------------------------------
// The walk: one room, every phase in order, a scan at every revision.
// --------------------------------------------------------------------------

/// What the walk leaves behind for a suite's own positive controls.
pub struct Evidence {
    pub seen: Seen,
    /// The wall's projection on entering `reveal`.
    pub reveal_wall: Value,
    pub reveal_host: Value,
    pub reveal_buzzer: Value,
    /// Every wall projection while stepping `reveal` back to step 0.
    pub reveal_walk: Vec<Value>,
    /// Every `work` wall projection, and the refused step past the bound.
    pub work_walk: Vec<Value>,
    pub work_refused: u16,
    /// The room made by *Run it again*, if one was asked for.
    pub next_room: Option<Value>,
    /// Every frame a buzzer socket received, in order, per socket.
    pub buzzer_frames: Vec<(String, Vec<Value>)>,
    /// The real page's whole log and its snapshots, by phase.
    pub page_log: Vec<Value>,
    pub page_snaps: Vec<Value>,
    pub page_token: String,
    /// The server's request log from the moment the room closed.
    pub server_after_close: Vec<Logged>,
    pub stops: usize,
}

pub struct Options {
    /// A question for *Run it again* after `released`.
    pub run_again_with: Option<&'static str>,
    /// In `closed`: drop and re-attach buzzer B, and replace buzzer C's
    /// socket with a second one, then check each attach frame (test-full).
    pub reconnect: bool,
}

pub struct Walk<'s> {
    pub server: &'s Server,
    pub id: String,
    pub code: String,
    pub host: String,
    pub probe: String,
    pub watchers: Vec<Watcher>,
    /// The three attached buzzers' tokens, answering A, B, C.
    pub tokens: [String; 3],
    pub seen: Seen,
    pub wall_frames: Vec<Value>,
    pub fonts: Vec<(String, Vec<u8>)>,
    pub stops: usize,
    /// PQ-37: the bars the wall showed at `split`, which the phone must still
    /// carry in `work` (where the wall shows the trace) and the wall shows
    /// again at `reveal`.
    pub split_bars: Option<Value>,
    /// T-12: `/last` as it was at this walk's first stop, before this room
    /// could release. Only a release rebuilds it (§13).
    pub last_before: Option<String>,
}

/// The `question_id` in `/last`'s snapshot slot; `None` for the no-release page.
fn last_question(page: &str) -> Option<String> {
    let open = "<script type=\"application/json\" id=\"take-home\">";
    let start = page.find(open).expect("/last has its snapshot slot") + open.len();
    let end = start + page[start..].find("</script>").expect("the slot closes");
    let v: Value = serde_json::from_str(&page[start..end]).expect("the slot is JSON");
    v.get("question_id").and_then(Value::as_str).map(str::to_string)
}

fn js(v: &Value) -> String {
    v.to_string()
}

impl<'s> Walk<'s> {
    pub async fn create(server: &'s Server, question: &str) -> Walk<'s> {
        let created = server
            .request(Method::POST, "/rooms", Some(ORGANIZER), Some(json!({ "question_id": question })))
            .await;
        assert_eq!(created.status, 201, "{}", created.text());
        let mut seen = Seen::default();
        check(&mut seen, Surface::Host, Phase::Idle, &created.everything(), Some(&created.json()), "POST /rooms");
        Walk::adopt(server, created.json(), seen).await
    }

    /// Take over a room that already exists (the one *Run it again* made).
    pub async fn adopt(server: &'s Server, c: Value, seen: Seen) -> Walk<'s> {
        let id = c["id"].as_str().unwrap().to_string();
        let code = c["code"].as_str().unwrap().to_string();
        let host = c["host_session"].as_str().unwrap().to_string();

        let mut walk = Walk {
            server,
            id,
            code,
            host,
            probe: String::new(),
            watchers: Vec::new(),
            tokens: Default::default(),
            seen,
            wall_frames: Vec::new(),
            fonts: Vec::new(),
            stops: 0,
            split_bars: None,
            last_before: None,
        };
        walk.probe = walk.join("the probe session").await;
        let mut tokens: Vec<String> = Vec::new();
        for label in ["buzzer A", "buzzer B", "buzzer C"] {
            tokens.push(walk.join(label).await);
        }
        walk.tokens = tokens.try_into().unwrap();
        let wall = Watcher::attach(server, &walk.id, "wall", None, "wall socket").await;
        let host_socket = Watcher::attach(server, &walk.id, "host", Some(&walk.host.clone()), "host socket").await;
        walk.watchers.push(wall);
        walk.watchers.push(host_socket);
        for (i, label) in ["buzzer A socket", "buzzer B socket", "buzzer C socket"].iter().enumerate() {
            let t = walk.tokens[i].clone();
            walk.watchers.push(Watcher::attach(server, &walk.id, "buzzer", Some(&t), label).await);
        }
        for font in served_files("shared/fonts") {
            let r = server.get(&format!("/shared/fonts/{font}")).await;
            assert_eq!(r.status, 200, "font {font}");
            check(&mut walk.seen, Surface::Page, Phase::Idle, &r.everything(), None, &format!("/shared/fonts/{font}"));
            walk.fonts.push((font, r.body));
        }
        walk
    }

    /// `POST /join` for a fresh session; its response is scanned.
    pub async fn join(&mut self, label: &str) -> String {
        let r = self
            .server
            .request(Method::POST, "/join", None, Some(json!({ "code": self.code })))
            .await;
        let v = r.json();
        if r.status != 201 {
            // `released` has ended: the refusal is scanned like any response.
            let phase = phase_of(self.projection("wall").await["phase"].as_str().unwrap());
            assert_eq!((phase, r.status), (Phase::Released, 409), "{label}: {}", r.text());
            check(&mut self.seen, Surface::BuzzerOther, phase, &r.everything(), Some(&v), &format!("POST /join ({label}), refused"));
            return String::new();
        }
        let phase = phase_of(v["buzzer"]["phase"].as_str().unwrap());
        check(&mut self.seen, Surface::Buzzer, phase, &r.everything(), Some(&v), &format!("POST /join ({label})"));
        v["token"].as_str().unwrap().to_string()
    }

    pub async fn act(&mut self, slug: &str) -> Resp {
        let r = self
            .server
            .request(Method::POST, &format!("/rooms/{}/{slug}", self.id), Some(&self.host), None)
            .await;
        if r.status == 200 {
            let v = r.json();
            let phase = phase_of(v["phase"].as_str().unwrap());
            check(&mut self.seen, Surface::Host, phase, &r.everything(), Some(&v), &format!("POST {slug}"));
        } else {
            check(&mut self.seen, Surface::Page, Phase::Idle, &r.everything(), None, &format!("POST {slug} (refused)"));
        }
        r
    }

    pub async fn answer(&mut self, token: &str, letter: &str, at: &str) -> Resp {
        let r = self
            .server
            .request(Method::PUT, &format!("/rooms/{}/answer", self.id), Some(token), Some(json!({ "letter": letter })))
            .await;
        let phase = phase_of(self.projection("wall").await["phase"].as_str().unwrap());
        check(&mut self.seen, Surface::BuzzerOther, phase, &r.everything(), Some(&r.json()), at);
        r
    }

    pub async fn projection(&self, viewer: &str) -> Value {
        let bearer = (viewer == "host").then_some(self.host.as_str());
        let r = self
            .server
            .request(Method::GET, &format!("/rooms/{}/{viewer}", self.id), bearer, None)
            .await;
        assert_eq!(r.status, 200, "GET {viewer}");
        r.json()
    }

    /// One stop: every probe, projection, frame and page, scanned.
    pub async fn stop(&mut self, label: &str) -> [Value; 3] {
        self.stops += 1;
        let at = |what: &str| format!("{label}: {what}");

        // The probes a participant or the wall can make, each response scanned.
        self.join(&at("a new join")).await;
        let probe = self.probe.clone();
        self.answer(&probe, "A", &at("PUT answer (probe)")).await;
        let fit = self
            .server
            .request(Method::PUT, &format!("/rooms/{}/fit", self.id), None, Some(json!({"fit": "fits"})))
            .await;
        assert_eq!(fit.status, 204, "{}", at("PUT fit"));

        // The three projections, and the denials.
        let wall = self.projection("wall").await;
        let buzzer = self.projection("buzzer").await;
        let host = self.projection("host").await;
        let phase = phase_of(wall["phase"].as_str().unwrap());
        assert!(wall["phase"] == buzzer["phase"] && buzzer["phase"] == host["phase"], "{}", at("the three projections disagree"));
        check(&mut self.seen, Surface::Wall, phase, &js(&wall), Some(&wall), &at("GET wall"));
        check(&mut self.seen, Surface::Buzzer, phase, &js(&buzzer), Some(&buzzer), &at("GET buzzer"));
        check(&mut self.seen, Surface::Host, phase, &js(&host), Some(&host), &at("GET host"));
        assert_eq!(buzzer.get("yours"), None, "{}", at("the public buzzer carries a session's answer"));
        self.assert_one_split(phase, &wall, &buzzer, &at("the phone and the wall"));
        for bearer in [None, Some("not-the-session"), Some(self.probe.as_str())] {
            let r = self
                .server
                .request(Method::GET, &format!("/rooms/{}/host", self.id), bearer, None)
                .await;
            assert_eq!((r.status, r.body.len()), (401, 0), "{}", at("a host denial says nothing"));
        }
        let r = self.server.get("/rooms/no-such-room/wall").await;
        check(&mut self.seen, Surface::Page, phase, &r.everything(), None, &at("unknown room"));
        // T-25: the admin prefix at every stop.
        let question = self.server.state.with_room(&self.id, |r| r.question_id().to_string()).unwrap();
        self.server.admin_probe(&question, &at("the admin prefix")).await;

        // Every frame every socket received since the last stop.
        for w in &mut self.watchers {
            if w.closed.is_some() {
                continue;
            }
            let target = match w.viewer {
                "wall" => &wall,
                "host" => &host,
                _ => &buzzer,
            };
            w.catch_up(target).await;
            for f in w.take_fresh() {
                let fp = phase_of(f["phase"].as_str().unwrap());
                let surface = match w.viewer {
                    "wall" => Surface::Wall,
                    "host" => Surface::Host,
                    _ => Surface::Buzzer,
                };
                check(&mut self.seen, surface, fp, &js(&f), Some(&as_payload(&f)), &format!("{label}: {} frame", w.label));
                if w.viewer == "wall" {
                    self.wall_frames.push(f);
                }
            }
        }

        // Every page and every file the router serves (fonts: once, then equal).
        let mut pages: Vec<String> = vec![
            format!("/wall/{}", self.id),
            "/host".into(),
            format!("/host/{}", self.id),
            "/join".into(),
            format!("/join?code={}", self.code),
            "/wall/wall.js".into(),
            "/wall/wall.css".into(),
            "/wall/qr.js".into(),
            "/host/host.js".into(),
            "/host/host.css".into(),
            "/join/buzzer.js".into(),
            "/join/buzzer.css".into(),
            "/home/home.js".into(),
            "/home/home.css".into(),
        ];
        pages.extend(served_files("shared").into_iter().map(|f| format!("/shared/{f}")));
        for page in &pages {
            let r = self.server.get(page).await;
            assert_eq!(r.status, 200, "{}", at(page));
            check(&mut self.seen, Surface::Page, phase, &r.everything(), None, &at(page));
            if page.starts_with("/wall/") && !page.contains('.') {
                assert_no_control(&r.text(), &at("the served wall page"));
                for src in page_refs(&r.text()) {
                    assert!(pages.contains(&src) || src.starts_with("/shared/"), "{}", at(&format!("the wall loads {src}, which is not scanned")));
                }
            }
        }
        // T-12: /last is the one page whose bytes change — at a release.
        let last = self.server.get("/last").await;
        assert_eq!(last.status, 200, "{}", at("/last"));
        let body = last.text();
        // D-26: `/last/nyc` is `/last`, and a club that has released nothing
        // (here `la`) carries nothing of this room, at any stop.
        let by_club = self.server.get("/last/nyc").await;
        assert_eq!(by_club.text(), body, "{}", at("/last/nyc is not /last"));
        let other = self.server.get("/last/la").await;
        assert_eq!(last_question(&other.text()), None, "{}", at("/last/la carries another club's release"));
        let before = self.last_before.get_or_insert_with(|| body.clone()).clone();
        if phase == Phase::Released {
            // This room's release rebuilt it, as a question it did not hold before.
            assert_ne!(last_question(&body), None, "{}", at("/last after release"));
            assert_ne!(last_question(&body), last_question(&before), "{}", at("/last was not rebuilt at release"));
        } else {
            assert_eq!(body, before, "{}", at("/last changed before this room's release"));
        }
        // Before release: the no-release page carries no plant (TakeHome);
        // an earlier room's release is held unchanged by the line above.
        let surface = if phase != Phase::Released && last_question(&before).is_some() {
            Surface::TakeHomeEarlier
        } else {
            Surface::TakeHome
        };
        check(&mut self.seen, surface, phase, &last.everything(), None, &at("/last"));
        let short = self.server.get(&format!("/{}", self.code)).await;
        assert!((300..400).contains(&short.status), "{}", at("the short link redirects"));
        check(&mut self.seen, Surface::Page, phase, &short.everything(), None, &at("GET /{code}"));
        for (font, bytes) in &self.fonts {
            let r = self.server.get(&format!("/shared/fonts/{font}")).await;
            assert!(r.body == *bytes, "{}", at(&format!("font {font} changed between stops")));
        }
        [wall, buzzer, host]
    }

    /// PQ-37, the §4.5 pattern: the phone and the wall can never show
    /// different splits. At `split` and `reveal` the phone's `split` is the
    /// wall's `split.bars`, in the same revision; in `work` it is the bars the
    /// wall showed at `split` (the totals froze at `closed`, §4.4). In every
    /// other phase the phone carries none.
    pub fn assert_one_split(&mut self, phase: Phase, wall: &Value, buzzer: &Value, at: &str) {
        match phase {
            Phase::Split | Phase::Reveal => {
                let bars = &wall["split"]["bars"];
                assert!(bars.is_array(), "{at}: the wall shows no bars in {}", phase_name(phase));
                assert_eq!(&buzzer["split"], bars, "{at}: the phone's split is not the wall's (PQ-37)");
                if let Some(seen) = &self.split_bars {
                    assert_eq!(bars, seen, "{at}: the wall's bars moved after the split");
                }
                self.split_bars = Some(bars.clone());
            }
            Phase::Work => {
                let seen = self.split_bars.as_ref().expect("the walk passed the split before work");
                assert_eq!(&buzzer["split"], seen, "{at}: the phone's split in work is not the wall's (PQ-37)");
            }
            _ => assert_eq!(buzzer.get("split"), None, "{at}: the phone carries a split in {} (PQ-37)", phase_name(phase)),
        }
    }

    /// A phase transition, then a stop.
    pub async fn transition(&mut self, slug: &str) -> [Value; 3] {
        let r = self.act(slug).await;
        assert_eq!(r.status, 200, "{slug}: {}", r.text());
        self.stop(slug).await
    }

    /// The page's scripts, as the room served them for `path`.
    pub async fn scripts(&self, path: &str) -> Vec<(String, String)> {
        let html = self.server.get(path).await.text();
        let mut out = Vec::new();
        for src in script_srcs(&html) {
            let r = self.server.get(&src).await;
            assert_eq!(r.status, 200, "{path} loads {src}");
            out.push((src, r.text()));
        }
        out
    }

    pub fn watcher(&mut self, label: &str) -> &mut Watcher {
        self.watchers.iter_mut().find(|w| w.label == label).unwrap()
    }
}

/// Scan one snapshot of the real buzzer page: its rendered HTML, the frames
/// it received, and everything it sent.
pub fn check_page(seen: &mut Seen, snap: &Value, at: &str) {
    let phase = phase_of(snap["phase"].as_str().unwrap_or("idle"));
    check(seen, Surface::RenderedBuzzer, phase, snap["html"].as_str().unwrap(), None, &format!("{at}: the page"));
    for entry in snap["log"].as_array().unwrap() {
        match entry["kind"].as_str().unwrap() {
            "ws-frame" => {
                let f: Value = serde_json::from_str(entry["data"].as_str().unwrap()).unwrap();
                let fp = phase_of(f["phase"].as_str().unwrap());
                check(seen, Surface::Buzzer, fp, &js(&f), Some(&as_payload(&f)), &format!("{at}: a frame the page received"));
            }
            _ => check(seen, Surface::PageTraffic, phase, &js(entry), None, &format!("{at}: what the page sent")),
        }
    }
}

/// Where a walk starts: a new room on a question, or a room that exists.
pub enum Start<'a> {
    Question(&'a str),
    Room(Value),
}

/// One room through every phase, the real buzzer page riding along.
pub async fn walk_room(server: &Server, start: Start<'_>, options: Options) -> Evidence {
    let p = plants();
    // T-21: AC-55's request log is the room's log too; every line it writes
    // during the walk is scanned below.
    room::requestlog::start_capture();
    let mut w = match start {
        Start::Question(q) => Walk::create(server, q).await,
        Start::Room(created) => Walk::adopt(server, created, Seen::default()).await,
    };
    let search = format!("?code={}", w.code);
    let scripts = w.scripts(&format!("/join{search}")).await;
    let mut page = Page::mount(&server.base(), &search, &scripts);
    let mut snaps: Vec<Value> = Vec::new();
    let mut page_do = |page: &mut Page, cmd: Value| -> Value {
        let snap = tokio::task::block_in_place(|| page.send(cmd));
        snaps.push(snap.clone());
        snap
    };

    // idle
    let snap = page_do(&mut page, json!({"cmd": "wait", "phase": "idle"}));
    let page_token = snap["token"].as_str().unwrap().to_string();
    w.stop("idle").await;

    // live: the hint first, and nothing moves; then everyone answers.
    let [live_wall, _, live_host] = w.transition(HostAction::PutOnScreen.slug()).await;
    page_do(&mut page, json!({"cmd": "wait", "phase": "live"}));
    // Settle: the stop's own probes (a join, an answer) are still being
    // broadcast; each settled frame is scanned like any other.
    for label in ["wall socket", "host socket"] {
        let late = w.watcher(label).quiet_for(Duration::from_millis(300)).await;
        let surface = if label == "wall socket" { Surface::Wall } else { Surface::Host };
        for f in late {
            check(&mut w.seen, surface, Phase::Live, &js(&f), Some(&as_payload(&f)), "live: a settled frame");
            if surface == Surface::Wall {
                w.wall_frames.push(f);
            }
        }
        w.watcher(label).take_fresh();
    }
    let hint = page_do(&mut page, json!({"cmd": "hint"}));
    let sent: Vec<&Value> = hint["log"].as_array().unwrap().iter().filter(|e| e["kind"] != "ws-frame").collect();
    assert!(sent.is_empty(), "AC-48: taking the hint made a request or sent a message: {sent:?}");
    assert!(hint["html"].as_str().unwrap().contains(&p.hint), "the page shows the hint it was sent (AC-48)");
    for label in ["wall socket", "host socket"] {
        let extra = w.watcher(label).quiet_for(Duration::from_millis(300)).await;
        assert!(extra.is_empty(), "AC-48: the {label} moved when a hint was taken: {extra:?}");
    }
    assert_eq!(w.projection("wall").await, live_wall, "AC-48: the wall changed when a hint was taken");
    assert_eq!(w.projection("host").await, live_host, "AC-48: the host changed when a hint was taken");
    for (i, letter) in ["A", "B", "C"].iter().enumerate() {
        let t = w.tokens[i].clone();
        let r = w.answer(&t, letter, &format!("live: buzzer {letter} answers")).await;
        assert_eq!(r.json()["saved"], json!(letter));
    }
    let tapped = page_do(&mut page, json!({"cmd": "tap", "letter": "B"}));
    assert_eq!(tapped["saved"], json!("B"), "the page saved its answer while live");
    w.stop("live, answered").await;

    // closed: from here on the page must send nothing that carries an answer.
    let close_mark = server.log_len();
    w.transition(HostAction::CloseAnswers.slug()).await;
    page_do(&mut page, json!({"cmd": "wait", "phase": "closed"}));
    page_do(&mut page, json!({"cmd": "tap", "letter": "C"}));
    page_do(&mut page, json!({"cmd": "drop"}));
    if options.reconnect {
        reconnect(&mut w).await;
    }
    w.stop("closed, after the page re-attached").await;

    // split
    w.transition(HostAction::ShowSplit.slug()).await;
    let split = page_do(&mut page, json!({"cmd": "wait", "phase": "split"}));
    // PQ-37 (HC-0): the real page draws the room's five bars and marks its own
    // letter, B, which it knows and never sends. What AC-58 guards — nothing
    // carrying the answer leaves the phone after close — is the request-log
    // scan below, and it is unchanged. The phone still computes no count.
    assert_page_bars(split["html"].as_str().unwrap(), "B", None, "split");

    // work: every step to the bound, and one more is refused.
    let mut work_walk = Vec::new();
    let [wall, ..] = w.transition(HostAction::WalkIt.slug()).await;
    work_walk.push(wall);
    page_do(&mut page, json!({"cmd": "wait", "phase": "work"}));
    let work_refused = loop {
        let r = w.act(Step::Forward.slug()).await;
        if r.status != 200 {
            break r.status;
        }
        let [wall, ..] = w.stop("work, a step").await;
        work_walk.push(wall);
    };

    // reveal: enters at the final step; step back through all of it.
    let [reveal_wall, reveal_buzzer, reveal_host] = w.transition(HostAction::Reveal.slug()).await;
    let revealed = page_do(&mut page, json!({"cmd": "wait", "phase": "reveal"}));
    let correct = reveal_wall["reveal"]["correct"].as_str().unwrap().to_string();
    assert_page_bars(revealed["html"].as_str().unwrap(), "B", Some(&correct), "reveal");
    let mut reveal_walk = Vec::new();
    while w.act(Step::Back.slug()).await.status == 200 {
        let [wall, ..] = w.stop("reveal, a step back").await;
        reveal_walk.push(wall);
    }

    // released
    w.transition(HostAction::ReleaseRoom.slug()).await;
    // T-25: the ledger the pipeline syncs names this room, and the question
    // it ran is refused if pushed again (G-10).
    let question = server.state.with_room(&w.id, |r| r.question_id().to_string()).unwrap();
    let used = server.admin_probe(&question, "released: the admin prefix").await.json();
    assert!(
        used.as_array().unwrap().iter().any(|e| e["question_id"] == question.as_str() && e["used"]["room_id"] == w.id.as_str()),
        "GET /admin/used names the released room: {used}"
    );
    let again = server
        .request(Method::PUT, &format!("/admin/questions/{question}"), Some(&plants().admin), Some(canary_question_named(&question)))
        .await;
    assert_eq!(again.status, 409, "a used question is refused: {}", again.text());
    check_admin(&again.everything(), "released: PUT a used question");
    page_do(&mut page, json!({"cmd": "wait", "phase": "released"}));
    page_do(&mut page, json!({"cmd": "snap"}));
    drop(page);

    let next_room = match options.run_again_with {
        Some(q) => {
            let r = server
                .request(Method::POST, &format!("/rooms/{}/run-it-again", w.id), Some(ORGANIZER), Some(json!({"question_id": q})))
                .await;
            assert_eq!(r.status, 201, "run it again: {}", r.text());
            check(&mut w.seen, Surface::Host, Phase::Idle, &r.everything(), Some(&r.json()), "POST run-it-again");
            Some(r.json())
        }
        None => None,
    };

    // The page: every snapshot, every frame it got and everything it sent.
    for (i, snap) in snaps.iter().enumerate() {
        check_page(&mut w.seen, snap, &format!("page snapshot {i}"));
    }
    let page_log: Vec<Value> = snaps.iter().flat_map(|s| s["log"].as_array().unwrap().clone()).collect();

    // The wall's frames as the served wall.js renders them.
    let wall_scripts = w.scripts(&format!("/wall/{}", w.id)).await;
    let rendered = render_walls(&wall_scripts, &w.wall_frames);
    assert_eq!(rendered.len(), w.wall_frames.len());
    for (f, html) in w.wall_frames.iter().zip(&rendered) {
        let fp = phase_of(f["phase"].as_str().unwrap());
        check(&mut w.seen, Surface::RenderedWall, fp, html, None, &format!("the rendered wall, revision {}", f["revision"]));
        assert_no_control(html, &format!("the rendered wall in {}", phase_name(fp)));
    }

    let buzzer_frames = w
        .watchers
        .iter()
        .filter(|x| x.viewer == "buzzer")
        .map(|x| (x.label.clone(), x.all.clone()))
        .collect();
    // T-21: no line of the request log carries a plant (the room id is its
    // one identifier; tokens, sessions, codes and answers are never fields).
    for line in room::requestlog::captured_all() {
        for (name, plant) in p.all() {
            assert!(!line.contains(plant), "the request log carries the {name} plant: {line}");
        }
    }
    Evidence {
        seen: w.seen,
        reveal_wall,
        reveal_host,
        reveal_buzzer,
        reveal_walk,
        work_walk,
        work_refused,
        next_room,
        buzzer_frames,
        page_log,
        page_snaps: snaps,
        page_token,
        server_after_close: server.log_since(close_mark),
        stops: w.stops,
    }
}

/// In `closed`: buzzer B's socket drops and a new one attaches with the same
/// token; buzzer C attaches a second socket, which replaces the first. Each
/// attach frame carries that session's own saved letter and nothing of any
/// other session's (AC-37, AC-57, AC-58).
pub async fn reconnect(w: &mut Walk<'_>) {
    let b = w.tokens[1].clone();
    let c = w.tokens[2].clone();
    {
        let old = w.watcher("buzzer B socket");
        old.socket.close(None).await.unwrap();
        old.closed = Some(None);
    }
    let fresh = Watcher::attach(w.server, &w.id, "buzzer", Some(&b), "buzzer B socket, re-attached").await;
    let attach = fresh.all[0].clone();
    assert_eq!(attach["session"], json!({"saved": "B"}), "the re-attach frame carries B's own answer");
    assert_eq!(attach["phase"], "closed");
    assert_eq!(attach.get("yours"), None);
    w.watchers.push(fresh);

    let second = Watcher::attach(w.server, &w.id, "buzzer", Some(&c), "buzzer C socket, second").await;
    assert_eq!(second.all[0]["session"], json!({"saved": "C"}), "the second socket's attach frame carries C's own answer");
    let first = w.watcher("buzzer C socket");
    let extra = first.quiet_for(Duration::from_millis(500)).await;
    assert!(extra.iter().all(|f| f.get("session").is_none()), "a broadcast carries a session");
    assert_eq!(first.closed, Some(Some(room::ws::close::REPLACED)), "the first socket is replaced");
    w.watchers.push(second);

    // Every attach frame names one session, its own; no broadcast names any.
    for x in w.watchers.iter().filter(|x| x.viewer == "buzzer") {
        for (i, f) in x.all.iter().enumerate() {
            if i > 0 {
                assert!(f.get("session").is_none(), "{}: a broadcast frame carries a session", x.label);
            }
            assert!(f.get("yours").is_none(), "{}: a frame carries `yours`", x.label);
        }
    }
}

/// AC-58 / G-4 over what the walk recorded: after the page's `closed` frame
/// arrived, nothing it sent carries an answer, and nothing bearing its token
/// reached the server.
pub fn assert_nothing_personal_after_close(e: &Evidence) {
    let closed_at = e
        .page_log
        .iter()
        .position(|x| {
            x["kind"] == "ws-frame"
                && serde_json::from_str::<Value>(x["data"].as_str().unwrap()).unwrap()["phase"] == "closed"
        })
        .expect("the page received the closed frame");
    let after = &e.page_log[closed_at..];
    assert!(after.iter().any(|x| x["kind"] == "ws-send"), "the page re-attached after close (the scan saw its traffic)");
    for x in after {
        match x["kind"].as_str().unwrap() {
            "fetch" => {
                let url = x["url"].as_str().unwrap();
                assert!(!url.ends_with("/answer"), "AC-58: after close the page called {url}");
                let body = x["body"].as_str().unwrap_or("");
                assert!(!body.contains("letter"), "AC-58: after close the page sent {body}");
            }
            "ws-send" => {
                let m: Value = serde_json::from_str(x["data"].as_str().unwrap()).unwrap();
                assert_eq!(m, json!({"t": "attach", "token": e.page_token}), "AC-58: after close the page sent {m}");
            }
            _ => {}
        }
    }
    for logged in &e.server_after_close {
        assert!(
            logged.bearer.as_deref() != Some(e.page_token.as_str()),
            "AC-58: after close the server received {} {} with the page's session",
            logged.method,
            logged.path
        );
        if logged.path.ends_with("/answer") {
            // Only the canary's own probe writes after close, and it is refused.
            assert_ne!(logged.bearer.as_deref(), Some(e.page_token.as_str()));
        }
    }
}

// --------------------------------------------------------------------------
// Positive controls: a scanner that saw nothing cannot pass.
// --------------------------------------------------------------------------

/// The positive controls both variants share: the plants reached every
/// surface that may show them, in the phase that may show them.
pub fn assert_the_plants_arrived(e: &Evidence) {
    let p = plants();
    let s = &e.seen;
    // live: the source on the wall, the hint in the buzzer's view and on the page.
    assert!(s.has(Surface::Wall, Phase::Live, &p.source), "the live wall shows the source");
    assert!(s.has(Surface::RenderedWall, Phase::Live, &p.source), "the rendered live wall shows the source");
    assert!(s.has(Surface::Buzzer, Phase::Live, &p.hint), "the live buzzer carries the hint (AC-48)");
    assert!(s.has(Surface::RenderedBuzzer, Phase::Live, &p.hint), "the page shows the hint once asked");
    // The correct option's text is public from live: on the wall, as option E.
    assert!(s.has(Surface::Wall, Phase::Live, &p.correct));
    // work: the walk's notes reach the wall and the host.
    assert!(s.has(Surface::Wall, Phase::Work, &p.notes[0]) && s.has(Surface::Host, Phase::Work, &p.notes[0]));
    assert!(s.has(Surface::RenderedWall, Phase::Work, &p.trace_value), "the rendered work wall shows a trace value");
    // reveal: the resolving step, the stdout rows, the explanation (host only).
    assert!(s.has(Surface::Wall, Phase::Reveal, &p.resolving), "reveal enters at the resolving step");
    assert!(s.has(Surface::RenderedWall, Phase::Reveal, &p.resolving));
    assert!(s.has(Surface::Wall, Phase::Reveal, &p.middle_stdout), "reveal may step back through every stdout row");
    for note in &p.notes {
        assert!(s.has(Surface::Wall, Phase::Reveal, note), "reveal steps back through the whole trace");
    }
    assert!(s.has(Surface::Host, Phase::Reveal, &p.what) && s.has(Surface::Host, Phase::Reveal, &p.takeaway));
    // T-12: once released, /last is the canary question — every beat (§13).
    assert!(s.has(Surface::TakeHome, Phase::Released, &p.what) && s.has(Surface::TakeHome, Phase::Released, &p.takeaway));
    // Every incorrect option's, in whichever walk this is (its answer is the one it revealed).
    let correct = e.reveal_wall["reveal"]["correct"].as_str().unwrap();
    for (letter, why) in ["A", "B", "C", "D", "E"].iter().zip(&p.why) {
        if *letter != correct {
            assert!(s.has(Surface::TakeHome, Phase::Released, why), "/last carries {letter}'s why_tempting (§13)");
        }
    }
    assert_eq!(e.reveal_wall["reveal"]["mark"], "✓");
    assert_eq!(e.reveal_wall["reveal"]["receipt"]["heading"], copy::RECEIPT_HEADING);
    let m = canary_question()["trace"]["steps"].as_array().unwrap().len();
    assert_eq!(e.reveal_wall["trace"]["at"], m - 1, "reveal enters at M-1");
    // §4.5: the wall and the host name the same option, and the host reads its why.
    let named = e.reveal_wall["reveal"]["middle"]["letter"].as_str().expect("an incorrect option was chosen");
    let i = ["A", "B", "C", "D", "E"].iter().position(|l| *l == named).unwrap();
    assert!(e.reveal_host["read_aloud"]["middle"]["heading"].as_str().unwrap().ends_with(named));
    assert!(s.has(Surface::Host, Phase::Reveal, &p.why[i]), "the host reads the named option's why_tempting");
    // AC-97: work walked 0..=M-2 and one more step was refused.
    assert_eq!(e.work_walk.len(), m - 1, "work shows steps 0..=M-2");
    assert_eq!(e.work_refused, 409, "a step past M-2 in work is refused");
    // AC-58: the page counted its own letter; nothing personal after close.
    assert_nothing_personal_after_close(e);
    // Every stop scanned something.
    assert!(e.stops >= 18, "{} stops", e.stops);
}
