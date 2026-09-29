//! Discord: who may create a room (SPEC §8, AC-64…AC-70, T-10).
//!
//! The one credential path for *Create a room* (G-9). Two halves:
//!
//! - **Sign-in** — `GET /auth/discord` sends the organizer to Discord's
//!   consent screen with the scopes `identify guilds.members.read`; the
//!   callback exchanges the code, reads the organizer's Discord id, stores the
//!   organizer record (§3.5) and mints an opaque **organizer session**. The
//!   session is the only thing the browser ever holds: it rides the host
//!   page's URL fragment and is presented as the bearer on `POST /rooms`.
//!   Discord's tokens never leave this module — not in a page, a payload or a
//!   log line.
//! - **The check** — [`HostAuth::authorize_create`]: the session names an
//!   organizer; an access token at or near its expiry is refreshed first; then
//!   `GET /users/@me/guilds/{guild}/member` with the organizer's bearer, and
//!   the organizer hosts iff `roles` contains the configured role **ID**,
//!   compared as a string. The member record is read for `roles` and nothing
//!   else: never `permissions`, never a role name, never ownership (AC-65).
//!   A `404` is *wrong server* — what a non-member and a member of another
//!   guild both hear, so membership never leaks (AC-70); a member without the
//!   role is *wrong role*.
//!
//! Discord is asked at creation and never again: a room's host commands use
//! the room's own host session, so an open room runs to release with Discord
//! down (AC-69).
//!
//! **Refresh-token rotation (AC-66).** Discord rotates the refresh token on
//! every refresh and refuses the old one afterwards. The rotated pair is
//! written into the organizer record in the same step that removes the old
//! one, before anything else can read the record. Refreshes of one organizer
//! are serialized, and a refresh that finds the record already rotated does
//! not refresh again, so two creates at once never replay a spent token. If
//! Discord refuses the refresh token anyway (`invalid_grant`: revoked, or
//! replayed by something outside this process), the organizer is signed out —
//! the host page offers *Sign in* again — and their open rooms are untouched.
//! The check runs as its own task, so a browser that goes away mid-refresh
//! cannot drop a rotated token on the floor.
//!
//! **Retries.** Every Discord call gets at most [`Retry::attempts`] tries,
//! each bounded by [`Retry::per_attempt`], with exponential backoff from
//! [`Retry::first_backoff`] or Discord's own `Retry-After` when it sends one,
//! and the whole check is bounded by [`Retry::deadline`]. Only a transport
//! failure, a timeout, `429` and `5xx` are retried: a denial is an answer, and
//! Discord's 10,000-invalid-requests ban is per IP. When the budget runs out
//! the check says [`CreateRefusal::Unavailable`], the plain server error.
//!
//! **Where the records live.** In memory, in this backend, which the
//! [`AppState`](crate::rooms::AppState) owns beside the rooms (§9: one
//! authoritative state, in memory). Only this module reads or writes them, and
//! no room lock is held while Discord is awaited. A restart forgets every
//! organizer, as it forgets every room: the organizer signs in again.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

use serde::Deserialize;
use subtle::ConstantTimeEq;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use crate::auth::{decided, CreateCheck, CreateRefusal, HostAuth, OrganizerId};
use crate::lifecycle::{Clock, SystemClock};

/// The OAuth2 scopes, exactly SPEC §8's.
pub const SCOPES: &str = "identify guilds.members.read";

/// Where the callback lives, under the public URL. The Discord application
/// registers `{POPQUIZ_PUBLIC_URL}{CALLBACK_PATH}` as a redirect.
pub const CALLBACK_PATH: &str = "/auth/discord/callback";

/// The cookie that binds a sign-in's `state` to the browser that started it.
pub const STATE_COOKIE: &str = "pq_oauth";

/// How long an organizer session is good for.
pub const SESSION_LIFETIME: Duration = Duration::from_secs(12 * 60 * 60);

/// How long a sign-in may sit at Discord's consent screen.
pub const SIGN_IN_WINDOW: Duration = Duration::from_secs(10 * 60);

/// An access token this close to its expiry is refreshed before use.
const REFRESH_MARGIN: Duration = Duration::from_secs(60);

const MAX_PENDING: usize = 256;
const MAX_SESSIONS: usize = 1024;
const MAX_BODY: usize = 64 * 1024;

