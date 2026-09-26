//! The T-03 burst spike's server. **Throwaway.**
//!
//! One room, in memory, `idle -> live -> closed`, plus a reveal broadcast. That
//! is the whole of it, and it is deliberately not the room: T-04a writes the
//! real phase machine on a blank page, and nothing here should be carried over.
//! What this exists to do is let `burst.rs` put 200 real WebSockets against a
//! real Fly machine and measure AC-41, AC-52, AC-53 and AC-54 before any room
//! code is written (BUILDPLAN T-03, and §5 — a p95 miss re-opens D-A option 2).
//!
//! No auth, no wall, no persistence, no sessions module, no `answers` module.
//! Control frames are unauthenticated: the app is public for the minutes a run
//! takes. That is a real exposure and is named rather than hidden — a stray
//! request would surface as an epoch or reconciliation mismatch in the report,
//! not as a quietly wrong number.

#[path = "spike_shared.rs"]
mod shared;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use shared::{answer_contribution, index_letter, letter_index};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::{broadcast, Mutex};

/// SPEC.md §4.1: capacity is 200, configurable. The 201st participant join is
/// refused. The driver's own control socket is not a participant and does not
/// count against this. `app()` takes the capacity as a parameter so the loopback
/// test can prove the refusal at 13 rather than opening 201 sockets.
pub const CAPACITY: usize = 200;

/// Bounds on what an unauthenticated caller can make this process allocate.
///
/// Control frames are deliberately unauthenticated — the ticket says *no auth*,
/// and adding a real auth story here would be building T-10's job into a
/// throwaway. But "no auth" is not the same as "no limits" while this app is
/// briefly public on a 256 MB machine:
///
/// - `pad_bytes` is clamped, so one frame cannot ask for gigabytes of reveal.
/// - A new session id is refused past capacity and ids are length-capped, so the
///   answer map cannot be grown without joining.
/// - Inbound frames and messages are capped at 64 KiB — the largest thing a
///   client legitimately sends is an ~80-byte answer. tungstenite's own default
///   is 64 MiB per message, which would let a handful of connections exhaust
///   memory.
/// - Concurrent connections are bounded outside this process, by the Fly proxy:
///   `http_service.concurrency.hard_limit = 400` in `fly.spike.toml`.
///
/// So the worst an anonymous caller can do is make the room's numbers obviously
/// wrong — which the AC-52 reconciliation would catch — rather than kill the
/// process mid-run. Run locally without the proxy, the connection bound is gone;
/// that is acceptable for a loopback harness and is stated rather than implied.
const MAX_PAD_BYTES: usize = 64 * 1024;
const MAX_SESSION_LEN: usize = 64;
const MAX_INBOUND_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Idle,
    Live,
    Closed,
}

impl Phase {
    fn as_str(self) -> &'static str {
        match self {
            Phase::Idle => "idle",
            Phase::Live => "live",
            Phase::Closed => "closed",
        }
    }
}

struct Room {
    epoch: u64,
    phase: Phase,
    /// session -> (letter index, the seq that was applied)
    answers: HashMap<String, (u8, u32)>,
    /// Computed once at `close` and frozen from then on (SPEC.md §4.4).
    totals: Option<[u32; 5]>,
    answered_frozen: usize,
    /// Σ of the applied seq over sessions. Cheap, and an aggregate — which is
    /// exactly why `applied_fingerprint` exists beside it.
    applied_seq_sum: u64,
    /// XOR over sessions of `answer_contribution`. Binds the session id into
    /// each term, so a two-session swap is visible where the sum is not.
    applied_fingerprint: u64,
    present: usize,
    /// Counted, never swallowed: a client whose broadcast receiver fell behind
    /// may have missed a reveal, and a missed reveal must not look like a fast
    /// one. Surfaced in `control_ok`.
    lagged: u64,
}

impl Room {
    fn new() -> Self {
        Self {
            epoch: 0,
            phase: Phase::Idle,
            answers: HashMap::new(),
            totals: None,
            answered_frozen: 0,
            applied_seq_sum: 0,
            applied_fingerprint: 0,
            present: 0,
            lagged: 0,
        }
    }

    fn reset(&mut self) {
        let present = self.present;
        let lagged = self.lagged;
        *self = Room::new();
        // Connections outlive a reset — the burst runs several cycles over the
        // same 200 sockets, which is the point of measuring without reconnects.
        self.present = present;
        self.lagged = lagged;
        self.epoch += 1;
    }

    fn live_totals(&self) -> [u32; 5] {
        let mut t = [0u32; 5];
        for (letter, _) in self.answers.values() {
            t[*letter as usize] += 1;
        }
        t
    }

    /// SPEC.md §4.4: `closed` computes totals from sessions **once** and freezes
    /// them. Calling close twice must not recompute.
    fn close(&mut self) {
        if self.totals.is_none() {
            self.totals = Some(self.live_totals());
            self.answered_frozen = self.answers.len();
        }
        self.phase = Phase::Closed;
    }
}

