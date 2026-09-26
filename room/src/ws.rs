//! The transport: one broadcast per room, over WebSockets (T-04c).
//!
//! Three routes, one per viewer — `GET /rooms/{id}/ws/wall`, `/ws/buzzer`,
//! `/ws/host`. Every frame is the viewer's **whole** state, never a delta:
//!
//! ```text
//! {"t":"state","revision":N, …the view::wall / view::buzzer / view::host payload…}
//! ```
//!
//! The payload is flattened into the frame, so its own `phase` is the frame's
//! only `phase` (AC-81: exactly one, read from the room's one phase value).
//!
//! **One broadcast per room.** Each room has one [`watch`] channel per viewer
//! kind. A room's three payloads are built and serialized once per revision,
//! here and nowhere else, and every subscriber of that kind is handed the same
//! bytes. `watch` rather than `broadcast` because every frame is a whole state:
//! a subscriber only ever needs the newest one, so a slow phone skips
//! superseded states instead of lagging behind them, and there is no `Lagged`
//! path on which a reveal could be dropped (the spike had to count those).
//!
//! **What triggers a push.** [`Transport::changed`] reads the room and publishes
//! only if its [`Room::revision`] is newer than what the channel last carried,
//! so a poke that changed nothing sends nothing, and two racing pokes cannot
//! publish out of order. It is called by the `notify` middleware after every
//! non-GET request under `/rooms/{id}/…` (every host action), and by the
//! transport after each call into the session map. Any writer that is not an
//! HTTP route — T-04b if it takes answers over a socket, T-11's reaper — calls
//! it too. There is no polling and no per-client timer.
//!
//! **Attach.** The wall needs no credential: its projection is the public one
//! `GET /rooms/{id}/wall` already serves. A buzzer and the host send one text
//! message first, `{"t":"attach","token":"…"}` — the buzzer its session token
//! ([`SessionTokens::resolve`]), the host its bearer (`HostAuth::authorize_host`).
//! The credential never rides in the URL: the host's resume link keeps its
//! session in a `#fragment` precisely to stay out of logs. On attach the
//! current state is sent **before anything else** (§4.3, AC-37), then one frame
//! per new revision. A buzzer's attach frame — and only that one — also carries
//! `"session":{"saved":<letter|null>}`, its own saved answer, so a phone that
//! reconnects in `closed` can show it; broadcasts are never personalized.
//!
//! **Reconnect.** A second socket for the same session replaces the first,
//! which is closed with [`close::REPLACED`] and does not report the session
//! gone. Hosts are not keyed: several host devices may be attached at once
//! (AC-50).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, watch};

use crate::question::Letter;
use crate::rooms::{AppState, Room, RoomError, Urls};
use crate::view::{self, Viewer};

/// A participant session, as the session map names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

/// The seam T-04b's session map implements. T-04b and T-04c were built in
/// parallel; the two are wired when the second of them merges.
pub trait SessionTokens: Send + Sync + 'static {
    /// A buzzer attaching with `token` to room `room_id`. This *is* the attach:
    /// the map may mark the session connected and move `present`, which is why
    /// the transport pokes the room after it. `None` refuses the socket.
    fn resolve(&self, room_id: &str, token: &str) -> Option<SessionId>;
    /// The session's saved answer, for its own attach frame. A read.
    fn saved(&self, room_id: &str, session: SessionId) -> Option<Letter>;
    /// The session's current socket closed. Not called for a socket that was
    /// replaced by a newer one for the same session. Called with no transport
    /// lock held, so a socket that dies at the very instant a new one for the
    /// same session attaches can report `gone` just after the new one's
    /// `resolve`; a map that counts `present` should treat `resolve` after
    /// `gone` as authoritative.
    fn gone(&self, room_id: &str, session: SessionId);
}

/// Resolves nobody. The default until T-04b's map is wired, so a buzzer is
/// refused rather than served without a session.
pub struct NoTokens;

