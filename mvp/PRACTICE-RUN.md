# Practice run — a rehearsal with two regulars

Instrument **and** capture form, like `FIELD-NOTES-TEMPLATE.md`. Copy to
`mvp/<YYYY-MM-DD>/practice-run.md`, fill the blanks in place, and that copy is
the evidence.

**Host:** the client  **Helpers:** two regulars  **Date:** `__________`
**Where:** `______________________`  **Question:** `q3` if the prototype (see *Before*)

**Origin.** Rust lunch, reported 2026-09-02. Three people at lunch; the two
who are not the client are regulars, saw nothing, and offered to help run a
rehearsal. They remembered the old quiz, liked the idea, and liked voting
anonymously (A-6). **The meetup is October; this runs before it.** Recorded in
`sequence/run-state.md` (T-18).

---

## What this is, and is not

**A rehearsal, not a room.** Two participants, both regulars, both of whom will
be in the October room. It cannot stand in for the field notes and must not be
read as if it did — see *What it cannot tell you* at the bottom, and take it
seriously, because two people who know Rust agreeing with the host is the
easiest false positive this project can produce.

**What it is for:** retiring the unknowns that need a host, a projector, and
someone looking — but not a crowd — so that October's one shot is spent only on
what a crowd can answer. Until now every phase of the segment has been driven
by the client alone, at a desk. Nobody has clocked it, nobody has watched it,
and the two numbers everything about type size rests on are still guesses.

**The client runs it.** The helpers are participants, not delegates. T-8 stands.

---

## Before

**Which room.** The built room if HC-0 **and** T-10's live Discord check have
both passed (`EVALUATION.md`, HC-1): drive it from `docs/RUNBOOK.md`, and its
question is spent through the room's ledger and `just sync`, not by hand.
Otherwise the prototype, as written below.

**If the prototype: drive `prototypes/C-projector-first.html`, not the MVP deck.** The MVP deck
predates Story B11 — it has no split phase and no walk-through, and those are
the two phases the rehearsal exists to watch. All seven phases, in order, from
`1 · title` to `7 · released`. The `PROTOTYPE` badge stays on; tell the helpers
what it means.

**If the prototype: the question is `q3`, and it is spent afterwards.** C's vote, split and
reveal run on mock room data that belongs to `q3` — swapping the source swaps
only the reading phases. Two regulars who will be in October's room will have
seen `q3`'s source and its answer. Treat it as used: bank **8 → 7**, and
**do not run `q3` in October**. Nothing writes the ledger for a prototype run
(`answer-history.json` is written by `build_deck.py` at build time, and this is
not a build), so record it by hand in `sequence/run-state.md`.

- [ ] Helpers told which question they are about to burn, so they know not to
      mention it in October.
- [ ] **Where is this happening?** The venue, if it is at all possible — the two
      measurements below are the highest-value output of the whole evening, and
      the only output that needs the venue. Anywhere else with a projector or a
      big screen works for everything else; then skip the measurements and
      **say so** in this sheet rather than estimating them.
- [ ] Lights **up**. AC-80 is answered; do not re-open it here.
- [ ] Whoever is not hosting takes this sheet. The host cannot take notes.

### The two measurements of the room — take these once, ever

Copied from `FIELD-NOTES-TEMPLATE.md` because they only need the venue, not a
crowd, and a rehearsal at the venue is the earliest chance to take them.

- [ ] **How wide is the projected image?** Pace it. → `______ ft`
- [ ] **How tall?** → `______ ft`. The type model assumes 16:9 and derives
      height from width; a 4:3 projector or a TV breaks that, and it is the
      height that decides how many lines fit. Client, mid-drive: *"practice
      will let us know how tall the screen is."*
- [ ] **How far back is the last row anyone actually sits in?** Not the back
      wall — the last row that fills at a normal meetup. → `______ ft`

At the current **guess** — 15ft wide, back row at 20ft — everything in the bank
fits, with `q1` (16 lines) at the floor. These two numbers replace the guess.

---

## During — clock it, and watch the host

⏱ **Total segment**, `2 · question live` on screen → `7 · released` on screen.

> **AC-89 target: ≤ 5 minutes.** Measured: `______ min ______ s`
> First time this has been clocked with anyone watching.

⏱ **Seconds from the code appearing to the first visible opinion.** Regulars
read fast, so this is a **lower bound** on the room's number, not the number.

