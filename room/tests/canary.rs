//! The `canary` seam — in-process, no socket.
//!
//! `EVALUATION.md`'s harness table splits the secrecy suite in two: the
//! in-process scan, which drives the router directly and runs inside `just
//! test`, and the scan of the deployed room's real frames and pages, which runs
//! inside `just test-full`. This file is the first half's landing place.
//!
//! Today it asserts one thing, and deliberately only one: that the router can be
//! built and driven to a response without a listener. That is the whole
//! mechanism T-08 needs. T-08 then plants canaries in the resolving trace step's
//! `note`, the explanation, the receipt and the hint, and asserts the first
//! three appear in no pre-reveal payload and the hint in none before `live`
//! (AC-32, AC-47, AC-48, AC-58, AC-60, AC-79). T-25 adds `POPQUIZ_ADMIN_TOKEN`
//! as a fifth plant (AC-101).
//!
//! Nothing here runs `rustc`, Miri or Docker, and nothing here opens a port.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn the_router_can_be_driven_without_a_socket() {
    let response = room::router()
        .oneshot(
            Request::builder()
                .uri("/this-route-does-not-exist")
                .body(Body::empty())
                .expect("the request builder should accept a bare GET"),
        )
        .await
        .expect("driving the router in-process should not error");

    // The router has no routes yet, so 404 is the correct and only answer. The
    // assertion is about the harness, not about room behaviour: it proves the
    // router builds, that it answers without a listener, and that a test can
    // read the status back. Every route this asserts against arrives later.
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
