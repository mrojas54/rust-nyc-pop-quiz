# Rust NYC Pop Quiz

A live quiz segment for the Rust NYC meetup: original Rust program-output
questions, machine-verified before anyone sees them, run from the front of the
room.

## Run guardrails

These rules apply to every run, including workflow resumes and delegated work.
A run ends when its agreed outcome is verified or progress requires the user.

- **One visible outcome per run.** State the user-visible result and the
  observable acceptance check before starting work. Use the user's current
  request to bound the run; a backlog or workflow resume does not authorize
  completing the whole project. Stop when the outcome and required checks are
  complete, and report the evidence.
- **No scope expansion.** Do only the work necessary for the agreed outcome.
  Record unrelated findings in a short deferred list. Do not create tickets,
  dispatch agents, start another workflow stage, or reopen the broader
  contract for those findings unless the user asks. If a new prerequisite
  changes the agreed scope, explain the blocker and wait for the user's
  direction.
- **Bounded review cycles.** Allow at most two review passes per run: an
  initial review and one re-review after fixes. Review the exact changed scope
  and affected behavior; the re-review checks the fixes and their effects.
  If blockers remain after the second pass, report them and stop for the
  user's direction. Do not restart the count by changing agents or sessions,
  weaken acceptance criteria, or mark unresolved work complete. Preserve
  required RED → GREEN → VERIFY → DOCS, phase commits, tests, and PR checks.
- **Stop polling when blocked on the user.** Record one compact checkpoint
  with the current branch/head, completed evidence, exact decision or action
  needed, and how to resume. Stop the blocked work and cancel its self-wakes
  or polling loops; do not repeatedly read tabs, CI, or trackers while waiting
  for the same user action. A pause also stops those loops. Resume only after
  the user supplies the needed input or explicitly authorizes continuation.

## Local reporting skills

[Project status briefing](.agents/skills/project-status-briefing/SKILL.md) is a general-purpose, report-only skill discovered by Codex from `.agents/skills/`. Claude Code does not automatically load this directory; read the linked skill explicitly when requested. Repository run guardrails still apply.

## Waiting for Michelle

When progress requires Michelle's response, approval, or action, finish any
independent work already authorized. Once no useful authorized work can proceed:

1. Save one compact checkpoint in the run's existing handoff or run-state record
   (or the final reply if no such record exists). Include the working directory,
   branch and HEAD, completed work and verification, the exact question or action
   needed from Michelle, and the command or next step to resume. Preserve other
   agents' entries and local changes.
2. Cancel this run's pending self-wakeup and heartbeat timers before ending the
   turn. In a harness with `ScheduleWakeup`, use `ScheduleWakeup({stop: true})`;
   otherwise use its supported timer-cancellation operation. Check the result.
   If cancellation is unavailable or fails, report the timer and limitation in
   the checkpoint; do not claim it stopped or create a replacement timer.
3. End the turn and wait for Michelle. Do not poll the board, PRs, terminals, or
   approval state, repeatedly report the same blocker, or re-arm a fallback timer
   just to check for her response. A longer interval is still polling.

Resume the blocked work after Michelle supplies the needed response or action.
Only continue monitoring that wait if she explicitly requests it. Cancel only
this run's waiting timers, not other runs, useful background jobs, or unrelated
user reminders. Waiting for an independently running test or CI job can use its
completion signal or a bounded process wait; it must not keep reasoning turns
running once Michelle's response is the only way forward.

## Orchestration

Runs are driven by the `lattice-orchestrator` skill on the Lattice board
(`.lattice/`), with delegators in c11 panels working in linked worktrees
(`~/rust-nyc-pop-quiz-worktrees/<slug>`; see `git worktree list`).

- **Read `--help` before composing a c11 or `lattice` command.** Both have many
  flags, and a required one left out has already cost a round trip (closing
  panel 43). c11 closes a panel with
  `c11 close-panel --panel panel:<n> [--workspace <ref>]`; it reaps the panel's
  children, `/quit` does not. State the exact command you ran.
- **Panels can share a worktree.** Before editing an ADR under `docs/adr/` or any
  other doc, run `git status --short` in that worktree and check
  `c11 list-panels` for another delegator on the same files (panel 44 was once
  editing the same ADR at the same time). If one is, coordinate through the
  board rather than editing over it.

