//! The room record (SPEC.md §3.4) and the in-memory set of rooms.
//!
//! T-04a owns the record's **shape** and the writes the phase machine drives:
//! `closed` freezes `answered` and `totals` from a snapshot the sessions module
//! hands over, and decides §4.5's middle beat once; `released` sets
//! `released_at`. Everything about sessions — joins, capacity, the answer
//! upsert, refusal after close — is T-04b's, and the socket is T-04c's. They
//! plug in here:
//!
//! - [`Sessions`] — T-04b implements it over its per-room session map. The
//!   close transition calls [`Sessions::close_snapshot`] exactly once; release
//!   calls [`Sessions::release`].
//! - [`AppState::set_live_counts`] — the counts' writer from outside the lock.
//!   T-04b's own join, leave and upsert write the same fields through
//!   [`Room::set_live_counts`] while they already hold it.
//! - [`Room::accepts_answers`] — T-04b's upsert asks it; it is `true` only in
//!   `live`.
//! - [`Room::revision`] — bumped on every change; T-04c broadcasts on it.
//! - [`crate::view`] — the three projections T-04c pushes down the socket.
//!
//! **The one way to the answers** is [`Room::open`], which asks the machine for
//! a [`crate::phase::RevealWitness`] and returns `None` in every phase but
//! `reveal`. There is no getter for the question's vault. [`Room::public`]
//! returns a [`PublicView`], which has no path to the vault at all.
//!
//! ```compile_fail
//! // AC-61: a room's question (and its vault) is not reachable by field.
//! fn peek(r: &room::rooms::Room) -> &room::answers::Scheduled { &r.question }
//! ```
//!
//! ```
//! // Twin: the same types, the one way there is.
//! fn read(r: &room::rooms::Room) -> Option<room::question::Letter> { r.open().map(|o| o.revealed.correct) }
//! ```
//!
//! ```compile_fail
//! // AC-61: the public view has no way to the vault.
//! fn peek(v: room::rooms::PublicView<'_>) -> Option<room::question::Letter> { v.open().map(|o| o.revealed.correct) }
//! ```
//!
//! ```
//! // Twin.
//! fn read(v: room::rooms::PublicView<'_>) -> room::phase::Phase { v.phase }
//! ```
//!
//! ```compile_fail
//! // G-3: what was opened in `reveal` cannot be held across a phase change.
//! use room::phase::{Command, HostAction};
//! use room::rooms::{CloseSnapshot, Room};
//! fn keep(r: &mut Room) {
//!     let opened = r.open();
//!     let _ = r.act(Command::Host(HostAction::ReleaseRoom), std::time::SystemTime::now(), || CloseSnapshot::from_totals([0; 5]));
//!     drop(opened);
//! }
//! ```
//!
//! ```
//! // Twin: the same calls, with the opened view dropped before the room moves.
//! use room::phase::{Command, HostAction};
//! use room::rooms::{CloseSnapshot, Room};
//! fn keep(r: &mut Room) {
//!     let opened = r.open();
//!     drop(opened);
//!     let _ = r.act(Command::Host(HostAction::ReleaseRoom), std::time::SystemTime::now(), || CloseSnapshot::from_totals([0; 5]));
//! }
//! ```

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime};

use serde::Serialize;
use subtle::ConstantTimeEq;

use crate::answers::{Middle, Revealed, Scheduled, Verdict};
use crate::auth::{Denied, HostAuth, OrganizerId};
use crate::copy;
use crate::phase::{apply, Applied, Command, HostAction, Machine, Phase, Refused, State};
use crate::question::PublicQuestion;

/// Per-option counts, A–E in arrival order. The only answer data a room holds
/// (AC-56, D-12).
pub type Totals = [u32; 5];

/// A room lives at most four hours from creation (AC-69).
pub const ROOM_LIFETIME: Duration = Duration::from_secs(4 * 60 * 60);

/// The code alphabet: letters and digits without `O 0 I 1` (SPEC §3.4). 32
/// symbols, so a random byte masked to five bits picks one without bias.
pub const CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Counts the sessions module keeps current (T-04b).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LiveCounts {
    /// Live participant sessions (§3.4 `present`).
    pub present: u32,
    /// Sessions currently holding an answer (§3.4 `answered_live`).
    pub answered_live: u32,
}

/// What the sessions module hands the close transition: the per-option
/// totals, taken once. `answered` is their sum, so the two cannot disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CloseSnapshot {
    totals: Totals,
}

