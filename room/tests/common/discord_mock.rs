//! T-10: an in-process Discord, and a room wired to it.
//!
//! [`Mock`] serves the three Discord routes the room calls, on loopback:
//! `POST /api/v10/oauth2/token` (both grants, with refresh-token rotation and
//! replay detection as Discord does it), `GET /api/v10/users/@me`, and
//! `GET /api/v10/users/@me/guilds/{guild}/member`. It can be told to answer
//! `429` with `Retry-After`, or `5xx`, a number of times, or to go down
//! entirely. Every value it is configured with or issues — the client id and
//! secret, the guild and role ids, every access and refresh token — is drawn
//! per run, so the secrecy scan plants canaries nobody wrote down.
//!
//! [`Rig`] is a room on the Discord backend over that mock, on a
//! `ManualClock`, with q3 and q3-again scheduled, and a captured log.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Form, Path, State};
use axum::http::{header, HeaderMap, Method, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use room::discord::{Discord, Endpoints, Retry, Secret, Settings};
use room::lifecycle::ManualClock;
use room::rooms::{AppState, Urls};
use serde_json::{json, Value};
use tower::ServiceExt;

/// A random lowercase hex string, tagged so a finding names what leaked.
pub fn canary(tag: &str) -> String {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).unwrap();
    format!("{tag}-{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>())
}

/// A random Discord-shaped id: 19 digits.
pub fn snowflake() -> String {
    let mut b = [0u8; 8];
    getrandom::fill(&mut b).unwrap();
    format!("1{:018}", u64::from_le_bytes(b) % 1_000_000_000_000_000_000)
}

/// What the member route says about an account.
#[derive(Clone, Debug)]
pub enum Membership {
    /// Not in the guild: `404 Unknown Guild`.
    Absent,
    /// In the guild with these role ids. `admin` sets every permission bit and
    /// marks the account as the guild's owner: things the check must ignore.
    Member { roles: Vec<String>, admin: bool },
}

#[derive(Default)]
struct Books {
    /// Authorization code → user id.
    codes: HashMap<String, String>,
    members: HashMap<String, Membership>,
    /// Live access token → user id.
    access: HashMap<String, String>,
    /// The one live refresh token per user.
    refresh: HashMap<String, String>,
    spent: HashSet<String>,
    replays: u32,
    /// Every refresh token presented, in order.
    presented: Vec<String>,
    /// Every token ever issued.
    issued: Vec<String>,
    /// `(status, times left, Retry-After)` answered before any route.
    fault: Option<(u16, u32, Option<f64>)>,
    hits: HashMap<&'static str, u32>,
    expires_in: u64,
}

/// The mock's configuration: what a real Discord application would hold.
pub struct MockState {
    pub client_id: String,
    pub client_secret: String,
    pub guild_id: String,
    pub role_id: String,
    /// D-26: a second club's host role, on the same server.
    pub la_role_id: String,
    pub redirect_uri: String,
    books: Mutex<Books>,
}

pub struct Mock {
    pub base: String,
    pub state: Arc<MockState>,
    task: tokio::task::JoinHandle<()>,
}

impl Mock {
    pub async fn start(redirect_uri: &str) -> Mock {
        let state = Arc::new(MockState {
            client_id: snowflake(),
            client_secret: canary("CANARY-CLIENT-SECRET"),
            guild_id: snowflake(),
            role_id: snowflake(),
            la_role_id: snowflake(),
            redirect_uri: redirect_uri.to_string(),
            books: Mutex::new(Books {
                expires_in: 604_800,
                ..Books::default()
            }),
        });
        let app = Router::new()
            .route("/api/v10/oauth2/token", post(token))
            .route("/api/v10/users/@me", get(me))
            .route("/api/v10/users/@me/guilds/{guild}/member", get(member))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Mock { base, state, task }
    }

    /// Stop answering: the listener closes, so every connection is refused.
    pub async fn down(&mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }

    fn books(&self) -> std::sync::MutexGuard<'_, Books> {
        self.state.books.lock().unwrap()
    }

    /// A Discord account that will consent: its authorization code and id.
    pub fn account(&self, membership: Membership) -> (String, String) {
        let (code, id) = (canary("code"), snowflake());
        let mut b = self.books();
        b.codes.insert(code.clone(), id.clone());
        b.members.insert(id.clone(), membership);
        (code, id)
    }

    pub fn set_membership(&self, user: &str, membership: Membership) {
        self.books().members.insert(user.to_string(), membership);
    }

    /// Answer the next `times` requests, on any route, with `status`.
    pub fn fault(&self, status: u16, times: u32, retry_after: Option<f64>) {
        self.books().fault = Some((status, times, retry_after));
    }

    /// Access tokens issued from now on last this long.
    pub fn expires_in(&self, secs: u64) {
        self.books().expires_in = secs;
    }

    /// Withdraw the user's access token (as a revoked authorization would).
    pub fn revoke_access(&self, user: &str) {
        self.books().access.retain(|_, u| u != user);
    }

    /// Withdraw the user's refresh token too: the next refresh is `invalid_grant`.
    pub fn revoke_refresh(&self, user: &str) {
        let mut b = self.books();
        if let Some(t) = b.refresh.remove(user) {
            b.spent.insert(t);
        }
    }

    pub fn hits(&self, route: &str) -> u32 {
        self.books().hits.get(route).copied().unwrap_or(0)
    }

    pub fn total_hits(&self) -> u32 {
        self.books().hits.values().sum()
    }

    pub fn replays(&self) -> u32 {
        self.books().replays
    }

    pub fn presented(&self) -> Vec<String> {
        self.books().presented.clone()
    }

    /// The live refresh token for `user`.
    pub fn live_refresh(&self, user: &str) -> Option<String> {
        self.books().refresh.get(user).cloned()
    }

    /// Every token the mock has issued.
    pub fn issued(&self) -> Vec<String> {
        self.books().issued.clone()
    }
}

