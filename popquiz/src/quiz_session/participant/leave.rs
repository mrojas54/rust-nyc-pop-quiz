//! Leaving for good. The session and its live counts share one `rooms` lock (`mod.rs`).

use crate::quiz_session::state_manager::{lock, AppState};
use crate::quiz_session::RoomError;

impl AppState {
    /// Drop one session whole. T-04c calls this when a socket is gone for
    /// good; a mere drop keeps the session, its answer and its slot (AC-37).
    /// `Ok(false)` if the token named no session.
    pub fn leave(&self, room_id: &str, token: &str) -> Result<bool, RoomError> {
        let mut rooms = lock(&self.rooms);
        let entry = rooms.get_mut(room_id).ok_or(RoomError::NotFound)?;
        let left = entry.sessions.leave(token);
        entry.room.set_live_counts(entry.sessions.counts());
        Ok(left)
    }
}
