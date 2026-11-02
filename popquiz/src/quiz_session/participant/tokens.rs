//! The session map, as the transport sees it.

use crate::quiz_session::state_manager::{lock, AppState};

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
