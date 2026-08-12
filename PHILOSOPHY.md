# Philosophy — Rust NYC Pop Quiz

## The one thing

**The quiz is a pretext. The argument is the product.**

Rust NYC does not need a scoring system. It needs fifty people to look at nine
lines of Rust, disagree out loud about what they print, and find out together
why they were wrong. Everything in this project is judged by whether it produces
that moment or gets in its way.

This is why the last quiz died. When attendees had seen every question in
dtolnay's bank, the answer arrived before the argument could start. A remembered
question is not an easy question — it is a question that produces *no
conversation at all*, which is worse than a hard one.

## Principles

### 1. Nobody in the room has seen this question

Freshness is not a feature, it is the precondition. The moment a question is
recognizable, the segment has failed at the only thing it does.

### 2. And nobody can learn a shortcut around it

This is the principle the original PRD missed, and it is the one most likely to
be violated by accident. A quiz can be perfectly original and still become
memorized *at the meta level*: "never pick `does not compile`," "`unsafe` in the
source means the answer is undefined behaviour." A learnable shortcut kills the
argument exactly as dead as a learnable question, and it is far cheaper for an
attendee to acquire.

Concretely, and enforceable:

- No published distribution of answer categories. Ever.
- No structural feature of a question may correlate with its answer — not the
  presence of `unsafe`, not source length, not topic, not option ordering.
- Correct-answer positions are balanced and shuffled, and **audited on every
  build**. The first deck we built shipped a visible B, D, B, D pattern from a
  badly seeded shuffle. It was caught by an audit, not by care.

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
because nobody can form the opinion the argument needs.

Monospace, generous size, real syntax highlighting, a scroll container that
never makes the page scroll sideways.

### 6. Nothing per-person is recorded

No accounts, no nicknames, no leaderboard, no history. Anonymous totals that
expire with the room. Someone should be able to be confidently, publicly wrong
in a room full of colleagues — that is the whole social contract of the segment,
and a scoreboard quietly destroys it.

### 7. The host plays too

The host gets no answer preview. It keeps them honest, it keeps them in the
argument, and it means the person running the segment is having the same
evening as everyone else.

### 8. Prefer the smaller mechanism

The first version of this project specified live generation inside a five-minute
window because it sounded like the same requirement as freshness. It was not.
Every time a requirement seems to demand machinery, check whether it demands the
machinery *at that moment* — supply problems rarely need runtime solutions.

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

## What this is not

Not a learning platform. Not a leaderboard. Not a question bank to publish. Not
a certification. Not a product with users — a segment with attendees.

---

*Stage 1 of the Tone arc. Amendable: if a prototype or a real meetup contradicts
something here, the artifact is what's wrong. Propose the amendment, update it,
propagate forward.*
