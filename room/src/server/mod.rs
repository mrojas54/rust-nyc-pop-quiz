//! The server: what the `room` binary (`main.rs`) builds and serves.
//!
//! - [`routes`] — the HTTP surface: pages, assets and the room's routes.
//! - [`admin`] — SPEC §8.3's pipeline channel: `PUT /admin/questions/{id}`
//!   and `GET /admin/used` behind one constant-time bearer check (T-25).
//! - [`config`] — what the binary reads from its environment (T-09).
//! - `reaper` — the task that sweeps ended rooms and closes their sockets
//!   (T-11); what ends a room is `rooms::lifecycle`.

pub mod admin;
pub mod config;
pub(crate) mod reaper;
pub(crate) mod routes;
