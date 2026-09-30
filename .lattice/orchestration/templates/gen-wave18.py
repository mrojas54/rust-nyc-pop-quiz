"""Wave 18 boot prompts: PQ-26 burst and smoke in CI (inline-full) and PQ-27 copy freeze and lints
(inline-full), both press-ahead off PQ-16's in-review branch `ai-c11-cc/a11y-sweep`.

Why stacked: PQ-26 and PQ-16 both edit the `justfile`'s PENDING line and `test-full`; PQ-27 and
PQ-16 both touch `web/shared`. Branching off the a11y branch makes those additive instead of a
union at merge. Each child PR opens against `main` with a "Based on #N" line and merges after #N.

    python3 gen-wave18.py PQ-26 <origin/ai-c11-cc/a11y-sweep sha> <PQ-16 PR number>
    python3 gen-wave18.py PQ-27 <origin/ai-c11-cc/a11y-sweep sha> <PQ-16 PR number>
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER  # noqa: E402

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"
PARENT = "ai-c11-cc/a11y-sweep"

SIBLINGS = """**Siblings this wave (both off the same a11y branch):** PQ-26 (burst and smoke in CI) owns the `justfile`, `.github/workflows/**`, `room/**` except `room/src/copy.rs` and `room/tests/twins.rs`, and `room/README.md`. PQ-27 (copy freeze and lints) owns `web/shared/**`, `web/test/copy*.test.js` and any new lint test under `web/test/`, string moves inside `web/wall/**`, `web/buzzer/**`, `web/host/**`, `web/home/**`, `room/src/copy.rs`, `room/tests/twins.rs`, `pipeline/src/popquiz/copylint.py` + its tests, `bank/fixtures/copy-lint/**`, and `web/shared/README.md`. Touch none of the other's files; if you must, stop and ask the Orchestrator by comment."""

TICKETS = {}

