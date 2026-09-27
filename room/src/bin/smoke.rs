//! `smoke` (T-09): the deployed room, driven end to end.
//!
//! EVALUATION.md's harness row: *the deployed room driven end to end through
//! all seven phases with mock participants — the pre-checkpoint sanity run.*
//! `just smoke <url> [--participants N]`. The host credential comes from
//! `HOST_DEV_TOKEN` in the environment and never from the command line, so it
//! stays out of shell history and process listings (SPEC §8.2).
//!
//! Against `<url>` (https/wss through the Fly edge, or plain http/ws on
//! loopback), in order:
//!
//! 1. the pages answer: `GET /join`, `GET /host`;
//! 2. *Create a room* with no bearer and with a wrong one is `401` with no body
//!    (AC-64, the stand-in half, on the deployed build); with the token it
//!    creates a room on q3;
//! 3. the join link is `<url>/<code>` and `GET /<code>` is `303 → /join?code=`
//!    (AC-28); half the participants join with the code as given, half with
//!    it typed lower-case with spaces;
//! 4. a wall socket, a host socket and one buzzer socket per participant are
//!    attached, and every phase — idle, live, closed, split, work, reveal,
//!    released — is driven by the host routes and awaited on **every** socket;
//! 5. in `live` each participant answers, a third of them change their minds,
//!    and then every final answer lands inside one 2 s window (the deadline
//!    burst); every write is timed on an already-open connection;
//! 6. after close, a write is refused (`409`, saved answer restated), and the
//!    split's totals equal the final answers exactly;
//! 7. `work` stops at step M-2: one more step is refused;
//! 8. the reveal's arrival on every buzzer is timed;
//! 9. after release a join is refused `already_ended`.
//!
//! **The secrecy scan.** Every frame and every JSON response in `idle` to
//! `work` is scanned for q3's secrets: both beats of `explains`, every
//! `why_tempting`, the resolving step's note, a ✓, and a key named `correct`.
//! At `reveal` the positive control checks that they do arrive (the host's
//! read-aloud, the wall's resolving step), so a scan that saw nothing fails.
//! This is the deployed-room scan the canary's `--url` hook cannot run until
//! T-25 can plant a canary question; it scans the real question instead.
//!
//! **What its numbers are not.** The write and reveal timings are smoke's, from
//! wherever it runs. They are not `burst`'s AC-53/AC-54/AC-41 figures, which
//! are T-21's harness against the deployed room.
//!
//! Exit: `0` every check passed; `1` a check failed; `2` it could not run
//! (arguments, no token, the room unreachable, q3 already run on this machine).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

/// The HC-0 question, as the verifier wrote it — where the secrets come from.
const Q3: &str = include_str!("../../../bank/questions/q3.json");
/// How long any one phase may take to reach every socket.
const WAIT: Duration = Duration::from_secs(20);
/// SPEC §9's deadline burst: the final writes all land inside this window.
const BURST_WINDOW: Duration = Duration::from_secs(2);
const DEFAULT_PARTICIPANTS: usize = 20;
/// The room's capacity (§4.1).
const MAX_PARTICIPANTS: usize = 200;
const PRE_REVEAL: [&str; 5] = ["idle", "live", "closed", "split", "work"];
const LETTERS: [&str; 5] = ["A", "B", "C", "D", "E"];

// --------------------------------------------------------------------------
// Arguments and the target.
// --------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
struct Args {
    url: String,
    participants: usize,
}

const USAGE: &str = "usage: smoke --url <http(s)://host[:port]> [--participants N]   (HOST_DEV_TOKEN in the environment)";

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut url = None;
    let mut participants = DEFAULT_PARTICIPANTS;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--url" => url = Some(it.next().ok_or("--url needs a value")?),
            "--participants" => {
                let n = it.next().ok_or("--participants needs a value")?;
                participants = n
                    .parse()
                    .ok()
                    .filter(|n| (1..=MAX_PARTICIPANTS).contains(n))
                    .ok_or(format!("--participants must be 1 to {MAX_PARTICIPANTS}, not {n:?}"))?;
            }
            "-h" | "--help" => return Err(USAGE.into()),
            other => return Err(format!("unknown argument {other:?}\n{USAGE}")),
        }
    }
    Ok(Args {
        url: url.ok_or(USAGE)?,
        participants,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct Target {
    tls: bool,
    host: String,
    port: u16,
    /// `scheme://host[:port]`, exactly as a join link starts.
    base: String,
}

impl Target {
    fn parse(url: &str) -> Result<Target, String> {
        let url = url.trim().trim_end_matches('/');
        let (tls, rest) = if let Some(r) = url.strip_prefix("https://") {
            (true, r)
        } else if let Some(r) = url.strip_prefix("http://") {
            (false, r)
        } else {
            return Err(format!("{url:?}: the URL must start with http:// or https://"));
        };
        if rest.is_empty() || rest.contains(['/', '?', '#', '@']) {
            return Err(format!("{url:?}: give the room's scheme and host only"));
        }
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) if !h.ends_with(']') || rest.starts_with('[') => {
                (h.to_string(), p.parse().map_err(|_| format!("{url:?}: bad port"))?)
            }
            _ => (rest.to_string(), if tls { 443 } else { 80 }),
        };
        Ok(Target { tls, host, port, base: url.to_string() })
    }

    fn host_header(&self) -> String {
        match (self.tls, self.port) {
            (true, 443) | (false, 80) => self.host.clone(),
            _ => format!("{}:{}", self.host, self.port),
        }
    }

    fn ws_url(&self, path: &str) -> String {
        format!("{}://{}:{}{path}", if self.tls { "wss" } else { "ws" }, self.host, self.port)
    }

    fn connect_host(&self) -> &str {
        self.host.trim_start_matches('[').trim_end_matches(']')
    }
}

