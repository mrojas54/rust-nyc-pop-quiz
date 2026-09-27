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
//! - [`sessions`] — participant sessions and the answer store (T-04b).
//! - [`view`] — the public state query: wall, buzzer and host payloads.
//! - [`auth`] — the `HostAuth` seam (G-9); T-09 and T-10 implement it.
//! - [`copy`] — SPEC §11's strings, mirrored from `web/shared/copy.js`.
//! - [`ws`] — the transport: one broadcast per room to the wall, the buzzers
//!   and the host, and reconnect with the same session token (T-04c).
//! - [`config`] — what the binary reads from its environment (T-09).
//! - `standin` — SPEC §8.2's `HOST_DEV_TOKEN` stand-in and the HC-0 seed, only
//!   with the `dev-host-token` feature (T-09; T-10 deletes it).
//!
//! `unsafe` is forbidden crate-wide: the seal is a safe-Rust guarantee, and a
//! zero-sized witness could otherwise be conjured from nothing.

#![forbid(unsafe_code)]

use std::sync::Arc;

use axum::Router;

pub mod answers;
pub mod auth;
pub mod config;
pub mod copy;
pub mod phase;
pub mod question;
pub mod rooms;
mod routes;
pub mod sessions;
#[cfg(feature = "dev-host-token")]
pub mod standin;
pub mod view;
pub mod ws;

pub use routes::host_routes;

/// The state the binary serves (T-09). Without `dev-host-token`: nothing
/// scheduled and nothing authorized ([`auth::DenyAll`]), so the room creates
/// nothing until T-10 and T-25. With it: SPEC §8.2's stand-in and the HC-0
/// question, through the same `AppState::new` seam the tests seed with.
pub fn serving_state(config: config::Config) -> rooms::AppState {
    #[cfg(feature = "dev-host-token")]
    return rooms::AppState::new(Arc::new(config.host_token), vec![standin::hc0_question()], config.urls);
    #[cfg(not(feature = "dev-host-token"))]
    rooms::AppState::new(Arc::new(auth::DenyAll), Vec::new(), config.urls)
}

/// The room's HTTP surface over a default build's state, with the sockets
/// wired to the real session map: nothing scheduled and nothing authorized
/// ([`auth::DenyAll`]), so no room can be created. The binary serves
/// [`serving_state`] instead, which is this same state unless the
/// `dev-host-token` feature is on.
pub fn router() -> Router {
    router_with(Arc::new(rooms::AppState::new(
        Arc::new(auth::DenyAll),
        Vec::new(),
        rooms::Urls::default(),
    )))
}

/// The room's HTTP surface over a given state, its sockets resolving buzzer
/// tokens against that state's sessions (PQ-32). Tests drive this in-process;
/// T-25 adds its two admin routes and T-09 whatever deploy needs.
pub fn router_with(state: Arc<rooms::AppState>) -> Router {
    routes::routes(state)
}
