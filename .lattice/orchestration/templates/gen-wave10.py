"""Wave 10 boot prompt: PQ-11 deploy with the host stand-in (T-09), fast-track, off `main`.

    python3 gen-wave10.py PQ-11 <origin/main sha>

Released by the client 2026-09-26 ("release PQ-11 with fly's fallback"): no DNS, the
Fly-provided hostname is the host. HEADER (header.py) carries the learned clauses; the
fast-track substitution is the same as wave 7's.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER  # noqa: E402

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"
PARENT = "main"

PLAN_REVIEW = "Then get fresh eyes: spawn a review subagent with the **Agent tool** (`subagent_type: general-purpose`, `model: sonnet`) whose prompt contains only the plan file path, the contract section paths from §2, and the instruction to find contradictions with the contract and missing pieces, returning Critical/Major/Minor findings with file references — not your conclusions. Triage every finding into a block appended to the plan: `## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)`, one entry per finding with concern / resolution. `lattice status {pq} planned`."
FAST_TRACK_PLAN = "Fast-track: no plan-review subagent. Re-read the plan once against the contract sections in §2 and the code on your branch, fix what contradicts, then `lattice status {pq} planned`."

TICKETS = {}

TICKETS["PQ-11"] = dict(
    t="T-09", slug="deploy", mode="fast-track", title="Deploy to Fly with the host stand-in", tab="Deploy", actor="pq11",
    oneliner="the M1 skeleton deployed on Fly under the fallback hostname, behind the §8.2 host stand-in, with q3 seeded, the real join link, and `just smoke` against it.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` §8.2 whole (the stand-in, verbatim — it is the contract for this ticket), §8.3 (T-25's admin channel: read so you leave its seam alone), §3.4 (`code`, `join_url`), the join paragraph near line 183 (the short link carries the code), §9 (one authoritative state per room, in memory), §12 and §13 (the take-it-home host the static fallback links to); `EVALUATION.md` AC-28, AC-64 (the stand-in half only), the `smoke` and `burst` harness rows (lines ~36–38), the HC-0 row (line ~168) and the human-track table below it (the M1 host token row: T-09 prints the host URL once); `sequence/USER_STORIES.md` AC-28, AC-64; `BUILDPLAN.md` the T-09 row (line ~167), D-A (Rust on Fly, the client pays), §4 human track H-2; `sequence/run-state.md` D-19; then the code on `main`: `room/src/main.rs` (binds `127.0.0.1:3000` and serves `room::router()`; PQ-6's DONE says it must call `ws::serve` — read `room/src/ws.rs::serve` and why), `room/src/auth.rs` whole (the `HostAuth` seam and `DenyAll`; its module doc says `HOST_DEV_TOKEN` and the `dev-host-token` feature are yours, behind that trait), `room/src/lib.rs` (`router`, `router_with`, how state and auth are provided), `room/src/rooms.rs` (`AppState::new`, `join_url`, `host_resume_url`, how a question is scheduled into a room — the seam T-25 later replaces), `room/src/question.rs` and the public surface of `room/src/answers.rs` (the record shape the room loads; **never** reach inside the sealed module), `room/src/routes.rs` (`POST /rooms {question_id}`, `GET /{code}`), `room/tests/common/mod.rs` (the test `HostAuth` implementation and how tests schedule a question — the model for your stand-in and your seeding), `room/README.md` (*Running it*, *Routes*, *Pages*, the sentence "As built, the binary schedules no question and authorizes nobody"), `web/host/` (the create screen reads `?question=` and sends the fragment as the bearer — PQ-9's DONE), `bank/questions/q3.json` and `pipeline/src/popquiz/bank.py` (q3 is the only migrated verified record and the HC-0 question; the loader shows the record's shape), the spike from PQ-3: `fly.spike.toml`, `room/src/bin/` and the `spike` feature in `room/Cargo.toml` (`cargo check --features spike` is the by-hand gate; the spike server is not the room), `.github/workflows/` (CI: what `test` and `test-full` run), the `justfile` whole (the reserved-name header: `smoke` is a reserved name that fails loudly today — you make it real), `.dockerignore` and any `Dockerfile` (note: `pipeline/sandbox/` holds PQ-19's verification image; the room has none yet); and on the board: `(cd "$LATTICE_ROOT" && lattice comments PQ-3)` (the Fly notes: `--ha=false`, `fly launch` rewrites toml, the trial org stops machines after 5 min, the spike app is scaled to zero, not destroyed), `… PQ-6` (deviation 4: `main.rs` → `ws::serve`), `… PQ-9` (the T-09 contract note: the host URL is `/host?question=<id>#<HOST_DEV_TOKEN>`), `… PQ-12` (the take-it-home link default vs the dev `Urls.home`), `… PQ-10` (`just canary --url U` is the deployed-room hook; the Orchestrator's ruling routes it to you).

**BUILDPLAN T-09, verbatim:** Deploy: Fly app, `popquiz.rustnyc.org` (or fallback host), the short link carrying the code, `smoke`; the skeleton is built with the `dev-host-token` feature, `HOST_DEV_TOKEN` is set as a Fly secret, and the host URL is printed once (`SPEC.md` §8.2, D-19). Criteria AC-28, AC-64 (stand-in). Depends on T-05–07. Human track: DNS. Serialized on `justfile` (`smoke`). Adds the name `HOST_DEV_TOKEN`, never a value, to `.env.example`.

**The client's release (2026-09-26):** H-2 DNS is absent; **the Fly-provided hostname is the host** (`https://<app>.fly.dev`). When DNS lands later, the switch is `fly certs add popquiz.rustnyc.org` plus one config value — record that as a one-line note in the README, do not wait for it. The Fly org has no payment method: Fly stops every machine after about five minutes and the room's state is in memory, so a stopped machine loses the room. Do not work around this in code; record it in the README's deploy section and in your DONE note (HC-0 runs inside a window, or the client adds a card).

**Deliverables:**

- **The §8.2 stand-in, behind `dev-host-token`:** a `HostAuth` implementation that accepts exactly the bearer equal to `HOST_DEV_TOKEN` (read from the environment at startup; a missing or empty value with the feature on is a startup error, never a silent deny-all) for *Create a room* and *Run it again*, grants nothing else, compares in constant time (hand-rolled over bytes is fine; no new dependency for it), and logs nothing about the token. Without the feature the binary keeps `DenyAll` and **no code path that reads or compares the token exists** — gate the whole module and its wiring with `#[cfg(feature = "dev-host-token")]`, and add the AC-64 proof: a `test` that a build **without** the feature refuses *Create a room* for any bearer (401), and a structural scan (the shape of `room/tests/boundary.rs`) that `HOST_DEV_TOKEN` and the stand-in module are unreachable when the feature is off; with the feature on, a wrong or missing bearer is 401 with no body and the right one creates a room.
- **The HC-0 question, seeded:** at startup, behind the same feature, schedule `q3` from `bank/questions/q3.json` (embed it with `include_str!` or read a path from `POPQUIZ_QUESTIONS_DIR` — decide and say why; the record is verifier-produced and is never edited by hand) through the same seam `tests/common` uses, so `POST /rooms {"question_id":"q3"}` works on the deployed skeleton. A production build without the feature schedules nothing until T-25's `PUT /admin/questions/{id}` (leave that seam exactly as PQ-4 and PQ-32 left it).
- **`main.rs` made deployable:** bind from the environment (`PORT` → `0.0.0.0:<PORT>` as Fly sets it; default `127.0.0.1:3000` locally), call `ws::serve` (PQ-6's deviation 4), and take the public base URL from `POPQUIZ_PUBLIC_URL` so `join_url`, the wall's *join @* line and `GET /{code}` carry the real host (AC-28: join via the short link with the code carried, and by the typed code). The take-it-home link stays what PQ-12 and SPEC §13 say; note in deviations if the dev `Urls.home` needs the same base.
- **The Fly app:** a `Dockerfile` for the room (multi-stage, release build with `--features dev-host-token`, small runtime image — say the size), `.dockerignore`, and a hand-written `fly.toml` for a **new** app (`rustnyc-popquiz` or the shape the spike used; the spike app `rustnyc-popquiz-spike` stays untouched), one machine in `ewr`, **never two** (`fly deploy --ha=false`; read PQ-3's note), `PORT` set, no dedicated IPv4. Use `fly apps create` + `fly deploy`, **not `fly launch`** (it rewrites the toml and strips comments). Every `fly` command prompts the client in this tab: ask once each, with the command visible. Generate the token with `openssl rand -hex 32` piped straight into `fly secrets set HOST_DEV_TOKEN=…` in one command so the value never lands in your transcript's plain output, a file, a comment or the PR; then **print the host URL `https://<app>.fly.dev/host?question=q3#<token>` exactly once, in this tab, and nowhere else** (EVALUATION's human-track row: the client keeps that URL). Under deviations record only that it was printed.
- **`just smoke <url>`** (replacing the reserved-name stub; keep the header comment honest): drive the deployed room end to end through all seven phases with mock participants over real sockets against `<url>` — reuse `room/tests/common`'s `Live`/driver code pointed at a URL, or a small `room/src/bin/smoke.rs` behind a `smoke` feature (say which); it needs the host credential from `HOST_DEV_TOKEN` in the environment, never on the command line. It is `test-full`'s deployed-room run only when T-21 wires it; today it is a by-hand gate. Then run **`just canary --url <url>`** (PQ-10's hook) against the deployed room and record the result. Then the **burst re-run** (F-2, HC-0's trigger): if PQ-3's client half can target the real room's routes, run it against the deployed skeleton and record AC-54/AC-53/AC-41 in the PR; if it cannot without rework, say so under deviations and record instead the `smoke` timing with 200 mock participants — do not rebuild the spike.
- **`.env.example`:** the Orchestrator already appended the name line `HOST_DEV_TOKEN=` (the sandbox cannot read this file; F-5). If you introduce `POPQUIZ_PUBLIC_URL` or `POPQUIZ_QUESTIONS_DIR`, append their names the same way with `printf '%s\\n' 'NAME=' >> .env.example` (a write, no read) and stage the file by name with the sandbox bypass. Never a value.
- **`room/README.md`:** *Running it* gains the local stand-in run (`HOST_DEV_TOKEN=<any value> cargo run --features dev-host-token`, then open `/host?question=q3#<that value>`, the wall at `/wall/{room_id}`, phones at `/join` — this is how HC-0 runs at a desk, and how the Orchestrator will hand it over), a *Deploying* section (the app, the one-machine rule, the secret, the trial-org five-minute stop, the DNS one-liner for later), and the "schedules no question and authorizes nobody" sentence rewritten to the truth.
- **Tests, in `test`** (`just test-room` stays well under 60 s; note the number): the AC-64 stand-in cases above, both with and without the feature (CI must build both — add the no-feature run to the workflow only if `test` does not already cover it; say so); AC-28 with a configured public URL (the short link and `join_url` carry it; the join form has no other field is PQ-8's test, keep it green); the seeding test; `main.rs`'s configuration parsing (a pure function, tested).

**Exit check before DONE** (the PR body carries the evidence): the app answers at the fallback host; `just smoke <url>` green; `just canary --url <url>` green; the burst re-run or its substitute recorded; the host URL printed once in this tab; the spike app untouched; `just test` warm time recorded.

**Out of scope:** DNS (H-2); Discord auth (T-10 deletes your feature later — leave it cleanly deletable); the admin channel and the repo token scan (T-25); `smoke`/`burst` in CI (T-21); the runbook (T-24); `web/**`; `pipeline/**`; the sealed module and the phase machine; a card on the Fly org (the client's).

**Shared files cleared for this ticket:** `room/src/main.rs`, `room/src/auth.rs`, `room/src/lib.rs` (feature wiring, additive), `room/src/rooms.rs` (the public base URL, additive), `room/src/config.rs` (new, if you want one), `room/src/standin.rs` or a module under `auth` (new), `room/src/bin/smoke.rs` (new, optional), `room/Cargo.toml` (the `dev-host-token` and `smoke` features; no new dependency unless the lock already holds it — say what and why), `room/tests/standin.rs` and `room/tests/smoke_config.rs` (new), `room/tests/common/mod.rs` (additive, under `// T-09`), `Dockerfile`, `.dockerignore`, `fly.toml` (all new, repo root), `.github/workflows/*` (only to add the no-feature build if needed), `justfile` (the `smoke` recipe and its header line only), `.env.example` (append only), `room/README.md`. Not `room/src/phase.rs`, `answers.rs`, `ws.rs`, `view.rs`, `routes.rs` (if a route is truly needed, comment first), not `web/**`, not `pipeline/**`, not `fly.spike.toml`.""",
)


