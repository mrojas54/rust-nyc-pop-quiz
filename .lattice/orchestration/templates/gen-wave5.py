import pathlib, sys
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"
PARENT = "main"
PARENT_SHA = "696b3cea87af33ec0a75a4c01ede0d820e460863"

BRANCH_NOTE = f"""**Branch.** Your branch starts from `origin/main` @ `{PARENT_SHA}` (M0 complete: scaffold, shared web layer, burst spike, bank format, sandbox image, bank audit all merged). Rebase onto `origin/main` before implementing and again before the PR. Your PR opens against `main`."""

EXTRA = """

**Additional standing clauses (learned this run).**
- You are a delegator, not a tutor. Never hand a piece of code to a human, never leave a `TODO(human)`, never stop to ask which design to pick: decide, record it under deviations, continue.
- Check your status line right after launch: it must read `auto mode on`. If a permission prompt offers "Yes, and switch to auto mode", take that option once; then continue.
- If `git commit` fails with a 1Password error (`failed to fill whole buffer`, `agent returned an error`), retry once; then use the client-approved fallback, per invocation and never written to config: `git -c gpg.ssh.program=ssh-keygen commit …` with `SSH_AUTH_SOCK` pointing at the 1Password agent socket (`ssh-add -l` must list the signing key). Verify each commit with `git log --show-signature -1` (expect `Good "git" signature`). Never `--no-gpg-sign`.
- `just test` runs `cargo test --offline --locked`. A new dependency (or a new feature on an existing one) needs one network step to update `Cargo.lock`: `cd room && cargo fetch` (or `cargo add …`) with the sandbox bypass; ask once, with the command visible, and say under deviations what you added and why. Prefer what the lock already holds (`serde`, `serde_json`, `tower` are there).
- The `PreToolUse` hook error `dyld: Library not loaded …libintl.8.dylib` is the client's environment, non-blocking; ignore it.
- The Orchestrator's comments on the ticket are instructions. Read `(cd "$LATTICE_ROOT" && lattice comments PQ-4)` at every phase change.
"""