impl SessionTokens for NoTokens {
    fn resolve(&self, _room_id: &str, _token: &str) -> Option<SessionId> {
        None
    }
    fn saved(&self, _room_id: &str, _session: SessionId) -> Option<Letter> {
        None
    }
    fn gone(&self, _room_id: &str, _session: SessionId) {}
}

/// Close codes. Each says what happened and nothing more (AC-70).
pub mod close {
    /// A newer socket attached for the same session.
    pub const REPLACED: u16 = 4000;
    /// The attach message was missing, malformed, or its credential was wrong.
    pub const UNAUTHORIZED: u16 = 4401;
    /// The room no longer exists.
    pub const ROOM_GONE: u16 = 4404;
    /// No attach message arrived in time.
    pub const ATTACH_TIMEOUT: u16 = 4408;
}

/// How long a buzzer or host socket has to send its attach message.
pub const ATTACH_TIMEOUT: Duration = Duration::from_secs(10);

/// A socket that cannot take a frame (or the close) for this long is dropped.
/// Per send, not a clock: nothing is sent because time passed.
const SEND_TIMEOUT: Duration = Duration::from_secs(10);

/// The largest thing a client sends is the ~100-byte attach message.
/// tungstenite's own default is 64 MiB per message.
const MAX_INBOUND_BYTES: usize = 4 * 1024;

fn slot(viewer: Viewer) -> usize {
    match viewer {
        Viewer::Wall => 0,
        Viewer::Buzzer => 1,
        Viewer::Host => 2,
    }
}

#[derive(Serialize)]
struct Frame<P> {
    t: &'static str,
    revision: u64,
    #[serde(flatten)]
    payload: P,
}

fn frame(revision: u64, payload: impl Serialize) -> Arc<str> {
    serde_json::to_string(&Frame {
        t: "state",
        revision,
        payload,
    })
    .expect("payloads are plain data")
    .into()
}

/// The room's three frames at its current revision. The one place payloads
/// are built for the socket.
fn frames(room: &Room, urls: &Urls) -> [Arc<str>; 3] {
    let revision = room.revision();
    [
        frame(revision, view::wall(room, urls)),
        frame(revision, view::buzzer(room)),
        frame(revision, view::host(room)),
    ]
}

/// One room's broadcast.
struct Channel {
    revision: u64,
    /// Indexed by [`slot`].
    senders: [watch::Sender<Arc<str>>; 3],
    /// The current socket of each attached session: its connection id, and
    /// the way to tell it it has been replaced.
    buzzers: HashMap<SessionId, (u64, oneshot::Sender<()>)>,
    subscribers: usize,
}

impl Channel {
    fn new(room: &Room, urls: &Urls) -> Channel {
        let senders = frames(room, urls).map(|f| watch::channel(f).0);
        Channel {
            revision: room.revision(),
            senders,
            buzzers: HashMap::new(),
            subscribers: 0,
        }
    }

    /// Publish the room's state if it is newer than the last broadcast.
    fn catch_up(&mut self, room: &Room, urls: &Urls) {
        if room.revision() > self.revision {
            self.revision = room.revision();
            for (sender, f) in self.senders.iter().zip(frames(room, urls)) {
                sender.send_replace(f);
            }
        }
    }
}

struct Inner {
    state: Arc<AppState>,
    tokens: Arc<dyn SessionTokens>,
    rooms: Mutex<HashMap<String, Channel>>,
    next_connection: AtomicU64,
    attach_timeout: Duration,
}

