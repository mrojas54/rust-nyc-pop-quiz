# PQ-13: Discord OAuth and role check

BUILDPLAN.md T-10 (M2).

Discord OAuth: scopes, member lookup, role-**ID** check at room creation, the Administrator test, refresh-token rotation persisted, bounded retries, denial wording; **deletes the M1 stand-in** (the `dev-host-token` feature) and adds the test that a build without it accepts no stand-in token (`SPEC.md` §8.2)

Criteria: AC-64–70, G-9
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a–b
BUILDPLAN notes: Human track: app id, secret, guild id, role id. The live check against the client's account is this ticket's exit criterion and gates HC-1

Orchestrator notes: Human track H-3. The live check against the client's account is the exit criterion and gates HC-1, never HC-0.

HELD: needs H-3 the Discord app: id, secret, guild id, role id. Do not dispatch until the client releases it.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-27)

Branch `ai-c11-cc/discord-auth` off `origin/main` @ f8f9caf. PQ-17 (admin channel) merges first; I then `git merge origin/main` (never rebase after first push) and re-run `just test`. PQ-17 touches `config.rs`, `lib.rs` (`serving_router`), `main.rs`, `rooms.rs` (`schedule`), `tests/standin.rs` and `tests/smoke_config.rs`: expect conflicts in those; resolve by keeping both blocks.

## 1. The seam (auth.rs) — G-9, AC-70

`HostAuth::authorize_create` is synchronous today and Discord is network I/O. It becomes the one async create check:

```rust
pub type CreateCheck<'a> = Pin<Box<dyn Future<Output = Result<OrganizerId, CreateRefusal>> + Send + 'a>>;
pub trait HostAuth: Send + Sync + 'static {
    fn authorize_create<'a>(&'a self, bearer: Option<&'a str>) -> CreateCheck<'a>;   // the one create check
    fn authorize_host(&self, room: &Room, bearer: Option<&str>) -> Result<(), Denied>; // unchanged (host session)
}
pub enum CreateRefusal { Denied, WrongServer, WrongRole, Unavailable }
pub fn decided(r: Result<OrganizerId, CreateRefusal>) -> CreateCheck<'static>  // for backends that decide without I/O
```

- `Denied` (unit struct) stays exactly as is for `authorize_host`: no reason.
- `CreateRefusal::Denied` = no/unknown/expired organizer session, or Discord refused the organizer's tokens → **401, no body** (the host page shows *Sign in* again).
- `WrongServer` = member lookup 404 (not in the guild, or the guild is not theirs — the same answer, so membership never leaks, AC-70) → **403 `{refusal:"wrong_server", reason}`**.
- `WrongRole` = member without the role ID → **403 `{refusal:"wrong_role", reason}`**.
- `Unavailable` = Discord did not answer within the retry budget, or answered something that is not a verdict (5xx after retries, 400/403 on the member route) → **503, no body** — the plain server error, never a denial.
- `DenyAll` keeps existing (router() default, tests) and answers `decided(Err(Denied))`.

## 2. AppState (rooms.rs) — additive only

