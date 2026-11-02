//! Join by code. The session and its live counts share one `rooms` lock (`mod.rs`).

use crate::phase::Phase;
use crate::quiz_session::state_manager::{lock, AppState};
use crate::quiz_session::Joined;

impl AppState {
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
}
