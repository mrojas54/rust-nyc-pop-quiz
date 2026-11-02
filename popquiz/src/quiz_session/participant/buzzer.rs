//! One session's buzzer payload.

use crate::quiz_session::state_manager::{lock, AppState};
use crate::quiz_session::RoomError;

impl AppState {
    /// The buzzer payload for one session, carrying its own saved answer and
    /// nothing about any other. T-04c pushes this on re-attach. A token that
    /// names no session is `Denied`.
    pub fn buzzer_for(
        &self,
        room_id: &str,
        token: &str,
    ) -> Result<crate::view::BuzzerPayload, RoomError> {
        let now = self.now();
        let rooms = lock(&self.rooms);
        let entry = rooms.get(room_id).ok_or(RoomError::NotFound)?;
        let yours = entry.sessions.answer_of(token).ok_or(RoomError::Denied)?;
        if let Some(why) = entry.room.ended(now) {
            return Err(RoomError::Ended(why));
        }
        // Invariant: pre-reveal isolation. The payload carries this session's
        // own letter and nothing about any other; `view::buzzer_for` builds it
        // from the public view until the room is in `reveal` (AC-57, AC-61).
        Ok(crate::view::buzzer_for(&entry.room, yours))
    }
}