BODY = """Read, in this order, before planning: `CLAUDE.md`; `PHILOSOPHY.md` §2 and §8 (the smaller mechanism); `SPEC.md` §2 (guardrails G-3, G-6, G-9, G-10 verbatim), §3.1 (the question record the room consumes), §3.4 (the room, verbatim, every field with its writer and readers), §4 whole (the phase table and 4.1–4.6), §8 (the auth seam) and §8.2 (the M1 stand-in — T-09 builds it; you build the seam it plugs into), §11 (the host strings and the phase labels — copy verbatim); `EVALUATION.md` AC-45, AC-47, AC-61, AC-81, AC-93, AC-97 and the `canary` row in the harness table; `BUILDPLAN.md` D-A, §2 (the module table: `auth`, `rooms`, `answers`, `ws`, `phase`), the T-04a, T-04b, T-04c, T-08 and T-09 rows (so you know what is theirs); `sequence/USER_STORIES.md` AC-45, AC-47, AC-61, AC-81, AC-93, AC-97; `sequence/run-state.md` D-8, D-9, D-10, D-12, D-19 (search the file for those IDs); then the code on `main`: `room/src/lib.rs` (the scaffold's `router()` and its module comment), `room/src/main.rs`, `room/tests/canary.rs` (the in-process seam you extend), `room/Cargo.toml` (what the lock holds), `room/README.md`, `room/src/bin/spike_shared.rs` (read-only; the spike's answer fingerprint idea, not to be reused), `bank/schema/question.schema.json` and `bank/questions/q3.json` (the record shape the room deserializes), `pipeline/src/popquiz/bank.py` (`correct_index`, `receipt_class` — the Python side of the same rules), `web/shared/copy.js` (the strings, already keyed).

**BUILDPLAN T-04a, verbatim:** The phase machine: `idle→…→released`, host-only transitions, no skipping, `trace_step`; **the sealed `answers` module** unreachable from the public state query, proven by a type/module boundary test. Criteria AC-45, AC-47, AC-61, AC-81, AC-93, AC-97, G-3, G-6. Depends on T-03. Built alone and first; `room/src/phase.rs` is consumed by everything in M1.

**Orchestrator notes on the ticket:** `room/src/phase.rs` is consumed by everything in M1: build it alone and first, and design its public surface for three consumers you will not meet — T-04b (sessions and the answer store), T-04c (WebSocket transport and reconnect), T-07 (the host phone). AC-61 is a type or module boundary proof, not a review. The D-A go was given by the client on 2026-09-26 on PQ-3's measured numbers; nothing about the stack is open.

**Deliverables, in `room/`:**

- **`room/src/phase.rs` — the machine.** A `Phase` enum with exactly the seven phases in §4 order and a `HostAction` enum with exactly the eight host actions of AC-45 (*Create a room*, *Put it on the screen*, *Close answers*, *Show the room its split*, *Let's walk it*, *Reveal*, *Release the room*, *Run it again*) plus `←`/`→`, which step the trace and are **not** transitions. One pure function `apply(state, action) -> Result<state, Refused>` that is the only way the phase changes: the seven legal transitions and nothing else; `←`/`→` legal only in `work` and `reveal`; every other pair refused with a plain-words reason. `trace_step` semantics per §3.4 and D-10: the `work` transition sets `0` and steps stay in `0..M-2`; the `reveal` transition sets `M-1` and may step the whole trace. `Run it again` yields a **new** room, never re-enters this one (G-10 is the schedule's refusal; here it is only that the old room is done). No timer, no auto-advance, no skipping (G-6).
- **`room/src/rooms.rs` (or the name BUILDPLAN §2 uses) — the room record** per §3.4: `id`, `code` (6 chars from the alphabet without `O 0 I 1`), `join_url`, `question_id`, `phase`, `host_session` (never rotates, AC-50), `host_resume_url`, `created_at`, `expires_at = created_at + 4h`, `present`, `answered_live`, `answered` (frozen at `closed`), `totals[A..E]`, `trace_step`, `released_at`, `fit`. You own the **shape** and the phase-driven writes (`closed` freezes `answered` and `totals` from a snapshot the sessions module hands you; `released` sets `released_at`). The sessions themselves, joins, capacity, the answer upsert and the socket are **T-04b and T-04c**: leave a narrow seam (a trait or a snapshot struct) for them and say in the plan what it is. In-memory only; no persistence.
- **`room/src/answers.rs` — the sealed module (G-3, AC-61).** Everything that joins an option to the verified output lives here and nowhere else: the verified `stdout`, the derived correct index (the same rule as `bank.correct_index`: by output text for a record that ran, by option kind for does-not-compile, `panic` and `ub`), the `explains` beats, the receipt lines (§7.5, rendered by the one step-list function — T-05 ports the JavaScript twin; here the Rust twin tested against `bank/fixtures/receipts/*.json`, the same files), the trace's resolving step (any `values` entry named `stdout`, and the final step). The **public state query** — the one function every wall, buzzer and host payload is built from before `reveal` — is typed so that it *cannot* reach this module: make the type system or the module boundary say so (a private module whose only exit is a function that demands a `Phase::Reveal` witness type constructible nowhere else; or a separate crate-private type the public projection has no path to). **Prove it in `test`** with a compile-time or module-boundary test: a `tests/` file that would fail to compile if the boundary opened (a `compile_fail` doctest, or `trybuild` if you add it — a dev-dependency, see the standing clause on the lock), plus a payload-shape assertion that every pre-reveal option object is exactly `{letter, text}` in the order the question arrived, and that no pre-reveal payload carries a `values` entry named `stdout`. "We reviewed it" is not the proof.
- **The question the room holds.** Deserialize the bank's record shape (`bank/schema/question.schema.json`; `bank/questions/q3.json` is your fixture) with `serde` — the lock already holds `serde` and `serde_json`; make them non-optional. Options arrive already in wall order (the schedule and the slot function are the pipeline's, T-19 and T-20; never reorder, never balance). The hint is part of every `live` payload and absent before `live` (§4.2, D-8).
- **Routes on `router()`** (the scaffold comment names you): host transition endpoints (one per `HostAction`, bearer-checked through the **auth seam**), the public state query per viewer (`wall`, `buzzer`, `host`) returning the phase-scoped payload of the §4 table, and room creation. **The auth seam (G-9, §8):** one trait or function type `HostAuth` that answers "may this bearer create a room / act as host of this room"; you ship the test implementation only. T-09's stand-in and T-10's Discord check implement it later; no `HOST_DEV_TOKEN`, no `dev-host-token` feature here. `apply` is the only path to a phase change and every route goes through it (AC-45: no transition fires without a host action).
- **Copy.** Host phase labels, host actions, the wall's beat lines and the buzzer's per-phase lines come from `SPEC.md` §11 verbatim; `web/shared/copy.js` has them keyed — mirror the keys in a Rust `copy` module so T-22's lint can compare the two, and say so in `room/README.md`. The client's own lines (*Time for a pop quiz.*, *Let's go to the bar.*) are never edited.
- **Tests, in `test` (hermetic, `cargo test --offline --locked`, well under 60 s — it is about 1 s today):** exhaustive `(phase, action)` table: the seven legal transitions and the two steps pass, every other pair is refused (AC-45, AC-93, G-6); `reveal` reachable only from `work`, `work` only from `split` (AC-93); `trace_step` bounds in `work` and entry at `M-1` in `reveal` (AC-97), on a fixture trace of M=5 and on q3's real trace; `closed` freezes `answered` and `totals` while `present` keeps moving; the most-chosen incorrect option and the *nobody read it another way* variant (§4.5) computed once at `closed`, wall and host agreeing; the AC-61 boundary test; extend `tests/canary.rs` so it drives `router()` through **every phase in order** and asserts, per phase, that pre-reveal payloads for all three viewers carry no ✓, no receipt lines, no `explains` text, no `stdout` values entry, no step beyond `M-2`, and no hint before `live` (AC-47, AC-97; T-08 plants the actual canary strings later — leave it the seam and a comment naming what it plants). AC-81 (wall and 200 buzzers agree within one broadcast) is `test-full` and T-04c's to prove over the socket; here `test` asserts there is exactly one `phase` field and all three projections read it.
- `room/README.md`: a section on the machine, the seam for T-04b/T-04c, the sealed module and how its proof works.

**Out of scope:** sessions, joins, capacity, the answer upsert, refusal after close (T-04b); WebSockets, broadcast, reconnect (T-04c); the wall, buzzer and host pages (T-05, T-06, T-07); the canary plants (T-08); the stand-in token and deploy (T-09); Discord (T-10); inactivity timers, expiry deletion and the `used` ledger (T-11); any `web/**`, `pipeline/**`, `bank/**` or `justfile` change.

**Shared files cleared for this ticket:** `room/src/**` (you are the only writer of `room/src/phase.rs` this wave), `room/tests/**`, `room/Cargo.toml` and `room/Cargo.lock` (dependency changes under the standing clause), `room/README.md`. Not the `justfile`, not CI, not `web/shared`, not `.env.example`.""" + EXTRA