// --------------------------------------------------------------------------
// HTTP/1.1, kept alive, over TCP or TLS. Just enough for the room's routes:
// Content-Length or chunked bodies, no redirects followed.
// --------------------------------------------------------------------------

trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

struct Http {
    target: Arc<Target>,
    tls: Option<tokio_rustls::TlsConnector>,
    conn: Option<BufReader<Box<dyn Io>>>,
}

struct Resp {
    status: u16,
    location: Option<String>,
    body: Vec<u8>,
}

impl Resp {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or(Value::Null)
    }
}

impl Http {
    fn new(target: &Arc<Target>, tls: &Option<tokio_rustls::TlsConnector>) -> Http {
        Http {
            target: target.clone(),
            tls: tls.clone(),
            conn: None,
        }
    }

    async fn open(&mut self) -> Result<(), String> {
        let t = &self.target;
        let tcp = TcpStream::connect((t.connect_host(), t.port))
            .await
            .map_err(|e| format!("connect {}:{}: {e}", t.host, t.port))?;
        tcp.set_nodelay(true).map_err(|e| e.to_string())?;
        let io: Box<dyn Io> = match &self.tls {
            None => Box::new(tcp),
            Some(connector) => {
                let name = rustls::pki_types::ServerName::try_from(t.connect_host().to_string())
                    .map_err(|e| format!("{}: {e}", t.host))?;
                Box::new(connector.connect(name, tcp).await.map_err(|e| format!("TLS to {}: {e}", t.host))?)
            }
        };
        self.conn = Some(BufReader::new(io));
        Ok(())
    }

    async fn call(&mut self, method: &str, path: &str, bearer: Option<&str>, body: Option<&Value>) -> Result<Resp, String> {
        if self.conn.is_none() {
            self.open().await?;
        }
        let result = self.exchange(method, path, bearer, body).await;
        if result.is_err() {
            self.conn = None;
        }
        result.map_err(|e| format!("{method} {path}: {e}"))
    }

