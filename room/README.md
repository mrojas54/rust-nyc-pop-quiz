# room

The live half. One room per meetup, one question, seven phases from `idle` to
`released`, driven from the host's phone and shown on the wall. Rust, `axum` and
`tokio`, one small machine on Fly.io.

## Running it

From the repository root:

    just setup
    just test       # the room's tests, along with the pipeline's and the web's
    just canary     # just the in-process secrecy seam

Or directly:

    cd room && cargo test
    cd room && cargo run     # serves on 127.0.0.1:3000

## What is here

`router()` and a binary that serves it. No routes yet — that is deliberate, and
it is what lets `tests/canary.rs` prove the harness without asserting anything
about room behaviour.

The modules that will hold the behaviour are named in `BUILDPLAN.md` section 2:

| Module | What it will do | Ticket |
|---|---|---|
| `phase` | `idle → live → closed → split → work → reveal → released`, no skipping | T-04a |
| `answers` | **The sealed one.** Correct option, receipt, explanation | T-04a |
| `rooms` | Sessions, capacity, totals frozen at close | T-04b |
| `ws` | One broadcast per room: wall, buzzers, host | T-04c |
| `auth` | Discord OAuth, role-**ID** check at room creation | T-10 |

`answers` is sealed in the structural sense: it must be unreachable from the
public state query by a module boundary or the type system, not by anyone
remembering to be careful (SPEC.md G-3, AC-61).

## The toolchain

This crate builds with whatever `cargo` the machine has, and is not pinned.

That is not an oversight, and it is worth being precise about because the two
things are easy to confuse: **the verification pin is a different pin.** The
full `rustc -Vv` release and commit-hash, plus the nightly's date for Miri, are
defined once by T-15a where the sandbox image is built, and they exist so that a
question's verified answer can be tied to an exact compiler. Nothing about
building this server needs that, and putting a `rust-toolchain.toml` here would
both invite the confusion and make `just test` reach for the network on any
machine that did not happen to have the pinned toolchain already.

## The canary seam

`tests/canary.rs` drives `router()` in-process, with no socket. Today it asserts
one thing — that the router answers at all — because the mechanism is the point.
T-08 lands the real scan on top of it: canaries planted in the resolving trace
step's `note`, the explanation, the receipt and the hint, asserted absent from
every pre-reveal payload. T-25 adds the admin token as a fifth plant.

The other half of that hook — scanning the deployed room's real frames and pages
— runs in `just test-full` once T-09 has something deployed.
