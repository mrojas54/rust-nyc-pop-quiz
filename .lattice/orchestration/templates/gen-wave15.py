"""Wave 15 boot prompts: PQ-15 take-it-home page (fast-track) and PQ-17 admin channel (inline-full).

Written 2026-09-27 22:3x as press-ahead tickets stacked on PQ-14's in-review branch; PR #30 merged
before either launched (2026-09-28T02:43Z), so both now base on `origin/main`, which holds PQ-14's
seams (`AppState::take_home()`, `AppState::used().all()`) and PQ-37.

    python3 gen-wave15.py PQ-15 <origin/main sha> 30
    python3 gen-wave15.py PQ-17 <origin/main sha> 30
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from header import HEADER  # noqa: E402

WT = "/Users/michellerojas/rust-nyc-pop-quiz-worktrees"
ROOT = "/Users/michellerojas/rust-nyc-pop-quiz"
PARENT = "main"  # PQ-14 (PR #30) merged before launch; no stacking

PLAN_REVIEW = "Then get fresh eyes: spawn a review subagent with the **Agent tool** (`subagent_type: general-purpose`, `model: sonnet`) whose prompt contains only the plan file path, the contract section paths from §2, and the instruction to find contradictions with the contract and missing pieces, returning Critical/Major/Minor findings with file references — not your conclusions. Triage every finding into a block appended to the plan: `## Plan-review resolutions (AUTHORITATIVE — overrides earlier text on conflict)`, one entry per finding with concern / resolution. `lattice status {pq} planned`."
FAST_TRACK_PLAN = "Fast-track: no plan-review subagent. Re-read the plan once against the contract sections in §2 and the code on your branch, fix what contradicts, then `lattice status {pq} planned`."

TICKETS = {}

TICKETS["PQ-15"] = dict(
    t="T-12", slug="take-it-home", mode="fast-track", title="Take it home page", tab="Take It Home", actor="pq15",
    oneliner="the /last page: the last released question with colour, the free-stepping trace, three beats, the receipt and the machine's record, no room state at all.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` **§13 whole** (the contract for this page: source with colour, the trace steppable both ways at the reader's pace, the three beats with the middle beat as *why you might have read it as ‹X›* for **every** incorrect option, the receipt, the five options with the correct one marked; **no room state of any kind — no counts, no most-chosen option** (D-12, AC-56 unamended); *How we know* with the wall's list and beneath it the compiler's full `-Vv` line, the edition, the target triple, the flag set and the Miri configuration, or for a `legacy` record *not recorded* and the Miri row saying the check was run separately (AC-87); then *The machine checked the answer only. An organizer approved the explanation.* (AC-71); rebuilt at every release; code scrolls in its container on a phone (AC-33)); `SPEC.md` §3.1–§3.2 (the record: `explains`, each option's `why_tempting`, `trace`, `verified` and its detail fields), §5.3 (trace semantics: the resolving step), §11 (every string the page uses — reuse; a new string needs the Orchestrator), §12 (the static fallback shares the type model); `sequence/USER_STORIES.md` AC-33, AC-71, AC-87, AC-95 (**note the tension**: the BUILDPLAN T-12 row says *AC-95 (count from release)* but §13 forbids counts on this page — **§13 wins**; the Orchestrator routes the row upstream as F-32; the page shows every incorrect option's why-tempting and no count); `EVALUATION.md` rows AC-33, AC-71, AC-87; `DESIGN.md` (the type model) and **`prototypes/take-it-home.html`** (the binding visual for this page — reproduce it one to one on the shipped copy); then the code **on your branch** (PQ-14's): `room/README.md` *The seams* (`AppState::take_home() -> Option<TakeHome>` and `tests/fixtures/take_home.shape.json` — what the snapshot carries today), `room/src/used.rs` (`take_home(...)`, the snapshot builder) and `room/src/rooms.rs` around the release transition, `room/src/answers.rs` (`Revealed`, `RevealWitness`, `Scheduled::open` — the witnessed read; the **additive witnessed read** you add gives every incorrect option's `why_tempting` and the `verified` detail rows, after reveal only, through the same witness; `room/tests/boundary.rs` must stay green), `room/src/routes.rs` (page routes, the `// T-05|T-06|T-07 pages` blocks — add a `// T-12 pages` block for `GET /last`), `room/src/view.rs`, `web/wall/` (how the wall renders source with colour, the trace and the receipt — reuse its modules from `web/shared`; the well, the type model and `trace.js`), `web/shared/` (never edited), `web/home/` (**exists already?** read it; PQ-12 or PQ-9 may have left a stub — build on it or replace it, say which), `web/test/`, `web/wall/fallback/` (the static fallback renders the same DOM shape for source/trace/receipt — the page shares those renderers), and `pipeline/src/popquiz/bank.py` (the record fields).

**BUILDPLAN T-12, verbatim:** Take it home: the last question with colour, the free-stepping trace, three beats, receipt, options with ✓, container-scroll on a phone. Criteria AC-33, AC-71, AC-87, AC-95 (count from release — superseded by §13, see above), `SPEC.md` §13. Depends on T-02, T-11.

**Deliverables:**

- **The witnessed read, extended (additive):** in `answers.rs`, after `Machine::revealed()` has minted the witness, expose every incorrect option's `why_tempting` and the `verified` detail rows (`-Vv` line, edition, target triple, flags, Miri configuration; `legacy` marked) through the same `Revealed`/witness path — nothing new reachable before reveal; `boundary.rs`'s structural scan and `just canary` green in every phase.
- **The snapshot, completed:** `used::take_home` carries the new fields; `tests/fixtures/take_home.shape.json` regenerated by the machine; still **no counts, no most-chosen option**.
- **`GET /last`:** the page, served like the wall and host pages (`include_str!`), rendering `AppState::take_home()`; when nothing has been released yet, a plain page saying so in a §11 string (or ask for one); rebuilt view on every request (the snapshot is replaced at each release, §13).
- **The page:** the prototype `prototypes/take-it-home.html` reproduced one to one on the type model: source with colour (the wall's well), the trace steppable **both** ways with `←`/`→` and the buttons at the reader's pace, entering wherever the reader likes (no phase lock), the three beats with the middle beat repeated per incorrect option, the receipt list, then *How we know* per §13, then the AC-71 sentence; the five options with ✓ on the correct one (glyph + label, AC-40); **code scrolls inside its container on a phone, the page never scrolls horizontally (AC-33)** — prove it with a 360 px viewport test. The take-it-home link on the released wall and the static fallback already points at SPEC §13's host; this page is what `/last` serves on the room's host, and the released wall's link on the *deployed* room should reach it — note under deviations what `Urls.home` resolves to and whether a one-line change is needed (do not change `web/shared`).
- **Tests, in `test`:** the witnessed-read boundary; the route; the no-release page; the DOM for each section on the fixture; AC-33 at phone width; AC-87 rows for a full record and a `legacy` record; AC-71 sentence present; no count anywhere on the page (grep the rendered DOM for the wall's count strings). `just test-web` and `just test-room` green under 60 s; note the numbers.

**Exit check before DONE:** a screenshot of `/last` on a phone viewport and on a laptop beside the prototype, attached as validation; the fallback compare still green; `just canary` green.

**Sibling running this wave:** PQ-17 (admin channel) owns `room/src/admin.rs` (new), `routes.rs`'s `// T-25 routes` block, `lib.rs` (router wiring, additive), `config.rs` (the token name), `.env.example`, the `justfile`, `room/tests/admin.rs` and the canary plant for the admin token. You add to `routes.rs` only your `// T-12 pages` block and to `lib.rs` nothing (ask if you must). Both of you sit on PQ-14's branch: never touch `rooms.rs`, `lifecycle.rs`, `sessions.rs`, `phase.rs`.

**Out of scope:** counts of any kind; the wall's or host's released screens; `web/shared/**`; `pipeline/**`; the schedule (T-20); DNS or the §13 host itself.

**Shared files cleared for this ticket:** `room/src/answers.rs` (the additive witnessed read only), `room/src/used.rs` (the snapshot fields), `room/src/routes.rs` (`// T-12 pages` block only), `room/src/view.rs` (additive, if the page needs a projection), `room/tests/take_home.rs` (new), `room/tests/fixtures/take_home.shape.json`, `room/tests/boundary.rs` (assertions only), `web/home/**`, `web/test/**`, `room/README.md` (the *Pages* and *The seams* rows for `/last`).""",
)

