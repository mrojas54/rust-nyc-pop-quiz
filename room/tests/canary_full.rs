//! `canary`, the `test-full` half (T-08): the same scan as `canary.rs`, with
//! every HTTP request over a real TCP connection to a room served on loopback
//! by `room::ws::serve` — the function the binary's deploy path serves with —
//! over two rooms back to back (the second made by *Run it again*), plus the
//! reconnect path: in `closed`, a buzzer's socket drops and re-attaches with
//! the same token, and another's is replaced by a second socket; each attach
//! frame carries only that session's own saved letter (AC-37, AC-57, AC-58).
//!
//! `#[ignore]`d so `just test` never runs it; `just canary --full` and
//! `just test-full` do.
//!
//! **The deployed room.** `EVALUATION.md` puts the scan of the *deployed*
//! room's frames and pages in `test-full`. The scan core takes a server base
//! URL, and `just canary --url <u>` is the hook; it cannot run yet, because
//! nothing can create a room on a deployed server before T-09's stand-in
//! token (SPEC §8.2) and nothing can put the planted question on one before
//! T-25's admin push (§8.3). With `CANARY_URL` set, this suite says so and
//! fails rather than scanning a local room under a deployed name.

mod common;
#[path = "canary_scan/mod.rs"]
mod canary_scan;

use canary_scan::*;
use common::*;
use room::phase::Phase;

fn refuse_a_deployed_url() {
    if let Ok(url) = std::env::var("CANARY_URL") {
        panic!(
            "canary --url {url}: the deployed-room scan needs T-09's stand-in token to create a room and \
             T-25's admin push to plant the canary question on it; neither exists yet. \
             T-09 wires this hook into `smoke` (room/README.md, Canary)."
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "test-full: `just canary --full`"]
async fn over_real_sockets_two_rooms_and_the_reconnect_path() {
    refuse_a_deployed_url();
    let server = Server::start(Http::Tcp).await;
    let p = plants();

    // Room one: the canary question, the reconnect path in `closed`, and
    // *Run it again* onto the does-not-compile twin.
    let first = walk_room(
        &server,
        Start::Question("canary"),
        Options { run_again_with: Some("canary-dnc"), reconnect: true },
    )
    .await;
    assert_the_plants_arrived(&first);
    assert_eq!(first.reveal_wall["reveal"]["correct"], "E");
    assert!(first.seen.has(Surface::Wall, Phase::Reveal, &p.receipt));

    // Room two: the room *Run it again* made, walked the whole way too.
    let next = first.next_room.clone().expect("run it again made a room");
    let second = walk_room(&server, Start::Room(next), Options { run_again_with: None, reconnect: true }).await;
    assert_the_plants_arrived(&second);
    assert_eq!(second.reveal_wall["reveal"]["correct"], "D");
    assert!(second.seen.has(Surface::Wall, Phase::Reveal, &p.error_code));

    // Every request above went over TCP: the server logged each one.
    assert!(server.log_len() > 500, "{} requests", server.log_len());
}