// --------------------------------------------------------------------------
// Configuration.
// --------------------------------------------------------------------------

/// A secret string. No `Debug`, `Display` or `Clone`, so it cannot be
/// formatted or copied into anything by accident.
pub struct Secret(Box<str>);

impl Secret {
    pub fn new(value: String) -> Secret {
        Secret(value.into_boxed_str())
    }

    fn expose(&self) -> &str {
        &self.0
    }

    fn matches(&self, other: &str) -> bool {
        bool::from(self.0.as_bytes().ct_eq(other.as_bytes()))
    }
}

/// The Discord application and the guild it guards. Built by
/// [`crate::config`] from the four `DISCORD_*` variables and the public URL.
pub struct Settings {
    pub client_id: String,
    pub client_secret: Secret,
    pub guild_id: String,
    pub role_id: String,
    /// `{POPQUIZ_PUBLIC_URL}{CALLBACK_PATH}`.
    pub redirect_uri: String,
}

/// Where Discord is. [`Endpoints::discord`] in the binary; the tests' mock
/// otherwise. Not configurable from the environment.
pub struct Endpoints {
    /// The API base, e.g. `https://discord.com/api/v10`.
    pub api: String,
    /// The consent screen, e.g. `https://discord.com/oauth2/authorize`.
    pub authorize: String,
}

impl Endpoints {
    pub fn discord() -> Endpoints {
        Endpoints {
            api: "https://discord.com/api/v10".into(),
            authorize: "https://discord.com/oauth2/authorize".into(),
        }
    }

    /// A Discord lookalike at `base` (`http://127.0.0.1:<port>`): the API under
    /// `/api/v10`, the consent screen at `/oauth2/authorize`.
    pub fn at(base: &str) -> Endpoints {
        let base = base.trim_end_matches('/');
        Endpoints {
            api: format!("{base}/api/v10"),
            authorize: format!("{base}/oauth2/authorize"),
        }
    }
}

/// How hard a check tries before it calls Discord unavailable.
#[derive(Clone, Copy, Debug)]
pub struct Retry {
    /// Tries per Discord call, the first included.
    pub attempts: u32,
    /// The wait before the second try; doubled for each one after.
    pub first_backoff: Duration,
    /// One try's bound, connection to last byte.
    pub per_attempt: Duration,
    /// The whole check's bound, every call and wait included.
    pub deadline: Duration,
}

impl Default for Retry {
    fn default() -> Retry {
        Retry {
            attempts: 3,
            first_backoff: Duration::from_millis(250),
            per_attempt: Duration::from_secs(3),
            deadline: Duration::from_secs(8),
        }
    }
}

/// Where the backend's log lines go. One line per Discord call: which call,
/// its status or that there was none, and the attempt. Never a token, a
/// header, a body, a code or a state.
pub trait Log: Send + Sync {
    fn line(&self, line: &str);
}

/// Standard error, which Fly collects.
pub struct Stderr;

impl Log for Stderr {
    fn line(&self, line: &str) {
        eprintln!("{line}");
    }
}

// --------------------------------------------------------------------------
// The records (SPEC §3.5).
// --------------------------------------------------------------------------

/// The organizer record: SPEC §3.5's four fields and nothing else.
struct Organizer {
    discord_user_id: String,
    access_token: Secret,
    refresh_token: Secret,
    token_expires_at: SystemTime,
}

struct Session {
    token: Secret,
    discord_user_id: String,
    issued_at: SystemTime,
}

struct Pending {
    state: Secret,
    question: String,
    started_at: SystemTime,
}