TICKETS["PQ-26"] = dict(
    t="T-21", slug="burst-in-ci", mode="inline-full", title="Burst and smoke in CI", tab="Burst In CI", actor="pq26",
    oneliner="the burst harness on the room's real protocol, burst and smoke in test-full against a room the job starts, a client-triggered deployed run, and failed-request logging the client can read after a meetup.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `BUILDPLAN.md` **T-21** (line ~205: `burst` and `smoke` in `test-full`, run against the deployed room in CI; failed-request logging the client can read after a meetup; criteria AC-41, AC-52–55; serialized on `justfile`) and the M4 header (~201); `EVALUATION.md` harness rows **`test-full`** (~32), **`burst`** (~36: 200 synthetic participants, the deadline write burst in isolation (AC-54), then a full segment (AC-52, AC-53, AC-41); `test-full`; *also run against the deployed room before each checkpoint*), **`smoke`** (~38), and rows **AC-41** (~103: reveal reaches 200 clients ≤ 2 s at p95; venue wifi is the oracle at HC-4), **AC-52** (~114: each session's final answer counted exactly once), **AC-53** (~115: answer-write p95 < 500 ms including the burst — *run against the deployed substrate, not a local mock*), **AC-54** (~116: the deadline burst as its own test, 200 writes inside a 2 s window, reported separately), **AC-55** (~117: `external-oracle` — venue wifi at HC-4; *the client reports failed-request rate from the room's own logs*; synthetic runs cannot prove it), and `sequence/USER_STORIES.md` AC-41, AC-52…AC-55 (~170–180); `SPEC.md` **§8.3** (exactly two admin routes — you add none), **§8** (only a signed-in organizer creates a room; sessions last 12 h; no check after creation), **§8.2** (the stand-in is gone: AC-64's scan of `room/src`, `room/Cargo.toml`, `Dockerfile` and `.github/workflows/` must stay green — you add no name it would catch and no door in the shipped binary or image), §9, §4.6 (a question runs once per machine until restart); then the code **on your branch**: `room/src/bin/burst.rs` (**read the header, lines 1–25**: the T-03 spike's load client, its four measures, raw samples in the report, `marginal` flagging, JSON on stdout, non-zero exit on a miss, and its own note that *T-21 turns this into the `burst` recipe* — but it speaks the **spike server's** protocol, not the room's: F-27), `room/src/bin/spike_shared.rs`, `room/src/bin/spike-server.rs` (the in-process server the spike's loopback test uses: `#[cfg(test)] mod server` serving the real router on `127.0.0.1:0` — the precedent for a job-started room), **`room/src/bin/smoke.rs`** (lines 1–51: it already speaks the room's protocol end to end — join, sessions, buzzer sockets, every phase, the deadline burst, the secrecy scan; it reads `POPQUIZ_ORGANIZER_SESSION` from the environment and *holds no shared secret that creates rooms*; its timings are smoke's, not `burst`'s), `room/Cargo.toml` (`[features]` ~56–70 `spike`/`smoke`; `[[bin]]` blocks ~35–54 with `required-features`, which is what keeps `just test` inside 60 s and also means neither `test` nor `test-full` type-checks those bins — PQ-3's note), `room/README.md` **Deploying** (~804–890: `just smoke <url>`, *smoke uses up q3*, `fly apps restart` between runs, `POPQUIZ_ORGANIZER_SESSION`), **For T-21's `burst`** (~501–530: the reveal broadcast is the path AC-41 measures), **The burst spike** (~892–920), and *Pipeline channel* (~152: `PUT /admin/questions/{id}` with `POPQUIZ_ADMIN_TOKEN` is how a question gets scheduled on a deployed room; `GET /admin/used`), the `justfile` (`smoke` ~222–227; `burst` ~245 is `_pending`; `PENDING` line 25 — after PQ-16 it names `burst:T-21` alone; `test-full` ~62–70 and its "Ran:" list; `_pending` ~253), `.github/workflows/ci.yml` (the `just test` job ~21–64 with *The smoke client* step ~61–64; the `test-full` job ~69–140 with Docker and the sandbox image), `room/src/ws.rs` and `room/src/routes.rs` (where a failed participant request or a dropped socket is seen), `room/src/main.rs` (tracing setup), `room/tests/canary.rs` + `room/tests/canary_scan/` (**captured log lines are scanned**: nothing you log may carry a token, a session, an answer or a plant), `room/tests/common/mod.rs` (the tests' `HostAuth`).

**BUILDPLAN T-21, verbatim:** `burst` and `smoke` in `test-full`, run against the deployed room in CI; failed-request logging the client can read after a meetup. Criteria AC-41, AC-52–55. Depends on T-09. Serialized on `justfile` (`burst`, `test-full`).

**The contract tension, ruled by the Orchestrator (logged; routed upstream as F-39):** T-21 says *run against the deployed room in CI*, but since T-10 the only thing that creates a room on the deployed app is a human signed in with Discord (a 12-hour organizer session), §8.3 allows exactly two admin routes, and the run has ruled that no shared secret creates rooms. So there is no unattended door to the deployed room, and you build none. The ruling is three runs, each honest about what it proves:

- **(a) `burst` on the room's real protocol.** Either port `burst.rs`'s measurement and report (AC-54 isolated window, AC-53 full-segment p95, AC-41 fan-out p95, AC-52 exact count; raw samples; `marginal`; JSON; non-zero exit) onto smoke's driver, or grow smoke's driver into the harness — take a side and say why. `just burst <url> [--participants N]` reads `POPQUIZ_ORGANIZER_SESSION` exactly as smoke does; the question is scheduled first over the admin channel (document the `PUT`, never a value). `PENDING` loses `burst:T-21` (then it is empty — keep the mechanism, say it holds nothing); the reserved-name header gains the line; `test-full`'s "Ran:" list stays true.
- **(b) In `test-full`, on every PR, unattended:** `burst` and `smoke` run against **a room the job itself starts on the runner** — the real router served in-process on loopback with real sockets, authorized by the tests' `HostAuth` (the spike's `#[cfg(test)] mod server` shape, or a test-only launcher behind `required-features`), the question loaded the way the tests load it. This proves the harness, AC-52's exact count, AC-54's window and AC-41's fan-out mechanics on every change, and records loopback numbers **labelled as loopback**. Nothing here reaches the production binary, the Dockerfile or the image; AC-64's scan stays green; `just test` stays under 60 s (these run in `test-full` only).
- **(c) The deployed run, the client's to start:** a `workflow_dispatch` job (name it, e.g. `deployed-burst`) that takes the URL as an input and reads `POPQUIZ_ORGANIZER_SESSION` and `POPQUIZ_ADMIN_TOKEN` from repository secrets the client sets right before triggering it (the session is 12-hour and personal — the README says to set it, run, then delete it; never echoed, never in a log), schedules the question, runs `just burst --url` and `just smoke --url`, uploads both JSON reports as artifacts, and tells the client to `fly apps restart` before the next run (no Fly token in CI). The laptop form — `POPQUIZ_ORGANIZER_SESSION=… just burst https://rustnyc-popquiz.fly.dev` — stays the documented pre-checkpoint path too. **AC-53's *deployed substrate* clause is met by (c) only**; (b) never claims it.
- **(d) AC-55's logging.** The room emits one structured `tracing` line per failed participant request (a write refused for a server-side reason, a socket dropped mid-phase, any 5xx — not an ordinary refusal such as a closed room or a bad code; say exactly which events count and why) with stable field names and the room id, and one summary line at release: failed and total participant requests for that room. `room/README.md` gains *After a meetup*: the `fly logs` recipe that reads the rate. The canary's log-line scan stays green: no token, session, answer or plant in any line.

