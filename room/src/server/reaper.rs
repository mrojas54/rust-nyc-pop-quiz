//! The reaper (T-11): the task that sweeps ended rooms and closes their sockets.
//! What ends a room is in [`crate::rooms::lifecycle`]; this only runs the sweep.

use std::sync::Arc;
use std::time::Duration;

use crate::rooms::AppState;

/// How often the reaper sweeps. A room past its bound is refused on every
/// touch before this; the sweep is only what deletes it and closes sockets.
pub const REAP_EVERY: Duration = Duration::from_secs(30);

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
