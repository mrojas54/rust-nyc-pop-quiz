"""Wave 7 boot prompts: the two M1 follow-ups, both fast-track, both off `main`.

    python3 gen-wave7.py PQ-31 <origin/main sha>
    python3 gen-wave7.py PQ-32 <origin/main sha>      # only after PR #17 (transport) is on main

HEADER (header.py) already carries the learned clauses (§1a), the serial attach-then-status
rule and the commit bypass; nothing is appended here. Fast-track drops the plan-review
subagent and keeps one Sonnet code review.
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

TICKETS["PQ-31"] = dict(
    t="follow-up of T-04a and T-15b", slug="correct-index-twin", mode="fast-track",
    title="Room correct-index twin follows F-19: panic and ub by option kind", tab="Correct-index twin", actor="pq31",
    oneliner="the room's correct-index twin follows bank.py's F-19 rule: panic and ub answers by option kind, never by text.",
    body="""Read, in this order, before planning: `CLAUDE.md` (house rules: never write down what a program prints; answers are derived, never stored); `SPEC.md` §2 (guardrail G-2 verbatim), §3.1 (the question record and the option kinds), §3.2 (the verified record: `exit_code`, `miri.clean`, `miri.configs`; a does-not-compile record has none of them); `EVALUATION.md` AC-7, AC-9 and the `test` row; `sequence/USER_STORIES.md` AC-7, AC-9; then the code on `main`: `pipeline/src/popquiz/bank.py` — `BORROW_MODELS` and the Miri-UB helper just above `correct_index`, then `correct_index` itself (the rule you mirror, precedence and ambiguity handling included); `pipeline/tests/test_bank.py` (its `correct_index` cases — you mirror every one); `room/src/answers.rs` — the module doc, `OptionKind`, and `Record::correct_index` (the twin, today by option text for `panic` and `ub`); `room/tests/twins.rs::the_correct_option_is_derived_the_way_bank_py_derives_it` and `room/tests/common/mod.rs` (`planted_dnc` and the fixtures); `room/tests/boundary.rs` and `room/tests/canary.rs` (the sealed-module proof you must keep green); `pipeline/tests/fixtures/verify/recordings/*.json` (the verifier's machine-produced records for a panic, UB under both models, UB under one model, a compile error, q3 — your `verified` blocks come from these, copied verbatim, never typed); `(cd "$LATTICE_ROOT" && lattice comments PQ-20)` for PQ-20's DONE comment, deviation (7).

**The ticket, verbatim:** PQ-4 built `room/src/answers.rs` as an exact twin of pipeline `bank.correct_index` (by text for `panic` and `ub`). PQ-20 was cleared under F-19 to derive `panic` and `ub` answers by option kind (`panic` ↔ non-zero `exit_code`; `ub` ↔ Miri UB with both borrow models agreeing), never by text equality. Once both PRs are on main, change the Rust twin to the same rule so the two cannot drift, with the same test cases as `bank.py`'s. Criteria: AC-7 (derived, never stored), G-2.

**The F-19 ruling (Orchestrator, 2026-09-26):** `bank.correct_index` derived only output and does-not-compile answers, so an accepted `panic` or `ub` question got `None` or a distractor. Ruling: map those by option kind — `panic` ↔ a non-zero `exit_code`, `ub` ↔ Miri reporting UB with **both** borrow models agreeing — never by text. PQ-20 implemented it in `bank.py` with the precedence *does-not-compile → UB under both models → non-zero exit (panic) → output text*. Routed upstream as an amendment to §3.1; the run builds to it.

**Deliverables, in `room/`:**

- **`Record::correct_index` follows `bank.correct_index` exactly**, precedence and all: a does-not-compile record resolves to the one `does_not_compile` option; else a `miri` block with `clean == false` whose `configs` hold both borrow models resolves to the one `ub` option; else a non-zero `exit_code` resolves to the one `panic` option; else the output text rule as today. Mirror `bank.py`'s handling of "no option of that kind" and "more than one" (whatever it returns or raises, the twin does the same) — read it, do not guess. The derived index stays inside the sealed vault; no new public surface; `boundary.rs`, `canary.rs` and the AC-61 `compile_fail` doctests stay green untouched.
- **Tests, in `twins.rs`, the same cases as `test_bank.py`'s:** q3 (output text, unchanged); a does-not-compile record; a `panic` record; a `ub` record with both models; a `ub` record with one model only (falls through — say to what); the ambiguity cases; and a **cross-language pin**: one test that loads the same fixture set the pipeline tests use (or a copy of it under `bank/fixtures/` if one exists — look before creating) so a future edit to either side fails one suite. Question fixtures for the new kinds: author the options, copy the `verified` block verbatim from a verifier recording, and say in the test which recording each came from. **Never type a `stdout`, an `exit_code` or a Miri verdict.** Keep every fixture inside `twins.rs` (private helpers) — do not touch `tests/common/mod.rs`, which PQ-32 is editing in parallel.
- `answers.rs` module doc: one sentence naming the rule's precedence and that `bank.py` is the reference (G-2: if one changes, both change). No `room/README.md` edit (PQ-32 owns its sections this wave).
- `just test-room` stays well under 60 s (it is ~14 s today); note the number.

**Out of scope:** any `pipeline/**` change (`bank.py` is the reference, not yours; if you find it wrong, say so by comment and mirror it anyway); sessions, transport and their wiring (PQ-32 runs in parallel — never touch `ws.rs`, `sessions.rs`, `rooms.rs`, `routes.rs`, `lib.rs`, `view.rs`, `tests/common/**`, `tests/transport*.rs`, `tests/sessions.rs`, `tests/wiring.rs`); `web/**`; the `justfile`; CI.

**Shared files cleared for this ticket:** `room/src/answers.rs`, `room/tests/twins.rs`. Nothing else in `room/src` or `room/tests`. Not `Cargo.toml` or `Cargo.lock`.""",
)

TICKETS["PQ-32"] = dict(
    t="follow-up of T-04b and T-04c", slug="wire-sessions", mode="fast-track",
    title="Wire the session map into the transport", tab="Wire sessions", actor="pq32",
    oneliner="wiring T-04b's session map into T-04c's transport so a real buzzer can attach, answer and reconnect.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` §3.4 (the room and the participant session), §4.3 (answering; the dropped socket re-attaches with the same session token), §4.6 (sessions die with the room), §9 (one authoritative state per room, server-push); `EVALUATION.md` AC-37, AC-46, AC-81 and the `test-full` row; `sequence/USER_STORIES.md` AC-37, AC-46, AC-81; `sequence/run-state.md` D-8, D-12; then the code on `main`: `room/README.md` whole — *Sessions (T-04b)* (ghost sessions and what `leave` means; the seams T-04c uses), *The transport (T-04c)* (the seam for T-04b, what triggers a push, why the `// T-04c routes` block stays last), *The seams for T-04b and T-04c*; `room/src/ws.rs` (`SessionTokens`, `NoTokens`, `SessionId`, `Transport::new`, `Transport::changed`, the notify layer, `ws::serve`); `room/src/sessions.rs` (`SessionMap`, `impl Sessions for SessionMap`); `room/src/rooms.rs` (`AppState::new` uses `SessionMap` by default; `leave`, `buzzer_for`, `set_live_counts`, `NoSessions`); `room/src/routes.rs` (the `// T-04b routes` and `// T-04c routes` blocks); `room/src/lib.rs` (`router`, `router_with`); `room/src/view.rs` (`yours`); `room/src/main.rs` (read-only; PQ-11 owns it); `room/tests/common/mod.rs` (`TestTokens`, `Live`, `http`, `host_room`), `room/tests/transport.rs`, `room/tests/transport_full.rs`, `room/tests/sessions.rs`; and `(cd "$LATTICE_ROOT" && lattice comments PQ-5)` and `… PQ-6` for both DONE comments and their deviations.

**The ticket, verbatim:** T-04b (PQ-5) and T-04c (PQ-6) were built in parallel off the phase-machine branch, each with its own seam. Once both PRs are on main: make T-04b's session map implement `ws::SessionTokens` (`resolve(room, token)`, `saved(room, session)`, `gone(room, session)`; a later `resolve` wins over a racing `gone`), layer `Extension(Transport::new(state, map))` over `router_with`, keep the `// T-04c routes` block last in `routes()`, have every non-HTTP room writer call `Transport::changed`, and run both tickets' test suites plus an end-to-end in-process test: join, answer, transitions, reconnect with the same token (AC-37, AC-81 at small scale). Criteria AC-37, AC-81, AC-46. Blocks PQ-7, PQ-8, PQ-9 in practice: they consume the wired room.

**Deliverables, in `room/`:**

- **`SessionTokens` implemented by the real map** (on `SessionMap`, or an adapter over `AppState` — decide and say why). `SessionId` is the transport's stable per-session handle; the map keys sessions by token, so give each session a stable id minimally, under a `// PQ-32` comment. `saved` returns that session's own answer and nothing about any other (AC-57 stands).
- **`gone` never removes a session or its answer.** PQ-5's rule is binding: `AppState::leave` runs only when a socket is gone *for good* (a grace T-11 owns), never on a mere drop, or AC-37's re-attach with the same token loses the answer and a re-attach could push a room past capacity. `present` counts sessions that exist. So `gone` is at most a presence hint that changes no count; "a later `resolve` wins over a racing `gone`" then costs nothing. Record what you chose under deviations.
- **Wired by default:** the production router (`router()` / `router_with(state)`) layers `Extension(Transport::new(state, <the real map>))`, so a real buzzer attaches; `NoTokens` remains only for tests that want it. The `// T-04c routes` block stays last in `routes()`. `main.rs` is untouched (PQ-11 switches it to `ws::serve`).
- **Every non-HTTP room writer calls `Transport::changed`.** Audit `rooms.rs` and `sessions.rs`: join, upsert and leave run inside HTTP routes registered above the T-04c block, so the notify layer publishes them — verify it rather than assume it, and name any writer outside a route (T-11's reaper is future; none is expected today).
- **Tests, in `test`:** both existing suites stay green; new `room/tests/wiring.rs` drives the real map end to end over the loopback listener: `POST /join` → attach with the returned token → the first frame is the full current state (AC-37) → `PUT …/answer` → the buzzer's next attach frame carries `saved` from the real map, and the wall and host each got exactly one new frame per revision (AC-46 counts moved, AC-81 one `phase` across all) → transitions through `closed` → drop the socket → re-attach with the same token → full state first, the saved answer intact (AC-37) → `Release the room` → the token resolves to nothing. If `transport_full.rs` is built on `TestTokens`, leave it and say so; run `just test-transport-full` once locally and report the time. `just test-room` stays well under 60 s.
- `room/README.md`: the sentences that say the router serves `NoTokens` until T-04b wires in, and that T-04b's real map replaces `TestTokens`, become the wired truth; one short *Wiring* paragraph under *The transport*. Keep the Sessions section's `leave` rule as written.

**Out of scope:** the wall, buzzer and host pages (T-05, T-06, T-07 — they consume your wired room); the grace and the reaper (T-11); `main.rs`, the bind and deploy (T-09 / PQ-11); the canary plants (T-08); the correct-index twin (PQ-31 runs in parallel — never touch `answers.rs` or `tests/twins.rs`); `web/**`, `pipeline/**`, the `justfile`, CI.

**Shared files cleared for this ticket:** `room/src/ws.rs`, `room/src/sessions.rs`, `room/src/rooms.rs` (additive), `room/src/lib.rs`, `room/src/routes.rs`, `room/src/view.rs` (only if the attach frame needs it), `room/tests/common/mod.rs`, `room/tests/wiring.rs` (new), `room/tests/transport.rs`, `room/tests/transport_full.rs`, `room/tests/sessions.rs`, `room/README.md` (the Sessions and transport sections). Not `answers.rs`, not `twins.rs`, not `phase.rs`, not `Cargo.toml` or `Cargo.lock`.""",
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
    branch_note = f"**Branch.** Your branch starts from `origin/main` @ `{parent_sha}` (M0 complete; M1's phase machine, sessions and transport merged; M3's verifier, dedupe and bank audit merged). Rebase onto `origin/main` before implementing and again before your first push, never after it. Your PR opens against `main`. **A sibling follow-up runs in parallel this wave** (PQ-31 the correct-index twin, PQ-32 the wiring); the shared-file list in §2 keeps you apart."
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
