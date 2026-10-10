# room

The live half. One room per meetup, one question, seven phases from `idle` to
`released`, driven from the host's phone and shown on the wall. Rust, `axum` and
`tokio`, one small machine on Fly.io.

## Running it

From the repository root:

    just setup
    just test       # the room's tests, along with the pipeline's and the web's
    just canary     # the secrecy suite (--full: over real sockets too)

Or directly:

    cd room && cargo test

**A room you can drive, at a desk (T-10).** Hosting is by Discord role (SPEC
§8), so the room needs the Discord application's four values. They are the
client's (H-3) and live only in your shell or in Fly's secrets — never in the
repository, a file, or a command anyone keeps:

    cd room && DISCORD_CLIENT_ID=… DISCORD_CLIENT_SECRET=… DISCORD_GUILD_ID=… DISCORD_ROLE_ID=… \
        POPQUIZ_PUBLIC_URL=http://localhost:3000 cargo run --bin room

(`--bin room`: the crate has four binaries, and `cargo run` will not pick one.)
The Discord application must list `http://localhost:3000/auth/discord/callback`
as a redirect (Developer Portal → OAuth2 → Redirects; the port is the room's).
Schedule a question over the pipeline channel (SPEC §8.3), then open
`http://localhost:3000/host?question=<id>` and press *Sign in with Discord*.
Discord asks once for `identify` and `guilds.members.read`, and sends you back
to the host page signed in; press *Create a room*. The room asks Discord then,
and only then, whether your account holds the role. The host phone moves to
`/host/{room_id}` and shows the wordmark, the room code, one primary action and
the count. Open the wall at `/wall/{room_id}`, the same id as the host page's
path. Phones (or other tabs) join at `/join`, or through the short link
`/{code}`. The binary refuses to start if any of the four is unset or empty.

The binary's environment (`src/config.rs`):

| Variable | Unset | Set |
|---|---|---|
| `PORT` | bind `127.0.0.1:3000` | bind `0.0.0.0:<PORT>` (Fly) |
| `POPQUIZ_PUBLIC_URL` | `http://127.0.0.1:<port>` | the base of every join link and resume link, e.g. `https://rustnyc-popquiz.fly.dev` |
| `DISCORD_CLIENT_ID` | a startup error | the Discord application's id (digits) |
| `DISCORD_CLIENT_SECRET` | a startup error | the application's OAuth2 secret; never printed, logged or sent anywhere but Discord's token endpoint |
| `DISCORD_GUILD_ID` | a startup error | the Rust NYC server's id (digits) |
| `DISCORD_ROLE_ID` | a startup error | the id of the role that may host the default club, `nyc` (digits) — an id, never a name (AC-65) |
| `POPQUIZ_CLUBS` | none (only `nyc`) | more clubs on this server (D-26): comma-separated `slug=roleid:Zone`, e.g. `la=123456789012345678:America/Los_Angeles`. Zones: `America/New_York`, `America/Chicago`, `America/Denver`, `America/Los_Angeles`. A bad entry, a repeated slug or a repeated role is a startup error |

The OAuth redirect URI is not a variable: it is `POPQUIZ_PUBLIC_URL` +
`/auth/discord/callback`, and the Discord application must register exactly it.

The released wall's take-it-home link is SPEC §13's `https://popquiz.rustnyc.org/last`
in every configuration, the same one the static fallback uses.

## What is here

`router()` serves the room: create a room, one route per host action, and the
public state query for the wall, the buzzer and the host. The modules are the
ones named in `BUILDPLAN.md` section 2:

| Module | What it does | Ticket |
|---|---|---|
| `phase` | The machine: `idle → live → closed → split → work → reveal → released`, host actions only, no skipping, `trace_step` | T-04a |
| `answers` | **The sealed one.** Correct option, receipt, explanation, the resolving step | T-04a |
| `question` | The public half of a question | T-04a |
| `rooms` | The room record's shape and its phase-driven writes; the seams for sessions and the socket | T-04a (shape), T-04b (sessions) |
| `sessions` | Participant sessions and the answer store: join, capacity, the upsert, `leave` | T-04b |
| `view` | The public state query: wall, buzzer and host payloads | T-04a (data), T-05/06/07 (pages) |
| `copy` | SPEC §11's strings, mirrored from `web/shared/copy.js` | T-04a, T-22 (lint) |
| `auth` | The `HostAuth` seam: may this bearer create a room (the one create check, G-9), may it host this one | T-04a (seam), T-10 (async, the refusals) |
| `discord` | Discord sign-in, the organizer records (§3.5), the role-ID check at creation, token rotation, bounded retries | T-10 |
| `ws` | One broadcast per room: wall, buzzers, host | T-04c |
| `config` | What the binary reads from its environment | T-09 |
| `lifecycle` | How long a room lives: the `Clock` seam, the four-hour bound, *closed for inactivity*, the sweep's reaper | T-11 |
| `used` | What outlives a room: the used-question ledger (written at release only) and the take-it-home snapshot | T-11 |
| `admin` | SPEC §8.3's pipeline channel: `PUT /admin/questions/{id}`, `GET /admin/used`, one constant-time bearer check | T-25 |

`router()` authorizes nobody (`DenyAll`), so it can create no room. The binary
serves `lib.rs::serving_state`: *Create a room* and *Run it again* authorized by
Discord, and nothing scheduled — questions arrive over `PUT
/admin/questions/{id}` (T-25). Handing questions to
`AppState::new` is the tests' fixtures, nothing else (*Pipeline channel* below).

**Hosting (T-10, SPEC §8).** Sign-in is Discord OAuth2 with the scopes
`identify guilds.members.read`. The callback stores the organizer record —
`discord_user_id`, `access_token`, `refresh_token`, `token_expires_at`, nothing
else (§3.5) — and hands the browser an opaque **organizer session** in the host
page's URL fragment. Discord's tokens never leave `discord.rs`. On *Create a
room* (and *Run it again*) the room refreshes the access token if it is due,
then asks `GET /users/@me/guilds/{guild}/member` with the organizer's bearer.
The organizer hosts a club iff `roles` contains *that club's* role ID (`DISCORD_ROLE_ID` for `nyc`, `POPQUIZ_CLUBS` for the rest; D-26, AC-103), compared as a
string. The member record is deserialized into `roles` alone, so `permissions`,
role names and ownership cannot grant anything (AC-65). A `404` is *wrong
server* — a non-member and a member of another guild hear the same thing — and
a member without the role is *wrong role* (AC-70): `403 {refusal, reason}`.
Discord not answering is `503` with no body, a server error and never a denial.
Nothing asks Discord after creation: host commands use the room's own host
session, so an open room runs to release with Discord down, bounded at 4 h
(AC-69).

- **Rotation (AC-66).** Every refresh writes the rotated pair into the record in
  the same step that drops the spent one. Refreshes are serialized per organizer,
  and one that finds the record already rotated does not refresh again, so two
  creates at once never replay a spent token. If Discord refuses the refresh
  token anyway (`invalid_grant`), the organizer is signed out — the page offers
  *Sign in* again — and their open rooms carry on. The check runs as its own
  task, so a request dropped mid-refresh cannot lose a rotated token.
- **Retries (§8).** Per Discord call: at most 3 tries, each bounded at 3 s, with
  250 ms doubling backoff or Discord's `Retry-After`; the whole check is bounded
  at 8 s. Only no answer, `429` and `5xx` are retried — a denial is an answer,
  and Discord's 10,000-invalid-requests ban is IP-wide. A `Retry-After` longer
  than the time left ends the check instead of waiting it out.
- **State.** The sign-in's `state` is bound twice: a one-time record in the
  backend (10 minutes) and an `HttpOnly; SameSite=Lax` cookie scoped to
  `/auth/discord`. A wrong, replayed, expired or cookieless state is `400` with
  no body, and Discord is never asked.
- **Where the records live.** In memory, in the Discord backend the `AppState`
  owns beside the rooms (§9). Sessions last 12 hours. A restart forgets every
  organizer, as it forgets every room.
- **Logs.** One line per Discord call — which call, its status, the try — and
  never a token, code, state, header or body.

The proof is `tests/auth.rs`, against an in-process Discord
(`tests/common/discord_mock.rs`); its module doc maps each test to AC-64…AC-70.
It includes a structural scan: `src/`, `Cargo.toml`, `Dockerfile` and
`.github/workflows/` name none of the stand-in's identifiers.

*History:* until T-10, SPEC §8.2's M1 stand-in — one shared secret behind a
Cargo feature, and q3 seeded with it — authorized *Create a room* for HC-0.
T-10 deleted it whole.

### Clubs (D-26)

One server, one Discord guild, one question bank, a host role per club
(`DISCORD_ROLE_ID` is `nyc`; `POPQUIZ_CLUBS` adds the rest). The organizer names
the club when they create a room: `/host?question=q3&club=la`, through sign-in
and back. The create check tests *that* club's role ID; a role from another
club, an unknown club and a malformed one are all `403 wrong_role` (AC-103).

- **Never twice is per club.** `used` is keyed `(club, question)`; NYC having run
  `q3` does not retire it for LA (AC-105). Two clubs may hold a room on one
  question at once (AC-104); one live room per `(club, question)` still holds.
- **Take it home is per club:** `/last/{club}`, and `/last` is `nyc`'s. A club
  that has released nothing, or one that does not exist, gets the empty page.
  A released LA wall links to `/last/la`.
- **Run it again** stays in the old room's club, whatever the request names.
- **Scheduling is per club.** What the laptop pushes is the question *arranged
  for one meetup date* (the answer slot moves with the date), so the room keeps
  it per club: `PUT /admin/clubs/{club}/questions/{id}`, and the old
  `PUT /admin/questions/{id}` is the `nyc` spelling. A club's room is built from
  that club's record only; a club with nothing scheduled is refused, never
  handed another club's arrangement (AC-23). Whether a club has run a question
  is checked when its room is made.
- A restart empties the questions *and* the ledger, for every club.

### Routes

| Route | What | Credential |
|---|---|---|
| `POST /rooms` `{question_id, club?}` | *Create a room* for a club (default `nyc`, D-26) | the organizer session → `HostAuth::authorize_create` (Discord) |
| `POST /rooms/{id}/put-on-screen`, `close-answers`, `show-split`, `walk-it`, `reveal`, `release`, `step-back`, `step-forward` | one per host action, plus `←`/`→` | the room's host session |
| `POST /rooms/{id}/run-it-again` `{question_id}` | a **new** room; this one stays released | `authorize_create`, same organizer |
| `GET /rooms/{id}/wall`, `/buzzer` | the public state query | none |
| `GET /rooms/{id}/host` | the host's projection | the room's host session |
| `GET /auth/discord?question=<id>` | T-10: `303` to Discord's consent screen; sets the state cookie | none |
| `GET /auth/discord/callback?code&state` | T-10: `303 /host?question=<id>#<organizer session>`; a bad `state` is `400`, no body | the state cookie |

A refusal is `409 {"reason": …}` in plain words; a missing or wrong credential
is `401` with no body; an unknown room is `404`. *Create a room* adds `403
{"refusal": "wrong_server" | "wrong_role", "reason": …}` and `503` (T-10). A host command on a room that
has ended but has not been swept yet is `409 {"refusal": "already_ended" |
"closed_for_inactivity", "reason": …}` (T-11). Once it is swept, it is `404`.

## Pipeline channel (T-25)

How the organizer's laptop reaches the room: SPEC §8.3, D-20, AC-101. Two
routes, one credential. T-20 builds the laptop side (`popquiz schedule` and
`popquiz sync`).

| Route | What | Answers |
|---|---|---|
| `PUT /admin/questions/{id}` (default club) · `PUT /admin/clubs/{club}/questions/{id}` | The question record, answer included, exactly the JSON `bank.py` writes. It goes through `answers::load` into the sealed module and nowhere else (AC-61). | `201 {id, scheduled: "new"}`, or `200 {id, scheduled: "replaced"}` for a re-push. `400 {reason}` if the record does not load or its `id` is not `{id}`. `409 {reason}` if the question has been run (G-10: never twice), or a room holds it. `413` for a body over 1 MiB. |
| `GET /admin/used` | The used-question ledger, `[{club, question_id, used: {meetup_date, room_id, released_at, fit}}]` (a question is used per club, D-26): `bank.py`'s `Used` per question. `fit` is `null` if no wall reported one (G-2). | `200` |

**The credential.** `Authorization: Bearer <token>`. The token is the value of
`POPQUIZ_ADMIN_TOKEN`: a Fly secret that the pipeline's local configuration
also holds, and that is never in the repository, `fly.toml` or a command line
anyone keeps. `.env.example` carries the name only. The room reads it once at
startup (`config.rs` → `admin.rs`), and **every build refuses to start without
it**, as the stand-in's token does.

**The check.** `admin::AdminToken::check` is one `subtle` `ct_eq` over the
presented bearer. A missing header, a scheme other than `Bearer` and a wrong
token take the same path, and each is `401` with an empty body and no
`WWW-Authenticate`. Every path under `/admin`, with any method, goes to one
handler (`admin::serve`) whose first act is that check. Before it passes,
nothing about the request is read, so a prober cannot tell a scheduled id from
an unknown one, or a real route from a made-up one. Past the check, an unknown
path is `404` and a wrong method is `405`. The admin routes are their own
router, with their own state. Participant, wall and host handlers hold
`AppState`, which has no path to the token. The token's type has no `Debug`,
`Display` or `Clone`, and nothing in `admin.rs` logs.

**Replacing a question.** A re-push replaces a scheduled question only while no
room record holds it. A room keeps the question it was created with until
release or expiry deletes it (§4.6), and the used ledger is keyed by id. So a
replacement under a running room would let the night's `used` record name
content that did not run. A released question is refused as *used*.

**H-11, the client's (the deployed check).** Generate the token once, into a
shell variable. Put it in the Fly secret and in your own keeping (1Password,
and the ignored `.env` T-20's pipeline will read), and print it nowhere else.
Generating it inside the `fly secrets set` command would leave no copy for the
laptop, and the pipeline could never authenticate.

    T=$(openssl rand -hex 32)
    fly secrets set POPQUIZ_ADMIN_TOKEN="$T" -a rustnyc-popquiz
    # save "$T" in 1Password now, then:
    unset T

Set the secret **before** the first deploy of a build that has this channel. A
machine without it does not start. Rotation is the same three lines with a new
value, plus one local edit where the pipeline reads it.

**The live half of AC-101** is one push from the client's laptop to the
deployed room before the first real batch is scheduled. That push is
`popquiz schedule` (`pipeline/README.md`, *Scheduling a meetup*), which arranges
the options for the meetup's date before it sends them. On a laptop without the
pipeline, `curl` can still stand in. It reads the token from 1Password, so the
token never lands in shell history. Note that it pushes the file's stored order,
which is not arranged for any date, so it is a check of the channel and not a
meetup's question:

    curl -i -X PUT \
      -H "Authorization: Bearer $(op read 'op://<vault>/<item>/<field>')" \
      --data-binary @bank/questions/q3.json \
      https://rustnyc-popquiz.fly.dev/admin/questions/q3

`201` (or `200` if q3 was already scheduled) passes. `401` means the value sent
is not the Fly secret.

**The proof.**

- `tests/admin.rs`:
  - refusals on every method × path × wrong credential;
  - the accept and replace rules;
  - the route table: the prefix served to `admin::serve` alone, the token named
    only in `admin.rs` and `config.rs`, and no participant, wall, host or auth
    route accepting it;
  - one `ct_eq`;
  - AC-61;
  - the startup requirement;
  - the **repository scan**: `just secret-scan`, also inside `just test`. It
    reads every tracked file and every untracked one not ignored, and fails on
    the name given an 8-or-more-character literal value, on a planted
    admin-token canary, and on the value this process holds for the variable.
- `tests/canary.rs`: the canary questions are *pushed*, with the planted token
  as the bearer. Every stop probes the prefix with no token, a wrong one and the
  organizer's, and every admin response is scanned for every plant. The real
  `room` binary is run with the planted token, pushed to and refused, and its
  whole stdout and stderr scanned. That is the room's only log.

## Pages

- `GET /host`, `GET /host/{room_id}` — the host phone (T-07, `web/host/`). `/host?question=<id>#<credential>` is *Create a room* (§8.2: the credential rides in the fragment and is sent as the bearer); `/host/{room_id}#<host session>` is one screen per phase, and is the resume link (AC-50).
- **Take it home (T-12):** `GET /last` (`web/home/`, embedded) — the last released question (SPEC §13), with the room's `AppState::take_home()` written into the page as JSON on every request (escaped so no prose can close the `<script>`), `null` before the first release, when the page says so; its assets at `/home/home.{js,css}`; `tests/take_home.rs`, `web/test/home.test.js`. The fixtures both suites read (`tests/fixtures/take_home.shape.json`, `web/home/fixtures/take-home.json`) are generated: `UPDATE_TAKE_HOME_FIXTURES=1 cargo test --test take_home`. The canary scans `/last` as its own surface (`Surface::TakeHome`): no plant before the canary question's release, all of them after. AC-33's layout half is by hand: `just wall-layout`, then open `/web/home/measure.html?run=1`. Last run 2026-09-28, WebKit 605.1.15 (Safari 26.2, the c11 browser): PASS, 28 renders — q3, the complete and the does-not-compile snapshots, and the complete snapshot carrying each bank question's source (q7 is the widest), at the first and last trace step, in 360 px and 375 px frames; no document wider than its frame, the code wells scrolling inside themselves.
- **The buzzer (T-06):** `GET /join` (`web/buzzer/`, embedded; `?code=` from the link, or typed), its assets at `/join/buzzer.{js,css}`, and the short link `GET /{code}` (the `join_url` shape) `303` → `/join?code=`; `tests/buzzer_page.rs`.

## Lifecycle and what outlives a room (T-11)

SPEC §4.6, AC-56, AC-69, AC-92, G-4, G-10. `src/lifecycle.rs`, `src/used.rs`,
and the release and sweep paths in `src/rooms.rs`.

**How long a room lives.** At most `ROOM_LIFETIME` (4 h) from `created_at`.
Before that, a room closes for inactivity after `IDLE_QUIET` (30 min, `idle`) or
`LATER_QUIET` (20 min, `live`…`reveal`) with no host action. A refused command
is not activity. Every entry point (`join`, `answer`, `buzzer_for`, `act`,
`run_again`) asks `Room::ended(now)` and refuses an ended room, so nothing gets
in between the bound and the sweep. **Only `AppState::sweep(now)` deletes.** It
removes the room record and its sessions whole, and remembers `{code, why}` for
`ENDED_MEMORY` (4 h) so a late join hears *already ended* / *closed for
inactivity* rather than *unknown*, and so no new room reuses the code. The
reaper (`lifecycle::spawn_reaper`, started by the router inside a tokio runtime)
sweeps every 30 s and pokes the transport for each room it deleted.

**Time comes from one clock.** `AppState::with_clock(Arc<dyn Clock>)`. The
routes read `state.now()`, and nothing in the lifecycle calls
`SystemTime::now()` but `SystemClock`. Tests use `lifecycle::ManualClock` and
never wait (`tests/common` → `Clocked`).

**Release.** The `released` transition is the one place the room's afterlife
is written. Inside `Room::act`, while the machine is still `reveal` and so still
holds the witness, it copies the take-it-home snapshot out of what `open()`
lends. It then drops what is per person: `present`/`answered_live` go, `fit`
moves into the used record, and `AppState::act`, which takes that parting under
the same lock, drops every session, appends the used record and replaces the
snapshot. What stays until the four hours are up is the phase shell (id, code,
organizer, host session, phase), so the wall can say *Let's go to the bar.* and
the host can *Run it again*. The shell also keeps the anonymous per-option
`totals`, `answered` and the §4.5 verdict, which expire with it at four hours
(AC-56, read literally: *only anonymous totals persist, and they expire with
it*). Nothing per person survives release, and nothing that outlives the room
(the used record, the snapshot) carries a count (AC-56, D-12;
`tests/lifecycle.rs` checks by inspecting `AppState`).

**Never twice (G-10).** `create_room` and `run_again` refuse a question the
ledger holds (*That question has already been run. Pick another.*). A room that
expires or goes quiet before release records nothing.

**One room per question (GAP-8).** `create_room` and `run_again` also refuse a
question another room holds (*That question is open in another room. Pick
another.*). A room holds its question while it is not released and has not
ended (`Room::holds`): releasing either of two rooms on one question would put
the answer on `/last` while the other is still before its reveal. A released
room's question is the ledger's refusal instead; a room that went quiet or
expired holds nothing, even before the sweep deletes it, since it can never
release. The checks and the insert are one critical section in `create_for`
(`questions`, then `rooms`, held until the room is in), so two creates cannot
both pass and a release cannot land between the ledger check and the insert;
`tests/one_room.rs` guards it by inspecting `create_for`'s lock order. A room
that ended stays ended once a new room was let in on its question, whatever
`now` a later action carries. `schedule`'s own hold is wider on purpose: any
room record until it is deleted.

### The seams

| Seam | For | Shape |
|---|---|---|
| `AppState::used().all() -> Vec<UsedEntry>` | T-25's `GET /admin/used`, T-20's `popquiz sync` | `[{question_id, used: {meetup_date, room_id, released_at, fit}}]`. `used` is `bank.py`'s `Used` field for field (`tests/used.rs` parses the class). `meetup_date` is `created_at`'s date in `used::MEETUP_ZONE` (America/New_York). `released_at` is RFC 3339 UTC. `fit` is the wall's last verdict, or `null` if it never reported one. |
| `AppState::take_home() -> Option<TakeHome>` | T-12's `/last` | `tests/fixtures/take_home.shape.json`. Source, colour, options with the one ✓, the whole trace, *what*, `why` (every incorrect option's `why_tempting`, letter order) and *takeaway*, the receipt, and `machine` — the verified record's detail rows for *How we know* (full `-Vv`, edition, target, flags, Miri version/configs/seeds; `null` where a legacy record holds nothing, never back-filled). No count, no most-chosen option, no number outside `trace` (D-12, AC-56). Rebuilt at every release; `None` before the first. The extra reads come from `Revealed` (`why_tempting`, `how_we_know`), lent by the vault's one `open` with the witness (T-12). |
| `AppState::with_clock`, `AppState::sweep` | T-10's `test-full` AC-69 row | `tests/lifecycle.rs::test_full_ac69_open_room_runs_to_release_with_auth_down`, on the Discord mock: `#[ignore]`d, run by `test-full`. |

## The phase machine

`phase::apply(state, command) -> Result<Applied, Refused>` is the only way a
phase changes, and it is pure: no clock, so no timer and no auto-advance. Each
phase has exactly one action that leaves it (`Phase::next_action`, which is also
the host screen's one primary action), so nothing can be skipped: seven legal
transitions (*Create a room* into `idle`, then one per phase), *Run it again*
from `released` (a new room; this one never moves again), and `←`/`→`, which
step the trace and never change the phase. `work` enters at step `0` and stops
at `M-2`; `reveal` enters at `M-1`, the step that prints, and may step the whole
trace (D-10). Every other pair is refused with a reason. `Room::act` is the one
place a room's machine is reassigned, from `apply`'s result.
`tests/phase_table.rs` checks the whole 8 × 10 table cell by cell.

## The seams for T-04b and T-04c

- **`rooms::Sessions`** — T-04b implements it over its session map.
  `close_snapshot()` is called by the close transition, once; the room freezes
  `answered` and `totals` from it and decides §4.5's middle beat then.
  `release()` is called at release.
- **`AppState::set_live_counts(room, LiveCounts { present, answered_live })`**
  — the counts' writer from outside the rooms lock. T-04b's own join, leave
  and upsert already hold that lock, so they write the same two fields through
  `Room::set_live_counts` before letting go (see *Sessions* below). `present`
  keeps moving after close; the frozen `answered` does not.
- **`Room::accepts_answers()`** — `true` only in `live`; T-04b's upsert asks it.
- **`Room::revision()`** — bumped on every change; T-04c broadcasts on it.
- **`view::wall` / `view::buzzer` / `view::host`** — the three payloads T-04c
  pushes. Each carries exactly one `phase`, read from the room's one phase value
  (AC-81).

## Sessions (T-04b)

`sessions::SessionMap` is the `rooms::Sessions` implementation, and
`AppState::new` uses it by default. A session is a `token` and an
`answer ∈ A..E | none` and nothing else (AC-57): an exhaustive pattern in
`sessions.rs` stops the crate compiling if a field is added. Sessions live in
memory beside their room and are dropped whole at release (AC-56).

| Route | What | Credential |
|---|---|---|
| `POST /join` `{code}` | a session: `201 {room_id, token, buzzer}` | none |
| `PUT /rooms/{id}/answer` `{letter}` | the answer upsert | the session token, as `Authorization: Bearer` |

**Joining (§4.1).** The code is trimmed and upper-cased, then must be six
symbols from the code alphabet. Any phase from `idle` to `reveal` admits a
join: the idle buzzer is *You're in.*, and `present` keeps counting after close.
A refusal is `{refusal, reason}`, where `refusal` is the name the buzzer
branches on and `reason` is its §11 sentence. It is `404` for `unknown` and
`409` for the rest:

| `refusal` | When |
|---|---|
| `malformed` | the code is not six alphabet symbols |
| `unknown` | no room has that code |
| `already_ended` | the room is `released`, or four hours old (T-11) |
| `full` | the room holds `capacity` sessions (200; `AppState::with_capacity`) |
| `not_yet_open` | ships with its string; no phase of the seven reaches it |
| `closed_for_inactivity` | no host action for 30 min in `idle`, 20 min in `live`…`reveal` (T-11) |

Capacity is checked before a token is drawn, so a refused join creates,
reserves and moves nothing (AC-30).

**Answering (§4.3): the response contract T-06 renders from.** The phone shows
*saving…* while a `PUT` is in flight, then exactly one of these:

| Response | Means | The buzzer shows |
|---|---|---|
| `200 {saved}` | stored; last write wins; the same letter twice is a no-op | *saved — ‹saved›* |
| `409 {reason, phase, saved}` | not `live`. `saved` is the stored answer (or `null`), restated | from `phase` and `saved`: *answers are closed · you said ‹saved›* / *you didn't answer* |
| `401`, no body | the token names no session: never joined, left, or released | re-join |
| `400 {reason}` / any other 4xx | a malformed write; nothing was stored | *couldn't save. Your last answer, ‹X›, is safe.* |
| no response, 5xx | — | the same *couldn't save…*, where ‹X› is the last `200`'s letter |

The `409` is a superset of the generic `{reason}`: a client that reads only
`reason` still works, but render from `phase` and `saved`, never from `reason`.
Every non-`200` leaves the stored answer as it was (AC-36).
`tests/sessions.rs::ac35_response_contract` holds the server to the table. The
phone half (exactly one of the three on screen) is T-06's.

**Counts.** Join, upsert and leave recompute `present` (sessions that exist) and
`answered_live` (those holding an answer) from the map. They write both through
`Room::set_live_counts` inside the same lock, so the host never sees counts that
disagree with the sessions (AC-46).

**Ghost sessions, and what `leave` means.** A session has no "connected" flag;
the record is only its token and its answer. A session whose socket dropped
therefore stays a session: it counts in `present`, holds a capacity slot, and
its answer counts in `totals`. `AppState::leave(room, token)` removes it
whole. **T-04c calls `leave` only when a socket is gone for good** (its
re-attach grace has run out), never on a mere drop, or AC-37's re-attach with
the same token would lose the answer. Keeping the ghost's slot also means a
re-attach can never push a room past capacity. While `live`,
`answered_live ≤ present` always holds.

**The seams T-04c uses:** `AppState::buzzer_for(room, token)` returns the
buzzer payload with that session's own saved answer in `yours`; it is `Denied`
when the token resolves to no session. `AppState::leave` is described above.
`yours` appears only on these per-session paths. The public
`GET /rooms/{id}/buzzer` never carries it, and it never names another session's
answer.

**Close and release.** `close_snapshot` counts each session's final answer
once. `release` empties the map, so afterwards no token resolves and a join is
`already_ended`. `tests/sessions.rs::ac52_in_process_reconciliation` checks
200 sessions in-process. It supplements AC-52, which is `burst`'s against the
deployed room, and does not discharge it.

## The transport (T-04c)

`src/ws.rs`. One broadcast per room to the wall, the buzzers and the host, and
reconnect with the same session token.

### Sockets

| Route | Credential | Close codes |
|---|---|---|
| `GET /rooms/{id}/ws/wall` | none — the public projection `GET …/wall` already serves | `4404` room gone |
| `GET /rooms/{id}/ws/buzzer` | first message `{"t":"attach","token":"<session token>"}` → `SessionTokens::resolve` | `4401`, `4408`, `4000`, `4404` |
| `GET /rooms/{id}/ws/host` | first message `{"t":"attach","token":"<host session>"}` → `HostAuth::authorize_host` | `4401`, `4408`, `4404` |

An unknown room is `404` before the upgrade. `4401` is a missing, malformed or
wrong credential and says nothing else (AC-70); `4408` is no attach message
within 10 s; `4000` is *replaced* — a newer socket attached for the same
session. The credential is a message, not a query string, because the host's
resume link keeps its session in a `#fragment` to keep it out of logs.

### Frames

Every frame is the viewer's **whole** state:

    {"t":"state","revision":N, …view::wall / view::buzzer / view::host, flattened…}

The payload's own `phase` is the frame's only `phase` (AC-81). Less `t` and
`revision`, a frame is byte-for-byte the `GET /rooms/{id}/<viewer>` projection
the canary scans; `tests/transport.rs` asserts that at every revision.

- **On attach, the current state comes first** (§4.3, AC-37), then one frame per
  new revision. The client leaves *paused* on that first frame (T-06).
- **A buzzer's attach frame, and only that one,** adds
  `"session":{"saved":<letter|null>}` — its own saved answer, so a phone that
  reconnects in `closed` can show it. That frame is parsed and re-serialized for
  that one socket; broadcasts are never personalized.
- Hosts are not keyed: several host devices may be attached at once (AC-50).
- A socket that cannot take a frame within 10 s is dropped; that bound is per
  send, and nothing is ever sent because time passed.

### One broadcast per room

Each room has one `tokio::sync::watch` channel per viewer kind, holding
pre-serialized frames. A room's three payloads are built and serialized once
per revision, in `Transport::changed`, and every subscriber of that kind gets
the same bytes. `watch`, not `broadcast`: each frame is a whole state, so a slow
phone skips superseded ones instead of lagging behind them, and no receiver can
drop a reveal the way the spike's `broadcast` could (it had to count `Lagged`).
The channel exists only while someone is subscribed.

**What triggers a push.** `rooms.rs` has no change hook, and polling is out.
`Transport::changed(room_id)` publishes only if `Room::revision()` moved, so a
poke that changed nothing sends nothing. It is called:

- by the `notify` layer after every non-GET request under `/rooms/{id}/…` — all
  host actions, and any HTTP writer whose routes are registered **above the
  `// T-04c routes` block in `routes.rs`, which must stay last** (`layer` wraps
  only the routes already registered);
- by the transport after each call into the session map;
- by `POST /join` itself: its path names no room, so `notify` cannot poke it;
- by anything else that writes a room outside an HTTP route. Such a writer
  calls `changed` itself. Today that is T-11's reaper, which pokes each room
  the sweep deleted, so its sockets close with `4404`. (A grace period before
  `AppState::leave` drops a gone socket's session is not built yet.)

### The seam for T-04b

    pub trait SessionTokens {
        fn resolve(&self, room_id: &str, token: &str) -> Option<SessionId>; // the attach
        fn saved(&self, room_id: &str, session: SessionId) -> Option<Letter>;
        fn gone(&self, room_id: &str, session: SessionId); // its current socket closed
    }

**T-04b's session map implements it**, through `AppState` (see *Wiring*).
`gone` is not called for a replaced socket. It is called with no transport
lock held, so a socket that dies at the very instant its session re-attaches
can report `gone` just after the new `resolve`; the map lets the later
`resolve` win, trivially, because `gone` changes nothing.
`tests/transport.rs` and `tests/transport_full.rs` layer `TestTokens`
(`tests/common`) to script sessions; `tests/wiring.rs` uses the real map.

### Wiring (PQ-32)

`router()` and `router_with(state)` serve the real session map: the router's
default `Transport` is `Transport::new(state, state)`, since
`impl ws::SessionTokens for AppState` is the adapter. Sessions live per room
under the rooms lock, so the adapter is the state, not a `SessionMap`.
`resolve` finds the token in that room's map (a released room's map is empty,
so nothing resolves); `saved` is that session's own answer and no other;
`gone` does nothing, because a drop keeps the session, its answer and its slot
(see *Ghost sessions* above). A `SessionId` is a keyed hash of the token under
a per-map `RandomState`, so it carries nothing of the credential; `join`
redraws a token whose id collides. A test that wants another map layers its
own `Extension(Transport::new(state, map))` over `router_with`, and the
`provide` layer keeps it; `NoTokens` remains for such tests.

### `TCP_NODELAY`

`ws::serve(listener, router)` sets it on every accepted socket (the spike's
finding: without it a 40 ms delayed-ACK mode reads as the server being slow).
The tests and `main.rs` serve through it.

### For T-21's `burst`

The reveal broadcast is the path AC-41 measures; its p95 is `burst`'s (*Burst*,
below), and is not claimed by these tests. The mapping `burst` made from the
spike's frames:

| Spike | Room |
|---|---|
| `GET /ws`, one socket for everything | `GET /rooms/{id}/ws/buzzer`, then `{"t":"attach","token"}` |
| `{"t":"reveal","reveal_id":n,…}` | `{"t":"state","phase":"reveal","revision":n,…}` — key on `phase` and use `revision` as the id |
| `{"t":"hello",…}` | the attach frame (`t:"state"`); no instance field yet |
| padded 2048-byte reveal | the real reveal payload |
| `control` frames | the HTTP host routes with the host bearer |

### Tests

- `tests/transport.rs` (in `test`): wall + host + three buzzers through every
  transition — one frame per revision each, one phase across all (AC-81); drop
  and re-attach — current state first, saved answer intact (AC-37);
  replacement; refusals; the attach deadline; teardown.
- `tests/transport_full.rs` (`#[ignore]`d; `just test-transport-full`, inside
  `test-full`): the wall and 200 buzzers agree at every transition, and twenty
  drop in `closed` and resume in `split` (AC-81 and AC-37 as written).
- `tests/wiring.rs` (in `test`): the default router and the real map — join,
  attach with the returned token, answer, transitions, drop, re-attach with the
  same token, release; one frame per revision to the wall, the host and the
  buzzer, one phase across all, the counts moving on join and answer and not
  on a drop (AC-37, AC-46, AC-81).

## Pages

The three surfaces are pages the room serves beside its JSON. Each is plain
HTML, CSS and JS from `web/<surface>/`, embedded at compile time
(`include_str!`) in its ticket's own block of `routes.rs` — no new crate, no
`ServeDir`, nothing read from disk at run time, and a name not listed is `404`.

| Path | What | Ticket |
|---|---|---|
| `GET /wall/{room_id}` | the wall; `404` for a room that does not exist | T-05 |
| `GET /wall/wall.js`, `/wall/wall.css`, `/wall/qr.js` | the wall's own files | T-05 |
| `GET /shared/{file}` | `web/shared/*.css` and `*.js`, as `text/css` / `text/javascript` | T-05 |
| `GET /shared/fonts/{file}` | the vendored fonts `fonts.css` loads (`font/ttf`, D-14) | T-05 |
| `PUT /rooms/{id}/fit` `{fit}` | the wall's measured verdict → the host's fit line; `204`, `400` for anything but the four verdicts. **No credential**: the wall is `fit`'s writer (§3.4) and has none. A stranger with the 128-bit room id can at most change the host's fit line | T-05 |
| *(none)* | the static fallback file and its host sheet (`popquiz.fallback`, SPEC §12, D-24) hold the answer and live on the organizer's laptop; no route serves either, nor its driver `web/wall/fallback/static.js` — `tests/no_fallback_route.rs` | T-26 |

**The wall page is static.** It is the same bytes in every phase: it carries
no room state and draws every phase from the wall socket's frames
(`/rooms/{id}/ws/wall`, no attach message), each frame the whole state, so a
reload shows what was on the screen. A page that never changes cannot carry an
answer before `reveal`; `tests/wall_page.rs` drives a planted room through
every phase and holds the page to that.

**The rendering is a pure function of a frame** — `PopQuiz.Wall.html(frame,
{fontPx, fit})` in `web/wall/wall.js`, no DOM, no socket, no clock. The socket
and the type model's measure-and-refit live in `mount()`/`boot()` around it.
T-26's static fallback (`mode: "static"`, SPEC §12) drives the same function
from a local state object, so the fallback is this design and never a second
one.

**Fixtures.** `web/wall/fixtures/q3-phases.json` is `view::wall` for q3 in every
phase and trace step, generated by `tests/wall_page.rs` and held equal to what
the room serves; `web/test/wall.test.js` renders from it. Regenerate after a
payload change with `UPDATE_WALL_FIXTURES=1 cargo test --test wall_page`.

**The measured layout (AC-100, AC-33)** needs a real layout engine, and there
is no headless browser on the build machine or in CI, so it is not inside
`test-full`: `just wall-layout` serves the repo and names the page to open —
`web/wall/measure.html?run=1` — which runs the real wall code over every bank
question in every phase that renders source and reports, per row, the derived
and refitted size, the passes, the text box, the overflow and the verdict.
Last run, 2026-09-26, c11 browser (WKWebView), default room 15 / 8.44 / 20 ft,
floor 14.22 px:

| Question | Reading (live · closed · split) | Trace (work · reveal) | Verdict |
|---|---|---|---|
| q3 · 5 × 42 | 22.1 px, 0 passes | 23.8 → 22.2 px, 1 pass | fits, overflow 0 |
| q4 · 6 × 56 | 18.4 px, 0 passes | 19.8 → 18.8 px, 1 pass | fits, overflow 0 |
| q7 · 5 × 69 | 22.1 px, 0 passes | 22.9 px, 0 passes | fits, overflow 0 |
| q8 · 6 × 30 | 18.4 px, 0 passes | 19.8 → 18.8 px, 1 pass | fits, overflow 0 |

Page overflow was 0 in both directions in every row. The reading layout's text
box measures 994 × 177 in `live`, SPEC §5.2's number; the trace layout's
measures 994 × 182 (§5.2 says 190 — the prototype's well overflowed its box by
8 px into a block that overlapped it), which is what the one refit pass
absorbs. The clip path, same run: a built 16 × 36 source (a shape, not a
question) renders at the 14.22 px floor, measures 178 px of overflow at the
bottom, reports `clipped_y`, and draws the 4 px red bottom edge, with the page
still not scrolling. The sources q4, q7 and q8 are run under q3's payloads — the type
model reads only the source — because only q3 has a trace today.

**The static fallback (AC-102, T-26)**, same reason for being outside
`test-full`: `python -m popquiz.fallback q3 --out <dir>/q3.html` writes the file
and `q3.host-sheet.txt` beside it; open the file in a browser and step it with
`Space`, `←` `→` and `Esc`. The file carries a Content-Security-Policy of
`default-src 'none'` and `connect-src 'none'`, so the browser itself refuses any
request, fetch and WebSocket included. Last run, 2026-09-26, c11 browser
(WKWebView), `file://`, q3. c11 drives keys and reads state by injecting script,
which the file's CSP refuses, so the run used a harness copy whose only difference
is `'unsafe-eval'` added to `script-src`; every network directive was unchanged.
Results:

- `Space` from `idle` walked idle → live → closed → split → work (Step 1 of 6)
  → reveal (Step 6 of 6) → released, and did nothing past released.
- `→` in work stopped at Step 5 of 6 (M-2); `←`/`→` in reveal covered 6 → 1
  and stopped at both ends. Arrows in every other phase did nothing.
- `Esc` from released walked back to idle, entering reveal at 6 of 6 and work
  at 1 of 6. At idle it did nothing.
- Syntax colour: 13 spans in live, closed and split, 0 in work and reveal.
- ✓ and receipt: none before reveal. In reveal there were 6 ✓ (the bar, the
  option chip and the 4 receipt lines).
- Strip: the key legend in idle, *answers are closed* in closed, empty elsewhere.
- Page overflow 0 × 0 in every view.
- Network: `performance.getEntriesByType('resource')` was 0 and no
  `securitypolicyviolation` fired. The fonts rendered from data URIs.

Opening the unmodified file showed the idle view with the legend. That c11 could
not script it is itself the CSP working. Stepping it on a real keyboard with the
network off is HC-0's *felt* half, and it is the client's.

## The sealed module, and how the proof works

`answers::load` reads a bank record and splits it on the spot. The public half
(`question::PublicQuestion`) holds option texts in arrival order, the source,
the hint, and trace steps `0..M-2` with any `stdout` row removed. Everything
that joins an option to the verified output goes into a vault. That covers the
correct option (derived the way `bank.correct_index` derives it), the receipt
(the twin of `receipt.receipt_lines`, tested against the same
`bank/fixtures/receipts/*.json`), the beats, every `why_tempting`, and the full
trace.

The vault is a private module nested inside `answers`, and its fields are
private to it. Rust field privacy is per module, so not even the rest of
`answers.rs` can read them. Its one read, `open`, demands a
`phase::RevealWitness`. `Machine::revealed()` mints that witness, only in
`reveal`, and nothing else can build one: its field is private, and `unsafe` is
forbidden crate-wide. The witness borrows the machine, and `Room::open()`
borrows the room, so nothing opened in `reveal` survives a phase change. The
three projections branch on `room.open()`, not on the phase. The pre-reveal
builders receive a `PublicView`, which has no path to the vault.

The proof is in two halves, and neither is "we reviewed it":

1. **`compile_fail` doctests**, in `src/phase.rs`, `src/answers.rs` and
   `src/rooms.rs`, cover six attempts to cross the boundary:
   - forge a witness;
   - forge a machine in `reveal`;
   - read a room's question;
   - read the vault;
   - open the vault from a `PublicView`;
   - hold what was opened across `act`.

   Each is paired with an ordinary doctest, its twin, which compiles the same
   paths and differs only in the forbidden line. So a rename or typo breaks the
   twin loudly rather than letting the `compile_fail` pass for the wrong reason.
   They carry no error codes: on stable, `compile_fail,E0xxx` silently stops
   running.
2. **`tests/boundary.rs`** is the in-crate half that a doctest cannot see. It
   reads the source and asserts:
   - the witness is built in one place;
   - the vault holds only `seal`, `judge` and `open`, and every read takes the
     witness;
   - nothing sealed derives or implements `Debug`, `Clone`, `Copy` or
     `Serialize`;
   - nothing outside `answers.rs` touches the vault;
   - `Room::act` alone reassigns the machine;
   - the pre-reveal builders take only a `PublicView`.

Both halves were mutation-checked when they were written: opening each boundary
in a scratch copy turned exactly the matching test red.

`tests/canary.rs` is the runtime complement (*Canary*, below). It drives a room
through every phase. Before `reveal`, it asserts that no payload for any viewer carries:
- a ✓, a receipt line, the explanation or any `why_tempting`;
- the resolving step, a `stdout` values entry, or a step beyond `M-2`;
- the hint (before `live`, and outside the buzzer's `live` payload).

It also asserts that every option object is exactly `{letter, text}` in arrival
order, and that the buzzer never carries source, trace or option text. At
`reveal` it asserts that the plants do appear. T-08's canary set, pages,
frames and the real buzzer page are described under *Canary*, below.

### Choices worth knowing

- **Option order.** The room never arranges options: the pushed record's order
  is the wall's order. The date-drawn arrangement (AC-23) belongs to the
  pipeline's push (T-20). `bank/questions/q3.json` is in bank order, which is
  fine for tests but is not a record to put in front of a room.
- **The correct-option rule mirrors `bank.correct_index` exactly.** It matches
  by kind for a does-not-compile record, and otherwise by the option equal to
  `stdout` less one trailing newline. If the rule changes, both twins change
  together.
- **Copy.** `src/copy.rs` has one constant per `copy.js` key, with the same
  name upper-cased. `tests/twins.rs` compares the two in both directions, so
  T-22's lint has matching keys to compare. Edit SPEC §11 first, then
  `copy.js`, then `copy.rs`.

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

## Canary

The secrecy suite (T-08): `tests/canary.rs` in `just test`, `tests/canary_full.rs`
in `just test-full`, both over one scan core in `tests/canary_scan/` (not a test
target). `just canary` runs the first; `just canary --full` both.

**What is planted.** `common::plants()` draws one set per test process, each
`CANARY-<NAME>-<16 hex>` from the OS generator, so no line of the room can have
been written to match one. `canary_question()` (id `canary`) is q3's shape with a
plant in: option E's text **and** the synthetic verified record's `stdout` (that
equality is the join G-3 withholds, so it is what is planted); the receipt
(`runs.count`, rendered `✓ Ran ‹N› times`); `explains.what` and `takeaway`; every
`why_tempting`; the hint; the source (a trailing comment on line 1); every trace
step's `note` (the final one is the resolving step's); a non-`stdout` trace
value; a `stdout` row in a middle step. `canary_question_dnc()` is the same prose
with one canary error code (receipt `✓ Error ‹code›`). Both records say
`SYNTHETIC - no compiler ran` and their source is itself a plant: nothing here
writes down what a program prints. `POPQUIZ_ADMIN_TOKEN` is set to a fifth-kind
plant in the test process and every child; it is never a literal in the
repository, which is what T-25's repo-wide scan looks for.

**What is scanned, at every revision** — every host transition, every `→` in
`work` up to the bound and the one refused past it, every `←` in `reveal` back to
step 0 — with a wall, a host and three buzzers attached over the loopback
listener and the real session map:

- the three HTTP projections, and the host denials (`401`, empty);
- every response a client provokes: the host action's, `POST /join` (a new
  session each stop; the refusal in `released`), `PUT …/answer`, `PUT …/fit`,
  an unknown room's `404`, *Run it again*'s;
