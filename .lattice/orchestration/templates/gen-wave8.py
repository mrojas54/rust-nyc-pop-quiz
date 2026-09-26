"""Wave 8 boot prompts: the three M1 surfaces, inline-full.

    python3 gen-wave8.py PQ-7 main <origin/main sha>
    python3 gen-wave8.py PQ-9 main <origin/main sha>
    python3 gen-wave8.py PQ-8 ai-c11-cc/wire-sessions <origin/ai-c11-cc/wire-sessions sha>   # press-ahead off PQ-32

HEADER (header.py) carries the learned clauses; nothing is appended here.
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER  # noqa: E402

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"

SPLIT = """
**The wave-8 split (Orchestrator ruling, binding).** Three surfaces build in parallel: PQ-7 the wall, PQ-8 the buzzer, PQ-9 the host phone. To keep the three PRs mergeable as unions:
- **Page paths.** The JSON projections stay where they are (`GET /rooms/{id}/wall|buzzer|host`, the `/ws/…` sockets, `POST /join`, `PUT /rooms/{id}/answer`, the host action routes). Pages are new paths: the wall at **`/wall/{room_id}`**, the host phone at **`/host/{room_id}`** (its first screen, *Create a room*, at **`/host`**), the buzzer at **`/join`** (the short link carries the code as `?code=`; typing the code is the same page). Each ticket registers **only its own page route(s)** in `room/src/routes.rs`, inside one delimited block (`// T-05 pages`, `// T-06 pages`, `// T-07 pages`), placed **above** the `// T-04c routes` block, which stays last. Never touch another ticket's block or `lib.rs`.
- **Files.** Each page's HTML, CSS and JS live under `web/<wall|buzzer|host>/` and are served by embedding at compile time (`include_str!`) from the ticket's own route block. No new crate, no `ServeDir`, nothing to install (`web/README.md`). `web/shared/**` is consumed, never edited (T-02's, serialized; a defect there goes to the Orchestrator by comment).
- **Shared assets over HTTP.** PQ-7 owns the route that serves `web/shared/*` (`GET /shared/{file}`, `include_str!`, correct content types) and `room/Cargo.toml`/`Cargo.lock` if anything is needed (prefer nothing). PQ-8 and PQ-9 load `/shared/tokens.css`, `/shared/fonts.css`, `/shared/components.css` and the shared scripts by those URLs and, until PQ-7 is on their branch, serve them in **tests only** through a helper in their own test file (not in `tests/common/mod.rs`, which PQ-32 just changed). The Orchestrator tells you when to merge `origin/ai-c11-cc/wall` into your branch.
- **Tests.** Web unit tests in `web/test/<surface>.test.js` through the existing `_load.js` loader (`just test-web`); room tests in a new `room/tests/<surface>_page.rs` using `tests/common` read-only. `just test` stays under 60 s warm. Measured layout (the wall's AC-33/AC-100) is `test-full` and the c11 browser.
- **Copy.** Every participant-facing string comes from `web/shared/copy.js` (keyed, mirrored by `room/src/copy.rs`); none is retyped. The client's own lines (*Time for a pop quiz.*, *Let's go to the bar.*) are never edited. `argue` is the prototype's key; the field is `takeaway`.
"""

TICKETS = {}

TICKETS["PQ-7"] = dict(
    t="T-05", slug="wall", mode="inline-full", title="The wall", tab="Wall", actor="pq7",
    oneliner="the wall: seven phases on the 1120×630 canvas, the source well and type model, split bars, reveal with the step-list receipt, released with link and QR; no interaction.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `PHILOSOPHY.md`; `DESIGN.md` whole (the three surfaces, the seven phases, the aesthetic, the wall's type model, *For the build*); `SPEC.md` §4 (the phase table's wall column), §5 whole (geometry, layout and the type model, the source well, the split, released), §7.5 (the receipt as a step list, D-25), §11 (the wall's strings); `EVALUATION.md` AC-33, AC-39, AC-40, AC-74, AC-78, AC-79, AC-99, AC-100 and G-7; `sequence/USER_STORIES.md` the same IDs; `sequence/run-state.md` D-8, D-10, D-15, D-25; then `prototypes/C-projector-first.html` with `prototypes/_shared/{proto.js,data.js,tokens.css}` (the binding visual reference; open it in the c11 browser); then the code on `main`: `web/README.md`, `web/shared/**` (tokens, fonts, components, `copy.js`, `phase.js`, `typemodel.js`, `well.js`, `trace.js`, `dom.js`, `check.js` — T-02 ported the pieces you assemble; read their READMEs and tests), `bank/fixtures/receipts/` (the receipt fixtures both twins pass), `room/src/view.rs` (`view::wall`, the payload you render), `room/src/ws.rs` (the wall socket: `GET /rooms/{id}/ws/wall`, frames are the whole state), `room/src/routes.rs`, `room/README.md`, `room/tests/canary.rs`.

**BUILDPLAN T-05, verbatim:** The wall: seven phases on the 1120×630 canvas, full-width source with options beneath, type model, split bars, reveal with **the receipt rendered by the one step-list function** (§7.5), released with link + QR, no interaction. Criteria AC-33, AC-39, AC-40, AC-74, AC-78, AC-79, AC-99, AC-100, G-7. Depends on T-02, T-04a–c.

**Deliverables:**

- **`web/wall/`** — the page: connects to the wall socket for its room, renders every frame as the whole state (a reload shows the same thing), one view per phase in §4's wall column, `idle` with the code and join line, `live` with the source well at the type model's size and the five options beneath (one line each, ≤ 29 characters, D-15), `closed`, `split` with the bars and the named most-chosen incorrect option, `work` stepping the trace (`0..M-2`), `reveal` with ✓ **and** the glyph (AC-40, never colour alone), totals, the *How we know* step list from the one receipt function (`web/shared`'s twin of `receipt_lines`; every line a step the record holds, D-25) and the beats with the human-provenance marker (AC-74), `released` with the link and a QR (generate it client-side from a small vendored routine or a `<canvas>` implementation with its licence; no CDN, AC-77). **No interactive control** on the wall (AC-79). Coloured token spans present in `live`, `closed`, `split` and **zero** in `work` and `reveal` (AC-99). No horizontal scroll ever (AC-33): the well's size comes from `typemodel.js`, never read off the prototype.
- **`GET /shared/{file}`** and **`GET /wall/{room_id}`** in `routes.rs`, your block; a missing room is `404`.
- **Tests, in `test`:** `web/test/wall.test.js` (phase → rendered structure for every phase, on q3's fixture payloads; the ✓ glyph; zero token spans in `work`/`reveal`; no control elements; the receipt lines equal the fixture set in `bank/fixtures/receipts/`), `room/tests/wall_page.rs` (the page and the shared assets serve; the page carries no pre-reveal answer; drive the room through every phase and assert the served HTML never contains a `values` entry named `stdout` before reveal — extend the canary's idea, do not edit `canary.rs`). In `test-full`: the measured layout (AC-100, AC-33) for every bank question at the configured room, in the c11 browser or headless Chrome if available — say which, and record the numbers.
- `room/README.md`: a *Pages* section naming the three page paths and the shared-asset route (you write the section; PQ-8 and PQ-9 add one line each later).

**Out of scope:** the buzzer and host pages (PQ-8, PQ-9 in parallel); the static fallback and host sheet (T-26 / PQ-12, which builds on your page — keep the phase rendering a pure function of a state object so `mode: "static"` can drive it later, and say so in the README); the canary plants (T-08); deploy (T-09); `web/shared` edits.

**Shared files cleared for this ticket:** `web/wall/**` (yours, new), `web/test/wall.test.js`, `room/src/routes.rs` (your `// T-05 pages` block and the `/shared/{file}` route only), `room/tests/wall_page.rs` (new), `room/Cargo.toml` and `Cargo.lock` (only if truly needed; say why), `room/README.md` (the *Pages* section), the `justfile` **only** for a `test-full` line naming the measured-layout run if you add one (read the header comment first; PQ-32 is not touching it). Not `web/shared/**`, not `tests/common/mod.rs`, not `lib.rs`, not `ws.rs`, not `phase.rs`, not `answers.rs`.""" + SPLIT,
)

