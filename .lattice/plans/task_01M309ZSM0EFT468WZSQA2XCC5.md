# PQ-29: Organizer runbook

BUILDPLAN.md T-24 (M4).

The organizer runbook: successor to `mvp/README.md`'s run-of-show for the built room; host script per phase; the statements that option text is public and that a Rust-expert host can infer the answer; the rehearsal and field-notes sheets pointed at the build

Criteria: AC-51, AC-62, AC-63, AC-90
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-09, T-20
BUILDPLAN notes: —

Workflow mode: fast-track. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-10-04)

Base `de1da5f` (origin/main, PR #47 merged). Fast-track; no plan-review subagent.

## Facts run on the base (quoted from output, not from docs)

- `python -m popquiz.schedule schedule --help`: `--date DATE (--room ROOM | --no-push) --out OUT [--home-link] [--bank] [--lead-time] [--threshold] question_id`. **`--room` and `--no-push` are mutually exclusive** (argparse group).
- `sync --help`: `--room ROOM [--bank] [--lead-time] [--threshold]`.
- `reserve` exists (third subcommand) → QUESTION comment, no recipe.
- `schedule q3 --date 2026-10-14 --no-push --out "$TMPDIR/runbook-check"` → exit 1, stderr `popquiz.schedule: q3 is not affirmed - it has no affirmed_by and no affirmed_at, and it has not been accepted at review - and an unaffirmed question cannot be scheduled (AC-72)`, then the three reserve lines (`reserve: 0 ready …`, `trend: +0 … +4 not yet reviewed …`, `warning: 0 ready is below the threshold of 2 …`). Nothing written.
- Room refusals (code): `That question has already been run. Pick another.` / `No question is scheduled with that id.` (rooms.rs create); `<id> has already been run; a question is never run twice.` / `A room is running <id>; it can be replaced once that room is gone.` (rooms.rs schedule, 409 on PUT); admin.rs 400s `The record is not UTF-8 JSON.`, `The path names <id>; the record is <id>.`, 413 `The record is larger than a question record can be.`; 401 empty body. schedule.py: `… is not set in the environment …`, `the room refused the admin token (401); the value in POPQUIZ_ADMIN_TOKEN is not the room's`, `… is inside the repository; …`, `… is not an https URL …`.
- Host denials (copy.js, F-33): `That Discord account isn't in the Rust NYC server.` / `That Discord account doesn't have the organizer role.`
- Room measurements are not configuration: `web/shared/typemodel.js` `ROOM_DEFAULTS` (15 / 8.44 / 20) is code; `just bank-audit --screen-width-ft --screen-height-ft --back-row-ft` checks the bank against a measured room.
- Host phone shows no wall link; the wall is `/wall/<room_id>`, the id in the host page's path; the host page address after *Create a room* (`/host/<room_id>#<host session>`) **is** the resume link (host.js header, AC-50).

## docs/RUNBOOK.md — headings and the criterion each serves

1. `# Running the Pop Quiz` (title) + one line: describes the room on `main` at `de1da5f`; the deck's run-of-show stays in `mvp/README.md`.
2. `## What this is` — one question, 3–5 min, last; three reasons by pointer to `mvp/README.md` *The shape* (AC-89, AC-90).
3. `## Before the night` — `### The reserve` (`just schedule` / `just sync` print it first; the three lines quoted from the run) · `### Choosing a question` (from the reserve; position is the date's, never balanced — `PHILOSOPHY.md` §2) · `### Affirming a question — pending F-52` (ruling 1: what the gate checks, who the contract says writes it — T-18's review surface, SPEC §3.1/§7.4 — that no committed question is schedulable today, quoted refusal, and that the path to the first one is the owner's open decision; no workaround) · `### Scheduling` (`just schedule <id> --date … --room https://rustnyc-popquiz.fly.dev --out ~/popquiz-<date>`; token via `op read` into the env for that one command, §8.3; what it prints; the two files; `--out` absolute and outside the repo; `--no-push` form for a night the room is unreachable — AC-102) · `### Deploy before the rehearsal` (ruling 2: deployed v11 = `1263add` is behind; deploy is the client's step; `fly deploy --ha=false --remote-only`; `just smoke` with `smoke-q3` then `fly apps restart` — never during a night).
4. `## At the venue` — three measurements (AC-100, EVALUATION ~179 hypothesis 15 ft / 16:9 / 20 ft), `just bank-audit` with them, projector, fallback open.
5. `## Creating the room` — `/host?question=<id>`, *Sign in with Discord*, *Create a room*, save the address (resume link), `/wall/<room_id>` on the projector, join strip (§4 idle). Before the last talk ends; the room closes after 30 min idle (§4.6) — so create it during the last talk, not at the start of the night.
6. `## The host script` — pinned AC-90 sentence first; then one `###` block per phase in §4 order (`idle` … `released`): host action verbatim from §11 *Host, actions*, what the wall shows, what to say (`> **Say:**` lines) and how long. The hint (§4.2): on every phone from `live`, host has no control; one optional *Say* line. Nothing asks for a contribution (AC-98).
7. `## Do not` — never two questions (G-10); never announce at the start; never read the answer off the laptop — **the two §8.1 sentences verbatim here** (AC-62, AC-63); never *Run it again* at the meetup; never restart before `just sync`; no balancing.
8. `## After` — `just sync` before any restart; what `used` records (AC-92) and that the ledger is the record; the night's request rate (`fly logs` lines from room/README *After a meetup*); file the sheets at `mvp/<YYYY-MM-DD>/field-notes.md` (HC-4) and `…/practice-run.md` (HC-1).
9. `## If something fails` — two-column table, REHEARSAL-GUIDE's rows corrected, strings only from code/§11.
10. `## The instruments` — pointers to `mvp/PRACTICE-RUN.md`, `mvp/FIELD-NOTES-TEMPLATE.md`, `mvp/README.md`.

**Pinned AC-90 sentence** (host script, first paragraph): `The Pop Quiz is one question, and it is scheduled last: after the last talk, with nothing after it, and the wrap-up releases the room.`

## The other files

- **justfile**, after `bank-audit` (line 292), nothing else:
  `# Schedule one question for a meetup: the affirm gate, the push, the fallback and host sheet (T-20).` + `schedule *ARGS:` / `    cd pipeline && uv run --offline --no-sync python -m popquiz.schedule schedule {{ ARGS }}`;
  `# After a meetup, before any restart: the room's used ledger into the bank (T-20).` + `sync *ARGS:` / `… popquiz.schedule sync {{ ARGS }}`.
- **pipeline/README.md** *Scheduling a meetup* only: the two code blocks become `just schedule …` / `just sync …` from the repo root; one line keeps the `python -m popquiz.schedule …` form (from `pipeline/`) for a machine without `just`; one line pointing at `docs/RUNBOOK.md`. `--out` example stays absolute.
- **README.md** *Where things live*: one row `[docs/RUNBOOK.md](docs/RUNBOOK.md) | Running the Pop Quiz at a meetup: schedule, host, sync`.
- **mvp/PRACTICE-RUN.md** *Before* only: one paragraph before *Drive `prototypes/…`* — the built-room branch from EVALUATION HC-1 (built room iff HC-0 and T-10's live Discord check have passed, driven from `docs/RUNBOOK.md`; else the prototype as written below), and that a built-room run spends its question through the room's ledger and `just sync`, not by hand. Capture questions, measurements, origin untouched.
- **mvp/FIELD-NOTES-TEMPLATE.md**: header line gains `**Condition:** the deck, hands only / the built room, phones` (one line); one sentence after the hands-only paragraph pointing at `docs/RUNBOOK.md` for a built-room night. Nothing else.

## The test — web/test/runbook.test.js

CommonJS + `__dirname` like every other suite there (deviation from the brief's `import.meta.url`: the directory has no ESM test; `_load.js` is CJS). Cases:
1. AC-62 sentence present verbatim (constant copied from SPEC §8.1). Mutation: change one char → fail.
2. AC-63 sentence present verbatim. Same mutation.
3. AC-90: pinned sentence present inside the `## The host script` section. Mutation: delete it → fail.
4. Every relative path in a code span (ending `.md .json .py .rs .js .html`, or starting `mvp/ docs/ room/ pipeline/ web/`) exists, after stripping `<placeholder>` spans — placeholder paths (`mvp/<YYYY-MM-DD>/…`) are checked by their existing parent. Mutation: rename a path → fail.
5. Every `just <recipe>` names a justfile recipe (regex `^[a-z][a-z0-9_-]*( \*ARGS)?:` plus recipes with other params, e.g. `smoke URL *ARGS:` — the brief's regex misses `smoke`/`burst`/`wall-layout`; widen to `^([a-z][a-z0-9_-]*)(\s[^:]*)?:` and say so). Mutation: `just reserve` → fail.
6. Forbidden row over the words-to-say (`> **Say:**` lines), via `web/test/copylint.js` (`load('copylint')`, imported, not edited); asserts ≥ 5 such lines so it cannot pass vacuously. Mutation: add *turn to your neighbour* → fail.

## Kept from REHEARSAL-GUIDE / corrected

Kept: the structure (prepare → schedule → create → before → run → fails → after → sheet), the failure table's shape and rows (400 on empty `question=`, *No question is scheduled with that id.*, unauthorized upload, Discord denial, *already been run*, lost phone, clipping, answer during walk-through, host asks for contribution), the "do not rename a used question" rule, lights up, observer takes notes. Corrected: scheduling is `just schedule`, not a raw `PUT`; button labels are §11's (*Start*, *Trace*, *End Pop Quiz*, not *Put it on the screen* / *Let's walk it* / *Release the room*); generation/review status stated per ticket; restart-before-sync rule added.

## Contract tensions — side taken

1. `EVALUATION.md` AC-45 row names *Put it on the screen*, *Let's walk it*, *Release the room*; SPEC §11 and `copy.js` say *Start*, *Trace*, *End Pop Quiz*. Side: §11 (the brief and the shipped buttons). Routed as a contract note.
2. The brief's validation command passes `--room` and `--no-push` together; argparse rejects that (mutually exclusive). Ran `--no-push` alone.
3. `reserve` subcommand exists — QUESTION comment, no recipe.
4. `mvp/README.md` walk line *"if nobody does, nothing is missing"* matches the §11.1 trope list; the runbook's walk line does not carry it.