- every socket frame each socket received since the last stop, drained until
  it equals its viewer's projection;
- every page and every file the router serves (`/wall/{id}`, `/host`,
  `/host/{id}`, `/join`, `/join?code=`, `/{code}`'s redirect, every asset list
  in `routes.rs`; fonts once per room and byte-compared at every stop);
- the wall's frames as the served `wall.js` renders them (`PQ.Wall.html`, under
  `node`);
- the **real buzzer page**: the served `buzzer.js` mounted under `node`
  (`canary_scan/page.js`) with node's own `fetch` and `WebSocket` pointed at the
  room. It joins by the code, attaches, takes the hint, answers, sees its socket
  drop in `closed` and re-attaches, and its rendered HTML, the frames it received
  and every request and message it sent are scanned.

**The rules** (`canary_scan::check`, keyed on the phase each payload names):
before `reveal`, nothing on any surface carries the resolving note, the
explanation, a `why_tempting`, the receipt or the error code, the middle `stdout`
row, a ✓, a receipt line, a key that names the answer, a `stdout` values row or
a step beyond `M-2`, and the correct option's text appears only as option E on
the wall (G-3, AC-47, AC-60); every option object is exactly `{letter, text}`;
the hint is in every buzzer view in `live` and on no surface outside it (AC-48),
and taking it on the page makes no request, sends no message and moves neither
the wall nor the host; no participant surface ever carries the source, a trace
note or value, an option text, the explanation or the receipt (G-8, AC-32); the
`work` wall is uncoloured (AC-97); no served or rendered wall has an interactive
control (AC-79 — `wall_page.rs` holds the served page's half too); after the
page's `closed` frame it sends nothing but `{t:"attach", token}` and the server
receives nothing bearing its session (AC-58, G-4), and its *n people said X*
equals the broadcast total for the letter it holds; `released` carries no plant;
no surface carries the admin token. Positive controls then check the plants did
reach the surfaces that may show them — the source on the live wall, the hint on
the live buzzer and the page, the walk's notes in `work`, the resolving step,
every note and the middle `stdout` row while stepping `reveal`, the explanation
and the named option's `why_tempting` on the host, the receipt on the wall — so a
scan that saw nothing cannot pass. `the_rules_catch_a_planted_leak` checks the
rules themselves against payloads the room must never send.

**The route walk.** `canary_scan::route_table()` reads every `.route(` in
`src/routes.rs`; a route that is neither driven nor on `UNSCANNED` fails
`every_route_in_routes_rs_is_scanned_or_listed`. **Unscanned: none.**

**test-full.** `canary_full.rs` runs the same walk with every HTTP request over a
real TCP connection, over two rooms (the second made by *Run it again*, on the
does-not-compile twin), and the reconnect path: in `closed` a buzzer's socket
drops and re-attaches, another's is replaced by a second socket (`4000`), and
each attach frame carries only that session's own saved letter.

**What it does not prove, yet.**

- *The deployed room, with the canary set.* The scan core takes a base URL and
  `just canary --url U` is the hook, but it still refuses. T-09's stand-in can
  now create a room on the deployed server, but only on the question it seeds,
  q3. Nothing can plant the canary question there before T-25's admin push
  (§8.3). Until then `just smoke <url>` carries the deployed-room scan with q3's
  own secrets as the plants (*Deploying*, below).
- *The admin token.* The room reads no `POPQUIZ_ADMIN_TOKEN` and has no admin
  route until T-25, so today that assertion can only fail if a payload echoed
  the environment. It is a scaffold T-25 turns live. The room writes no log
  line yet either (its only output is the bind line), so "no log line" is
  T-25's to check alongside the code that first reads the token.
- *A browser.* The pages run under `node` with a stub DOM, not a layout engine;
  what a real browser adds is layout, which the canary does not judge.
- `exit_code` renders nowhere (the receipt never reads it) and cannot carry a
  string; it is asserted absent as a key rather than planted as text.

## Deploying (T-09)

The room runs on Fly.io as the app **`rustnyc-popquiz`**, at
`https://rustnyc-popquiz.fly.dev` (BUILDPLAN D-A). The image is the repo root's
`Dockerfile`: a release build on a `debian:bookworm-slim` runtime. The config
is the root's `fly.toml`, written by hand. `.dockerignore` is an allowlist: the
crate and `web/`.

**From GitHub Actions.** `.github/workflows/deploy.yml` runs `flyctl deploy
--remote-only --ha=false` on every push to main, and when started by hand
(Actions, *deploy*, Run workflow).
It needs one repository secret, `FLY_API_TOKEN`, from
`fly tokens create deploy -a rustnyc-popquiz`. This route works where a cloud
session's does not, since the runner has no proxy in front of the builder.

**flyctl in cloud sessions.** `.claude/hooks/install-flyctl.sh` (a `SessionStart`
hook, remote sessions only) installs flyctl into `~/.fly` and puts it on `PATH`,
so `fly deploy --ha=false --remote-only` works from a Claude Code cloud session.
It needs `fly.io` and `github.com` (release assets) in the environment's allowed
domains, and `api.fly.io` and `rustnyc-popquiz.fly.dev` for the deploy itself.
If the network blocks the install, the hook prints one line saying so and the
session starts normally. The hook installs the binary only: `FLY_API_TOKEN` is a
separate environment secret (a deploy token for `rustnyc-popquiz`), never
committed. The environment's setup script is the alternative place for the same
install line.

**One machine, never two.** The room is one state in memory (SPEC §9). A second
machine would be a second room that phones could land in. `fly deploy` adds a
second machine "for high availability" whatever `fly.toml` says, so every
deploy takes `--ha=false`. Never run `fly launch`: it rewrites `fly.toml` and
strips its comments.

    fly apps create rustnyc-popquiz                      # once
    fly deploy --ha=false --remote-only                  # from the repo root

**Discord (T-10).** The four values are Fly secrets, set by the client (H-3).
They are never in the repository, in `fly.toml`, or on a command line anyone
keeps; `fly secrets set` reads them from your shell:

    fly secrets set -a rustnyc-popquiz DISCORD_CLIENT_ID=… DISCORD_CLIENT_SECRET=… DISCORD_GUILD_ID=… DISCORD_ROLE_ID=…

The Discord application must list the deployed redirect, exactly:

    https://rustnyc-popquiz.fly.dev/auth/discord/callback

(and `https://popquiz.rustnyc.org/auth/discord/callback` once DNS points there).
A deploy without the four does not start: the binary refuses, by design. The
host URL is `https://rustnyc-popquiz.fly.dev/host?question=<id>`; it holds no
secret, and every organizer signs in with their own Discord account.

**The pipeline's secret.** `POPQUIZ_ADMIN_TOKEN` is the other Fly secret, and
every build needs it (*Pipeline channel*, H-11). Set it before deploying.

**Smoke uses up q3.** `just smoke <url>` runs a whole segment on q3, which must
be scheduled first (SPEC §8.3), and release writes
q3's `used` record into the machine's in-memory ledger (G-10, T-11). That
machine then refuses to create a room on q3 until it restarts. After every
smoke run against the deployed room, run `fly apps restart rustnyc-popquiz`
before anyone hosts on it. The same restart (or `fly machine restart <id>`)
resets q3 for another HC-0 drive after a real one. The ledger is in memory until
T-20's `popquiz sync` pulls it (through T-25's `GET /admin/used`), so a restart
before a sync loses the night's record.