- `RoomError` gains `NotHost(CreateRefusal-subset: WrongServer|WrongRole)` and `Unavailable` (new variants; `ws.rs` matches with a wildcard, `routes::failure` is mine).
- New `pub async fn create_room_checked(&self, bearer, question_id, now)` and `pub async fn run_again_checked(...)`: await `auth.authorize_create`, then the existing `create_for` / the run-again body. The routes call these.
- The existing sync `create_room` / `run_again` keep their signatures (used by `tests/used.rs`, `transport.rs`, `common::Clocked` — files I may not rewrite); they poll the checked future once (`futures_util::FutureExt::now_or_never`) and map a pending check to `RoomError::Unavailable`. Documented: the synchronous door serves backends that decide without I/O (the tests' `TestAuth`); the Discord backend only answers through the async one. `run_again` body is factored into one private fn so the two doors share it.
- `AppState::with_discord(Arc<discord::Discord>)` (builder, additive field `discord: Option<Arc<Discord>>`) so the T-10 routes reach the backend's sign-in half. The organizer store lives **inside the Discord backend, which AppState owns** — i.e. in the same in-memory authoritative state as the rooms (§9), behind its own mutex: only the backend reads or writes it, and no room lock is ever held while Discord is awaited. A restart loses every organizer session (and every room): the organizer signs in again.

## 3. The Discord backend (new `room/src/discord.rs`)

- `Settings { client_id, client_secret: Secret, guild_id, role_id, redirect_uri }` built by config; no `Debug`/`Display`/`Clone` on `Secret`.
- `Endpoints { api: "https://discord.com/api/v10", authorize: "https://discord.com/oauth2/authorize" }`; tests pass the in-process mock's `http://127.0.0.1:<port>` base. No env var for it.
- `Retry { attempts: 3, first_backoff: 250 ms (×2), per_attempt: 3 s, deadline: 8 s }` — per Discord call bounded attempts, exponential backoff, `Retry-After` (header seconds, float) honoured when it fits the remaining deadline and otherwise ends the call; one deadline for the whole create check. Retried: transport error, timeout, 429, 5xx. Never retried: 2xx, 401, 404, other 4xx (a denial is an answer, and the 10,000-invalid-requests ban is IP-wide). Tests pass millisecond timings.
- **Organizer store (§3.5):** `Organizer { discord_user_id, access_token, refresh_token, token_expires_at }`, keyed by `discord_user_id`; nothing else. Sessions: `session -> (discord_user_id, issued_at)`, opaque 32-byte hex from `getrandom`, matched in constant time (`subtle`), lifetime 12 h (session TTL; a new sign-in mints a new session, older ones stay valid until TTL). Pending sign-ins: `state -> (question, created_at)`, one-time, 10 min TTL, capped at 256 (oldest evicted).
- **`authorize_create`**: resolve session → organizer (none → `Denied`). The work runs in a `tokio::spawn`ed task the check awaits, so a browser that disconnects mid-refresh cannot drop Discord's rotated token on the floor. Per-organizer async mutex around refresh: under it, if `token_expires_at <= now + 60 s`, refresh (`POST /oauth2/token grant_type=refresh_token`); the rotated `{access, refresh, expires_at}` is written into the record **before** the old pair is released (single replace under the store lock; AC-66). A second concurrent create waits, sees the record changed, and does not refresh. `invalid_grant` (a replayed/stale refresh token) → the organizer record and its sessions are dropped and the answer is `Denied` (401 → the page offers *Sign in* again: detected, not silent; open rooms are untouched because host actions never consult Discord). Then `GET /users/@me/guilds/{guild}/member` with the bearer: 200 → host iff `roles` (array of strings) contains `DISCORD_ROLE_ID` by string equality — `permissions`, role names and ownership are never read (the response is deserialized into a struct with only `roles`); 404 → `WrongServer`; 401 → one refresh then one retry, then `Denied`; anything else after retries → `Unavailable`. Ok → `OrganizerId(discord_user_id)`.
- **Sign-in half** (called by the routes): `begin(question) -> (state, authorize_url)`; `finish(state, code) -> session`: exchange code (`grant_type=authorization_code`, client id+secret in the form body, `redirect_uri`), `GET /users/@me` for the id (scope `identify`), store the organizer, mint the session.
- HTTP: a minimal HTTP/1.1 client, one request per connection (`Connection: close`), Content-Length / chunked / to-EOF bodies, over `tokio::net::TcpStream` and, for https, `tokio-rustls` with `webpki-roots` and rustls' `ring` provider passed explicitly (no global install). Shape adapted from `bin/smoke.rs`. **No new crate:** `rustls`, `tokio-rustls`, `webpki-roots` move from optional to ordinary dependencies — all three already in `Cargo.lock`, so no fetch and the lock's content is unchanged except nothing (they are listed already). Form encoding and URL encoding are ten lines each, written here.
- **Logging:** the backend logs through a `Log` seam (default `eprintln!`, one line per Discord call: endpoint name, status, attempt — never a token, header, body, code or state). Tests install a capturing sink and scan it.

## 4. Routes (`// T-10 routes` block in routes.rs)

- `GET /auth/discord?question=<id>` → `303` to Discord's authorize URL: `response_type=code`, `client_id`, `scope=identify guilds.members.read`, `redirect_uri={POPQUIZ_PUBLIC_URL}/auth/discord/callback`, `state`. **State binding: both** — the one-time server record *and* a `pq_oauth` cookie holding the state (`HttpOnly; SameSite=Lax; Path=/auth/discord; Max-Age=600; Secure` when the public URL is https). `question` must be `[A-Za-z0-9_-]{1,64}` else `400`, no body.
- `GET /auth/discord/callback?code&state` → the cookie must equal `state` and the record must exist (consumed); otherwise `400` with no body (wrong, replayed, or cookieless — nothing said). `error=…` from Discord with a valid state → `303 /host?question=<id>` (the sign-in button again). Exchange failure → `503`, no body. Success → `303 /host?question=<id>#<session>`, cookie cleared, `Cache-Control: no-store`, `Referrer-Policy: no-referrer`.
- `POST /rooms` and `run-it-again` call the checked (async) AppState doors. `failure()` maps the new variants (403 with `{refusal, reason}` from the new copy keys; 503 no body).
- The client registers `http://localhost:3000/auth/discord/callback` (local) and `https://<deployed host>/auth/discord/callback` in the Discord application.

## 5. Config (config.rs) — additive Discord block

`DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET`, `DISCORD_GUILD_ID`, `DISCORD_ROLE_ID`: all required and non-empty; the three ids must be decimal snowflakes. `ConfigError::Discord { var, why }` names the variable and never echoes a value. `Config` stays non-`Debug`; holds `discord: discord::Settings`. Redirect URI derived from the public base. The `HOST_DEV_TOKEN` field, variant and `#[cfg]`s go.

## 6. The stand-in, deleted (§8.2)

`room/src/standin.rs` (and `hc0_question()`, the q3 seed), the `dev-host-token` feature, every `#[cfg(feature = "dev-host-token")]`, `main.rs`'s `dev-host-token on/off` log, `lib.rs`'s two-arm `serving_state` → one arm: `AppState::new(discord.clone(), Vec::new(), urls).with_discord(discord)` — nothing scheduled; PQ-17's `PUT /admin/questions/{id}` is the only way in. CI: the stand-in step becomes the smoke client's own step (`--features smoke --test smoke_config --bin smoke`, `--test standin` gone). Dockerfile: `--features dev-host-token` and its comment go. `tests/standin.rs` → `tests/auth.rs`.

## 7. `smoke` — side taken

`smoke` no longer creates rooms with a shared secret. It reads `POPQUIZ_ORGANIZER_SESSION` from the environment (never argv): the organizer session a signed-in human's host page holds in its fragment after Discord sign-in. Create-with-no-bearer and create-with-a-wrong-bearer still assert `401`, no body; create with the session runs the live Discord check. q3 must be scheduled first (PQ-17's channel); a `409 No question is scheduled` says so. No new shared secret creates rooms.

