# Design — Rust NYC Pop Quiz

**Status: binding. Written 2026-09-03 at the close of the T-20 drive and made
the build's visual contract the same day on the client's word: *"yes i love
it."*** Everything below describes what `prototypes/C-projector-first.html` and
`prototypes/take-it-home.html` actually do; nothing here is a proposal the
prototype does not already carry. Where this file and the prototype disagree,
the prototype wins and this file is corrected.

The converged direction is **C — projector-first, phone as buzzer.** The code
lives on the big screen and nowhere else; the phone is five letters. The client
stated the assumption in her own words before the takes were compared (A-4,
*"only options to vote on, code and tracing need to be on screen"*), then
confirmed it by driving all three: *"C is it."* Directions A (hands, no phones)
and B (phone-first) stay in `prototypes/` as built comparisons for October's
field notes, not as candidates.

## What the design serves

`PHILOSOPHY.md`'s one thing: *the quiz is a pretext, the understanding is the
product, and the room gets there by working through it together.* Every screen
below is judged by whether it produces that. Two principles do most of the
shaping:

- **§9 — being wrong is the ordinary thing.** The split is shown before the
  answer, nobody is ever marked wrong, the popular wrong answer is named and
  counted, and help costs nothing. AC-93…AC-98.
- **§5 — readable from the back of the room.** The wall sizes its type to the
  question, and legibility is the floor. AC-78, AC-99, and AC-33's sibling: on
  the wall the source must *fit*, because container-scroll was a phone escape
  hatch and A-4 took the phone away.

## The three surfaces

| Surface | Who sees it | What it carries |
|---|---|---|
| **The wall** | the whole room | brand line top-left; the join strip; the source, line-numbered, full-width; five options beneath in two columns; the split as bars with counts; the trace; the reveal; the handover |
| **The buzzer** | a participant, on their own phone | the room code as the room's identity; five letters; *Show me a hint*; after the split, a **count** beside their letter, never a ✗; at the end, *nothing about you was recorded* |
| **The host phone** | the host | one button per phase transition; `←` `→` to step the trace; the read-aloud script at the reveal; **no answer preview** (§7) |

Neither source nor trace is ever rendered on a participant device (AC-32 as
rewritten). The buzzer is deliberately dull.

## The segment, in seven phases

The order **repeat → resolve → release** is the mechanism, not a preference
(AC-97). Host controls step it; nothing else does.

| # | Phase | Wall | Buzzer | Host phone |
|---|---|---|---|---|
| 1 | **Title** | *Time for a pop quiz.* One line. Join strip: *join @ ‹short link›* | code, waiting | *Put it on the screen* |
| 2 | **Question live** | source in colour, options beneath, *join @ ‹link›*, no timer | A–E, *Show me a hint* (private — the host is not told) | *Close answers* |
| 3 | **Answers closed** | *answers are closed*, nothing else | locked | *Show the room its split* |
| 4 | **The split** | five bars with counts, *N of M in the room answered*, **no answer** | your letter gets a count: *N of us read it the same way* | the walk-through button |
| 5 | **Work through it** | *Let's walk it. Still no answer. Nobody has to say anything.* Trace stepped by the host — highlight-and-dim, *Step N of M*, **no colour, no ✓, no receipt** | unchanged | `←` `→`, the words for each step |
| 6 | **Reveal** | ✓ on the correct option; the most-chosen wrong answer named and counted; the receipt; the trace steppable again | count stays a count | *Read it aloud* — the three-beat script |
| 7 | **Released** | *Let's go to the bar.* The link, at 40px, and a QR. Nothing competes with it | *nothing about you was recorded* | *Release the room*, then *Run it again* |

The client, from the host's chair, on phase 5: *"as the host i liked setting
the pace."* On 6 and 7: *"love it."*

**The reveal script has three beats**, and the order is the point: what
happened, plain enough for a beginner to repeat (AC-44); why *N of us* read it
the other way, with the count, which is the beat that makes being wrong ordinary;
then what to remember, the real decision, which carries on at the
bar. This makes `explanation` three fields rather than one; `SPEC.md` §3.1
carries the model (`what`, a `why_tempting` per incorrect option, `takeaway`).

## Voice

Every string is written to be said by a host standing in front of people, and
to talk **to** the person who got it wrong, never **about** them. In practice:

- **Short declaratives, lower-case where the room reads them.** *answers are
  closed.* *join @ …* *Let's walk it.*
- **Counts, not verdicts.** *N of us read it the same way.* Never *wrong*,
  never ✗, never a score. Correct gets a ✓ because that is a fact about the
  answer.
- **No instruction to speak, ever** (AC-98). *Nobody has to say anything* is
  the only sentence about talking, and it releases rather than asks. Two
  earlier lines — *someone from each, why?* and *find someone who picked
  something else* — were written, cut, and are recorded as the shape to avoid.
- **The client's own lines where they exist.** *Time for a pop quiz.* *Let's go
  to the bar.* They beat anything written for them.
- **The verification receipt says exactly what it proves** (§4): in plain words: run five
  times on one kind of computer with the same output, Miri found no problems in
  the code that ran, and the machine checked the answer only. Never more.

## Aesthetic

