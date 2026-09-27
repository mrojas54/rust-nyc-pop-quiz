//! Binds a socket and serves the room.
//!
//! The environment decides the bind and the public address (`room::config`):
//! on Fly, `PORT` and `POPQUIZ_PUBLIC_URL`; locally, neither, and the room is
//! at `http://127.0.0.1:3000`. `ws::serve` sets `TCP_NODELAY` on every socket
//! (the spike's finding, PQ-6). A `dev-host-token` build also needs
//! `HOST_DEV_TOKEN`, and refuses to start without it (SPEC §8.2).

use std::process::ExitCode;
use std::sync::Arc;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match room::config::Config::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("room: {e}");
            return ExitCode::from(2);
        }
    };
    let bind = config.bind;
    let base = config.urls.base.clone();
    let state = room::serving_state(config);
    let listener = match tokio::net::TcpListener::bind(bind).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("room: cannot bind {bind}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let dev_host = if cfg!(feature = "dev-host-token") { "on" } else { "off" };
    eprintln!("room listening on {bind}; public at {base}; dev-host-token {dev_host}");
    match room::ws::serve(listener, room::router_with(Arc::new(state))).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("room: {e}");
            ExitCode::FAILURE
        }
    }
}