/// The broadcast hub, and the session map it asks about buzzers.
///
/// It travels as an axum [`Extension`]: layer `Extension(Transport::new(state,
/// tokens))` over [`crate::router_with`] to choose the session map. Without
/// one, the router supplies a `Transport` over [`NoTokens`], so the wall and
/// host sockets work and every buzzer is refused.
#[derive(Clone)]
pub struct Transport {
    inner: Arc<Inner>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Transport {
    pub fn new(state: Arc<AppState>, tokens: Arc<dyn SessionTokens>) -> Transport {
        Transport {
            inner: Arc::new(Inner {
                state,
                tokens,
                rooms: Mutex::new(HashMap::new()),
                next_connection: AtomicU64::new(0),
                attach_timeout: ATTACH_TIMEOUT,
            }),
        }
    }

    /// The same transport with a different attach deadline (tests).
    pub fn with_attach_timeout(self, timeout: Duration) -> Transport {
        let inner = Arc::try_unwrap(self.inner)
            .unwrap_or_else(|_| panic!("set the attach deadline before the transport is shared"));
        Transport {
            inner: Arc::new(Inner {
                attach_timeout: timeout,
                ..inner
            }),
        }
    }

    /// Room `room_id` may have changed: push it to its subscribers if its
    /// revision moved. Cheap and idempotent; a room nobody watches costs one
    /// map lookup. A room that no longer exists is dropped, which closes its
    /// sockets with [`close::ROOM_GONE`].
    pub fn changed(&self, room_id: &str) {
        let inner = &*self.inner;
        if !lock(&inner.rooms).contains_key(room_id) {
            return;
        }
        // Rooms lock, then hub lock — the only order either is taken in
        // together — so the revision check and the send are one step.
        let found = inner.state.with_room(room_id, |room| {
            if let Some(channel) = lock(&inner.rooms).get_mut(room_id) {
                channel.catch_up(room, inner.state.urls());
            }
        });
        if found.is_err() {
            lock(&inner.rooms).remove(room_id);
        }
    }

    /// How many rooms have a live broadcast. Diagnostics and tests.
    pub fn broadcasting(&self) -> usize {
        lock(&self.inner.rooms).len()
    }

    fn subscribe(&self, room_id: &str, viewer: Viewer) -> Result<Subscription, RoomError> {
        let inner = &*self.inner;
        let rx = inner.state.with_room(room_id, |room| {
            let urls = inner.state.urls();
            let mut rooms = lock(&inner.rooms);
            let channel = rooms
                .entry(room_id.to_string())
                .or_insert_with(|| Channel::new(room, urls));
            channel.catch_up(room, urls);
            channel.subscribers += 1;
            channel.senders[slot(viewer)].subscribe()
        })?;
        Ok(Subscription {
            transport: self.clone(),
            room_id: room_id.to_string(),
            rx,
        })
    }

    /// Make `connection` the session's current socket, replacing any other.
    fn register(&self, room_id: &str, session: SessionId, connection: u64) -> Option<oneshot::Receiver<()>> {
        let (kick, kicked) = oneshot::channel();
        let mut rooms = lock(&self.inner.rooms);
        let channel = rooms.get_mut(room_id)?;
        if let Some((_, previous)) = channel.buzzers.insert(session, (connection, kick)) {
            let _ = previous.send(());
        }
        Some(kicked)
    }

    /// Whether `connection` was still the session's current socket.
    fn unregister(&self, room_id: &str, session: SessionId, connection: u64) -> bool {
        let mut rooms = lock(&self.inner.rooms);
        let Some(channel) = rooms.get_mut(room_id) else {
            return false;
        };
        match channel.buzzers.get(&session) {
            Some((current, _)) if *current == connection => {
                channel.buzzers.remove(&session);
                true
            }
            _ => false,
        }
    }

    /// Everything a socket does, from attach to close.
    async fn run(self, mut socket: WebSocket, room_id: String, viewer: Viewer) {
        let inner = &*self.inner;
        let connection = inner.next_connection.fetch_add(1, Ordering::Relaxed);

        let session = match viewer {
            Viewer::Wall => None,
            Viewer::Host | Viewer::Buzzer => {
                let token = match attach_token(&mut socket, inner.attach_timeout).await {
                    Attach::Token(t) => t,
                    Attach::Refused(code) => return bye(socket, code).await,
                    Attach::Closed => return,
                };
                match viewer {
                    Viewer::Host => match inner.state.with_hosted_room(&room_id, Some(&token), |_| ()) {
                        Ok(()) => None,
                        Err(RoomError::NotFound) => return bye(socket, close::ROOM_GONE).await,
                        Err(_) => return bye(socket, close::UNAUTHORIZED).await,
                    },
                    _ => match inner.tokens.resolve(&room_id, &token) {
                        Some(session) => Some(session),
                        None => return bye(socket, close::UNAUTHORIZED).await,
                    },
                }
            }
        };

        // Subscribing publishes the room's current state first, so resolving
        // a session (which may have moved `present`) reaches everyone.
        let Ok(mut sub) = self.subscribe(&room_id, viewer) else {
            return bye(socket, close::ROOM_GONE).await;
        };
        let mut kicked = None;
        if let Some(session) = session {
            match self.register(&room_id, session, connection) {
                Some(k) => kicked = Some(k),
                None => return bye(socket, close::ROOM_GONE).await,
            }
        }

        // Full current state, before anything else (§4.3, AC-37).
        let first = sub.rx.borrow_and_update().clone();
        let first: Utf8Bytes = match session {
            Some(s) => with_session(&first, inner.tokens.saved(&room_id, s)).into(),
            None => first.as_ref().into(),
        };
        let mut replaced = false;
        if send(&mut socket, first).await {
            let kick = async {
                match kicked {
                    Some(k) => k.await.is_ok(),
                    None => std::future::pending().await,
                }
            };
            tokio::pin!(kick);
            loop {
                tokio::select! {
                    changed = sub.rx.changed() => {
                        if changed.is_err() {
                            bye(socket, close::ROOM_GONE).await;
                            break;
                        }
                        let f = sub.rx.borrow_and_update().clone();
                        if !send(&mut socket, f.as_ref().into()).await {
                            break;
                        }
                    }
                    true = &mut kick => {
                        replaced = true;
                        bye(socket, close::REPLACED).await;
                        break;
                    }
                    incoming = socket.recv() => match incoming {
                        None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                        // Nothing else is read from an attached socket yet.
                        Some(Ok(_)) => {}
                    },
                }
            }
        }

        if let (Some(session), false) = (session, replaced) {
            if self.unregister(&room_id, session, connection) {
                inner.tokens.gone(&room_id, session);
                self.changed(&room_id);
            }
        }
        drop(sub);
    }
}

/// A socket's hold on its room's broadcast. Dropping the last one ends the
/// room's broadcast, so nothing is kept for a room nobody watches.
struct Subscription {
    transport: Transport,
    room_id: String,
    rx: watch::Receiver<Arc<str>>,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        let mut rooms = lock(&self.transport.inner.rooms);
        if let Some(channel) = rooms.get_mut(&self.room_id) {
            channel.subscribers -= 1;
            if channel.subscribers == 0 {
                rooms.remove(&self.room_id);
            }
        }
    }
}

