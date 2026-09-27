"""Wave 12 boot prompt: PQ-35 README in the client's voice, with a demo; fast-track, off `main`.

    python3 gen-wave12.py PQ-35 <origin/main sha>

Minted 2026-09-27 on the client's "add a ticket to make readme sound more like me, add demo".
Same shape as waves 10–11. No contract exception: README.md is not a contract file.
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

TICKETS["PQ-35"] = dict(
    t="client request", slug="readme-voice", mode="fast-track", title="README in the client's voice, with a demo", tab="README Voice", actor="pq35",
    oneliner="the root README rewritten in the client's own voice, with a demo section of real screenshots and the public entry points.",
    body="""Read, in this order, before planning: `CLAUDE.md` (the house rules — *never write down what a program prints* applies to every command you quote); the **client's writing-voice profile**: load it with the Skill tool as `anthropic-skills:my-writing-style` (it is the reference for tone, sentence shape, what she never says); `README.md` at the repo root (the file you rewrite — every fact, command and path in it stays true; you change how it reads, not what it says); `PHILOSOPHY.md` §1 (the one thing, so the README leads with it in her words); `room/README.md` *Running it* (the local stand-in run you will use for the screenshots; do not edit that file); `web/README.md` and `pipeline/README.md` (technical, stay as they are; the root README links to them); `mvp/README.md` (the hand-run deck; the root README says what it is in one line); `web/shared/copy.js` on `main` after PR #27 (the shipped copy — the screenshots must show it); and on the board `(cd "$LATTICE_ROOT" && lattice comments PQ-34)` (what the copy trim changed and why, so the README's description of the surfaces matches).

**The client's request (2026-09-27, verbatim):** *"make readme sound more like me, add demo."*

**Deliverables:**

- **The voice.** Rewrite the prose of `README.md` in the client's voice per the profile: her sentence length, her way of opening, what she leaves out. Keep the structure a reader needs (what it is, how to run it, where things live), but the words are hers. No marketing register, no em-dashes if the profile says so, no narration. Every command in the file is run by you on this branch before it is quoted, and quoted exactly; every path is checked with `test -e`. Nothing invented: if a fact is not in the code or the contract, it does not go in.
- **The demo section**, near the top, under a heading in her voice: (1) the public entry points of the live room — join at `/join`, the short link `/{code}`, the wall at `/wall/{room_id}` — described, with the live host `https://rustnyc-popquiz.fly.dev` named once; **never the host URL, never `/host?question=…`, never the token or its name as a URL**; (2) **screenshots** of the wall, the buzzer and the host phone, at idle and at reveal, captured from a **real local run** of the stand-in (`cd room && HOST_DEV_TOKEN=<any value> cargo run --features dev-host-token`, then drive a room through to reveal with a couple of buzzer tabs) with the c11 browser (`c11-browser` skill; its screenshot capability) or a headless browser already on this machine — no new dependency, no mocked or hand-edited images; stored under `docs/demo/` as PNG, each well under 500 KB (resize, do not compress into artefacts), with alt text that says what the reader is looking at; (3) a **try-it walkthrough** of five lines or fewer: open the wall, join from a phone, answer, reveal. Note in one line that the deployed room's copy updates when `main` is next deployed (`fly deploy`, the client's).
- **Links** from the root README to `room/README.md`, `web/README.md`, `pipeline/README.md`, `mvp/README.md` and the contract files (`PHILOSOPHY.md`, `SPEC.md`), each checked.

**Exit check before DONE** (the PR body carries the evidence): a Sonnet review with two questions only — *does every sentence sound like the profile?* and *is every fact, command and path true on this branch?* — Verdict PASS required on both; the list of commands you ran with their exit codes; `docs/demo/` sizes listed; `rg -n "host\\?question|HOST_DEV_TOKEN=[0-9a-f]" README.md docs/` returns nothing; `just test` still green (you touched no code, prove it anyway).

**Out of scope:** `room/README.md`, `web/README.md`, `pipeline/README.md`, `mvp/**`, any code, any contract file, the design-system doc, deploying.

**Shared files cleared for this ticket:** `README.md` and `docs/demo/**` (new). Nothing else. If the README needs a fact the code does not have, say so under deviations rather than adding it.""",
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
    branch_note = f"**Branch.** Your branch starts from `origin/main` @ `{parent_sha}` (M1 complete, the copy trim PR #27 merged). Rebase onto `origin/main` before implementing and again before your first push, never after it. Your PR opens against `main`. No sibling runs this wave."
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