def main():
    pq, parent_sha = sys.argv[1], sys.argv[2]
    assert len(parent_sha) == 40, "pass the full 40-char origin/main sha"
    d = TICKETS[pq]
    wt = f"{WT}/{d['slug']}"
    branch = f"ai-c11-cc/{d['slug']}"
    text = HEADER.format(
        title=d["title"], pq=pq, t=d["t"], mode=d["mode"], parent=PARENT, parent_sha=parent_sha,
        wt=wt, root=ROOT, tab=d["tab"], oneliner=d["oneliner"], actor=d["actor"],
        body=d["body"], planner_extra="", branch=branch,
    )
    branch_note = f"**Branch.** Your branch starts from `origin/main` @ `{parent_sha}` (M0 complete; every M1 ticket but this one merged: phase machine, sessions, transport, wiring, wall, buzzer, host phone, canary, static fallback). Rebase onto `origin/main` before implementing and again before your first push, never after it. Your PR opens against `main`. No sibling runs this wave."
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", branch_note),
        (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
         f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line", "the line"),
        (PLAN_REVIEW.replace("{pq}", pq), FAST_TRACK_PLAN.replace("{pq}", pq)),
    ]
    for a, b in reps:
        assert a in text, (pq, a[:70])
        text = text.replace(a, b)
    assert "#8" not in text, "a #8 reference survived"
    assert "stacked" not in text
    out = pathlib.Path(wt) / ".claude" / "boot-prompt.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(text)
    print(out, len(text.splitlines()), "lines")


if __name__ == "__main__":
    main()