#[derive(Default)]
struct Store {
    organizers: HashMap<String, Organizer>,
    sessions: Vec<Session>,
    pending: Vec<Pending>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn random_hex() -> String {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).expect("the OS random source is available");
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// A question id as the host page carries it. Anything else is refused before
/// it could reach a redirect.
pub fn is_question_id(id: &str) -> bool {
    (1..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

// --------------------------------------------------------------------------
// The backend.
// --------------------------------------------------------------------------

/// The Discord backend: [`HostAuth`] for *Create a room*, and sign-in.
pub struct Discord {
    inner: Arc<Inner>,
}

struct Inner {
    settings: Settings,
    endpoints: Endpoints,
    retry: Retry,
    http: Client,
    clock: Arc<dyn Clock>,
    log: Arc<dyn Log>,
    store: Mutex<Store>,
    /// One lock per organizer, held across a refresh (AC-66).
    refreshing: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

/// How a sign-in callback ended, for the route to say.
#[derive(Debug, PartialEq, Eq)]
pub enum Finish {
    /// A wrong, replayed, expired or cookieless `state`: refused, nothing said.
    Refused,
    /// The organizer declined at Discord. Back to the sign-in screen.
    Declined { question: String },
    /// Discord did not complete the exchange.
    Unavailable,
    /// Signed in: land on the host page with this session in the fragment.
    SignedIn { question: String, session: String },
}

impl Discord {
    /// The binary's backend: real Discord, the default retry budget, the
    /// system clock, standard error.
    pub fn serving(settings: Settings) -> Discord {
        Discord::new(settings, Endpoints::discord(), Retry::default(), Arc::new(SystemClock), Arc::new(Stderr))
    }

    pub fn new(settings: Settings, endpoints: Endpoints, retry: Retry, clock: Arc<dyn Clock>, log: Arc<dyn Log>) -> Discord {
        Discord {
            inner: Arc::new(Inner {
                settings,
                endpoints,
                retry,
                http: Client::new(),
                clock,
                log,
                store: Mutex::new(Store::default()),
                refreshing: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// Start a sign-in for `question`: the `state` (for the cookie) and the
    /// URL of Discord's consent screen. `None` if `question` is not an id.
    pub fn begin(&self, question: &str) -> Option<(String, String)> {
        if !is_question_id(question) {
            return None;
        }
        let inner = &self.inner;
        let now = inner.clock.now();
        let state = random_hex();
        {
            let mut store = lock(&inner.store);
            store.pending.retain(|p| now < p.started_at + SIGN_IN_WINDOW);
            if store.pending.len() >= MAX_PENDING {
                store.pending.remove(0);
            }
            store.pending.push(Pending {
                state: Secret::new(state.clone()),
                question: question.to_string(),
                started_at: now,
            });
        }
        let s = &inner.settings;
        let url = format!(
            "{}?response_type=code&client_id={}&scope={}&redirect_uri={}&state={}",
            inner.endpoints.authorize,
            encode(&s.client_id),
            encode(SCOPES),
            encode(&s.redirect_uri),
            state
        );
        Some((state, url))
    }

    /// The callback. `cookie` is the [`STATE_COOKIE`] the browser sent; it must
    /// equal `state`, and `state` must be a sign-in this backend started and
    /// has not finished. Either way the pending sign-in is spent.
    pub async fn finish(&self, state: Option<&str>, cookie: Option<&str>, code: Option<&str>, declined: bool) -> Finish {
        let inner = &self.inner;
        let Some(state) = state else { return Finish::Refused };
        let now = inner.clock.now();
        let question = {
            let mut store = lock(&inner.store);
            let Some(i) = store.pending.iter().position(|p| p.state.matches(state)) else {
                return Finish::Refused;
            };
            let pending = store.pending.remove(i);
            let bound = cookie.is_some_and(|c| pending.state.matches(c));
            if !bound || now >= pending.started_at + SIGN_IN_WINDOW {
                return Finish::Refused;
            }
            pending.question
        };
        if declined {
            return Finish::Declined { question };
        }
        let Some(code) = code.filter(|c| !c.is_empty()) else {
            return Finish::Refused;
        };
        let deadline = Instant::now() + inner.retry.deadline;
        let s = &inner.settings;
        let form = form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &s.redirect_uri),
            ("client_id", &s.client_id),
            ("client_secret", s.client_secret.expose()),
        ]);
        let Ok(r) = inner.call("sign-in exchange", deadline, Req::form(&inner.token_url(), form)).await else {
            return Finish::Unavailable;
        };
        let Some(tokens) = (r.status == 200).then(|| r.json::<Tokens>()).flatten() else {
            return Finish::Unavailable;
        };
        let me = Req::get(&format!("{}/users/@me", inner.endpoints.api), &tokens.access_token);
        let Ok(r) = inner.call("sign-in user", deadline, me).await else {
            return Finish::Unavailable;
        };
        let Some(user) = (r.status == 200).then(|| r.json::<User>()).flatten() else {
            return Finish::Unavailable;
        };
        let now = inner.clock.now();
        let session = random_hex();
        let mut store = lock(&inner.store);
        store.organizers.insert(
            user.id.clone(),
            Organizer {
                discord_user_id: user.id.clone(),
                access_token: Secret::new(tokens.access_token),
                refresh_token: Secret::new(tokens.refresh_token),
                token_expires_at: now + Duration::from_secs(tokens.expires_in),
            },
        );
        store.sessions.retain(|x| now < x.issued_at + SESSION_LIFETIME);
        if store.sessions.len() >= MAX_SESSIONS {
            store.sessions.remove(0);
        }
        store.sessions.push(Session {
            token: Secret::new(session.clone()),
            discord_user_id: user.id,
            issued_at: now,
        });
        Finish::SignedIn { question, session }
    }

    /// How many organizers are signed in on this machine.
    pub fn organizer_count(&self) -> usize {
        lock(&self.inner.store).organizers.len()
    }
}

impl HostAuth for Discord {
    fn authorize_create<'a>(&'a self, bearer: Option<&'a str>) -> CreateCheck<'a> {
        // Resolving the session needs no I/O: a bearer that names no organizer
        // is refused here, before any task or any call to Discord.
        let Some(user) = bearer.and_then(|b| self.inner.session_user(b)) else {
            return decided(Err(CreateRefusal::Denied));
        };
        let inner = self.inner.clone();
        Box::pin(async move {
            // Its own task: see "Refresh-token rotation" above.
            match tokio::runtime::Handle::try_current() {
                Ok(rt) => rt.spawn(inner.check(user)).await.unwrap_or(Err(CreateRefusal::Unavailable)),
                Err(_) => Err(CreateRefusal::Unavailable),
            }
        })
    }
}

/// `GET /users/@me/guilds/{guild}/member`, read for `roles` alone (AC-65).
#[derive(Deserialize)]
struct Member {
    roles: Vec<String>,
}

#[derive(Deserialize)]
struct Tokens {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Deserialize)]
struct User {
    id: String,
}

#[derive(Deserialize)]
struct OAuthError {
    error: String,
}

/// A call that got no answer Discord meant: the budget ran out.
struct Outage;

impl Inner {
    fn token_url(&self) -> String {
        format!("{}/oauth2/token", self.endpoints.api)
    }

    /// The organizer a live session names.
    fn session_user(&self, bearer: &str) -> Option<String> {
        let now = self.clock.now();
        let store = lock(&self.store);
        // Every session is compared, in constant time, whatever matches.
        let mut found = None;
        for s in &store.sessions {
            if s.token.matches(bearer) && now < s.issued_at + SESSION_LIFETIME {
                found = Some(s.discord_user_id.clone());
            }
        }
        found.filter(|user| store.organizers.contains_key(user))
    }

    fn refresh_lock(&self, user: &str) -> Arc<tokio::sync::Mutex<()>> {
        lock(&self.refreshing).entry(user.to_string()).or_default().clone()
    }

    /// Sign the organizer out: the record and every session naming it.
    fn forget(&self, user: &str) {
        let mut store = lock(&self.store);
        store.organizers.remove(user);
        store.sessions.retain(|s| s.discord_user_id != user);
        drop(store);
        lock(&self.refreshing).remove(user);
    }

    /// Who the record names, the access token to present, and whether it is
    /// due for refresh.
    fn access(&self, user: &str) -> Option<(OrganizerId, String, bool)> {
        let now = self.clock.now();
        let store = lock(&self.store);
        let o = store.organizers.get(user)?;
        Some((
            OrganizerId(o.discord_user_id.clone()),
            o.access_token.expose().to_string(),
            o.token_expires_at <= now + REFRESH_MARGIN,
        ))
    }

    async fn check(self: Arc<Self>, user: String) -> Result<OrganizerId, CreateRefusal> {
        let deadline = Instant::now() + self.retry.deadline;
        let refresh = self.refresh_lock(&user);
        let mut refreshed = false;
        {
            let _one_at_a_time = refresh.lock().await;
            let (_, _, due) = self.access(&user).ok_or(CreateRefusal::Denied)?;
            if due {
                self.refresh(&user, deadline).await?;
                refreshed = true;
            }
        }
        loop {
            let (organizer, access, _) = self.access(&user).ok_or(CreateRefusal::Denied)?;
            let url = format!(
                "{}/users/@me/guilds/{}/member",
                self.endpoints.api, self.settings.guild_id
            );
            let r = self
                .call("member", deadline, Req::get(&url, &access))
                .await
                .map_err(|Outage| CreateRefusal::Unavailable)?;
            match r.status {
                200 => {
                    let member = r.json::<Member>().ok_or(CreateRefusal::Unavailable)?;
                    return if member.roles.iter().any(|role| *role == self.settings.role_id) {
                        Ok(organizer)
                    } else {
                        Err(CreateRefusal::WrongRole)
                    };
                }
                404 => return Err(CreateRefusal::WrongServer),
                401 if !refreshed => {
                    // The access token was refused before its expiry (revoked,
                    // or the clock is off): one refresh, unless another check
                    // rotated it meanwhile, then one more try.
                    let _one_at_a_time = refresh.lock().await;
                    if self.access(&user).is_some_and(|(_, now_access, _)| now_access == access) {
                        self.refresh(&user, deadline).await?;
                    }
                    refreshed = true;
                }
                401 => {
                    self.forget(&user);
                    return Err(CreateRefusal::Denied);
                }
                _ => return Err(CreateRefusal::Unavailable),
            }
        }
    }

    /// Refresh the organizer's tokens. The caller holds their refresh lock.
    async fn refresh(&self, user: &str, deadline: Instant) -> Result<(), CreateRefusal> {
        let spent = {
            let store = lock(&self.store);
            let o = store.organizers.get(user).ok_or(CreateRefusal::Denied)?;
            o.refresh_token.expose().to_string()
        };
        let s = &self.settings;
        let body = form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", &spent),
            ("client_id", &s.client_id),
            ("client_secret", s.client_secret.expose()),
        ]);
        let r = self
            .call("refresh", deadline, Req::form(&self.token_url(), body))
            .await
            .map_err(|Outage| CreateRefusal::Unavailable)?;
        match r.status {
            200 => {
                let tokens = r.json::<Tokens>().ok_or(CreateRefusal::Unavailable)?;
                let now = self.clock.now();
                let mut store = lock(&self.store);
                let o = store.organizers.get_mut(user).ok_or(CreateRefusal::Denied)?;
                // The rotation, persisted: the new pair replaces the spent one
                // in one write, under the lock every reader takes.
                o.access_token = Secret::new(tokens.access_token);
                o.refresh_token = Secret::new(tokens.refresh_token);
                o.token_expires_at = now + Duration::from_secs(tokens.expires_in);
                Ok(())
            }
            400 | 401 if r.json::<OAuthError>().is_some_and(|e| e.error == "invalid_grant") => {
                self.forget(user);
                Err(CreateRefusal::Denied)
            }
            _ => Err(CreateRefusal::Unavailable),
        }
    }

    /// One Discord call, retried within the budget.
    async fn call(&self, what: &str, deadline: Instant, req: Req) -> Result<Resp, Outage> {
        let mut backoff = self.retry.first_backoff;
        for attempt in 1..=self.retry.attempts.max(1) {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            let answer = tokio::time::timeout(self.retry.per_attempt.min(left), self.http.send(&req)).await;
            let wait = match answer {
                Ok(Ok(r)) => {
                    self.log.line(&format!("discord: {what} {} (try {attempt})", r.status));
                    if r.status != 429 && r.status < 500 {
                        return Ok(r);
                    }
                    if r.status == 429 { r.retry_after() } else { None }.unwrap_or(backoff)
                }
                Ok(Err(_)) | Err(_) => {
                    self.log.line(&format!("discord: {what} no answer (try {attempt})"));
                    backoff
                }
            };
            if attempt == self.retry.attempts {
                break;
            }
            if Instant::now() + wait >= deadline {
                break;
            }
            tokio::time::sleep(wait).await;
            backoff = backoff.saturating_mul(2);
        }
        Err(Outage)
    }
}

// --------------------------------------------------------------------------
// Encoding.
// --------------------------------------------------------------------------

/// Percent-encode everything but RFC 3986's unreserved characters.
fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `application/x-www-form-urlencoded`.
fn form(pairs: &[(&str, &str)]) -> Vec<u8> {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&")
        .into_bytes()
}

// --------------------------------------------------------------------------
// HTTP/1.1: one request per connection, over TCP or TLS. Enough for three
// Discord routes: Content-Length, chunked or to-EOF bodies, no redirects.
// --------------------------------------------------------------------------

struct Req {
    method: &'static str,
    url: String,
    bearer: Option<Secret>,
    form: Option<Vec<u8>>,
}

impl Req {
    fn get(url: &str, bearer: &str) -> Req {
        Req {
            method: "GET",
            url: url.to_string(),
            bearer: Some(Secret::new(bearer.to_string())),
            form: None,
        }
    }

    fn form(url: &str, body: Vec<u8>) -> Req {
        Req {
            method: "POST",
            url: url.to_string(),
            bearer: None,
            form: Some(body),
        }
    }
}

struct Resp {
    status: u16,
    retry_after: Option<String>,
    body: Vec<u8>,
}

impl Resp {
    fn json<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        serde_json::from_slice(&self.body).ok()
    }

    /// Discord's `Retry-After`, in seconds (fractions allowed), from the header
    /// or else the body's `retry_after`.
    fn retry_after(&self) -> Option<Duration> {
        #[derive(Deserialize)]
        struct Body {
            retry_after: f64,
        }
        let secs = self
            .retry_after
            .as_deref()
            .and_then(|v| v.trim().parse::<f64>().ok())
            .or_else(|| self.json::<Body>().map(|b| b.retry_after))?;
        (secs.is_finite() && secs >= 0.0).then(|| Duration::from_secs_f64(secs.min(3600.0)))
    }
}

trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

struct Client {
    tls: tokio_rustls::TlsConnector,
}

impl Client {
    fn new() -> Client {
        let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let mut config = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .expect("ring supports the default protocol versions")
            .with_root_certificates(roots)
            .with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Client {
            tls: tokio_rustls::TlsConnector::from(Arc::new(config)),
        }
    }