Inherited from the **Rust NYC Design System** (the house brand, claude.ai/design
`d608a53a…`), not reinvented. `prototypes/_shared/tokens.css` §1 is its four
token files verbatim.

- **Type.** Cascadia Mono for everything — body, controls, code, data.
  Instrument Serif only for headings at 24px and above. The fonts are not
  vendored in the prototypes; a build ships them.
- **Colour.** Muted technical greys, one warm amber accent, white source well.
  **Lights up** — the light treatment, endorsed by the brand's own imagery
  author (AC-80 answered; *no dark mode in v1* stands). Colour is never the
  only signal (AC-40).
- **Syntax colour, scoped** (AC-99, A-7). Present in phases 2–4, muted
  values rather than an editor theme; phases 1 and 7 render no source. Absent in 5 and 6, where the trace
  signals by highlight-and-dim and nothing may compete with it. This diverges
  from the design system's `SourceCode`, on purpose; the client carries it to
  the component's owner.
- **Form.** One radius (4px), 1px borders, one card shadow. No gradients, no
  glass, no bounce. Motion minimal and under `prefers-reduced-motion`. It should
  look like a well-made terminal.
- **Every prototype carries a persistent `PROTOTYPE` badge.** A mockup is
  never mistakable for the product.

## The wall's type model

The wall **sizes itself to the question; legibility is the floor.** Nothing is
chosen by hand.

1. The layout is **source full-width, options beneath in two columns** — the
   client's call, touchpoint T-20, item 15. Full-width buys line width; the options block
   spends ~200px of height.
2. The smallest comfortable size is **derived** from screen width and back-row
   distance (the 15ft / 20ft venue guess gives a 14.2px floor). The type grows
   to fill the wall for a short program and shrinks toward the floor for a
   long one.
3. After rendering, the well is **measured** and the type refitted until it fits
   or hits the floor, up to six passes; the wall refits on entering every phase
   that renders the source. *Fits, legibly* is a measured claim.
4. When a question will not fit even at the floor, the wall shows a **red edge
   on the side that lost content** and says so. That is the room's limit, not
   the generator's constant.

At the venue guess: `q3` (5 lines) at ~22px, `q7` (69 characters) at 22.1px,
`q1` (16 lines) does not fit — the bound is set by `SPEC.md` §5.2 and it is the
**rehearsal's screen measurements** (width *and* height; the projector may not
be 16:9) that set the real number. The wall lays out at 1120px design size and
scales as a unit.

## What is deliberately absent

- No code or trace on any phone. No timer on the wall. No scoreboard, name,
  account, or history (§6).
- No public hint: the hint is pulled privately on the buzzer and the host phone
  does not know (AC-48 as rewritten, touchpoint T-13). The host has no hint control.
- No answer preview for the host (§7).
- No ✗, anywhere. No congratulating anyone.
- No instruction to talk to anyone (AC-98).
- The room code is **off the wall**; the way in is a short link (AC-28 as
  rewritten). The code remains the room's identity on the phones.
- No answer-position balancing of any kind. The slot is drawn from the date
  (AC-23/23a/23b; `PHILOSOPHY.md` §2).

## Take it home

The second surface, behind the link on the released wall: the last meetup's
question, alone, at the reader's own pace — the trace steppable both ways,
the three-beat explanation, the receipt. §9's private pace lives here (touchpoint T-15:
wall host-stepped, phones step freely). It carries no room state, and
no count — `tone-architect` settled the AC-95 conflict as D-12 (2026-09-03).

## The design-system divergences, all deliberate

| Divergence | Why | Who carries it |
|---|---|---|
| Syntax colour in the reading phases (`SourceCode` renders none) | the trace reason only holds while a trace runs — §5, AC-99 | the client, to Cole |
| Room code off the wall; a short link instead | the client's join strip; the link can carry the code | AC-28 rewritten |
| `explanation` becomes three fields | AC-44 needs a beat a beginner can repeat, and §9 needs the popular wrong answer named | `SPEC.md` §3.1 (`explains`), AC-95 as read by D-9 |

## Hypotheses this design rests on

- **Screen 15ft, back row 20ft.** A guess. The rehearsal (`mvp/PRACTICE-RUN.md`)
  measures width, height and the distance to the last occupied row, and every
  number in the type model moves with them.
- **AC-91's 30 seconds.** The client had an opinion at 10 seconds on a question
  she had seen many times; the real sample is an attendee's first look.
- **Participation rate.** Never measured; it sizes the deadline burst (AC-52).

## For the build

The converged take is reproduced **one-to-one**: every designed control present,
even if it only toasts *not yet implemented (AC-x)*. The prototype, not this
file, is the reference where they disagree. Everything this file once listed
as open for `tone-architect` is settled in `SPEC.md` (the static fallback §12,
the used-question ledger G-10, the explanation model §3.1, fonts D-14, AC-21's
measurement at HC-2); the real short link behind the placeholder
`bit.ly/jnfnvd` is the client's, `BUILDPLAN.md` H-2.

---

*Stage 2 of the Tone arc, `tone-prototype`. Referenced from `CLAUDE.md`. If the
built product or a real meetup contradicts this, the prototype and this file
are what change — propose the amendment, update, propagate.*
