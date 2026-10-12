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

mod create;
mod errors;
pub mod lifecycle;
mod open;
mod participants;
mod room;
mod seams;
mod state;

pub use create::{new_code, Created};
pub use errors::{AnswerError, Joined, RoomError, Scheduling};
pub use open::{Opened, PublicView};
pub use room::{Acted, Fit, Room, Urls};
pub use seams::{CloseSnapshot, LiveCounts, NoSessions, Sessions};
pub use state::AppState;

/// Per-option counts, A–E in arrival order. The only answer data a room holds
/// (AC-56, D-12).
pub type Totals = [u32; 5];

/// A room lives at most four hours from creation (AC-69).
pub const ROOM_LIFETIME: Duration = Duration::from_secs(4 * 60 * 60);

/// The code alphabet: letters and digits without `O 0 I 1` (SPEC §3.4). 32
/// symbols, so a random byte masked to five bits picks one without bias.
pub const CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