PLANNER = """
**Planner subagent (sub-agent-full).** Before writing the plan yourself, spawn a planner subagent (Agent tool, `subagent_type: Plan`, `model: opus`) with the contract section paths and the ticket text; it returns a plan naming the module layout, the sealing mechanism and its proof, the seam T-04b and T-04c consume, and the test table. You own the plan file and the status bumps; merge its output into the plan, then run the review subagent as above. Ask the plan reviewer specifically whether the AC-61 proof would fail if the boundary opened.
"""

d = dict(
    t="T-04a", slug="phase-machine", mode="sub-agent-full (run inline: planner and reviewers as Agent-tool subagents, one tab)",
    title="Phase machine and sealed answers module", tab="Phase machine", actor="pq4",
    oneliner="the phase machine: seven phases, host-only transitions, trace_step bounds, and the sealed answers module.",
)
pq = "PQ-4"
wt = f"{WT}/{d['slug']}"
branch = f"ai-c11-cc/{d['slug']}"
text = HEADER.format(
    title=d["title"], pq=pq, t=d["t"], mode=d["mode"], parent=PARENT, parent_sha=PARENT_SHA,
    wt=wt, root=ROOT, tab=d["tab"], oneliner=d["oneliner"], actor=d["actor"],
    body=BODY, planner_extra=PLANNER, branch=branch,
)

reps = [
    (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{PARENT_SHA}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", BRANCH_NOTE),
    (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
     f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
    ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line", "the line"),
]
for a, b in reps:
    assert a in text, a[:60]
    text = text.replace(a, b)
assert "#8" not in text, "a #8 reference survived"
assert "stacked" not in text

out = pathlib.Path(wt) / ".claude" / "boot-prompt.md"
out.write_text(text)
print(out, len(text.splitlines()), "lines")