/// Count the hit, then answer with the configured fault if one is armed.
fn enter(state: &MockState, route: &'static str) -> Option<Response> {
    let mut b = state.books.lock().unwrap();
    *b.hits.entry(route).or_default() += 1;
    let (status, left, retry_after) = b.fault?;
    if left == 0 {
        b.fault = None;
        return None;
    }
    b.fault = Some((status, left - 1, retry_after));
    let status = StatusCode::from_u16(status).unwrap();
    let mut body = json!({ "message": "mock fault" });
    if let Some(s) = retry_after {
        body["retry_after"] = json!(s);
    }
    let mut r = (status, Json(body)).into_response();
    if let Some(s) = retry_after {
        r.headers_mut().insert("retry-after", s.to_string().parse().unwrap());
    }
    Some(r)
}

fn bearer_user(state: &MockState, headers: &HeaderMap) -> Option<String> {
    let token = headers.get(header::AUTHORIZATION)?.to_str().ok()?.strip_prefix("Bearer ")?;
    state.books.lock().unwrap().access.get(token).cloned()
}

fn issue(b: &mut Books, user: &str) -> Value {
    let (access, refresh) = (canary("CANARY-ACCESS"), canary("CANARY-REFRESH"));
    b.access.insert(access.clone(), user.to_string());
    if let Some(old) = b.refresh.insert(user.to_string(), refresh.clone()) {
        b.spent.insert(old);
    }
    b.issued.push(access.clone());
    b.issued.push(refresh.clone());
    json!({
        "access_token": access,
        "token_type": "Bearer",
        "expires_in": b.expires_in,
        "refresh_token": refresh,
        "scope": "identify guilds.members.read",
    })
}

fn invalid_grant() -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": "invalid_grant" }))).into_response()
}

async fn token(State(state): State<Arc<MockState>>, Form(form): Form<HashMap<String, String>>) -> Response {
    if let Some(r) = enter(&state, "token") {
        return r;
    }
    let field = |k: &str| form.get(k).map(String::as_str).unwrap_or("");
    if field("client_id") != state.client_id || field("client_secret") != state.client_secret {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "error": "invalid_client" }))).into_response();
    }
    let mut b = state.books.lock().unwrap();
    match field("grant_type") {
        "authorization_code" => {
            if field("redirect_uri") != state.redirect_uri {
                return invalid_grant();
            }
            // A code works once.
            match b.codes.remove(field("code")) {
                Some(user) => Json(issue(&mut b, &user)).into_response(),
                None => invalid_grant(),
            }
        }
        "refresh_token" => {
            let presented = field("refresh_token").to_string();
            b.presented.push(presented.clone());
            let user = b.refresh.iter().find(|(_, t)| **t == presented).map(|(u, _)| u.clone());
            match user {
                Some(user) => {
                    // Rotation: the old access token dies with the old refresh token.
                    b.access.retain(|_, u| *u != user);
                    Json(issue(&mut b, &user)).into_response()
                }
                None => {
                    if b.spent.contains(&presented) {
                        b.replays += 1;
                    }
                    invalid_grant()
                }
            }
        }
        _ => (StatusCode::BAD_REQUEST, Json(json!({ "error": "unsupported_grant_type" }))).into_response(),
    }
}

