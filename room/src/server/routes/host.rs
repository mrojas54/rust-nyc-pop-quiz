//! The organizer's routes: create a room, drive it, run it again, sign in with
//! Discord, and the host phone's page.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use super::{bearer, failure};
use crate::club::ClubSlug;
use crate::phase::{Command, HostAction};
use crate::rooms::{AppState, RoomError};
use crate::view;

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
pub(super) struct QuestionBody {
    question_id: String,
    /// D-26: the club the room is for. Absent means the default club, so a
    /// host page from before clubs keeps working.
    #[serde(default)]
    club: Option<String>,
}

/// The club a request names. A name that is not a club slug at all is refused
/// as a club without the role is (AC-103): the same answer as an unknown club.
fn club_of(named: Option<&str>) -> Result<ClubSlug, RoomError> {
    match named {
        None => Ok(ClubSlug::default_club()),
        Some(raw) => ClubSlug::parse(raw).ok_or(RoomError::WrongRole),
    }
}

pub(super) async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<QuestionBody>,
) -> Response {
    let club = match club_of(body.club.as_deref()) {
        Ok(club) => club,
        Err(e) => return failure(e),
    };
    match state
        .create_room_checked_for(bearer(&headers), &club, &body.question_id, state.now())
        .await
    {
        Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
        Err(e) => failure(e),
    }
}

pub(super) async fn run_again(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<QuestionBody>,
) -> Response {
    match state.run_again_checked(&id, bearer(&headers), &body.question_id, state.now()).await {
        Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
        Err(e) => failure(e),
    }
}

pub(super) async fn act(state: Arc<AppState>, id: String, headers: HeaderMap, command: Command) -> Response {
    let token = bearer(&headers);
    let result = state
        .act(&id, token, command, state.now())
        .and_then(|()| state.with_hosted_room(&id, token, view::host));
    match result {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => failure(e),
    }
}

// T-10 routes --------------------------------------------------------------
//
// | `GET /auth/discord?question=<id>` | `303` to Discord's consent screen, scopes `identify guilds.members.read`; sets the `pq_oauth` state cookie |
// | `GET /auth/discord/callback?code&state` | `303 /host?question=<id>#<organizer session>`; a wrong, replayed or cookieless `state` is `400` with no body |
//
// The `state` is bound twice: a one-time record in the Discord backend and a
// cookie in the browser that started the sign-in (`HttpOnly`, `SameSite=Lax`
// so it survives the top-level redirect back from discord.com, scoped to
// `/auth/discord`). Both must agree, and the record is spent either way. The
// organizer session travels only in the landing URL's fragment, which no
// request carries; Discord's tokens never leave `discord.rs`. A state without
// the Discord backend (tests on `TestAuth`, `router()`) serves neither route.

fn state_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|c| c.trim().strip_prefix(crate::discord::STATE_COOKIE)?.strip_prefix('='))
}

fn cookie_attributes(state: &AppState) -> &'static str {
    if state.urls().base.starts_with("https://") {
        "Path=/auth/discord; HttpOnly; SameSite=Lax; Secure"
    } else {
        "Path=/auth/discord; HttpOnly; SameSite=Lax"
    }
}

fn see_other(location: &str, cookie: Option<String>) -> Response {
    let mut r = (
        StatusCode::SEE_OTHER,
        [
            (header::LOCATION, location.to_string()),
            (header::CACHE_CONTROL, "no-store".to_string()),
            (header::REFERRER_POLICY, "no-referrer".to_string()),
        ],
    )
        .into_response();
    if let Some(c) = cookie.and_then(|c| axum::http::HeaderValue::from_str(&c).ok()) {
        r.headers_mut().insert(header::SET_COOKIE, c);
    }
    r
}

async fn sign_in(
    State(state): State<Arc<AppState>>,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let Some(discord) = state.discord() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Ok(club) = club_of(q.get("club").map(String::as_str)) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match discord.begin(q.get("question").map(String::as_str).unwrap_or(""), &club) {
        Some((token, url)) => see_other(
            &url,
            Some(format!(
                "{}={token}; Max-Age={}; {}",
                crate::discord::STATE_COOKIE,
                crate::discord::SIGN_IN_WINDOW.as_secs(),
                cookie_attributes(&state)
            )),
        ),
        None => StatusCode::BAD_REQUEST.into_response(),
    }
}

/// `&club=<slug>` for a club that is not the default; nothing for the default,
/// so every pre-D-26 link keeps its shape (D-26).
fn club_param(club: &ClubSlug) -> String {
    if club.as_str() == crate::club::DEFAULT_CLUB {
        String::new()
    } else {
        format!("&club={club}")
    }
}

async fn signed_in(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Response {
    use crate::discord::Finish;
    let Some(discord) = state.discord() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let get = |k: &str| q.get(k).map(String::as_str);
    let finish = discord
        .finish(get("state"), state_cookie(&headers), get("code"), get("error").is_some())
        .await;
    let spent = Some(format!("{}=; Max-Age=0; {}", crate::discord::STATE_COOKIE, cookie_attributes(&state)));
    match finish {
        Finish::Refused => StatusCode::BAD_REQUEST.into_response(),
        Finish::Unavailable => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        Finish::Declined { question, club } => see_other(&format!("/host?question={question}{}", club_param(&club)), spent),
        Finish::SignedIn { question, club, session } => {
            see_other(&format!("/host?question={question}{}#{session}", club_param(&club)), spent)
        }
    }
}

pub(super) fn discord_routes(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/auth/discord", get(sign_in))
        // The literal is `discord::CALLBACK_PATH`; the redirect URI is built from
        // that constant, and `tests/auth.rs` holds the two equal.
        .route("/auth/discord/callback", get(signed_in))
}

// end T-10 routes -----------------------------------------------------------

// T-07 pages ------------------------------------------------------------
// The host phone: `/host` (the first screen, *Create a room*) and
// `/host/{room_id}` (one screen per phase) are one page, embedded at
// compile time with its script and style. The credential rides in the URL
// fragment, which no request carries, so `no-store` and `no-referrer` are
// belt and braces for the page that reads it (SPEC §8.2, AC-50).
pub(super) fn host_pages(mut router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    const HOST_HTML: &str = include_str!("../../../../web/host/index.html");
    const HOST_JS: &str = include_str!("../../../../web/host/host.js");
    const HOST_CSS: &str = include_str!("../../../../web/host/host.css");
    fn asset(body: &'static str, content_type: &'static str) -> Response {
        (
            [
                (header::CONTENT_TYPE, content_type),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            body,
        )
            .into_response()
    }
    let html = || async {
        (
            [
                (header::CACHE_CONTROL, "no-store"),
                (header::REFERRER_POLICY, "no-referrer"),
            ],
            asset(HOST_HTML, "text/html; charset=utf-8"),
        )
    };
    router = router
        .route("/host", get(html))
        .route("/host/{room_id}", get(html))
        .route("/host/host.js", get(|| async { asset(HOST_JS, "text/javascript; charset=utf-8") }))
        .route("/host/host.css", get(|| async { asset(HOST_CSS, "text/css; charset=utf-8") }));
    router
}
// end T-07 pages --------------------------------------------------------