**Deliverables:** the harness and recipe (a); the in-process run wired into `test-full` and the CI `test-full` job (b), with the *The smoke client* step in the `just test` job kept or folded — say which; the `workflow_dispatch` job and its README section (c); the logging and its README section (d); tests in `test` for the report's shape and the pass/miss/marginal decision on recorded samples (no sockets in `test`); `just test-room` warm under 60 s, numbers noted.

**Exit check before DONE:** `just test-full` green locally with the in-process burst and smoke in its "Ran:" list and their loopback numbers printed; CI green on the PR head for both jobs; the `workflow_dispatch` job present and its dry syntax validated (`gh workflow view` or a `workflow_dispatch` on the PR branch if the client triggers it — otherwise say untriggered); one deployed run recorded on the ticket if the client runs it while you are open (numbers, never a token), else stated as pending; AC-64's scan green; canary fast and full green; `PENDING` empty.

""" + SIBLINGS + """

**Out of scope:** any third admin route; any token or shared secret that creates rooms; a door in the production binary or image; the phase machine, sessions, answers, Discord; `web/**`; the pipeline; venue wifi (HC-4).

**Shared files cleared for this ticket:** the `justfile` (serialized to you this wave: the `burst` recipe, its header line, `PENDING`, `test-full`), `.github/workflows/ci.yml` and a new workflow file for the dispatch job, `room/src/bin/burst.rs`, `room/src/bin/smoke.rs`, `room/src/bin/spike_shared.rs`, `room/src/bin/spike-server.rs`, `room/Cargo.toml` and `room/Cargo.lock` (features and bins; a new crate needs one `cd room && cargo fetch` with the bypass, asked once — prefer what the lock holds), `room/src/main.rs`, `room/src/lib.rs` (additive), `room/src/routes.rs` and `room/src/ws.rs` (the logging lines only), `room/tests/**` except `twins.rs`, `room/README.md`. **Not** `room/src/copy.rs`, `room/tests/twins.rs` (PQ-27's), `web/**`, `pipeline/**`, `Dockerfile`, `fly.toml`, `.env.example` (no new name — say so if one is needed and ask).""",
)

