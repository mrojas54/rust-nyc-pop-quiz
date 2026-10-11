//! The public pages: take it home (`/last`) and the front door (`/`).

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

use crate::club::ClubSlug;
use crate::rooms::AppState;

// T-12 pages -----------------------------------------------------------
// Take it home (SPEC §13): `GET /last` is the last released question, and
// its two assets live under `/home/`. The page is embedded at compile time
// like the others; what changes per request is the snapshot, written into
// the page as JSON from `AppState::take_home()` on every request, so a
// release is on the page the moment it happens (§13: rebuilt at every
// release). `null` before the first release, and the page says so.
//
// The snapshot is the question after its room was released: it carries the
// answer, and it may (§13). Before any release there is nothing to carry,
// and a room that is still running never reaches it — the snapshot is
// written by the `released` transition only (`used.rs`).
pub(super) fn take_home_pages(mut router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    const HOME_HTML: &str = include_str!("../../../../web/home/index.html");
    const SNAPSHOT_SLOT: &str = "<script type=\"application/json\" id=\"take-home\">null</script>";

    /// JSON inside a `<script>` element: nothing in it may close the
    /// element or open a comment, whatever the question's prose says.
    fn script_json(v: &impl serde::Serialize) -> String {
        serde_json::to_string(v)
            .expect("the snapshot serializes")
            .replace('<', "\\u003c")
            .replace('>', "\\u003e")
            .replace('&', "\\u0026")
            .replace('\u{2028}', "\\u2028")
            .replace('\u{2029}', "\\u2029")
    }
    fn asset(content_type: &'static str, body: &'static str) -> Response {
        (
            [
                (header::CONTENT_TYPE, content_type),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            body,
        )
            .into_response()
    }
    async fn last(State(state): State<Arc<AppState>>) -> Response {
        last_of(state, Some(ClubSlug::default_club()))
    }
    // D-26: each club's own last released question. An unknown or
    // malformed club is the page with nothing on it, as before a first
    // release: it says nothing about which clubs exist.
    async fn last_club(State(state): State<Arc<AppState>>, Path(club): Path<String>) -> Response {
        last_of(state, ClubSlug::parse(&club))
    }
    fn last_of(state: Arc<AppState>, club: Option<ClubSlug>) -> Response {
        let slot = format!(
            "<script type=\"application/json\" id=\"take-home\">{}</script>",
            script_json(&club.and_then(|c| state.take_home_of(&c)))
        );
        (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            HOME_HTML.replacen(SNAPSHOT_SLOT, &slot, 1),
        )
            .into_response()
    }
    assert!(HOME_HTML.contains(SNAPSHOT_SLOT), "web/home/index.html lost its snapshot slot");
    router = router
        .route("/last", get(last))
        .route("/last/{club}", get(last_club))
        .route("/home/home.js", get(|| async { asset("text/javascript; charset=utf-8", include_str!("../../../../web/home/home.js")) }))
        .route("/home/home.css", get(|| async { asset("text/css; charset=utf-8", include_str!("../../../../web/home/home.css")) }));
    router
}
// end T-12 pages -------------------------------------------------------
// The front door ------------------------------------------------------
// `GET /`: a headline and two links (`/join`, `/last`), drawn by
// `landing.js` from the copy module. Embedded at compile time like the
// other pages; it carries no room state, so there is nothing to render
// per request and nothing for a participant count to see.
pub(super) fn front_door(mut router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    fn asset(content_type: &'static str, body: &'static str) -> Response {
        (
            [
                (header::CONTENT_TYPE, content_type),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            body,
        )
            .into_response()
    }
    router = router
        .route("/", get(|| async { asset("text/html; charset=utf-8", include_str!("../../../../web/landing/index.html")) }))
        .route("/landing/landing.js", get(|| async { asset("text/javascript; charset=utf-8", include_str!("../../../../web/landing/landing.js")) }))
        .route(
            "/landing/ferris.png",
            get(|| async { ([(header::CONTENT_TYPE, "image/png"), (header::CACHE_CONTROL, "no-cache")], include_bytes!("../../../../web/landing/ferris.png").as_slice()) }),
        )
        .route("/landing/landing.css", get(|| async { asset("text/css; charset=utf-8", include_str!("../../../../web/landing/landing.css")) }));
    router
}
// end front door -------------------------------------------------------
