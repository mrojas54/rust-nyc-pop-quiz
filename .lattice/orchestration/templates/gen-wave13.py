"""Wave 13 boot prompts: PQ-36 HC-0 copy round two (fast-track) and PQ-14 room lifecycle (inline-full), both off `main`.

    python3 gen-wave13.py PQ-36 <origin/main sha>
    python3 gen-wave13.py PQ-14 <origin/main sha>

HC-0 closed 2026-09-27 18:5x; M2 opens with PQ-14. PQ-36 carries the drive's copy findings and
inherits PQ-34's ruled exception to clause 7 (SPEC §5/§11 rows and prototype strings, the ruled
changes only). PQ-37 (guest split) presses ahead off PQ-36 at review; its generator is wave 14.
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

TICKETS["PQ-36"] = dict(
    t="HC-0 finding", slug="copy-round-two", mode="fast-track", title="HC-0 copy, round two", tab="Copy Round Two", actor="pq36",
    oneliner="the second round of the client's HC-0 copy rulings landed across buzzer, wall and host, with SPEC and the prototype kept in step.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` §5 (the phase table) and §11 (the copy contract); `web/shared/copy.js` whole; `room/src/copy.rs` (the Rust mirror, pinned both ways by `room/tests/twins.rs`); `room/src/phase.rs` only to see which `copy::HOST_ACTION_*` consts it reads (you do not edit it); `web/wall/wall.js` (the hard-coded `BRAND` constant near line 41), `web/wall/fallback/`, `web/host/`, `web/buzzer/`; `room/tests/canary.rs` and `room/tests/phase_table.rs` (host-action label lists); and on the board `(cd "$LATTICE_ROOT" && lattice comments PQ-34)` (how the first round was landed — follow its shape) then `lattice comments PQ-36` (**the findings, one comment each — they are the ticket**).

**The client's rulings (HC-0, 2026-09-27, from the deployed drive), each already on the ticket:**
1. Buzzer, live, no answer yet: *tap a letter* → **Vote** (`buzzer_no_answer_yet` / `BUZZER_NO_ANSWER_YET`; SPEC §11 *Buzzer, live, submission*).
2. Wall, live join strip: keep *join @ ‹link›*, drop *· still open* (`wall_live_join` / `WALL_LIVE_JOIN`; SPEC §5 live row, §11 *Wall, live*).
3. Host action *Let's walk it* → **Trace** (`host_action_walk` / `HOST_ACTION_WALK`; SPEC §5 split and work rows, §11 *Host, actions*; label lists in the tests).
4. Host action *Release the room* → **End Pop Quiz** (`host_action_release` / `HOST_ACTION_RELEASE`; SPEC §5 reveal and released rows, §11 *Host, actions*). The phase name `released` in code and payloads does not change.
5. Wall brand line: the hard-coded *RUST NYC · POP QUIZ* becomes the wordmark **Rust NYC Pop Quiz** from `title_wordmark`, rendered in the wordmark face at the same top-left position, sized to read from the back row; the `BRAND` constant is deleted so the string has one source; the static fallback renders the same; SPEC §5 idle row and the §11 *Title* row gain the wall. This closes contract gap F-22.

**Ruled exception to clause 7, this ticket only (PQ-34's ruling extended by the Orchestrator):** you edit `SPEC.md` §5 and §11 and `prototypes/C-projector-first.html` strings for exactly these five changes, nothing else in those files. Each change lands in `copy.js`, `copy.rs`, the rendering surface, its tests, the SPEC rows and the prototype in **one commit per finding** (five commits), so a reviewer can read each ruling as a diff.

**Sibling running this wave:** PQ-14 (room lifecycle) owns `room/src/rooms.rs`, `sessions.rs`, `phase.rs`, `lib.rs`, `routes.rs`, `view.rs`. You touch **none** of those; `phase.rs` reads your consts by name, so a label change needs no edit there. PQ-35 (README) owns `README.md` and `docs/demo/**`. PQ-37 (guest split) starts off your branch once you are at `review`: keep your buzzer edits minimal and additive.

**Exit check before DONE:** `rg -n "tap a letter|still open|walk it|Release the room|RUST NYC · POP QUIZ|Rust NYC · Pop Quiz"` across `web/ room/ SPEC.md prototypes/` returns nothing; `just test-room` green including `twins.rs` both directions, the label tests and `just canary` (9/9); `just test-web` green; the fallback rebuilt (`python -m popquiz.fallback q3 --out …`) and compared to the live wall DOM; one screenshot each of the wall's header, the host's action buttons and the buzzer's live line, after the change, attached as validation.

**Out of scope:** any string the client did not rule on; layout beyond the header; the buzzer split (PQ-37); `room/src/**` except `copy.rs`; `pipeline/**` except the fallback build; `DESIGN.md`, `EVALUATION.md`, `BUILDPLAN.md`, `sequence/**`.

**Shared files cleared for this ticket:** `web/shared/copy.js` (serialized: yours alone this wave), `room/src/copy.rs`, `web/wall/wall.js`, `web/wall/wall.css` (header sizing only), `web/wall/fallback/**`, `web/host/**` and `web/buzzer/**` (label rendering only), `web/test/**`, `room/tests/**` (assertions on the changed strings only), `SPEC.md` §5/§11 rows and the prototype's strings under the exception.""",
)

