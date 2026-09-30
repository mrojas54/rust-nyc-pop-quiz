"""Wave 17 boot prompt: PQ-16 accessibility sweep (fast-track), off `origin/main`.

Dispatched 2026-09-29 23:xx on the client's resume (`/lattice-orchestrator`), the ticket the
2026-09-29 02:3x checkpoint named as the next dispatch. No sibling runs this wave; PQ-26 (burst
and smoke in CI) and PQ-27 (copy freeze and lints) press ahead off this branch once it is at
`review`, because both touch files this ticket holds (`justfile`, `web/shared`).

    python3 gen-wave17.py PQ-16 <origin/main sha>
"""
import importlib.util
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER  # noqa: E402

# The fast-track plan clause, shared with wave 15 (one source, not a second copy).
_w15_spec = importlib.util.spec_from_file_location("w15", pathlib.Path(__file__).parent / "gen-wave15.py")
_w15 = importlib.util.module_from_spec(_w15_spec)
_w15_spec.loader.exec_module(_w15)
PLAN_REVIEW, FAST_TRACK_PLAN = _w15.PLAN_REVIEW, _w15.FAST_TRACK_PLAN

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"
PARENT = "main"

TICKETS = {}

TICKETS["PQ-16"] = dict(
    t="T-13", slug="a11y-sweep", mode="fast-track", title="Accessibility sweep", tab="A11y Sweep", actor="pq16",
    oneliner="the a11y suite over every surface and phase (keyboard, the live region's strings, AA contrast, 44 px targets, reduced motion), and the fixes where a surface falls short.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` **§10** (lines ~588–592, the whole contract in one paragraph: keyboard operable with visible focus; a polite live region per state change; AA contrast; 44 px targets; `prefers-reduced-motion` disables all animation with every state still legible; colour never the only signal), **§6** (~352–366: the buzzer's 44 px targets and its live region announcing each phase and the hint reveal; the host phone's one screen per phase with the phase label), §4 (the phase table — which phases exist on each surface; `idle` has no live-region line by design), **§11** rows *Live region (AC-83), verbatim* (~626: the ten strings) and *Host, phase labels (AC-49)* (~627: the seven labels, announced and never shown), and §11's rule that every string is authored there and only there (G-5: you add no participant-facing string); `sequence/USER_STORIES.md` **Story B9, AC-82…AC-86** (~221–235; read AC-83's amendment note: the list names the phases minted since), AC-40 (colour never the only signal), AC-33 (no horizontal page scroll), AC-48 (the hint is private: its announcement reaches only the phone that took it), AC-97 (the walk-through), AC-102 (the static fallback is the same DOM, driven from the keyboard); `EVALUATION.md` harness row **`a11y`** (line 37: it runs in `test-full`) and rows **AC-82…AC-86** (~132–136, every one `autonomous`, every one `a11y:`); `BUILDPLAN.md` T-13 (line 182: serialized on `justfile` (`a11y`)); `DESIGN.md` ~99–100 (**lights up; *no dark mode in v1* stands** — see the AC-84 ruling below); then the code **on your branch**: the `justfile` (header lines 1–25: the reserved-name convention and `PENDING := "burst:T-21 a11y:T-13"`; `_pending` at ~253; `test-full` at ~62 — the PQ-20 precedent for `verify:T-15b` is the shape: add the recipe, name it in the header, run it from `test-full`, drop `a11y:T-13` from `PENDING`), `web/README.md` (node's built-in runner, `node:vm`, **no `package.json`, no dependency** — the a11y suite lives under that rule too), `web/test/_load.js` (the loader and its document stub: it does no layout, so nothing in `test` measures a rendered pixel — say which checks a stub can prove and which need a browser), `web/shared/dom.js` `announce` (~42–61: the one polite region, `role="status"`, class `sr-only`, the 40 ms clear-then-set; `_resetLiveRegion` for tests), `web/shared/tokens.css` (`--touch-target: 44px` ~75; `--border-focus` ~50; the global `:focus-visible` ~87; the colour tokens ~38–50 are the contrast pairs), `web/shared/components.css` (`.rn-src-scroll` reading region and its focus ring ~39; **`[data-room="dim"]` ~112 is a prototype instrument, not a product mode**), `web/shared/copy.js` (`live_*` ~99–108, `host_phase_*` ~113–119 — the strings you assert, never edit), `web/shared/well.js` (~116: the source region, `role="region"`, `tabindex="0"`, `aria-label`), `web/shared/trace.js` (~134–137: the ←/→ step buttons, `aria-label`, disabled at the bounds), `web/wall/wall.js` (`announcement(frame)` ~213–216 and its call ~284) + `web/wall/wall.css` (reduced motion ~155) + `web/wall/index.html` + `web/wall/fallback/static.js` (the keyboard driver, AC-102) + fixtures `web/wall/fixtures/q3-phases.json`, `q3-static.json`, `web/buzzer/buzzer.js` (the phase→string map ~45–48; every `announce` effect ~125, 224, 286, 301, 318; `io.announce` ~621, 670) + `web/buzzer/index.html` (**`<main aria-live="off">`** — explain it or fix it) + `web/buzzer/buzzer.css` (targets ~20, 60, 78, 123; focus ~24, 65, 129; reduced motion ~134), `web/host/host.js` (`HOST_PHASE_LABEL` ~110 and ~289; `role="status"` ~101; the step buttons ~128) + `web/host/host.css` (targets ~33; focus ~38; reduced motion ~74), `web/home/home.js` (`announce` on trace steps ~241; the page has no phases) + `web/home/home.css` (no focus or motion rule of its own today) + `web/home/fixtures/take-home.json`, `web/wall/measure.html` and `web/home/measure.html` (pages that render from fixtures for a browser — the shape for your browser pass), and the existing suites `web/test/wall.test.js` (~319–322: the tabindex count), `well.test.js` (~85), `trace.test.js` (~134–139), `buzzer.test.js`, `host.test.js`, `home.test.js`, `layout.test.js` (how each surface is rendered per phase in a test — reuse those drivers). `room/**` is read-only for you.

**BUILDPLAN T-13, verbatim:** `a11y`: keyboard, live regions per phase and the private hint, AA contrast, 44 px, reduced motion, over every surface and phase. Criteria AC-82–86. Depends on T-05–07. Serialized on `justfile` (`a11y`).

**Deliverables:**

- **`just a11y`, the suite EVALUATION names.** A node suite on the existing runner and loader (no dependency, no `package.json`), rendering **every surface at every phase from the fixtures** — wall (seven phases plus the static fallback), buzzer (every screen: join, each phase, saving/saved/failed, the hint taken, refusals), host phone (every phase), take-it-home — and asserting, per criterion, what a DOM without layout can prove: **AC-82** every interactive element is a native control or carries `tabindex="0"`, no control is `tabindex="-1"`, and a `:focus-visible` rule exists for each control class (the wall has no controls by design — say what reachability means there: the reading region); **AC-83** each state change in AC-83's list produces exactly its §11 string through `PQ.announce` and nothing else is announced (the wall's phase entries, the buzzer's ten strings including saving/saved/failed and *Hint shown, only to you.*, the host's seven labels, take-it-home's trace steps), with `idle` announcing nothing; **AC-85** every buzzer and host control carries `min-height` and `min-width` of at least `--touch-target` (parse the stylesheets for the control selectors); **AC-86** every stylesheet that declares an animation or transition has a `prefers-reduced-motion: reduce` block that stops it, and no state is conveyed by motion alone (name the state and the static cue for each); **AC-84** AA contrast (4.5:1 body text, 3:1 large text and essential UI, the WCAG relative-luminance formula written in the test) over the token pairs each surface uses — enumerate the pairs (text on each background, the focus ring on each background, the ✓ and the correct-option marker, the bar colours, the trace note) and print the ratio for each. The suite prints a surface × phase × criterion matrix so a reader sees what was covered.
- **Wired in:** `just a11y` runs from `test-full`; `PENDING` drops `a11y:T-13`; the reserved-name header gains the `a11y` line; if the suite is hermetic and fast, also run it from `test` and keep `just test` under 60 s warm — say which you chose and the time.
- **The browser pass, once, by you.** What a stub cannot prove — the real focus order and visible ring, rendered target sizes, the live region actually speaking (VoiceOver or the accessibility tree), reduced motion honoured under the OS setting — you check in the c11 browser against pages served locally (the `measure.html` shape, or a local room started with placeholder values for the four `DISCORD_*` names and `POPQUIZ_ADMIN_TOKEN`: `/join`, `/host`, `/last` load without a sign-in; a room itself needs an organizer, so phase screens come from fixtures). Write the steps into `web/README.md` under *Accessibility* so the validator repeats them, and attach what you saw as validation.
- **Fixes, in each surface's own files,** wherever the sweep finds a surface short of a criterion. Do not assume the list; find it. Places to look first: `web/home` (no focus or motion rule of its own); the buzzer's `aria-live="off"` on `<main>`; whether every phase entry on every surface announces (the buzzer and wall maps against §4's phases); the hint announcement (AC-48: only the phone that took it); the host's step buttons and the trace controls under `:focus-visible`; the static fallback's keyboard driver announcing what the live wall announces; anything animated with no reduced-motion rule.
- **AC-84's dim-room clause — ruled by the Orchestrator, this ticket only.** The built product has one presentation: *lights up*, *no dark mode in v1* (DESIGN.md ~99, touchpoint T-16). Prove AA on that presentation. Record the dim-room half as *no built presentation; T-16* under deviations and do not build one; the Orchestrator routes AC-84's wording upstream as F-37. `[data-room="dim"]` stays what it is.
- **Copy:** you author no string. If a state change in AC-83's list has no §11 string, post a QUESTION on the ticket and continue with the rest. The reveal string carries the answer letter **after reveal only** — `just canary` stays green in every phase.
- **Docs:** `web/README.md` gains *Accessibility*: what `just a11y` proves, what the browser pass proves, how to run each.

**Tests, in `test-full` (and `test` if fast):** one named test per criterion per surface (`ac82_…`, `ac83_…`, …), the matrix printed; `just test-web` and `just test-room` green under 60 s warm, numbers noted; `just canary` green.

**Exit check before DONE:** `just a11y` green on the branch with the matrix in the output; the browser pass done and attached; `just test` and `just canary` green; `PENDING` no longer names `a11y`; `git diff origin/main --stat` shows `room/**` untouched.

**Siblings:** none this wave. **After you:** PQ-26 (burst and smoke in CI) and PQ-27 (copy freeze and lints) press ahead off your branch once you are at `review`; both touch the `justfile` and `web/shared`, so keep your `justfile` change to the `a11y` lines (the recipe, its header line, `PENDING`, the `test-full` line) and your `web/shared` edits additive and small.

**Out of scope:** a dim or dark presentation; new copy; any payload or `room/src/**` change; `pipeline/**`; `mvp/**`; `prototypes/**`; the deploy.

**Shared files cleared for this ticket:** the `justfile` (the `a11y` recipe, its header line, `PENDING`, the `test-full` line — serialized to you this wave), `web/test/a11y.test.js` (new; a `web/test/a11y/` folder if it needs helpers), `web/README.md`, `web/buzzer/**`, `web/host/**`, `web/wall/**` (including `fallback/static.js`), `web/home/**`, and **additively** `web/shared/tokens.css`, `web/shared/components.css`, `web/shared/dom.js` — a change to a colour token's *value* reaches every surface, so post a QUESTION with the failing pair and the proposed value and wait for the ruling before changing one; prefer a per-surface fix. `.github/workflows/ci.yml` only if `test-full` needs a step (it should not). **Not** `web/shared/copy.js`, `room/**`, `pipeline/**`, `fly.toml`, `Dockerfile`.""",
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
    branch_note = (
        f"**Base.** Your branch starts from `origin/main` @ `{parent_sha}`, which holds every M1 and M2 ticket but this one "
        f"(the wall, buzzer and host phone, take-it-home at `/last`, the Discord sign-in, the admin channel, the room lifecycle) "
        f"and is what Fly release v11 runs. Rebase onto `origin/main` before your first push and never after it (an in-review "
        f"branch is never rewritten). Your PR opens against `main`. No sibling runs this wave; touch nothing under `room/`."
    )
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", branch_note),
        (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
         f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line",
         "and the line"),
        (PLAN_REVIEW.replace("{pq}", pq), FAST_TRACK_PLAN.replace("{pq}", pq)),
    ]
    for a, b in reps:
        assert a in text, (pq, a[:70])
        text = text.replace(a, b)
    assert "#8 " not in text and "#8)" not in text and "#8," not in text, "a #8 reference survived"
    assert "stacked" not in text
    out = pathlib.Path(wt) / ".claude" / "boot-prompt.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(text)
    print(out, len(text.splitlines()), "lines")


if __name__ == "__main__":
    main()
