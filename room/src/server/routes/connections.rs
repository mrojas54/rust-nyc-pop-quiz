//! The audience's routes: join by code, answer, and the buzzer page.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::Deserialize;

use super::bearer;
use crate::rooms::AppState;

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
            crate::requestlog::request(&joined.room_id); // T-21: AC-55's denominator
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
        // T-11: a buzzer on a room that ended hears PQ-8's sentence for it.
        Err(AnswerError::Ended(why)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "refusal": why.refusal().slug(), "reason": why.refusal().message() })),
        )
            .into_response(),
    }
}

pub(super) fn session_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/join", post(join))
        .route("/rooms/{id}/answer", axum::routing::put(answer))
}

// end T-04b routes ---------------------------------------------------------

// T-06 pages -----------------------------------------------------------
// The buzzer: `GET /join` (typed code, or `?code=` from the link), its
// two assets under the page's own path, and the short link `GET /{code}`
// — the shape `rooms/room.rs` writes into `join_url` and the wall prints —
// redirected to `/join?code=`. Embedded at compile time; nothing on disk
// is read at runtime. The page loads `/shared/*`, which T-05 serves.
pub(super) fn buzzer_pages(mut router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    use axum::http::HeaderValue;
    use axum::response::Redirect;

    fn asset(content_type: &'static str, body: &'static str) -> Response {
        (
            [
                (header::CONTENT_TYPE, HeaderValue::from_static(content_type)),
                (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
            ],
            body,
        )
            .into_response()
    }
    async fn buzzer_page() -> Response {
        asset("text/html; charset=utf-8", include_str!("../../../../web/buzzer/index.html"))
    }
    async fn buzzer_js() -> Response {
        asset("text/javascript; charset=utf-8", include_str!("../../../../web/buzzer/buzzer.js"))
    }
    async fn buzzer_css() -> Response {
        asset("text/css; charset=utf-8", include_str!("../../../../web/buzzer/buzzer.css"))
    }
    // Only a well-formed room code redirects, so this never answers for a
    // path that is not one (`/host`, `/last`, …); those are 404 here.
    async fn short_link(Path(code): Path<String>) -> Response {
        match crate::sessions::parse_code(&code) {
            Some(code) => Redirect::to(&format!("/join?code={code}")).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
    router = router
        .route("/join", get(buzzer_page))
        .route("/join/buzzer.js", get(buzzer_js))
        .route("/join/buzzer.css", get(buzzer_css))
        .route("/{code}", get(short_link));
    router
}
// end T-06 pages -------------------------------------------------------