**The trial org stops the machine.** The Fly org has no payment method, so Fly
stops every machine after about five minutes. Rooms live in memory, so a
stopped machine has lost every room it held. `auto_start_machines` starts a
fresh one (with no rooms and q3 unused) on the next request. HC-0 therefore
runs inside one such window: open the host URL, create the room, and walk it
through within five minutes. Otherwise, add a card at https://fly.io/trial. No
code works around this.

**DNS, later (H-2).** When `popquiz.rustnyc.org` points at the app, run
`fly certs add popquiz.rustnyc.org -a rustnyc-popquiz` and set
`POPQUIZ_PUBLIC_URL = "https://popquiz.rustnyc.org"` in `fly.toml`, then deploy.
Nothing else changes.

**Smoke.** `just smoke <url> [--participants N] [--question ID] [--out PATH]`
(`src/bin/smoke.rs`, behind the `smoke` feature) reads `POPQUIZ_ORGANIZER_SESSION` from the environment: the
organizer session a signed-in host page holds — sign in at
`<url>/host?question=q3`, and it is what follows the `#` in the address the
page lands on. It lasts 12 hours and is one person's; every room it creates
passes Discord's live check. It checks that *Create a room* is `401` with no
body for no bearer and for a wrong one, then creates the room with the
session. It checks that the join link is
`<url>/<code>` and that `GET /<code>` is `303 → /join?code=` (AC-28). Half the
participants join with the code as the link carries it, and half with it typed
lower-case with spaces. Then it drives all seven phases through the host routes
with a wall, a host and N buzzer sockets attached, awaiting every phase on
every socket. Along the way it checks these things:

