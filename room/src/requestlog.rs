//! AC-55's evidence (T-21): failed participant requests, as the room saw them.
//!
//! AC-55 — *failed participant requests stay under 0.1% across a full session
//! on venue wifi* — is settled by the client at HC-4 from the room's own logs.
//! This module writes those logs: one line per failed participant request, and
//! one summary per room when it ends. Each line is one JSON object on standard
//! error, which Fly collects (`fly logs`; room/README.md, *After a meetup*) —
//! the same sink `discord.rs` writes to. Not `tracing`: the crate is in the
//! lock but no subscriber is, and a `tracing` event without one prints nothing.
//!
//! **What a participant request is** (the denominator): `POST /join` that
//! found a room, `PUT /rooms/{id}/answer`, `GET /rooms/{id}/buzzer`, and each
//! buzzer socket that attached — to a room that has not been released, so a
//! phone re-reading an ended room never reopens its counters.
//!
//! **What failed** (the numerator), one line each:
//! - `server_error`: any `5xx` answer to one of those routes — the room failed
//!   the phone;
//! - `socket_dropped`: an attached buzzer socket that ended while its room was
//!   still running (not yet `released`) without a close frame: the
//!   connection broke (a read error or end of stream — the network, but also a
//!   phone locked or a tab killed, which the room cannot tell apart), or the
//!   room could not send to it within its send timeout and let it go.
//!
//! **Not failures**, because the room answered correctly: `409` refusals (a
//! closed room, one that ended, a full one), `404` (no such room or code),
//! `401` (a token the room does not know), `400` (a malformed body), a close
//! frame from the phone (a page leaving or reloading sends one), a socket replaced by the same
//! phone re-attaching, and sockets the room closes itself when it ends.
//!
//! **What the room cannot see** — a request venue wifi lost before it reached
//! the room — is in no log. The rate these lines give is therefore a floor.
//!
//! Fields are fixed and carry no secret: `event`, `room` (the room id; `null`
//! for a `/join` that failed before it found one), `route` (a name, never the
//! request's path), `kind`, and `status` or `phase`; the summary carries
//! `room`, `failed`, `total` and `ended` (`released`, or `gone` when a watched
//! room was swept without a release). Never a token, a host session, a code or
//! an answer.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware::Next;
use axum::response::Response;
use serde_json::{json, Value};

use crate::rooms::AppState;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts {
    pub failed: u64,
    pub total: u64,
}

/// Per-room counters, keyed by room id. Room ids are unique random values, so
/// rooms (and parallel tests) never share an entry; an entry lives from the
/// room's first participant request to its summary.
static COUNTS: LazyLock<Mutex<HashMap<String, Counts>>> = LazyLock::new(Default::default);

fn counts() -> MutexGuard<'static, HashMap<String, Counts>> {
    // A poisoned lock still holds good counters; logging must never panic.
    COUNTS.lock().unwrap_or_else(|e| e.into_inner())
}

static CAPTURE: Mutex<Option<Vec<String>>> = Mutex::new(None);

fn emit(line: Value) {
    let text = line.to_string();
    eprintln!("{text}");
    if let Some(lines) = CAPTURE.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        lines.push(text);
    }
}

/// Tests only: keep every line from now on, beside standard error. Nothing is
/// kept unless a test asks, so a serving room never accumulates its log.
pub fn start_capture() {
    CAPTURE.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Vec::new);
}

/// The captured lines about `room`, parsed. Empty unless [`start_capture`] ran.
pub fn captured(room: &str) -> Vec<Value> {
    CAPTURE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .flatten()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["room"] == room)
        .collect()
}

/// Every captured line, raw — for scans that look for secrets in all of them.
pub fn captured_all() -> Vec<String> {
    CAPTURE.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_default()
}

/// This room's counters so far.
pub fn counts_for(room: &str) -> Counts {
    counts().get(room).copied().unwrap_or_default()
}

/// One participant request to `room`.
pub fn request(room: &str) {
    counts().entry(room.to_string()).or_default().total += 1;
}

/// A participant request that answered `5xx`.
pub fn server_error(room: Option<&str>, route: &'static str, status: u16) {
    if let Some(room) = room {
        counts().entry(room.to_string()).or_default().failed += 1;
    }
    emit(json!({ "event": "participant_request_failed", "room": room, "route": route, "kind": "server_error", "status": status }));
}

