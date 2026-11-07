//! The questions a club can run today, for the host phone's picker (SPEC
//! §8.5, PQ-80), and the one check that decides it.
//!
//! [`refusal`] is *Create a room*'s check, read-only: [`AppState::create_for`]
//! calls it under its lock and stays the authority, and
//! [`AppState::shortlist`] calls it the same way, under the same locks in the
//! same order (`questions`, then `rooms`), to offer exactly what a create would
//! accept at that moment. Listing uses nothing (G-10) and arranges nothing; it
//! hands out each record's `id` and `topic` and nothing else of it.

use std::collections::HashMap;
use std::time::SystemTime;

use serde::Serialize;

use crate::club::ClubSlug;
use crate::copy;
use crate::used::{meetup_date_in, UsedLedger};

use crate::quiz_session::state_manager::{lock, Entry, ScheduledEntry};
use crate::quiz_session::{AppState, RoomError};

/// One question on the picker: its id and its topic, nothing else (§8.5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Offered {
    pub id: String,
    pub topic: String,
}

/// Why `club` cannot make a room on `question_id` at `now`, or `None` when it
/// can: a live room holds it (GAP-8), the club has run it (G-10), it is not
/// scheduled for the club, it was arranged for another meetup date than
/// `today` (R1-a; an undated record is not gated), or its record may not run
/// (G-12). In that order, which is the order *Create a room* has always
/// refused in.
pub(in crate::quiz_session) fn refusal(
    questions: &HashMap<(ClubSlug, String), ScheduledEntry>,
    rooms: &HashMap<String, Entry>,
    used: &UsedLedger,
    club: &ClubSlug,
    question_id: &str,
    today: &str,
    now: SystemTime,
) -> Option<String> {
    if rooms.values().any(|e| e.room.holds(club, question_id, now)) {
        return Some(copy::ROOM_QUESTION_OPEN_ELSEWHERE.into());
    }
    if used.contains(club, question_id) {
        return Some(copy::ROOM_QUESTION_ALREADY_RUN.into());
    }
    // No fallback to another club's record: it was arranged for another date.
    let Some(entry) = questions.get(&(club.clone(), question_id.to_string())) else {
        return Some(copy::ROOM_NOTHING_SCHEDULED.into());
    };
    if entry.meetup_date.as_deref().is_some_and(|d| d != today) {
        return Some(copy::ROOM_QUESTION_OTHER_DATE.into());
    }
    entry.question.refusal(question_id)
}

impl AppState {
    /// `GET /organizer/questions`: authorized by the one create check (G-9,
    /// AC-103), exactly as *Create a room* is, then [`AppState::shortlist`].
    pub async fn shortlist_checked(
        &self,
        bearer: Option<&str>,
        club: &ClubSlug,
        now: SystemTime,
    ) -> Result<Vec<Offered>, RoomError> {
        self.auth.authorize_create(bearer, club).await?;
        Ok(self.shortlist(club, now))
    }

    /// The questions scheduled for `club` that a create would accept at
    /// `now`, sorted by id.
    pub fn shortlist(&self, club: &ClubSlug, now: SystemTime) -> Vec<Offered> {
        let today = meetup_date_in(now, self.clubs.zone_of(club));
        let questions = lock(&self.questions);
        let rooms = lock(&self.rooms);
        let mut offered: Vec<Offered> = questions
            .iter()
            .filter(|((c, id), _)| {
                c == club
                    && refusal(&questions, &rooms, &self.used, club, id, &today, now).is_none()
            })
            .map(|((_, id), entry)| Offered {
                id: id.clone(),
                topic: entry.question.public().topic().to_string(),
            })
            .collect();
        drop(rooms);
        drop(questions);
        offered.sort_by(|a, b| a.id.cmp(&b.id));
        offered
    }
}
