import pathlib, sys
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"
PARENT = "ai-c11-cc/phase-machine"
PARENT_SHA = "f7faccdf20e79c783e3945e4598eba4cc29e4962"
PR = "#15"

PRESS_AHEAD = f"""**Press-ahead ticket.** Your branch starts from the in-review phase-machine branch `{PARENT}` @ `{PARENT_SHA}` (PR {PR}, code review PASS, CI green on `just test`), not from `main`, because you need the machine, the room record and the seams it adds (`room/README.md`, *The seams for T-04b and T-04c*). Rebase onto `origin/{PARENT}` while {PR} is open (the Orchestrator will tell you if it moves); the Orchestrator retargets your PR to `main` once {PR} merges. **Your sibling ticket runs in parallel on the same parent** (T-04b sessions and T-04c transport are one wave): the shared-file list in §2 is what keeps you apart; whichever of the two PRs merges second merges `origin/main` and wires the other's seam, and the Orchestrator will say which."""

# Historical record of wave 6 (2026-09-26). Since then header.py carries these learned
# clauses itself (§1a) plus the serial attach-then-status rule and the commit bypass, so a
# later generator must NOT append EXTRA again — import HEADER and pass the body only.
EXTRA = """

**Additional standing clauses (learned this run).**
- You are a delegator, not a tutor. Never hand a piece of code to a human, never leave a `TODO(human)`, never stop to ask which design to pick: decide, record it under deviations, continue.
- Check your status line right after launch: it must read `auto mode on`. If a permission prompt offers "Yes, and switch to auto mode", take that option once; then continue.
- If `git commit` fails with a 1Password error (`failed to fill whole buffer`, `agent returned an error`), retry once; then use the client-approved fallback, per invocation and never written to config: `git -c gpg.ssh.program=ssh-keygen commit …` with `SSH_AUTH_SOCK` pointing at the 1Password agent socket (`ssh-add -l` must list the signing key). Verify each commit with `git log --show-signature -1` (expect `Good "git" signature`). Never `--no-gpg-sign`.
- `just test` runs `cargo test --offline --locked`. A dependency change needs one network step to update `Cargo.lock` (`cd room && cargo fetch`) with the sandbox bypass; ask once, with the command visible, and say under deviations what you added and why. Prefer what the lock already holds.
- `just test-pipeline` cannot start on this machine (the client's `pyenv` python lacks `libintl.8.dylib`); that is the client's environment. Run `just test-room` (and `just test-web` if you touch web, which you do not) and say so; the PR's CI runs the whole `just test`.
- The `PreToolUse` hook error `dyld: Library not loaded …libintl.8.dylib` is the client's environment, non-blocking; ignore it.
- The Orchestrator's comments on the ticket are instructions. Read `(cd "$LATTICE_ROOT" && lattice comments {pq})` at every phase change.
"""

TICKETS = {}