impl CloseSnapshot {
    pub fn from_totals(totals: Totals) -> CloseSnapshot {
        CloseSnapshot { totals }
    }

    pub fn totals(&self) -> Totals {
        self.totals
    }

    pub fn answered(&self) -> u32 {
        self.totals.iter().sum()
    }
}

/// The seam T-04b implements over its per-room sessions.
pub trait Sessions: Send {
    /// Called by the close transition and nothing else, once per room.
    fn close_snapshot(&self) -> CloseSnapshot;
    /// Called at release: drop every session (§3.4, D-12).
    fn release(&mut self) {}

    // T-04b: the participant half, implemented by `sessions::SessionMap`. The
    // defaults describe a room nobody can join, which is what `NoSessions`
    // and the test doubles are.

    /// A new session, if fewer than `capacity` exist. Nothing is created or
    /// reserved when the answer is `None` (AC-30).
    fn join(&mut self, _capacity: usize) -> Option<crate::sessions::Token> {
        None
    }
    /// Last write wins (§4.3). `None` if the token names no session. The
    /// caller has already asked [`Room::accepts_answers`].
    fn upsert(&mut self, _token: &str, _letter: crate::question::Letter) -> Option<crate::question::Letter> {
        None
    }
    /// The session's saved answer; `None` if the token names no session.
    fn answer_of(&self, _token: &str) -> Option<Option<crate::question::Letter>> {
        None
    }
    /// Drop one session whole — T-04c's call when a socket is gone for good.
    fn leave(&mut self, _token: &str) -> bool {
        false
    }
    /// `present` and `answered_live`, from the sessions as they stand.
    fn counts(&self) -> LiveCounts {
        LiveCounts::default()
    }
    /// How many sessions exist (capacity and AC-30's "nothing reserved").
    fn session_count(&self) -> usize {
        0
    }
}

/// T-04a's stand-in until T-04b lands: nobody has answered.
pub struct NoSessions;

impl Sessions for NoSessions {
    fn close_snapshot(&self) -> CloseSnapshot {
        CloseSnapshot::from_totals([0; 5])
    }
}

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
struct Frozen {
    answered: u32,
    totals: Totals,
    verdict: Verdict,
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

/// One room. Every field is private; each has one writer (§3.4).
pub struct Room {
    id: String,
    code: String,
    join_url: String,
    question_id: String,
    question: Arc<Scheduled>,
    machine: Machine,
    host_session: String,
    host_resume_url: String,
    organizer: OrganizerId,
    created_at: SystemTime,
    expires_at: SystemTime,
    present: u32,
    answered_live: u32,
    frozen: Option<Frozen>,
    released_at: Option<SystemTime>,
    fit: Option<Fit>,
    revision: u64,
}

/// What a room shows once it is in `reveal`: the vault's contents and §4.5's
/// verdict, borrowed from the room so they cannot outlive the phase.
pub struct Opened<'r> {
    pub revealed: Revealed<'r>,
    pub middle: &'r Middle,
}

/// Everything about a room that is public in some phase — and nothing that is
/// not. It borrows the room's public fields one by one, never the room, so
/// there is no path from here to the vault.
pub struct PublicView<'r> {
    pub phase: Phase,
    pub trace_step: Option<u16>,
    pub code: &'r str,
    pub join_url: &'r str,
    pub question: &'r PublicQuestion,
    pub present: u32,
    pub answered_live: u32,
    /// `(answered, totals)` from `closed` on.
    pub frozen: Option<(u32, Totals)>,
    pub fit: Option<Fit>,
}

/// What the machine did with a command.
#[derive(Debug, PartialEq, Eq)]
pub enum Acted {
    /// The room moved: a transition, or a step.
    Changed,
    /// *Run it again*: the caller makes a new room. This one is unchanged.
    NewRoomRequested,
}