    async fn send(&self, req: &Req) -> std::io::Result<Resp> {
        let bad = |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidInput, why.to_string());
        let (tls, rest) = if let Some(r) = req.url.strip_prefix("https://") {
            (true, r)
        } else if let Some(r) = req.url.strip_prefix("http://") {
            (false, r)
        } else {
            return Err(bad("not an http(s) URL"));
        };
        let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
        let path = if path.is_empty() { "/" } else { path };
        let default_port = if tls { 443 } else { 80 };
        let (host, port) = match authority.strip_prefix('[') {
            // `[::1]:8080`
            Some(v6) => {
                let (h, rest) = v6.split_once(']').ok_or_else(|| bad("bad host"))?;
                match rest.strip_prefix(':') {
                    Some(p) => (h, p.parse::<u16>().map_err(|_| bad("bad port"))?),
                    None => (h, default_port),
                }
            }
            None => match authority.split_once(':') {
                Some((h, p)) => (h, p.parse::<u16>().map_err(|_| bad("bad port"))?),
                None => (authority, default_port),
            },
        };
        let tcp = tokio::net::TcpStream::connect((host, port)).await?;
        tcp.set_nodelay(true)?;
        let io: Box<dyn Io> = if tls {
            let name = rustls::pki_types::ServerName::try_from(host.to_string()).map_err(|_| bad("bad host"))?;
            Box::new(self.tls.connect(name, tcp).await?)
        } else {
            Box::new(tcp)
        };
        let mut conn = BufReader::new(io);

