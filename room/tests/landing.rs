//! The front door: `GET /` serves the landing page and its two assets, and
//! `/join` and `/last` (its two exits) are still where the links point.

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use common::*;
use room::rooms::{AppState, Urls};
use tower::ServiceExt;

async fn get(app: &axum::Router, uri: &str) -> (StatusCode, String, String) {
    let res = app.clone().oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = res.status();
    let kind = res.headers().get(header::CONTENT_TYPE).map(|v| v.to_str().unwrap().to_owned()).unwrap_or_default();
    let body = axum::body::to_bytes(res.into_body(), 1 << 20).await.unwrap();
    (status, kind, String::from_utf8_lossy(&body).into_owned())
}

#[tokio::test]
async fn the_root_is_the_landing_page_not_a_404() {
    let state = Arc::new(AppState::new(Arc::new(TestAuth), vec![q3()], Urls::default()));
    let app = room::router_with(state);

    let (status, kind, page) = get(&app, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(kind.starts_with("text/html"), "{kind}");
    assert!(page.contains(r#"id="land-copy""#));

    for (path, want) in [("/landing/landing.js", "text/javascript"), ("/landing/landing.css", "text/css"), ("/landing/ferris.png", "image/png")] {
        let (status, kind, body) = get(&app, path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(kind.starts_with(want), "{path}: {kind}");
        assert!(!body.is_empty(), "{path}");
    }

    // Its exits.
    assert_eq!(get(&app, "/join").await.0, StatusCode::OK);
    assert_eq!(get(&app, "/last").await.0, StatusCode::OK);
    assert_eq!(get(&app, "/host").await.0, StatusCode::OK);
}
