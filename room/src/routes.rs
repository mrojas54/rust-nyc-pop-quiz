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
use axum::{Extension, Json, Router};
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

async fn join(
    State(state): State<Arc<AppState>>,
    Extension(transport): Extension<crate::ws::Transport>,
    Json(body): Json<JoinBody>,
) -> Response {
    use crate::sessions::JoinRefusal;
    match state.join(&body.code) {
        Ok(joined) => {
            // PQ-32: `/join` is not under `/rooms/{id}/`, so the `notify` layer
            // cannot tell which room moved; the new `present` is pushed here.
            transport.changed(&joined.room_id);
            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "room_id": joined.room_id,
                    "token": joined.token.as_str(),
                    "buzzer": joined.buzzer,
                })),
            )
                .into_response()
        }
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

// T-05 pages ---------------------------------------------------------------
//
// | `GET /wall/{room_id}` | the wall page; `404` for a room that does not exist |
// | `GET /wall/wall.js`, `/wall/wall.css`, `/wall/qr.js` | the wall's own files |
// | `GET /shared/{file}` | `web/shared/*` — tokens, fonts.css, components, the shared scripts |
// | `GET /shared/fonts/{file}` | the vendored fonts `fonts.css` points at (D-14) |
// | `PUT /rooms/{id}/fit` `{fit}` | the wall's measured verdict (§3.4, §5.2): `204`; `400` for anything but the four verdicts; `404` no room |
//
// `fit` has no credential because its writer is the wall (§3.4: "the wall,
// after each refit"), and the wall has none — its socket and projection are
// public. The worst a stranger holding a room's 32-hex id can do is change the
// host's fit line; it carries no answer and moves no phase.
//
// Every file is embedded at compile time (`include_str!`/`include_bytes!`): no
// new crate, no `ServeDir`, nothing read from disk at run time, and a name not
// in these lists is `404` — there is no path from a URL to the filesystem.
// The wall page is the same bytes in every phase: it draws each phase from the
// wall socket's frames, so it cannot carry an answer before `reveal` (G-3).

const WALL_HTML: &str = include_str!("../../web/wall/index.html");

type Asset = (&'static str, &'static str, &'static [u8]);

const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

const WALL_ASSETS: &[Asset] = &[
    ("wall.js", JS, include_str!("../../web/wall/wall.js").as_bytes()),
    ("wall.css", CSS, include_str!("../../web/wall/wall.css").as_bytes()),
    ("qr.js", JS, include_str!("../../web/wall/qr.js").as_bytes()),
];

const SHARED_ASSETS: &[Asset] = &[
    ("tokens.css", CSS, include_str!("../../web/shared/tokens.css").as_bytes()),
    ("fonts.css", CSS, include_str!("../../web/shared/fonts.css").as_bytes()),
    ("components.css", CSS, include_str!("../../web/shared/components.css").as_bytes()),
    ("dom.js", JS, include_str!("../../web/shared/dom.js").as_bytes()),
    ("phase.js", JS, include_str!("../../web/shared/phase.js").as_bytes()),
    ("check.js", JS, include_str!("../../web/shared/check.js").as_bytes()),
    ("well.js", JS, include_str!("../../web/shared/well.js").as_bytes()),
    ("trace.js", JS, include_str!("../../web/shared/trace.js").as_bytes()),
    ("typemodel.js", JS, include_str!("../../web/shared/typemodel.js").as_bytes()),
    ("copy.js", JS, include_str!("../../web/shared/copy.js").as_bytes()),
];

const FONT_ASSETS: &[Asset] = &[
    (
        "CascadiaMono-VariableFont_wght.ttf",
        "font/ttf",
        include_bytes!("../../web/shared/fonts/CascadiaMono-VariableFont_wght.ttf"),
    ),
    (
        "CascadiaMono-Italic-VariableFont_wght.ttf",
        "font/ttf",
        include_bytes!("../../web/shared/fonts/CascadiaMono-Italic-VariableFont_wght.ttf"),
    ),
    (
        "InstrumentSerif-Regular.ttf",
        "font/ttf",
        include_bytes!("../../web/shared/fonts/InstrumentSerif-Regular.ttf"),
    ),
    (
        "InstrumentSerif-Italic.ttf",
        "font/ttf",
        include_bytes!("../../web/shared/fonts/InstrumentSerif-Italic.ttf"),
    ),
];

fn asset(list: &[Asset], file: &str) -> Response {
    match list.iter().find(|(name, _, _)| *name == file) {
        Some((_, ty, bytes)) => ([(header::CONTENT_TYPE, *ty)], *bytes).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn wall_page(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match state.with_room(&id, |_| ()) {
        Ok(()) => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store"),
            ],
            WALL_HTML,
        )
            .into_response(),
        Err(e) => failure(e),
    }
}

#[derive(Deserialize)]
struct FitBody {
    fit: String,
}

async fn record_fit(State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(body): Json<FitBody>) -> Response {
    use crate::rooms::Fit;
    let fit = match body.fit.as_str() {
        "fits" => Fit::Fits,
        "clipped_x" => Fit::ClippedX,
        "clipped_y" => Fit::ClippedY,
        "clipped_xy" => Fit::ClippedXy,
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    match state.record_fit(&id, fit) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => failure(e),
    }
}

fn wall_pages(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/rooms/{id}/fit", axum::routing::put(record_fit))
        .route("/wall/{room_id}", get(wall_page))
        .route("/wall/wall.js", get(|| async { asset(WALL_ASSETS, "wall.js") }))
        .route("/wall/wall.css", get(|| async { asset(WALL_ASSETS, "wall.css") }))
        .route("/wall/qr.js", get(|| async { asset(WALL_ASSETS, "qr.js") }))
        .route("/shared/{file}", get(|Path(file): Path<String>| async move { asset(SHARED_ASSETS, &file) }))
        .route(
            "/shared/fonts/{file}",
            get(|Path(file): Path<String>| async move { asset(FONT_ASSETS, &file) }),
        )
}

// end T-05 pages -----------------------------------------------------------

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
    router = wall_pages(router); // T-05 pages
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
    // PQ-32: by default a buzzer's token resolves against the real session
    // map (`impl ws::SessionTokens for AppState`). A test that wants another
    // map layers its own `Extension(Transport)`; `provide` keeps it.
    let default = crate::ws::Transport::new(state.clone(), state.clone());
    let router = router
        .layer(axum::middleware::from_fn(crate::ws::notify))
        .layer(axum::middleware::from_fn_with_state(default, crate::ws::provide));
    // end T-04c routes ------------------------------------------------------
    router.with_state(state)
}