## Read these first

| Artifact | What it is |
|---|---|
| [`PHILOSOPHY.md`](PHILOSOPHY.md) | **The one thing, and the principles.** Read before changing anything user-facing. |
| [`sequence/USER_STORIES.md`](sequence/USER_STORIES.md) | Stories with stable acceptance-criteria IDs (**AC-1 … AC-102**, 20 stories). IDs never change meaning. |
| [`PRODUCT.md`](PRODUCT.md) | Descriptive product reading guide and dated source inventory; subordinate to the purpose and build contracts, not a completion ledger. |
| [`ECONOMICS.md`](ECONOMICS.md) | What it costs to run and who absorbs it. |
| [`DESIGN.md`](DESIGN.md) | **The converged design — Direction C, voice, aesthetic, the wall's type model.** The binding visual contract; the build reproduces `prototypes/C-projector-first.html` one-to-one. |
| [`SPEC.md`](SPEC.md) · [`EVALUATION.md`](EVALUATION.md) · [`BUILDPLAN.md`](BUILDPLAN.md) | **The build contract** (`tone-architect`): what to build and its twelve enforced guardrails; how every criterion is proven and the five human-use checkpoints; the decided stack and the ticket path. Passed the adversarial judge gate 2026-09-18; amended 2026-09-19 after the build stage's contract check (D-16…D-25), then judged the same day (round 1, 6/10 fail; round 2, 8/10 pass). |
| [`sequence/run-state.md`](sequence/run-state.md) | Where the Tone arc is. **Read on every invoke.** |
| [`sequence/research/00-synthesis.md`](sequence/research/00-synthesis.md) | Phase 2 fold-back — what the research changed and why the shape is what it is. |
| [`sequence/research/`](sequence/research/) | The dossier: problem & people, Discord seam, economics, generation & verification, realtime substrate, attendees. |
| [`docs/RUNBOOK.md`](docs/RUNBOOK.md) | **How to run the segment on the built room**: schedule, host, sync. |
| [`mvp/README.md`](mvp/README.md) | The retired hand-run projector deck. Still read for *The shape*; its practice-run and field-notes templates are still used. |
| [`docs/adr/`](docs/adr/) | Architecture decision records, with diagrams. ADR-0001: several clubs share one room server (D-26). ADR-0002…0004: the room source grouped by who uses it, the sealed read kept small, and the guardrail scans walking the tree. |

## Standing of `PRD.md`

**`PRD.md` is a prior artifact, not the contract.** It was written outside this
workflow, its own status is *"Draft — blocked on feasibility evidence,"* and
Phase 2 superseded its central architectural choice. Where the PRD and
`sequence/research/00-synthesis.md` disagree, **the synthesis wins.**

Specifically superseded:

- **Live generation inside a five-minute room-ready window** → generation is
  **offline and batch**, with organizer review. Freshness is a supply problem,
  not a runtime problem.
- **A fixed, published answer-category distribution** → no distribution is ever
  published. It was a learnable shortcut (see `PHILOSOPHY.md` §2).
- **Val Town as participant host** → **Rust (`axum`) on Fly.io**, decided at touchpoint T-21 (`BUILDPLAN.md` D-A); the client pays.

Still good and still load-bearing: the Experience State Contract, the pre-reveal
answer-isolation requirements, the Discord authorization model, and the brand
and accessibility contract.

## Shape

Two decoupled components:

- **Offline question pipeline** — generate → pinned rustc + Miri verify →
  dedupe → organizer review → a growing private bank. No deadline.
- **Live room app** — join by code, read code, answer, reveal, aggregate.

Plus **`mvp/`**: the hand-run HTML projector deck from before the room existed.
**Retired 2026-10-11.** The room and `docs/RUNBOOK.md` run the segment, and the
backup on the night is the static `<id>.html` that `just schedule` writes, not
the deck. Don't build decks. `mvp/` stays for its templates
(`PRACTICE-RUN.md`, `FIELD-NOTES-TEMPLATE.md`), *The shape* in its README, and
the `answer-history.json` record.

## House rules

- **Never write down what a program prints.** Run it. Correct answers come from
  `verified.json`, which is written by the verifier, never by hand (AC-7).
