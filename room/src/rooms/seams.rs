//! The seam T-04b implements over its per-room sessions: the counts it keeps,
//! the close snapshot, and the trait the room calls.

use super::Totals;

/// Counts the sessions module keeps current (T-04b).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiveCounts {
    /// Live participant sessions (§3.4 `present`).
    pub present: u32,
    /// Sessions currently holding an answer (§3.4 `answered_live`).
    pub answered_live: u32,
}

/// What the sessions module hands the close transition: the per-option
/// totals, taken once. `answered` is their sum, so the two cannot disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CloseSnapshot {
    totals: Totals,
}

impl CloseSnapshot {
    pub fn from_totals(totals: Totals) -> CloseSnapshot {
        CloseSnapshot { totals }
    }

    pub fn totals(&self) -> Totals {
        self.totals
    }

    pub fn answered(&self) -> u32 {
        self.totals.iter().sum()
    }
}

/// The seam T-04b implements over its per-room sessions.
pub trait Sessions: Send {
    /// Called by the close transition and nothing else, once per room.
    fn close_snapshot(&self) -> CloseSnapshot;
    /// Called at release: drop every session (§3.4, D-12).
    fn release(&mut self) {}

    // T-04b: the participant half, implemented by `sessions::SessionMap`. The
    // defaults describe a room nobody can join, which is what `NoSessions`
    // and the test doubles are.

    /// A new session, if fewer than `capacity` exist. Nothing is created or
    /// reserved when the answer is `None` (AC-30).
    fn join(&mut self, _capacity: usize) -> Option<crate::sessions::Token> {
        None
    }
    /// Last write wins (§4.3). `None` if the token names no session. The
    /// caller has already asked [`Room::accepts_answers`].
    fn upsert(&mut self, _token: &str, _letter: crate::question::Letter) -> Option<crate::question::Letter> {
        None
    }
    /// The session's saved answer; `None` if the token names no session.
    fn answer_of(&self, _token: &str) -> Option<Option<crate::question::Letter>> {
        None
    }
    /// Drop one session whole — T-04c's call when a socket is gone for good.
    fn leave(&mut self, _token: &str) -> bool {
        false
    }
    /// `present` and `answered_live`, from the sessions as they stand.
    fn counts(&self) -> LiveCounts {
        LiveCounts::default()
    }
    /// How many sessions exist (capacity and AC-30's "nothing reserved").
    fn session_count(&self) -> usize {
        0
    }

    // PQ-32: what the transport asks, through `impl ws::SessionTokens for
    // AppState`.

    /// The session's stable handle, if `token` names a session.
    fn session_id(&self, _token: &str) -> Option<crate::ws::SessionId> {
        None
    }
    /// That session's own saved answer; `None` if the handle names no session.
    fn saved_by_id(&self, _id: crate::ws::SessionId) -> Option<Option<crate::question::Letter>> {
        None
    }
}

/// T-04a's stand-in until T-04b lands: nobody has answered.
pub struct NoSessions;

impl Sessions for NoSessions {
    fn close_snapshot(&self) -> CloseSnapshot {
        CloseSnapshot::from_totals([0; 5])
    }
}
