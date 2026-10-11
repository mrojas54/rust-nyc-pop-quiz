//! What a room request can come back with.

use super::*;

/// Why a room request did not happen.
#[derive(Debug, PartialEq, Eq)]
pub enum RoomError {
    /// No room with that id.
    NotFound,
    /// The credential was missing or wrong. Says nothing else (AC-70).
    Denied,
    /// The machine or the schedule refused it, in plain words.
    Refused(String),
    /// T-11: the room has ended — four hours up, or quiet too long.
    Ended(Ended),
    /// T-10: Discord says the organizer is not in the configured guild. Also
    /// what a non-member hears, so membership never leaks (AC-70).
    WrongServer,
    /// T-10: a member of the guild without the configured role ID (AC-65).
    WrongRole,
    /// T-10: Discord gave no verdict within the retry budget. A server error,
    /// never a denial.
    Unavailable,
}

impl From<Denied> for RoomError {
    fn from(_: Denied) -> RoomError {
        RoomError::Denied
    }
}

// T-10: the create check's refusals, as room errors.
impl From<CreateRefusal> for RoomError {
    fn from(r: CreateRefusal) -> RoomError {
        match r {
            CreateRefusal::Denied => RoomError::Denied,
            CreateRefusal::WrongServer => RoomError::WrongServer,
            CreateRefusal::WrongRole => RoomError::WrongRole,
            CreateRefusal::Unavailable => RoomError::Unavailable,
        }
    }
}

impl From<Refused> for RoomError {
    fn from(r: Refused) -> RoomError {
        RoomError::Refused(r.reason)
    }
}

/// What [`AppState::schedule`] did (T-25).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scheduling {
    /// The id was not scheduled before.
    New,
    /// It was, and no room held it; the new record replaced it.
    Replaced,
}


// T-04b: what the participant routes hand back.

/// A successful join: the token, once, and the caller's buzzer view.
pub struct Joined {
    pub room_id: String,
    pub token: crate::sessions::Token,
    pub buzzer: crate::view::BuzzerPayload,
}

/// Why an answer write did not happen. The previous answer is intact in every
/// case (AC-36).
#[derive(Debug, PartialEq, Eq)]
pub enum AnswerError {
    /// No room with that id.
    NotFound,
    /// The token names no session in this room — never joined, left, or the
    /// room was released.
    UnknownSession,
    /// Not `live`: refused, with the saved answer restated (§4.3).
    Refused { phase: Phase, saved: Option<crate::question::Letter> },
    /// T-11: the room has ended.
    Ended(Ended),
}