- **The segment is one question, 3–5 minutes, scheduled last** (D-7, Story B10 /
  AC-89…AC-92). Not a round. Anything that assumes several questions a night is
  stale.
- **Answer position is drawn uniformly from the date and nothing else**
  (AC-23/23a/23b). Never balance it, never quota it, never avoid last meetup's
  letter — every such rule converts an attendee's memory into a free
  elimination, and enforced balance makes every fifth meetup certain. Uneven
  counts and repeated letters are correct; do not "fix" them.
  `mvp/answer-history.json` is a **record**, never an input.
- **A question is used when its room is released** (AC-92, G-10). The room
  writes the used record at release and `just sync` brings it into the bank
  (`room/src/quiz_session/models/used.rs`). Building a schedule, a fallback or a deck uses nothing,
  and neither does reaching reveal. The ledger lives in memory until synced, so
  sync before the room restarts. The retired deck's `build_deck.py` still writes
  `answer-history.json` at **build** time, which falsely retired q3 on 2026-08-12
  (rolled back by hand); that is one more reason not to build decks.
- **The build audits the generator, not the sequence**, in both tails — too even
  means someone reintroduced balancing. An audit that can change tonight's
  output is a rule, and rules leak. This area has regressed four times,
  including once *while fixing* the previous regression. Read `PHILOSOPHY.md` §2
  before touching it.
- **Never overstate verification.** Miri proves absence of UB *on executed
  paths* only (AC-43).
- Colour is never the only signal (AC-40).
- Code never causes horizontal page scroll (AC-33).

## Testing

- **`just test`** is the inner loop: room, pipeline, web and organizer suites,
  offline and parallel, budgeted at 60 s. **`just test-full`** adds the sandbox
  image, the AC-12 containment suite, the verifier's recorded cases, the
  200-buzzer transport test and the in-process burst/smoke harness. CI runs
  both as separate jobs (`.github/workflows/ci.yml`).
- **Confirm the tree before any run:**
  `pwd && git rev-parse --show-toplevel && git branch --show-current`. This
  repo has several worktrees, and a run in another copy proves nothing about
  this one. pytest runs from `pipeline/` as `uv run --offline --no-sync pytest`
  (what `just test-pipeline` does), never from the repo root.
- **`EPERM` binding a loopback port is the tool sandbox, not the code.** Room
  tests and the canary bind real sockets. Rerun that recipe with the sandbox
  off (or `sandbox.network.allowLocalBinding: true`) instead of asking Michelle
  to paste output.
- **Known flakes.** `just test-full` has failed in the 200-participant load
  suites (`test-transport-full`: `room/tests/transport_full.rs`;
  `harness-full`: `room/tests/harness_full.rs`) while `just test` stayed green,
  and the AC-12 containment suite (`just test-sandbox`) has flaked in CI. When
  one of those is the only failure: name it, rerun that recipe once, and rely
  on CI's `just test-full` job rather than chasing it locally. Anything else is
  a real failure until shown otherwise.
- **`AGENTS.md` is a byte copy of `CLAUDE.md`**, held equal by
  `web/test/agents-sync.test.js` (a regular file, never a symlink). After
  editing `CLAUDE.md`, run `cp CLAUDE.md AGENTS.md`, or `just test` fails on
  that one test.

## Conventions

- **`lib.rs`, `main.rs` and `mod.rs` are required, not banned** (ADR-0005,
  "File names"). The crate root is `lib.rs`, the server binary is
  `server/main.rs`, and every directory module is `<folder>/mod.rs`, never a
  `<folder>.rs` beside its folder. Every other file is a noun named for what
  it holds.
- **Room commands take `--room <url>`.** `just schedule` and `just sync` need
  it (`--no-push` replaces it on `schedule` when the room is down). `just
  --list` shows these recipes only as `*ARGS`, so read the required arguments
  in `docs/RUNBOOK.md` before running one, rather than learning them from a
  failed run.

## Tone arc

`tone-initiation` (done) → `tone-prototype` (done, **2026-09-03: *"yes i love it"***)
→ `tone-architect` (contract written and judged 2026-09-18; handoff waits on the client) → `lattice-orchestrator`. Each stage reads `sequence/run-state.md` first and is
licensed to reopen upstream artifacts — if a client answer contradicts one,
the artifact is what's wrong.
