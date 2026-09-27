# PQ-11: Deploy to Fly with the host stand-in

BUILDPLAN.md T-09 (M1).

Deploy: Fly app, `popquiz.rustnyc.org` (or fallback host), the short link carrying the code, `smoke`; the skeleton is built with the `dev-host-token` feature, `HOST_DEV_TOKEN` is set as a Fly secret, and the host URL is printed once (`SPEC.md` §8.2, D-19)

Criteria: AC-28, AC-64 (stand-in)
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-05–07
BUILDPLAN notes: Human track: DNS. Serialized on `justfile` (`smoke`). Adds the name `HOST_DEV_TOKEN`, never a value, to `.env.example`

Orchestrator notes: DNS may be absent: BUILDPLAN allows the Fly-provided hostname as fallback. Adds the name `HOST_DEV_TOKEN` to `.env.example`, a file the sandbox cannot read: that one-line edit is made by the Orchestrator outside the sandbox (F-5). The host URL is printed once, to the Orchestrator, never into a log or comment.

HELD: needs H-2 DNS (fallback host allowed) and a Fly deploy. Do not dispatch until the client releases it.

Workflow mode: fast-track. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator-pq11, 2026-09-26)

Base: origin/main @ 66a962c. Branch ai-c11-cc/deploy. Fast-track, no plan-review subagent.

## Files

