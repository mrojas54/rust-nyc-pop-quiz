//! The live room: one meetup, one question, seven phases.
//!
//! The modules are the ones `BUILDPLAN.md` §2 names:
//!
//! - [`phase`] — the machine: `idle → … → released`, host actions only, no
//!   skipping, `trace_step` (G-6). Consumed by everything in M1.
//! - [`answers`] — **sealed**: everything that joins an option to the verified
//!   output, readable only with a witness that exists only in `reveal` (G-3,
//!   AC-61). The proof is a set of `compile_fail` doctests with compiling twins,
//!   plus `tests/boundary.rs`.
//! - [`question`] — the public half of a question.
//! - [`rooms`] — the room record (§3.4) and the seams for sessions (T-04b) and
//!   the socket (T-04c).
//! - [`view`] — the public state query: wall, buzzer and host payloads.
//! - [`auth`] — the `HostAuth` seam (G-9); T-09 and T-10 implement it.
//! - [`copy`] — SPEC §11's strings, mirrored from `web/shared/copy.js`.
//!
//! `unsafe` is forbidden crate-wide: the seal is a safe-Rust guarantee, and a
//! zero-sized witness could otherwise be conjured from nothing.

#![forbid(unsafe_code)]

use std::sync::Arc;

use axum::Router;

pub mod answers;
pub mod auth;
pub mod copy;
pub mod phase;
pub mod question;
pub mod rooms;
mod routes;
pub mod view;

pub use routes::host_routes;

/// The room's HTTP surface as the binary serves it today: nothing scheduled
/// and nothing authorized ([`auth::DenyAll`]), so no room can be created until
/// T-09 wires the stand-in (§8.2) or T-10 wires Discord, and T-25 or T-09
/// supplies a question.
pub fn router() -> Router {
    router_with(Arc::new(rooms::AppState::new(
        Arc::new(auth::DenyAll),
        Vec::new(),
        rooms::Urls::default(),
    )))
}

/// The room's HTTP surface over a given state. Tests drive this in-process; T-25
/// adds its two admin routes and T-09 whatever deploy needs.
pub fn router_with(state: Arc<rooms::AppState>) -> Router {
    routes::routes(state)
}
