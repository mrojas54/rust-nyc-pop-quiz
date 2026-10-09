//! SPEC §8.3's pipeline channel (T-25): AC-101, AC-61, G-9.
//!
//! - **Refusal:** a missing or wrong token is `401` with an empty body and no
//!   `WWW-Authenticate`, on every method and every path under the prefix, and
//!   a scheduled id is indistinguishable from an unknown one.
//! - **Acceptance:** the right token schedules a question (new, replaced;
//!   refused when used or held by a room) and reads the used ledger in the
//!   pipeline's `Used` shape.
//! - **The route table:** the prefix is served to the admin check alone; no
//!   other module names the token; no participant, wall, host or auth route
//!   accepts it.
//! - **The repository:** no secret-shaped value for the token and no planted
//!   canary value anywhere (`just secret-scan`; inside `just test` too).
//!
//! The canary half (the token in no payload, page or log line) is in
//! `canary.rs`.

mod common;

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use axum::Router;
use common::*;
use room::admin::{AdminToken, VAR};
use room::config::{Config, ConfigError};
use room::lifecycle::ManualClock;
use room::phase::{HostAction, Step};
use room::rooms::{AppState, Urls};
use serde_json::{json, Value};
use tower::ServiceExt;

/// A token for these tests, drawn per process and never written down.
fn token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).unwrap();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct Admin {
    app: Router,
    state: Arc<AppState>,
    token: String,
    clock: Arc<ManualClock>,
}

fn admin() -> Admin {
    let token = token();
    let clock = ManualClock::new(SystemTime::UNIX_EPOCH + Duration::from_secs(1_790_000_000));
    let state = Arc::new(AppState::new(Arc::new(TestAuth), Vec::new(), Urls::default()).with_clock(clock.clone()));
    let app = room::router_with_admin(state.clone(), AdminToken::from_var(Some(token.clone())).unwrap());
    Admin {
        app,
        state,
        token,
        clock,
    }
}

/// One request: status, every header, the raw body.
async fn raw(app: &Router, method: Method, uri: &str, auth: Option<&str>, body: Option<String>) -> (StatusCode, Vec<(String, String)>, Vec<u8>) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(a) = auth {
        req = req.header(header::AUTHORIZATION, a);
    }
    let req = match body {
        Some(b) => req.header(header::CONTENT_TYPE, "application/json").body(Body::from(b)),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let mut headers: Vec<(String, String)> = res
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), String::from_utf8_lossy(v.as_bytes()).into_owned()))
        .collect();
    headers.sort();
    let body = axum::body::to_bytes(res.into_body(), 4 << 20).await.unwrap().to_vec();
    (status, headers, body)
}

async fn push(a: &Admin, id: &str, record: &Value) -> (StatusCode, Value) {
    http(&a.app, Method::PUT, &format!("/admin/questions/{id}"), Some(&a.token), Some(record.clone())).await
}

/// `PUT /admin/clubs/{club}/questions/{id}` (D-26).
async fn push_club(a: &Admin, club: &str, id: &str, record: &Value) -> (StatusCode, Value) {
    http(&a.app, Method::PUT, &format!("/admin/clubs/{club}/questions/{id}"), Some(&a.token), Some(record.clone())).await
}

async fn used(a: &Admin) -> Value {
    let (status, body) = http(&a.app, Method::GET, "/admin/used", Some(&a.token), None).await;
    assert_eq!(status, StatusCode::OK);
    body
}

async fn act(app: &Router, room: &HostedRoom, slug: &str) -> StatusCode {
    http(app, Method::POST, &format!("/rooms/{}/{slug}", room.id), Some(&room.host), None).await.0
}

/// A room on q3, walked from `idle` to `released`.
async fn run_to_release(app: &Router) -> HostedRoom {
    let room = host_room(app).await;
    for a in [HostAction::PutOnScreen, HostAction::CloseAnswers, HostAction::ShowSplit, HostAction::WalkIt] {
        assert_eq!(act(app, &room, a.slug()).await, StatusCode::OK, "{}", a.slug());
    }
    while act(app, &room, Step::Forward.slug()).await == StatusCode::OK {}
    for a in [HostAction::Reveal, HostAction::ReleaseRoom] {
        assert_eq!(act(app, &room, a.slug()).await, StatusCode::OK, "{}", a.slug());
    }
    room
}

