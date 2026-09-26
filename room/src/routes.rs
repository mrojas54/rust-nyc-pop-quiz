//! The room's HTTP routes (T-04a).
//!
//! | Route | What |
//! |---|---|
//! | `POST /rooms` `{question_id}` | *Create a room*; `HostAuth::authorize_create` |
//! | `POST /rooms/{id}/<action>` | one per host action and `←`/`→`; the host session |
//! | `POST /rooms/{id}/run-it-again` `{question_id}` | a **new** room; `authorize_create`, same organizer |
//! | `GET /rooms/{id}/wall`, `/buzzer` | the public state query |
//! | `GET /rooms/{id}/host` | the host's projection; the host session |
//!
//! Every phase change goes through `Room::act`, which goes through
//! `phase::apply` (AC-45). A refusal is `409 {reason}`; a missing or wrong
//! credential is `401` with no body, so it says nothing about what exists
//! (AC-70); an unknown room is `404`.

use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use crate::phase::{Command, HostAction};
use crate::rooms::{AppState, RoomError};
use crate::view::{self, Viewer};

/// Every host command that has a route on a room, and its path. *Create a
/// room* is `POST /rooms` and has no room yet, so it is not in this list.
pub fn host_routes() -> Vec<(Command, String)> {
    Command::all()
        .into_iter()
        .filter(|c| *c != Command::Host(HostAction::CreateRoom))
        .map(|c| (c, format!("/rooms/{{id}}/{}", c.slug())))
        .collect()
}

#[derive(Deserialize)]
struct QuestionBody {
    question_id: String,
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

fn failure(e: RoomError) -> Response {
    match e {
        RoomError::NotFound => StatusCode::NOT_FOUND.into_response(),
        RoomError::Denied => StatusCode::UNAUTHORIZED.into_response(),
        RoomError::Refused(reason) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "reason": reason })),
        )
            .into_response(),
    }
}

async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<QuestionBody>,
) -> Response {
    match state.create_room(bearer(&headers), &body.question_id, SystemTime::now()) {
        Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
        Err(e) => failure(e),
    }
}

async fn run_again(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<QuestionBody>,
) -> Response {
    match state.run_again(&id, bearer(&headers), &body.question_id, SystemTime::now()) {
        Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
        Err(e) => failure(e),
    }
}

async fn act(state: Arc<AppState>, id: String, headers: HeaderMap, command: Command) -> Response {
    let token = bearer(&headers);
    let result = state
        .act(&id, token, command, SystemTime::now())
        .and_then(|()| state.with_hosted_room(&id, token, view::host));
    match result {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => failure(e),
    }
}

async fn show(state: Arc<AppState>, id: String, headers: HeaderMap, viewer: Viewer) -> Response {
    let urls = state.urls().clone();
    let result = match viewer {
        Viewer::Host => state.with_hosted_room(&id, bearer(&headers), |r| view::project(r, viewer, &urls)),
        Viewer::Wall | Viewer::Buzzer => state.with_room(&id, |r| view::project(r, viewer, &urls)),
    };
    match result {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => failure(e),
    }
}

pub(crate) fn routes(state: Arc<AppState>) -> Router {
    let mut router = Router::new().route("/rooms", post(create));
    for (command, path) in host_routes() {
        router = match command {
            Command::Host(HostAction::RunItAgain) => router.route(&path, post(run_again)),
            _ => router.route(
                &path,
                post(move |State(s): State<Arc<AppState>>, Path(id): Path<String>, headers: HeaderMap| {
                    act(s, id, headers, command)
                }),
            ),
        };
    }
    for viewer in Viewer::ALL {
        let name = match viewer {
            Viewer::Wall => "wall",
            Viewer::Buzzer => "buzzer",
            Viewer::Host => "host",
        };
        router = router.route(
            &format!("/rooms/{{id}}/{name}"),
            get(move |State(s): State<Arc<AppState>>, Path(id): Path<String>, headers: HeaderMap| {
                show(s, id, headers, viewer)
            }),
        );
    }
    router.with_state(state)
}
