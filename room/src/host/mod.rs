//! The organizer's side: signing in with Discord and the check that lets
//! them create a room.
//!
//! - [`auth`] — the `HostAuth` seam (G-9): the one create check.
//! - [`discord`] — its implementation: Discord sign-in, the organizer records
//!   (§3.5) and the role-ID check at creation (§8, T-10).
//!   - `discord/mod.rs` — OAuth sign-in, organizer records and sessions,
//!     refresh coordination, the authorization check and its retries.
//!   - `discord/config.rs` — settings and secrets, where Discord is, the retry
//!     budget, the log sink. Its types are re-exported, so
//!     `room::discord::Settings` is the path.
//!   - `discord/http.rs` — encoding and the HTTP/1.1 client over TCP or TLS.
//!     It sends what it is handed and returns status and body; it neither
//!     retries nor decides.
//!
//! ## Two sessions
//!
//! An **organizer session** is minted by [`discord::Discord::finish`] after
//! Discord sign-in. It names an organizer, lives twelve hours, and is the
//! bearer for one thing: [`auth::HostAuth::authorize_create`], which lets that
//! organizer create a room (or run one again) for a club whose role they hold.
//! It never controls a room.
//!
//! A **room-host session** is minted with the room and handed back by *Create a
//! room*. It names that room alone, never rotates (AC-50), and is the bearer
//! for every host command and the host's view of that room
//! ([`auth::HostAuth::authorize_host`]). Using it never asks Discord, so an
//! open room runs to release with Discord down (AC-69). Neither session is the
//! other (AC-68).

pub mod auth;
pub mod discord;
