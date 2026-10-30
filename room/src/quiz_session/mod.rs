//! The room record (SPEC.md §3.4) and the in-memory set of rooms.
//!
//! T-04a owns the record's **shape** and the writes the phase machine drives:
//! `closed` freezes `answered` and `totals` from a snapshot the sessions module
//! hands over, and decides §4.5's middle beat once; `released` sets
//! `released_at`. Everything about sessions — joins, capacity, the answer
//! upsert, refusal after close — is T-04b's, and the socket is T-04c's. They
//! plug in here:
//!
//! - [`Sessions`] — T-04b implements it over its per-room session map. The
//!   close transition calls [`Sessions::close_snapshot`] exactly once; release
//!   calls [`Sessions::release`].
//! - [`AppState::set_live_counts`] — the counts' writer from outside the lock.
//!   T-04b's own join, leave and upsert write the same fields through
//!   [`Room::set_live_counts`] while they already hold it.
//! - [`Room::accepts_answers`] — T-04b's upsert asks it; it is `true` only in
//!   `live`.
//! - [`Room::revision`] — bumped on every change; T-04c broadcasts on it.
//! - [`crate::view`] — the three projections T-04c pushes down the socket.
//!
//! ## Where things are
//!
//! | File | Holds |
//! |---|---|
//! | `room.rs` | One room's record and the transitions the phase machine drives |
//! | `open.rs` | The one way to the answers: [`Room::open`], [`Room::public`] |
//! | `state.rs` | [`AppState`]'s fields and setup, scheduling, host commands and release, the read paths, expiry |
//! | `participants.rs` | Join, answer, leave, one session's buzzer payload, the session and live counts, the socket's token map |
//! | `create.rs` | *Create a room* and *Run it again* |
//! | `seams.rs` · `errors.rs` · `lifecycle.rs` | What sessions plug in; the error types; the clock and §4.6's bounds |
//!
//! Every file above is an `impl` of the one [`AppState`] or a type it holds:
//! one state, one `rooms` lock, split by who calls it.
//!
//! | Operation | Main path |
//! |---|---|
//! | Sign in | `server/routes/host.rs` → [`crate::discord::Discord::begin`] / [`finish`](crate::discord::Discord::finish) |
//! | Create a room | host route → [`HostAuth::authorize_create`](crate::auth::HostAuth::authorize_create) (the organizer session, Discord) → [`AppState::create_room_checked`] (`create.rs`) |
//! | Host command | host route → [`AppState::act`]: the room's host session, then [`Room::act`] and release's side effects (`state.rs`) |
//! | Join or answer | participant route → [`AppState::join`] / [`AppState::answer`] → sessions and live counts under the one `rooms` lock (`participants.rs`) |
//! | Expire a room | the reaper (`server/reaper.rs`) → [`AppState::sweep`] → removal and a tombstone (`state.rs`) |
//!
//! **The one way to the answers** is [`Room::open`], which asks the machine for
//! a [`crate::phase::RevealWitness`] and returns `None` in every phase but
//! `reveal`. There is no getter for the question's vault. [`Room::public`]
//! returns a [`PublicView`], which has no path to the vault at all.
//!
//! ```compile_fail
//! // AC-61: a room's question (and its vault) is not reachable by field.
//! fn peek(r: &room::rooms::Room) -> &room::answers::Scheduled { &r.question }
//! ```
//!
//! ```
//! // Twin: the same types, the one way there is.
//! fn read(r: &room::rooms::Room) -> Option<room::question::Letter> { r.open().map(|o| o.revealed.correct) }
//! ```
//!
//! ```compile_fail
//! // AC-61: the public view has no way to the vault.
//! fn peek(v: room::rooms::PublicView<'_>) -> Option<room::question::Letter> { v.open().map(|o| o.revealed.correct) }
//! ```
//!
//! ```
//! // Twin.
//! fn read(v: room::rooms::PublicView<'_>) -> room::phase::Phase { v.phase }
//! ```
//!
//! ```compile_fail
//! // G-3: what was opened in `reveal` cannot be held across a phase change.
//! use room::phase::{Command, HostAction};
//! use room::rooms::{CloseSnapshot, Room};
//! fn keep(r: &mut Room) {
//!     let opened = r.open();
//!     let _ = r.act(Command::Host(HostAction::ReleaseRoom), std::time::SystemTime::now(), || CloseSnapshot::from_totals([0; 5]));
//!     drop(opened);
//! }
//! ```
//!
//! ```
//! // Twin: the same calls, with the opened view dropped before the room moves.
//! use room::phase::{Command, HostAction};
//! use room::rooms::{CloseSnapshot, Room};
//! fn keep(r: &mut Room) {
//!     let opened = r.open();
//!     drop(opened);
//!     let _ = r.act(Command::Host(HostAction::ReleaseRoom), std::time::SystemTime::now(), || CloseSnapshot::from_totals([0; 5]));
//! }
//! ```

use std::time::Duration;

mod errors;
pub mod host;
pub mod lifecycle;
pub mod models;
mod open;
mod participants;
mod seams;
mod state_manager;

pub use errors::{AnswerError, Joined, RoomError, Scheduling};
pub use host::create::{new_code, Created};
pub use models::rooms::{Acted, Fit, Room, Totals, Urls};
pub use open::{Opened, PublicView};
pub use seams::{CloseSnapshot, LiveCounts, NoSessions, Sessions};
pub use state_manager::AppState;

/// A room lives at most four hours from creation (AC-69).
pub const ROOM_LIFETIME: Duration = Duration::from_secs(4 * 60 * 60);

/// The code alphabet: letters and digits without `O 0 I 1` (SPEC §3.4). 32
/// symbols, so a random byte masked to five bits picks one without bias.
pub const CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).expect("the OS random source is available");
    buf
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
