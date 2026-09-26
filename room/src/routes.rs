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

// T-04b routes -------------------------------------------------------------
//
// | `POST /join` `{code}` | a participant session: `201 {room_id, token, buzzer}`, or `{refusal, reason}` — `404` unknown, `409` otherwise |
// | `PUT /rooms/{id}/answer` `{letter}` | the upsert, the session token as bearer: `200 {saved}`; `409 {reason, phase, saved}` when not live; `401` unknown session; `400` bad letter |

#[derive(Deserialize)]
struct JoinBody {
    code: String,
}

#[derive(Deserialize)]
struct AnswerBody {
    letter: String,
}

async fn join(State(state): State<Arc<AppState>>, Json(body): Json<JoinBody>) -> Response {
    use crate::sessions::JoinRefusal;
    match state.join(&body.code) {
        Ok(joined) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "room_id": joined.room_id,
                "token": joined.token.as_str(),
                "buzzer": joined.buzzer,
            })),
        )
            .into_response(),
        Err(refusal) => {
            let status = match refusal {
                JoinRefusal::Unknown => StatusCode::NOT_FOUND,
                _ => StatusCode::CONFLICT,
            };
            (
                status,
                Json(serde_json::json!({ "refusal": refusal.slug(), "reason": refusal.message() })),
            )
                .into_response()
        }
    }
}

async fn answer(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<AnswerBody>,
) -> Response {
    use crate::rooms::AnswerError;
    let Some(token) = bearer(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(letter) = crate::sessions::parse_letter(&body.letter) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "reason": "An answer is one of A, B, C, D or E." })),
        )
            .into_response();
    };
    match state.answer(&id, token, letter) {
        Ok(saved) => Json(serde_json::json!({ "saved": saved })).into_response(),
        Err(AnswerError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(AnswerError::UnknownSession) => StatusCode::UNAUTHORIZED.into_response(),
        Err(AnswerError::Refused { phase, saved }) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "reason": match phase {
                    crate::phase::Phase::Idle => "Answers open when the question is on the screen.",
                    _ => crate::copy::BUZZER_CLOSED,
                },
                "phase": phase,
                "saved": saved,
            })),
        )
            .into_response(),
    }
}

fn session_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/join", post(join))
        .route("/rooms/{id}/answer", axum::routing::put(answer))
}

// end T-04b routes ---------------------------------------------------------

pub(crate) fn routes(state: Arc<AppState>) -> Router {
    let mut router = Router::new().route("/rooms", post(create));
    router = session_routes(router); // T-04b
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
    // T-07 pages ------------------------------------------------------------
    // The host phone: `/host` (the first screen, *Create a room*) and
    // `/host/{room_id}` (one screen per phase) are one page, embedded at
    // compile time with its script and style. The credential rides in the URL
    // fragment, which no request carries, so `no-store` and `no-referrer` are
    // belt and braces for the page that reads it (SPEC §8.2, AC-50).
    {
        const HOST_HTML: &str = include_str!("../../web/host/index.html");
        const HOST_JS: &str = include_str!("../../web/host/host.js");
        const HOST_CSS: &str = include_str!("../../web/host/host.css");
        fn page(body: &'static str, content_type: &'static str) -> Response {
            (
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "no-store"),
                    (header::REFERRER_POLICY, "no-referrer"),
                    (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                ],
                body,
            )
                .into_response()
        }
        let html = || async { page(HOST_HTML, "text/html; charset=utf-8") };
        router = router
            .route("/host", get(html))
            .route("/host/{room_id}", get(html))
            .route("/host/host.js", get(|| async { page(HOST_JS, "text/javascript; charset=utf-8") }))
            .route("/host/host.css", get(|| async { page(HOST_CSS, "text/css; charset=utf-8") }));
    }
    // end T-07 pages --------------------------------------------------------
    // T-04c routes ----------------------------------------------------------
    // The three sockets, and the two layers the transport needs. KEEP THIS
    // BLOCK LAST: `layer` wraps only the routes registered above it, and the
    // `notify` layer is what pushes a room after each of its writes. A block
    // added below this one would change rooms without anyone being told.
    for viewer in Viewer::ALL {
        let name = match viewer {
            Viewer::Wall => "wall",
            Viewer::Buzzer => "buzzer",
            Viewer::Host => "host",
        };
        router = router.route(
            &format!("/rooms/{{id}}/ws/{name}"),
            get(move |ws, id, transport| crate::ws::upgrade(ws, id, transport, viewer)),
        );
    }
    let default = crate::ws::Transport::new(state.clone(), Arc::new(crate::ws::NoTokens));
    let router = router
        .layer(axum::middleware::from_fn(crate::ws::notify))
        .layer(axum::middleware::from_fn_with_state(default, crate::ws::provide));
    // end T-04c routes ------------------------------------------------------
    router.with_state(state)
}
