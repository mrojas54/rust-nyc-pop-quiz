//! The room's client side, shared by `smoke` and `burst` (T-21): a target URL,
//! kept-alive HTTP/1.1 over TCP or TLS, and sockets watched by their own tasks.
//!
//! Not a binary (`autobins = false`); each harness includes it with
//! `#[path = "room_client.rs"] mod client;`. It was `smoke.rs`'s own driver
//! (T-09) and moved here unchanged when `burst` came to speak the room's
//! protocol, so the two harnesses cannot drift apart on how they reach it.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

/// The room's capacity (§4.1).
pub const MAX_PARTICIPANTS: usize = 200;
pub const LETTERS: [&str; 5] = ["A", "B", "C", "D", "E"];

/// How long a TCP connect, a TLS handshake or a socket's upgrade may take.
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// How long one request may wait for its whole response. A write the room
/// never answers ends here, as an error, rather than hanging the run.
pub const READ_TIMEOUT: Duration = Duration::from_secs(30);

// --------------------------------------------------------------------------
// Which question a harness may run.
// --------------------------------------------------------------------------

/// The flag that lets a harness create a room on a bank question off loopback.
/// Its release retires that question in the machine's ledger, and from there
/// in `/admin/used` and the bank (the false retirement CLAUDE.md records).
pub const SPEND_BANK_FLAG: &str = "--spend-bank-question";

/// A harness id: q3's record scheduled under a name no bank question has
/// (`burst-q3`, `smoke-q3`), so releasing it retires nothing.
pub fn is_harness_id(id: &str) -> bool {
    ["burst-", "smoke-"].iter().any(|p| id.len() > p.len() && id.starts_with(p))
}

/// Refuse a bank question off loopback unless [`SPEND_BANK_FLAG`] was given.
/// Any id that is not a harness id counts as a bank question here: the bank
/// grows, and the safe guess about an unknown id is that someone needs it.
pub fn question_allowed(target: &Target, id: &str, spend_bank: bool) -> Result<(), String> {
    if target.is_loopback() || is_harness_id(id) || spend_bank {
        return Ok(());
    }
    Err(format!(
        "{id:?} is not a harness id, and {} is not this machine: the run would release it and retire a bank question on the deployed room. Schedule q3's record under a harness id (burst-…, smoke-…; room/README.md, Burst) and pass that with --question, or pass {SPEND_BANK_FLAG} if spending {id} is what you mean.",
        target.base
    ))
}

/// A `409` on *Create a room*: the room's reason, verbatim, and what to do
/// about that reason. The three reasons are the room's own copy
/// (`rooms.rs`, `create_for`). None of the hints is a restart: a restart
/// wipes every room on the machine, the night's included.
pub fn create_refused(reason: &str, id: &str) -> String {
    let hint = if reason.contains("open in another room") {
        format!(
            "another room on this machine holds {id} and has not been released. Wait for it to go quiet (30 minutes with no host action before it starts, 20 once started, 4 hours at most; docs/RUNBOOK.md, the failure table), or schedule q3's record under another harness id and pass that with --question"
        )
    } else if reason.contains("already been run") {
        format!("{id} has been run on this machine. Schedule q3's record under another harness id (room/README.md, Burst, Scheduling) and pass that with --question")
    } else if reason.contains("No question is scheduled") {
        format!("schedule q3's record as {id} over the pipeline channel first (room/README.md, Burst, Scheduling)")
    } else {
        "the room gave a reason this harness does not know; read it above".to_string()
    };
    format!("create: 409 {reason:?} — {hint}")
}

// --------------------------------------------------------------------------
// The target.
// --------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub struct Target {
    pub tls: bool,
    pub host: String,
    pub port: u16,
    /// `scheme://host[:port]`, exactly as a join link starts.
    pub base: String,
}

impl Target {
    pub fn parse(url: &str) -> Result<Target, String> {
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
        let target = Target { tls, host, port, base: url.to_string() };
        // The organizer session rides every host request: never in cleartext
        // across a network. Plain http stays for a room on this machine.
        if !tls && !target.is_loopback() {
            return Err(format!("{url:?}: plain http to a host that is not this machine would send the organizer session in cleartext; use https://"));
        }
        Ok(target)
    }

    pub fn host_header(&self) -> String {
        match (self.tls, self.port) {
            (true, 443) | (false, 80) => self.host.clone(),
            _ => format!("{}:{}", self.host, self.port),
        }
    }

    pub fn ws_url(&self, path: &str) -> String {
        format!("{}://{}:{}{path}", if self.tls { "wss" } else { "ws" }, self.host, self.port)
    }

    pub fn connect_host(&self) -> &str {
        self.host.trim_start_matches('[').trim_end_matches(']')
    }

    /// A room on this machine's own loopback: a harness run against it proves
    /// the harness, and its numbers are never the deployed substrate's.
    pub fn is_loopback(&self) -> bool {
        let h = self.connect_host();
        h == "localhost" || h.parse::<std::net::IpAddr>().is_ok_and(|ip| ip.is_loopback())
    }