New:
- `room/src/standin.rs`: the whole module is `#![cfg(feature = "dev-host-token")]`, and its `mod` line in lib.rs carries the same cfg. `DevHostToken` (no Debug, no Clone, no Display) implements `HostAuth::authorize_create` with a `subtle` ct_eq over bytes (subtle is already a dependency) and keeps the default `authorize_host` (the room's own host session), so it grants nothing else. `DevHostToken::from_var(Option<String>)`: missing or empty → `ConfigError::MissingHostToken`, whose message names the variable and never the value. `hc0_question()` = `answers::load(include_str!("../../bank/questions/q3.json"))`.
- `room/src/config.rs` (always compiled): `Config::from_vars(|name| …)`, a pure function. `PORT` → `0.0.0.0:<PORT>`, unset → `127.0.0.1:3000`, and a bad port is an error. `POPQUIZ_PUBLIC_URL` → `Urls.base` (http/https only, trailing `/` trimmed, no path, query or fragment), unset → `http://127.0.0.1:<port>`. `Urls.home` = SPEC §13's `/last` on the production host (`https://popquiz.rustnyc.org/last`, the same default PQ-12's fallback links to). The `HOST_DEV_TOKEN` read exists only in a `#[cfg(feature = "dev-host-token")]` line that calls `standin`. `Config::from_env()` is the one impure wrapper.
- `room/src/bin/smoke.rs`, `required-features = ["smoke"]`: drives a URL (http or https) end to end. It GETs the pages; checks that POST /rooms with no bearer and with a wrong one gives 401 and an empty body (AC-64 on the deployed host); creates the room with `HOST_DEV_TOKEN` from the environment (never argv); checks join_url == `<url>/<code>` and that `GET /<code>` 303s to `/join?code=` (AC-28). N participants join (half by the short link's code, half by the code typed lower-case with spaces) and attach buzzer sockets; a wall socket and a host socket attach too. Then all seven phases through the HTTP host routes, each phase awaited on every socket. The answer writes go over kept-alive connections and include a deadline burst (all N inside 2 s); some participants change their answer. It records write latency p50/p95/max, and after close checks the frozen totals == the final answers exactly (AC-52 shape), that a post-close write is 409, and that `work` stops at M-2 (the next step is 409). It measures reveal fan-out to every buzzer (p50/p95/max) and in `released` checks that a join is `already_ended`. A q3 secrecy scan runs over every frame and response before `reveal`: none carries q3's explains.what/takeaway, any why_tempting, the resolving note, or a ✓. The positive control is that they arrive at reveal. Exit 0 green, 1 a check failed.
- `room/tests/standin.rs`: AC-64. Always compiled: a no-feature case (`#[cfg(not(feature))]`): the binary's own state (`Config::from_vars` with HOST_DEV_TOKEN *set*, then `room::serving_state`) refuses POST /rooms for no bearer, a wrong bearer and that very token: 401 with no body. A structural scan in the `boundary.rs` shape, which runs in every build: `HOST_DEV_TOKEN` appears in no non-comment line of room/src outside standin.rs; every non-comment reference to `standin` outside standin.rs sits under a `#[cfg(feature = "dev-host-token")]` attribute; standin.rs opens with the inner cfg. Feature-on cases (`#[cfg(feature)]`): missing and empty are startup errors; a wrong or missing bearer gives 401 and no body; the right one creates a room on q3; the stand-in does not authorize host actions (a bearer equal to the token on a host route gives 401); *Run it again* goes through the same check; and the error's Display holds no token value.
- `room/tests/smoke_config.rs`: `Config::from_vars` cases (bind, public URL normalisation and rejection, defaults). AC-28 with a configured public URL: `join_url` and the wall's *join @* line carry it, `GET /{code}` 303s to `/join?code=`, and a typed join works.
- `Dockerfile` (repo root, context = root because include_str! reaches `web/` and `bank/questions/q3.json`): `rust:1.96-slim-bookworm` builder, which matches the cargo that resolved the lock, as the spike's did. It runs `cargo build --release --locked --features dev-host-token --bin room`. The runtime is `debian:bookworm-slim`, running as a non-root uid with `PORT=8080`. I'll record the size from the build.
- `.dockerignore`: an allowlist (`*` then `!room/Cargo.*`, `!room/src`, `!web`, `!bank/questions/q3.json`).
- `fly.toml`: a new app `rustnyc-popquiz`, `ewr`, env `PORT=8080` and `POPQUIZ_PUBLIC_URL=https://rustnyc-popquiz.fly.dev`, `force_https = true`. `auto_stop_machines = "off"`, `auto_start_machines = true` (after the trial's five-minute stop, the next request starts a fresh machine), `min_machines_running = 1`. Concurrency type `connections`, hard 400 and soft 350: Fly's default of 25 would shed 200 sockets (PQ-3). shared-cpu-1x 256 MB, one http check on `GET /join`. Deployed with `fly apps create` + `fly deploy --ha=false --remote-only`, never `fly launch`.

Changed:
- `room/src/main.rs`: `Config::from_env()` (an error goes to stderr and exit 2, which is the startup error when the feature is on and the token is missing), then `room::serving_state(config)`, bind, then `ws::serve` (PQ-6 dev. 4). The startup line names the bind, the public base and whether the stand-in is on, and nothing about the token.
- `room/src/lib.rs` (additive): `pub mod config;`, `#[cfg(feature)] pub mod standin;`, and `pub fn serving_state(Config) -> AppState`. Without the feature it is `DenyAll` and nothing scheduled; with it, `DevHostToken` and q3, through `AppState::new`, the same seam tests/common uses. `router()` is untouched.
- `room/src/auth.rs`: the module doc only, rewritten to the truth.
- `room/Cargo.toml`: features `dev-host-token = []` and `smoke = ["dep:tokio-tungstenite", "dep:rustls", "dep:tokio-rustls", "dep:webpki-roots"]`. The optional deps `tokio-rustls 0.26` and `webpki-roots 1` are already in the lock and the local registry; only room's own entry in Cargo.lock changes (offline). `[[bin]] smoke`, `required-features = ["smoke"]`.
- `justfile`: `smoke URL *ARGS` → `cd room && cargo run --release --offline --locked --features smoke --bin smoke -- --url URL ARGS`. `smoke:T-09` leaves PENDING (otherwise test-full would still list it as pending), and the header's stale count is made honest.
- `.github/workflows/ci.yml`: one step in the `test` job, `cd room && cargo test --offline --locked --features dev-host-token,smoke --test standin --test smoke_config --bin smoke`. `just test` already covers the no-feature build; it is the *feature* build that nothing builds. (Deviation, flagged: the brief says to add the no-feature run only if needed; the no-feature run isn't needed, but the with-feature run is.)
- `room/README.md`: *Running it* gets the local stand-in run. There's a new *Deploying* section (the app, the one-machine rule, `--ha=false`, the secret, the trial org's five-minute stop, that smoke uses up q3 until a restart, and the DNS one-liner). The "schedules no question and authorizes nobody" sentence is rewritten, and the Canary paragraph on `--url` is updated.
- `.env.example`: append `POPQUIZ_PUBLIC_URL=` (a name only; the printf append, no read).

## Tests by criterion
- AC-64 (stand-in half): tests/standin.rs, both builds (the no-feature build in `just test`; the feature build in CI's new step), plus smoke's 401 checks against the deployed host.
- AC-28: tests/smoke_config.rs (configured public URL), with PQ-8's buzzer_page.rs kept green; smoke on the deployed host.
- AC-1 runtime half (EVALUATION: "run to release with the pipeline binary absent … (smoke)"): the image carries only the room binary, so smoke against it is that proof. I'll say so and not overclaim.
- Seeding: tests/standin.rs (feature on: POST /rooms q3 → 201 through serving_state).
- Config parsing: tests/smoke_config.rs.

## Choices
1. `include_str!`, not `POPQUIZ_QUESTIONS_DIR`: nothing is read from disk at run time, which matches how routes.rs embeds the pages. The image needs no bank directory. The binary can't be pointed at an unverified file. And there's one env var fewer.
2. The smoke driver is its own bin behind a `smoke` feature, not tests/common's `Live`. It must speak TLS (https/wss) to the Fly edge, while the dev-deps' tungstenite is deliberately TLS-free, and turning TLS on there would compile rustls into every `just test`. HTTP/1.1 is hand-rolled over tokio-rustls (hyper's client would need `want`, which is not in the lock).
3. A smoke run releases q3, and a released question is refused until the machine restarts (in memory, G-10's stand-in). So the deployed sequence is one command, shown to the client for approval: generate the token → `fly secrets set` → `just smoke` → `fly apps restart` → smoke with 200 → `fly apps restart` → print the host URL once. The token exists only in that shell's variable and in the one printed URL.
4. The burst re-run: burst.rs speaks the spike's protocol (`GET /ws`, control frames), not the room's routes, so it cannot target the real room without rework. The substitute is smoke with `--participants 200` (write p95 including a deadline burst, reveal fan-out p95, exact reconciliation), recorded, not claimed as AC-53/54/41's `burst`.
5. `just canary --url U`: the scan needs the planted canary question on the server, and only T-25's admin push can put it there. The deployed skeleton seeds q3 only. It will be run and will refuse by design; the result is recorded. Smoke carries a q3-based secrecy scan of the deployed room in its place (canary_full.rs says "T-09 wires this hook into smoke"). This is a contract tension, reported to the Orchestrator by comment.
6. The take-it-home link in the binary = `https://popquiz.rustnyc.org/last` (SPEC §13, PQ-12's fallback default). The dev `Urls::default()` (`/home`) is left as it is: the tests and the wall fixtures use it.

## Contract tensions
- The exit check says "`just canary --url <url>` green", but the hook cannot be green before T-25 (see 5). I'm taking the side of recording the refusal plus smoke's q3 scan.
- ci.yml's clearance names the no-feature build; it's the feature build that is missing (see above).

## Reset 2026-09-27 by agent:delegator-pq11