/// The buzzer's attach frame: the shared frame plus its own saved answer.
/// Parsed and re-serialized for this one socket, once per attach.
fn with_session(frame: &str, saved: Option<Letter>) -> String {
    let mut v: serde_json::Value = serde_json::from_str(frame).expect("frames are JSON objects");
    v["session"] = serde_json::json!({ "saved": saved });
    v.to_string()
}

#[derive(Deserialize)]
struct AttachMessage {
    t: String,
    token: String,
}

enum Attach {
    Token(String),
    Refused(u16),
    Closed,
}

async fn attach_token(socket: &mut WebSocket, deadline: Duration) -> Attach {
    let wait = async {
        loop {
            match socket.recv().await {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => return Attach::Closed,
                Some(Ok(Message::Text(text))) => {
                    return match serde_json::from_str::<AttachMessage>(&text) {
                        Ok(m) if m.t == "attach" => Attach::Token(m.token),
                        _ => Attach::Refused(close::UNAUTHORIZED),
                    }
                }
                Some(Ok(Message::Binary(_))) => return Attach::Refused(close::UNAUTHORIZED),
                Some(Ok(_)) => {}
            }
        }
    };
    tokio::time::timeout(deadline, wait)
        .await
        .unwrap_or(Attach::Refused(close::ATTACH_TIMEOUT))
}

