//! The organizer's side: signing in with Discord and the check that lets
//! them create a room.
//!
//! - [`auth`] — the `HostAuth` seam (G-9): the one create check.
//! - [`discord`] — its implementation: Discord sign-in, the organizer records
//!   (§3.5) and the role-ID check at creation (§8, T-10).

pub mod auth;
pub mod discord;
