//! The live room: one meetup, one question, seven phases.
//!
//! The modules are the ones `BUILDPLAN.md` §2 names, grouped by who uses
//! them: [`host`] (the organizer), [`connections`] (the audience's sockets and
//! sessions), [`models`] (the data) and [`server`] (config, routes, admin). Each is also reachable at the crate
//! root, so `room::phase` and `room::models::phase` are the same module.
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
//!   - `rooms/create` — the one path that makes a room: *Create a room* and
//!     *Run it again*, ending at one maker (G-9, GAP-8, G-10).
//! - [`sessions`] — participant sessions and the answer store (T-04b).
//! - [`view`] — the public state query: wall, buzzer and host payloads.
//! - [`auth`] — the `HostAuth` seam (G-9): the one create check.
//! - [`discord`] — its implementation: Discord sign-in, the organizer records
//!   (§3.5) and the role-ID check at creation (§8, T-10); its configuration
//!   and HTTP transport are private submodules.
//! - [`copy`] — SPEC §11's strings, mirrored from `web/shared/copy.js`.
//! - [`ws`] — the transport: one broadcast per room to the wall, the buzzers
//!   and the host, and reconnect with the same session token (T-04c).
//! - [`config`] — what the binary reads from its environment (T-09).
//! - [`lifecycle`] — how long a room lives and what ends it: the clock seam
//!   and §4.6's bounds (T-11); lives in `rooms/`, the reaper is `server/`.
//! - [`used`] — what outlives a room: the used-question ledger, written only
//!   at release (G-10), and the take-it-home snapshot (T-11).
//! - [`admin`] — SPEC §8.3's pipeline channel: `PUT /admin/questions/{id}`
//!   and `GET /admin/used` behind one constant-time bearer check (T-25).
//! - [`requestlog`] — AC-55's evidence: a line per failed participant request
//!   and a per-room summary, on standard error for `fly logs` (T-21).
//!
//! SPEC §8.2's M1 stand-in (a shared-secret create check behind a Cargo
//! feature, and the HC-0 question seeded with it) was deleted by T-10;
//! `tests/auth.rs` holds the build to that.
//!
//! `unsafe` is forbidden crate-wide: the seal is a safe-Rust guarantee, and a
//! zero-sized witness could otherwise be conjured from nothing.

#![forbid(unsafe_code)]

use std::sync::Arc;

use axum::Router;

pub mod connections;
pub mod copy;
pub mod host;
mod quiz_session;
pub mod server;

// The groups' modules keep their crate-root paths (`room::phase`, `room::ws`).
pub use connections::{requestlog, sessions, ws};
pub use host::{auth, discord};
pub use models::{affirmation, answers, club, phase, question, used, view};
pub use quiz_session::{lifecycle, models};
use server::routes;
pub use server::{admin, config};

pub use server::routes::host_routes;

/// The room's public path. The source lives in `quiz_session/` (ADR-0005);
/// `room::rooms::AppState`, `room::rooms::Room` and the rest stay as they were.
pub mod rooms {
    pub use crate::quiz_session::{
        lifecycle, new_code, Acted, AnswerError, AppState, CloseSnapshot, Created, Fit, Joined,
        LiveCounts, NoSessions, Opened, PublicView, Room, RoomError, Scheduling, Sessions, Totals,
        Urls, CODE_ALPHABET, ROOM_LIFETIME,
    };
}

/// The state the binary serves: *Create a room* authorized by Discord (T-10),
/// which also signs organizers in, and nothing scheduled — questions arrive
/// over the pipeline channel (T-25).
pub fn serving_state(config: config::Config) -> rooms::AppState {
    let clubs = config.discord.clubs.clone();
    let discord = Arc::new(discord::Discord::serving(config.discord));
    rooms::AppState::new(discord.clone(), Vec::new(), config.urls)
        .with_clubs(clubs)
        .with_discord(discord)
}

/// What the binary serves: [`serving_state`] behind the room's routes, and
/// the admin channel opened by the configured token (SPEC §8.3).
pub fn serving_router(mut config: config::Config) -> Router {
    let token = std::mem::replace(&mut config.admin_token, admin::AdminToken::unguessable());
    router_with_admin(Arc::new(serving_state(config)), token)
}

/// The room's HTTP surface over a default build's state, with the sockets
/// wired to the real session map: nothing scheduled and nothing authorized
/// ([`auth::DenyAll`]), so no room can be created. The binary serves
/// [`serving_state`] instead, whose create check is Discord's.
pub fn router() -> Router {
    router_with(Arc::new(rooms::AppState::new(
        Arc::new(auth::DenyAll),
        Vec::new(),
        rooms::Urls::default(),
    )))
}

/// The room's HTTP surface over a given state, its sockets resolving buzzer
/// tokens against that state's sessions (PQ-32). Tests drive this in-process.
/// The admin routes are served, behind a token nobody holds
/// ([`admin::AdminToken::unguessable`]): same check, opens to no one.
pub fn router_with(state: Arc<rooms::AppState>) -> Router {
    router_with_admin(state, admin::AdminToken::unguessable())
}

/// [`router_with`], with the admin channel opened by `token` (T-25).
pub fn router_with_admin(state: Arc<rooms::AppState>, token: admin::AdminToken) -> Router {
    routes::routes(state.clone()).merge(routes::admin_routes(state, token))
}
