# PQ-17: Admin channel for the pipeline

BUILDPLAN.md T-25 (M2).

The pipeline channel, server side: `PUT /admin/questions/{id}` into the sealed `answers` module and `GET /admin/used`, one constant-time bearer check on `POPQUIZ_ADMIN_TOKEN`, a route-table test that nothing else is served under the admin prefix and no other route reads the token (`SPEC.md` §8.3, D-20); extends `canary` with `POPQUIZ_ADMIN_TOKEN` as a plant and adds the repo-wide scan for the token to `test` (`EVALUATION.md` AC-101)

Criteria: AC-101, AC-61, G-9
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-04a, T-08, T-11
BUILDPLAN notes: Human track: H-11, for the deployed check only. Adds the name `POPQUIZ_ADMIN_TOKEN`, never a value, to `.env.example`. Serialized on `justfile`

Orchestrator notes: Touches `room/src/phase.rs` (F-10) and `.env.example` (F-5, edit made outside the sandbox by the Orchestrator). Serialized on `justfile`.

Workflow mode: inline-full. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-27)

Base: origin/main @ f8f9caf (PQ-14 + PQ-37 merged). Branch ai-c11-cc/admin-channel.

## Shape

- **`room/src/admin.rs` (new).** `pub const VAR: &str = "POPQUIZ_ADMIN_TOKEN"` — the one place in `src/` the name is a code literal (config.rs refers to it as `admin::VAR`).
  - `AdminToken { token: Box<[u8]> }`: no `Debug`/`Display`/`Clone`. `from_var(Option<String>) -> Result<AdminToken, ConfigError>`, which rejects a missing, empty or whitespace-only value. `unguessable()` draws 32 OS-random bytes, hex-encoded and held in memory only. `router()`/`router_with()` use it, so the default router still runs the same single check, but no one can know the token (the admin equivalent of `DenyAll`).
  - A **concrete struct, not a trait.** Choice: G-9 asks for one function per path, and this path has exactly one backend. A `HostAuth`-style trait would invite a second one.
  - `fn check(&self, headers) -> Result<(), Denied>`, **the one constant-time compare**: `subtle::ConstantTimeEq`. A missing bearer compares `b""`, so missing and wrong take the same code path and get the same response.
  - `Admin { token: AdminToken, rooms: Arc<AppState> }` is the admin sub-router's own state. Participant, wall and host handlers extract `State<Arc<AppState>>`, which has no path to the token. That is structural, not a convention.
  - **One entry point**, `serve(State<Arc<Admin>>, Request) -> Response`. Its first statement is `check`. On refusal it returns `401` with an empty body and no `WWW-Authenticate`, before any dispatch, so the path, method and body are never looked at. After the check it dispatches on (method, path):
    - `PUT /admin/questions/{id}`
    - `GET /admin/used`
    - anything else under the prefix: `404`, or `405` for a known path with the wrong method, both after auth.
  - The body is read only after the check (`axum::body::to_bytes`, 1 MiB limit → `413`).
  - Nothing in the module logs. The crate has no logger, and `admin.rs` gets no `eprintln!`/`println!`, which a test asserts.
- **`PUT /admin/questions/{id}`:**
  - The body goes through `answers::load(&str)`, so the answer enters `Scheduled` (the sealed module) and nowhere else (AC-61). `admin.rs` never names `Revealed`, the witness, `verified` or `correct`; a test asserts that.
  - Checks, in order:
    - body not UTF-8, or `LoadError` → `400 {reason}`, where `reason` is the loader's message and names only the id and structural facts;
    - `{id}` ≠ the record's `id` → `400 {reason}`;
    - `AppState::schedule`:
      - question already used → `409 {reason}` (G-10: never twice);
      - question held by any room record → `409 {reason}`;
      - otherwise: newly scheduled → `201 {id, scheduled:"new"}`, replacing an unheld one → `200 {id, scheduled:"replaced"}`.
  - The response carries the id and nothing else from the record.
  - **Replacement rule (§4.6 decision):** a question may be replaced only while **no room record holds it**. A room takes its `Arc<Scheduled>` at creation and keeps it until release or expiry deletes the record (§4.6). The used ledger (G-10) is keyed by id. So replacing an id while a room runs it would let the night's `used` record name content that did not run. Released rooms are already refused as *used*, and a room that is swept (deleted) holds nothing. Replacing an unheld, unused question is how the organizer re-pushes after an edit.