- the answer writes, with changed minds and a deadline burst of every final
  answer inside 2 s, each timed on an already-open connection;
- a write after close is refused with the saved answer restated;
- the split's totals equal the final answers exactly on every buzzer;
- `work` stops at M-2;
- the reveal's arrival on every buzzer is timed;
- a join after release is `already_ended`.

Every frame and JSON response from `idle` to `work` is scanned for q3's secrets:
both `explains` beats, every `why_tempting`, the resolving note, a ✓, and a key
named `correct`. At `reveal`, a positive control checks that they do arrive. The
image holds the room binary and nothing else, so a green smoke run also shows a
room created and run to release with no pipeline present (AC-1's runtime half).
Its timings are smoke's, not `burst`'s AC-41/53/54 figures (*Burst*, below).
`--question` names the id q3's record was scheduled under (default `q3`; the
deployed workflow uses `smoke-q3`, so a run never releases a bank question), and
`--out` also writes the result as JSON. `just test-full` runs it against an
in-process room on loopback (`tests/harness_full.rs`).

## Burst (T-21)

`just burst <url> [--participants N] [--question ID] [--out PATH]`
(`src/bin/burst.rs`, behind the `burst` feature; `--help` lists the rest) puts
200 participants on the room's own protocol, through the client smoke uses
(`src/bin/room_client.rs`): joins by code, a buzzer socket each, answers as
`PUT /rooms/{id}/answer` on kept-alive connections, phases through the host
routes. One room, walked to release:

