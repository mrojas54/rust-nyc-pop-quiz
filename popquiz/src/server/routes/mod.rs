//! The room's HTTP routes (T-04a).
//!
//! | Route | What |
//! |---|---|
//! | `POST /rooms` `{question_id}` | *Create a room*; `HostAuth::authorize_create` (Discord, T-10) |
//! | `POST /rooms/{id}/<action>` | one per host action and `←`/`→`; the host session |
//! | `POST /rooms/{id}/run-it-again` `{question_id}` | a **new** room; `authorize_create`, same organizer |
//! | `GET /rooms/{id}/wall`, `/buzzer` | the public state query |
//! | `GET /rooms/{id}/host` | the host's projection; the host session |
//! | `ANY /admin`, `/admin/{*rest}` | SPEC §8.3's pipeline channel: [`admin_routes`], the admin check alone |
//!
//! Every phase change goes through `Room::act`, which goes through
//! `phase::apply` (AC-45). A refusal is `409 {reason}`; a missing or wrong
//! credential is `401` with no body, so it says nothing about what exists
//! (AC-70); an unknown room is `404`. T-10: Discord's two denials are
//! `403 {refusal, reason}` — `wrong_server` or `wrong_role`, nothing else
//! (AC-70) — and Discord not answering is `503` with no body, a server error
//! and never a denial.

mod connections;
mod host;
mod pages;
mod pipeline;
mod wall;

use connections::session_routes;
pub use host::host_routes;
use host::{act, create, discord_routes, run_again};
pub(crate) use pipeline::admin_routes;
use wall::wall_pages;

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::phase::{Command, HostAction};
use crate::rooms::{AppState, RoomError};
use crate::view::{self, Viewer};

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
        // T-11: the room ended — four hours up, or quiet too long.
        RoomError::Ended(why) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "refusal": why.refusal().slug(), "reason": why.host_reason() })),
        )
            .into_response(),
        // T-10: Discord answered no, or did not answer.
        RoomError::WrongServer => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "refusal": "wrong_server", "reason": crate::copy::HOST_DENIED_WRONG_SERVER })),
        )
            .into_response(),
        RoomError::WrongRole => (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "refusal": "wrong_role", "reason": crate::copy::HOST_DENIED_WRONG_ROLE })),
        )
            .into_response(),
        RoomError::Unavailable => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn show(state: Arc<AppState>, id: String, headers: HeaderMap, viewer: Viewer) -> Response {
    let urls = state.urls().clone();
    let result = match viewer {
        Viewer::Host => {
            state.with_hosted_room(&id, bearer(&headers), |r| view::project(r, viewer, &urls))
        }
        Viewer::Wall | Viewer::Buzzer => state.with_room(&id, |r| view::project(r, viewer, &urls)),
    };
    match result {
        Ok(payload) => Json(payload).into_response(),
        Err(e) => failure(e),
    }
}

pub(crate) fn routes(state: Arc<AppState>) -> Router {
    let mut router = Router::new().route("/rooms", post(create));
    router = session_routes(router); // T-04b
    for (command, path) in host_routes() {
        router = match command {
            Command::Host(HostAction::RunItAgain) => router.route(&path, post(run_again)),
            _ => router.route(
                &path,
                post(
                    move |State(s): State<Arc<AppState>>,
                          Path(id): Path<String>,
                          headers: HeaderMap| { act(s, id, headers, command) },
                ),
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
            get(
                move |State(s): State<Arc<AppState>>,
                      Path(id): Path<String>,
                      headers: HeaderMap| { show(s, id, headers, viewer) },
            ),
        );
    }
    router = wall_pages(router); // see T-05 pages
    router = host::host_pages(router); // see T-07 pages
    router = connections::buzzer_pages(router); // see T-06 pages
    router = pages::take_home_pages(router); // see T-12 pages
    router = pages::front_door(router); // the front door
    router = discord_routes(router); // see T-10 routes
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
    // T-11: the reaper sweeps ended rooms and closes their sockets through the
    // default transport. Only inside a tokio runtime.
    crate::server::reaper::spawn_reaper(state.clone(), default.clone());
    let router = router
        // T-21: AC-55's log — participant requests counted, 5xx logged, the
        // room's summary at release (`requestlog.rs`).
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::requestlog::layer,
        ))
        .layer(axum::middleware::from_fn(crate::ws::notify))
        .layer(axum::middleware::from_fn_with_state(
            default,
            crate::ws::provide,
        ));
    // end T-04c routes ------------------------------------------------------
    router.with_state(state)
}
