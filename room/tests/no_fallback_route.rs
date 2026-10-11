//! The static fallback and its host sheet are never served (SPEC §12, D-24).
//!
//! Both files hold the answer and live on the organizer's laptop; `popquiz
//! schedule` writes them there (T-26, T-20). So no route of the room carries
//! either, and the wall still shows none of the host's words (AC-39):
//!
//! - no route path literal in the room's source names `sheet` or `fallback`;
//! - the real router answers `404` to the paths either could have been served at,
//!   including the static driver the fallback file inlines.

use std::fs;
use std::path::PathBuf;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

fn src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every string literal passed to `.route(` in the room's source, including
/// the ones built with `format!` for the host actions.
fn route_literals() -> Vec<String> {
    let mut out = Vec::new();
    for entry in fs::read_dir(src()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        for (i, _) in text.match_indices(".route(") {
            let rest = &text[i..];
            let end = rest.find(')').unwrap_or(rest.len());
            // The first argument, whatever it is; a path is its first quoted run.
            if let Some(q) = rest[..end].find('"') {
                let lit = &rest[q + 1..];
                if let Some(close) = lit.find('"') {
                    out.push(lit[..close].to_string());
                }
            }
        }
        // Paths assembled with format! (the host actions: "/rooms/{{id}}/{slug}").
        for (i, _) in text.match_indices("format!(\"/") {
            let lit = &text[i + "format!(\"".len()..];
            if let Some(close) = lit.find('"') {
                out.push(lit[..close].to_string());
            }
        }
    }
    out
}

#[test]
fn no_route_names_a_sheet_or_a_fallback() {
    let routes = route_literals();
    assert!(
        routes.iter().any(|r| r == "/wall/{room_id}") && routes.len() >= 10,
        "the scan found the router's paths: {routes:?}"
    );
    for r in &routes {
        let lower = r.to_lowercase();
        assert!(
            !lower.contains("sheet") && !lower.contains("fallback"),
            "a room route serves the host sheet or the fallback: {r}"
        );
    }
}

#[tokio::test]
async fn the_router_serves_neither_file() {
    let app = room::router();
    for uri in [
        "/fallback",
        "/fallback.html",
        "/q3.html",
        "/q3.host-sheet.txt",
        "/host-sheet",
        "/sheet",
        "/wall/static.js",
        "/wall/fallback/static.js",
        "/wall/fallback.html",
        "/shared/static.js",
        "/shared/fallback.html",
        "/shared/q3.host-sheet.txt",
    ] {
        let res = app
            .clone()
            .oneshot(Request::builder().method(Method::GET).uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NOT_FOUND, "{uri} is served");
    }
}