| Criterion | Measure | Pass |
|---|---|---|
| AC-54 | isolated deadline bursts — every participant writes once inside 2 s, nothing else in flight, one or more per shape (`uniform`; `spike`, the window's last 50 ms); the worst cycle's p95 | p95 < 500 ms |
| AC-53 | every write while `live`: the isolated bursts, a churn of changed minds, the final deadline burst | p95 < 500 ms |
| AC-41 | the reveal, from the host's `POST reveal` to each buzzer's first `reveal` frame, on one clock | p95 ≤ 2 s |
| AC-52 | every write saved as sent; the host's live count one per session; after close, each session's refused write restates **its own** final answer (a swap between two sessions fails this though every total survives it); the split's totals are the finals exactly | exact |

The report is JSON on stdout (and `--out`), every raw sample in it. The rules
are the spike's (*How it tries not to lie*, below), carried over in
`src/bin/burst_report.rs`, which `tests/burst_report.rs` checks inside `just
test` on recorded samples. Write latency starts on an already open connection,
and the client's own scheduling delay is measured beside it, never added. A pass
whose interval crosses its threshold is `MARGINAL` in words. Exit `0` pass, `1`
a criterion missed, `2` invalid or could not run: an invalid report says
`"quotable": false` and judges nothing. The spike's "more than one Fly machine"
check is gone — the room exposes no machine id, and `fly.toml` runs one; a
machine lost mid-run shows as `404`s and is reported as the room gone.

**Where its numbers count.** Every report says `"substrate": "loopback"` or
`"deployed"`. `just test-full` runs burst at 200 and smoke at 50 against a room
the job itself starts — the real router on `127.0.0.1`, authorized by the tests'
`HostAuth` (`tests/harness_full.rs`) — on every PR: the harness, AC-52's exact
count, AC-54's window and AC-41's fan-out are proven there; the latencies are
the runner's loopback and are printed as such. **AC-53's "run against the
deployed substrate" is met only by a deployed run**, one of the two below,
which a person starts: nothing unattended can create a room on the deployed app.

**Scheduling.** A question runs once per machine, and burst and smoke each run
one room to release, so each gets q3's record under a harness id — `burst-q3`,
`smoke-q3` — which no bank question has, so neither run can retire one. Over
the pipeline channel (*Pipeline channel*, above), the token read from 1Password
and handed to curl on stdin, never on its command line:

    for id in burst-q3 smoke-q3; do
      jq --arg id "$id" '.id = $id' bank/questions/q3.json > "/tmp/$id.json"
      printf 'header = "Authorization: Bearer %s"\n' "$(op read 'op://<vault>/<item>/<field>')" |
        curl -sS --config - -X PUT --data-binary "@/tmp/$id.json" \
          https://rustnyc-popquiz.fly.dev/admin/questions/$id
    done

**From the laptop** (the pre-checkpoint path), right after scheduling, inside
one trial window (*The trial org stops the machine*, above):

    export POPQUIZ_ORGANIZER_SESSION=…   # what follows the # after signing in at /host
    just burst https://rustnyc-popquiz.fly.dev --question burst-q3 --out burst.json
    just smoke https://rustnyc-popquiz.fly.dev --question smoke-q3 --participants 200
    unset POPQUIZ_ORGANIZER_SESSION

**From CI** (`.github/workflows/deployed-burst.yml`, `workflow_dispatch` only):

    gh secret set POPQUIZ_ORGANIZER_SESSION     # pasted at the prompt, never typed on the line
    gh secret set POPQUIZ_ADMIN_TOKEN
    gh workflow run deployed-burst -f url=https://rustnyc-popquiz.fly.dev -f participants=200

It builds both clients first, then schedules the two ids, runs burst (one
isolated burst per shape, a 10 s churn, to stay inside the trial window) and
smoke back to back, and uploads both JSON reports as an artifact; the job
summary lists each criterion. There is no Fly token in CI.

**Afterwards, every time:** `fly apps restart rustnyc-popquiz` — before anyone
hosts and **before any `popquiz sync`**, because the machine's ledger holds the
harness ids until it restarts — then `gh secret delete POPQUIZ_ORGANIZER_SESSION`
and `gh secret delete POPQUIZ_ADMIN_TOKEN`. Record the four criteria on the
ticket, never a token.

## After a meetup (T-21, AC-55)

AC-55 — failed participant requests under 0.1% across a session on venue wifi
— is read from the room's own log (`src/requestlog.rs`). Each is one JSON line
on standard error, which Fly keeps:

- `{"event":"participant_request_failed","room":…,"route":…,"kind":…,…}` — a
  `5xx` on `join`, `answer` or `buzzer_state` (`kind: "server_error"`,
  `status`), or a buzzer socket that broke while its room was running
  (`kind: "socket_dropped"`, `phase`);
- `{"event":"participant_requests","room":…,"failed":F,"total":T,"ended":…}` —
  once per room, at release (or when a watched room is swept, `ended: "gone"`).

The night's rate, the morning after (before anything restarts the machine):

    fly logs -a rustnyc-popquiz --no-tail | grep '"event":"participant_requests"'
    fly logs -a rustnyc-popquiz --no-tail | grep '"event":"participant_request_failed"'

The rate is `failed / total`; AC-55 passes under 0.001. `total` counts
successful joins, answer writes, state reads and buzzer sockets attached;
refusals the room gave correctly (`409` closed or ended, `404`, `401`, `400`),
a socket closed with a close frame (a page leaving or reloading sends one) and
a re-attach are counted but never failed. A socket that breaks without one is
failed, whether the network dropped it or the phone was locked or its tab
killed — the room cannot tell those apart, so read `socket_dropped` lines as
an upper bound on the network's share. Nothing after a room's release counts.

**It is a floor.** A request venue wifi lost before it reached the room is in
no log. A burst of `401`s or a machine restart during the meetup (every phone
stranded at once) does not show as failures here; look at the surrounding
`fly logs` lines for it. A machine stopped mid-meetup writes no summary at all.
A buzzer socket's failures are its drops only: its upgrade is `101` or `404`,
never a `5xx`.

## The burst spike (T-03)

The one real engineering risk in this project is the deadline write burst:
200 people answering inside the two seconds before the host closes answers.
`BUILDPLAN.md` §5 retires it at T-03, **before any room code is written**, and §3
is explicit about why — if p95 misses, D-A option 2 (Cloudflare Durable Objects)
re-opens and the substrate decision is taken again. So this spike exists to be
allowed to fail, and everything below is built so that a failure is impossible
to miss and a pass is impossible to overstate.

Two throwaway binaries, both behind the default-off `spike` feature:

| Bin | What it is |
|---|---|
| `spike-server` | One room, in memory, `idle → live → closed` plus a reveal broadcast. Not the phase machine — T-04a writes that on a blank page. |
| `spike-burst` | The load client: 200 WebSockets, timing, a JSON report, a non-zero exit on a miss. T-21 renamed it (it was `burst`) when the room got its own; it speaks only the spike server's protocol. |

### Running it

```sh
# Both bins are feature-gated, so the inner loop never builds them (see below).
cd room
cargo test  --features spike          # the hermetic tests: estimator, verdict, reconciliation
cargo build --release --features spike --bin spike-server --bin spike-burst

# Against a loopback server — proves the harness, never a headline number.
PORT=8099 ./target/release/spike-server &
./target/release/spike-burst --url http://127.0.0.1:8099 --clients 12 --cycles 2 \
  --churn-secs 3 --reveals 2 --window-ms 500

# Against the deployed machine — this is where the real numbers come from.
./target/release/spike-burst --url https://rustnyc-popquiz-spike.fly.dev \
  --clients 200 --seed 20260920 --out spike/reports/$(date -u +%F).json
```

`--help` lists every flag. Exit codes carry meaning and are worth knowing:

| Exit | Means |
|---|---|
| `0` | all four criteria pass as measured |
| `1` | **a criterion missed** — the server did not meet the threshold |
| `2` | **the run is invalid; do not quote its numbers** — fewer than 200 clients connected, an ack timed out, a reveal receipt never arrived, more than one Fly machine answered, `ulimit -n` under 1024, or the laptop's own send lag exceeded 25 ms |

Separating 1 from 2 is the point. Without it, a broken harness reads as a failing
server and a failing server can be excused as a broken harness. The report says
the same thing in its own fields: an invalid run carries `"quotable": false` and
each criterion's `pass` is `null`, not judged. A pass whose confidence interval
crosses its threshold still exits 0, but the verdict notes call it `MARGINAL` in
words and drop the clean-pass sentence.

`cargo test --features spike` includes four loopback tests that serve the real
spike server in-process on `127.0.0.1:0` and drive whole segments through it:
last-write-wins, frozen totals, post-close refusal, every reveal received, exact
reconciliation, the join past capacity refused `full`, a deliberately invalid run
left unjudged, and the report surviving a round trip. They prove the harness. They
never produce a headline number.

### Where the numbers come from

**The headline figures for AC-41, AC-53 and AC-54, and AC-52 at 200 sessions,
come from a run against the deployed Fly machine, from a laptop.** No loopback
number is ever quoted as any of them — `EVALUATION.md`'s AC-53 row says it
outright: *run against the deployed substrate, not a local mock.* Committed runs
live in `spike/reports/`.

The client-side network in those runs is the laptop's. **Venue wifi is AC-55's
oracle and it is settled at HC-4 in October, not here**; the report records a
failure rate but labels it as the laptop's link.

### How it tries not to lie

A load test that flatters its subject is worse than none, so the specific ways
this one could have been wrong are each defended against, and the defenses are
visible in the report rather than asserted here:

- **The write clock** starts on an already-open, already-handshaken connection
  and stops when that socket reads the matching `ack` — one clock, one task. TLS
  and connection setup land in `diagnostics.connect_ms` and can never leak into a
  write latency.
- **The ack means the write is in the authoritative map**, not that a frame
  arrived: the server drops the room lock and then acks.
- **`send_lag_ms`** — the laptop's own scheduling delay — is measured, reported,
  and never added to write latency. 200 real phones do not queue behind each
  other. Above 25 ms at p95 the run is invalid, because at that point the laptop
  is what was measured.
- **Both burst shapes run, and the worst drives AC-54.** `uniform` spreads the
  200 writes over the window; `spike` puts all of them in its last 50 ms. Both
  satisfy `SPEC.md` §9's *200 writes inside 2 s*, but AC-54 calls the burst *the
  highest-risk moment in the system*, and letting only the gentler shape reach
  the exit code would mean the harder one could never affect the go/no-go.
- **`p95` travels with its own uncertainty.** At n=200 it is the 190th sample —
  one slow client moves it. The report prints the rank, its 95% interval (ranks
  183–197, rounded outward), and the max; a pass whose upper bound crosses the
  threshold is flagged `marginal` rather than rounded into a clean pass.
- **Every raw sample ships** in `samples_ms`, so any percentile can be recomputed
  by someone who does not trust this code.
- **AC-52 is checked four ways** — per-letter totals, `answered`, the applied
  `seq` sum, and an XOR fingerprint over `(session, letter, seq)`. The first
  three are aggregates and two sessions swapping answers would slip past all of
  them; the fingerprint is what makes it a per-session claim. The server still
  exposes no per-session answer.
- **`TCP_NODELAY` on both ends.** axum does not set it (`tap_io` is the seam);
  without it a 40 ms delayed-ACK mode appears in the histogram and reads as the
  server being slow.
- **The reveal frame is padded to 2048 bytes**, because a fan-out measured on a
  60-byte frame is a measurement that lies by being easy.
- **`auto_stop_machines` is off** in `fly.spike.toml`, and every `hello` frame
  carries `FLY_MACHINE_ID`. If more than one machine ever answered, room state
  would be split across two maps and AC-52 would be reconciling against a
  fiction — so the run is invalidated rather than reported.
- **The report is written before the exit code is computed**, so a failing run
  still leaves a complete artifact behind.

### Deploying the spike

`fly.spike.toml` and `spike/Dockerfile` describe a **throwaway** app,
`rustnyc-popquiz-spike` in `ewr` on the smallest shared-cpu machine. This is not
the room's deploy; that is *Deploying*, above (`fly.toml` and `Dockerfile` at
the repository root).