async fn me(State(state): State<Arc<MockState>>, headers: HeaderMap) -> Response {
    if let Some(r) = enter(&state, "me") {
        return r;
    }
    match bearer_user(&state, &headers) {
        Some(id) => Json(json!({ "id": id, "username": "organizer", "global_name": null })).into_response(),
        None => (StatusCode::UNAUTHORIZED, Json(json!({ "message": "401: Unauthorized", "code": 0 }))).into_response(),
    }
}

async fn member(State(state): State<Arc<MockState>>, Path(guild): Path<String>, headers: HeaderMap) -> Response {
    if let Some(r) = enter(&state, "member") {
        return r;
    }
    let Some(user) = bearer_user(&state, &headers) else {
        return (StatusCode::UNAUTHORIZED, Json(json!({ "message": "401: Unauthorized", "code": 0 }))).into_response();
    };
    let unknown = || (StatusCode::NOT_FOUND, Json(json!({ "message": "Unknown Guild", "code": 10004 }))).into_response();
    if guild != state.guild_id {
        return unknown();
    }
    let membership = state.books.lock().unwrap().members.get(&user).cloned();
    match membership {
        Some(Membership::Member { roles, admin }) => Json(json!({
            "user": { "id": user, "username": "organizer" },
            "nick": null,
            "roles": roles,
            "joined_at": "2024-01-01T00:00:00.000000+00:00",
            "deaf": false,
            "mute": false,
            "flags": 0,
            // Every permission bit, Administrator included, and ownership:
            // none of it may grant hosting (AC-65).
            "permissions": if admin { "2251799813685247" } else { "0" },
            "is_owner": admin,
        }))
        .into_response(),
        _ => unknown(),
    }
}

// --------------------------------------------------------------------------
// The rig.
// --------------------------------------------------------------------------

/// Every line the Discord backend logged.
#[derive(Default)]
pub struct CaptureLog(pub Mutex<Vec<String>>);

impl room::discord::Log for CaptureLog {
    fn line(&self, line: &str) {
        self.0.lock().unwrap().push(line.to_string());
    }
}

/// The retry budget the tests run on: the default's shape, in milliseconds.
pub fn fast_retry() -> Retry {
    Retry {
        attempts: 3,
        first_backoff: Duration::from_millis(20),
        per_attempt: Duration::from_millis(800),
        deadline: Duration::from_secs(3),
    }
}

/// A raw response: status, headers, body.
pub struct Raw {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl Raw {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or(Value::Null)
    }
}

pub struct Rig {
    pub mock: Mock,
    pub discord: Arc<Discord>,
    pub state: Arc<AppState>,
    pub app: Router,
    pub clock: Arc<ManualClock>,
    pub log: Arc<CaptureLog>,
    /// Every response body and header value the tests saw, for the scan.
    pub seen: Mutex<Vec<String>>,
}

impl Rig {
    pub async fn start() -> Rig {
        Rig::start_with(fast_retry()).await
    }

    pub async fn start_with(retry: Retry) -> Rig {
        Rig::start_custom(retry, None).await
    }

    /// A room configured for a guild the mock does not have: every account is
    /// in the wrong server, members of the mock's guild included.
    pub async fn start_elsewhere() -> Rig {
        Rig::start_custom(fast_retry(), Some(snowflake())).await
    }

    pub async fn start_custom(retry: Retry, guild: Option<String>) -> Rig {
        let urls = Urls::default();
        let redirect_uri = format!("{}{}", urls.base, room::discord::CALLBACK_PATH);
        let mock = Mock::start(&redirect_uri).await;
        let clock = ManualClock::new(super::meetup_evening());
        let log = Arc::new(CaptureLog::default());
        let m = &mock.state;
        let settings = Settings {
            client_id: m.client_id.clone(),
            client_secret: Secret::new(m.client_secret.clone()),
            guild_id: guild.unwrap_or_else(|| m.guild_id.clone()),
            clubs: room::club::Clubs::parse(m.role_id.clone(), Some(&format!("la={}:America/Los_Angeles", m.la_role_id)))
                .expect("the mock's clubs"),
            redirect_uri,
        };
        let discord = Arc::new(Discord::new(settings, Endpoints::at(&mock.base), retry, clock.clone(), log.clone()));
        let state = Arc::new(
            AppState::new(discord.clone(), vec![super::q3(), super::q3_again()], urls)
                .with_clock(clock.clone())
                .with_discord(discord.clone()),
        );
        // D-26: the fixtures are scheduled for the second club too, as a push
        // to /admin/clubs/la/questions/{id} would.
        let la = room::club::ClubSlug::parse("la").unwrap();
        for q in [super::q3(), super::q3_again()] {
            state.schedule_for(&la, q).expect("the fixtures schedule for la");
        }
        let app = room::router_with(state.clone());
        Rig {
            mock,
            discord,
            state,
            app,
            clock,
            log,
            seen: Mutex::new(Vec::new()),
        }
    }