struct AppState {
    room: Mutex<Room>,
    tx: broadcast::Sender<Arc<str>>,
    instance: String,
    region: String,
    app: String,
    capacity: usize,
}

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// The whole spike server as a router, with no socket bound.
///
/// `main` serves it on `$PORT`; `burst.rs`'s loopback test serves it on
/// `127.0.0.1:0` in-process, so the full segment can be proven end to end
/// without a deploy and without a second process to manage.
pub fn app(instance: String, region: String, app_name: String, capacity: usize) -> Router {
    let (tx, _) = broadcast::channel::<Arc<str>>(256);
    let state = Arc::new(AppState {
        room: Mutex::new(Room::new()),
        tx,
        instance,
        region,
        app: app_name,
        capacity,
    });
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws_handler))
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = env_or("PORT", "8080").parse().unwrap_or(8080);

    let app = app(
        // FLY_MACHINE_ID is what proves all 200 sessions met one process. If a
        // second machine ever answered, the totals would be split across two
        // maps and AC-52 would be counting against a fiction.
        env_or("FLY_MACHINE_ID", "local"),
        env_or("FLY_REGION", "local"),
        env_or("FLY_APP_NAME", "spike-local"),
        CAPACITY,
    );

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    eprintln!("spike-server listening on 0.0.0.0:{port}");

    // axum does not set TCP_NODELAY, and `tap_io` is the documented seam for it.
    // Without this, a 40 ms delayed-ACK mode would show up in the write
    // histogram and read as the server being slow.
    use axum::serve::ListenerExt;
    let listener = listener.tap_io(|stream| {
        let _ = stream.set_nodelay(true);
    });

    axum::serve(listener, app).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.max_message_size(MAX_INBOUND_BYTES)
        .max_frame_size(MAX_INBOUND_BYTES)
        .on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    {
        let room = state.room.lock().await;
        let hello = json!({
            "t": "hello",
            "instance": state.instance,
            "region": state.region,
            "app": state.app,
            "epoch": room.epoch,
            "phase": room.phase.as_str(),
            "capacity": state.capacity,
        });
        if socket.send(Message::Text(hello.to_string().into())).await.is_err() {
            return;
        }
    }

    let mut my_session: Option<String> = None;

    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(Ok(msg)) = incoming else { break };
                let Message::Text(text) = msg else { continue };
                let Ok(frame) = serde_json::from_str::<Value>(&text) else { continue };
                let reply = handle_frame(&frame, &state, &mut my_session).await;
                if let Some(reply) = reply {
                    if socket.send(Message::Text(reply.to_string().into())).await.is_err() {
                        break;
                    }
                }
            }
            broadcast = rx.recv() => {
                match broadcast {
                    Ok(payload) => {
                        if socket.send(Message::Text(payload.to_string().into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // A dropped reveal must never be invisible. Count it;
                        // the client will also notice a missing receipt and
                        // invalidate the run.
                        state.room.lock().await.lagged += n;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    if my_session.is_some() {
        let mut room = state.room.lock().await;
        room.present = room.present.saturating_sub(1);
    }
}

async fn handle_frame(
    frame: &Value,
    state: &Arc<AppState>,
    my_session: &mut Option<String>,
) -> Option<Value> {
    match frame.get("t").and_then(Value::as_str)? {
        "join" => {
            let session = frame.get("session")?.as_str()?.to_string();
            if session.len() > MAX_SESSION_LEN {
                return Some(json!({"t": "refused", "reason": "bad_session"}));
            }
            let mut room = state.room.lock().await;
            // SPEC.md §4.1 / AC-30: capacity is checked before a session is
            // created and reserves nothing.
            if room.present >= state.capacity {
                return Some(json!({"t": "refused", "reason": "full"}));
            }
            room.present += 1;
            *my_session = Some(session.clone());
            Some(json!({
                "t": "joined",
                "session": session,
                "present": room.present,
                "phase": room.phase.as_str(),
                "epoch": room.epoch,
                "capacity": state.capacity,
            }))
        }

        "answer" => {
            let session = frame.get("session")?.as_str()?.to_string();
            let seq = frame.get("seq")?.as_u64()? as u32;
            let letter = frame.get("letter")?.as_str()?.chars().next()?;
            let li = letter_index(letter)?;

            let mut room = state.room.lock().await;

            // SPEC.md §4.3: a write after `closed` is refused with the saved
            // answer restated. Nothing is mutated on this path.
            if room.phase != Phase::Live {
                let saved = room.answers.get(&session).map(|(l, _)| index_letter(*l));
                return Some(json!({
                    "t": "ack",
                    "seq": seq,
                    "letter": letter.to_string(),
                    "accepted": false,
                    "reason": "closed",
                    "saved_letter": saved.map(|c| c.to_string()),
                    "epoch": room.epoch,
                }));
            }

            // A new session id beyond capacity is refused rather than inserted.
            // Without this an anonymous caller could grow the map without ever
            // joining, and the room's totals would stop meaning anything.
            if !room.answers.contains_key(&session)
                && (room.answers.len() >= state.capacity || session.len() > MAX_SESSION_LEN)
            {
                return Some(json!({
                    "t": "ack", "seq": seq, "letter": letter.to_string(),
                    "accepted": false, "reason": "full", "epoch": room.epoch,
                }));
            }

            // The upsert. Last write wins (SPEC.md §4.3, AC-34). The fingerprint
            // and the seq sum are maintained here rather than recomputed, so the
            // ack means "in the authoritative map" and costs no scan.
            if let Some((old_letter, old_seq)) = room.answers.get(&session).copied() {
                room.applied_fingerprint ^= answer_contribution(&session, old_letter, old_seq);
                room.applied_seq_sum -= old_seq as u64;
            }
            room.answers.insert(session.clone(), (li, seq));
            room.applied_fingerprint ^= answer_contribution(&session, li, seq);
            room.applied_seq_sum += seq as u64;
            let epoch = room.epoch;

            // The guard is dropped before the ack is written, so the ack means
            // the write is in the map — not that the frame was received. That is
            // what makes `t_ack - t_send` a write latency rather than an echo.
            drop(room);

            Some(json!({
                "t": "ack",
                "seq": seq,
                "letter": letter.to_string(),
                "accepted": true,
                "applied_seq": seq,
                "epoch": epoch,
            }))
        }

        "ping" => Some(json!({
            "t": "pong",
            "nonce": frame.get("nonce").cloned().unwrap_or(Value::Null),
            "server_ts_ms": now_ms(),
        })),

        "control" => {
            let cmd = frame.get("cmd")?.as_str()?;
            let mut room = state.room.lock().await;
            match cmd {
                "reset" => room.reset(),
                "live" => {
                    room.phase = Phase::Live;
                }
                "close" => room.close(),
                "reveal" => {
                    let reveal_id = frame.get("reveal_id").and_then(Value::as_u64).unwrap_or(0);
                    // Clamped: unbounded, one frame asking for gigabytes would
                    // take the 256 MB machine down mid-run, and the run would
                    // look like a server that could not take the load.
                    let pad_len = (frame.get("pad_bytes").and_then(Value::as_u64).unwrap_or(2048)
                        as usize)
                        .min(MAX_PAD_BYTES);
                    let payload = json!({
                        "t": "reveal",
                        "reveal_id": reveal_id,
                        "answer": "C",
                        "server_ts_ms": now_ms(),
                        // Sized so the fan-out is not flattered by a 60-byte
                        // frame; the real reveal payload is kilobytes.
                        "pad": "x".repeat(pad_len),
                    });
                    let _ = state.tx.send(Arc::from(payload.to_string().as_str()));
                }
                "totals" => {}
                _ => {}
            }

            let totals = room.totals.unwrap_or_else(|| room.live_totals());
            let answered = if room.totals.is_some() {
                room.answered_frozen
            } else {
                room.answers.len()
            };
            Some(json!({
                "t": "control_ok",
                "cmd": cmd,
                "epoch": room.epoch,
                "phase": room.phase.as_str(),
                "frozen": room.totals.is_some(),
                "totals": totals,
                "answered": answered,
                "present": room.present,
                "applied_seq_sum": room.applied_seq_sum,
                "applied_fingerprint": room.applied_fingerprint.to_string(),
                "lagged": room.lagged,
            }))
        }

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_freezes_totals_once_and_a_second_close_does_not_recompute() {
        let mut room = Room::new();
        room.phase = Phase::Live;
        room.answers.insert("s-1".into(), (0, 1));
        room.answers.insert("s-2".into(), (0, 1));
        room.close();
        assert_eq!(room.totals.unwrap()[0], 2);
        assert_eq!(room.answered_frozen, 2);

        // A late write that slipped in cannot move a frozen total (SPEC.md §4.4).
        room.answers.insert("s-3".into(), (4, 1));
        room.close();
        assert_eq!(room.totals.unwrap()[0], 2);
        assert_eq!(room.totals.unwrap()[4], 0);
        assert_eq!(room.answered_frozen, 2);
    }

    #[test]
    fn reset_clears_answers_and_bumps_the_epoch_but_keeps_connections() {
        let mut room = Room::new();
        room.present = 200;
        room.phase = Phase::Live;
        room.answers.insert("s-1".into(), (2, 9));
        room.applied_seq_sum = 9;
        room.close();

        room.reset();
        assert_eq!(room.epoch, 1);
        assert!(room.answers.is_empty());
        assert!(room.totals.is_none());
        assert_eq!(room.applied_seq_sum, 0);
        // The burst runs several cycles over the same sockets; a reset that
        // dropped `present` would make the next cycle refuse every join.
        assert_eq!(room.present, 200);
    }
}