> AC-91 target: ≤ 30 s. Measured: `______ s` — *(lower bound)*

📐 **Did the source fit the wall?** At the projector's real resolution, no
scrolling, no clipping, options not on top of the source.

> `fit / clipped / scrolled` · lines `____` · longest line `____` chars
> `________________________________________________`

This is the AC-33 sibling: on the wall the source must *fit*, because
container-scroll was a phone escape hatch and A-4 took the phone away.

**Anything the host apologised for or improvised around** (AC-51). First time
with an audience.

> `________________________________________________`

**`5 · work through it` — did the phase have anything in it?** (AC-97)
Two regulars are the *people who know it* half of the one thing, with nobody in
the room who doesn't. So the honest outcome may be that the host stepped the
trace to silence. Write that down if it happened; it is not a failure of the
phase, it is the reason the phase needs a room.

> `________________________________________________`

**The host's mouth** (AC-98). The rule that nothing obliges a participant to
speak has been cut from the *copy* twice in one day; it can come straight back
in *speech*. Did the host ask anyone to explain their pick, compare answers, or
volunteer? Note the exact words if so.

> `________________________________________________`

Do **not** rehearse the MVP README's run-of-show beat *"Someone who said E —
why?"* — that line predates AC-98 and is the thing AC-98 forbids. A
contribution that is offered can be taken; none is asked for.

---

## After — three questions to two people, ~3 minutes

**1. Legibility from the back** (AC-78 / AC-38). Stand them at the measured
last row, or as far back as the space allows, on `2 · question live`:

> *"Could you read the code from there, or were you guessing?"*
> helper 1: `read it fine / squinted / couldn't` · helper 2: `read it fine / squinted / couldn't`

**2. The split before the answer** (AC-93, `felt`). This one is the client's
judgment to make — the helpers are two more pairs of eyes, not the verdict.

> *"When how-the-room-voted came up before the answer, did it do anything for
> you, or was it a delay?"*
> `________________________________________________`

**3. Syntax colour in the reading phases** (T-17). Show `2 · question live`
with colour on and off. This does **not** decide T-17 — that is Cole's call,
because it diverges from his component — it is two data points for it.

> helper 1: `colour / plain / no preference` · helper 2: `colour / plain / no preference`
> `________________________________________________`

**And the one there is no box for:**

> *"Did anything in it make you want to say something? Did you?"*
> `________________________________________________`

That is the contribution AC-98 leaves room for. Whether it arrives unprompted
at n=2 is worth one line.

---

## What it cannot tell you

Do not infer any of these from two regulars, however clear the signal feels.

| Unknown | Why not here |
|---|---|
| **Participation rate** | n=2. The variable that sizes AC-52/AC-54 stays unmeasured until October. |
| **AC-44 / AC-96** — a beginner can explain it | No beginner present. Regulars get it right or wrong for expert reasons. |
| **§9's moment** — being wrong is ordinary | No newcomer present, and the two know the host. The moment cannot be observed. |
| **Q2 — phones help or hurt** | Story B1/B5's fate. Two friends of the host voting is not a signal — and both already said they like anonymous voting, so their read is known before they sit down. |
| **Q1 — timer wanted** | Weak at n=2; leave for the room. |
| **AC-88 difficulty** | Regulars' read of "about right" is biased upward. |
| **AC-21** — the review clock | Not a segment question at all; the client measures it alone at the first real batch. |

---

## Bookkeeping

- [ ] Copy of this sheet, filled in, at `mvp/<YYYY-MM-DD>/practice-run.md`.
- [ ] `sequence/run-state.md`: T-18 answered, `q3` marked spent by hand, the
      two room numbers replacing the 15ft / 20ft hypothesis everywhere it
      appears (`FIELD-NOTES-TEMPLATE.md`, `prototypes/C-projector-first.html`'s
      derived-type defaults, run-state).
- [ ] If the host caught herself asking for a contribution: record the words in
      run-state under AC-98. That is the third time, and it means the rule needs
      to reach the host script, not just the surfaces.

---

*Written 2026-09-02 during `tone-prototype`, Round 3, from the offer made at
Rust lunch. The field notes for a real room are `FIELD-NOTES-TEMPLATE.md`; this
sheet is deliberately narrower.*