    async fn exchange(&mut self, method: &str, path: &str, bearer: Option<&str>, body: Option<&Value>) -> Result<Resp, String> {
        let mut req = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nUser-Agent: popquiz-smoke\r\nAccept: */*\r\n",
            self.target.host_header()
        );
        if let Some(b) = bearer {
            req.push_str(&format!("Authorization: Bearer {b}\r\n"));
        }
        let payload = body.map(|v| v.to_string()).unwrap_or_default();
        if body.is_some() {
            req.push_str("Content-Type: application/json\r\n");
        }
        if body.is_some() || method != "GET" {
            req.push_str(&format!("Content-Length: {}\r\n", payload.len()));
        }
        req.push_str("\r\n");
        req.push_str(&payload);

        let conn = self.conn.as_mut().expect("opened");
        conn.get_mut().write_all(req.as_bytes()).await.map_err(|e| e.to_string())?;
        conn.get_mut().flush().await.map_err(|e| e.to_string())?;

        let mut line = String::new();
        if conn.read_line(&mut line).await.map_err(|e| e.to_string())? == 0 {
            return Err("the connection closed before a response".into());
        }
        let status: u16 = line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .ok_or(format!("bad status line {line:?}"))?;
        let mut headers = HashMap::new();
        loop {
            line.clear();
            conn.read_line(&mut line).await.map_err(|e| e.to_string())?;
            let l = line.trim_end();
            if l.is_empty() {
                break;
            }
            if let Some((k, v)) = l.split_once(':') {
                headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
            }
        }
        let mut body = Vec::new();
        let chunked = headers
            .get("transfer-encoding")
            .is_some_and(|v| v.to_ascii_lowercase().contains("chunked"));
        if chunked {
            read_chunked(conn, &mut body).await?;
        } else if let Some(n) = headers.get("content-length") {
            let n: usize = n.parse().map_err(|_| format!("bad content-length {n:?}"))?;
            body.resize(n, 0);
            conn.read_exact(&mut body).await.map_err(|e| e.to_string())?;
        } else if !(status == 204 || status == 304 || (100..200).contains(&status)) {
            conn.read_to_end(&mut body).await.map_err(|e| e.to_string())?;
            self.conn = None;
        }
        if headers.get("connection").is_some_and(|v| v.eq_ignore_ascii_case("close")) {
            self.conn = None;
        }
        Ok(Resp {
            status,
            location: headers.remove("location"),
            body,
        })
    }
}

async fn read_chunked<R: AsyncBufReadExt + Unpin>(r: &mut R, out: &mut Vec<u8>) -> Result<(), String> {
    let mut line = String::new();
    loop {
        line.clear();
        r.read_line(&mut line).await.map_err(|e| e.to_string())?;
        let size_hex = line.trim().split(';').next().unwrap_or("");
        let size = usize::from_str_radix(size_hex, 16).map_err(|_| format!("bad chunk size {line:?}"))?;
        if size == 0 {
            // Trailers, if any, end at a blank line.
            loop {
                line.clear();
                if r.read_line(&mut line).await.map_err(|e| e.to_string())? == 0 || line.trim().is_empty() {
                    return Ok(());
                }
            }
        }
        let start = out.len();
        out.resize(start + size, 0);
        r.read_exact(&mut out[start..]).await.map_err(|e| e.to_string())?;
        let mut crlf = [0u8; 2];
        r.read_exact(&mut crlf).await.map_err(|e| e.to_string())?;
    }
}

// --------------------------------------------------------------------------
// The secrets, and the scan.
// --------------------------------------------------------------------------

struct Secrets {
    texts: Vec<(String, String)>,
}

impl Secrets {
    fn from_record(q: &Value) -> Secrets {
        let mut texts = vec![
            ("explains.what".to_string(), q["explains"]["what"].as_str().unwrap_or_default().to_string()),
            ("explains.takeaway".to_string(), q["explains"]["takeaway"].as_str().unwrap_or_default().to_string()),
            ("check mark".to_string(), "✓".to_string()),
        ];
        for (i, o) in q["options"].as_array().into_iter().flatten().enumerate() {
            if let Some(w) = o["why_tempting"].as_str() {
                texts.push((format!("why_tempting {}", LETTERS[i]), w.to_string()));
            }
        }
        if let Some(last) = q["trace"]["steps"].as_array().and_then(|s| s.last()) {
            texts.push(("the resolving note".to_string(), last["note"].as_str().unwrap_or_default().to_string()));
        }
        texts.retain(|(_, t)| !t.is_empty());
        Secrets { texts }
    }

    /// The names of every secret `v` carries, and `key correct` if any object
    /// has that key.
    fn found_in(&self, v: &Value) -> Vec<String> {
        let mut strings = Vec::new();
        let mut keys = false;
        walk(v, &mut strings, &mut keys);
        let mut hits: Vec<String> = self
            .texts
            .iter()
            .filter(|(_, t)| strings.iter().any(|s| s.contains(t.as_str())))
            .map(|(name, _)| name.clone())
            .collect();
        if keys {
            hits.push("key correct".into());
        }
        hits
    }

    fn get(&self, name: &str) -> &str {
        &self.texts.iter().find(|(n, _)| n == name).expect("a named secret").1
    }
}

fn walk<'a>(v: &'a Value, strings: &mut Vec<&'a str>, correct_key: &mut bool) {
    match v {
        Value::String(s) => strings.push(s),
        Value::Array(items) => items.iter().for_each(|i| walk(i, strings, correct_key)),
        Value::Object(map) => {
            for (k, child) in map {
                *correct_key |= k == "correct";
                strings.push(k);
                walk(child, strings, correct_key);
            }
        }
        _ => {}
    }
}

/// What every check found wrong, and how much was scanned.
#[derive(Default)]
struct Findings {
    failures: Vec<String>,
    scanned: usize,
}

type Shared = Arc<Mutex<Findings>>;

fn fail(f: &Shared, msg: impl Into<String>) {
    let msg = msg.into();
    eprintln!("  FAIL {msg}");
    f.lock().unwrap().failures.push(msg);
}

fn check(f: &Shared, ok: bool, msg: impl Into<String>) {
    if !ok {
        fail(f, msg);
    }
}

/// Scan a payload that belongs to `phase`; only pre-reveal phases are held to
/// the rule.
fn scan(f: &Shared, secrets: &Secrets, phase: &str, surface: &str, v: &Value) {
    if !PRE_REVEAL.contains(&phase) {
        return;
    }
    f.lock().unwrap().scanned += 1;
    let hits = secrets.found_in(v);
    if !hits.is_empty() {
        fail(f, format!("LEAK: {surface} in {phase} carries {}", hits.join(", ")));
    }
}

// --------------------------------------------------------------------------
// Sockets. Each is read by its own task, which notes when each phase first
// arrived, keeps the last frame per phase, and scans every frame.
// --------------------------------------------------------------------------

type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

#[derive(Default)]
struct Seen {
    first: HashMap<String, Instant>,
    last: HashMap<String, Value>,
    frames: usize,
    closed: bool,
}

struct Watcher {
    seen: Arc<Mutex<Seen>>,
    task: tokio::task::JoinHandle<()>,
}

impl Watcher {
    fn arrived(&self, phase: &str) -> Option<Instant> {
        self.seen.lock().unwrap().first.get(phase).copied()
    }

    fn last(&self, phase: &str) -> Option<Value> {
        self.seen.lock().unwrap().last.get(phase).cloned()
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn open_ws(target: &Target, path: &str, attach: Option<&str>) -> Result<Ws, String> {
    let tcp = TcpStream::connect((target.connect_host(), target.port))
        .await
        .map_err(|e| format!("connect for {path}: {e}"))?;
    tcp.set_nodelay(true).map_err(|e| e.to_string())?;
    let (mut ws, _) = tokio_tungstenite::client_async_tls_with_config(target.ws_url(path), tcp, None, None)
        .await
        .map_err(|e| format!("socket {path}: {e}"))?;
    if let Some(token) = attach {
        let msg = json!({ "t": "attach", "token": token }).to_string();
        ws.send(Message::Text(msg.into())).await.map_err(|e| format!("attach {path}: {e}"))?;
    }
    Ok(ws)
}

fn watch(mut ws: Ws, surface: String, secrets: Arc<Secrets>, findings: Shared) -> Watcher {
    let seen = Arc::new(Mutex::new(Seen::default()));
    let mine = seen.clone();
    let task = tokio::spawn(async move {
        while let Some(msg) = ws.next().await {
            let Ok(Message::Text(text)) = msg else {
                if matches!(msg, Ok(Message::Close(_)) | Err(_)) {
                    break;
                }
                continue;
            };
            let at = Instant::now();
            let Ok(frame) = serde_json::from_str::<Value>(&text) else {
                fail(&findings, format!("{surface}: a frame that is not JSON"));
                continue;
            };
            let phase = frame["phase"].as_str().unwrap_or("").to_string();
            scan(&findings, &secrets, &phase, &surface, &frame);
            let mut s = mine.lock().unwrap();
            s.frames += 1;
            s.first.entry(phase.clone()).or_insert(at);
            s.last.insert(phase, frame);
        }
        mine.lock().unwrap().closed = true;
    });
    Watcher { seen, task }
}

/// Wait until every watcher has seen `phase`; the arrival instants, in order.
async fn all_reach(watchers: &[&Watcher], phase: &str) -> Result<Vec<Instant>, String> {
    let deadline = Instant::now() + WAIT;
    loop {
        let arrived: Vec<Option<Instant>> = watchers.iter().map(|w| w.arrived(phase)).collect();
        if arrived.iter().all(Option::is_some) {
            return Ok(arrived.into_iter().flatten().collect());
        }
        if Instant::now() > deadline {
            let missing = arrived.iter().filter(|a| a.is_none()).count();
            let closed = watchers.iter().filter(|w| w.seen.lock().unwrap().closed).count();
            return Err(format!(
                "{missing} of {} sockets never saw `{phase}` within {}s ({closed} closed)",
                watchers.len(),
                WAIT.as_secs()
            ));
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

// --------------------------------------------------------------------------
// Numbers.
// --------------------------------------------------------------------------

/// The nearest-rank percentile, in milliseconds.
fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return f64::NAN;
    }
    let rank = ((p / 100.0) * sorted_ms.len() as f64).ceil().max(1.0) as usize;
    sorted_ms[rank.min(sorted_ms.len()) - 1]
}

fn summary(name: &str, mut ms: Vec<f64>) -> String {
    ms.sort_by(f64::total_cmp);
    format!(
        "{name}: n={} p50={:.1}ms p95={:.1}ms max={:.1}ms",
        ms.len(),
        percentile(&ms, 50.0),
        percentile(&ms, 95.0),
        ms.last().copied().unwrap_or(f64::NAN)
    )
}

fn millis(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// xorshift64*: enough to scatter letters and burst offsets; the seed is printed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

// --------------------------------------------------------------------------
// The run.
// --------------------------------------------------------------------------

struct Participant {
    http: Http,
    token: String,
    watcher: Watcher,
    fin: usize,
}

struct Ctx {
    target: Arc<Target>,
    tls: Option<tokio_rustls::TlsConnector>,
    secrets: Arc<Secrets>,
    findings: Shared,
}

impl Ctx {
    fn http(&self) -> Http {
        Http::new(&self.target, &self.tls)
    }
}

/// Every socket: the wall, the host and each buzzer.
fn everyone<'a>(wall: &'a Watcher, host: &'a Watcher, people: &'a [Participant]) -> Vec<&'a Watcher> {
    let mut all = vec![wall, host];
    all.extend(people.iter().map(|p| &p.watcher));
    all
}

/// A host action; its response is the host payload, scanned under the phase
/// it names.
async fn act(ctx: &Ctx, host: &mut Http, room: &str, session: &str, slug: &str) -> Result<Value, String> {
    let r = host.call("POST", &format!("/rooms/{room}/{slug}"), Some(session), None).await?;
    let body = r.json();
    if r.status != 200 {
        return Err(format!("{slug}: {} {body}", r.status));
    }
    let phase = body["phase"].as_str().unwrap_or("").to_string();
    scan(&ctx.findings, &ctx.secrets, &phase, &format!("POST {slug}"), &body);
    Ok(body)
}

/// The pages answer. The first request also waits out a stopped machine's
/// start (Fly starts it on the first request after the trial's stop).
async fn pages_answer(ctx: &Ctx) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let mut http = ctx.http();
        match http.call("GET", "/join", None, None).await {
            Ok(r) if r.status == 200 => break,
            Ok(r) if Instant::now() < deadline && r.status >= 500 => {}
            Ok(r) => return Err(format!("GET /join: {}", r.status)),
            Err(_) if Instant::now() < deadline => {}
            Err(e) => return Err(e),
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    let mut http = ctx.http();
    for page in ["/join", "/host"] {
        let r = http.call("GET", page, None, None).await?;
        check(&ctx.findings, r.status == 200, format!("GET {page}: {}", r.status));
        let html = String::from_utf8_lossy(&r.body);
        check(&ctx.findings, html.contains("<html") || html.contains("<!doctype") || html.contains("<!DOCTYPE"), format!("GET {page}: not a page"));
    }
    Ok(())
}

async fn run(args: Args, token: String) -> Result<Vec<String>, String> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let target = Arc::new(Target::parse(&args.url)?);
    let tls = target.tls.then(|| {
        let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let mut config = rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        tokio_rustls::TlsConnector::from(Arc::new(config))
    });
    let q3: Value = serde_json::from_str(Q3).map_err(|e| format!("q3.json: {e}"))?;
    let ctx = Ctx {
        target: target.clone(),
        tls,
        secrets: Arc::new(Secrets::from_record(&q3)),
        findings: Shared::default(),
    };
    let f = &ctx.findings;
    let n = args.participants;
    let seed = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) | 1;
    let mut rng = Rng(seed);
    let mut report = vec![format!("smoke {} with {n} participants (seed {seed})", target.base)];
    let started = Instant::now();

    println!("· the pages");
    pages_answer(&ctx).await?;

    println!("· Create a room: refused without the token, made with it (AC-64)");
    let mut host_http = ctx.http();
    let create = json!({ "question_id": "q3" });
    for (label, bearer) in [("no bearer", None), ("a wrong bearer", Some("not-the-host-token"))] {
        let r = host_http.call("POST", "/rooms", bearer, Some(&create)).await?;
        check(f, r.status == 401, format!("create with {label}: {} (want 401)", r.status));
        check(f, r.body.is_empty(), format!("create with {label}: the refusal has a body"));
    }
    let r = host_http.call("POST", "/rooms", Some(&token), Some(&create)).await?;
    let created = r.json();
    match r.status {
        201 => {}
        401 => return Err("create with HOST_DEV_TOKEN: 401 — the token is not the one the room was deployed with".into()),
        409 => {
            return Err(format!(
                "create: 409 {} — q3 has already been run on this machine; restart it (fly apps restart) and run smoke again",
                created["reason"]
            ))
        }
        s => return Err(format!("create: {s} {created}")),
    }
    scan(f, &ctx.secrets, "idle", "POST /rooms", &created);
    let field = |k: &str| created[k].as_str().map(str::to_string).ok_or(format!("create: no {k} in the response"));
    let (room, code, session) = (field("id")?, field("code")?, field("host_session")?);

    println!("· the join link carries the host and the code (AC-28)");
    let join_url = field("join_url")?;
    check(f, join_url == format!("{}/{code}", target.base), format!("join_url is {join_url}, not {}/{code}", target.base));
    let r = host_http.call("GET", &format!("/{code}"), None, None).await?;
    check(f, r.status == 303, format!("GET /{code}: {} (want 303)", r.status));
    check(
        f,
        r.location.as_deref() == Some(&format!("/join?code={code}")),
        format!("GET /{code} goes to {:?}", r.location),
    );

    println!("· the wall and the host attach");
    let wall = watch(open_ws(&target, &format!("/rooms/{room}/ws/wall"), None).await?, "wall".into(), ctx.secrets.clone(), f.clone());
    let host = watch(open_ws(&target, &format!("/rooms/{room}/ws/host"), Some(&session)).await?, "host".into(), ctx.secrets.clone(), f.clone());

    println!("· {n} participants join and attach");
    let t = Instant::now();
    let joins = (0..n).map(|i| {
        let mut http = ctx.http();
        let (target, secrets, findings) = (target.clone(), ctx.secrets.clone(), f.clone());
        let (code, room) = (code.clone(), room.clone());
        tokio::spawn(async move {
            // Half with the code as the short link carries it, half as typed.
            let given = if i % 2 == 0 { code.clone() } else { format!("  {} ", code.to_lowercase()) };
            let r = http.call("POST", "/join", None, Some(&json!({ "code": given }))).await?;
            let body = r.json();
            if r.status != 201 {
                return Err(format!("join {i} with {given:?}: {} {body}", r.status));
            }
            scan(&findings, &secrets, "idle", "POST /join", &body);
            if body["room_id"] != room.as_str() {
                return Err(format!("join {i}: room_id {}", body["room_id"]));
            }
            let token = body["token"].as_str().ok_or(format!("join {i}: no token"))?.to_string();
            let ws = open_ws(&target, &format!("/rooms/{room}/ws/buzzer"), Some(&token)).await?;
            let watcher = watch(ws, format!("buzzer {i}"), secrets, findings);
            Ok(Participant { http, token, watcher, fin: 0 })
        })
    });
    let mut people = Vec::with_capacity(n);
    for j in futures_util::future::join_all(joins).await {
        people.push(j.map_err(|e| e.to_string())??);
    }
    report.push(format!("joined and attached {n} in {:.0}ms", millis(t.elapsed())));

    all_reach(&everyone(&wall, &host, &people), "idle").await?;
    let deadline = Instant::now() + WAIT;
    loop {
        let present = host.last("idle").map(|h| h["present"].clone());
        if present == Some(json!(n)) {
            break;
        }
        if Instant::now() > deadline {
            fail(f, format!("the host's idle count is {present:?}, not {n}"));
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    println!("· live");
    act(&ctx, &mut host_http, &room, &session, "put-on-screen").await?;
    all_reach(&everyone(&wall, &host, &people), "live").await?;

    println!("· answers, changed minds, then the deadline burst");
    for p in people.iter_mut() {
        p.fin = rng.below(5) as usize;
    }
    let plans: Vec<(usize, Option<usize>, Duration)> = people
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let first = if i % 3 == 0 { Some((p.fin + 1 + rng.below(4) as usize) % 5) } else { None };
            (p.fin, first, Duration::from_millis(rng.below(BURST_WINDOW.as_millis() as u64)))
        })
        .collect();
    let burst_start = Instant::now() + Duration::from_millis(500);
    let writes = people.into_iter().zip(plans).map(|(mut p, (fin, first, offset))| {
        let room = room.clone();
        tokio::spawn(async move {
            let mut lat = Vec::new();
            let mut errs = Vec::new();
            let path = format!("/rooms/{room}/answer");
            let mut write = async |http: &mut Http, token: &str, letter: usize| -> Result<(), String> {
                let t = Instant::now();
                let r = http.call("PUT", &path, Some(token), Some(&json!({ "letter": LETTERS[letter] }))).await?;
                let took = t.elapsed();
                let body = r.json();
                if r.status != 200 || body["saved"] != LETTERS[letter] {
                    return Err(format!("PUT answer {}: {} {body}", LETTERS[letter], r.status));
                }
                lat.push(millis(took));
                Ok(())
            };
            if let Some(first) = first {
                if let Err(e) = write(&mut p.http, &p.token, first).await {
                    errs.push(e);
                }
            }
            tokio::time::sleep_until((burst_start + offset).into()).await;
            if let Err(e) = write(&mut p.http, &p.token, fin).await {
                errs.push(e);
            }
            (p, lat, errs)
        })
    });
    let mut people = Vec::with_capacity(n);
    let mut latencies = Vec::new();
    let mut expected = [0u32; 5];
    for w in futures_util::future::join_all(writes).await {
        let (p, lat, errs) = w.map_err(|e| e.to_string())?;
        errs.into_iter().for_each(|e| fail(f, e));
        latencies.extend(lat);
        expected[p.fin] += 1;
        people.push(p);
    }
    let burst_took = burst_start.elapsed();
    report.push(summary("answer writes (kept-alive connection, incl. the burst)", latencies));
    report.push(format!("burst: {n} final writes scheduled inside {}ms; done {:.0}ms after it opened", BURST_WINDOW.as_millis(), millis(burst_took)));

    println!("· closed");
    act(&ctx, &mut host_http, &room, &session, "close-answers").await?;
    all_reach(&everyone(&wall, &host, &people), "closed").await?;
    let p0 = &mut people[0];
    let letter = LETTERS[(p0.fin + 1) % 5];
    let r = p0.http.call("PUT", &format!("/rooms/{room}/answer"), Some(&p0.token), Some(&json!({ "letter": letter }))).await?;
    let body = r.json();
    scan(f, &ctx.secrets, "closed", "PUT answer after close", &body);
    check(f, r.status == 409, format!("a write after close: {} (want 409)", r.status));
    check(f, body["saved"] == LETTERS[p0.fin], format!("the refused write restates {} (want {})", body["saved"], LETTERS[p0.fin]));

    println!("· split: the totals are the final answers, exactly");
    act(&ctx, &mut host_http, &room, &session, "show-split").await?;
    all_reach(&everyone(&wall, &host, &people), "split").await?;
    let mut disagree = 0;
    for p in &people {
        let frame = p.watcher.last("split").unwrap_or(Value::Null);
        if frame["counts"]["totals"] != json!(expected) || frame["counts"]["answered"] != json!(n) {
            disagree += 1;
        }
    }
    check(f, disagree == 0, format!("{disagree} buzzers' split totals differ from the final answers {expected:?}"));
    report.push(format!("split totals {expected:?}, answered {n}: {}", if disagree == 0 { "exact on every buzzer" } else { "MISMATCH" }));

    println!("· work stops at M-2");
    let body = act(&ctx, &mut host_http, &room, &session, "walk-it").await?;
    all_reach(&everyone(&wall, &host, &people), "work").await?;
    let m = body["step"]["m"].as_u64().ok_or(format!("work: no step.m in {body}"))?;
    let mut at = body["step"]["at"].as_u64().unwrap_or(u64::MAX);
    check(f, at == 0, format!("work enters at step {at}, not 0"));
    while at + 2 < m {
        at = act(&ctx, &mut host_http, &room, &session, "step-forward").await?["step"]["at"].as_u64().unwrap_or(u64::MAX);
    }
    check(f, at + 2 == m, format!("work reached step {at} of M={m}"));
    let r = host_http.call("POST", &format!("/rooms/{room}/step-forward"), Some(&session), None).await?;
    scan(f, &ctx.secrets, "work", "POST step-forward (refused)", &r.json());
    check(f, r.status == 409, format!("a step past M-2 in work: {} (want 409)", r.status));

    println!("· reveal");
    let t0 = Instant::now();
    act(&ctx, &mut host_http, &room, &session, "reveal").await?;
    let arrivals = all_reach(&people.iter().map(|p| &p.watcher).collect::<Vec<_>>(), "reveal").await?;
    all_reach(&[&wall, &host], "reveal").await?;
    report.push(summary("reveal reaching every buzzer (from the host's POST)", arrivals.iter().map(|a| millis(a.duration_since(t0))).collect()));
    // The positive control: the secrets do reach the surfaces that may show them.
    let s = &ctx.secrets;
    let host_reveal = host.last("reveal").unwrap_or(Value::Null);
    let wall_reveal = wall.last("reveal").unwrap_or(Value::Null);
    check(f, s.found_in(&host_reveal).contains(&"explains.what".to_string()), "reveal: the host's read-aloud does not carry the explanation — the scan may be blind");
    check(f, s.found_in(&wall_reveal).contains(&"the resolving note".to_string()), "reveal: the wall does not carry the resolving step — the scan may be blind");
    // Across every reveal surface: each kind of secret the scan looks for shows
    // up at least once, so none of its pre-reveal checks can pass by being blind.
    // Only the most-chosen incorrect option's why_tempting is read aloud (§4.5),
    // so one why_tempting is what reveal owes.
    let mut surfaced: Vec<String> = s.found_in(&host_reveal);
    surfaced.extend(s.found_in(&wall_reveal));
    for p in &people {
        surfaced.extend(s.found_in(&p.watcher.last("reveal").unwrap_or(Value::Null)));
    }
    for want in ["explains.what", "explains.takeaway", "the resolving note", "check mark", "key correct"] {
        check(f, surfaced.iter().any(|h| h == want), format!("reveal: no surface carries {want} — the scan may be blind to it"));
    }
    check(f, surfaced.iter().any(|h| h.starts_with("why_tempting")), "reveal: no surface carries any why_tempting — the scan may be blind to it");
    let correct = people[0].watcher.last("reveal").map(|b| b["correct"].clone()).unwrap_or(Value::Null);
    check(f, correct.is_string(), "reveal: the buzzer names no correct letter");
    check(f, !s.get("the resolving note").is_empty(), "q3 has no resolving note to scan for");

    println!("· released");
    act(&ctx, &mut host_http, &room, &session, "release").await?;
    all_reach(&everyone(&wall, &host, &people), "released").await?;
    let r = host_http.call("POST", "/join", None, Some(&json!({ "code": code }))).await?;
    check(f, r.status == 409 && r.json()["refusal"] == "already_ended", format!("a join after release: {} {}", r.status, r.json()));

    let findings = f.lock().unwrap();
    report.push(format!(
        "secrecy scan: {} pre-reveal frames and responses, {} leaks",
        findings.scanned,
        findings.failures.iter().filter(|m| m.starts_with("LEAK")).count()
    ));
    report.push(format!("all seven phases on {} sockets in {:.1}s", n + 2, started.elapsed().as_secs_f64()));
    if findings.failures.is_empty() {
        Ok(report)
    } else {
        report.push(format!("{} checks failed:", findings.failures.len()));
        report.extend(findings.failures.iter().map(|m| format!("  - {m}")));
        Err(report.join("\n"))
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::process::ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            return std::process::ExitCode::from(2);
        }
    };
    let Some(token) = std::env::var("HOST_DEV_TOKEN").ok().filter(|t| !t.is_empty()) else {
        eprintln!("smoke: HOST_DEV_TOKEN must be set in the environment (never on the command line)");
        return std::process::ExitCode::from(2);
    };
    match run(args, token).await {
        Ok(report) => {
            println!("\nSMOKE PASS");
            report.iter().for_each(|l| println!("  {l}"));
            std::process::ExitCode::SUCCESS
        }
        Err(e) if e.contains("checks failed") => {
            println!("\nSMOKE FAIL\n{e}");
            std::process::ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("\nSMOKE COULD NOT RUN: {e}");
            std::process::ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Result<Args, String> {
        parse_args(s.split_whitespace().map(String::from))
    }

    #[test]
    fn arguments() {
        assert_eq!(args("--url https://x.fly.dev").unwrap(), Args { url: "https://x.fly.dev".into(), participants: 20 });
        assert_eq!(args("--url http://127.0.0.1:3000 --participants 200").unwrap().participants, 200);
        for bad in ["", "--participants 5", "--url", "--url u --participants 0", "--url u --participants 201", "--url u --token t"] {
            assert!(args(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn targets() {
        let t = Target::parse("https://rustnyc-popquiz.fly.dev/").unwrap();
        assert_eq!((t.tls, t.host.as_str(), t.port, t.base.as_str()), (true, "rustnyc-popquiz.fly.dev", 443, "https://rustnyc-popquiz.fly.dev"));
        assert_eq!(t.host_header(), "rustnyc-popquiz.fly.dev");
        assert_eq!(t.ws_url("/rooms/a/ws/wall"), "wss://rustnyc-popquiz.fly.dev:443/rooms/a/ws/wall");
        let t = Target::parse("http://127.0.0.1:3000").unwrap();
        assert_eq!((t.tls, t.port, t.host_header().as_str()), (false, 3000, "127.0.0.1:3000"));
        assert_eq!(Target::parse("http://[::1]:8080").unwrap().connect_host(), "::1");
        for bad in ["x.fly.dev", "https://x.fly.dev/path", "https://", "ws://x", "http://h:port"] {
            assert!(Target::parse(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn nearest_rank() {
        let v: Vec<f64> = (1..=200).map(f64::from).collect();
        assert_eq!(percentile(&v, 95.0), 190.0);
        assert_eq!(percentile(&v, 50.0), 100.0);
        assert_eq!(percentile(&[7.0], 95.0), 7.0);
    }

    #[test]
    fn the_scan_finds_each_secret_and_only_those() {
        let q: Value = serde_json::from_str(Q3).unwrap();
        let s = Secrets::from_record(&q);
        // explains ×2, the check mark, four why_tempting, the resolving note.
        assert_eq!(s.texts.len(), 8, "{:?}", s.texts.iter().map(|t| &t.0).collect::<Vec<_>>());
        for (name, text) in &s.texts {
            let planted = json!({ "phase": "live", "deep": [{ "x": format!("…{text}…") }] });
            assert_eq!(s.found_in(&planted), vec![name.clone()]);
        }
        assert_eq!(s.found_in(&json!({ "correct": "E" })), vec!["key correct".to_string()]);
        let clean = json!({ "phase": "live", "source": q["source"], "options": q["options"].as_array().unwrap().iter().map(|o| &o["text"]).collect::<Vec<_>>() });
        assert!(s.found_in(&clean).is_empty());
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