    /// The TLS client for an `https` target (rustls with ring, Mozilla's
    /// roots); `None` for plain http.
    pub fn tls_connector(&self) -> Option<tokio_rustls::TlsConnector> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        self.tls.then(|| {
            let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            let mut config = rustls::ClientConfig::builder().with_root_certificates(roots).with_no_client_auth();
            config.alpn_protocols = vec![b"http/1.1".to_vec()];
            tokio_rustls::TlsConnector::from(Arc::new(config))
        })
    }
}

// --------------------------------------------------------------------------
// HTTP/1.1, kept alive, over TCP or TLS. Just enough for the room's routes:
// Content-Length or chunked bodies, no redirects followed.
// --------------------------------------------------------------------------

pub trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}

pub struct Http {
    target: Arc<Target>,
    tls: Option<tokio_rustls::TlsConnector>,
    conn: Option<BufReader<Box<dyn Io>>>,
    read_timeout: Duration,
}

pub struct Resp {
    pub status: u16,
    pub location: Option<String>,
    pub body: Vec<u8>,
}

impl Resp {
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or(Value::Null)
    }
}

impl Http {
    pub fn new(target: &Arc<Target>, tls: &Option<tokio_rustls::TlsConnector>) -> Http {
        Http {
            target: target.clone(),
            tls: tls.clone(),
            conn: None,
            read_timeout: READ_TIMEOUT,
        }
    }

    /// The same client with a different per-request deadline (tests).
    pub fn with_read_timeout(mut self, d: Duration) -> Http {
        self.read_timeout = d;
        self
    }

    /// Open the connection now, so a later request is timed on an already
    /// open, already handshaken connection.
    pub async fn open(&mut self) -> Result<(), String> {
        let t = &self.target;
        let tcp = connect(t).await?;
        let io: Box<dyn Io> = match &self.tls {
            None => Box::new(tcp),
            Some(connector) => {
                let name = rustls::pki_types::ServerName::try_from(t.connect_host().to_string())
                    .map_err(|e| format!("{}: {e}", t.host))?;
                let tls = tokio::time::timeout(CONNECT_TIMEOUT, connector.connect(name, tcp))
                    .await
                    .map_err(|_| format!("TLS to {}: no handshake within {}s", t.host, CONNECT_TIMEOUT.as_secs()))?;
                Box::new(tls.map_err(|e| format!("TLS to {}: {e}", t.host))?)
            }
        };
        self.conn = Some(BufReader::new(io));
        Ok(())
    }

    pub fn is_open(&self) -> bool {
        self.conn.is_some()
    }

    pub async fn call(&mut self, method: &str, path: &str, bearer: Option<&str>, body: Option<&Value>) -> Result<Resp, String> {
        if self.conn.is_none() {
            self.open().await?;
        }
        let result = match tokio::time::timeout(self.read_timeout, self.exchange(method, path, bearer, body)).await {
            Ok(r) => r,
            Err(_) => Err(format!("no response within {}s", self.read_timeout.as_secs_f64())),
        };
        if result.is_err() {
            self.conn = None;
        }
        result.map_err(|e| format!("{method} {path}: {e}"))
    }