The single most important line in `fly.spike.toml` is
`http_service.concurrency.hard_limit`. Fly's default is **25 connections**; at
200 WebSockets the proxy would shed connections before the server was under any
strain, and the spike would report a p95 miss belonging to the config rather
than to Rust on Fly — a false no-go on the substrate decision.

```sh
cd room
fly launch --no-deploy --copy-config --config fly.spike.toml \
  --name rustnyc-popquiz-spike --region ewr --dockerfile spike/Dockerfile
fly deploy --remote-only --config fly.spike.toml --app rustnyc-popquiz-spike \
  --dockerfile spike/Dockerfile --ignorefile spike/.dockerignore .
fly machine stop <ID> -a rustnyc-popquiz-spike   # leave it scaled to zero
```

`fly launch` may drop a `fly.toml`, `Dockerfile` or `.dockerignore` into `room/`.
None of those belong to this ticket; check `git status` afterwards and delete
them.

### What became of it

T-21 carried this client's measurement onto the room's own routes (*Burst*,
above) and kept the spike's two bins, renamed and frozen, so the reports in
`spike/reports/` stay reproducible. Nothing in `test` builds them (they are
behind `required-features = ["spike"]`, which is what holds its 60 s budget);
`just test-full` type-checks and unit-tests them (`harness-full`).
