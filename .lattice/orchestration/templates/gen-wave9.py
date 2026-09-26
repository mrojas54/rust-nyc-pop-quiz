"""Wave 9 boot prompts: the last two M1 tickets, inline-full.

    python3 gen-wave9.py PQ-12 ai-c11-cc/wall <sha> '#21'     # press-ahead off the wall (PR #21) while it is at review
    python3 gen-wave9.py PQ-12 main <sha>                      # or off main once #21 is merged
    python3 gen-wave9.py PQ-10 main <sha>                      # after #19..#23 are on main (it scans the assembled tree)

HEADER (header.py) carries the learned clauses; nothing is appended here.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER  # noqa: E402

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"

TICKETS = {}

TICKETS["PQ-12"] = dict(
    t="T-26", slug="static-fallback", mode="inline-full", title="Static fallback and host sheet", tab="Static fallback", actor="pq12",
    oneliner="the static fallback: the wall in mode static as one offline file with a keyboard driver, and the host-sheet function beside it.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` §12 whole (verbatim: what is built and by whom; the host sheet), §5 (the wall you reuse), §4 (the phase order and the `work`/`reveal` rules), §7.5 (the receipt as a step list), §11; `EVALUATION.md` AC-102, AC-97, AC-99 and the `test-full` row; `sequence/USER_STORIES.md` AC-102, AC-97, AC-99, AC-77; `sequence/run-state.md` D-14, D-21, D-24, D-25; `BUILDPLAN.md` the T-26, T-20 and T-05 rows; then the code on your branch: `web/wall/**` (PQ-7's page — the phase rendering is a function of a state object by design, so `mode: "static"` drives it; read its README notes), `web/shared/**` (`fonts.css` and the vendored `fonts/*.ttf` you inline, `tokens.css`, `components.css`, the scripts), `room/README.md` (*Pages*), `bank/questions/q3.json` and `bank/schema/question.schema.json` (the record you bake in), `pipeline/src/popquiz/` (`bank.py` — the record loader; `schedule` does not exist yet, T-20 calls your function later, so you ship the function and a CLI entry), `pipeline/tests/`.

**BUILDPLAN T-26, verbatim:** The static fallback: `mode: "static"` in the wall (no socket, the phase held locally, the question baked in), the `Space`/`←`/`→`/`Esc` driver, and a one-file build with fonts and tokens inlined that makes no network request; the same fixture renders identically in both modes; and the **host sheet** function, a question record to plain text, that T-20 calls beside the build (`SPEC.md` §12, D-21, D-24). Criteria AC-102, AC-97, AC-99. Depends on T-02, T-05. Not part of HC-0's trigger, but the client opens the file offline at HC-0 and reads the sheet against it.

**Deliverables:**

- **`mode: "static"` in `web/wall/`:** no WebSocket, the phase in a local variable, the question record baked in as a literal, the join strip omitted; `Space` next phase, `←`/`→` step the trace inside `work` and `reveal`, `Esc` back a phase; the same phase rules as the live wall (`work`: no ✓, no receipt, no colour, steps `0..M-2` only; `reveal` enters at `M-1`; AC-97, AC-99). The receipt lines come from the baked record the same way the live wall reads them from the payload (the room's `answers::receipt_lines` is the one function, G-7): bake the lines the pipeline produces — add a small helper in `pipeline/src/popquiz/` that renders a record's step list through `popquiz.receipt` (the Python twin) so the two never diverge, and pin it against `bank/fixtures/receipts/`.
- **The one-file build:** a `pipeline` function and CLI (`popquiz fallback <question-id> --out <file>` or the shape `pyproject.toml` suggests) that inlines the vendored fonts (base64 `@font-face`, D-14), tokens, components and the wall code into one HTML file that makes **no network request of any kind**. The same fixture renders identically in live and static mode (a test that diffs the rendered DOM of both modes on q3's fixture, minus the join strip).
- **The host sheet function:** a question record to plain text, one block per phase in the room's phase order, holding the `explains` beats and each trace step's `note` verbatim, nothing added (AC-72, AC-95); written beside the fallback file by the same CLI (`<name>.host-sheet.txt`). Never served by any room route (assert in a room test that no route path contains `sheet` or `fallback`).
- **Tests, in `test`:** `web/test/wall-static.test.js` (driver keys walk the seven views in order; `Esc` goes back; `work` and `reveal` bounds; both modes render the same DOM on the fixture), `pipeline/tests/test_fallback.py` (the built file holds no `http://`, `https://`, `//`-scheme or `ws://` reference outside the baked record's own strings — say how you exclude those; fonts present as data URIs; the sheet's lines equal the record's beats and notes verbatim, in phase order). In `test-full`: open the built file in the c11 browser with network blocked (or a headless browser if one exists; say which) and drive the seven views by keyboard, recording the run in `room/README.md` beside PQ-7's measured-layout note (AC-102's `felt` half is HC-0, the client's).
- `justfile`: only if a recipe is needed for the `test-full` browser run (read the header comment; PQ-10 is not touching it this wave); `pyproject.toml` only for the CLI entry point.

**Out of scope:** the schedule and sync (T-20 / PQ-25, which calls your function); the live wall's own behaviour (PQ-7, at review — if you find a defect there, comment, do not fix); take-it-home (T-12); deploy; `web/shared` edits.

**Shared files cleared for this ticket:** `web/wall/**` (static mode and the driver; keep PQ-7's live behaviour intact — every existing wall test stays green), `web/test/wall-static.test.js` (new), `pipeline/src/popquiz/fallback.py` (new) and `pipeline/src/popquiz/hostsheet.py` (new) or one module — your call, `pipeline/tests/test_fallback.py` (new), `pyproject.toml` (the CLI entry only), `room/README.md` (one *Pages* line and the `test-full` run note), `justfile` (one `test-full` line at most). Not `room/src/**`, not `web/shared/**`, not `bank/**`, not `tests/common/mod.rs`.""",
)