TICKETS["PQ-27"] = dict(
    t="T-22", slug="copy-freeze", mode="inline-full", title="Copy freeze and lints", tab="Copy Freeze", actor="pq27",
    oneliner="every participant-facing string in the copy module, the forbidden-copy lint and the trope check as one-source tools in test (JS over the module, a Python twin over question prose), and the proof that nothing asks for a contribution.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `BUILDPLAN.md` **T-22** (line ~206: the copy freeze — every participant-facing string from `SPEC.md` §11 in one module; the forbidden-copy lint and the §11.1 trope check in `test`, the trope check failing the build on any match in the copy module (D-23); criteria AC-98, AC-59, AC-42, G-5); `SPEC.md` **§11 whole** (~596–640: authored there and only there; the lint runs over the *ported copy module* — every authored participant-facing string as its own entry, taken from the strings and not from row labels or notes; the **Forbidden row** ~639 is the one normative list, with its two allowed near-misses), **§11.1 whole** (~641–672: the trope check, its two homes and two consequences — the copy module fails the build, question prose warns on the review screen — and the five pattern groups verbatim), **G-5** (~62), §7.4 (the review screen, T-18 — not built yet); `sequence/USER_STORIES.md` **AC-98** (~278: nothing obliges a participant to speak or interact; no count, prompt or absence-of-response displayed), **AC-42** (~154, `felt`), **AC-59** (~191, `felt`: the wrap-up states what was and was not recorded); `EVALUATION.md` rows **AC-98** (~146: the lint using *exactly* the Forbidden row's patterns over every participant-facing string; *no UI counts or waits for a contribution*), **AC-42** (~104: over the copy module a match fails; the retired-strings fixture proves every pattern group fires; over `explains`, `why_tempting` and `hint` it produces warnings on the review screen and never a failure), **AC-59** (~121); then the code **on your branch**: **`web/shared/copy.js`** (`COPY`, `COPY_ROWS`, `t()`; the rows added since §11 was written — *Host, sign-in and denials (T-10, F-33)*, the wordmark row — are client-approved and stay), **`web/test/copy.test.js`** (lines 1–30: its own scope note says *T-22 owns the forbidden-copy lint and the trope check as reusable tools*; lines 150–271 hold PQ-2's transcription of the Forbidden row and the §11.1 groups plus the retired-strings fixture — your one source replaces that transcription), `room/src/copy.rs` + `room/tests/twins.rs` (the Rust mirror pinned both ways; the room puts strings on the wire through `room/src/view.rs`), **`web/home/home.js`** (~41–60: `PROPOSED` — strings this page needs that §11 does not author; F-34 routed upstream, words unchanged), `web/shared/trace.js` (~134–137: the step buttons' `aria-label` literals *previous step* / *next step* — F-38 upstream), `web/wall/wall.js`, `web/buzzer/buzzer.js`, `web/host/host.js`, `web/home/home.js`, `web/wall/fallback/static.js` (every literal that reaches a participant, an `aria-label` included; find them, do not assume), `web/shared/dom.js` (`announce` — the live-region strings are §11's), `pipeline/src/popquiz/bank.py` (`explains`, each option's `why_tempting`, `hint`), `pipeline/src/popquiz/receipt.py` (`FORBIDDEN_IN_A_LINE` ~44 is D-25's receipt rule, a different lint — leave it), `pipeline/src/popquiz/review.py` (T-18's surface; you provide the function it will call, you do not build the screen), `bank/fixtures/receipts/` (the one-fixture-two-languages shape you mirror).

**BUILDPLAN T-22, verbatim:** The copy freeze: every participant-facing string from `SPEC.md` §11 in one module; the forbidden-copy lint and the §11.1 trope check in `test`, the trope check failing the build on any match in the copy module (D-23). Criteria AC-98, AC-59, AC-42, G-5. Depends on T-05–07, T-12.

**Deliverables:**

- **One source for the patterns.** A data module (say `web/shared/copylint.js`) exporting the Forbidden row's patterns and the five §11.1 groups **verbatim from SPEC**, and a Python twin (`pipeline/src/popquiz/copylint.py`, stdlib `re` only) holding the same patterns; one fixture set under `bank/fixtures/copy-lint/` (the seven retired strings from AC-42's row, the two allowed near-misses, one clean string per group) read by both suites so the two can never drift (the `receipt` precedent). Python's `re` and JavaScript regexes differ on `\\b` with non-ASCII and on lookarounds — prove equivalence on the fixtures, and say where they cannot be identical.
- **The freeze.** Every participant-facing string under `web/` lives in `web/shared/copy.js` and reaches a page only through `PQ.t` (or the module's export): a structural test walks the render paths of wall, buzzer, host, home and the static fallback and fails on a string literal that is not a module key — `aria-label`s and live-region strings included — with an allowlist that is empty or justified entry by entry. `home.js`'s `PROPOSED` strings and `trace.js`'s two `aria-label`s move **into the module under a marked `PROPOSED-§11` section, words unchanged** (F-34, F-38 stay upstream; the lints now cover them). `room/src/copy.rs` and `twins.rs` follow the module.
- **The lints, in `test`.** Over the copy module: any Forbidden match fails; any trope match fails (D-23); the retired-strings fixture proves every group fires; the near-misses stay allowed. Over question prose: `copylint.check_prose(record) -> list[Warning]` (field, group, pattern, the matched text) that **never raises** — the review screen (T-18) will render it; a pipeline test runs it over every record in `bank/questions/` and prints the warnings (a match there is information, not a failure). `just test` picks both suites up by glob (`web/test/*.test.js`, pytest discovery) — no `justfile` edit (PQ-26 holds it); say so.
- **AC-98's second half.** A test over the rendered surfaces that no control, count, prompt or absence-of-response asks for a contribution: no button or copy keyed to speaking, comparing or volunteering; the host's counts are joined/answered totals, never *who*; take-it-home carries no count. Name what you checked and how.
- **AC-59.** The released buzzer's *nothing about you was recorded* line is a module key, served at release, asserted in a test; nothing else (the `felt` half is HC-1/HC-3).
- **PQ-4's note for T-22:** the room's refusal `reason` values are API diagnostics, not §11 copy, and the buzzer renders refusals from §11 keys by code — confirm by test that no `reason` string reaches a page verbatim; if one does, it is a finding (QUESTION), not a string you author.
- **Docs.** `web/shared/README.md` *Copy*: §11 first, then the key, then the mirror; how the two lints run and what each consequence is.

**Copy:** you author no string and change no words. A string §11 lacks is a QUESTION on the ticket and a `PROPOSED-§11` entry, never a new sentence.

**Exit check before DONE:** `just test-web` and `just test-pipeline` green under 60 s warm with numbers (if `test-pipeline` cannot start on this machine, say so and cite CI); the structural freeze test green with its allowlist printed; the fixture equivalence proven in both languages; twins green; `just canary` green; `git diff origin/main --stat` shows no `room/src` file but `copy.rs`.

""" + SIBLINGS + """

**Out of scope:** the review screen (T-18 / PQ-23); any wording change; the receipt's own rule (D-25); `justfile`; CSS; the room crate beyond the mirror.

**Shared files cleared for this ticket:** `web/shared/copy.js` (serialized to you), new `web/shared/copylint.js`, `web/shared/README.md`, `web/test/copy.test.js` (rewrite around the one source) and new tests under `web/test/`, `web/home/home.js`, `web/shared/trace.js`, `web/wall/wall.js`, `web/buzzer/buzzer.js`, `web/host/host.js`, `web/wall/fallback/static.js` (**only** to replace a literal with a module key — no behaviour change; PQ-16's a11y suite on your base branch must stay green), `room/src/copy.rs`, `room/tests/twins.rs`, new `pipeline/src/popquiz/copylint.py` and `pipeline/tests/test_copylint.py`, new `bank/fixtures/copy-lint/**`. **Not** the `justfile`, `pyproject.toml` (stdlib only), `.github/**`, `room/**` beyond the two files, `web/**/*.css`, `SPEC.md`.""",
)


def main():
    pq, parent_sha, parent_pr = sys.argv[1], sys.argv[2], sys.argv[3]
    assert len(parent_sha) == 40, "pass the full 40-char parent sha"
    assert parent_pr.isdigit(), "pass PQ-16's PR number"
    d = TICKETS[pq]
    wt = f"{WT}/{d['slug']}"
    branch = f"ai-c11-cc/{d['slug']}"
    text = HEADER.format(
        title=d["title"], pq=pq, t=d["t"], mode=d["mode"], parent=PARENT, parent_sha=parent_sha,
        wt=wt, root=ROOT, tab=d["tab"], oneliner=d["oneliner"], actor=d["actor"],
        body=d["body"], planner_extra="", branch=branch,
    )
    branch_note = (
        f"**Press-ahead ticket.** Your branch starts from PQ-16's in-review branch `origin/{PARENT}` @ `{parent_sha}` "
        f"(PR #{parent_pr}, the accessibility sweep, not yet merged), because you share files with it (`justfile`, `web/shared`). "
        f"Rebase onto `origin/{PARENT}` before your first push and never after it. Your PR opens **against `main`** with the line "
        f"`Based on #{parent_pr} — merge that first.` in its body; while #{parent_pr} is open your PR shows its diff too, which is expected. "
        f"**When #{parent_pr} merges** (the Orchestrator tells you by comment): `git branch --unset-upstream` if set, `git merge origin/main` "
        f"as a plain merge commit (never a rebase after your first push), re-run `just test`, push. A sibling runs this wave; its files are named in §2."
    )
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", branch_note),
        (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
         f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line",
         f"the line `Based on #{parent_pr} — merge that first.`, and the line"),
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
