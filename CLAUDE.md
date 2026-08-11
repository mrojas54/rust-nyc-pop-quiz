# Rust NYC Pop Quiz

A live quiz segment for the Rust NYC meetup: original Rust program-output
questions, machine-verified before anyone sees them, run from the front of the
room.

## Read these first

| Artifact | What it is |
|---|---|
| [`PHILOSOPHY.md`](PHILOSOPHY.md) | **The one thing, and the principles.** Read before changing anything user-facing. |
| [`sequence/USER_STORIES.md`](sequence/USER_STORIES.md) | Stories with stable acceptance-criteria IDs (**AC-1 … AC-70**). IDs never change meaning. |
| [`ECONOMICS.md`](ECONOMICS.md) | What it costs to run and who absorbs it. |
| [`sequence/run-state.md`](sequence/run-state.md) | Where the Tone arc is. **Read on every invoke.** |
| [`sequence/research/00-synthesis.md`](sequence/research/00-synthesis.md) | Phase 2 fold-back — what the research changed and why the shape is what it is. |
| [`sequence/research/`](sequence/research/) | The dossier: problem & people, Discord seam, economics, generation & verification, realtime substrate. |
| [`mvp/README.md`](mvp/README.md) | The hand-run projector deck. How to run it at a meetup. |

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
- **Val Town as participant host** → open, leaning Cloudflare Durable Objects.

Still good and still load-bearing: the Experience State Contract, the pre-reveal
answer-isolation requirements, the Discord authorization model, and the brand
and accessibility contract.

## Shape

Two decoupled components:

- **Offline question pipeline** — generate → pinned rustc + Miri verify →
  dedupe → organizer review → a growing private bank. No deadline.
- **Live room app** — join by code, read code, answer, reveal, aggregate.

Plus **`mvp/`** — a self-contained HTML projector deck, no server, already
running real questions. Not the product; it says `MVP` on it.

## House rules

- **Never write down what a program prints.** Run it. Correct answers come from
  `verified.json`, which is written by the verifier, never by hand (AC-7).
- **Audit the answer positions on every deck build** (AC-23). This has already
  regressed once — the first build shipped a visible B, D, B, D pattern.
- **Never overstate verification.** Miri proves absence of UB *on executed
  paths* only (AC-43).
- Colour is never the only signal (AC-40).
- Code never causes horizontal page scroll (AC-33).

## Tone arc

`tone-initiation` (done) → `tone-prototype` → `tone-architect` →
`lattice-orchestrator`. Each stage reads `sequence/run-state.md` first and is
licensed to reopen upstream artifacts — if a client answer contradicts one,
the artifact is what's wrong.