TICKETS["PQ-17"] = dict(
    t="T-25", slug="admin-channel", mode="inline-full", title="Admin channel for the pipeline", tab="Admin Channel", actor="pq17",
    oneliner="the two admin routes behind one constant-time bearer: schedule a question into the sealed module, read the used ledger; the token planted in the canary and scanned for in the repo.",
    body="""Read, in this order, before planning: `CLAUDE.md`; `SPEC.md` **§8.3 whole** (the contract: exactly two routes, `PUT /admin/questions/{id}` into the sealed `answers` module and `GET /admin/used`; `Authorization: Bearer ‹POPQUIZ_ADMIN_TOKEN›`, a Fly secret never in the repository; one constant-time check; a missing or wrong token refused with no information about what is stored; the token in no payload, page or log line; the admin prefix served to that check alone, a route-table test asserting no participant, wall, host or auth-module route reads it), §8.2 (the stand-in — the model for a constant-time bearer, and what T-10 later deletes), guardrail **G-9** (every credential path enumerated and checked by one function), §3.1–§3.2 (the record `PUT` accepts, answer included — exactly the JSON `bank.py` writes), §3.3 (`used`: what `GET /admin/used` returns, the pipeline's `Used` shape), §4.6 (a scheduled question and *never twice*); `sequence/USER_STORIES.md` AC-101, AC-61; `EVALUATION.md` row AC-101 whole (the `test`, static scan, `canary` plant and the live oracle) and the `canary` row (line ~35); `BUILDPLAN.md` T-25 (line ~183), D-20, H-11 (the client generates the token, `fly secrets set`, keeps it locally — the deployed check is theirs, not yours); then the code **on your branch** (PQ-14's): `room/src/standin.rs` and `room/src/auth.rs` (the constant-time compare, `ct_eq`, the feature gating — your check is **not** feature-gated: it ships in every build), `room/src/config.rs` (env parsing; add `POPQUIZ_ADMIN_TOKEN`, required, non-empty, never `Debug`), `room/src/rooms.rs` (how a question is scheduled today: the `dev-host-token` seeding path and `AppState`'s question map — `PUT` replaces the seeding as the production path; the seeding stays behind its feature), `room/src/answers.rs` (`load(json)` → `Scheduled`; the sealed module is entered only through it — you never read the answer), `room/README.md` *The seams* (`AppState::used().all() -> Vec<UsedEntry>` — the shape `GET /admin/used` serialises, `bank.py`'s `Used` fields), `room/src/routes.rs` (the route table; `// T-04c routes`, page blocks; add `// T-25 routes`), `room/src/lib.rs` (`router`/`router_with`, how state and auth are provided), `room/tests/canary.rs` and `room/tests/canary_scan/` (PQ-10 planted `POPQUIZ_ADMIN_TOKEN` as a runtime-random value already — read how; your job is to make the plant live: the token is set, the routes use it, and it appears nowhere), `room/tests/standin.rs` (the AC-64 test shapes to mirror), the `justfile` (the `test` recipe: where the repo-wide scan hooks in), `.github/workflows/`, `.env.example` (append the name line with `printf '%s\\n' 'POPQUIZ_ADMIN_TOKEN=' >> .env.example` — a write, no read; F-5), `fly.toml` (no secret there ever), and `pipeline/src/popquiz/bank.py` (`Used`, the record JSON).

**BUILDPLAN T-25, verbatim:** The pipeline channel, server side: `PUT /admin/questions/{id}` into the sealed `answers` module and `GET /admin/used`, one constant-time bearer check on `POPQUIZ_ADMIN_TOKEN`, a route-table test that nothing else is served under the admin prefix and no other route reads the token (`SPEC.md` §8.3, D-20); extends `canary` with `POPQUIZ_ADMIN_TOKEN` as a plant and adds the repo-wide scan for the token to `test` (`EVALUATION.md` AC-101). Criteria AC-101, AC-61, G-9. Depends on T-04a, T-08, T-11. Human track H-11 for the deployed check only. Adds the name `POPQUIZ_ADMIN_TOKEN`, never a value, to `.env.example`. Serialized on `justfile`.

**Deliverables:**

- **`room/src/admin.rs`:** one `AdminAuth` (or the `HostAuth`-style trait, say which) holding the token bytes read at startup from `POPQUIZ_ADMIN_TOKEN` (missing or empty → startup error, like the stand-in's); one constant-time compare; the two handlers. Refusals: 401 with an empty body for missing or wrong bearer, identical timing shape, no `WWW-Authenticate` detail, nothing logged about the token or the request body. `PUT /admin/questions/{id}`: body is the record JSON; loaded through `answers::load` (so the answer enters the sealed module and nowhere else, AC-61); `{id}` must equal the record's id; a used question is refused (G-10, PQ-14's ledger); a question already scheduled is replaced only while no room holds it (decide the rule from §4.6, say why). `GET /admin/used`: the ledger as `bank.py`'s `Used` shape, from `AppState::used().all()`.
- **Route-table proof (AC-101):** a test that walks the router and asserts the admin prefix is served to the admin check alone, no other route reads `POPQUIZ_ADMIN_TOKEN` (a structural scan of `room/src` for the name outside `admin.rs` and `config.rs`), and no participant, wall, host or auth-module route accepts the bearer.
- **The canary plant, live:** with the token set to a runtime-random canary, `just canary` scans every payload, frame, page and log line in every phase and finds it nowhere; a positive control plants it in one response and proves the scan fails. Log lines: capture the tracing output in the test and scan it.
- **The repo-wide scan, in `test`:** a `just` recipe (name it in the reserved-name header) or a pipeline test that fails on any secret-shaped literal for the token and on any planted canary value anywhere in the repository; the *name* is allowed in `.env.example` and in the code that reads it. Keep `just test` under 60 s.
- **CI:** the test runs there unchanged; no secret in the workflow.
- **`room/README.md`:** a *Pipeline channel* section (the two routes, the token's name, H-11: generate, `fly secrets set`, keep locally; rotation), and the seeding path marked as the stand-in's only.
- **Never:** print, log, echo or commit a token value; the deployed check (`fly secrets set` and a real push) is the client's, after merge — say so in DONE.

**Exit check before DONE:** every AC-101 clause above mapped to a test name; `just test-room` and `just canary` green with times; the structural scan green; the repo-wide scan proven with a planted literal that it catches (then removed); `.env.example` diff is one appended line.

**Sibling running this wave:** PQ-15 (take-it-home) owns `answers.rs`'s additive witnessed read, `used.rs`, `routes.rs`'s `// T-12 pages` block, `web/home/**`. You add to `routes.rs` only your `// T-25 routes` block. Both of you sit on PQ-14's branch: never touch `rooms.rs` beyond an additive scheduling seam (ask first), `lifecycle.rs`, `sessions.rs`, `phase.rs`.

**Out of scope:** the laptop side (`popquiz schedule` / `popquiz sync`, T-20); Discord (T-10); the deployed check (H-11, the client's); `web/**`; `pipeline/**` except reading `bank.py`.

**Shared files cleared for this ticket:** `room/src/admin.rs` (new), `room/src/config.rs` (the token, additive), `room/src/lib.rs` (router wiring, additive), `room/src/routes.rs` (`// T-25 routes` block only), `room/src/rooms.rs` (only an additive `schedule` seam if the map has none — comment first), `room/tests/admin.rs` (new), `room/tests/canary.rs` and `room/tests/canary_scan/**` (the plant and log scan), `room/tests/boundary.rs` (assertions only), the `justfile` (the scan recipe and its header line — serialized to you), `.github/workflows/*` (only if the scan needs a step), `.env.example` (append only), `room/README.md`. Not `answers.rs` (PQ-15's this wave), not `standin.rs`, `auth.rs`, `ws.rs`, `view.rs`, `web/**`, `fly.toml`.""",
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
    branch_note = f"**Base.** Your branch starts from `origin/main` @ `{parent_sha}`, which already holds PQ-14 (room lifecycle, PR #{parent_pr}, merged) and PQ-37 (guest split, PR #31, merged) — the seams you consume are on `main`. Rebase onto `origin/main` before your first push and never after it (an in-review branch is never rewritten). Your worktree was cut earlier at PQ-14's PR head and fast-forwarded to this sha by the Orchestrator; `git log --oneline -1` should show `{parent_sha[:7]}`."
    reps = [
        (f"**Press-ahead ticket.** Your branch starts from the in-review scaffold branch `{PARENT}` @ `{parent_sha}` (PR #8), not from `main`, because you need the harness it adds. Rebase onto `origin/{PARENT}` while #8 is open; the Orchestrator retargets your PR to `main` once #8 merges.", branch_note),
        (f"Open the PR **against `{PARENT}`** (stacked on #8): `gh pr create --base {PARENT} --head {branch}`",
         f"Open the PR **against `main`**: `gh pr create --base main --head {branch}`"),
        ("the line `Based on #8 — merge that first; this PR retargets to main afterwards.`, and the line",
         "and the line"),
        ("Both of you sit on PQ-14's branch:", "PQ-14's code is on `main` under you both:"),
    ]
    if d["mode"] == "fast-track":
        reps.append((PLAN_REVIEW.replace("{pq}", pq), FAST_TRACK_PLAN.replace("{pq}", pq)))
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