TICKETS["PQ-10"] = dict(
    t="T-08", slug="canary", mode="inline-full", title="Canary secrecy suite", tab="Canary", actor="pq10",
    oneliner="the canary: plant secrets in the answer, explanation, receipt and pre-live hint, then scan every payload, frame and page in every phase.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `PHILOSOPHY.md` §2; `SPEC.md` §2 (G-3, G-4, G-8 verbatim), §4 (the phase table: what each viewer may see when), §4.2 (the hint), §4.3 (post-close traffic), §8 (`POPQUIZ_ADMIN_TOKEN`), §9; `EVALUATION.md` AC-32, AC-47, AC-48, AC-58, AC-60, AC-79, AC-97, AC-101 and the `canary` row in the harness table; `sequence/USER_STORIES.md` the same IDs; `BUILDPLAN.md` the T-08 row (in-process scan in `test`; the deployed-room scan in `test-full`; serialized on `justfile`) and T-25's row (the token scan it shares); then the code on `main`: `room/tests/canary.rs` whole (T-04a's seam and its module doc: the plants it uses, `scan_pre_reveal`, `scan_every_phase`, `drive_and_scan`, and what it says T-08 replaces), `room/tests/common/mod.rs` (`planted()`, `planted_dnc()`), `room/tests/wiring.rs`, `room/tests/transport.rs`, `room/tests/wall_page.rs`, `room/tests/host_page.rs`, `room/tests/buzzer_page.rs` (the pages' own no-leak assertions — yours subsumes them), `room/src/view.rs`, `room/src/ws.rs`, `room/src/routes.rs` (every route is a path to scan), `web/wall`, `web/host`, `web/buzzer` (the page JS: what state it holds and what it fetches), the `justfile` (`canary` recipe today; the header comment on reserved names).

**BUILDPLAN T-08, verbatim:** `canary`: plant secrets in answer, explanation, receipt, hint-before-live; scan every payload, frame, and page in every phase; post-close traffic carries no answer. Criteria AC-32, AC-47, AC-48, AC-58, AC-60, AC-79, G-3, G-4, G-8. Depends on T-04a–07.

**Deliverables:**

- **The plants:** a question fixture whose correct answer text, `explains` beats, `why_tempting`, receipt-bearing values (`stdout`, `exit_code`, error code), hint, source and trace notes each carry a distinct unguessable canary string (extend `planted()` or add `canary_question()` in `tests/common/mod.rs` under a `// T-08` comment), plus `POPQUIZ_ADMIN_TOKEN` set to a fifth plant in the test environment.
- **The scan, in `test` (`just canary`, kept inside `just test`'s budget):** drive a room through **every phase in order** with a wall, a host and several buzzers attached over the loopback listener (the `Live` harness), and at every revision scan **everything a client can receive**: each HTTP projection, each socket frame per viewer, each served page's HTML and the JS it loads, and the response bodies of every route in `routes.rs` (walk the router's paths; a route you cannot drive is listed in the README as unscanned). Assert: before `reveal`, no plant from the answer, explanation, receipt or `why_tempting` anywhere for any viewer (AC-47, AC-60, G-3); the hint plant absent before `live` and present in the live buzzer payload only (AC-48); the source and trace plants never in any buzzer payload, frame or page in any phase (AC-32, G-8); at `reveal` the answer plants **do** appear on the wall and host (the positive control, so a silent no-op cannot pass); after `closed`, no request the buzzer page makes carries the participant's answer (AC-58 — intercept the page's fetches/frames in the test harness or assert on the server's request log); the wall's HTML has no interactive control (AC-79); `work` carries no ✓, receipt, colour, `stdout` value or step beyond `M-2` (AC-97); the admin token plant appears in **no** payload, page or frame in any phase (AC-101's canary half; T-25 owns the repo scan).
- **`test-full`:** the same scan against a running server on loopback through real sockets over several transitions, plus the reconnect path (drop and re-attach a buzzer mid-`closed`: the attach frame carries only that session's own saved letter). The deployed-room variant is T-09's `smoke`; leave the hook (a documented `--url` flag) and say so.
- **`justfile`:** the `canary` recipe runs the real scan (replace the "in-process scan only" echo); keep the reserved-name header honest.
- `room/README.md`: a *Canary* section: what is planted, what is scanned, what is listed as unscanned and why.

**Out of scope:** fixing a leak you find — a leak is a `needs_human` stop with the exact payload and phase in the comment (the Orchestrator routes it to the owning ticket); the repo-wide token scan (T-25); the a11y sweep (T-13); deploy and `smoke` (T-09).

**Shared files cleared for this ticket:** `room/tests/canary.rs` (yours to rewrite), `room/tests/common/mod.rs` (additive, under `// T-08`), `room/tests/canary_full.rs` (new), the `justfile` (`canary` recipe and its `test-full` line only), `room/README.md` (the *Canary* section). Not `room/src/**`, not `web/**`.""",
)


def main():
    pq, parent, parent_sha = sys.argv[1], sys.argv[2], sys.argv[3]
    assert len(parent_sha) == 40, "pass the full 40-char sha"
    d = TICKETS[pq]
    wt = f"{WT}/{d['slug']}"
    branch = f"ai-c11-cc/{d['slug']}"
    text = HEADER.format(
        title=d["title"], pq=pq, t=d["t"], mode=d["mode"], parent=parent, parent_sha=parent_sha,
        wt=wt, root=ROOT, tab=d["tab"], oneliner=d["oneliner"], actor=d["actor"],
        body=d["body"], planner_extra="", branch=branch,
    )
    old_press = f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{parent}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{parent}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges."
    if parent == "main":
        branch_note = f"**Branch.** Your branch starts from `origin/main` @ `{parent_sha}` (M0 complete; M1's phase machine, sessions, transport, wiring, wall, buzzer and host phone merged). Rebase onto `origin/main` before implementing and again before your first push, never after it. Your PR opens against `main`."
        reps = [
            (old_press, branch_note),
            (f"Open the PR **against `{parent}`** (stacked on #8): `gh pr create --base {parent} --head {branch}`", f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
            ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line", "the line"),
        ]
    else:
        pr = sys.argv[4]
        branch_note = f"**Press-ahead ticket.** Your branch starts from the in-review wall branch `{parent}` @ `{parent_sha}` (PR {pr}: PQ-7, the wall page and the `/shared` asset routes you build on), not from `main`. Rebase onto `origin/{parent}` while {pr} is open (the Orchestrator will tell you if it moves); the Orchestrator retargets your PR to `main` once {pr} merges."
        reps = [
            (old_press, branch_note),
            (f"Open the PR **against `{parent}`** (stacked on #8)", f"Open the PR **against `{parent}`** (stacked on {pr})"),
            ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`", f"the line `Based on {pr} — merge that first; this PR retargets to main afterwards.`"),
        ]
    for a, b in reps:
        assert a in text, (pq, a[:70])
        text = text.replace(a, b)
    assert "#8" not in text, "a #8 reference survived"
    out = pathlib.Path(wt) / ".claude" / "boot-prompt.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(text)
    print(out, len(text.splitlines()), "lines")


if __name__ == "__main__":
    main()
