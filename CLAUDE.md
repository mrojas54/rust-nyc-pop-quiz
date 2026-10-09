# Rust NYC Pop Quiz

A live quiz segment for the Rust NYC meetup: original Rust program-output
questions, machine-verified before anyone sees them, run from the front of the
room.

## Read these first

| Artifact | What it is |
|---|---|
| [`PHILOSOPHY.md`](PHILOSOPHY.md) | **The one thing, and the principles.** Read before changing anything user-facing. |
| [`sequence/USER_STORIES.md`](sequence/USER_STORIES.md) | Stories with stable acceptance-criteria IDs (**AC-1 … AC-102**, 20 stories). IDs never change meaning. |
| [`ECONOMICS.md`](ECONOMICS.md) | What it costs to run and who absorbs it. |
| [`DESIGN.md`](DESIGN.md) | **The converged design — Direction C, voice, aesthetic, the wall's type model.** The binding visual contract; the build reproduces `prototypes/C-projector-first.html` one-to-one. |
| [`SPEC.md`](SPEC.md) · [`EVALUATION.md`](EVALUATION.md) · [`BUILDPLAN.md`](BUILDPLAN.md) | **The build contract** (`tone-architect`): what to build and its twelve enforced guardrails; how every criterion is proven and the five human-use checkpoints; the decided stack and the ticket path. Passed the adversarial judge gate 2026-09-18; amended 2026-09-19 after the build stage's contract check (D-16…D-25), then judged the same day (round 1, 6/10 fail; round 2, 8/10 pass). |
| [`sequence/run-state.md`](sequence/run-state.md) | Where the Tone arc is. **Read on every invoke.** |
| [`sequence/research/00-synthesis.md`](sequence/research/00-synthesis.md) | Phase 2 fold-back — what the research changed and why the shape is what it is. |
| [`sequence/research/`](sequence/research/) | The dossier: problem & people, Discord seam, economics, generation & verification, realtime substrate, attendees. |
| [`mvp/README.md`](mvp/README.md) | The hand-run projector deck. How to run it at a meetup. |
| [`docs/adr/`](docs/adr/) | Architecture decision records, with diagrams. ADR-0001: several clubs share one room server (D-26). |

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

Plus **`mvp/`** — a self-contained HTML projector deck, no server, already
running real questions. Not the product; it says `MVP` on it. Build the deck the
segment actually runs with `--only <qid>`; the multi-question form is for
reviewing a batch.

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
- **A built deck is not a run segment.** `answer-history.json` is written by
  `build_deck.py` at **build** time, so building a deck for a meetup that then
  does not happen falsely retires a question — it happened on 2026-08-12 and was
  rolled back by hand. `AC-92` asks for what each meetup *used*; no code
  currently satisfies that. Before trusting the ledger, confirm the segment ran.
- **The build audits the generator, not the sequence**, in both tails — too even
  means someone reintroduced balancing. An audit that can change tonight's
  output is a rule, and rules leak. This area has regressed four times,
  including once *while fixing* the previous regression. Read `PHILOSOPHY.md` §2
  before touching it.
- **Never overstate verification.** Miri proves absence of UB *on executed
  paths* only (AC-43).
- Colour is never the only signal (AC-40).
- Code never causes horizontal page scroll (AC-33).

## Tone arc

`tone-initiation` (done) → `tone-prototype` (done, **2026-09-03: *"yes i love it"***)
→ `tone-architect` (contract written and judged 2026-09-18; handoff waits on the client) → `lattice-orchestrator`. Each stage reads `sequence/run-state.md` first and is
licensed to reopen upstream artifacts — if a client answer contradicts one,
the artifact is what's wrong.
