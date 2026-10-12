//! The wall's routes: its page, its assets, and the fit it reports.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use super::failure;
use crate::rooms::AppState;

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

const WALL_HTML: &str = include_str!("../../../../web/wall/index.html");

type Asset = (&'static str, &'static str, &'static [u8]);

const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

const WALL_ASSETS: &[Asset] = &[
    (
        "wall.js",
        JS,
        include_str!("../../../../web/wall/wall.js").as_bytes(),
    ),
    (
        "wall.css",
        CSS,
        include_str!("../../../../web/wall/wall.css").as_bytes(),
    ),
    (
        "qr.js",
        JS,
        include_str!("../../../../web/wall/qr.js").as_bytes(),
    ),
];

const SHARED_ASSETS: &[Asset] = &[
    (
        "tokens.css",
        CSS,
        include_str!("../../../../web/shared/tokens.css").as_bytes(),
    ),
    (
        "fonts.css",
        CSS,
        include_str!("../../../../web/shared/fonts.css").as_bytes(),
    ),
    (
        "components.css",
        CSS,
        include_str!("../../../../web/shared/components.css").as_bytes(),
    ),
    (
        "dom.js",
        JS,
        include_str!("../../../../web/shared/dom.js").as_bytes(),
    ),
    (
        "phase.js",
        JS,
        include_str!("../../../../web/shared/phase.js").as_bytes(),
    ),
    (
        "check.js",
        JS,
        include_str!("../../../../web/shared/check.js").as_bytes(),
    ),
    (
        "well.js",
        JS,
        include_str!("../../../../web/shared/well.js").as_bytes(),
    ),
    (
        "trace.js",
        JS,
        include_str!("../../../../web/shared/trace.js").as_bytes(),
    ),
    (
        "typemodel.js",
        JS,
        include_str!("../../../../web/shared/typemodel.js").as_bytes(),
    ),
    (
        "copy.js",
        JS,
        include_str!("../../../../web/shared/copy.js").as_bytes(),
    ),
];

const FONT_ASSETS: &[Asset] = &[
    (
        "CascadiaMono-VariableFont_wght.ttf",
        "font/ttf",
        include_bytes!("../../../../web/shared/fonts/CascadiaMono-VariableFont_wght.ttf"),
    ),
    (
        "CascadiaMono-Italic-VariableFont_wght.ttf",
        "font/ttf",
        include_bytes!("../../../../web/shared/fonts/CascadiaMono-Italic-VariableFont_wght.ttf"),
    ),
    (
        "InstrumentSerif-Regular.ttf",
        "font/ttf",
        include_bytes!("../../../../web/shared/fonts/InstrumentSerif-Regular.ttf"),
    ),
    (
        "InstrumentSerif-Italic.ttf",
        "font/ttf",
        include_bytes!("../../../../web/shared/fonts/InstrumentSerif-Italic.ttf"),
    ),
    (
        "CascadiaMono-Italic-VariableFont_wght.woff2",
        "font/woff2",
        include_bytes!("../../../../web/shared/fonts/CascadiaMono-Italic-VariableFont_wght.woff2"),
    ),
    (
        "InstrumentSerif-Italic.woff2",
        "font/woff2",
        include_bytes!("../../../../web/shared/fonts/InstrumentSerif-Italic.woff2"),
    ),
    (
        "InstrumentSerif-Regular.woff2",
        "font/woff2",
        include_bytes!("../../../../web/shared/fonts/InstrumentSerif-Regular.woff2"),
    ),
    (
        "CascadiaMono-VariableFont_wght.woff2",
        "font/woff2",
        include_bytes!("../../../../web/shared/fonts/CascadiaMono-VariableFont_wght.woff2"),
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

async fn record_fit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<FitBody>,
) -> Response {
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

pub(super) fn wall_pages(router: Router<Arc<AppState>>) -> Router<Arc<AppState>> {
    router
        .route("/rooms/{id}/fit", axum::routing::put(record_fit))
        .route("/wall/{room_id}", get(wall_page))
        .route(
            "/wall/wall.js",
            get(|| async { asset(WALL_ASSETS, "wall.js") }),
        )
        .route(
            "/wall/wall.css",
            get(|| async { asset(WALL_ASSETS, "wall.css") }),
        )
        .route("/wall/qr.js", get(|| async { asset(WALL_ASSETS, "qr.js") }))
        .route(
            "/shared/{file}",
            get(|Path(file): Path<String>| async move { asset(SHARED_ASSETS, &file) }),
        )
        .route(
            "/shared/fonts/{file}",
            get(|Path(file): Path<String>| async move { asset(FONT_ASSETS, &file) }),
        )
}

// end T-05 pages -----------------------------------------------------------