- **`GET /admin/used`:** `200` with `AppState::used().all()` serialized as `[{question_id, used:{meetup_date, room_id, released_at, fit}}]`. That is the README *seams* shape, and `used` matches `bank.py`'s `Used` fields.
- **`room/src/rooms.rs`, additive seam only** (commented to the Orchestrator first):
  - `pub fn schedule(&self, q: Scheduled) -> Result<Scheduling, RoomError>` with `pub enum Scheduling { New, Replaced }`.
  - It locks `questions`, then `rooms`. That order is safe: nothing locks `questions` while holding `rooms` (`create_for` drops the questions guard first).
  - It checks `used.contains` and whether any room record holds the id, then inserts. No other change; seeding through `AppState::new` stays as it is (tests plus the stand-in's HC-0 seed).
- **`room/src/config.rs`:**
  - New `pub(crate) admin_token: admin::AdminToken`, **required in every build**. Missing, empty or whitespace gives `ConfigError::MissingAdminToken`, whose Display names the variable and never a value.
  - Checked **after** the stand-in's token, so `tests/standin.rs`'s `MissingHostToken` cases hold unchanged.
- **`room/src/lib.rs`:**
  - `pub mod admin;`
  - `router_with(state)` = `routes::routes(state.clone()).merge(routes::admin_routes(state, AdminToken::unguessable()))`
  - new `router_with_admin(state, token)`
  - new `serving_router(config) -> Router` (the binary's). `serving_state` is kept for the tests that use it.
- **`room/src/routes.rs`:** a `// T-25 routes` block only.
  - `pub(crate) fn admin_routes(state, token) -> Router` registers `/admin`, `/admin/` and `/admin/{*rest}`, all `any(crate::admin::serve)`, with `.with_state(Arc<Admin>)`.
  - It is merged in `lib.rs` **after** the T-04c layers. Admin routes change no room, so they are deliberately outside `notify`/`provide`; the block comment says why, which keeps the T-04c "keep this block last" rule true for room routes.
  - One more line: the module's table doc gets the admin rows.
- **`room/src/main.rs`:** `room::serving_router(config)` in place of `serving_state` plus `router_with`. **Deviation 1:** main.rs is not on the cleared list, but the token has to reach the router, and main is the binary's only wiring. It is two lines, and no sibling touches it.

## Tests, by criterion

`room/tests/admin.rs` (new):

- **AC-101 refusal:**
  - `missing_or_wrong_token_is_refused_saying_nothing`: every method (GET/PUT/POST/DELETE/PATCH) × every path under the prefix (`/admin`, `/admin/`, `/admin/used`, `/admin/questions/{scheduled id}`, `/admin/questions/{unknown id}`, `/admin/questions`, `/admin/x/y`) × {no header, wrong bearer, `Basic`, empty bearer, right token plus one byte, right token's prefix}. Each gets `401`, an empty body, no `WWW-Authenticate`, and a header set identical across cases. A scheduled id and an unknown id are indistinguishable.
- **AC-101 accept:**
  - `the_right_token_schedules_and_reads_used`: PUT q3 → 201 new; PUT again → 200 replaced; body is exactly `{id, scheduled}`. GET used → `[]`. Create a room from the pushed question (the create route, TestAuth), walk it to release, then GET used → one entry with exactly the `{question_id, used:{meetup_date, room_id, released_at, fit}}` keys.
  - `a_used_question_is_refused` (409, G-10).
  - `a_held_question_is_not_replaced` (409 while a room holds it; allowed again once the room is swept after expiry, if the question is not used).
  - `id_must_match_the_record` (400).
  - `a_record_that_does_not_load_is_refused` (400, carries the loader's reason).
  - `oversized_body_is_refused_after_auth`.
- **AC-101 route table:**
  - `the_admin_prefix_is_served_to_the_check_alone`: parse `src/routes.rs`. Every `.route(` whose path starts `/admin` is inside the T-25 block, and its handler is `any(crate::admin::serve)`. No other route's path starts `/admin`. `admin::serve`'s first statement calls `check`.
  - `no_other_module_reads_the_token`: in `room/src/**`, the name `POPQUIZ_ADMIN_TOKEN` appears only in `admin.rs` and `config.rs`; `AdminToken` is named only in `admin.rs`, `config.rs`, `lib.rs` and routes.rs's T-25 block. `auth.rs`, `standin.rs`, `ws.rs`, `view.rs`, `rooms.rs`, `sessions.rs` and `routes.rs` outside the block name neither.
  - `no_participant_wall_host_or_auth_route_accepts_the_admin_bearer`: with the admin token as bearer, `POST /rooms`, `POST /rooms/{id}/run-it-again`, every host action, `GET /rooms/{id}/host` and `PUT /rooms/{id}/answer` → 401. Every GET page and projection is byte-identical with and without that bearer.
- **G-9:** `one_constant_time_compare`: `admin.rs` holds exactly one `ct_eq(` and no `==` on the token. The admin-token path is the only admin check.
- **AC-61:** `the_pushed_answer_enters_only_the_sealed_module`: `admin.rs` names none of `Revealed`, `Witness`, `verified`, `correct` or `stdout`. A pushed planted question's correct-option plant is absent from every pre-reveal projection. The canary covers this in full; below, it is fed only by pushes.
- **Config:** `the_token_is_required` (missing, empty or whitespace → `MissingAdminToken`; Display names `POPQUIZ_ADMIN_TOKEN` and not the value) and `serving_router_uses_the_configured_token` (end to end through `serving_router`).
- **Static repo scan (AC-101):** `the_repository_holds_no_admin_token`.
  - Files: `git ls-files -co --exclude-standard` from the repo root, so tracked and to-be-added files; UTF-8 only.
  - It fails on:
    - (a) the name followed by `=` or `:` and a literal value of 8 or more characters (`$`, `<`, `‹`, `…` or `{` openers are allowed: shell substitution and placeholders);
    - (b) any `CANARY-ADMIN-TOKEN-` followed by 16 hex (the plant's shape; the pattern is assembled at run time so the test never spells it);
    - (c) the value of `POPQUIZ_ADMIN_TOKEN` in the test's own environment, if it is set and 8 or more characters. That catches the real token if the organizer runs `just test` with it exported.
  - If `git` is missing or lists nothing, it **fails** (never green by not scanning).
  - `the_repo_scan_catches_a_plant`: positive control on the matcher, with runtime-built strings.
- **Canary live plant (`room/tests/canary_scan/mod.rs`, `canary.rs`):**
  - `Server::start` builds with `router_with_admin(state, token = plants().admin)` and **schedules both canary questions by `PUT /admin/questions/{id}`** instead of seeding `AppState::new`. So every walk runs on pushed questions.
  - The walk drives GET `/admin/used` after release, plus a missing and a wrong bearer in each phase. Every admin response is scanned with a strict rule: no plant of any kind, including the admin plant.
  - `route_table()` learns `any(` (method `ANY`), and `DRIVEN` gains `ANY /admin`, `ANY /admin/` and `ANY /admin/{*rest}`.
  - **Log lines:** `the_binary_logs_no_admin_token` (canary.rs):
    - spawn `env!("CARGO_BIN_EXE_room")` with `POPQUIZ_ADMIN_TOKEN` = the plant and `PORT` = a free port;
    - drive a push (right token), a wrong-token PUT, a missing-token GET and `GET /admin/used`;
    - stop the process, then scan its whole stdout and stderr for the plant;
    - also start it once with the token empty: exit 2, stderr names the variable, not a value.
  - Positive control: a wrapper plants the token in one response, and `check()` fails (the existing `the_rules_catch_a_planted_leak` already carries the admin plant; I extend it with an admin-response case).
- **`room/tests/boundary.rs`:** an assertion that `admin.rs` is outside the sealed boundary (it does not name the witness). Assertions only.
- **Existing tests needing the now-required variable:** `tests/smoke_config.rs` (one pushed pair in `config()`, the name assembled as for `HOST_DEV_TOKEN`) and `tests/standin.rs` (the two `expect("configured")` envs). **Deviation 2:** neither file is on the cleared list; these are one-line fixture additions, forced by "required in every build". No assertion changes.

## justfile, CI, env, README

- **justfile:** a `secret-scan` recipe, `cd room && cargo test --offline --locked --test admin repository`, named in the reserved-name header as the AC-101 hook. `test` already runs it through `test-room`, and CI runs `just test` unchanged, so there is no workflow edit and no secret in the workflow.
- **.env.example:** `printf '%s\n' 'POPQUIZ_ADMIN_TOKEN=' >> .env.example` (append, no read).
- **room/README.md:** a *Pipeline channel* section. It covers:
  - the two routes, the codes, and the replace rule;
  - the token's name, where it lives, and H-11: `fly secrets set POPQUIZ_ADMIN_TOKEN="$(openssl rand -hex 32)"`, keep the same value in the pipeline's local config, rotate with `fly secrets set` plus one local edit;
  - **the room now refuses to start without it**: deploy after H-11, never before.
  - The *seeding* path is marked as the stand-in's and the tests' only.

## Contract tensions and sides taken

- **T1.** SPEC §3.1 `used.fit` vs `bank.py` `Used.fit: Fit` (non-optional), while PQ-14's `UsedRecord.fit` is `Option<Fit>`: an unobserved fit is `null`. I serialize what the ledger holds (`null` included), because G-2 says nothing may be written that no code observed. T-20's reader must accept `null`. Flagged to the Orchestrator for T-20.
- **T2.** The Orchestrator note says "touches `room/src/phase.rs` (F-10)". Nothing in this ticket needs `phase.rs`, so I don't touch it (it is a serialized file).
- **T3.** Required-at-startup means the deployed skeleton will not boot after this merges until H-11's `fly secrets set` has run. That is intended (a channel with no token is broken, not closed), but the client must set the secret **before** the next deploy. I say so in DONE and in the README.
- **T4.** EVALUATION says the canary covers "every payload, page and log line". The room has no logger; its only log line is the binary's stderr. The log scan therefore runs the real binary, not a tracing capture.

## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)

Reviewer: Sonnet subagent, 2026-09-27. No Critical findings. It confirmed:
- routing for `/admin`, `/admin/` and `/admin/{*rest}` alongside `/{code}`;
- the questions → rooms lock order;
- the scanner's placeholder exemptions.

1. **Major: how the token leaves `Config` is unstated, since `AdminToken` has no `Clone`.** Resolution: `lib.rs::serving_router(mut config)` takes the token with `std::mem::replace(&mut config.admin_token, AdminToken::unguessable())`, then hands the rest to the unchanged `serving_state(config)`. `AdminToken` stays non-`Clone`. Implemented in e6b07ee.
2. **Minor: `secret-scan` in the header that says "reserved by EVALUATION.md"; the recipe is manual and redundant with `test`.** Resolution: kept as-is. The brief says in so many words: "a `just` recipe (name it in the reserved-name header)". The header already lists non-table names (`setup`, `sandbox-build`). The enforcing path is `test` → `test-room` → `tests/admin.rs::ac101_the_repository_holds_no_admin_token`, which CI runs through `just test`. The recipe is the named hook for running it alone, and its comment says the scan also runs inside `test`.
3. **Minor: `fit` is `Option` in the ledger, non-optional in `bank.py`.** Resolution: unchanged (G-2). Flagged to the Orchestrator for T-20 in the PLAN-NOTE comment and repeated in DONE.

## Reset 2026-09-28 by agent:delegator-pq17