        let mut head = format!(
            "{} {path} HTTP/1.1\r\nHost: {authority}\r\nUser-Agent: popquiz-room (https://popquiz.rustnyc.org, 1)\r\nAccept: application/json\r\nConnection: close\r\n",
            req.method
        );
        if let Some(b) = &req.bearer {
            head.push_str("Authorization: Bearer ");
            head.push_str(b.expose());
            head.push_str("\r\n");
        }
        let body: &[u8] = req.form.as_deref().unwrap_or(&[]);
        if req.form.is_some() {
            head.push_str("Content-Type: application/x-www-form-urlencoded\r\n");
        }
        if req.form.is_some() || req.method != "GET" {
            head.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        head.push_str("\r\n");
        let w = conn.get_mut();
        w.write_all(head.as_bytes()).await?;
        w.write_all(body).await?;
        w.flush().await?;

        let proto = |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, why.to_string());
        let mut line = String::new();
        if conn.read_line(&mut line).await? == 0 {
            return Err(proto("closed before a response"));
        }
        let status: u16 = line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| proto("bad status line"))?;
        let (mut chunked, mut length, mut retry_after) = (false, None, None);
        for _ in 0..200 {
            line.clear();
            conn.read_line(&mut line).await?;
            let l = line.trim_end();
            if l.is_empty() {
                break;
            }
            if let Some((k, v)) = l.split_once(':') {
                let v = v.trim();
                match k.trim().to_ascii_lowercase().as_str() {
                    "transfer-encoding" => chunked = v.to_ascii_lowercase().contains("chunked"),
                    "content-length" => length = Some(v.parse::<usize>().map_err(|_| proto("bad content-length"))?),
                    "retry-after" => retry_after = Some(v.to_string()),
                    _ => {}
                }
            }
        }
        let mut body = Vec::new();
        if chunked {
            read_chunked(&mut conn, &mut body).await?;
        } else if let Some(n) = length {
            if n > MAX_BODY {
                return Err(proto("body too large"));
            }
            body.resize(n, 0);
            conn.read_exact(&mut body).await?;
        } else {
            (&mut conn).take(MAX_BODY as u64 + 1).read_to_end(&mut body).await?;
            if body.len() > MAX_BODY {
                return Err(proto("body too large"));
            }
        }
        Ok(Resp { status, retry_after, body })
    }
}

