//! The server: what the `room` binary (`main.rs`) builds and serves.
//!
//! - [`routes`] — the HTTP surface: pages, assets and the room's routes.
//! - [`admin`] — SPEC §8.3's pipeline channel: `PUT /admin/questions/{id}`
//!   and `GET /admin/used` behind one constant-time bearer check (T-25).
//! - [`config`] — what the binary reads from its environment (T-09).
//! - [`lifecycle`] — how long a room lives and what ends it: the clock seam,
//!   §4.6's bounds, the sweep's reaper (T-11).

pub mod admin;
pub mod config;
pub mod lifecycle;
pub(crate) mod routes;
