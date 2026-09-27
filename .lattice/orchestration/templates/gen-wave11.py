"""Wave 11 boot prompt: PQ-34 HC-0 copy trim, fast-track, off `main`.

    python3 gen-wave11.py PQ-34 <origin/main sha>

Minted 2026-09-27 from the client's first HC-0 findings during the deployed drive
("remove any text that is absolutely unnecessary. no need to narrate the demo.
intuitive workflow rather than guided"). Same shape as wave 10; the one new thing
is a ruled exception to clause 7: this ticket edits the contract's copy rows and
the prototype's strings, because the client ruled those strings off and the run
keeps the copy contract and the build one thing.
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

TICKETS["PQ-34"] = dict(
    t="HC-0 finding", slug="copy-trim", mode="fast-track", title="HC-0 copy trim: strip narration from the buzzer and wall", tab="Copy Trim", actor="pq34",
    oneliner="the buzzer, wall and host stripped of narration copy per the client's HC-0 ruling, with SPEC §11 and the prototype amended to match.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `PHILOSOPHY.md` (the one thing and the principles — the ruling below is an application of them, not an exception); `SPEC.md` §5 (the phase table: every surface's copy per phase, lines ~170–185) and §11 whole (the copy contract, lines ~600–640); `DESIGN.md` (voice); `sequence/USER_STORIES.md` AC-40 (colour is never the only signal), AC-45, AC-49, AC-93, AC-94, AC-97 (the felt criteria HC-0 judges); `prototypes/C-projector-first.html` (the binding visual contract; the strings it renders, around lines 740–790 for the buzzer); then the code on `main`: `web/shared/copy.js` whole (every string, keyed; this is the single source of copy for wall, buzzer and host), `web/test/copy.test.js` (how strings are pinned), `web/buzzer/`, `web/wall/`, `web/host/` (which keys each surface renders, and where a removed line leaves a layout gap), `web/wall/fallback/` (the static fallback renders the same DOM — PQ-12), and on the board: `(cd "$LATTICE_ROOT" && lattice comments PQ-34)` (the client's findings so far, each a comment; more may land while you plan — re-read at every phase change).

**The client's ruling (HC-0, 2026-09-27, verbatim):** *"remove any text that is absolutely unnecessary. no need to narrate the demo. intuitive workflow rather than guided."* Two named instances so far: the buzzer's reveal sentence *The host is reading out the why now.* (`buzzer_reveal_host_reading`, `buzzer_noanswer_reveal`) and the buzzer footer *no account · no name · no score* (`buzzer_foot`). The screenshot that prompted the first: a phone at reveal showing *Look up.* / a tall empty space / *You didn't answer.* / *✓ It was E.* / the narration line.

**Ruled exception to clause 7, for this ticket only:** you **do** edit `SPEC.md` §5 and §11 and `prototypes/C-projector-first.html`, and only to remove or shorten the rows and strings the client has ruled off — never to add copy, never to touch anything else in those files. The run keeps the copy contract and the build one thing, so a removal lands in `copy.js`, the surface that renders it, its test, the §11 row, the §5 cell and the prototype's string in the **same commit**. `DESIGN.md`, `PHILOSOPHY.md`, `EVALUATION.md`, `BUILDPLAN.md` and `sequence/**` stay untouched. If you think a line should go but it carries a criterion's visible form (search `sequence/USER_STORIES.md` for its words), keep it and list it under deviations instead.

**Deliverables:**

- **The named removals** (both instances above), everywhere each appears: `copy.js`, the rendering surface, the test, §11, §5, the prototype. Where a removed line leaves a layout gap or a dangling separator, close the gap; the surface's remaining lines keep their type sizes and spacing from the prototype.
- **The sweep.** Walk every key in `copy.js` for the buzzer, the wall and the host and classify each line as one of: **state** (a fact the reader needs now — the room code, the join address, the count, *✓ It was Y.*, *You didn't answer.*, the phase's one action on the host), **narration** (what someone else is doing, what happens next, reassurance, commentary — *We're walking it through.*, *The answer is on the screen.*, *Nothing to do. Nobody knows the answer yet.*, *Everything happens on the screen at the front — look up.* are candidates; decide each on its own), or **brand** (*Time for a pop quiz.*, the brand line — keep). Write the proposed removal list into your plan, one line per key with the current text and your reason, then **print that list in your tab and raise a c11 flag** (`c11 raise-flag --surface "$MY_SURF" "PQ-34: removal list ready for your yes in this tab"`) and wait for the client's answer in the tab; they may strike or add lines. Only then implement the sweep. Lines the client did not approve stay.
- **What never goes:** any line that is the only signal of a state (AC-40 for colour applies to words too: if the ✓ glyph and the letter are the signal, the sentence around them may go, the glyph and letter may not); the room code; the join address and the short link; the counts; the host phone's action labels; error and refusal lines (§11's refusal rows, PQ-8's six refusals, the reconnect lines); the take-it-home link.
- **Tests:** `web/test/copy.test.js` pinned to the new set; any snapshot or DOM test in `web/test/` that rendered a removed line; the static fallback's build (`python -m popquiz.fallback q3 --out …` must still produce the same DOM as the live wall — run it and compare, PQ-12's method). `just test-web` and `just test-room` green (the room embeds the pages with `include_str!`, so its page tests may pin strings too — `rg` for each removed string across `room/` before you call it done).

**Exit check before DONE** (the PR body carries the evidence): `rg -n "<each removed string>"` across the repo returns nothing outside `.lattice/` and git history; the client's approved list matches the diff one for one; `just test-web`, `just test-room` green with warm times; a screenshot or DOM dump of the buzzer at reveal and idle, the wall at reveal, and the host at reveal, after the change, attached as validation; the static fallback rebuilt and compared.

**Out of scope:** any new copy; layout changes beyond closing the gaps a removal leaves; the wall's typographic model; the sealed module, the phase machine and every `room/src/*.rs` file except test files that pin strings; `pipeline/**` except the fallback comparison run; the copy-lint machinery (T-22, PQ-27 — note under deviations anything it should later enforce).

**Shared files cleared for this ticket:** `web/shared/copy.js` (the serialized shared file — cleared to you alone this wave; no sibling runs), `web/buzzer/**`, `web/wall/**` (including `fallback/`), `web/host/**`, `web/test/**`, `room/tests/**` (only assertions on removed strings), `SPEC.md` §5 and §11 rows and `prototypes/C-projector-first.html` strings under the ruled exception above. Not `justfile`, not `web/shared/tokens.css`, `components.css`, `typemodel.js`, `phase.js`, not `room/src/**`.""",
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
    branch_note = f"**Branch.** Your branch starts from `origin/main` @ `{parent_sha}` (every M1 ticket but the deploy, PR #26, is merged; #26 touches no file you are cleared for). Rebase onto `origin/main` before implementing and again before your first push, never after it. Your PR opens against `main`. No sibling runs this wave."
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", branch_note),
        (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
         f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line", "the line"),
        (PLAN_REVIEW.replace("{pq}", pq), FAST_TRACK_PLAN.replace("{pq}", pq)),
        ("and never edit the contract files (`SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `sequence/**`, `prototypes/**`, `mvp/**`).",
         "and never edit the contract files (`SPEC.md`, `EVALUATION.md`, `BUILDPLAN.md`, `DESIGN.md`, `PHILOSOPHY.md`, `sequence/**`, `prototypes/**`, `mvp/**`) — **except the ruled exception in §2 for this ticket: `SPEC.md` §5/§11 rows and the prototype's strings: removals, plus the one rewrite the ticket's comments rule (the host phone's not-a-guarantee sentences and its title).**"),
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
