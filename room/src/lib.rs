//! The live room.
//!
//! This is scaffold. The crate builds, serves, and can be driven in-process by a
//! test with no socket; it has no room behaviour yet. The modules that will hold
//! that behaviour are named in `BUILDPLAN.md` section 2 — `auth`, `rooms`,
//! `answers`, `ws`, `phase` — and each arrives with its own ticket. `answers` is
//! the sealed one: unreachable from the public state query, proven by a module
//! boundary rather than by convention (SPEC.md G-3, AC-61).

use axum::Router;

/// The room's HTTP surface.
///
/// It has no routes. That is the point of it today: `router()` exists so that a
/// test can drive the whole surface in-process, and so that every later ticket
/// has one place to add its routes rather than each building its own server.
///
/// T-04a adds the phase machine's routes, T-25 the two admin routes behind the
/// bearer check (SPEC.md 8.3), T-09 whatever deploy needs.
pub fn router() -> Router {
    Router::new()
}
