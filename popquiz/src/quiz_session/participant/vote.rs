//! The answer upsert. The vote and its live counts share one `rooms` lock (`mod.rs`).

use crate::quiz_session::state_manager::{lock, AppState};
use crate::quiz_session::AnswerError;

impl AppState {
    /// The answer upsert (§4.3): only while `live`, last write wins, and the
    /// saved letter comes back. Anything else leaves the saved answer as it
    /// was (AC-36).
    pub fn answer(
        &self,
        room_id: &str,
        token: &str,
        letter: crate::question::Letter,
    ) -> Result<crate::question::Letter, AnswerError> {
        let now = self.now();
        let mut rooms = lock(&self.rooms);
        let entry = rooms.get_mut(room_id).ok_or(AnswerError::NotFound)?;
        let saved = entry
            .sessions
            .answer_of(token)
            .ok_or(AnswerError::UnknownSession)?;
        if let Some(why) = entry.room.ended(now) {
            return Err(AnswerError::Ended(why));
        }
        if !entry.room.accepts_answers() {
            return Err(AnswerError::Refused {
                phase: entry.room.phase(),
                saved,
            });
        }
        let saved = entry
            .sessions
            .upsert(token, letter)
            .ok_or(AnswerError::UnknownSession)?;
        entry.room.set_live_counts(entry.sessions.counts());
        Ok(saved)
    }
}