/// An attached buzzer socket that broke while `room` was in `phase`.
pub fn socket_dropped(room: &str, phase: Value) {
    counts().entry(room.to_string()).or_default().failed += 1;
    emit(json!({ "event": "participant_request_failed", "room": room, "route": "buzzer_socket", "kind": "socket_dropped", "phase": phase }));
}

/// The room ended: its summary line, once, and its counters dropped. A room
/// nobody ever sent a participant request is summarized as `0 of 0`.
pub fn ended(room: &str, how: &'static str) {
    let c = counts().remove(room);
    if c.is_none() && how == "gone" {
        return; // already summarized at release, or never used
    }
    let c = c.unwrap_or_default();
    emit(json!({ "event": "participant_requests", "room": room, "failed": c.failed, "total": c.total, "ended": how }));
}

/// Which participant route a request is, and the room id in its path. `None`
/// for everything else (host routes, pages, the wall, the admin channel).
pub fn participant_route<'a>(method: &Method, path: &'a str) -> Option<(&'static str, Option<&'a str>)> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match (method, segments.as_slice()) {
        (&Method::POST, ["join"]) => Some(("join", None)),
        (&Method::PUT, ["rooms", id, "answer"]) => Some(("answer", Some(id))),
        (&Method::GET, ["rooms", id, "buzzer"]) => Some(("buzzer_state", Some(id))),
        _ => None,
    }
}

/// A room that exists and has not been released — the only kind whose
/// participant requests are counted.
pub fn running(state: &AppState, room: &str) -> bool {
    state.with_room(room, |r| r.phase() != crate::phase::Phase::Released).unwrap_or(false)
}

/// `POST /rooms/{id}/release` — the host's release, whose `200` ends the room.
fn release_of<'a>(method: &Method, path: &'a str) -> Option<&'a str> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match (method, segments.as_slice()) {
        (&Method::POST, ["rooms", id, "release"]) => Some(id),
        _ => None,
    }
}

/// The layer over the room's routes. Counts participant requests to a room
/// that exists (a successful join is counted by its handler, which alone
/// knows the room), logs every `5xx` among them, and writes the summary when
/// a release succeeds.
pub(crate) async fn layer(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status();
    if let Some((route, room)) = participant_route(&method, &path) {
        // Only a room that is still running: after release its summary is
        // written, and a phone re-reading the ended room must not reopen it.
        let room = room.filter(|id| running(&state, id));
        if let Some(id) = room {
            self::request(id);
        }
        if status.is_server_error() {
            server_error(room, route, status.as_u16());
        }
    } else if let Some(id) = release_of(&method, &path) {
        if status.is_success() {
            ended(id, "released");
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_participant_routes_are_counted() {
        assert_eq!(participant_route(&Method::POST, "/join"), Some(("join", None)));
        assert_eq!(participant_route(&Method::PUT, "/rooms/abc/answer"), Some(("answer", Some("abc"))));
        assert_eq!(participant_route(&Method::GET, "/rooms/abc/buzzer"), Some(("buzzer_state", Some("abc"))));
        for (m, p) in [
            (Method::GET, "/join"),
            (Method::GET, "/rooms/abc/ws/buzzer"),
            (Method::GET, "/rooms/abc/wall"),
            (Method::GET, "/rooms/abc/host"),
            (Method::POST, "/rooms/abc/reveal"),
            (Method::POST, "/rooms"),
            (Method::PUT, "/admin/questions/q3"),
            (Method::GET, "/ABCDEF"),
        ] {
            assert_eq!(participant_route(&m, p), None, "{m} {p}");
        }
        assert_eq!(release_of(&Method::POST, "/rooms/abc/release"), Some("abc"));
        assert_eq!(release_of(&Method::GET, "/rooms/abc/release"), None);
    }

    #[test]
    fn counters_add_up_and_the_summary_ends_them_once() {
        let room = "unit-test-room-counters";
        request(room);
        request(room);
        server_error(Some(room), "answer", 503);
        socket_dropped(room, json!("live"));
        assert_eq!(counts_for(room), Counts { failed: 2, total: 2 });
        ended(room, "released");
        assert_eq!(counts_for(room), Counts::default());
        // Gone after a release says nothing more.
        start_capture();
        ended(room, "gone");
        assert!(captured(room).is_empty());
    }
}