## 8. Host page (web/host)

- `/host?question=<id>` with no fragment → the first screen shows **Sign in** (an `<a>` to `/auth/discord?question=<id>`, styled as the primary button) instead of *Create a room*.
- With a fragment → *Create a room* as today (the fragment is the organizer session, a bearer on `POST /rooms`). `401` → the session is dropped from the page and *Sign in* shows again. `403` → the server's `reason` (the new denial copy). `503` → the status line.
- `renderCreate(ui, signedIn)` stays pure; header comment and `parseLocation` docs updated. Tests in `web/test/host.test.js`.

## 9. Copy (three keys, client's wording, pending)

`host_action_sign_in`, `host_denied_wrong_server`, `host_denied_wrong_role` in `copy.js` + `copy.rs` + (twins walks them automatically), in a new COPY_ROWS row `"Host, sign-in and denials (F-33)"` (the completeness test demands a row). Proposed, avoiding the Forbidden row (`\bwrong\b` is forbidden in every copy string, so the denials name the condition without that word) and §11.1: **"Sign in with Discord"** · **"That Discord account isn't in the Rust NYC server."** · **"That Discord account doesn't have the organizer role."** Posted under a c11 flag; they land only on the client's word.

## 10. Tests (all in `just test` unless marked)

`room/tests/auth.rs` (new; replaces `tests/standin.rs`), against an in-process Discord mock (axum on loopback: token endpoint with rotation + replay detection, `/users/@me`, member endpoint; switchable to 429+Retry-After, 5xx, down; hit counters):