TICKETS["PQ-9"] = dict(
    t="T-07", slug="host-phone", mode="inline-full", title="The host phone", tab="Host phone", actor="pq9",
    oneliner="the host phone: one screen per phase, one primary action, counts, trace stepping, the read-aloud script, resume on refresh or another device, and the M1 stand-in token in the URL.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `DESIGN.md` (the three surfaces, the seven phases, voice); `SPEC.md` §4 whole (the phase table's host column and the eight host actions), §3.4 (`host_session`, `host_resume_url`, counts), §6 (**Host phone** paragraph, verbatim), §8 and **§8.2 (the M1 stand-in: *Create a room* presents the stand-in host token carried in the page URL; T-09 builds the feature and the secret, you build the page that reads the token from the URL and sends it as the bearer)**, §8.1's two sentences (they go on the first screen), §11 (every host string, the phase labels, the resume-link line); `EVALUATION.md` AC-45, AC-46, AC-47, AC-49, AC-50, AC-74; `sequence/USER_STORIES.md` the same IDs; `sequence/run-state.md` D-10, D-19; then `prototypes/C-projector-first.html` (the host-phone panel) and `prototypes/_shared/*`; then the code on `main`: `web/README.md`, `web/shared/**` (`copy.js` has the host labels and actions keyed; `phase.js`), `room/src/view.rs` (`view::host`), `room/src/routes.rs` (the host action routes, one per `HostAction`, bearer-checked through `HostAuth`; room creation), `room/src/auth.rs` (the seam; tests use `TestAuth`), `room/src/ws.rs` (`GET /rooms/{id}/ws/host`, attach with the host session), `room/src/rooms.rs` (`host_session` never rotates, `host_resume_url`), `room/README.md`, `room/tests/canary.rs` (AC-47: no answer in any host payload before reveal — your page must not fetch one either).

**BUILDPLAN T-07, verbatim:** The host phone: one screen per phase, one primary action, counts, trace stepping, the read-aloud script with the beats' *human* provenance marker (AC-74), resume on refresh/device change; in M1 *Create a room* presents the stand-in host token carried in the page URL (`SPEC.md` §8.2). Criteria AC-45–47, AC-49, AC-50, AC-74. Depends on T-02, T-04a–c.

**Deliverables:**

- **`web/host/`** — `/host` (first screen: *Create a room* using the bearer from the URL, then the room code, the resume link per §11 and §8.1's two sentences) and `/host/{room_id}` (one screen per phase: the phase label, **exactly one primary action** — the next legal `HostAction`, nothing else offered (AC-45, AC-49) — `present` and `answered` counts moving while live over the host socket (AC-46), the code where returning is possible, `←`/`→` and the step's words in `work` and `reveal`, the three beats under *Read it aloud* in `reveal` with the human-provenance marker (AC-74), and **no answer preview before reveal** (AC-47)). Resume: a refresh keeps the session (the host session cookie or the URL fragment — read §6 and `rooms.rs`; the session never rotates, AC-50); the resume link opened on a second device attaches that device to the same room, both control it.
- **Tests, in `test`:** `web/test/host.test.js` (every phase renders its label and exactly one primary action; the action set equals the eight of AC-45 across the phases and nothing else; no answer text in any pre-reveal render), `room/tests/host_page.rs` (the page serves; drive the room through every phase as the host and assert the served HTML and every fetched payload before reveal carry no correct option — the canary's idea). AC-50's two-device resume is `test-full`: two in-process clients with the same resume link both act on the room.
- `room/README.md`: one line under *Pages* (PQ-7 writes the section; if it is not on your branch yet, add the section header yourself in the same words: `## Pages`).

**Out of scope:** the wall and buzzer pages (PQ-7, PQ-8 in parallel); the `dev-host-token` feature, `HOST_DEV_TOKEN` and deploy (T-09 / PQ-11 — you consume the URL contract of §8.2 only); Discord (T-10); the canary plants (T-08); `web/shared` edits.

**Shared files cleared for this ticket:** `web/host/**` (yours, new), `web/test/host.test.js`, `room/src/routes.rs` (your `// T-07 pages` block only), `room/tests/host_page.rs` (new), `room/README.md` (one *Pages* line). Not `Cargo.toml`, not the `justfile`, not `web/shared/**`, not `tests/common/mod.rs`, not `lib.rs`, not `auth.rs`, not `ws.rs`.""" + SPLIT,
)

TICKETS["PQ-8"] = dict(
    t="T-06", slug="buzzer", mode="inline-full", title="The buzzer", tab="Buzzer", actor="pq8",
    oneliner="the buzzer: join by link or code, the six refusals, letters, saving/saved/failed, reconnect, the private hint, the count, the released line.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `DESIGN.md` (the three surfaces, voice); `SPEC.md` §4 whole (the buzzer column), §4.1 (joining and the six failure states), §4.2 (the hint lives in the live payload; no request ever asks for it), §4.3 (answering; saving/saved/failed; the dropped socket re-attaches with the same token and shows *paused* until fresh state), §4.5, §6 (**Buzzer** paragraph, verbatim), §11 (every buzzer string, the refusals, the foot lines, *✓ It was X.*, *n people said A, including you*); `EVALUATION.md` AC-28…AC-32, AC-34…AC-38, AC-48, AC-58, AC-83, AC-85, AC-94; `sequence/USER_STORIES.md` the same IDs; `sequence/run-state.md` D-8, D-12; then `prototypes/B-phone-first.html` and `prototypes/C-projector-first.html` (the buzzer panel) with `prototypes/_shared/*`; then the code on your branch (**it includes PQ-32's wiring**): `room/README.md` (*Sessions*, *The transport*, *Wiring*), `room/src/routes.rs` (`POST /join` — the refusal shape `{refusal, reason}`; `PUT /rooms/{id}/answer` — the response contract table in the README that you render *saving / saved / failed* from), `room/src/ws.rs` (`GET /rooms/{id}/ws/buzzer`, the attach message with the token, the attach frame's `session.saved`), `room/src/view.rs` (`view::buzzer`, `yours`), `room/src/sessions.rs`, `room/tests/sessions.rs`, `room/tests/transport.rs`, `room/tests/wiring.rs` (PQ-32's end-to-end test is your model), `web/shared/**`.

**BUILDPLAN T-06, verbatim:** The buzzer: join by link or code, six failure states, capacity refusal, letters, saving/saved/failed, reconnect, the hint hidden in the live payload, the count on split/work/reveal, the released line. Criteria AC-28–32, AC-34–38, AC-48, AC-58, AC-83, AC-85, AC-94. Depends on T-02, T-04a–c.

**Deliverables:**

- **`web/buzzer/`** at `/join`: the code from `?code=` or typed (one field, nothing else, AC-28); each of the six refusals shows its own §11 message and next step (AC-29), *full* included (AC-30 is the server's; you render it); after a join, keep the token client-side and attach to the buzzer socket; render the phase from every frame; `live`: five `ChoiceButton`s ≥ 44 px (AC-85), the hint behind a tap that reads it **from the payload already held** — never a request (AC-48); **exactly one of saving / saved / failed at all times while live** (AC-35), from the `PUT` response contract: a failed write shows the last saved answer as safe with a retry (AC-36); `closed` restates the saved answer; `split`/`work`/`reveal`: *n people said A, including you* computed **client-side** from totals and the saved letter, no request carrying the answer after close (AC-58); `reveal`: *✓ It was X.* with the glyph (AC-40) and **never** ✗, red, or *wrong* against the participant's own choice (AC-94); `released`: the released line. Reconnect: on a dropped socket show *paused* until the attach frame arrives, then the saved answer from `session.saved` (AC-37). Never source, never trace (G-8). Live region announces each phase and the hint reveal (AC-83; T-13 audits later).
- **`GET /join`** in `routes.rs`, your block.
- **Tests, in `test`:** `web/test/buzzer.test.js` (every phase's render on fixture payloads; the one-of-three saving state machine over the response table; the six refusals; the count line; no ✗/red/wrong; the hint read from state, not fetched — assert no fetch), `room/tests/buzzer_page.rs` (the page serves; join by code and by link; through the wiring: join → attach → answer → close → reconnect with the same token, the page's state matching the server's; the page's HTML and every payload carry no source and no trace — the canary's idea). AC-37 at scale and AC-31 are `test-full`/HC-1.
- `room/README.md`: one line under *Pages*.

**Out of scope:** the wall and host pages (PQ-7, PQ-9 in parallel); the session map, capacity and the upsert (T-04b, done); the transport (T-04c, done) and its wiring (PQ-32, on your branch); the canary plants (T-08); take-it-home (T-12); deploy (T-09); `web/shared` edits.

**Shared files cleared for this ticket:** `web/buzzer/**` (yours, new), `web/test/buzzer.test.js`, `room/src/routes.rs` (your `// T-06 pages` block only), `room/tests/buzzer_page.rs` (new), `room/README.md` (one *Pages* line). Not `Cargo.toml`, not the `justfile`, not `web/shared/**`, not `tests/common/mod.rs`, not `lib.rs`, not `ws.rs`, not `sessions.rs`.""" + SPLIT,
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
        branch_note = f"**Branch.** Your branch starts from `origin/main` @ `{parent_sha}` (M0 complete; M1's phase machine, sessions and transport merged, PQ-32's wiring at review on its own branch). Rebase onto `origin/main` before implementing and again before your first push, never after it. Your PR opens against `main`. **Two sibling surfaces build in parallel this wave**; the split below keeps you apart."
        reps = [
            (old_press, branch_note),
            (f"Open the PR **against `{parent}`** (stacked on #8): `gh pr create --base {parent} --head {branch}`", f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
            ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line", "the line"),
        ]
    else:
        pr = sys.argv[4] if len(sys.argv) > 4 else "#NN"
        branch_note = f"**Press-ahead ticket.** Your branch starts from the in-review wiring branch `{parent}` @ `{parent_sha}` (PR {pr}: PQ-32, the session map wired into the transport — you need `POST /join` pushing to screens and the buzzer attach path), not from `main`. Rebase onto `origin/{parent}` while {pr} is open (the Orchestrator will tell you if it moves); the Orchestrator retargets your PR to `main` once {pr} merges. **Two sibling surfaces build in parallel this wave** off `main`; the split below keeps you apart."
        reps = [
            (old_press, branch_note),
            (f"Open the PR **against `{parent}`** (stacked on #8)", f"Open the PR **against `{parent}`** (stacked on {pr})"),
            ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`", f"the line `Based on {pr} — merge that first; this PR retargets to main afterwards.`"),
        ]
    for a, b in reps:
        assert a in text, (pq, a[:70])
        text = text.replace(a, b)
    assert "#8" not in text.replace("#NN", ""), "a #8 reference survived"
    out = pathlib.Path(wt) / ".claude" / "boot-prompt.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(text)
    print(out, len(text.splitlines()), "lines")


if __name__ == "__main__":
    main()
