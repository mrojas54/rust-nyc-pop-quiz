//! *Create a room* and *Run it again*: the one path that makes a [`Room`].
//!
//! [`AppState::create_room_checked_for`] is the routes' door: the one create
//! check (G-9), then [`AppState::create_for`], the one room maker. *Run it
//! again* ends at the same maker, so every refusal is the same refusal.

use std::time::SystemTime;

use serde::Serialize;

use crate::auth::OrganizerId;
use crate::club::ClubSlug;
use crate::phase::{Command, HostAction, Phase};

use super::state::{lock, Entry};
use super::{Acted, AppState, Room, RoomError, CODE_ALPHABET};

/// What a new room's creator is handed, once.
#[derive(Clone, Debug, Serialize)]
pub struct Created {
    pub id: String,
    pub code: String,
    pub join_url: String,
    pub host_session: String,
    pub host_resume_url: String,
}

pub(super) fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).expect("the OS random source is available");
    buf
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Six characters from [`CODE_ALPHABET`].
pub fn new_code() -> String {
    random_bytes::<6>()
        .iter()
        .map(|b| CODE_ALPHABET[usize::from(b & 31)] as char)
        .collect()
}

impl AppState {
    /// *Create a room*: authorized by the one create check (G-9, §8). The
    /// routes' door: the check may ask Discord, so it is awaited.
    pub async fn create_room_checked(
        &self,
        bearer: Option<&str>,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        self.create_room_checked_for(bearer, &ClubSlug::default_club(), question_id, now)
            .await
    }

    /// [`AppState::create_room_checked`] for `club`: the check tests *that*
    /// club's role (AC-103), and the room is that club's.
    pub async fn create_room_checked_for(
        &self,
        bearer: Option<&str>,
        club: &ClubSlug,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        let organizer = self.auth.authorize_create(bearer, club).await?;
        self.create_for(organizer, club, question_id, now)
    }

    /// [`AppState::create_room_checked`] for a backend that decides without
    /// I/O (the tests' `TestAuth`, [`crate::auth::DenyAll`]). A check that
    /// would have to wait — Discord's — is [`RoomError::Unavailable`] here.
    pub fn create_room(
        &self,
        bearer: Option<&str>,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        use futures_util::FutureExt;
        self.create_room_checked(bearer, question_id, now)
            .now_or_never()
            .unwrap_or(Err(RoomError::Unavailable))
    }

    /// The one room maker, for *Create a room* and *Run it again*. One room per
    /// question (GAP-8, the client's ruling of 2026-10-05): refused while
    /// another room holds the question — not released, and not ended at `now`
    /// ([`Room::holds`]) — because releasing either of two rooms on one
    /// question would put its answer on `/last` while the other is still
    /// before reveal (G-3). Refused, too, if the question has been run (G-10)
    /// or is not scheduled.
    ///
    /// Every check and the insert are one critical section: `questions`, then
    /// `rooms` (the order [`AppState::schedule`] takes), held until the room
    /// is in. The used check sits under `rooms` because the release that
    /// writes `used` happens under `rooms` ([`AppState::act`]), so a release
    /// cannot land between the check and the insert; `questions` is held so
    /// `schedule` cannot replace the record the room is built from.
    fn create_for(
        &self,
        organizer: OrganizerId,
        club: &ClubSlug,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        let questions = lock(&self.questions);
        let mut rooms = lock(&self.rooms);
        if rooms.values().any(|e| e.room.holds(club, question_id, now)) {
            return Err(RoomError::Refused(
                "That question is open in another room. Pick another.".into(),
            ));
        }
        // The rooms on this question that ended without release stay ended:
        // the new room is let in on the strength of it.
        for entry in rooms
            .values_mut()
            .filter(|e| &e.room.club == club && e.room.question_id == question_id)
        {
            if entry.room.phase() != Phase::Released {
                entry.room.latched_end = entry.room.ended(now);
            }
        }
        if self.used.contains(club, question_id) {
            return Err(RoomError::Refused(
                "That question has already been run. Pick another.".into(),
            ));
        }
        // No fallback to another club's record: it was arranged for another date.
        let question = questions
            .get(&(club.clone(), question_id.to_string()))
            .cloned()
            .ok_or_else(|| RoomError::Refused("No question is scheduled with that id.".into()))?;
        let ended = lock(&self.ended);
        let code = loop {
            let code = new_code();
            if !rooms.values().any(|e| e.room.code == code)
                && !ended.values().any(|t| t.code == code)
            {
                break code;
            }
        };
        drop(ended);
        let room = Room::create(
            question,
            organizer,
            club.clone(),
            self.clubs.zone_of(club),
            code,
            &self.urls,
            now,
        )?;
        let created = Created {
            id: room.id.clone(),
            code: room.code.clone(),
            join_url: room.join_url.clone(),
            host_session: room.host_session.clone(),
            host_resume_url: room.host_resume_url.clone(),
        };
        rooms.insert(
            room.id.clone(),
            Entry {
                room,
                sessions: (self.sessions)(),
            },
        );
        drop(rooms);
        drop(questions);
        Ok(created)
    }

    /// *Run it again*: a **new** room, for the organizer who created this one,
    /// on a question that has not been run and no other room holds. This room
    /// stays `released`, and a released room holds nothing ([`Room::holds`]);
    /// the refusals are `AppState::create_for`'s. The routes' door, like
    /// [`AppState::create_room_checked`].
    pub async fn run_again_checked(
        &self,
        room_id: &str,
        bearer: Option<&str>,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        // The new room is the old room's club's, whichever the caller names
        // (D-26): *Run it again* never moves a room to another club.
        let club = {
            let rooms = lock(&self.rooms);
            rooms
                .get(room_id)
                .ok_or(RoomError::NotFound)?
                .room
                .club
                .clone()
        };
        let organizer = self.auth.authorize_create(bearer, &club).await?;
        self.run_again_for(organizer, room_id, question_id, now)
    }

    /// [`AppState::run_again_checked`] for a backend that decides without I/O,
    /// as [`AppState::create_room`] is.
    pub fn run_again(
        &self,
        room_id: &str,
        bearer: Option<&str>,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        use futures_util::FutureExt;
        self.run_again_checked(room_id, bearer, question_id, now)
            .now_or_never()
            .unwrap_or(Err(RoomError::Unavailable))
    }

    fn run_again_for(
        &self,
        organizer: OrganizerId,
        room_id: &str,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        let club = {
            let mut rooms = lock(&self.rooms);
            let entry = rooms.get_mut(room_id).ok_or(RoomError::NotFound)?;
            if entry.room.organizer != organizer {
                return Err(RoomError::Denied);
            }
            if let Some(why) = entry.room.ended(now) {
                return Err(RoomError::Ended(why));
            }
            match entry
                .room
                .act(Command::Host(HostAction::RunItAgain), now, || {
                    unreachable!("Run it again never closes answers")
                })? {
                Acted::NewRoomRequested => {}
                Acted::Changed => unreachable!("Run it again never moves this room"),
            }
            entry.room.club.clone()
        };
        self.create_for(organizer, &club, question_id, now)
    }
}
