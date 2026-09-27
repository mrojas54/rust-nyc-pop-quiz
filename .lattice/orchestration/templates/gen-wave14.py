"""Wave 14 boot prompt: PQ-37 guest view shows the answer split (inline-full), press-ahead off PQ-36's branch.

    python3 gen-wave14.py PQ-37 <origin/ai-c11-cc/copy-round-two sha> <PQ-36 PR number>

Stacked on PQ-36 (copy round two) because both touch web/buzzer and copy.js; the PR is
retargeted to main by the Orchestrator before PQ-36's PR merges (the #17/#18 hazard).
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER  # noqa: E402

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"
PARENT = "ai-c11-cc/copy-round-two"

TICKETS = {}

TICKETS["PQ-37"] = dict(
    t="HC-0 finding", slug="guest-split", mode="inline-full", title="Guest view shows the answer split", tab="Guest Split", actor="pq37",
    oneliner="the phone showing the room's per-letter counts from the split onward, fed from the wall's own totals so the two can never disagree.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `PHILOSOPHY.md` (no scores, no leaderboards — a count per letter is the room's shape, not a person's); `SPEC.md` §3.4 (the room's `totals`), §4.4 (frozen `answered`/`totals` after close), §4.5 (the most-chosen incorrect option: wall and host can never disagree — the model for your wall-and-phone agreement test), §5 (the phase table: today the buzzer cell reads *Room code only* at split and work — you change that), §11 (the copy contract: the wall's bar labels, `n · p%`, *N of M in the room answered*); `SPEC.md` guardrails G-3 (pre-reveal secrecy) and G-4 (nothing per-person leaves the phone after close); `sequence/USER_STORIES.md` AC-40 (colour never the only signal), AC-58 (any personal recap is computed on-device and never transmitted), AC-94 (no ✗), AC-95; `EVALUATION.md` the `canary` row (line ~35: the payload-shape assertions you extend) and the AC-58 row; `DESIGN.md` (the buzzer's type model and the phone column); `prototypes/C-projector-first.html` (the buzzer at split and reveal, and the wall's bars — the visual you scale down); then the code **on your branch** (PQ-36's, which carries the round-two copy): `room/src/view.rs` (`BuzzerPayload`, `WallPayload` and how the wall's bars are projected), `room/src/answers.rs` public surface and the `sealed::wall` / `sealed::buzzer` projections (read how the wall gets its totals; never reach inside the vault), `room/src/rooms.rs` (where totals live and freeze), `room/src/ws.rs` (the hub republishes every projection on every revision — nothing to change), `room/tests/canary.rs`, `room/tests/canary_scan/`, `room/tests/buzzer_page.rs`, `room/tests/phase_table.rs`; `web/buzzer/buzzer.js` and `buzzer.css`, `web/wall/wall.js` (the bars' DOM and CSS classes to reuse), `web/shared/copy.js` (the wall's bar strings), `web/test/`; and on the board `(cd "$LATTICE_ROOT" && lattice comments PQ-36)` (the copy rulings you build on) and `lattice comments PQ-37` (the client's ruling and the Orchestrator's ticket text — the contract for this ticket).

**The client's ruling (HC-0, 2026-09-27, verbatim):** *"guest view should have stats for answers."*

**Deliverables:**

- **The payload.** `BuzzerPayload` gains one optional field (name it `split` or `counts`; say why) carrying, from `split` on (split, work, reveal), the same five entries the wall's bars are built from: letter, count, percent, in the wall's order; absent in idle, live and closed; absent after release. It is filled server-side from **the same totals the wall's projection uses** — the wall and the phone can never disagree; a `test` asserts equality for every room state the harness can reach (the §4.5 pattern). Nothing per-participant: the reader's own letter is already known on the phone (AC-58 stays exactly as it is).
- **The phone.** At split, work and reveal the buzzer renders the five counts under its existing lines as bars scaled to the phone column, reusing the wall's bar markup and CSS classes where they fit; the reader's own letter marked as theirs by a glyph and a label, never by colour alone (AC-40); no ✗ anywhere (AC-94); at reveal the ✓ letter marked as the wall marks it. Copy: reuse the wall's bar strings from `copy.js`; if a phone-only string is truly needed, stop and ask the Orchestrator by comment (no new sentence without the client). The buzzer's existing lines (*Look up.*, the count line, *✓ It was Y.*, *You didn't answer.*) keep their places.
- **Secrecy proof.** Extend `canary` and `canary_scan`: the new field is allowed only from split on, carries letters, counts and percents only, and every plant is still absent from every buzzer payload, frame and page in every phase; pre-reveal option objects stay exactly `{letter, text}`. The mutation check: plant the field in `live` once and prove the canary fails.
- **Contract, under PQ-34's ruled exception (extended by the Orchestrator to this ticket):** `SPEC.md` §5 split, work and reveal rows' buzzer cells say what the phone now shows; §11 gains nothing unless you add a string; `prototypes/C-projector-first.html`'s buzzer at those phases shows the bars. `DESIGN.md` is **not** edited — its rule that the phone shows only its own count is routed upstream as F-29 by the Orchestrator; name the tension under deviations. Nothing else in the contract files.
- **The client's look.** Before the PR: run the stand-in locally, drive a room to split with three buzzer tabs, screenshot the phone at split and at reveal, print the paths in your tab and `c11 raise-flag --surface "$MY_SURF" "PQ-37: phone split screenshots ready for your look in this tab"`, and wait for the client's word; adjust once if asked, then continue.
- **Tests, in `test`:** the wall-equals-phone totals test; the field absent before split and after release; the shape assertions; the buzzer page test for the bars' DOM at each phase; `just test-web` for the renderer; `just test-room` under 60 s (note the number); `just canary` 9/9 plus your new plant.

**Exit check before DONE:** every item above mapped to a test name or a screenshot; `just canary --full` green over real sockets; the fallback (`web/wall/fallback/`) untouched and its compare still green.

**Siblings this wave:** PQ-36 (your parent branch; do not edit the strings it changed); PQ-14 room lifecycle owns `rooms.rs`, `sessions.rs`, `phase.rs`, `lib.rs`, `routes.rs` and the lifecycle parts of `view.rs` — you add to `view.rs` only the payload field and its projection, in an additive block under `// PQ-37`; PQ-35 README.

**Out of scope:** the wall (already shows the bars); the host phone; per-person anything; `DESIGN.md`; copy beyond reuse; `pipeline/**`.

**Shared files cleared for this ticket:** `room/src/view.rs` (additive block), `room/src/answers.rs` **projection functions only** (`sealed::buzzer` or its equivalent, the same reads `sealed::wall` makes — never the vault, never `RevealWitness` discipline; `boundary.rs` must stay green), `room/tests/canary.rs`, `room/tests/canary_scan/**`, `room/tests/buzzer_page.rs`, `room/tests/phase_table.rs` (assertions only), `web/buzzer/**`, `web/test/**`, `SPEC.md` §5 buzzer cells and the prototype's buzzer under the exception. Not `copy.js` (ask if you need a string), not `web/wall/**`, not `web/host/**`, not `rooms.rs`, `sessions.rs`, `phase.rs`, `ws.rs`, `routes.rs`, `lib.rs`.""",
)


