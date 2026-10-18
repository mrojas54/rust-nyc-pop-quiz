//! Every room on this machine, and the seams around them.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

use crate::answers::Scheduled;
use crate::auth::HostAuth;
use crate::club::{ClubSlug, Clubs};
use crate::phase::Command;
use crate::used::{TakeHome, UsedEntry, UsedLedger, UsedRecord};

use super::lifecycle::{Clock, Ended, SystemClock, ENDED_MEMORY};
use super::{Acted, Fit, Room, RoomError, Scheduling, Sessions, Urls};

// --------------------------------------------------------------------------
// The rooms, in memory. No persistence (D-12: nothing outlives the room but
// the `used` record and the take-it-home snapshot, both T-11's, below).
// --------------------------------------------------------------------------

pub(super) struct Entry {
    pub(super) room: Room,
    pub(super) sessions: Box<dyn Sessions>,
}

pub(super) type SessionFactory = Box<dyn Fn() -> Box<dyn Sessions> + Send + Sync>;

/// Every room on this machine, the questions scheduled into it, and the seams.
///
/// Questions arrive over T-25's admin channel ([`AppState::schedule`]). The
/// other way in, handing them to [`AppState::new`], is the tests' fixtures
/// only (T-10 deleted the M1 stand-in's seed).
pub struct AppState {
    pub(super) rooms: Mutex<HashMap<String, Entry>>,
    // D-26: a scheduled record is the question *arranged for one club's
    // meetup date* (the pipeline moves the answer slot by date), so it is kept
    // per club. One club's push never replaces another's.
    pub(super) questions: Mutex<HashMap<(ClubSlug, String), Arc<Scheduled>>>,
    // T-11: what outlives a room — the used ledger, whose only writer is the
    // release transition (G-10), and the take-it-home snapshot, replaced at
    // every release (§13).
    pub(super) used: UsedLedger,
    // D-26: one snapshot per club, so one club's release never shows on
    // another's take-it-home page (AC-104).
    pub(super) take_home: Mutex<HashMap<ClubSlug, TakeHome>>,
    // D-26: the clubs this server runs (zones, for the meetup date).
    pub(super) clubs: Clubs,
    // T-11: rooms that expired or went quiet, by id: their code and why, for
    // `ENDED_MEMORY`, so a late join is told what happened (AC-29).
    pub(super) ended: Mutex<HashMap<String, Tombstone>>,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) auth: Arc<dyn HostAuth>,
    pub(super) sessions: SessionFactory,
    pub(super) urls: Urls,
    // T-04b: sessions per room (§4.1, §9).
    pub(super) capacity: usize,
    // T-10: the Discord backend's sign-in half, for the OAuth routes. The same
    // object is `auth`; it holds the organizer records (§3.5) beside the rooms.
    pub(super) discord: Option<Arc<crate::discord::Discord>>,
}

/// T-11: what is remembered of a room that expired or went quiet. Its id is
/// the map's key; nothing else about it — no question, no person, no count.
pub(super) struct Tombstone {
    pub(super) code: String,
    pub(super) ended: Ended,
    pub(super) at: SystemTime,
}

