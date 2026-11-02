//! The pipeline's channel (SPEC §8.3), mounted on its own router.

use std::sync::Arc;

use axum::Router;

use crate::rooms::AppState;

// T-25 routes --------------------------------------------------------------
//
// SPEC §8.3's pipeline channel. Every path under `/admin`, with any method,
// goes to one handler, `admin::serve`, whose first act is the token check; it
// dispatches `PUT /admin/questions/{id}` and `GET /admin/used` only after it.
// Its own router, with its own state (`admin::Admin`), merged in `lib.rs`
// after the T-04c layers: the admin routes move no room, so nothing needs
// telling, and no room handler's `AppState` can reach the token. Nothing
// else in this file serves a path under `/admin`.
pub(crate) fn admin_routes(state: Arc<AppState>, token: crate::admin::AdminToken) -> Router {
    Router::new()
        .route("/admin", axum::routing::any(crate::admin::serve))
        .route("/admin/", axum::routing::any(crate::admin::serve))
        .route("/admin/{*rest}", axum::routing::any(crate::admin::serve))
        .with_state(Arc::new(crate::admin::Admin::new(token, state)))
}
// end T-25 routes ----------------------------------------------------------
