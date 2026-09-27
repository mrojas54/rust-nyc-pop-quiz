//! How long a room lives, and what ends it (T-11, SPEC.md §4.6, AC-69).
//!
//! A room lives at most [`crate::rooms::ROOM_LIFETIME`] from creation. Before
//! that it can **close for inactivity**: in `idle` after [`IDLE_QUIET`] with no
//! host action, in any later phase up to `reveal` after [`LATER_QUIET`]. A
//! released room is already over; it keeps only its phase shell (no sessions,
//! no per-person anything; only the anonymous totals, which expire with it) until its four hours are up, so the wall can say
//! *Let's go to the bar.* and the host can *Run it again*.
//!
//! **Two halves, one deletion path.** The participant and host entry points
//! on [`crate::rooms::AppState`] — `join`, `answer`, `buzzer_for`, `act`,
//! `run_again` — ask [`verdict`] and refuse a room that has ended, so there is
//! never a window in which an expired room admits a join, an answer or a host
//! command. The read-only projections and the wall's `fit` writer do not ask:
//! until the next sweep (at most [`REAP_EVERY`]) they still serve the room as
//! it last stood, which carries nothing a released room would not. Only
//! [`crate::rooms::AppState::sweep`] deletes, and it hands
//! back the ids it deleted so [`spawn_reaper`] can tell the transport, which
//! closes those rooms' sockets with `4404` (`ws::close::ROOM_GONE`).
//!
//! **Time comes from a [`Clock`]**, never from `SystemTime::now()` inline:
//! [`SystemClock`] in the binary, [`ManualClock`] in tests, which is how a test
//! drives a room to its fourth hour without waiting for it.

use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::phase::Phase;
use crate::rooms::AppState;
use crate::sessions::JoinRefusal;

/// §4.6: a room in `idle` with no host action for this long closes.
pub const IDLE_QUIET: Duration = Duration::from_secs(30 * 60);

/// §4.6: a room in `live` … `reveal` with no host action for this long closes.
pub const LATER_QUIET: Duration = Duration::from_secs(20 * 60);

/// How long an ended room's code is remembered, so a late join is told the
/// room *ended* or *went quiet* rather than that it never existed (AC-29).
/// The memory is the code and the reason, nothing else.
pub const ENDED_MEMORY: Duration = Duration::from_secs(4 * 60 * 60);

/// How often the reaper sweeps. A room past its bound is refused on every
/// touch before this; the sweep is only what deletes it and closes sockets.
pub const REAP_EVERY: Duration = Duration::from_secs(30);

/// Where the room reads the time.
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}

/// The wall clock. The binary's, and every state's by default.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

/// A clock that moves only when told to. Tests, and `test-full`'s AC-69 row.
pub struct ManualClock(Mutex<SystemTime>);

impl ManualClock {
    pub fn new(at: SystemTime) -> Arc<ManualClock> {
        Arc::new(ManualClock(Mutex::new(at)))
    }

    pub fn set(&self, at: SystemTime) {
        *self.0.lock().unwrap_or_else(|p| p.into_inner()) = at;
    }

    pub fn advance(&self, by: Duration) {
        let mut t = self.0.lock().unwrap_or_else(|p| p.into_inner());
        *t += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> SystemTime {
        *self.0.lock().unwrap_or_else(|p| p.into_inner())
    }
}

/// Why a room is over, other than having been released.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ended {
    /// Four hours from creation (AC-69).
    Expired,
    /// No host action for §4.6's bound.
    Inactive,
}

impl Ended {
    /// What a participant is told (AC-29): PQ-8's two states and copy.
    pub fn refusal(self) -> JoinRefusal {
        match self {
            Ended::Expired => JoinRefusal::AlreadyEnded,
            Ended::Inactive => JoinRefusal::ClosedForInactivity,
        }
    }

    /// What a host command is told. An API diagnostic for the host page, like
    /// `phase::Refused`'s reasons — not §11 copy, never shown to participants.
    pub fn host_reason(self) -> &'static str {
        match self {
            Ended::Expired => "This room has ended.",
            Ended::Inactive => "This room closed for inactivity.",
        }
    }
}

/// Whether a room with these facts has ended at `now`. Expiry first: a room
/// past four hours has ended however recently the host touched it.
pub fn verdict(phase: Phase, expires_at: SystemTime, last_host_action: SystemTime, now: SystemTime) -> Option<Ended> {
    if now >= expires_at {
        return Some(Ended::Expired);
    }
    let quiet = now.duration_since(last_host_action).unwrap_or(Duration::ZERO);
    let bound = match phase {
        Phase::Idle => IDLE_QUIET,
        Phase::Live | Phase::Closed | Phase::Split | Phase::Work | Phase::Reveal => LATER_QUIET,
        // Already over; it goes at four hours.
        Phase::Released => return None,
    };
    (quiet >= bound).then_some(Ended::Inactive)
}

/// Sweep every [`REAP_EVERY`] and tell the transport which rooms went. Does
/// nothing outside a tokio runtime, so building a router in a plain `#[test]`
/// starts no task. The task lives as long as its runtime (the transport holds
/// the state, so there is no earlier moment at which the state is gone).
pub fn spawn_reaper(state: Arc<AppState>, transport: crate::ws::Transport) {
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        return;
    };
    runtime.spawn(async move {
        let mut tick = tokio::time::interval(REAP_EVERY);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        tick.tick().await;
        loop {
            tick.tick().await;
            for id in state.sweep(state.now()) {
                transport.changed(&id);
            }
        }
    });
}
