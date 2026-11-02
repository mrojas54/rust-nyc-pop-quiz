//! The live counts' writer and the session count.

use crate::quiz_session::state_manager::{lock, AppState};
use crate::quiz_session::{LiveCounts, RoomError};

impl AppState {
    /// T-04b's writer for the live counts.
    pub fn set_live_counts(&self, room_id: &str, counts: LiveCounts) -> Result<(), RoomError> {
        let mut rooms = lock(&self.rooms);
        let entry = rooms.get_mut(room_id).ok_or(RoomError::NotFound)?;
        entry.room.set_live_counts(counts);
        Ok(())
    }

    /// How many sessions a room holds (AC-30's "nothing reserved", AC-56).
    pub fn session_count(&self, room_id: &str) -> Result<usize, RoomError> {
        let rooms = lock(&self.rooms);
        let entry = rooms.get(room_id).ok_or(RoomError::NotFound)?;
        Ok(entry.sessions.session_count())
    }
}