    async fn exchange(&mut self, method: &str, path: &str, bearer: Option<&str>, body: Option<&Value>) -> Result<Resp, String> {
        let mut req = format!(
            "{method} {path} HTTP/1.1\r\nHost: {}\r\nUser-Agent: popquiz-harness\r\nAccept: */*\r\n",
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

/// A TCP connection to the target, under [`CONNECT_TIMEOUT`], with Nagle off.
async fn connect(t: &Target) -> Result<TcpStream, String> {
    let tcp = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect((t.connect_host(), t.port)))
        .await
        .map_err(|_| format!("connect {}:{}: timed out after {}s", t.host, t.port, CONNECT_TIMEOUT.as_secs()))?
        .map_err(|e| format!("connect {}:{}: {e}", t.host, t.port))?;
    tcp.set_nodelay(true).map_err(|e| e.to_string())?;
    Ok(tcp)
}

pub async fn read_chunked<R: AsyncBufReadExt + Unpin>(r: &mut R, out: &mut Vec<u8>) -> Result<(), String> {
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

/// Wait for the room to answer `GET /join`. The first request also waits out
/// a stopped machine's start (Fly starts it on the first request after a
/// stop or a deploy).
pub async fn wait_for_room(target: &Arc<Target>, tls: &Option<tokio_rustls::TlsConnector>) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let mut http = Http::new(target, tls);
        match http.call("GET", "/join", None, None).await {
            Ok(r) if r.status == 200 => return Ok(()),
            Ok(r) if Instant::now() < deadline && r.status >= 500 => {}
            Ok(r) => return Err(format!("GET /join: {}", r.status)),
            Err(_) if Instant::now() < deadline => {}
            Err(e) => return Err(e),
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

// --------------------------------------------------------------------------
// Sockets. Each is read by its own task, which notes when each phase first
// arrived and keeps the last frame per phase, and hands every frame to an
// optional hook (smoke's secrecy scan).
// --------------------------------------------------------------------------

pub type Ws = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

/// Called with every frame a socket reads; `None` for a frame that is not JSON.
pub type OnFrame = Arc<dyn Fn(Option<&Value>) + Send + Sync>;

#[derive(Default)]
pub struct Seen {
    pub first: HashMap<String, Instant>,
    pub last: HashMap<String, Value>,
    pub frames: usize,
    pub closed: bool,
}

pub struct Watcher {
    pub seen: Arc<Mutex<Seen>>,
    task: tokio::task::JoinHandle<()>,
}

impl Watcher {
    pub fn arrived(&self, phase: &str) -> Option<Instant> {
        self.seen.lock().unwrap().first.get(phase).copied()
    }

    pub fn last(&self, phase: &str) -> Option<Value> {
        self.seen.lock().unwrap().last.get(phase).cloned()
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub async fn open_ws(target: &Target, path: &str, attach: Option<&str>) -> Result<Ws, String> {
    let tcp = connect(target).await.map_err(|e| format!("{e} (for {path})"))?;
    let (mut ws, _) = tokio::time::timeout(
        CONNECT_TIMEOUT,
        tokio_tungstenite::client_async_tls_with_config(target.ws_url(path), tcp, None, None),
    )
    .await
    .map_err(|_| format!("socket {path}: no upgrade within {}s", CONNECT_TIMEOUT.as_secs()))?
    .map_err(|e| format!("socket {path}: {e}"))?;
    if let Some(token) = attach {
        let msg = json!({ "t": "attach", "token": token }).to_string();
        ws.send(Message::Text(msg.into())).await.map_err(|e| format!("attach {path}: {e}"))?;
    }
    Ok(ws)
}

pub fn watch(mut ws: Ws, on_frame: Option<OnFrame>) -> Watcher {
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
                if let Some(hook) = &on_frame {
                    hook(None);
                }
                continue;
            };
            if let Some(hook) = &on_frame {
                hook(Some(&frame));
            }
            let phase = frame["phase"].as_str().unwrap_or("").to_string();
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
pub async fn all_reach(watchers: &[&Watcher], phase: &str, wait: Duration) -> Result<Vec<Instant>, String> {
    let deadline = Instant::now() + wait;
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
                wait.as_secs()
            ));
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

/// An error, with what it most likely means on Fly when the room vanished
/// mid-run: the machine runs until something stops or redeploys it, and a
/// restarted machine has no rooms (room/README.md, *Deploying*). Never read as
/// "a second machine" — `fly.toml` runs one.
pub fn explain(e: &str) -> String {
    // Connection-level failures, a socket that stopped hearing the room, or a
    // host action on the room answered 404. Not a create's 404 or 409: those
    // are the question or the session, and say so themselves.
    let connection = e.contains("closed before a response") || e.contains("connect ") || e.contains("never saw");
    let room_404 = e.contains(": 404") && !e.starts_with("create");
    let gone = connection || room_404;
    if gone {
        format!("{e} — the room may be gone: if this is the deployed app, the machine may have been stopped or restarted mid-run (it runs until something stops or redeploys it, and a restart or a deploy loses every room); check `fly status`, then schedule the ids again and rerun")
    } else {
        e.to_string()
    }
}

// --------------------------------------------------------------------------
// Small things.
// --------------------------------------------------------------------------

pub fn millis(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

/// xorshift64*: enough to scatter letters and burst offsets; the seed is printed.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets() {
        let t = Target::parse("https://rustnyc-popquiz.fly.dev/").unwrap();
        assert_eq!((t.tls, t.host.as_str(), t.port, t.base.as_str()), (true, "rustnyc-popquiz.fly.dev", 443, "https://rustnyc-popquiz.fly.dev"));
        assert_eq!(t.host_header(), "rustnyc-popquiz.fly.dev");
        assert_eq!(t.ws_url("/rooms/a/ws/wall"), "wss://rustnyc-popquiz.fly.dev:443/rooms/a/ws/wall");
        assert!(!t.is_loopback());
        let t = Target::parse("http://127.0.0.1:3000").unwrap();
        assert_eq!((t.tls, t.port, t.host_header().as_str()), (false, 3000, "127.0.0.1:3000"));
        assert!(t.is_loopback());
        assert!(Target::parse("http://localhost:8080").unwrap().is_loopback());
        let t = Target::parse("http://[::1]:8080").unwrap();
        assert_eq!(t.connect_host(), "::1");
        assert!(t.is_loopback());
        for bad in ["x.fly.dev", "https://x.fly.dev/path", "https://", "ws://x", "http://h:port"] {
            assert!(Target::parse(bad).is_err(), "{bad:?}");
        }
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