async fn read_chunked<R: AsyncBufReadExt + Unpin>(r: &mut R, out: &mut Vec<u8>) -> std::io::Result<()> {
    let proto = |why: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, why.to_string());
    let mut line = String::new();
    loop {
        line.clear();
        r.read_line(&mut line).await?;
        let size = usize::from_str_radix(line.trim().split(';').next().unwrap_or(""), 16).map_err(|_| proto("bad chunk size"))?;
        if size == 0 {
            loop {
                line.clear();
                if r.read_line(&mut line).await? == 0 || line.trim().is_empty() {
                    return Ok(());
                }
            }
        }
        if out.len() + size > MAX_BODY {
            return Err(proto("body too large"));
        }
        let start = out.len();
        out.resize(start + size, 0);
        r.read_exact(&mut out[start..]).await?;
        line.clear();
        r.read_line(&mut line).await?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_leaves_unreserved_alone() {
        assert_eq!(encode("identify guilds.members.read"), "identify%20guilds.members.read");
        assert_eq!(encode("http://localhost:3000/auth/discord/callback"), "http%3A%2F%2Flocalhost%3A3000%2Fauth%2Fdiscord%2Fcallback");
        assert_eq!(encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn question_ids() {
        for ok in ["q3", "q3-again", "2026_10_14"] {
            assert!(is_question_id(ok), "{ok}");
        }
        for bad in ["", "q3#x", "q3&state=1", "../q", &"q".repeat(65)] {
            assert!(!is_question_id(bad), "{bad}");
        }
    }

    #[test]
    fn retry_after_reads_the_header_then_the_body() {
        let r = Resp { status: 429, retry_after: Some("0.25".into()), body: b"{}".to_vec() };
        assert_eq!(r.retry_after(), Some(Duration::from_millis(250)));
        let r = Resp { status: 429, retry_after: None, body: br#"{"retry_after": 1.5}"#.to_vec() };
        assert_eq!(r.retry_after(), Some(Duration::from_millis(1500)));
        let r = Resp { status: 429, retry_after: Some("soon".into()), body: Vec::new() };
        assert_eq!(r.retry_after(), None);
    }

    #[tokio::test]
    async fn chunked_bodies() {
        let raw = b"4\r\nWiki\r\n6;x=y\r\npedia \r\n0\r\nTrailer: t\r\n\r\n";
        let mut r = BufReader::new(&raw[..]);
        let mut out = Vec::new();
        read_chunked(&mut r, &mut out).await.unwrap();
        assert_eq!(out, b"Wikipedia ");
    }
}