TICKETS["PQ-14"] = dict(
    t="T-11", slug="room-lifecycle", mode="inline-full", title="Room lifecycle and the used ledger", tab="Room Lifecycle", actor="pq14",
    oneliner="the room's whole life bounded: 4 h expiry, deletion at release, totals that expire, the used record written at release and nowhere else, the take-it-home snapshot rebuilt at release, Run it again refusing a used question.",
    body="""Read, in this order, before planning: `CLAUDE.md` (the house rules; *a built deck is not a run segment* and *AC-92 asks for what each meetup used* are this ticket's reason to exist); `PHILOSOPHY.md` §2 (the ledger is a record, never an input); `SPEC.md` §3.3 (the bank's `used` record: `{meetup_date, room_id, released_at, fit}`, its only writer the release transition, G-10), §3.4 (the room record: `created_at`, `expires_at = created_at + 4h`, `released_at`, `fit`, and which writer owns each), §4.6 *Lifecycle* whole (≤ 4 h, deleted with its sessions at release or expiry, *closed for inactivity*, what survives: the `used` record and the rebuilt take-it-home page, never twice), §5 released row (*Run it again* → a new room, never the same question), §8.3 (T-25 later serves the used record over `GET /admin/used` — leave that seam shaped for it), §9 (one authoritative state per room, in memory), §13 (take-it-home: what `/last` shows; T-12 renders it, you produce the snapshot it renders); `SPEC.md` guardrails **G-4** and **G-10** (lines ~61 and ~67); `sequence/USER_STORIES.md` AC-56, AC-57, AC-69, AC-92; `EVALUATION.md` rows AC-56, AC-57, AC-69 (`test-full`: a room dies at 4 h, an open room runs to release with the auth provider down — design the clock so the test can drive it), AC-92 (**written at release, not at build; a room that never reaches release records nothing**); `BUILDPLAN.md` T-11 (line ~180), T-12 (consumes your snapshot), T-25 (consumes your used record), T-19 (retired the old ledger write); then the code on `main`: `room/src/rooms.rs` whole (`expires_at`, `released_at`, `ROOM_LIFETIME`, `close_snapshot`, the `Released` arm near line 450, the `Sessions` seam and `release`), `room/src/sessions.rs` (what `release` drops today), `room/src/phase.rs` (the transition table; `Released` is terminal — you hang the release side effects on the transition, in `rooms.rs`, never inside `phase.rs` unless a transition itself must change), `room/src/answers.rs` public surface only (the question record the snapshot copies from; never reach inside the sealed module — the take-it-home snapshot is built **after** `Machine::revealed()` has minted the witness, through the same reads the wall's reveal uses), `room/src/view.rs` (projection shapes), `room/src/routes.rs` (`POST /rooms`, the host routes, `GET /{code}`), `room/src/lib.rs` (`AppState`, how state is provided), `room/tests/common/mod.rs` and `room/tests/sessions.rs` (the harness and the AC-56 tests that exist), `room/README.md` (*What is here*, the seams section), `pipeline/src/popquiz/bank.py` (`Used` dataclass: the field names and types the pipeline expects — your record must serialize to exactly that shape, `fit` from the room's last `reveal` verdict).

**BUILDPLAN T-11, verbatim:** Lifecycle: 4 h expiry, room deleted at release, totals with expiry, **`used` written at release**, take-it-home rebuild at release, *Run it again* refuses a used question. Criteria AC-56, AC-57, AC-69, AC-92, G-4, G-10. Depends on T-04b. **The only writer of the used-question ledger.**

**Deliverables:**

- **Expiry.** A room lives ≤ 4 h from `created_at` (AC-69): a sweep (a tokio task or a check on every state touch — decide, say why; it must be testable with an injected clock, never `SystemTime::now()` inline) deletes an expired room and all its sessions whole; joins and host commands on an expired room answer with the *already ended* refusal PQ-8 built (AC-29's states, reuse the copy key); the wall and phones learn through the transport as they do today. *Closed for inactivity* per §4.6 (a room in `idle` with no host activity for the SPEC's bound) uses the same deletion path and the *closed for inactivity* refusal. Creation is unchanged (the live auth check is T-10's).
- **Release.** The `released` transition, in `rooms.rs`: (1) writes the **used record** `{meetup_date, room_id, released_at, fit}` (`fit` = the wall's verdict at `reveal`, copied at release per §3.4; `meetup_date` from the room's date in the configured zone — say which and where it is configured) into an in-memory `Used` ledger on `AppState`, append-only, the **only** writer in the crate (a structural test scans `room/src` for any other write); (2) rebuilds the **take-it-home snapshot**: the last released question with colour, the trace, three beats, receipt, options with ✓ and the counts at release (§13, AC-95's *count from release*), stored on `AppState` as the one thing `/last` will render (T-12 builds the page; you expose `GET /last.json` or a typed accessor — decide, say why — and a `room/tests` fixture of its shape); (3) then deletes the room record and every session (AC-56): after release nothing per-participant exists anywhere — test it by inspection of `AppState`, not by absence of a route.
- **Totals with expiry.** The anonymous totals that survive release (§3.4, AC-56) carry `expires_at` and are dropped by the same sweep; the take-it-home snapshot is the only long-lived copy and carries counts, not people.
- **Never twice.** *Run it again* (`POST /rooms/{id}/again` or whatever PQ-4 named it) creates a new room and **refuses the used question** with a specific refusal (a copy key exists or is added to `copy.js` — if you need a new string, stop and ask the Orchestrator by comment; `web/shared/copy.js` is PQ-36's this wave); creating a room on a `question_id` present in the used ledger is refused the same way (G-10, AC-92). Building a deck writes nothing (already true after T-19; add the assertion to your tests as the AC-92 row asks).
- **Seams left for later:** T-25 reads the used ledger (`GET /admin/used`) — expose a `Used::all()` (or equivalent) with the pipeline's JSON shape, tested against `bank.py`'s field names; T-12 reads the snapshot; T-20's sync reads used records. Name each seam in `room/README.md`.
- **Tests, in `test`** (`just test-room` stays well under 60 s; note the number): AC-69 expiry with the injected clock (room gone at 4 h, sessions gone, refusals after); AC-56 nothing per-person survives release; AC-57 static (no participant identity field anywhere, extend PQ-5's test if it exists); AC-92 used written at release only, not at build, not at reveal, not on an expired room; the never-twice refusals; the snapshot shape fixture; the single-writer structural scan. The `test-full` AC-69 row (auth mock down) is T-10's to complete — leave a named `#[ignore]` test with its shape.

**Exit check before DONE** (the PR body carries the evidence): every criterion above mapped to a test name; `just canary` green in every phase (the snapshot must not leak anything before release — it is built only at release, from the witnessed reads); `just test-web` untouched and green; `just test-room` time recorded.

**Out of scope:** the take-it-home page itself (T-12); the admin routes (T-25); Discord auth and the live check at creation (T-10); the schedule and `popquiz sync` (T-20); any copy string (PQ-36 owns `copy.js` and `copy.rs` this wave — ask by comment if you need a new key); `web/**`; `pipeline/**` except reading `bank.py` for the shape.

**Shared files cleared for this ticket:** `room/src/rooms.rs`, `room/src/sessions.rs`, `room/src/lib.rs` (additive), `room/src/routes.rs` (additive, under `// T-11`), `room/src/view.rs` (the snapshot shape, additive), `room/src/phase.rs` (**serialized to you this wave; only if a transition must change — say so in the plan**), a new `room/src/lifecycle.rs` and/or `room/src/used.rs`, `room/tests/lifecycle.rs` and `room/tests/used.rs` (new), `room/tests/common/mod.rs` (additive, under `// T-11`), `room/README.md`. **Not** `answers.rs` internals, `copy.rs`, `ws.rs`, `standin.rs`, `config.rs`, `web/**`, `pipeline/**`, `justfile`.""",
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
    branch_note = f"**Branch.** Your branch starts from `origin/main` @ `{parent_sha}` (M1 complete and live, HC-0 closed, the copy trim PR #27 merged). Rebase onto `origin/main` before implementing and again before your first push, never after it. Your PR opens against `main`. Siblings this wave are named in §2; touch none of their files."
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", branch_note),
        (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
         f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line", "the line"),
    ]
    if d["mode"] == "fast-track":
        reps.append((PLAN_REVIEW.replace("{pq}", pq), FAST_TRACK_PLAN.replace("{pq}", pq)))
    if pq == "PQ-36":
        reps.append(("and never edit the contract files (`SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `sequence/**`, `prototypes/**`, `mvp/**`).",
                     "and never edit the contract files (`SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `sequence/**`, `prototypes/**`, `mvp/**`) — **except the ruled exception in §2 for this ticket: `SPEC.md` §5/§11 rows and the prototype's strings, the five ruled changes only.**"))
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