    /// The host role's id.
    pub fn role(&self) -> String {
        self.mock.state.role_id.clone()
    }

    /// The second club's (`la`) host role id.
    pub fn la_role(&self) -> String {
        self.mock.state.la_role_id.clone()
    }

    /// A member with the second club's host role and not the first's.
    pub fn la_host(&self) -> Membership {
        Membership::Member {
            roles: vec![snowflake(), self.la_role()],
            admin: false,
        }
    }

    /// A member with the host role.
    pub fn host(&self) -> Membership {
        Membership::Member {
            roles: vec![snowflake(), self.role()],
            admin: false,
        }
    }

    /// One request through the router, recorded for the secrecy scan.
    pub async fn raw(&self, method: Method, uri: &str, headers: &[(&str, &str)], body: Option<Value>) -> Raw {
        let mut req = Request::builder().method(method).uri(uri);
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        let req = match body {
            Some(v) => req.header(header::CONTENT_TYPE, "application/json").body(Body::from(v.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let res = self.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let headers = res.headers().clone();
        let body = axum::body::to_bytes(res.into_body(), 1 << 22).await.unwrap().to_vec();
        let mut seen = self.seen.lock().unwrap();
        for v in headers.values() {
            seen.push(String::from_utf8_lossy(v.as_bytes()).into_owned());
        }
        seen.push(String::from_utf8_lossy(&body).into_owned());
        Raw { status, headers, body }
    }

    /// A JSON call with an optional bearer: `(status, body or Null)`.
    pub async fn call(&self, method: Method, uri: &str, bearer: Option<&str>, body: Option<Value>) -> (StatusCode, Value) {
        let auth = bearer.map(|b| format!("Bearer {b}"));
        let headers: Vec<(&str, &str)> = auth.iter().map(|a| ("authorization", a.as_str())).collect();
        let r = self.raw(method, uri, &headers, body).await;
        let json = r.json();
        (r.status, json)
    }

    /// `GET /auth/discord?question=…`: the `state`, and the cookie to send back.
    pub async fn begin(&self, question: &str) -> (Raw, String, String) {
        let r = self.raw(Method::GET, &format!("/auth/discord?question={question}"), &[], None).await;
        assert_eq!(r.status, StatusCode::SEE_OTHER, "sign-in start");
        let location = r.header("location").unwrap().to_string();
        let state = query(&location, "state").expect("state in the authorize URL");
        let cookie = r
            .header("set-cookie")
            .and_then(|c| c.split(';').next())
            .expect("the state cookie")
            .to_string();
        (r, state, cookie)
    }

    /// The callback, as the browser would make it.
    pub async fn callback(&self, query: &str, cookie: Option<&str>) -> Raw {
        let headers: Vec<(&str, &str)> = cookie.iter().map(|c| ("cookie", *c)).collect();
        self.raw(Method::GET, &format!("{}?{query}", room::discord::CALLBACK_PATH), &headers, None).await
    }

    /// Sign a new account in, all the way: its organizer session and id.
    pub async fn sign_in(&self, membership: Membership) -> (String, String) {
        let (code, id) = self.mock.account(membership);
        let (_, state, cookie) = self.begin("q3").await;
        let r = self.callback(&format!("code={code}&state={state}"), Some(&cookie)).await;
        assert_eq!(r.status, StatusCode::SEE_OTHER, "callback: {:?}", String::from_utf8_lossy(&r.body));
        let location = r.header("location").unwrap();
        let (path, session) = location.split_once('#').expect("a session in the fragment");
        assert_eq!(path, "/host?question=q3");
        (session.to_string(), id)
    }

    /// *Create a room* with this bearer.
    pub async fn create(&self, bearer: Option<&str>, question: &str) -> (StatusCode, Value) {
        self.call(Method::POST, "/rooms", bearer, Some(json!({ "question_id": question }))).await
    }

    /// *Create a room* for a named club (D-26).
    pub async fn create_for(&self, bearer: Option<&str>, club: &str, question: &str) -> (StatusCode, Value) {
        self.call(Method::POST, "/rooms", bearer, Some(json!({ "question_id": question, "club": club }))).await
    }
}

/// A query parameter from a URL, percent-decoding nothing (the values here are
/// hex or already encoded).
pub fn query(url: &str, name: &str) -> Option<String> {
    let q = url.split_once('?')?.1;
    q.split('&').find_map(|kv| kv.strip_prefix(name)?.strip_prefix('=').map(str::to_string))
}
