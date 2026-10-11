//! SPEC §8.3's pipeline channel (D-20, AC-101, T-25): how the organizer's
//! laptop reaches the room server.
//!
//! | Route | What |
//! |---|---|
//! | `PUT /admin/questions/{id}` | `popquiz schedule`: the question record, answer included, into the sealed [`crate::answers`] module, for the default club |
//! | `PUT /admin/clubs/{club}/questions/{id}` | the same, for `club` (D-26): the record is the question arranged for that club's meetup date |
//! | `GET /admin/used` | `popquiz sync`: the used-question ledger |
//!
//! Both require `Authorization: Bearer ‹token›`, the token being the value of
//! [`VAR`], a Fly secret the organizer's pipeline also holds and that is never
//! in the repository (H-11). The rules this module keeps:
//!
//! - **One check** ([`AdminToken::check`]), one constant-time compare. It is
//!   the admin path's whole auth function (G-9); nothing else in the room
//!   reads the token, and the token authorizes nothing else.
//! - **The prefix is served to the check alone.** Every path under `/admin`
//!   reaches [`serve`], whose first act is the check. A missing or wrong token
//!   is `401` with an empty body and no `WWW-Authenticate`, before the method,
//!   the path or the body is looked at: a prober cannot tell a scheduled id
//!   from an unknown one, or a real route from a made-up one.
//! - **The token is its own state.** The admin routes are a separate router
//!   whose state is [`Admin`]; participant, wall and host handlers extract
//!   `AppState`, which has no path to it.
//! - **Nothing here logs**, and [`AdminToken`] has no `Debug`, `Display` or
//!   `Clone`, so the token cannot be formatted by accident.
//! - **The answer enters the sealed module and nowhere else** (AC-61): the
//!   body goes to [`answers::load`] whole, and what comes back is a
//!   `Scheduled`, which this module hands to [`AppState::schedule`] unopened.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use subtle::ConstantTimeEq;

use crate::answers;
use crate::auth::Denied;
use crate::club::ClubSlug;
use crate::config::ConfigError;
use crate::rooms::{AppState, RoomError, Scheduling};

/// The variable the token is read from — the one place in `src/` it is named.
pub const VAR: &str = "POPQUIZ_ADMIN_TOKEN";

/// The largest record `PUT /admin/questions/{id}` reads. A bank record is a
/// few kilobytes; anything near this is not one.
pub const MAX_RECORD_BYTES: usize = 1 << 20;

/// The admin token, held as bytes. No `Debug`, `Display` or `Clone`.
pub struct AdminToken {
    token: Box<[u8]>,
}

impl AdminToken {
    /// The token from the variable's value. Missing, empty or blank is an
    /// error: a channel no token opens is broken, not closed.
    pub fn from_var(value: Option<String>) -> Result<AdminToken, ConfigError> {
        match value {
            Some(v) if !v.trim().is_empty() => Ok(AdminToken {
                token: v.into_bytes().into_boxed_slice(),
            }),
            _ => Err(ConfigError::MissingAdminToken),
        }
    }

    /// A token nobody holds: 32 bytes from the OS, hex-encoded, never shown.
    /// What `router()` and `router_with()` serve, so a router built without
    /// configuration runs the same check and opens to no one.
    pub fn unguessable() -> AdminToken {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).expect("the OS random source is available");
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        AdminToken {
            token: hex.into_bytes().into_boxed_slice(),
        }
    }

    /// The one check. A missing header, a scheme other than `Bearer`, and a
    /// wrong token take the same path: each compares a presented slice (empty
    /// when there is none) against the token in constant time. `ct_eq` on
    /// slices of different lengths is `false` without reading the bytes; the
    /// length of a 64-hex-character token is not the secret.
    pub fn check(&self, headers: &HeaderMap) -> Result<(), Denied> {
        let presented = headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or("");
        if bool::from(self.token.ct_eq(presented.as_bytes())) {
            Ok(())
        } else {
            Err(Denied)
        }
    }
}

/// The admin routes' state: the token and the rooms it schedules into.
pub struct Admin {
    token: AdminToken,
    rooms: Arc<AppState>,
}

impl Admin {
    pub fn new(token: AdminToken, rooms: Arc<AppState>) -> Admin {
        Admin { token, rooms }
    }
}

/// Every request under `/admin`. The check comes first; nothing about the
/// request is read until it passes.
pub async fn serve(State(admin): State<Arc<Admin>>, request: Request) -> Response {
    if admin.token.check(request.headers()).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let (parts, body) = request.into_parts();
    let path = parts.uri.path();
    let rest = path.strip_prefix("/admin").unwrap_or(path);
    let segments: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    match (segments.as_slice(), &parts.method) {
        (["questions", id], &Method::PUT) => {
            put_question(&admin.rooms, &ClubSlug::default_club(), id, body).await
        }
        // D-26: a scheduled record is the question arranged for one club's
        // meetup date, so it is pushed to that club. A name that is not a club
        // slug gets the same answer as any record the room refuses.
        (["clubs", club, "questions", id], &Method::PUT) => match ClubSlug::parse(club) {
            Some(club) => put_question(&admin.rooms, &club, id, body).await,
            None => refused(StatusCode::BAD_REQUEST, "That is not a club name."),
        },
        (["used"], &Method::GET) => Json(admin.rooms.used().all()).into_response(),
        (["questions", _], _) | (["clubs", _, "questions", _], _) | (["used"], _) => {
            StatusCode::METHOD_NOT_ALLOWED.into_response()
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

fn refused(status: StatusCode, reason: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "reason": reason.into() }))).into_response()
}

async fn put_question(rooms: &AppState, club: &ClubSlug, id: &str, body: Body) -> Response {
    let Ok(bytes) = axum::body::to_bytes(body, MAX_RECORD_BYTES).await else {
        return refused(StatusCode::PAYLOAD_TOO_LARGE, "The record is larger than a question record can be.");
    };
    let Ok(json) = std::str::from_utf8(&bytes) else {
        return refused(StatusCode::BAD_REQUEST, "The record is not UTF-8 JSON.");
    };
    let question = match answers::load(json) {
        Ok(q) => q,
        Err(e) => return refused(StatusCode::BAD_REQUEST, e.to_string()),
    };
    if question.public().id() != id {
        return refused(
            StatusCode::BAD_REQUEST,
            format!("The path names {id}; the record is {}.", question.public().id()),
        );
    }
    match rooms.schedule_for(club, question) {
        Ok(Scheduling::New) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": id, "scheduled": "new" })),
        )
            .into_response(),
        Ok(Scheduling::Replaced) => Json(serde_json::json!({ "id": id, "scheduled": "replaced" })).into_response(),
        Err(RoomError::Refused(reason)) => refused(StatusCode::CONFLICT, reason),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