async fn send(socket: &mut WebSocket, text: Utf8Bytes) -> bool {
    matches!(
        tokio::time::timeout(SEND_TIMEOUT, socket.send(Message::Text(text))).await,
        Ok(Ok(()))
    )
}

async fn bye(mut socket: WebSocket, code: u16) {
    let frame = CloseFrame {
        code,
        reason: Utf8Bytes::from_static(""),
    };
    let _ = tokio::time::timeout(SEND_TIMEOUT, socket.send(Message::Close(Some(frame)))).await;
}

/// `GET /rooms/{id}/ws/<viewer>`. An unknown room is `404` before upgrade.
pub(crate) async fn upgrade(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    Extension(transport): Extension<Transport>,
    viewer: Viewer,
) -> Response {
    if transport.inner.state.with_room(&room_id, |_| ()).is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }
    ws.max_message_size(MAX_INBOUND_BYTES)
        .max_frame_size(MAX_INBOUND_BYTES)
        .on_upgrade(move |socket| transport.run(socket, room_id, viewer))
}

/// The room id a request path is about, if it is under `/rooms/{id}/`.
fn room_of(path: &str) -> Option<&str> {
    let mut segments = path.strip_prefix("/rooms/")?.split('/');
    let id = segments.next()?;
    segments.next()?;
    Some(id)
}

/// After every non-GET request under `/rooms/{id}/…`, poke that room. A
/// refused request changed nothing, and the poke then sends nothing.
pub(crate) async fn notify(request: Request, next: Next) -> Response {
    let room = (request.method() != Method::GET)
        .then(|| room_of(request.uri().path()).map(str::to_string))
        .flatten();
    let transport = request.extensions().get::<Transport>().cloned();
    let response = next.run(request).await;
    if let (Some(transport), Some(room)) = (transport, room) {
        transport.changed(&room);
    }
    response
}

/// Supply the router's default [`Transport`] unless one was layered on.
pub(crate) async fn provide(State(default): State<Transport>, mut request: Request, next: Next) -> Response {
    if request.extensions().get::<Transport>().is_none() {
        request.extensions_mut().insert(default);
    }
    next.run(request).await
}

/// Serve `router` with `TCP_NODELAY` on every accepted socket. axum does not
/// set it, and the T-03 spike found that without it a 40 ms delayed-ACK mode
/// shows up in the latency histogram and reads as the server being slow.
pub async fn serve(listener: tokio::net::TcpListener, router: Router) -> std::io::Result<()> {
    use axum::serve::ListenerExt;
    let listener = listener.tap_io(|stream| {
        let _ = stream.set_nodelay(true);
    });
    axum::serve(listener, router).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_paths_under_a_room_name_one() {
        assert_eq!(room_of("/rooms/abc/reveal"), Some("abc"));
        assert_eq!(room_of("/rooms/abc/ws/wall"), Some("abc"));
        assert_eq!(room_of("/rooms"), None);
        assert_eq!(room_of("/rooms/abc"), None);
        assert_eq!(room_of("/admin/rooms/abc/x"), None);
    }

    #[test]
    fn the_attach_frame_adds_the_session_and_keeps_everything_else() {
        let shared = frame(7, serde_json::json!({"phase": "closed", "code": "ABCDEF"}));
        let v: serde_json::Value = serde_json::from_str(&with_session(&shared, Some(Letter::B))).unwrap();
        assert_eq!(v["t"], "state");
        assert_eq!(v["revision"], 7);
        assert_eq!(v["phase"], "closed");
        assert_eq!(v["session"]["saved"], "B");
        let none: serde_json::Value = serde_json::from_str(&with_session(&shared, None)).unwrap();
        assert!(none["session"]["saved"].is_null());
    }
}