def main():
    pq, parent_sha, parent_pr = sys.argv[1], sys.argv[2], sys.argv[3]
    assert len(parent_sha) == 40, "pass the full 40-char parent sha"
    d = TICKETS[pq]
    wt = f"{WT}/{d['slug']}"
    branch = f"ai-c11-cc/{d['slug']}"
    text = HEADER.format(
        title=d["title"], pq=pq, t=d["t"], mode=d["mode"], parent=PARENT, parent_sha=parent_sha,
        wt=wt, root=ROOT, tab=d["tab"], oneliner=d["oneliner"], actor=d["actor"],
        body=d["body"], planner_extra="", branch=branch,
    )
    branch_note = f"**Press-ahead ticket.** Your branch starts from PQ-36's in-review branch `{PARENT}` @ `{parent_sha}` (PR #{parent_pr}), not from `main`, because you build on its round-two copy and share `web/buzzer/`. Rebase onto `origin/{PARENT}` while PR #{parent_pr} is open and never after your first push; if PR #{parent_pr} merges first, `git branch --unset-upstream && git merge origin/main` (a plain merge, no rebase) and open or retarget your PR against `main`. The Orchestrator retargets your PR to `main` before PR #{parent_pr} merges."
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", branch_note),
        (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
         f"Open the PR **against `{PARENT}`** (stacked on PR #{parent_pr}): `gh pr create --base {PARENT} --head {branch}`"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line",
         f"the line `Based on #{parent_pr} — merge that first; this PR retargets to main afterwards.`, and the line"),
        ("and never edit the contract files (`SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `sequence/**`, `prototypes/**`, `mvp/**`).",
         "and never edit the contract files (`SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `sequence/**`, `prototypes/**`, `mvp/**`) — **except the ruled exception in §2 for this ticket: `SPEC.md` §5 buzzer cells and the prototype's buzzer at split/work/reveal, this change only.**"),
    ]
    for a, b in reps:
        assert a in text, (pq, a[:70])
        text = text.replace(a, b)
    assert "#8 " not in text and "#8)" not in text and "#8," not in text, "a #8 reference survived"
    out = pathlib.Path(wt) / ".claude" / "boot-prompt.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(text)
    print(out, len(text.splitlines()), "lines")


if __name__ == "__main__":
    main()
