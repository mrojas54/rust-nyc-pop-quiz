//! The room record and the writes the phase machine drives.

use std::sync::Arc;
use std::time::SystemTime;

use serde::Serialize;
use subtle::ConstantTimeEq;

use crate::answers::{Scheduled, Verdict};
use crate::auth::OrganizerId;
use crate::club::{ClubSlug, Zone};
use crate::copy;
use crate::phase::{apply, Applied, Command, HostAction, Machine, Phase, Refused, State};
use crate::used::TakeHome;

use super::create::{hex, random_bytes};
use super::lifecycle::Ended;
use super::{CloseSnapshot, LiveCounts, Totals, ROOM_LIFETIME};

/// The wall's measured verdict after refit (§5.2, AC-100). Written by the wall
/// (T-05 adds its route); read by the host's fit line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Fit {
    Fits,
    ClippedX,
    ClippedY,
    ClippedXy,
}

impl Fit {
    pub fn host_line(self) -> &'static str {
        match self {
            Fit::Fits => copy::HOST_FIT_FITS,
            Fit::ClippedX => copy::HOST_FIT_CLIPPED_X,
            Fit::ClippedY => copy::HOST_FIT_CLIPPED_Y,
            Fit::ClippedXy => copy::HOST_FIT_CLIPPED_XY,
        }
    }
}

/// Frozen at `closed`: never written again.
pub(super) struct Frozen {
    pub(super) answered: u32,
    pub(super) totals: Totals,
    pub(super) verdict: Verdict,
}

/// Where links point. Configuration, owned by T-09 on deploy.
#[derive(Clone, Debug)]
pub struct Urls {
    /// Joins are `{base}/{code}`; the host's resume link is under it too.
    pub base: String,
    /// The take-it-home page the released wall links to (§5.5).
    pub home: String,
}

impl Default for Urls {
    fn default() -> Urls {
        Urls {
            base: "http://127.0.0.1:3000".into(),
            home: "http://127.0.0.1:3000/home".into(),
        }
    }
}

/// One room. No field is public (§3.4). Fields a sibling file reads are
/// `pub(super)`; the six none of them reads are private to this file.
pub struct Room {
    pub(super) id: String,
    pub(super) code: String,
    pub(super) join_url: String,
    // D-26: which club's room this is, and the zone its meetup date is read in.
    pub(super) club: ClubSlug,
    zone: Zone,
    pub(super) question_id: String,
    pub(super) question: Arc<Scheduled>,
    pub(super) machine: Machine,
    pub(super) host_session: String,
    pub(super) host_resume_url: String,
    pub(super) organizer: OrganizerId,
    created_at: SystemTime,
    expires_at: SystemTime,
    pub(super) present: u32,
    pub(super) answered_live: u32,
    pub(super) frozen: Option<Frozen>,
    pub(super) released_at: Option<SystemTime>,
    // GAP-8: how it ended, once a new room on its question was let in on the
    // strength of that ending. From then on it stays ended at any `now`.
    pub(super) latched_end: Option<Ended>,
    pub(super) fit: Option<Fit>,
    revision: u64,
    // T-11: §4.6's quiet clock, written by `create` and every host action.
    last_host_action: SystemTime,
    // T-11: what the release transition built while it still held the reveal
    // witness, until `AppState::act` takes it under the same lock.
    parting: Option<Parting>,
}

/// T-11: what a room leaves behind at release — the take-it-home snapshot and
/// the wall's last verdict, for the used record.
pub(super) struct Parting {
    pub(super) take_home: TakeHome,
    pub(super) meetup_date: String,
    pub(super) fit: Option<Fit>,
}

/// What the machine did with a command.
#[derive(Debug, PartialEq, Eq)]
pub enum Acted {
    /// The room moved: a transition, or a step.
    Changed,
    /// *Run it again*: the caller makes a new room. This one is unchanged.
    NewRoomRequested,
}

