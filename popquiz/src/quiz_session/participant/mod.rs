//! Participant operations on [`AppState`](crate::rooms::AppState): join, answer, leave, the buzzer
//! payload for one session, the session count, the live counts, and the
//! socket's token map ([`crate::ws::SessionTokens`]). The state itself, host
//! commands, release and the sweep are `state_manager.rs`'s.

// ----------------------------------------------------------------------
// T-04b: joins, answers and leaves. Each runs under the one `rooms` lock
// and writes `present` and `answered_live` through `Room::set_live_counts`
// before letting go, so the counts and the sessions are never seen apart.
// Invariant: a session change and its live counts share one `rooms` lock.
// ----------------------------------------------------------------------

mod buzzer;
mod counts;
mod join;
mod leave;
mod tokens;
mod vote;