/// What a new room's creator is handed, once.
#[derive(Clone, Debug, Serialize)]
pub struct Created {
    pub id: String,
    pub code: String,
    pub join_url: String,
    pub host_session: String,
    pub host_resume_url: String,
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    getrandom::fill(&mut buf).expect("the OS random source is available");
    buf
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Six characters from [`CODE_ALPHABET`].
pub fn new_code() -> String {
    random_bytes::<6>()
        .iter()
        .map(|b| CODE_ALPHABET[usize::from(b & 31)] as char)
        .collect()
}

impl Room {
    /// *Create a room*: the one transition into `idle`.
    pub fn create(
        question: Arc<Scheduled>,
        organizer: OrganizerId,
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
            fit: None,
            revision: 0,
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

    /// The wall's writer for `fit` (§5.2).
    pub fn record_fit(&mut self, fit: Fit) {
        if self.fit != Some(fit) {
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
                Phase::Released => self.released_at = Some(now),
                Phase::Idle | Phase::Live | Phase::Split | Phase::Work | Phase::Reveal => {}
            }
        }
        self.machine = next;
        self.revision += 1;
        Ok(Acted::Changed)
    }

    /// The public state: what any projection may read before `reveal`.
    pub fn public(&self) -> PublicView<'_> {
        PublicView {
            phase: self.machine.phase(),
            trace_step: self.machine.trace_step(),
            code: &self.code,
            join_url: &self.join_url,
            question: self.question.public(),
            present: self.present,
            answered_live: self.answered_live,
            frozen: self.frozen.as_ref().map(|f| (f.answered, f.totals)),
            fit: self.fit,
        }
    }

    /// The answers — only in `reveal`, and only for as long as this borrow.
    pub fn open(&self) -> Option<Opened<'_>> {
        let proof = self.machine.revealed()?;
        let frozen = self
            .frozen
            .as_ref()
            .expect("reveal is reached only through closed, which froze the totals");
        Some(Opened {
            revealed: self.question.open(&proof),
            middle: frozen.verdict.open(&proof),
        })
    }
}

// --------------------------------------------------------------------------
// The rooms, in memory. No persistence (D-12: nothing outlives the room but
// the `used` record, which is T-11's).
// --------------------------------------------------------------------------

struct Entry {
    room: Room,
    sessions: Box<dyn Sessions>,
}

type SessionFactory = Box<dyn Fn() -> Box<dyn Sessions> + Send + Sync>;

/// Why a room request did not happen.
#[derive(Debug, PartialEq, Eq)]
pub enum RoomError {
    /// No room with that id.
    NotFound,
    /// The credential was missing or wrong. Says nothing else (AC-70).
    Denied,
    /// The machine or the schedule refused it, in plain words.
    Refused(String),
}

impl From<Denied> for RoomError {
    fn from(_: Denied) -> RoomError {
        RoomError::Denied
    }
}

impl From<Refused> for RoomError {
    fn from(r: Refused) -> RoomError {
        RoomError::Refused(r.reason)
    }
}

/// Every room on this machine, the questions scheduled into it, and the seams.
///
/// Before T-25's admin channel exists, questions are handed in at
/// construction: tests seed fixtures, `main` seeds none (so creation is
/// refused), and T-09 wires the HC-0 mock question.
pub struct AppState {
    rooms: Mutex<HashMap<String, Entry>>,
    questions: Mutex<HashMap<String, Arc<Scheduled>>>,
    /// Questions whose room was released. Its only writer is the release
    /// transition (G-10); T-11's `used` ledger replaces it.
    released_questions: Mutex<HashSet<String>>,
    auth: Arc<dyn HostAuth>,
    sessions: SessionFactory,
    urls: Urls,
    // T-04b: sessions per room (§4.1, §9).
    capacity: usize,
}

// T-04b: what the participant routes hand back.

/// A successful join: the token, once, and the caller's buzzer view.
pub struct Joined {
    pub room_id: String,
    pub token: crate::sessions::Token,
    pub buzzer: crate::view::BuzzerPayload,
}

