# Field notes — attendee evidence (T-6, room half)

Instrument **and** capture form. Copy to `mvp/<YYYY-MM-DD>/field-notes.md`, fill
the blanks in place, and that copy becomes the evidence.

**Runner:** `______________________`  **Date:** `__________`  **Question:** `______`

**This does not have to be run by the client.** Everything below is written so a
co-organizer can run it cold. If you are a co-organizer picking this up: you are
capturing the only user evidence this project has, and rough numbers beat none.

**Why this exists:** `sequence/USER_STORIES.md` has three questions marked
*"Open — needs attendee evidence"* and several `felt` criteria nobody has ever
observed in a room. One of them — *do phones help or hurt* — decides whether
Stories B1 and B5 survive, which is most of the build.

**A hand-run projector deck with no phones is the hands-only condition** those
stories hang on. You do not have to set anything up to test it; you only have to
watch.

---

## Before the segment — 30 seconds, while someone else is talking

- [ ] Note the wall-clock time you start: `______`
- [ ] Pick **one beginner** to ask afterwards. Someone who'd call themselves new
      to Rust. → `______________________`
- [ ] Pick **one person sitting at the back**. → `______________________`
- [ ] If a second organizer is free, hand them the "During" section — whoever is
      hosting cannot take notes. If nobody is free, do only the two ⏱ items and
      the participation count; they need a clock or a glance at the room and
      cannot be reconstructed afterwards.

**Run it without a timer on screen.** That is the deliberate test (see Q1).

### Two measurements of the room itself — take these once, ever

Added 2026-08-14, during `tone-prototype`. **These two numbers decide how long a
question is allowed to be**, and both are currently guesses.

- [ ] **How wide is the projected image?** Pacing it heel-to-toe is fine — a
      foot or two of error does not matter. → `______ ft`
- [ ] **How far back is the last row anyone actually sits in?** Not the back
      wall — the last occupied row. → `______ ft`

*Why it matters, in one line:* legible type is roughly cap-height ≥ distance
÷ 150, so those two numbers set the smallest readable text on the wall, and that
sets how many lines fit. At the current **guess** — 15ft wide, back row at 20ft —
the wall holds **16 lines**, and `q1` in the bank is exactly 16. There is no
headroom, and nobody has checked the guess.

**Also worth one line:** could the person at the back read the code without
leaning in or asking? That is **AC-78**, and it is the criterion Direction C
lives on.

---

## During — two clocks, one ratio, one shape

⏱ **Seconds from the code appearing to the first person visibly having an
opinion** — a hand, a nudge, someone turning to their neighbour and pointing at
the screen. Not the first *answer* — the first *opinion*.

> **AC-91 target: ≤ 30 seconds.** Measured: `______ s`

⏱ **Total segment length**, question on screen → you stop talking.

> **AC-89 target: ≤ 5 minutes.** Measured: `______ min ______ s`

📌 **Participation — the highest-value number on this page after the beginner
ask.** `AC-52` counts *concurrent participants*, and what sizes the deadline
write burst (`AC-54`, the highest-risk moment in the system) is
**attendance × participation rate**. Only attendance has ever been measured. 200
in the room at 40% is 80 concurrent sessions; at 90% it is 180. Guess
generously rather than leaving it blank.

> **took part at all** `______` of `______` in the room

**Rough vote split** (show of hands, one line, don't be precise):

> A `___` B `___` C `___` D `___` E `___` · didn't vote `___`

**Anything the host had to apologise for or improvise around** (AC-51):

> `________________________________________________`

---

## After — three questions to three people, ~2 minutes

**1. The one that matters most — AC-44.** To the beginner you picked. Ask it
*exactly* this way, and do not help them:

> *"Can you explain to me why the answer was what it was?"*

Then shut up and let them try. What you are testing is the **explanation**, not
them — if they can't, the explanation failed.

> Could they? `yes / partly / no`
> What they actually said: `________________________________________________`
> Where they got stuck: `________________________________________________`

**2. Legibility from the back — AC-38 / AC-78 / AC-80.** To the back-row person:

> *"Could you read the code from there, or were you guessing?"*

> `read it fine / squinted / couldn't` · lights were `up / down / mixed`
> `________________________________________________`

**3. The receipt — an open question.** To anyone who reacted to the verification
panel, or to nobody if nobody did (which is itself the finding):

> *"Did the verified-by-machine part land, or was it noise?"*

> `interesting / neutral / noise / nobody noticed`
> `________________________________________________`

---

## The three open questions — the runner's read, one line each

These change the build. The judgement of the person in the room beats any metric.

**Q1 — Is a timer wanted, or does it cut arguments short?**
Run without one. Did the room need one?

> `wanted / not wanted / didn't matter` — `______________________________`

**Q2 — Do phones help or hurt?** ← *the consequential one*
This was hands-only. Was the argument better or worse than a phone vote would
have been? Be honest even if the answer shrinks the project — **especially** then.

> `phones would help / hands are better / genuinely unclear`
> `________________________________________________`
>
> If *hands are better*: Stories B1 and B5 lose most of their weight, AC-52…AC-55
> (200 concurrent, deadline write burst — the single biggest engineering risk in
> the project) may stop being load-bearing, and the build gets much smaller.
> That is a legitimate and cheap outcome. It is not a failure.

**Q3 — Difficulty (AC-88).** The question was authored at difficulty `___` of 5.
Did the room find it that?

> `too easy / about right / too hard` — `______________________________`

---

## Anything that surprised you

The most valuable line on this page is usually the one there was no box for.

> `________________________________________________`
> `________________________________________________`

---

## After the meetup — one line of bookkeeping

- [ ] **Confirm the question was actually run**, then check
      `mvp/answer-history.json` says so. That file is written at **build** time,
      not run time (see `sequence/run-state.md`, AC-92 defect), so a deck built
      for a meetup that did not happen records a question as spent when nobody
      ever saw it. If the segment did not run, the question is still fresh.

---

*The records half of T-6 is `sequence/research/06-attendees.md` — population and
voice from the group's own Meetup/Luma exports. This file is the part only a room
can answer. Once filled, it feeds `tone-prototype`, and Q2's answer may reopen
`USER_STORIES.md`.*