impl Room {
    pub fn create(
        question: Arc<Scheduled>,
        organizer: OrganizerId,
        club: ClubSlug,
        zone: Zone,
        code: String,
        urls: &Urls,
        now: SystemTime,
    ) -> Result<Room, Refused> {
        let len = question.public().trace_len();
        let machine = match apply(State::Unmade(len), Command::Host(HostAction::CreateRoom))? {
            Applied::Room(m) => m,
            Applied::NewRoom => unreachable!("Create a room never asks for another room"),
        };
        let id = hex(&random_bytes::<16>());
        let host_session = hex(&random_bytes::<32>());
        Ok(Room {
            join_url: format!("{}/{code}", urls.base),
            host_resume_url: format!("{}/host/{id}#{host_session}", urls.base),
            club,
            zone,
            question_id: question.public().id().to_string(),
            question,
            machine,
            host_session,
            organizer,
            code,
            id,
            created_at: now,
            expires_at: now + ROOM_LIFETIME,
            present: 0,
            answered_live: 0,
            frozen: None,
            released_at: None,
            latched_end: None,
            fit: None,
            revision: 0,
            last_host_action: now,
            parting: None,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn question_id(&self) -> &str {
        &self.question_id
    }

    /// The club this room belongs to (D-26).
    pub fn club(&self) -> &ClubSlug {
        &self.club
    }

    /// The one phase field. The wall, buzzer and host projections all read it
    /// (AC-81).
    pub fn phase(&self) -> Phase {
        self.machine.phase()
    }

    pub fn trace_step(&self) -> Option<u16> {
        self.machine.trace_step()
    }

    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    pub fn organizer(&self) -> &OrganizerId {
        &self.organizer
    }

    pub fn host_resume_url(&self) -> &str {
        &self.host_resume_url
    }

    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    pub fn expires_at(&self) -> SystemTime {
        self.expires_at
    }

    pub fn released_at(&self) -> Option<SystemTime> {
        self.released_at
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether an answer write is accepted now: only while `live` (§4.3).
    pub fn accepts_answers(&self) -> bool {
        self.phase() == Phase::Live
    }

    /// Whether `bearer` is this room's host session. Constant-time. The
    /// session never rotates during the room (AC-50): there is no setter.
    pub fn host_matches(&self, bearer: &str) -> bool {
        self.host_session.as_bytes().ct_eq(bearer.as_bytes()).into()
    }

    /// T-04b's writer for `present` and `answered_live`. `present` keeps
    /// counting after `closed`; the frozen `answered` does not (§4.4).
    pub fn set_live_counts(&mut self, counts: LiveCounts) {
        if (self.present, self.answered_live) != (counts.present, counts.answered_live) {
            self.present = counts.present;
            self.answered_live = counts.answered_live;
            self.revision += 1;
        }
    }

    /// The wall's writer for `fit` (§5.2). A released room has handed its
    /// verdict to the used record and takes no more.
    pub fn record_fit(&mut self, fit: Fit) {
        if self.phase() != Phase::Released && self.fit != Some(fit) {
            self.fit = Some(fit);
            self.revision += 1;
        }
    }

    /// Apply a host command. The only place `machine` is reassigned, and only
    /// from [`apply`]'s result (G-6, AC-45). `close` is called when — and only
    /// when — the room enters `closed`.
    pub fn act(
        &mut self,
        command: Command,
        now: SystemTime,
        close: impl FnOnce() -> CloseSnapshot,
    ) -> Result<Acted, Refused> {
        let next = match apply(State::Room(self.machine.clone()), command)? {
            Applied::NewRoom => return Ok(Acted::NewRoomRequested),
            Applied::Room(next) => next,
        };
        if next.phase() != self.machine.phase() {
            match next.phase() {
                Phase::Closed => {
                    let snapshot = close();
                    let verdict = self.question.judge(&snapshot.totals());
                    self.frozen = Some(Frozen {
                        answered: snapshot.answered(),
                        totals: snapshot.totals(),
                        verdict,
                    });
                }
                // T-11: the machine is still `reveal` here, so `open` holds the
                // witness. Copy out what outlives the room, then drop what is
                // per person: the live counts here, the sessions in
                // `AppState::act` (AC-56). The anonymous totals, `answered`
                // and the verdict stay on the shell and expire with it at four
                // hours — AC-56 read literally, the Orchestrator's ruling.
                Phase::Released => {
                    let meetup_date = crate::used::meetup_date_in(self.created_at, self.zone);
                    let take_home = {
                        let opened = self.open().expect("release is entered only from reveal");
                        crate::used::take_home(&self.question_id, meetup_date.clone(), &self.public(), &opened)
                    };
                    self.parting = Some(Parting {
                        take_home,
                        meetup_date,
                        fit: self.fit.take(),
                    });
                    self.present = 0;
                    self.answered_live = 0;
                    self.released_at = Some(now);
                }
                Phase::Idle | Phase::Live | Phase::Split | Phase::Work | Phase::Reveal => {}
            }
        }
        self.machine = next;
        self.revision += 1;
        self.last_host_action = now;
        Ok(Acted::Changed)
    }

    /// T-11: whether this room has ended at `now` — four hours up, or quiet for
    /// §4.6's bound (`lifecycle::verdict`).
    ///
    /// Once [`AppState::create_for`] has let a new room in on this room's
    /// question because it had ended, the ending is latched: a later call with
    /// an earlier `now` — a wall-clock step back, or a host action whose clock
    /// reading was taken just before the lock — cannot revive it, so there are
    /// never two live rooms on one question (GAP-8).
    pub fn ended(&self, now: SystemTime) -> Option<Ended> {
        self.latched_end
            .or_else(|| super::lifecycle::verdict(self.phase(), self.expires_at, self.last_host_action, now))
    }

    /// GAP-8: whether this room holds `question_id` at `now` against a new
    /// room — it was created on it, is not released, and has not ended. A
    /// released room's question is refused by the used ledger instead; a room
    /// that ended without release can never release, so it can never put its
    /// answer on `/last` ([`AppState::act`] refuses an ended room).
    pub fn holds(&self, club: &ClubSlug, question_id: &str, now: SystemTime) -> bool {
        &self.club == club && self.question_id == question_id && self.phase() != Phase::Released && self.ended(now).is_none()
    }

    /// T-11: the last host action (§4.6).
    pub fn last_host_action(&self) -> SystemTime {
        self.last_host_action
    }

    pub(super) fn take_parting(&mut self) -> Option<Parting> {
        self.parting.take()
    }
}