pub(super) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panic while holding the lock leaves plain data behind; keep serving.
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl AppState {
    pub fn new(auth: Arc<dyn HostAuth>, questions: Vec<Scheduled>, urls: Urls) -> AppState {
        AppState {
            rooms: Mutex::new(HashMap::new()),
            questions: Mutex::new(
                questions
                    .into_iter()
                    .map(|q| {
                        (
                            (ClubSlug::default_club(), q.public().id().to_string()),
                            Arc::new(q),
                        )
                    })
                    .collect(),
            ),
            used: UsedLedger::default(),
            take_home: Mutex::new(HashMap::new()),
            clubs: Clubs::only_default(String::new()),
            ended: Mutex::new(HashMap::new()),
            clock: Arc::new(SystemClock),
            auth,
            // T-04b: the real sessions by default.
            sessions: Box::new(|| Box::new(crate::sessions::SessionMap::new())),
            urls,
            capacity: crate::sessions::DEFAULT_CAPACITY,
            discord: None,
        }
    }

    /// T-10: serve Discord sign-in (`/auth/discord`) through this backend.
    /// The binary passes the same backend as `auth` ([`crate::serving_state`]).
    pub fn with_discord(mut self, discord: Arc<crate::discord::Discord>) -> AppState {
        self.discord = Some(discord);
        self
    }

    /// T-10: the Discord backend, if this state signs organizers in.
    pub fn discord(&self) -> Option<&Arc<crate::discord::Discord>> {
        self.discord.as_ref()
    }

    /// T-04b: sessions per room; 200 unless configured (§4.1, §9).
    pub fn with_capacity(mut self, capacity: usize) -> AppState {
        self.capacity = capacity;
        self
    }

    /// Replace the sessions factory (T-04b; tests).
    pub fn with_sessions(
        mut self,
        factory: impl Fn() -> Box<dyn Sessions> + Send + Sync + 'static,
    ) -> AppState {
        self.sessions = Box::new(factory);
        self
    }

    pub fn urls(&self) -> &Urls {
        &self.urls
    }

    // ----------------------------------------------------------------------
    // T-11: the clock, what outlives a room, and the sweep.
    // ----------------------------------------------------------------------

    /// Replace the clock (tests; `lifecycle::ManualClock`).
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> AppState {
        self.clock = clock;
        self
    }

    /// The time, from the one clock.
    pub fn now(&self) -> SystemTime {
        self.clock.now()
    }

    /// The used-question ledger (G-10). T-25's `GET /admin/used` and T-20's
    /// sync read [`UsedLedger::all`].
    pub fn used(&self) -> &UsedLedger {
        &self.used
    }

    /// The last released question, for T-12's `/last`. `None` until a room
    /// has been released on this machine.
    pub fn take_home(&self) -> Option<TakeHome> {
        self.take_home_of(&ClubSlug::default_club())
    }

    /// `club`'s last released question (D-26): `None` until that club has
    /// released a room on this machine.
    pub fn take_home_of(&self, club: &ClubSlug) -> Option<TakeHome> {
        lock(&self.take_home).get(club).cloned()
    }

    /// The clubs this server runs (D-26). Until set, the default club alone.
    pub fn with_clubs(mut self, clubs: Clubs) -> AppState {
        self.clubs = clubs;
        self
    }

    pub fn clubs(&self) -> &Clubs {
        &self.clubs
    }

    /// How many room records exist, released shells included (AC-56, by
    /// inspection).
    pub fn room_count(&self) -> usize {
        lock(&self.rooms).len()
    }

    /// Why a room that is gone ended, while it is remembered.
    pub fn ended_room(&self, room_id: &str) -> Option<Ended> {
        lock(&self.ended).get(room_id).map(|t| t.ended)
    }

    /// Delete every room that has ended at `now` — the record and its
    /// sessions, whole — and remember its code and why. Forget remembered
    /// rooms older than [`ENDED_MEMORY`]. The one deletion path (§4.6); the
    /// reaper calls it and tells the transport about the ids it returns.
    pub fn sweep(&self, now: SystemTime) -> Vec<String> {
        let mut rooms = lock(&self.rooms);
        let mut ended = lock(&self.ended);
        ended.retain(|_, t| now < t.at + ENDED_MEMORY);
        let gone: Vec<(String, Ended)> = rooms
            .iter()
            .filter_map(|(id, e)| e.room.ended(now).map(|why| (id.clone(), why)))
            .collect();
        for (id, why) in &gone {
            if let Some(entry) = rooms.remove(id) {
                ended.insert(
                    id.clone(),
                    Tombstone {
                        code: entry.room.code.clone(),
                        ended: *why,
                        at: now,
                    },
                );
            }
        }
        gone.into_iter().map(|(id, _)| id).collect()
    }

    /// T-25: schedule a question — `PUT /admin/questions/{id}` (SPEC §8.3).
    ///
    /// Refused if the question is not affirmed for its text (G-12, GAP-24:
    /// unaffirmed, stale or unbound, by the pipeline's rule's twin); the
    /// record already scheduled under its id, if any, stays. Refused if the
    /// question has been run (G-10, never twice) or if a room
    /// record holds it: a room keeps the question it was created with until
    /// release or expiry deletes it (§4.6), and the used ledger is keyed by
    /// id, so replacing it under a room would let the night's `used` record
    /// name content that did not run. Otherwise the question is added, or
    /// replaces the unheld one with its id (the organizer's re-push after an
    /// edit). Locks `questions` then `rooms`; nothing takes them the other way.
    ///
    /// The hold here is wider than `AppState::create_for`'s on purpose: any
    /// room record, released shells and ended rooms included, until the sweep
    /// deletes it. A new room needs only that no live room has the question
    /// (one room per question, GAP-8); a replaced record needs that nothing
    /// still names it.
    pub fn schedule(&self, question: Scheduled) -> Result<Scheduling, RoomError> {
        self.schedule_for(&ClubSlug::default_club(), question)
    }

    /// [`AppState::schedule`] for `club` (D-26). The record is the question as
    /// arranged for that club's meetup, and only that club's room can hold it:
    /// another club's push of the same id is a different record.
    pub fn schedule_for(
        &self,
        club: &ClubSlug,
        question: Scheduled,
    ) -> Result<Scheduling, RoomError> {
        let id = question.public().id().to_string();
        if let Some(why) = question.affirmation().refusal(&id) {
            return Err(RoomError::Refused(why));
        }
        // Invariant: `questions`, then `rooms`, the order `create_for` takes
        // them in too; nothing takes them the other way round.
        let mut questions = lock(&self.questions);
        let rooms = lock(&self.rooms);
        // Whether a question has been run is a fact about a club too, so that
        // refusal lives in `create_for`; a question one club has run is still
        // scheduled for the others.
        if rooms
            .values()
            .any(|e| &e.room.club == club && e.room.question_id() == id)
        {
            return Err(RoomError::Refused(format!(
                "A room is running {id}; it can be replaced once that room is gone."
            )));
        }
        drop(rooms);
        Ok(
            match questions.insert((club.clone(), id), Arc::new(question)) {
                None => Scheduling::New,
                Some(_) => Scheduling::Replaced,
            },
        )
    }

    /// A host command on a room, bearer-checked against the room's host
    /// session. *Run it again* goes through [`AppState::run_again`] instead.
    pub fn act(
        &self,
        room_id: &str,
        bearer: Option<&str>,
        command: Command,
        now: SystemTime,
    ) -> Result<(), RoomError> {
        let mut rooms = lock(&self.rooms);
        let entry = rooms.get_mut(room_id).ok_or(RoomError::NotFound)?;
        // Invariant: the host check comes before anything about the room's
        // end is said, so only the room's host learns that it ended.
        self.auth.authorize_host(&entry.room, bearer)?;
        if let Some(why) = entry.room.ended(now) {
            return Err(RoomError::Ended(why));
        }
        let Entry { room, sessions } = entry;
        match room.act(command, now, || sessions.close_snapshot())? {
            Acted::Changed => {}
            Acted::NewRoomRequested => {
                return Err(RoomError::Refused(
                    "Run it again needs the organizer's credential and a question.".into(),
                ))
            }
        }
        // T-11: the release transition's side effects, under the same lock as
        // the transition itself. The used record's one writer (G-10).
        // Invariant: release, clearing the sessions, the used record and the
        // take-it-home snapshot happen together, with `rooms` held throughout.
        if let Some(parting) = room.take_parting() {
            sessions.release();
            self.used.append(UsedEntry {
                club: room.club.clone(),
                question_id: room.question_id.clone(),
                used: UsedRecord {
                    meetup_date: parting.meetup_date,
                    room_id: room.id.clone(),
                    released_at: crate::used::rfc3339(now),
                    fit: parting.fit,
                    // The room's own record: `create_for` let it in bound only.
                    affirmed_by: room
                        .question
                        .affirmation()
                        .affirmed_by()
                        .map(str::to_string),
                },
            });
            lock(&self.take_home).insert(room.club.clone(), parting.take_home);
        }
        Ok(())
    }

    // T-05: the wall's writer for `fit` (§3.4, §5.2), behind `PUT /rooms/{id}/fit`.
    /// Record the wall's measured verdict. A verdict that did not change does
    /// not bump the revision, so it pushes nothing.
    pub fn record_fit(&self, room_id: &str, fit: Fit) -> Result<(), RoomError> {
        let mut rooms = lock(&self.rooms);
        let entry = rooms.get_mut(room_id).ok_or(RoomError::NotFound)?;
        entry.room.record_fit(fit);
        Ok(())
    }

    /// Run `f` over a room, read-only. Views go through here.
    pub fn with_room<T>(&self, room_id: &str, f: impl FnOnce(&Room) -> T) -> Result<T, RoomError> {
        let rooms = lock(&self.rooms);
        let entry = rooms.get(room_id).ok_or(RoomError::NotFound)?;
        Ok(f(&entry.room))
    }

    /// The host check, then `f` — for the host projection.
    pub fn with_hosted_room<T>(
        &self,
        room_id: &str,
        bearer: Option<&str>,
        f: impl FnOnce(&Room) -> T,
    ) -> Result<T, RoomError> {
        let rooms = lock(&self.rooms);
        let entry = rooms.get(room_id).ok_or(RoomError::NotFound)?;
        self.auth.authorize_host(&entry.room, bearer)?;
        Ok(f(&entry.room))
    }
}
