# Rust NYC Pop Quiz

Get your Rust friends together for a live pop quiz. You host, they play from
their phones, and every Rust program is generated fresh and compiler-verified.

## Status

Design phase. There is no implementation yet — no Rust, TypeScript, or Python
source, and no build or test tooling. This repository currently holds the
product requirements document and the configuration scaffolding that goes with
it.

## What's here

- **[`PRD.md`](PRD.md)** — the full design, and the place to start: product
  summary, Discord-based organizer authorization, the Val Town / Modal trust
  boundary, public interfaces, the brand and accessibility contract, acceptance
  tests, and delivery requirements.
- **[`.env.example`](.env.example)** — every environment variable the PRD
  requires, as names only. No value is ever assigned anywhere in this
  repository; Val Town and Modal each store secrets server-side and reference
  them by name.

## How it is meant to work

Rust NYC Pop Quiz is a host-led meetup quiz that generates original Rust
program-output questions for every session. Participants join anonymously from
their phones. Hosts sign in through Discord and must hold the `nyc-organizers`
role in the Rust East Coast server.

Every accepted question is checked by a pinned Rust compiler and Miri
configuration before the room opens, and the correct choice, explanation, and
verification outcome are withheld from every pre-reveal client payload.

A Val Town application is the participant-facing host and durable system of
record. A separate Modal application performs candidate generation and isolated
Rust verification.

## Reference implementation

[`colelawrence/rust-nyc-talk-submissions`](https://github.com/colelawrence/rust-nyc-talk-submissions)
is a deployed Val Town application on the same stack — React plus Hono,
val-scoped SQLite, git-synced to GitHub through an Actions workflow — wearing
the Rust NYC brand. It is the layout and tooling model to follow when code lands
here.

Its `BRAND_STYLE_GUIDE.md` is the source of the PRD's brand section, with two
deliberate overrides where the PRD corrects AA contrast failures. Where the two
disagree, the PRD is normative.