- AC-64: `ac64_member_with_the_role_creates_a_room`; `ac64_member_without_the_role_is_denied`; `ac64_no_stand_in_survives_in_src_cargo_dockerfile_or_ci` (structural scan of `room/src`, `room/Cargo.toml`, `Dockerfile`, `.github/workflows/`); `ac64_any_bearer_that_is_not_an_organizer_session_is_401` (none, empty, garbage, a 64-hex token, a host session, a Discord access token).
- AC-65: `ac65_an_administrator_without_the_role_id_is_denied` (permissions = all bits, owner); `ac65_a_role_with_the_host_roles_name_but_another_id_is_denied`; `ac65_the_member_record_is_read_for_roles_only` (static: discord.rs's member struct has one field, `roles`).
- AC-66: `ac66_an_expired_token_is_refreshed_and_the_rotation_persisted` (mock asserts each refresh presents the latest-issued token; two successive refreshes); `ac66_concurrent_creates_refresh_once`; `ac66_a_replayed_refresh_token_signs_the_organizer_out_but_not_their_open_room` (401 → sign-in; the open room keeps taking host actions).
- AC-67: `ac67_no_participant_route_touches_the_auth_module` (static, route-table/source scan of the participant blocks and `web/buzzer`); `ac67_no_participant_request_carries_a_credential` (a full room with participants; every participant request scanned for the organizer session and Discord tokens).
- AC-68: `ac68_organizer_b_cannot_read_or_control_as_room`; `ac68_an_organizer_session_is_not_a_host_session`; `ac68_organizer_b_cannot_run_it_again_on_as_room`.
- AC-69 (`test`): `ac69_create_with_discord_down_is_a_plain_server_error`; `ac69_host_actions_never_call_discord` (mock hit count flat across all seven phases). **`test-full`:** `tests/lifecycle.rs::test_full_ac69_open_room_runs_to_release_with_auth_down` completed (tokio test on the Discord backend + ManualClock; mock down after creation; runs to release; create refused `Unavailable`; dies at 4 h).
- AC-70: `ac70_non_member_and_wrong_guild_get_the_same_wrong_server` (byte-identical responses); `ac70_member_without_role_gets_wrong_role`.
- Retries: `retry_after_is_honoured_and_attempts_are_bounded`; `server_errors_back_off_then_fail_plain`; `a_denial_is_never_retried`; `the_deadline_bounds_the_whole_check`.
- Flow: `sign_in_redirects_with_both_scopes_and_the_public_redirect_uri`; `callback_lands_on_the_host_page_with_a_session`; `a_wrong_replayed_or_cookieless_state_is_refused_with_nothing_said`; `discord_error_returns_to_sign_in`.
- Secrecy: `secrecy_no_discord_value_or_token_in_any_payload_frame_page_or_log` (the four `DISCORD_*` values and issued access/refresh tokens as runtime-random canaries; every response, socket frame, page and captured log line in every phase scanned) + `secrecy_positive_control_the_scan_bites`.
- Config: `config_requires_the_four_discord_variables`; `config_errors_never_echo_a_value`.
- `web/test/host.test.js`: sign-in screen without a fragment; create screen with one.

## 11. Docs

`room/README.md`: `auth` row rewritten, `discord` row added, `standin` row → one-line history note; *Running it* with the four variables and `http://localhost:3000/auth/discord/callback`; *Deploying* with `fly secrets set DISCORD_CLIENT_ID=… …` (names only) and the deployed redirect URI to register. Root `README.md` *Running it* likewise. `.env.example`: append the four names (write, no read).

## Open choices made
1. Async seam + sync compatibility doors (§2) rather than rewriting other tickets' tests.
2. Organizer store inside the Discord backend owned by AppState (§2).
3. State bound by cookie *and* one-time server record (§4).
4. Session TTL 12 h; pending sign-in TTL 10 min, cap 256.
5. Retry: 3 attempts, 250 ms×2, 3 s per attempt, 8 s total.
6. No new crate; rustls/tokio-rustls/webpki-roots de-optionalized (in the lock).
7. smoke takes an organizer session from the environment (§7).
8. A third Discord call, `GET /users/@me`, at sign-in (the id for the organizer record; the member route 404s for a non-member so it cannot supply it).

## Contract tensions (sides taken)
- **Copy vs Forbidden row:** AC-70 says denials name *wrong server/wrong role*; §11's Forbidden row bans `\bwrong\b` in every copy string. Side: the strings name the condition without the word (proposed above); keys keep the names the brief gave. Routed with F-33.
- **COPY_ROWS "authored in §11 first":** the completeness test requires every key to belong to a row; the new row is labelled F-33 pending §11 — Orchestrator's ruling.
- **Files outside my clearance I need:** `room/tests/smoke_config.rs` (its `config()` helper must supply the four now-required variables, as PQ-17 did for its token) and the `justfile` `test-full` line (it does not run `tests/lifecycle.rs`'s ignored tests, so the AC-69 row would never run). Asked by comment before touching.
- `fly.toml` line 22 and `.env.example` still name `HOST_DEV_TOKEN` (not mine to edit / append-only); outside the scan's scope; noted for the Orchestrator.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: sonnet subagent, 2026-09-27. Six findings.

1. **Critical — `tests/common/mod.rs::TestAuth` not listed as updated.** Concern: the trait change breaks every test crate. Resolution: explicit step — `TestAuth::authorize_create` answers `decided(...)` with the same three cases; `Denied` → `CreateRefusal::Denied`. `common/mod.rs` is cleared (additive); this one signature change is forced by the seam and is recorded under deviations.
2. **Major — de-optionalizing rustls/tokio-rustls/webpki-roots vs the 60 s budget.** Concern: `just test` now compiles a TLS stack. Resolution: the room binary itself must speak HTTPS to Discord, so the TLS client cannot sit behind a feature that `cargo test` omits without leaving the shipped path untested. The budget is **warm** (EVALUATION): the TLS crates compile once and are cached. I measure warm `just test-room` before (origin/main) and after and report both numbers; if the warm time regresses past 60 s I revisit.
3. **Major — organizer tokens need the `Secret` wrapper.** Resolution: already so — `Organizer.access_token`/`refresh_token`, session tokens and pending states are all `discord::Secret` (no `Debug`/`Display`/`Clone`); `Organizer`/`Session`/`Pending`/`Store` derive nothing that formats.
4. **Minor — session store uncapped.** Resolution: capped at 1024 (oldest evicted) plus the 12 h TTL purge at each sign-in; pending sign-ins capped at 256 with a 10 min TTL.
5. **Minor — `create_room`/`run_again` bodies rewritten in rooms.rs.** Resolution: the rewrite is the seam change's minimum (the sync doors keep their signatures and delegate; `run_again_for` is the old body moved, unchanged). Comment posted to the Orchestrator per the brief; merge with PQ-17's `schedule` is disjoint (different functions).
6. **Minor/forward — smoke's organizer session vs T-21's automated smoke.** Resolution: recorded under deviations for T-21: an automated deployed smoke needs a fresh organizer session (12 h) from a human sign-in, or T-21 chooses another way to create the room (e.g. drive a room a human created via its host resume fragment). Not T-10's to decide.