// --------------------------------------------------------------------------
// AC-101: refused, saying nothing.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac101_a_missing_or_wrong_token_is_refused_saying_nothing() {
    let a = admin();
    assert_eq!(push(&a, "q3", &q3_json()).await.0, StatusCode::CREATED);
    let wrong_scheme = format!("Basic {}", a.token);
    let longer = format!("Bearer {}x", a.token);
    let prefix = format!("Bearer {}", &a.token[..a.token.len() - 1]);
    let lower = format!("bearer {}", a.token);
    let credentials: Vec<Option<&str>> = vec![
        None,
        Some("Bearer not-the-token"),
        Some("Bearer "),
        Some(""),
        Some(&wrong_scheme),
        Some(&longer),
        Some(&prefix),
        Some(&lower),
        Some("Bearer test-organizer-credential"),
    ];
    let paths = [
        "/admin",
        "/admin/",
        "/admin/used",
        "/admin/used/extra",
        "/admin/questions",
        "/admin/questions/q3",
        "/admin/questions/no-such-question",
        "/admin/x/y/z",
    ];
    let methods = [Method::GET, Method::PUT, Method::POST, Method::DELETE, Method::PATCH, Method::HEAD];
    let mut shapes = std::collections::BTreeSet::new();
    for auth in &credentials {
        for path in paths {
            for method in &methods {
                let body = (*method == Method::PUT).then(|| q3_json().to_string());
                let (status, headers, body) = raw(&a.app, method.clone(), path, *auth, body).await;
                let at = format!("{method} {path} with {:?}", auth.map(|s| s.split(' ').next().unwrap_or("")));
                assert_eq!(status, StatusCode::UNAUTHORIZED, "{at}");
                assert!(body.is_empty(), "{at}: a refusal carries no body");
                assert!(!headers.iter().any(|(k, _)| k == "www-authenticate"), "{at}: no WWW-Authenticate");
                shapes.insert(headers);
            }
        }
    }
    assert_eq!(shapes.len(), 1, "every refusal has the same headers: {shapes:?}");
    // The refusals changed nothing: q3 is still scheduled, once.
    assert_eq!(push(&a, "q3", &q3_json()).await.1["scheduled"], "replaced");
}

