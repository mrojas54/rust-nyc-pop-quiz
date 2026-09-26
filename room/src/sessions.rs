//! Participant sessions and the answer store (T-04b, SPEC.md §3.4, §4.1, §4.3).
//!
//! A session is a `token` and an `answer ∈ A..E | none`, and nothing else: no
//! identity, no device record, no hint flag (AC-57, AC-48). Sessions live in
//! memory beside their room and are dropped whole at release (AC-56, D-12).
//!
//! [`SessionMap`] implements [`crate::rooms::Sessions`]. The room asks it for
//! the close snapshot once, at `closed`, and tells it to drop everything at
//! `released`; [`crate::rooms::AppState`] routes joins, answers and leaves to it
//! and writes `present` and `answered_live` from [`Sessions::counts`] under the
//! same lock.
//!
//! **Ghost sessions.** A session whose socket dropped is still a session until
//! [`Sessions::leave`] or release: it counts in `present`, it holds a capacity
//! slot, and its answer counts in `totals`. `leave` is T-04c's call once a
//! socket is gone for good — never on a mere drop, or re-attaching with the same
//! token (AC-37) would lose the answer.

use std::collections::HashMap;
use std::hash::{BuildHasher, RandomState};

use crate::copy;
use crate::question::Letter;
use crate::rooms::{CloseSnapshot, LiveCounts, Sessions, Totals, CODE_ALPHABET};
use crate::ws::SessionId;

/// Sessions per room (SPEC §4.1, §9). Configurable through
/// [`crate::rooms::AppState::with_capacity`].
pub const DEFAULT_CAPACITY: usize = 200;

/// The only credential a buzzer ever has: 32 random bytes, hex. Opaque and
/// unguessable; it names nothing and outlives nothing.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Token(String);

impl Token {
    fn new() -> Token {
        let mut buf = [0u8; 32];
        getrandom::fill(&mut buf).expect("the OS random source is available");
        Token(buf.iter().map(|b| format!("{b:02x}")).collect())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One participant session: exactly these two fields (AC-57).
pub struct Session {
    pub token: Token,
    pub answer: Option<Letter>,
}

// AC-57, at compile time: this pattern names every field and has no `..`, so
// adding a field to `Session` stops the crate compiling.
const _: fn(Session) = |Session { token: _, answer: _ }| {};

/// Why a join did not happen: SPEC §4.1's six failure states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinRefusal {
    Malformed,
    Unknown,
    NotYetOpen,
    AlreadyEnded,
    ClosedForInactivity,
    Full,
}

impl JoinRefusal {
    pub const ALL: [JoinRefusal; 6] = [
        JoinRefusal::Malformed,
        JoinRefusal::Unknown,
        JoinRefusal::NotYetOpen,
        JoinRefusal::AlreadyEnded,
        JoinRefusal::ClosedForInactivity,
        JoinRefusal::Full,
    ];

    /// The machine-readable name the buzzer branches on.
    pub fn slug(self) -> &'static str {
        match self {
            JoinRefusal::Malformed => "malformed",
            JoinRefusal::Unknown => "unknown",
            JoinRefusal::NotYetOpen => "not_yet_open",
            JoinRefusal::AlreadyEnded => "already_ended",
            JoinRefusal::ClosedForInactivity => "closed_for_inactivity",
            JoinRefusal::Full => "full",
        }
    }

    /// Its §11 sentence, with the next step in it (AC-29).
    pub fn message(self) -> &'static str {
        match self {
            JoinRefusal::Malformed => copy::JOIN_FAIL_MALFORMED,
            JoinRefusal::Unknown => copy::JOIN_FAIL_UNKNOWN,
            JoinRefusal::NotYetOpen => copy::JOIN_FAIL_NOT_YET_OPEN,
            JoinRefusal::AlreadyEnded => copy::JOIN_FAIL_ALREADY_ENDED,
            JoinRefusal::ClosedForInactivity => copy::JOIN_FAIL_CLOSED_INACTIVITY,
            JoinRefusal::Full => copy::JOIN_FAIL_FULL,
        }
    }
}

/// A typed or linked room code, normalised: surrounding space trimmed and
/// letters upper-cased, then six symbols from [`CODE_ALPHABET`] or nothing.
pub fn parse_code(input: &str) -> Option<String> {
    let code = input.trim().to_ascii_uppercase();
    (code.len() == 6 && code.bytes().all(|b| CODE_ALPHABET.contains(&b))).then_some(code)
}

/// `"A"`…`"E"`, exactly.
pub fn parse_letter(input: &str) -> Option<Letter> {
    Letter::ALL.into_iter().find(|l| l.as_str() == input)
}

/// One room's sessions.
#[derive(Default)]
pub struct SessionMap {
    sessions: HashMap<Token, Session>,
    // PQ-32: the key for the transport's per-session handle. One per map, not
    // per session, so a `Session` stays its token and its answer (AC-57).
    ids: RandomState,
}

impl SessionMap {
    pub fn new() -> SessionMap {
        SessionMap::default()
    }

    // PQ-32: a session's stable handle for the transport — a keyed hash of its
    // token, so the handle carries nothing of the credential. `join` never
    // admits two sessions with the same handle.
    fn id_of(&self, token: &Token) -> SessionId {
        SessionId(self.ids.hash_one(token))
    }
}

impl Sessions for SessionMap {
    fn close_snapshot(&self) -> CloseSnapshot {
        let mut totals: Totals = [0; 5];
        for letter in self.sessions.values().filter_map(|s| s.answer) {
            totals[letter.index()] += 1;
        }
        CloseSnapshot::from_totals(totals)
    }

    fn release(&mut self) {
        self.sessions = HashMap::new();
    }

    fn join(&mut self, capacity: usize) -> Option<Token> {
        // Checked before a token is drawn: a refused join creates nothing and
        // reserves nothing (AC-30).
        if self.sessions.len() >= capacity {
            return None;
        }
        let token = loop {
            let token = Token::new();
            // PQ-32: distinct handles imply distinct tokens.
            let id = self.id_of(&token);
            if self.sessions.keys().all(|t| self.id_of(t) != id) {
                break token;
            }
        };
        self.sessions.insert(
            token.clone(),
            Session {
                token: token.clone(),
                answer: None,
            },
        );
        Some(token)
    }

    fn upsert(&mut self, token: &str, letter: Letter) -> Option<Letter> {
        let session = self.sessions.get_mut(&Token(token.to_string()))?;
        session.answer = Some(letter);
        Some(letter)
    }

    fn answer_of(&self, token: &str) -> Option<Option<Letter>> {
        self.sessions.get(&Token(token.to_string())).map(|s| s.answer)
    }

    fn leave(&mut self, token: &str) -> bool {
        self.sessions.remove(&Token(token.to_string())).is_some()
    }

    fn counts(&self) -> LiveCounts {
        LiveCounts {
            present: self.sessions.len() as u32,
            answered_live: self.sessions.values().filter(|s| s.answer.is_some()).count() as u32,
        }
    }

    fn session_count(&self) -> usize {
        self.sessions.len()
    }

    // PQ-32: the transport's view of the map.

    fn session_id(&self, token: &str) -> Option<SessionId> {
        let token = Token(token.to_string());
        self.sessions.contains_key(&token).then(|| self.id_of(&token))
    }

    fn saved_by_id(&self, id: SessionId) -> Option<Option<Letter>> {
        self.sessions.values().find(|s| self.id_of(&s.token) == id).map(|s| s.answer)
    }
}
