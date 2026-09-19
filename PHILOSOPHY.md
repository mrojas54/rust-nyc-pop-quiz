# Philosophy — Rust NYC Pop Quiz

## The one thing

**The quiz is a pretext. The understanding is the product — and the room gets
there by working through it together: going over it again, and the people who
know it telling the people who don't.**

Rust NYC does not need a scoring system. It needs fifty people to look at nine
lines of Rust, find out they do not agree about what it prints, and then work
through it until they do. Everything in this project is judged by whether it
produces that or gets in its way.

*Amended 2026-08-14 on client testimony, during `tone-prototype`.* This said
**the argument is the product** from Stage 1 until the client corrected it:
*"not argument, understanding through repetition and knowledge sharing."*

The correction matters and it is not a synonym. **An argument has sides and a
winner**, and §9 exists because this room's feedback is that everything is
intimidating and people are not allowed to be wrong. A newcomer will not enter an
argument. They will let someone show them how it works. The design beat that
followed from the old wording — *A says this, E says that, someone from each,
why?* — asked two people to defend a position publicly, which is the exact thing
§9 was minted to prevent.

The corpus already agreed with the client. **AC-44**, the criterion this project
calls the one that matters most, is *a beginner can explain the solution to
someone else* — knowledge sharing, not debate. Dimension 6 records an attendee
asking, unprompted, for *"pairs up new people with more experienced engineers."*
And the origin below is that a memorised bank produced **no conversation at
all**; *argument* was an inference drawn from that in Stage 1 and never had
evidence behind it.

This is why the last quiz died. When attendees had seen every question in
dtolnay's bank, the answer arrived before anyone had to work for it. A remembered
question is not an easy question — it is a question that produces *no
conversation at all*, which is worse than a hard one.

It is also why the segment is **one question in the last five minutes of the
night**, and not a round in the middle of it. If the understanding is the
product, the segment has to end somewhere the working-through can keep going.
Anywhere but last, the only way to finish on time is to interrupt the exact thing you were
trying to cause. Placed last, nobody has to stop it — it walks out with them.

One question a night is also eight months of supply from a single batch, which
is the kind of thing principle 8 keeps predicting.

## Principles

### 1. Nobody in the room has seen this question

Freshness is not a feature, it is the precondition. The moment a question is
recognizable, the segment has failed at the only thing it does.

### 2. And nobody can learn a shortcut around it

This is the principle the original PRD missed, and it is the one most likely to
be violated by accident. A quiz can be perfectly original and still become
memorized *at the meta level*: "never pick `does not compile`," "`unsafe` in the
source means the answer is undefined behaviour." A learnable shortcut kills the
conversation exactly as dead as a learnable question, and it is far cheaper for an
attendee to acquire.

Concretely, and enforceable:

- No published distribution of answer categories. Ever.
- No structural feature of a question may correlate with its answer — not the
  presence of `unsafe`, not source length, not topic, not option ordering.
- Correct-answer position is drawn **uniformly and independently of history**,
  and the **generator** is audited on every build.

That last one has now failed four times, and the fourth is the instructive one.

The first deck shipped a visible B, D, B, D from a badly seeded shuffle. The
second shipped four options where five were authored. The third arrived when the
segment became a single question: the balancer, which balanced positions *within
a deck*, had nothing left to balance and returned the same slot every time —
answer at A, every meetup, forever.

The fourth was the *fix* for the third. Balance across the series instead: take
the least-used letter, never repeat last month. It is what anyone would write,
it reads like diligence, and it hands the room a certain answer every fifth
meetup — because enforcing "the counts stay even" means that once four letters
are spent, the fifth is arithmetic. It measured **45.7%** against a 20%
baseline, worse than the bug it replaced. It was caught by a client reading one
sentence of prose describing it, not by any test.

So, stated as a principle rather than a bug:

> **A fairness mechanism that shapes output is an oracle.** If a check can
> change what tonight's answer is, an attendee can run the same check.

Balance and unpredictability are not the same goal and they trade against each
other. Unpredictability wins outright: under a uniform draw an uneven-looking
history is not exploitable — past frequency carries zero information — whereas
enforced evenness is exploitable *by construction*. So the counts are allowed to
look lopsided, letters repeat, and the audit tests the **generator** against
synthetic draws in both tails, where it cannot touch a real night's output.

The through-line across all four: none were caught by reading the code, and the
worst one was introduced *while fixing* the one before it. Audit the mechanism,
not your intentions.

### 3. The machine decides the answer, never the author

A human may write the program, the distractors, and the explanation. A human may
never write down what the program prints. That comes from running it, and
nothing ships that has not been run.

This is the difference between a quiz and a trivia deck, and it is the entire
reason to build software instead of a spreadsheet.

### 4. Say exactly what verification proves — and what it does not

A clean Miri run proves there was no undefined behaviour **on the paths actually
executed**. It is evidence, not a proof. Byte-identical output across five runs
is evidence of determinism, not a guarantee of it.

The verification receipt is shown to the room because it is *interesting*, and
because a claim you display is a claim you have to keep honest. Overstating it
would be the one unforgivable bug in a project about Rust correctness.

### 5. Code must be readable from the back of the room and on a phone

This is the gap no off-the-shelf tool fills — Kahoot caps a question at 120
characters. If the code is not comfortably legible, nothing else matters,
because nobody can form the opinion that working through it starts from.