// --------------------------------------------------------------------------
// AC-101: the right token schedules and reads.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac101_the_right_token_schedules_and_reads_the_used_ledger() {
    let a = admin();
    // Nothing scheduled: creation is refused.
    let (status, _) = http(&a.app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": "q3"}))).await;
    assert_eq!(status, StatusCode::CONFLICT);

    let (status, body) = push(&a, "q3", &q3_json()).await;
    assert_eq!((status, body.clone()), (StatusCode::CREATED, json!({"id": "q3", "scheduled": "new"})));
    let (status, body) = push(&a, "q3", &q3_json()).await;
    assert_eq!((status, body), (StatusCode::OK, json!({"id": "q3", "scheduled": "replaced"})));
    assert_eq!(used(&a).await, json!([]));

    let room = run_to_release(&a.app).await;
    let ledger = used(&a).await;
    let entries = ledger.as_array().unwrap();
    assert_eq!(entries.len(), 1, "{ledger}");
    let entry = &entries[0];
    let keys = |v: &Value| v.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    assert_eq!(keys(entry), ["club", "question_id", "used"]);
    assert_eq!(entry["club"], "nyc");
    // bank.py's `Used`: meetup_date, room_id, released_at, fit.
    assert_eq!(keys(&entry["used"]), ["fit", "meetup_date", "released_at", "room_id"]);
    assert_eq!(entry["question_id"], "q3");
    assert_eq!(entry["used"]["room_id"], room.id.as_str());
    assert_eq!(entry["used"]["fit"], Value::Null, "no wall reported a fit, so none is written (G-2)");

    // G-10, per club (D-26): the released room's record still names q3, so it
    // cannot be replaced yet ...
    let (status, body) = push(&a, "q3", &q3_json()).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(body["reason"].as_str().unwrap().contains("A room is running q3"), "{body}");
    // ... and the club that ran it may not run it again, while another may.
    let body = |club: &str| json!({"question_id": "q3", "club": club});
    // LA has not been pushed anything yet: nothing is scheduled for it, though
    // NYC's q3 is. Its record is its own (arranged for its own date).
    let (status, none) = http(&a.app, Method::POST, "/rooms", Some(ORGANIZER), Some(body("la"))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{none}");
    assert!(none["reason"].as_str().unwrap().contains("No question is scheduled"), "{none}");
    let (status, pushed) = push_club(&a, "la", "q3", &q3_json()).await;
    assert_eq!((status, pushed), (StatusCode::CREATED, json!({"id": "q3", "scheduled": "new"})));
    let (status, refused) = http(&a.app, Method::POST, "/rooms", Some(ORGANIZER), Some(body("nyc"))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");
    assert!(refused["reason"].as_str().unwrap().contains("already been run"), "{refused}");
    let (status, other) = http(&a.app, Method::POST, "/rooms", Some(ORGANIZER), Some(body("la"))).await;
    assert_eq!(status, StatusCode::CREATED, "{other}");
}

#[tokio::test]
async fn ac101_a_question_a_room_holds_is_not_replaced() {
    let a = admin();
    push(&a, "q3", &q3_json()).await;
    let room = host_room(&a.app).await;
    let (status, body) = push(&a, "q3", &q3_json()).await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(body["reason"].as_str().unwrap().contains("A room is running q3"), "{body}");
    // The room goes quiet in `idle`, is swept, and holds nothing: q3 was
    // never run, so it can be pushed again.
    a.clock.advance(room::lifecycle::IDLE_QUIET + Duration::from_secs(1));
    assert_eq!(a.state.sweep(a.state.now()), vec![room.id.clone()]);
    assert_eq!(push(&a, "q3", &q3_json()).await, (StatusCode::OK, json!({"id": "q3", "scheduled": "replaced"})));
}

#[tokio::test]
async fn ac101_the_path_and_the_record_name_the_same_question() {
    let a = admin();
    let (status, body) = push(&a, "q4", &q3_json()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body.as_object().unwrap().keys().collect::<Vec<_>>(), ["reason"]);
    let (status, _) = http(&a.app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": "q3"}))).await;
    assert_eq!(status, StatusCode::CONFLICT, "nothing was scheduled");
}

#[tokio::test]
async fn ac101_a_record_that_does_not_load_is_refused_with_the_loaders_reason() {
    let a = admin();
    let mut four = q3_json();
    four["options"].as_array_mut().unwrap().pop();
    let (status, body) = push(&a, "q3", &four).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["reason"].as_str().unwrap().contains("4 options"), "{body}");
    let (status, _, _) = raw(&a.app, Method::PUT, "/admin/questions/q3", Some(&format!("Bearer {}", a.token)), Some("{not json".into())).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let huge = "x".repeat(room::admin::MAX_RECORD_BYTES + 1);
    let (status, _, _) = raw(&a.app, Method::PUT, "/admin/questions/q3", Some(&format!("Bearer {}", a.token)), Some(huge)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn ac101_past_the_check_other_paths_and_methods_are_not_found_or_not_allowed() {
    let a = admin();
    let bearer = format!("Bearer {}", a.token);
    for (method, path, want) in [
        (Method::GET, "/admin", StatusCode::NOT_FOUND),
        (Method::GET, "/admin/x/y", StatusCode::NOT_FOUND),
        (Method::PUT, "/admin/used", StatusCode::METHOD_NOT_ALLOWED),
        (Method::GET, "/admin/questions/q3", StatusCode::METHOD_NOT_ALLOWED),
        (Method::DELETE, "/admin/questions/q3", StatusCode::METHOD_NOT_ALLOWED),
    ] {
        assert_eq!(raw(&a.app, method.clone(), path, Some(&bearer), None).await.0, want, "{method} {path}");
    }
}

// --------------------------------------------------------------------------
// AC-101: the route table. The prefix is served to the check alone.
// --------------------------------------------------------------------------

fn src(file: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(file)).unwrap()
}

fn sources() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(dir) = stack.pop() {
        for e in std::fs::read_dir(dir).unwrap() {
            let path = e.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "rs") {
                let rel = path.strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")).join("src")).unwrap();
                out.push((rel.to_string_lossy().replace('\\', "/"), std::fs::read_to_string(&path).unwrap()));
            }
        }
    }
    assert!(out.len() > 10, "found the sources");
    out
}

/// `routes.rs`'s `// T-25 routes` block, and everything outside it.
fn routes_split() -> (String, String) {
    let s = src("routes.rs");
    let start = s.find("// T-25 routes ---").expect("the T-25 block");
    let end = s.find("// end T-25 routes ---").expect("the T-25 block ends");
    (s[start..end].to_string(), format!("{}{}", &s[..start], &s[end..]))
}

#[test]
fn ac101_the_admin_prefix_is_served_to_the_check_alone() {
    let (block, rest) = routes_split();
    // Every route in the block is the one entry point, with any method.
    let routes: Vec<&str> = block.split(".route(").skip(1).collect();
    assert_eq!(routes.len(), 3, "{block}");
    for r in &routes {
        let path = r.trim_start().strip_prefix('"').and_then(|p| p.split('"').next()).unwrap();
        assert!(path == "/admin" || path.starts_with("/admin/"), "{path}");
        assert!(r.contains("axum::routing::any(crate::admin::serve)"), "{path} goes to admin::serve alone: {r}");
    }
    for p in ["\"/admin\"", "\"/admin/\"", "\"/admin/{*rest}\""] {
        assert!(block.contains(p), "the block serves {p}");
    }
    // Nothing outside the block serves under the prefix, or names the module.
    assert!(!rest.contains("\"/admin"), "routes.rs serves /admin outside the T-25 block");
    assert!(!rest.contains("admin::"), "routes.rs names the admin module outside the T-25 block");
    // `serve` checks before it reads anything.
    let admin = src("admin.rs");
    let serve = &admin[admin.find("pub async fn serve(").unwrap()..];
    let body = &serve[serve.find('{').unwrap() + 1..];
    let first = body.trim_start().lines().next().unwrap();
    assert!(first.starts_with("if admin.token.check(request.headers()).is_err()"), "serve's first act is the check: {first}");
}

#[test]
fn ac101_g9_no_other_module_reads_the_token() {
    for (file, text) in sources() {
        let names_var = text.contains(VAR);
        let names_type = text.contains("AdminToken");
        match file.as_str() {
            "admin.rs" => assert!(names_var && names_type),
            "config.rs" => assert!(names_type, "config reads it through admin"),
            "lib.rs" => assert!(!names_var, "lib.rs wires the token and never names the variable"),
            "routes.rs" => {
                let (_, rest) = routes_split();
                assert!(!names_var && !rest.contains("AdminToken"), "routes.rs names the token outside the T-25 block");
            }
            _ => assert!(!names_var && !names_type, "{file} names the admin token"),
        }
    }
    // The variable's name is a code literal in admin.rs alone; config.rs
    // refers to it as `admin::VAR`.
    let literal = format!("\"{VAR}\"");
    for (file, text) in sources() {
        assert_eq!(text.contains(&literal), file == "admin.rs", "{file}");
    }
}

#[test]
fn g9_one_constant_time_compare_and_no_log() {
    let admin = src("admin.rs");
    assert_eq!(admin.matches("ct_eq(").count(), 1, "one constant-time compare");
    assert_eq!(admin.matches("fn check(").count(), 1, "one check");
    assert!(!admin.contains("token =="), "the token is never compared with ==");
    for logger in ["println!", "eprintln!", "print!", "eprint!", "dbg!", "tracing", "log::"] {
        assert!(!admin.contains(logger), "admin.rs logs ({logger})");
    }
    let lines: Vec<&str> = admin.lines().collect();
    let at = lines.iter().position(|l| l.starts_with("pub struct AdminToken")).unwrap();
    assert!(!lines[at - 1].starts_with("#["), "AdminToken derives nothing: {}", lines[at - 1]);
    for formatter in ["for AdminToken", "Debug for", "Display for"] {
        assert!(!admin.contains(formatter), "AdminToken is not formattable or clonable ({formatter})");
    }
}

#[tokio::test]
async fn ac101_no_participant_wall_host_or_auth_route_accepts_the_admin_token() {
    let a = admin();
    push(&a, "q3", &q3_json()).await;
    let room = host_room(&a.app).await;
    let bearer = Some(a.token.as_str());
    // The credential routes: create, run it again, every host action, the
    // host's view, the answer.
    let (status, _) = http(&a.app, Method::POST, "/rooms", bearer, Some(json!({"question_id": "q3"}))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "POST /rooms");
    for (_, path) in room::host_routes() {
        let uri = path.replace("{id}", &room.id);
        let (status, _) = http(&a.app, Method::POST, &uri, bearer, Some(json!({"question_id": "q3"}))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "POST {uri}");
    }
    let (status, _) = http(&a.app, Method::GET, &format!("/rooms/{}/host", room.id), bearer, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "GET host");
    let (status, _) = http(&a.app, Method::PUT, &format!("/rooms/{}/answer", room.id), bearer, Some(json!({"letter": "A"}))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "PUT answer");
    // The public routes: the same bytes with the admin bearer as without.
    for uri in [
        format!("/rooms/{}/wall", room.id),
        format!("/rooms/{}/buzzer", room.id),
        format!("/wall/{}", room.id),
        "/host".into(),
        "/join".into(),
        "/shared/tokens.css".into(),
        format!("/{}", room.code),
    ] {
        let with = raw(&a.app, Method::GET, &uri, Some(&format!("Bearer {}", a.token)), None).await;
        let without = raw(&a.app, Method::GET, &uri, None, None).await;
        assert_eq!(with, without, "GET {uri}");
    }
}

// --------------------------------------------------------------------------
// AC-61: a pushed answer enters the sealed module and nowhere else.
// --------------------------------------------------------------------------

#[tokio::test]
async fn ac61_a_pushed_answer_is_reachable_only_through_the_sealed_module() {
    let admin_rs = src("admin.rs");
    for name in ["Revealed", "Witness", "witness", "verified", "correct", "stdout", "vault"] {
        assert!(!admin_rs.contains(name), "admin.rs names {name}");
    }
    let a = admin();
    let planted = planted();
    assert_eq!(push(&a, "planted", &planted).await.0, StatusCode::CREATED);
    let (status, created) = http(&a.app, Method::POST, "/rooms", Some(ORGANIZER), Some(json!({"question_id": "planted"}))).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap();
    let host = created["host_session"].as_str().unwrap();
    let p = plants();
    for step in [None, Some(HostAction::PutOnScreen)] {
        if let Some(s) = step {
            http(&a.app, Method::POST, &format!("/rooms/{id}/{}", s.slug()), Some(host), None).await;
        }
        for (viewer, bearer) in [("wall", None), ("buzzer", None), ("host", Some(host))] {
            let (_, body) = http(&a.app, Method::GET, &format!("/rooms/{id}/{viewer}"), bearer, None).await;
            let text = body.to_string();
            for plant in [&p.correct, &p.receipt, &p.resolving] {
                assert!(!text.contains(plant.as_str()), "{viewer} before reveal carries a pushed secret");
            }
        }
    }
}

// --------------------------------------------------------------------------
// Configuration: required in every build.
// --------------------------------------------------------------------------

fn with(pairs: Vec<(String, String)>) -> impl Fn(&str) -> Option<String> {
    move |name| pairs.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone())
}