/// Why an answer write did not happen. The previous answer is intact in every
/// case (AC-36).
#[derive(Debug, PartialEq, Eq)]
pub enum AnswerError {
    /// No room with that id.
    NotFound,
    /// The token names no session in this room — never joined, left, or the
    /// room was released.
    UnknownSession,
    /// Not `live`: refused, with the saved answer restated (§4.3).
    Refused { phase: Phase, saved: Option<crate::question::Letter> },
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
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
                    .map(|q| (q.public().id().to_string(), Arc::new(q)))
                    .collect(),
            ),
            released_questions: Mutex::new(HashSet::new()),
            auth,
            // T-04b: the real sessions by default.
            sessions: Box::new(|| Box::new(crate::sessions::SessionMap::new())),
            urls,
            capacity: crate::sessions::DEFAULT_CAPACITY,
        }
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

    /// *Create a room*: authorized by the one create check (G-9, §8).
    pub fn create_room(
        &self,
        bearer: Option<&str>,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        let organizer = self.auth.authorize_create(bearer)?;
        self.create_for(organizer, question_id, now)
    }

    fn create_for(
        &self,
        organizer: OrganizerId,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        if lock(&self.released_questions).contains(question_id) {
            return Err(RoomError::Refused(
                "That question has already been run. Pick another.".into(),
            ));
        }
        let question = lock(&self.questions)
            .get(question_id)
            .cloned()
            .ok_or_else(|| RoomError::Refused("No question is scheduled with that id.".into()))?;
        let mut rooms = lock(&self.rooms);
        let code = loop {
            let code = new_code();
            if !rooms.values().any(|e| e.room.code == code) {
                break code;
            }
        };
        let room = Room::create(question, organizer, code, &self.urls, now)?;
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
        Ok(created)
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
        self.auth.authorize_host(&entry.room, bearer)?;
        let Entry { room, sessions } = entry;
        match room.act(command, now, || sessions.close_snapshot())? {
            Acted::Changed => {}
            Acted::NewRoomRequested => {
                return Err(RoomError::Refused(
                    "Run it again needs the organizer's credential and a question.".into(),
                ))
            }
        }
        if command == Command::Host(HostAction::ReleaseRoom) {
            sessions.release();
            lock(&self.released_questions).insert(room.question_id.clone());
        }
        Ok(())
    }

    /// *Run it again*: a **new** room, for the organizer who created this one,
    /// on a question that has not been run. This room stays `released`.
    pub fn run_again(
        &self,
        room_id: &str,
        bearer: Option<&str>,
        question_id: &str,
        now: SystemTime,
    ) -> Result<Created, RoomError> {
        let organizer = self.auth.authorize_create(bearer)?;
        {
            let mut rooms = lock(&self.rooms);
            let entry = rooms.get_mut(room_id).ok_or(RoomError::NotFound)?;
            if entry.room.organizer != organizer {
                return Err(RoomError::Denied);
            }
            match entry.room.act(Command::Host(HostAction::RunItAgain), now, || {
                unreachable!("Run it again never closes answers")
            })? {
                Acted::NewRoomRequested => {}
                Acted::Changed => unreachable!("Run it again never moves this room"),
            }
        }
        self.create_for(organizer, question_id, now)
    }

    /// T-04b's writer for the live counts.
    pub fn set_live_counts(&self, room_id: &str, counts: LiveCounts) -> Result<(), RoomError> {
        let mut rooms = lock(&self.rooms);
        let entry = rooms.get_mut(room_id).ok_or(RoomError::NotFound)?;
        entry.room.set_live_counts(counts);
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
        let mut rooms = lock(&self.rooms);
        let entry = rooms
            .values_mut()
            .find(|e| e.room.code == code)
            .ok_or(JoinRefusal::Unknown)?;
        if entry.room.phase() == Phase::Released {
            return Err(JoinRefusal::AlreadyEnded);
        }
        let token = entry.sessions.join(self.capacity).ok_or(JoinRefusal::Full)?;
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
        let mut rooms = lock(&self.rooms);
        let entry = rooms.get_mut(room_id).ok_or(AnswerError::NotFound)?;
        let saved = entry.sessions.answer_of(token).ok_or(AnswerError::UnknownSession)?;
        if !entry.room.accepts_answers() {
            return Err(AnswerError::Refused {
                phase: entry.room.phase(),
                saved,
            });
        }
        let saved = entry.sessions.upsert(token, letter).ok_or(AnswerError::UnknownSession)?;
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
    pub fn buzzer_for(&self, room_id: &str, token: &str) -> Result<crate::view::BuzzerPayload, RoomError> {
        let rooms = lock(&self.rooms);
        let entry = rooms.get(room_id).ok_or(RoomError::NotFound)?;
        let yours = entry.sessions.answer_of(token).ok_or(RoomError::Denied)?;
        Ok(crate::view::buzzer_for(&entry.room, yours))
    }

    /// How many sessions a room holds (AC-30's "nothing reserved", AC-56).
    pub fn session_count(&self, room_id: &str) -> Result<usize, RoomError> {
        let rooms = lock(&self.rooms);
        let entry = rooms.get(room_id).ok_or(RoomError::NotFound)?;
        Ok(entry.sessions.session_count())
    }
}
