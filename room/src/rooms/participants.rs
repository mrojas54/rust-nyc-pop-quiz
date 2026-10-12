//! Participant operations on [`AppState`]: join, answer, leave, the buzzer
//! payload for one session, the session count, the live counts, and the
//! socket's token map ([`crate::ws::SessionTokens`]). The state itself, host
//! commands, release and the sweep are `state.rs`'s.

use crate::phase::Phase;

use super::state::{lock, AppState};
use super::{AnswerError, Joined, LiveCounts, RoomError};

impl AppState {
    /// T-04b's writer for the live counts.
    pub fn set_live_counts(&self, room_id: &str, counts: LiveCounts) -> Result<(), RoomError> {
        let mut rooms = lock(&self.rooms);
        let entry = rooms.get_mut(room_id).ok_or(RoomError::NotFound)?;
        entry.room.set_live_counts(counts);
        Ok(())
    }

    // ----------------------------------------------------------------------
    // T-04b: joins, answers and leaves. Each runs under the one `rooms` lock
    // and writes `present` and `answered_live` through `Room::set_live_counts`
    // before letting go, so the counts and the sessions are never seen apart.
    // ----------------------------------------------------------------------

    /// Join by code (§4.1). Any phase from `idle` to `reveal` admits a join;
    /// `released` has ended. Capacity is checked before anything is created.
    pub fn join(&self, code: &str) -> Result<Joined, crate::sessions::JoinRefusal> {
        use crate::sessions::{parse_code, JoinRefusal};
        let code = parse_code(code).ok_or(JoinRefusal::Malformed)?;
        let now = self.now();
        let mut rooms = lock(&self.rooms);
        let Some(entry) = rooms.values_mut().find(|e| e.room.code == code) else {
            // T-11: a room that expired or went quiet says so (AC-29).
            return Err(lock(&self.ended)
                .values()
                .find(|t| t.code == code)
                .map_or(JoinRefusal::Unknown, |t| t.ended.refusal()));
        };
        if let Some(why) = entry.room.ended(now) {
            return Err(why.refusal());
        }
        if entry.room.phase() == Phase::Released {
            return Err(JoinRefusal::AlreadyEnded);
        }
        let token = entry
            .sessions
            .join(self.capacity)
            .ok_or(JoinRefusal::Full)?;
        entry.room.set_live_counts(entry.sessions.counts());
        Ok(Joined {
            room_id: entry.room.id.clone(),
            token,
            buzzer: crate::view::buzzer_for(&entry.room, None),
        })
    }

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
        Ok(crate::view::buzzer_for(&entry.room, yours))
    }

    /// How many sessions a room holds (AC-30's "nothing reserved", AC-56).
    pub fn session_count(&self, room_id: &str) -> Result<usize, RoomError> {
        let rooms = lock(&self.rooms);
        let entry = rooms.get(room_id).ok_or(RoomError::NotFound)?;
        Ok(entry.sessions.session_count())
    }
}

// --------------------------------------------------------------------------
// PQ-32: the session map, as the transport sees it. Sessions live per room
// under the rooms lock, so the adapter is `AppState` itself and the map
// answers through `Sessions::session_id` / `Sessions::saved_by_id`.
// --------------------------------------------------------------------------

impl crate::ws::SessionTokens for AppState {
    /// A released room's map is empty, so its tokens resolve to nothing.
    fn resolve(&self, room_id: &str, token: &str) -> Option<crate::ws::SessionId> {
        let rooms = lock(&self.rooms);
        rooms.get(room_id)?.sessions.session_id(token)
    }

    /// Only that session's own answer (AC-57).
    fn saved(
        &self,
        room_id: &str,
        session: crate::ws::SessionId,
    ) -> Option<crate::question::Letter> {
        let rooms = lock(&self.rooms);
        rooms.get(room_id)?.sessions.saved_by_id(session).flatten()
    }

    /// Nothing. A dropped socket leaves its session, its answer and its slot
    /// exactly as they were: `present` counts sessions that exist, and only
    /// [`AppState::leave`], once a socket is gone for good (T-11's grace),
    /// removes one (AC-37). With nothing done here, a `resolve` that races a
    /// `gone` has nothing to win against.
    fn gone(&self, _room_id: &str, _session: crate::ws::SessionId) {}
}
