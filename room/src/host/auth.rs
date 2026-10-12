//! The auth seam (SPEC.md G-9, §8).
//!
//! One trait answers the only two questions the room asks: *may this bearer
//! create a room?* and *may this bearer act as host of this room?* The first
//! is the credential path: Discord's role-ID check (T-10, [`crate::discord`])
//! implements it, and nothing else in the crate does. The second, by default,
//! is the room's own host session, which never rotates (AC-50) and is compared
//! in constant time. Discord is asked at creation and never again (AC-69).
//!
//! The create check is asynchronous because Discord is a network call. A
//! backend that decides without one (the tests' `TestAuth`, [`DenyAll`])
//! answers with [`decided`].
//!
//! [`DenyAll`] is what [`crate::router`] runs with, so it can create no room.
//! The binary serves the Discord backend ([`crate::serving_state`]). The test
//! implementation lives in `tests/common`. SPEC §8.2's M1 stand-in was a third
//! backend here until T-10 deleted it.

use std::future::Future;
use std::pin::Pin;

use crate::club::ClubSlug;
use crate::rooms::Room;

/// Who created a room. Only that organizer controls it (AC-68). For the
/// Discord backend, the organizer's Discord user id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrganizerId(pub String);

/// A host-session refusal. It carries nothing, so it can say nothing (AC-70).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Denied;

/// Why *Create a room* was refused. Only the two denials Discord answered
/// with name a condition, and neither says whether the person is in the guild
/// (AC-70): a non-member and a member of another guild both get
/// [`CreateRefusal::WrongServer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateRefusal {
    /// No organizer session, an unknown or expired one, or one whose Discord
    /// grant was withdrawn. `401` with no body; the host page offers sign-in.
    Denied,
    /// Discord says this account is not a member of the configured guild.
    WrongServer,
    /// A member of the guild whose `roles` lacks the role ID of the club named
    /// (or the club is not one this server runs: the same answer, AC-103).
    WrongRole,
    /// Discord did not give a verdict within the retry budget. The plain server
    /// error — an outage is never reported as a denial.
    Unavailable,
}

/// The create check's answer, when it comes.
pub type CreateCheck<'a> = Pin<Box<dyn Future<Output = Result<OrganizerId, CreateRefusal>> + Send + 'a>>;

/// An answer that needs no I/O, as a [`CreateCheck`].
pub fn decided(answer: Result<OrganizerId, CreateRefusal>) -> CreateCheck<'static> {
    Box::pin(std::future::ready(answer))
}

pub trait HostAuth: Send + Sync + 'static {
    /// *Create a room* (and *Run it again*, which creates one). The one create
    /// check (G-9).
    ///
    /// `club` is the club the organizer is creating the room for: hosting
    /// needs *that* club's role (AC-103, D-26).
    fn authorize_create<'a>(&'a self, bearer: Option<&'a str>, club: &'a ClubSlug) -> CreateCheck<'a>;

    /// Every other host command, and the host's view of the room.
    fn authorize_host(&self, room: &Room, bearer: Option<&str>) -> Result<(), Denied> {
        match bearer {
            Some(b) if room.host_matches(b) => Ok(()),
            _ => Err(Denied),
        }
    }
}

/// Authorizes nothing. What [`crate::router`] runs with.
pub struct DenyAll;

impl HostAuth for DenyAll {
    fn authorize_create<'a>(&'a self, _bearer: Option<&'a str>, _club: &'a ClubSlug) -> CreateCheck<'a> {
        decided(Err(CreateRefusal::Denied))
    }
}