Monospace, generous size, line numbers, and a scroll container that never makes
the page scroll sideways.

*Amended 2026-08-14, during `tone-prototype` Phase 1c.* This line said **real
syntax highlighting** until the house design system was read. Its `SourceCode`
component renders with no highlighting at all, deliberately, and `PROJECTOR_SPEC`
§4.1 is where the reason becomes visible: the trace signals by highlighting the
lines currently executing and dimming everything outside the focus region. A
second colour channel running underneath competes with the one thing the room is
being asked to look at. **Legibility is the principle; highlighting was an
implementation of it that turned out to work against the trace.** Colour still
never carries a signal by itself — that constraint is unchanged.

*Amended again 2026-09-03, during the T-20 drive — client call: "keep the
colour, love it."* The reason above is airtight while a trace is running and
vacuous when it is not: in the reading phases nothing is competing with
anything, and the room is reading nine lines for thirty seconds. So colour
**returns to the reading phases** and stays **off wherever a trace runs** —
AC-99. This diverges from the design system's `SourceCode` on purpose; the
client carries that to its owner.

### 6. Nothing per-person is recorded

No accounts, no nicknames, no leaderboard, no history. Anonymous totals that
expire with the room. Someone should be able to be confidently, publicly wrong
in a room full of colleagues — that is the whole social contract of the segment,
and a scoreboard quietly destroys it.

### 7. The host plays too

The host gets no answer preview. It keeps them honest, it keeps them in the
room's own working-through, and the person running the segment is having the same
evening as everyone else.

### 8. Prefer the smaller mechanism

The first version of this project specified live generation inside a five-minute
window because it sounded like the same requirement as freshness. It was not.
Every time a requirement seems to demand machinery, check whether it demands the
machinery *at that moment* — supply problems rarely need runtime solutions.

The second time was the round itself. "A quiz" implied several questions, so the
design carried several questions, and with them: navigation, a per-deck balance
mechanism, three deadline write-bursts, fifteen minutes of room time, and a
supply problem that needed solving by October. One question dissolved all of it.
Nothing was traded away — the understanding the segment exists to cause happens once
per night either way, and it happens *better* with somewhere to go afterwards.

Both times the machinery came from a word — *live*, *quiz* — rather than from
the thing we actually wanted. That is worth checking for by name.

### 9. Being wrong has to be the ordinary thing, not the failing thing

*Added 2026-08-14 on client testimony, during `tone-prototype`.*

The feedback Rust NYC gets — surfaced through Women in Rust — is that
**everything is very intimidating, and people aren't allowed to be wrong.**
Principle 6 already protects the *record*: no accounts, no scores, nothing
per-person. That turns out to be necessary and nowhere near sufficient. A
segment can record absolutely nothing and still be the most exposing five
minutes of someone's evening.

This reframes what the segment is for without changing the one thing. An
understanding is the product, and **it is reached by people willing to be wrong
out loud** — so anything that raises the cost of being wrong is destroying the
product, not decorating it.

A quiz is a machine for manufacturing public wrongness. That makes it either
exactly the wrong format for this room or the best intervention available, and
the difference is entirely in how the wrongness is handled. The asset is
specific and no other format has it:

> **Majority wrongness, anonymous, in public.** On the first real question, 24
> of 58 people picked `[1, 2, 3]` — 41% of the room, wrong, together. Nothing a
> host can *say* about it being fine to be wrong does what watching 41% of your
> colleagues be wrong does.

Concretely, and enforceable:

- **The split is shown before the answer, always.** You find out you are not
  alone *before* you find out you were wrong. The order is the entire mechanism;
  revealing first and showing the split afterwards is a different product.
- **Nothing ever marks a person as wrong.** The correct option gets a ✓ because
  that is a fact about the answer. Your own choice gets no ✗ — it gets a count
  of how many people read it the same way you did.
- **The most-chosen wrong answer is the subject of the explanation, not an
  omission from it.** Name it, say how many chose it, and say why it is a
  reasonable reading. In `dedup`'s case it is reasonable: every other language
  would have been right.
- **Asking for help costs nothing.** A newcomer who wants the hint must be able
  to get it without anyone — the host included — being told they needed it.

The failure mode to watch for is the one that looks like kindness: softening the
question, or telling people it is fine to be wrong. Neither works. What works is
showing them, with a number, that most of the room was wrong too.

## Taste

**Nostalgic and technical.** It should look like a well-made terminal, not a
SaaS dashboard.

- Muted technical grays, one warm amber accent. Values live in the brand guide;
  they are not up for reinvention.
- Monospace for everything — body, controls, code, data. Serif only for headings
  at 24px and larger.
- One corner radius (4px), 1px borders, one subtle card shadow. No gradients, no
  glass, no bounce.
- Colour is never the only signal. Correct is a green border **and** a ✓.
- Motion is minimal and respects `prefers-reduced-motion`.
- Every string is written to be read aloud by a host standing in front of people.
  Explanations especially — they are a script, not documentation.
- Every string is also written so that the person who got it wrong is being
  talked **to**, not talked **about**. No congratulating the winners, because
  there are none.

## What this is not

Not a learning platform. Not a leaderboard. Not a question bank to publish. Not
a certification. Not a product with users — a segment with attendees.

---

*Stage 1 of the Tone arc. Amendable: if a prototype or a real meetup contradicts
something here, the artifact is what's wrong. Propose the amendment, update it,
propagate forward.*