fn stand_in() -> Vec<(String, String)> {
    // A dev-host-token build needs its own token first; the name is
    // assembled, as in smoke_config.rs.
    let mut env = vec![(["HOST", "DEV", "TOKEN"].join("_"), "a-test-value".into())];
    // T-10: every build needs the four Discord variables too (ids are digits).
    env.extend(room::config::DISCORD_VARS.iter().map(|v| (v.to_string(), if v.ends_with("SECRET") { "a-test-value".into() } else { "1234".into() })));
    env
}

#[test]
fn ac101_the_token_is_required_and_never_echoed() {
    for value in [None, Some(""), Some(" "), Some("\t\n")] {
        let mut env = stand_in();
        if let Some(v) = value {
            env.push((VAR.into(), v.into()));
        }
        match Config::from_vars(with(env)) {
            Err(ConfigError::MissingAdminToken) => {}
            Err(e) => panic!("{value:?}: {e}"),
            Ok(_) => panic!("{value:?}: configured without the admin token"),
        }
    }
    let said = ConfigError::MissingAdminToken.to_string();
    assert!(said.contains(VAR), "{said}");
}

#[tokio::test]
async fn ac101_the_binarys_router_opens_to_the_configured_token() {
    let t = token();
    let mut env = stand_in();
    env.push((VAR.into(), t.clone()));
    let app = room::serving_router(Config::from_vars(with(env)).unwrap());
    let (status, body) = http(&app, Method::GET, "/admin/used", Some(&t), None).await;
    assert_eq!((status, body), (StatusCode::OK, json!([])));
    let (status, _) = http(&app, Method::GET, "/admin/used", Some("not-it"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // The default router serves the same check with a token nobody holds.
    let (status, _) = http(&room::router(), Method::GET, "/admin/used", Some(&t), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// --------------------------------------------------------------------------
// AC-101, static: the repository holds no admin token (`just secret-scan`).
// --------------------------------------------------------------------------

/// What makes a line hold a token. Returns one description per hit.
///
/// 1. The variable's name assigned a literal: `NAME=value`, `NAME: value`,
///    `"NAME": "value"`, with a value of 8 or more characters. A value that
///    opens with `$`, `<`, `‹`, `…` or `{` is a substitution or a placeholder.
/// 2. The admin plant's shape: `CANARY-ADMIN-TOKEN-` and 16 hex digits.
/// 3. `live`, if given: a value this process holds for the variable.
fn token_findings(text: &str, live: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find(VAR) {
        let at = from + i;
        from = at + VAR.len();
        let mut rest = text[from..].trim_start_matches(['"', '\'']).trim_start_matches([' ', '\t']);
        let Some(after) = rest.strip_prefix(['=', ':']) else { continue };
        rest = after.trim_start_matches([' ', '\t']).trim_start_matches(['"', '\'']);
        let value: String = rest
            .chars()
            .take_while(|c| !c.is_whitespace() && !matches!(c, '"' | '\'' | '`' | ',' | ')' | ';'))
            .collect();
        let placeholder = value.starts_with(['$', '<', '‹', '…', '{']);
        if value.chars().count() >= 8 && !placeholder {
            out.push(format!("{VAR} is given a {}-character value", value.chars().count()));
        }
    }
    let shape = ["CANARY", "ADMIN", "TOKEN", ""].join("-");
    let mut from = 0;
    while let Some(i) = text[from..].find(&shape) {
        let at = from + i + shape.len();
        from = at;
        let tail: Vec<char> = text[at..].chars().take(16).collect();
        if tail.len() == 16 && tail.iter().all(|c| c.is_ascii_hexdigit()) {
            out.push("a planted admin-token canary".into());
        }
    }
    if let Some(v) = live.filter(|v| v.len() >= 8) {
        if text.contains(v) {
            out.push(format!("the value this process holds for {VAR}"));
        }
    }
    out
}

#[test]
fn ac101_the_repository_holds_no_admin_token() {
    let root = repo();
    let listed = std::process::Command::new("git")
        .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
        .current_dir(&root)
        .output()
        .expect("git runs: the scan never passes by not scanning");
    assert!(listed.status.success(), "git ls-files: {}", String::from_utf8_lossy(&listed.stderr));
    let live = std::env::var(VAR).ok();
    let mut files = 0;
    let mut hits = Vec::new();
    for name in listed.stdout.split(|b| *b == 0).filter(|n| !n.is_empty()) {
        let rel = String::from_utf8_lossy(name).into_owned();
        let Ok(bytes) = std::fs::read(root.join(&rel)) else { continue };
        let Ok(text) = String::from_utf8(bytes) else { continue };
        files += 1;
        for hit in token_findings(&text, live.as_deref()) {
            hits.push(format!("{rel}: {hit}"));
        }
    }
    assert!(files > 100, "the scan read the repository ({files} files)");
    assert!(hits.is_empty(), "the repository holds an admin token:\n{}", hits.join("\n"));
}

#[test]
fn ac101_the_repository_scan_catches_a_plant() {
    let hex: String = token();
    let plant = format!("{}{}", ["CANARY", "ADMIN", "TOKEN", ""].join("-"), &hex[..16].to_uppercase());
    for leak in [
        format!("{VAR}={hex}"),
        format!("export {VAR}='{hex}'"),
        format!("fly secrets set {VAR}={hex}"),
        format!("{{\"{VAR}\": \"{hex}\"}}"),
        format!("{VAR}: {hex}"),
        format!("a log line with {plant} in it"),
    ] {
        assert!(!token_findings(&leak, None).is_empty(), "missed: {leak}");
    }
    assert!(!token_findings(&format!("x {hex} y"), Some(&hex)).is_empty(), "missed the live value");
    for fine in [
        format!("{VAR}="),
        format!("{VAR}=\n"),
        format!("fly secrets set {VAR}=\"$(openssl rand -hex 32)\""),
        format!("fly secrets set {VAR}=…"),
        format!("{VAR}=<the token>"),
        format!("Authorization: Bearer ‹{VAR}›"),
        format!("pub const VAR: &str = \"{VAR}\";"),
        format!("room::admin::VAR, the variable {VAR}, is required"),
    ] {
        assert!(token_findings(&fine, None).is_empty(), "flagged: {fine}");
    }
}

// --------------------------------------------------------------------------
// D-26 / review of PR #52 (P1): a scheduled record belongs to one club.
// --------------------------------------------------------------------------

#[tokio::test]
async fn d26_the_club_route_and_the_old_route_are_two_records() {
    let a = admin();
    // The old path is the default club's; a club's path is that club's own.
    assert_eq!(push(&a, "q3", &q3_json()).await.1, json!({"id": "q3", "scheduled": "new"}));
    assert_eq!(push_club(&a, "la", "q3", &q3_json()).await.1, json!({"id": "q3", "scheduled": "new"}));
    assert_eq!(push_club(&a, "la", "q3", &q3_json()).await.1, json!({"id": "q3", "scheduled": "replaced"}));
    assert_eq!(push_club(&a, "nyc", "q3", &q3_json()).await.1, json!({"id": "q3", "scheduled": "replaced"}), "the old path is nyc's");
}

#[tokio::test]
async fn d26_the_club_route_is_behind_the_token_and_refuses_a_bad_club_name() {
    let a = admin();
    for uri in ["/admin/clubs/la/questions/q3", "/admin/clubs/NOPE/questions/q3"] {
        let (status, body) = http(&a.app, Method::PUT, uri, None, Some(q3_json())).await;
        assert_eq!((status, body), (StatusCode::UNAUTHORIZED, Value::Null), "{uri}: nothing is said before the check");
    }
    let (status, body) = push_club(&a, "NOPE", "q3", &q3_json()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert!(!body.to_string().contains("nyc"), "the answer names no club: {body}");
    let (status, _) = http(&a.app, Method::GET, "/admin/clubs/la/questions/q3", Some(&a.token), None).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}