TICKETS["PQ-5"] = dict(
    t="T-04b", slug="sessions", mode="inline-full",
    title="Sessions and the answer store", tab="Sessions", actor="pq5",
    oneliner="the participant sessions and the answer store: join, capacity, upsert while live, refusal after close, totals frozen at close, sessions dropped at release.",
    planner_extra="",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` §3.4 (the room fields your module writes: `present`, `answered_live`, `answered`, `totals`; and the *participant session* paragraph, verbatim: `token`, `answer`, no hint flag, no identity, no device record), §4.1 (joining and its six failure states), §4.2 (the hint lives in the payload, never a request), §4.3 (answering, verbatim), §4.4 (closing and totals), §4.5 (the most-chosen incorrect option is computed from `totals` at `closed` — T-04a already decides it from the snapshot you hand it), §4.6 (sessions die with the room), §9 (capacity 200, configurable), §11 (the buzzer's join failure strings and the saving/saved/failed strings — copy verbatim from `web/shared/copy.js` keys); `EVALUATION.md` AC-30, AC-34, AC-35, AC-36, AC-37, AC-46, AC-52, AC-56, AC-57 and the `burst` row; `BUILDPLAN.md` §2 (module table), the T-04a, T-04b, T-04c, T-06, T-11 rows; `sequence/USER_STORIES.md` AC-28…AC-31, AC-34…AC-37, AC-46, AC-52, AC-56, AC-57; `sequence/run-state.md` D-8, D-12; then the code on your branch: `room/README.md` (whole; *The seams for T-04b and T-04c* is your contract), `room/src/rooms.rs` (`Sessions` trait, `LiveCounts`, `close_snapshot`, `release`, `Room::accepts_answers`, `Room::revision`), `room/src/routes.rs`, `room/src/view.rs` (`view::buzzer`), `room/src/lib.rs`, `room/src/copy.rs`, `room/tests/**` (`common/mod.rs` is the in-process driver you extend), and read-only `room/src/bin/spike-server.rs` + `room/src/bin/spike_shared.rs` for PQ-3's note on **ghost sessions versus capacity** (a session whose socket dropped still exists until release; decide how it counts toward `present` and toward capacity, and say so).

**BUILDPLAN T-04b, verbatim:** Sessions and the answer store: join, capacity, answer upsert while live, refusal after close, totals frozen at close, sessions dropped at release. Criteria AC-30, AC-34–37, AC-46, AC-56. Depends on T-04a.

**Deliverables, in `room/`:**

- **The session map**, implementing T-04a's `rooms::Sessions` seam (put it in `room/src/sessions.rs` and wire it where `rooms.rs` expects the implementation; do not redesign the trait — if it lacks a method you need, add it under a `// T-04b` comment and say so under deviations). A session is `token` (opaque, random, unguessable; the only credential a buzzer ever has) and `answer ∈ A..E | none`. **No** identity, **no** device record, **no** hint flag (AC-57, AC-48). Sessions live in memory with the room and are dropped whole at `release()` (AC-56).
- **Join** (`POST /rooms/{code}/join` or the shape `routes.rs` suggests): by code, one field. The six failure states of §4.1 each with its §11 string: **malformed**, **unknown**, **not yet open** (room exists, phase `idle` — check §4: the buzzer joins in `idle` and sees the code; read the criteria rows to decide exactly which phases refuse), **already ended** (`released`), **closed for inactivity** (T-11's; ship the variant with the string and let T-11 set the condition), **full**. **Capacity is checked before any session is created and reserves nothing** (AC-30: the 201st join is refused and storage shows nothing reserved). Capacity 200, configurable through `AppState`. A successful join returns the token and the current buzzer view.
- **Answer upsert** (`PUT /rooms/{code}/answer` with the token): while `Room::accepts_answers()` (only `live`) the last write wins, idempotent, and answers the saved letter; after `closed` it is refused with the saved answer restated (AC-34, §4.3). A failed write leaves the previous answer intact on the server (AC-36). The response contract is what lets T-06 render exactly one of *saving / saved / failed* (AC-35): write it down in `room/README.md`.
- **Counts:** call `AppState::set_live_counts` on join, leave and upsert so the host sees `present` and `answered_live` move while live (AC-46). `leave` is a seam T-04c calls when a socket goes away for good; define it on your map and say what "for good" means for you (ghost sessions).
- **Close and release:** `close_snapshot()` is called once by the close transition and returns the frozen `answered` and `totals[A..E]` computed from the sessions (each session's final answer counted exactly once, AC-52); after release nothing per-session remains (AC-56).
- **The buzzer's own answer:** `view::buzzer` may carry the calling session's own saved answer and nothing about any other session; if `view.rs` needs a per-session field, add it minimally under a `// T-04b` comment (this is the one place you touch `view.rs`) and keep `tests/canary.rs` green — no other session's answer, no totals before `split`, nothing pre-reveal that the canary forbids.
- **Tests, in `test`:** AC-30 (201st join refused before creation, nothing reserved), AC-34 (five changes then close, then a refused write restating the saved answer), AC-36 (a failing write leaves the previous intact), AC-46 (counts move while live), AC-52 in-process (200 sessions answer, some change their minds, close: every final answer counted exactly once, totals sum to `answered`), AC-56 (after release: no session, no token resolves), AC-57 (a session record holds nothing but token and answer — assert the type's fields). AC-37 is T-04c's `test-full`. Keep `just test-room` well under 60 s.

**Out of scope:** the WebSocket, broadcast and reconnect (T-04c); the buzzer page (T-06); inactivity, expiry, the `used` ledger and deletion timers (T-11); the phase machine and sealed module (T-04a — never edit `phase.rs` or `answers.rs`); any `web/**` or `pipeline/**` change.

**Shared files cleared for this ticket:** `room/src/sessions.rs` (yours, new), `room/src/rooms.rs` (implementing the seam; additive), `room/src/routes.rs` (your join and answer routes, **additive only**, registered in one clearly delimited `// T-04b routes` block so T-04c's block beside it merges as a union), `room/src/view.rs` (the one per-session field, as above), `room/tests/**` (new files, plus additions to `common/mod.rs`), `room/README.md` (a sessions section). **Not** `room/Cargo.toml` or `Cargo.lock` (T-04c owns them this wave; the standard library and what the lock holds are enough for a map and a token — `getrandom` is already there), not `phase.rs`, not `answers.rs`, not `ws.rs`, not the `justfile`, not CI.""",
)

TICKETS["PQ-6"] = dict(
    t="T-04c", slug="transport", mode="inline-full",
    title="WebSocket transport and reconnect", tab="Transport", actor="pq6",
    oneliner="the WebSocket transport: one broadcast per room to wall, buzzers and host; reconnect with the same session token.",
    planner_extra="",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` §3.4 (`phase` has one writer and three readers, AC-81), §4 (the phase table's three viewer columns are the three payloads), §4.3 (the dropped socket re-attaches with the same session token; controls show *paused* until fresh state arrives), §9 (one authoritative state per room with server-push to the wall and buzzers; reveal reaches all connected buzzers ≤ 2 s p95); `EVALUATION.md` AC-37, AC-41, AC-81, the `burst` row and the `test-full` row; `BUILDPLAN.md` §2, the T-03, T-04a, T-04b, T-04c, T-21 rows; `sequence/USER_STORIES.md` AC-37, AC-41, AC-81; then the code on your branch: `room/README.md` (whole; *The seams for T-04b and T-04c* and the burst section *How it tries not to lie*), `room/src/rooms.rs` (`Room::revision`, `Sessions`), `room/src/view.rs` (`view::wall`, `view::buzzer`, `view::host` — the payloads you push), `room/src/routes.rs`, `room/src/auth.rs` (the `HostAuth` seam gates the host subscription), `room/src/lib.rs`, `room/Cargo.toml` (the `spike` feature already names `axum/ws`, `tokio/sync`, `tokio/time`, `futures-util`, `tokio-tungstenite`), `room/tests/**`; and read-only `room/src/bin/spike-server.rs` and `room/src/bin/burst.rs` (PQ-3's working WebSocket server and 200-client load generator — the shape T-21 will point at your transport; note its `TCP_NODELAY` finding).

**BUILDPLAN T-04c, verbatim:** Transport: one broadcast per room over WebSockets, reconnect with the same session token, host and wall subscriptions. Criteria AC-37, AC-41, AC-81. Depends on T-04a.

**Deliverables, in `room/`:**

- **`room/src/ws.rs`:** a WebSocket route per viewer (`wall`, `buzzer`, `host`), upgraded with `axum::extract::ws`. The wall subscribes by room code; the host by room code plus the host bearer through the `HostAuth` seam; a buzzer by room code plus its **session token**. Resolving a token to a session and telling the session map a socket is gone are **T-04b's map** — you and T-04b build in parallel, so define what you need as a small trait in `ws.rs` (`resolve(token) -> Option<SessionId>`, `gone(session)`), ship a test implementation, and say in `room/README.md` that T-04b's map implements it; the Orchestrator wires the two when the second PR merges.
- **One broadcast per room:** every change to a room bumps `Room::revision()`; on each bump push `view::wall`, `view::host`, and `view::buzzer` to every subscriber of that kind, once, from one place (a per-room `tokio::sync::watch` or `broadcast` channel — decide and say why). Each frame carries exactly one `phase` read from the room's one phase value, so the wall and every buzzer agree within one broadcast of every transition (AC-81). No polling, no per-client timers. Set `TCP_NODELAY` where the spike found it mattered.
- **Reconnect (AC-37, §4.3):** a buzzer that re-attaches with the same token gets its subscription back and a fresh `state` frame immediately, so the client can leave *paused* (the client behaviour is T-06's; the server contract is: on attach, send full current state before anything else). A second socket with the same token replaces the first.
- **AC-41's path:** the reveal broadcast is the path `burst.rs` measures; T-21 later points the `burst` recipe at your route. Keep the frame shape close to what `spike_shared.rs` expects where it costs nothing; where the real payload differs, document the mapping in the README so T-21's change is small.
- **Tests:** in `test` (hermetic, in-process, `cargo test --offline --locked`): subscribe as wall, host and a few buzzers against a bound loopback listener (like the spike's loopback tests), drive transitions through the routes, assert one frame per revision per subscriber and the same `phase` on every frame per revision (AC-81 at small scale); reconnect: drop a buzzer socket, re-attach with the same token, assert the first frame is full current state and the saved answer (from the test map) survives (AC-37 at small scale). In `test-full`: the same at 200 buzzers, asserting all 200 report the phase within one broadcast of each transition (AC-81 as written) — add a `test-full` hook the way T-15a did (a `just` recipe name and its line in `test-full` only; read the `justfile` header first; PQ-20 is editing a different line, touch nothing else). Keep `just test-room` well under 60 s.

**Out of scope:** the session map, join, capacity and the answer upsert (T-04b); the wall, buzzer and host pages (T-05, T-06, T-07); the `burst` recipe (T-21); the phase machine and sealed module (never edit `phase.rs` or `answers.rs`); deploy (T-09).

**Shared files cleared for this ticket:** `room/src/ws.rs` (yours, new), `room/Cargo.toml` and `Cargo.lock` (make the ws dependencies non-optional; keep `cargo check --features spike` green), `room/src/routes.rs` (your three ws routes, **additive only**, in one delimited `// T-04c routes` block beside T-04b's), `room/src/lib.rs` (`pub mod ws`), `room/tests/**` (new files only; `common/mod.rs` additions under a `// T-04c` comment), `room/README.md` (a transport section), the `justfile` **only** for the new `test-full` recipe line. **Not** `rooms.rs`, not `sessions.rs`, not `view.rs`, not `phase.rs`, not `answers.rs`, not CI.""",
)

for pq, d in TICKETS.items():
    wt = f"{WT}/{d['slug']}"
    branch = f"ai-c11-cc/{d['slug']}"
    text = HEADER.format(
        title=d["title"], pq=pq, t=d["t"], mode=d["mode"], parent=PARENT, parent_sha=PARENT_SHA,
        wt=wt, root=ROOT, tab=d["tab"], oneliner=d["oneliner"], actor=d["actor"],
        body=d["body"] + EXTRA.replace("{pq}", pq), planner_extra=d["planner_extra"], branch=branch,
    )
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{PARENT_SHA}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", PRESS_AHEAD),
        (f"Open the PR **against `{PARENT}`** (stacked on #8)", f"Open the PR **against `{PARENT}`** (stacked on {PR})"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`", f"the line `Based on {PR} — merge that first; this PR retargets to main afterwards.`"),
    ]
    for a, b in reps:
        assert a in text, (pq, a[:60])
        text = text.replace(a, b)
    assert "#8" not in text, (pq, "a #8 reference survived")
    out = pathlib.Path(wt) / ".claude" / "boot-prompt.md"
    out.write_text(text)
    print(out, len(text.splitlines()), "lines")
