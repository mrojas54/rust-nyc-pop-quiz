# Run state — Rust NYC Pop Quiz

The resume anchor for the whole Tone arc. Every stage reads this first on invoke.

---

## ▶ RESUME HERE — handoff written 2026-09-03, Stage 2 COMPLETE

**Stage 2 `tone-prototype` is DONE. T-20 answered: *"yes i love it."*** The
client drove Direction C end to end, sixteen items were applied, and the word
was said about the whole. **`DESIGN.md` is binding.** The stage handed off to
**`tone-architect`** the same day — see the Stage 3 section at the bottom of
this file for where that stands. Work continues on `ai-c11-cc/tone-architect`;
the prototype branch has a PR to `main` awaiting merge.

**First four things, in order:**

1. Read this file top to bottom. **`PHILOSOPHY.md`'s one thing CHANGED today
   (A-5)** — read it before touching anything user-facing, because every stage
   before this one was written against the old sentence.
2. `git checkout ai-c11-cc/tone-prototype` — pushed, ~50 commits ahead of `main`,
   **no PR**, because the stage is not finished.
3. Open `prototypes/C-projector-first.html` and **drive all seven phases**. That
   is the converged direction, and `DESIGN.md` describes it. **`sequence/T-20-drive-guide.md`
   is the client's closing drive, now fully driven.** `take-it-home.html` is the
   second surface. A and B are historical takes, not candidates — keep them,
   October compares against built things rather than hypotheses.
4. **The skill is disabled.** `tone-prototype` is `"off"` in
   `~/.claude/settings.json` → `skillOverrides`, so the Skill tool refuses it.
   Read `~/.claude/skills/tone-prototype/SKILL.md` directly.

### What changed today — the thesis, not a detail

- **A-5.** *"The argument is the product"* → **"the understanding is the product,
  and the room gets there by working through it together."** Client testimony.
  An argument has sides and a winner, which is what §9 exists to prevent.
- **A-4.** The source and its trace are **never** on a participant device. This
  eliminated Directions A and B before the drive-through confirmed C.
- **AC-97 / AC-98 minted.** The walk-through phase, and the prohibition that
  **nothing obliges a participant to speak.**

### What changed 2026-09-02 — Rust lunch, and a rehearsal is on offer

- **First attendee contact of the arc.** Two regulars at Rust lunch, cold —
  nothing shown. *Remembered the quiz, liked the idea, liked the idea of voting
  anonymously* (**A-6**). **They offered to help run a practice run** before the
  October meetup; date open. It retires the host-side unknowns before October's
  one shot. Sheet: `mvp/PRACTICE-RUN.md`. Full record at the bottom of this
  file (T-18).
- **It spends `q3`.** C's vote, split and reveal are bound to `q3`'s mock room
  data, and the two will be in October's room. Bank 8 → 7, by hand.
- **`mvp/README.md` was never swept for A-5 or AC-98** — its run-of-show still
  says *"Someone who said E — why?"* and *"go argue about it downstairs."*
  Flagged, not fixed (T-19).

### Open, and both need a person rather than an agent

| | |
|---|---|
| ~~**T-20**~~ | **Answered 2026-09-03: *"yes i love it."*** Sixteen items, all applied; AC-93 answered; T-17 answered; AC-28 rewritten. `DESIGN.md` binding. Stage closed. |
| **T-18** | **The rehearsal.** When, where — the venue, if at all possible, because the two room measurements are the highest-value output — and confirm `q3` is the one to spend. Two regulars offered at Rust lunch. Sheet written: `mvp/PRACTICE-RUN.md`. The meetup is **October**; the rehearsal precedes it and has no date yet. Testimony captured (A-6). |
| ~~**T-19**~~ | **Swept 2026-09-03.** Six places in `mvp/README.md`, plus one leak the A-5 sweep had missed in the product itself: the reveal script's third beat was headed *The bit worth arguing about* — read aloud by the host, shown on take-it-home. Now *talking about*. |
| ~~**T-17**~~ | **Answered 2026-09-03: keep the colour.** A-7, AC-99 minted, §5 amended. Cole's view is still worth having, and the client carries it; nothing waits on it. |
| **`felt` never answered** | ~~**AC-93** — does split-before-answer land?~~ **Answered 2026-09-03: *"i like seeing the split before the answer."*** **AC-21** — the review-surface clock, never once measured — remains, and is the client's alone. |
| **Never collected** | The client's list of what annoyed her in B and C. That list *is* the next iteration rounds and it will evaporate. **Ask for it.** |
| **Field notes** | The client will measure the projected image width and the distance to the last occupied row. **15ft / 20ft is a HYPOTHESIS** and everything about question length rests on it. |

### Traps that will cost an hour, or ship a defect

- **Do not reintroduce the argument framing** (A-5). It survived in copy in seven
  files and was swept; ordinary English uses — *"the build-vs-borrow argument"* —
  were deliberately left.
- **Do not add anything telling participants to talk to each other** (AC-98).
  It has been written twice and cut twice in one day: *"someone from each, why?"*
  and *"find someone who picked something else."* An instruction is an obligation
  however gently worded.
- **Do not mark the answer or show the receipt during `5 · work through it`**
  (AC-97). The first build fell through to the reveal branch and did exactly
  that, which makes the phase pointless.
- **Syntax colour is ON only where no trace is running** and must stay off in
  `work` and `reveal` — that is the whole reason it was allowed back.
- **The wall lays out at 1120px design size and scales as a unit.** Sizing it
  with `width:100%` shrinks the box while its px contents do not, and the options
  land on top of the source. Type is **derived** from screen size and back-row
  distance, never chosen.
- **Answer-position balancing is still the oldest trap in the repo** — four
  regressions, one introduced while fixing another. `PHILOSOPHY.md` §2 first.
- **`mvp/answer-history.json` is a record, never an input**, and a built deck is
  not a run segment (the AC-92 mechanism defect, still unfixed, → `tone-architect`).

**Carried forward, none blocking:** October field notes (T-6 room half), the
AC-92 mechanism defect, the never-measured participation rate, the static
fallback (`PROJECTOR_SPEC` §6), and the **AC-33 sibling** — on the wall the
source must *fit*, because container-scroll was a phone escape hatch and A-4 took
the phone away.

---

| | |
|---|---|
| **Stage** | **3 — `tone-architect`, opened 2026-09-03.** Stage 2 (`tone-prototype`) **complete 2026-09-03** — *"yes i love it."* Stage 1 (`tone-initiation`) **complete 2026-08-13**. |
| **Phase** | **2 — converging on Direction C. T-12 answered 2026-08-14: *"C is it."*** The fan-out is closed and iteration on C has not started. Queue: source full-width with options beneath; type size defaulted to the real venue (**needs the client's screen size and room depth**); the handover into `take-it-home`; and the `felt` judgments still unmade — AC-80, AC-93, AC-21. *Prior:* **1c — takes reconciled onto the house design system, awaiting the drive-through.** Five clickable prototypes in `prototypes/`, all on the real verified batch, all now on the design system's tokens, Cascadia Mono, `SourceCode`, and `PROJECTOR_SPEC` §4.1's trace model. **A-3 amendment: `PHILOSOPHY.md` §9 and Story B11 / AC-93…AC-96** — being wrong has to be the ordinary thing. Answered: T-11, T-13 (AC-48 rewritten), T-14 (A stays), T-15 (trace stepped both ways). **T-12 — the drive-through — is the one still open, and nothing converges without it.** Carried in from Stage 1, none blocking: T-6 room half (October), AC-92 mechanism defect (→ architect), participation rate unmeasured. New gap for architect: the static fallback. **Resume pass 2026-08-14: the reconcile reached the criteria** — `PHILOSOPHY.md` §5 amended, **AC-32 rewritten** (no syntax highlighting), room code now six characters (`KMT4XW`). |
| **Repo** | `~/rust-nyc-pop-quiz` (canonical) |
| **Branch** | `ai-c11-cc/tone-prototype`, off `main` @ `9d3fff7`. *(Stage 1 ran on `ai-c11-cc/tone-initiation` and three follow-on branches, all merged to `main` via PRs #1–#4.)* |
| **Remote** | `git@github.com:mrojas54/rust-nyc-pop-quiz.git` (private) |
| **Client** | Michelle Rojas (Rust NYC organizer) |
| **Opened** | 2026-08-11 |

## What this run is

Not a fresh commission. A 975-line `PRD.md` and a validated HTML state-explorer
prototype already exist, both produced **outside** the Tone arc. Stage 1's
artifacts did not exist. The client's Phase-0 call was **full re-open**: the
concept is treated as genuinely undecided, including whether to build it at all.
The PRD is one input among several and may be substantially rewritten.

## Prior artifacts inherited

| Artifact | Location | Standing |
|---|---|---|
| `PRD.md` (975 lines) | repo root | **Draft — blocked on feasibility evidence.** Input, not contract. Phase 2 recommends substantial revision. |
| State-explorer prototype | `~/Downloads/Pop Quiz - Rust NYC (offline).html` | 20 named states, both lanes, `RustNYCDesignSystem` component set. **Design work survives the fork** — see synthesis §4. Not yet in-repo. |
| Superseded copy | `~/Documents/rust-nyc-quiz` | Marked `SUPERSEDED.md` 2026-07-25. Do not work there. |
| Riptide workspace copy | `~/.humanlayer/workspaces/set-up-initial-github/rust-nyc-pop-quiz` | Byte-identical PRD. Branch `ai-humanlayer-cc/set-up-initial-github`. |
| **Rust NYC Design System** *(missed at intake — found 2026-08-14)* | claude.ai/design · `d608a53a-9b53-40c0-a64d-7017613f1956` | **The house brand.** 9 components incl. `SourceCode`, `ChoiceButton`, `RoomCode`, `Timer`; token CSS; **Cascadia Mono** + Instrument Serif with the fonts bundled; Ferris Liberty brand imagery; `BRAND_STYLE_GUIDE.md`; brand-voice guidelines. |
| **Pop Quiz design project** *(missed at intake — found 2026-08-14)* | claude.ai/design · `c3ae9b25-bbd3-4e96-b402-ef7ea2d9d387` | `DESIGN.md`, **`PROJECTOR_SPEC.md`**, 8 designed screens (`.dc.html`) incl. Projector / Host Controls / Organizer Setup / Participant ×4, a `design_handoff_pop_quiz/` bundle, and ~20 screenshots including `01-reveal-step5.png`. |

**Intake correction, 2026-08-14.** Stage 1 recorded the offline HTML as *"the
state-explorer prototype"* and stopped there. It is not a loose artifact — it is
the compiled output of a **bound design system** and a **written design spec**,
both of which existed and neither of which was inventoried. Consequence: Stage 2
Phase 1 built `prototypes/_shared/tokens.css` from `PRD.md` §Brand And
Accessibility when `tone-prototype`'s own contract says *where a house brand
exists, inherit it and say so*, and `PHILOSOPHY.md` says the brand values *are
not up for reinvention*. The palette came out identical — both derive from the
same PRD — but the typography, the component set, and several interaction rules
did not. See Phase 1c.

## Phase 1 — Research: complete

Five dimensions, 1,811 lines, 101 unique cited sources. Dossier in
`sequence/research/`. Fold-back in `sequence/research/00-synthesis.md`.

| # | Dimension | Headline |
|---|---|---|
| 01 | Problem & people *(mandatory)* | `rust-quiz` = 37 questions in ~7 years. No mainstream live-quiz tool can display a 35-line Rust program. **Verdict amended by client testimony** — see below. |
| 02 | Discord seam *(mandatory)* | Simpler than feared: role IDs immutable, no bot needed, Administrator cannot bypass by construction. 15-min grace window undersized vs. real outages (20 min–3 h). Refresh tokens rotate (undocumented). |
| 03 | Economics *(mandatory)* | Sponsorship worth $3–15/yr — **not load-bearing**. Real cost is Val Town Pro ~$252/yr, tail to ~$2,020/yr. |
| 04 | Generation & verification | Feasible but unproven. UB path ≈187s = 62% of the 5-min deadline; ~48.8% of rooms hit it. **`unsafe` in source is a near-perfect tell for UB-targeted questions.** |
| 05 | Realtime substrate | Val Town: needs measurement, lean skeptical. Turso single-writer vs. 200-write deadline burst. Durable Objects is a textbook fit and free at this scale. |
| 06 | Attendees *(added 2026-08-12, T-6)* | Current venue caps at **110 RSVPs**, ~86% show rate — but every event sold out with a waitlist and the group has hit **200 twice**, so that is a venue cap, not a demand ceiling (amendment A-2). **25–27% newcomers every meetup**, 15% core. 41 free-text responses, **none asking for a quiz**; five asking for beginner-friendly content. **Participation rate never measured** — the real unknown behind AC-52/AC-54. |

## Living-artifact amendments

| # | Artifact | Change |
|---|---|---|
| A-1 | `research/01-problem-and-people.md` | Verdict "problem unproven; solution in search of a problem" **withdrawn**. Client testimony — *"attendees had gotten all answers over and over, we stopped doing dtolnay's quiz at all"* — is primary evidence that the problem is observed, acute, and terminal. Amendment recorded in-file; original retained for the record. |
| A-3 | `PHILOSOPHY.md` (new §9), `sequence/USER_STORIES.md` (new Story B11 / AC-93…AC-96) | **Client testimony 2026-08-14, unprompted:** *"i want newbies to be able to be wrong. the feedback we get and it's surfaced in women in rust is: everything is very intimidating. people aren't allowed to be wrong."* Principle 6 protected the **record**; nothing protected the **moment**, and they are not the same thing. Nothing anywhere in the corpus had mentioned intimidation — Dimension 6 §4b could see the beginner *want* but had no account of the *cause*. Same evidence class as A-1. Details in Stage 2 Phase 1b below. |
| A-4 | `sequence/USER_STORIES.md` (**AC-32** rewritten) | **Client call 2026-08-14, during the T-12 drive-through:** *"AC-32, should not put it on the phone, only options to vote on, code and tracing need to be on screen, maybe explanation and summary."* AC-32 asked how well source reads on a phone; the answer is that it should not be there. Rewritten to: neither source nor trace is rendered on any participant device. **Eliminates Direction B** (its assumption is that the phone carries the code) **and Direction A** (*"only options to vote on"* presumes a device), leaving **C** — stated by the client in her own words, not picked off a menu. Open: explanation/summary on the device, and with it §9's private pace. |
| **A-5** | `PHILOSOPHY.md` (**the one thing**, §5, §7, §8, §9 and principle 2), all four takes | **Client testimony 2026-08-14, and the largest amendment in the arc — it changes the thesis, not a principle.** *"no, work through it. not argument, understanding through repetition and knowledge sharing."* Stage 1 wrote **"the quiz is a pretext, the argument is the product"** and everything downstream was judged against it. **An argument has sides and a winner**, which is what §9 exists to prevent — the room's own feedback is that everything is intimidating and people are not allowed to be wrong, and a newcomer will not enter an argument. The corpus already agreed with the client and nobody noticed: **AC-44**, called the criterion that matters most, is *a beginner can explain the solution to someone else* — knowledge sharing, not debate — and Dimension 6 records an attendee asking unprompted for *"pairs up new people with more experienced engineers."* The origin story is that a memorised bank produced **no conversation at all**; *argument* was an inference drawn from that in Stage 1 and never had evidence behind it. New one thing: **the quiz is a pretext, the understanding is the product, and the room gets there by working through it together — going over it again, and the people who know it telling the people who don't.** |
| **A-6** | `research/06-attendees.md` (§4a, §7) | **Attendee testimony via the client, 2026-09-02.** Two regulars at Rust lunch, shown nothing: *"remembered the quiz, liked the idea, liked the idea of voting anonymously."* §4a's *nobody asked for a quiz* is narrowed to *nobody in the records*; §7's *no evidence either way* becomes weak evidence for. First attendee-side corroboration of **§9**'s mechanism — anonymity — from the people who least need the cover, and a second source that the dtolnay quiz ran and is remembered (not that it was memorised; A-1 still rests on the client). Original text retained. |
| **A-7** | `PHILOSOPHY.md` §5, `sequence/USER_STORIES.md` (**AC-99** minted) | **Client call 2026-09-03, T-20 drive: *"keep the colour, love it."*** §5's 08-14 amendment removed highlighting because it competes with the trace; that reason holds only while a trace runs. Colour returns to the reading phases and stays off in the walk-through and reveal. Minted as **AC-99** under Story B8 rather than a third revision of AC-32, which no longer says anything about highlighting. Deliberate divergence from the design system's `SourceCode`; the client carries it to Cole. **T-17 closed.** |
| A-2 | `research/06-attendees.md` §2, §6 | Finding *"AC-52's 200 is ~2× the observed ceiling"* **withdrawn**. The 110-RSVP figure is a **venue supply cap, not a demand ceiling** — every event sold out with a waitlist. Client testimony: bigger venues in progress, **Jun 26 Summer Social 200 and sold out**. AC-52 stands at 200. Replaced by a sharper open variable: the **participation rate**, never measured. Original reasoning retained in-file. |

## Keystone decisions

| Decision | Resolution | Why |
|---|---|---|
| Build vs. borrow | **Resolved — build.** Borrowing rejected. | Every off-the-shelf tool fails on code display, which is disqualifying for a program-output quiz. |
| Live vs. batch generation | **Resolved — batch.** Client call, T-4. | Freshness is a supply problem, not a runtime problem. Decoupling dissolves 4 of 5 top risks. |
| Participant-facing host | **Leaning Durable Objects** over Val Town | Better fit for one-authoritative-room state, and free at this scale vs. ~$252/yr with an unpassed capacity gate. |
| Generation substrate | Deferred — matters far less once batch | An offline job that fails costs nothing and reruns. |
| Does this get built at all | **Resolved — yes**, as two decoupled components | Problem confirmed by primary evidence. |

## Decisions logged

| # | Decision | Rationale |
|---|---|---|
| D-1 | Home is `~/rust-nyc-pop-quiz` | `SUPERSEDED.md` is explicit; only this copy has the remote. |
| D-2 | Work on `ai-c11-cc/tone-initiation` | Initiation may rewrite the PRD; the existing branch shouldn't absorb that silently. |
| D-3 | Initiation scope = **full re-open** | Client call, Phase 0. |
| D-4 | `ECONOMICS.md` in scope | Client call. Written; conclusion inverted the client's stated risk. |
| D-5 | Attendee interviews in scope | Client has access. Not yet conducted — see T-6. **First contact 2026-09-02** at Rust lunch: two regulars, cold, no testimony captured; a rehearsal offered → T-18. |
| D-6 | Borrow option (C) rejected | Code display is disqualifying across all candidates. Recorded rather than escalated because the evidence is one-sided. |
| D-7 | **Segment = one question, 3–5 min, scheduled last.** Client call, 2026-08-12. | The argument is the product and an argument needs somewhere to go; anywhere but last, finishing on time means interrupting it. Shrinks the build (no navigation, one write-burst, ~5 min room life) and turns the 8 verified questions into ~8 months of supply. Minted Story B10 / AC-89…AC-92 and forced a rewrite of AC-23 — see below. |

## Conflicts

| # | Conflict | Status |
|---|---|---|
| C-1 | PRD's anti-gaming objective vs. its own published answer prior | **Resolved in synthesis §2** — and found to be worse than the PRD knew: `unsafe`-implies-UB is a second, unnoticed tell. Fix is cheap but must be deliberate. |
| C-2 | PRD Draft-blocked on two gates, yet a full spec and prototype were built atop it | **Resolved in synthesis §3** — decoupling removes one gate entirely; the other (live capacity) survives and is the project's one real engineering risk. |

## Touchpoints

| # | Phase | Subject | Status |
|---|---|---|---|
| T-1 | 0 | Initiation scope · client & user access · economics | **Answered** 2026-08-11 |
| T-2 | 0 | Opening interview — origin, history, purpose, timing | **Answered** 2026-08-11 |
| T-3 | 2 | Economic-model dialogue | Folded into T-4 — cost does not decide the fork |
| T-4 | 2 | Hard fork — build as written / build something else / don't build | **Answered** — decouple generation (option B) |
| T-5 | 4 | Stories review — AC-1…AC-88 | **Answered** 2026-08-11 |
| T-6 | 1–2 | Attendee interviews (client has access) | **Records half answered** 2026-08-12 → `research/06-attendees.md`. **Room half slipped to October** — Aug 12 skipped, Aug 20 unavailable, Sept is RustConf → `mvp/FIELD-NOTES-TEMPLATE.md`. **First contact 2026-09-02** — two regulars at Rust lunch, cold; a rehearsal offered → T-18, `mvp/PRACTICE-RUN.md` |
| T-7 | 1b | AC-52's 200-concurrent target — label as hypothesis? | **Answered** 2026-08-12 — **proposal withdrawn**, AC-52 stands at 200. Client testimony: bigger venues being pursued; Jun 26 Summer Social hit 200, sold out. |
| T-8 | 1b | Calendar fork — delegate Aug 20, slip to Oct, or build ahead of evidence | **Answered** 2026-08-13 — **October, client runs it herself.** Delegation declined: *"it's kind of my project and i don't want it taken from me."* |
| T-9 | 1b | Did the Aug 12 segment run without the client? (q3 rollback) | **Answered** 2026-08-13 — **nobody ran it.** q3 rolled back, bank restored to 8. |
| T-10 | 4 | Handoff to `tone-prototype` now vs. hold for October evidence | **Answered** 2026-08-13 — **hand off now.** |
| T-11 | 2·0 | Exploration directions — which core-assumption forks get built, plus review-surface scope and the Discord seam mock | **Answered** 2026-08-14 — **all three directions**; review surface **in scope as one take**, not fanned out. |
| T-12 | 2·1 | Drive the five takes and pick a direction to converge on | **Answered** 2026-08-14 — **Direction C.** *"C is it."* |
| T-16 | 2·2 | **AC-80 — lights up or lights down** | **Answered** 2026-08-14 — **lights up**, endorsed by Cole, author of the brand's Liberty Ferris imagery. No amendment to *no dark mode in v1*. |
| T-17 | 2·2 | Syntax colour in the reading phases — diverges from the design system's `SourceCode` | **Answered** 2026-09-03 — **keep it.** *"keep the colour, love it."* → A-7, AC-99. The divergence from `SourceCode` is the client's to carry to Cole; it does not block. |
| T-13 | 2·1 | Private pull-your-own hint vs. **AC-48** (one hint, everyone, at once) | **Answered** 2026-08-14 — **AC-48 rewritten**, hints are private. Took **AC-45**'s published-hint control with it. |
| T-14 | 2·1 | Does Direction A stay in the fan-out now that public commitment is the named problem? | **Answered** 2026-08-14 — **stays, as the honest loser.** Costs nothing to keep; becomes October's built comparison rather than a hypothetical. |
| T-15 | 2·1c | Who steps the trace — host-driven only (`PROJECTOR_SPEC` §4.2, resolved) vs. each participant at their own pace (`PHILOSOPHY` §9, minted a day ago) | **Answered** 2026-08-14 — **both.** Wall host-stepped, phones step freely. |
| T-18 | 2·3 | **A rehearsal with two regulars** — when, where, and spend `q3`? Offered at Rust lunch, reported 2026-09-02. | **Open** — sheet written: `mvp/PRACTICE-RUN.md`. Client's call on date and venue; venue strongly preferred. **Meetup is October, rehearsal before it.** Testimony captured → A-6. |
| T-19 | 2·3 | `mvp/README.md` run-of-show — sweep for AC-98 and A-5, or leave the MVP frozen? | **Answered by default 2026-09-03 — swept**, on the standing recommendation. Six places, and the beat heading in the prototypes' own reveal script. |
| T-20 | 2·3 | **The closing drive** — love C or list what annoys; AC-93 felt; colour or not | **Answered 2026-09-03 — *"yes i love it."*** Guide at `sequence/T-20-drive-guide.md`, every step done. Sixteen items applied (below). AC-93 *felt* and answered; T-17 answered; AC-28 rewritten. **Stage 2 complete.** |

## Client interview record (T-2)

- **Origin:** *"attendees had gotten all answers over and over, we stopped doing
  dtolnay's quiz at all."* Problem observed, not predicted. Segment abandoned.
- **History:** previously ran a quiz from an existing public Rust bank.
- **Purpose:** all four selected — spark technical discussion, energize the room,
  teach specific concepts, tie into tonight's talk.
- **Timing:** meetups Aug 12 (tomorrow) and Aug 20; September off for RustConf;
  *"you tell me, idc if it has to be october."* → Recommendation put to client at
  T-4: **October** for the built product, with an optional **Aug 20** manual dry
  run to generate real user evidence for the prototype stage.

## Stats

| Phase | Agents | Touchpoints | Wall-clock |
|---|---|---|---|
| 0 | 0 | 2 answered | ~25 min |
| 1 | 5 | — | ~10 min (parallel) |
| 2 | 0 | 1 answered | ~20 min |
| 3 | 0 | — | ~35 min |
| MVP | 0 | — | ~40 min |
| 1b (T-6) | 0 | 1 open (T-7) | ~20 min |
| 2·3 lunch (T-18) | 0 | 2 open (T-18, T-19) | ~30 min |
| 2·4 round 4 (T-20 drive) | 0 | — | ~45 min |
| 2·4 closing the drive (T-19, AC-28, `DESIGN.md` draft) | 0 | 1 answered (T-20: *love it*) | ~40 min |

## Phase 3 output

- `PHILOSOPHY.md` — the one thing (*the quiz is a pretext; the argument is the
  product*), 8 principles, taste. Referenced by `CLAUDE.md`.
- `sequence/USER_STORIES.md` — **18 stories, AC-1 … AC-92, 13 marked `felt`** after Phase 4b (17 / AC-1…AC-88 / 12 at Phase 4; 13 / AC-1…AC-70 / 10 as first minted).
- `CLAUDE.md` — references every root artifact and records that `PRD.md` is a
  prior artifact superseded in part by the Phase 2 synthesis.

## MVP shipped alongside (client call, T-2 timing)

`mvp/2026-08-12/pop-quiz-2026-08-12.html` — self-contained projector deck for
the Aug 12 meetup. 8 questions, all machine-verified: pinned rustc 1.96.1,
5 byte-identical native runs each, Miri clean under strict provenance with
output matching native, one confirmed E0502 rejection. No answer written by
hand. Satisfies AC-6…AC-11, AC-13, AC-23, AC-24, AC-40 today.

Two bugs found and fixed by audit during the build, both recorded because they
are exactly the failure mode `PHILOSOPHY.md` §2 exists to prevent:
1. Answer positions shipped a visible B,D,B,D pattern (badly seeded LCG on
   near-identical question ids). Fixed with a hashed, balanced shuffle.
2. Each question shipped 4 options instead of 5 — "does not compile" replaced
   an authored distractor instead of being appended. Fixed, plus three build
   assertions.

## Phase 4 — Stories review (T-5)

Self-review found ten findings; the client resolved all of them. Applied:

| Finding | Resolution |
|---|---|
| **Explanations were never verified** — the machine established the answer, but the LLM-written prose a host reads aloud sat unchecked under a verification badge | New **Story A6** (AC-71…AC-74) with a **blocking** organizer sign-off, a mechanical check that quoted output matches recorded output, and a receipt that states it covers the answer and not the prose |
| **AC-44 was unfalsifiable** (*"the reveal makes people talk to each other"*) | Rewritten to the client's formulation: a **beginner-Rust attendee can explain the solution to someone else** — observable by asking one, and it tests question and explanation together |
| **AC-69's 15-minute grace window** was shorter than real Discord outages (20 min – 3 h) | Rewritten: **authorize at room creation, not continuously**; a room runs to completion once opened, bounded by a 4-hour max lifetime. Deletes the degraded partial-service state entirely. Discord kept as the auth source per client call. |
| No story for the bank running dry | New **Story A7** (AC-75…AC-77) — a visible reserve, early warning, and the ability to run a meetup with zero generation that day |
| Zero accessibility criteria | New **Story B9** (AC-82…AC-86) — keyboard, live regions, AA contrast, 44px targets, reduced motion. Restores what the prototype already did. |
| Room display smuggled in as one `felt` on a phone story | New **Story B8** (AC-78…AC-81). AC-80 notes the brand's *no dark mode in v1* is what gets amended if a dim room wins. |
| Asserted figures and unbounded claims | AC-8 and AC-21 labelled hypotheses; AC-26's tested tells named and enumerated; AC-87 scopes determinism to the tested target triple; AC-88 restores difficulty calibration |

Bookkeeping corrected: run-state had claimed 14 `felt` criteria; the true count
at minting was 10. Two audits came back clean — AC IDs are continuous with no
gaps or duplicates, and the option-length tell measured 2/8 against a 1.6/8
chance baseline (clean, but by luck — nothing checked it, which is why AC-26 now
names it).

## Phase 4b — Segment shape (D-7), 2026-08-12

Client call: **one question, 3–5 minutes, at the very end of the meetup.**

Applying it surfaced a live defect. `answer_slots()` balanced correct-answer
positions *within a deck*; asked for one question it returned slot 0
unconditionally, so **every meetup would have shipped its answer at position A,
forever.** Third answer-position bug in this codebase, and the quietest — the
format change broke a mechanism nobody touched.

**The first fix was worse than the bug — recorded because it is the fix anyone
would write.** Balancing was moved to the meetup series: least-used letter next,
never repeat last month's. Enforced balance means that once four letters in a
cycle are spent the fifth is arithmetic, so an attendee with a memory got a
**certain** answer every fifth meetup and **45.7%** across a cycle against a 20%
baseline. Client caught it from the sentence describing it — *"idk man like
that's gonna be obvious that it's never last time's answer"* — before any test
did.

| | |
|---|---|
| **Fix** | Slot is drawn **uniformly from the date and nothing else**. `slot_for_day()` is pure and takes no history; `mvp/answer-history.json` demoted from input to write-only record. Perfect-memory attendee measured at **17.5%** across three strategies (chance = 20%), eliminating nothing. Counts are deliberately uneven and letters repeat. |
| **Audit** | Now tests the **generator**, not the sequence: 20,000 synthetic draws, chi-square df=4, **both tails** — upper catches a skewed generator, lower catches someone reintroducing balancing. Structurally cannot alter a real night's output. Both failure modes verified to fail the build. |
| **Principle** | *A fairness mechanism that shapes output is an oracle.* Added to `PHILOSOPHY.md` §2. |
| **AC-23** | Rewritten twice the same day; now split **AC-23 / 23a / 23b** and stated in terms of what an attendee can infer. Both rewrites recorded in `USER_STORIES.md` rather than replaced. |
| **Minted** | Story B10 / **AC-89…AC-92** — one question, ≤5 min end to end, scheduled last, 30-second time-to-opinion (`felt`), one question consumed and recorded per meetup. |
| **Closed** | Open question *"right number of questions per night"* — answered: one. |
| **Build** | `build_deck.py --only <qid>` writes `pop-quiz-<day>-<qid>.html`, so a single-question build never clobbers a batch build. Batch build retained for review. |
| **Aug 12 deck** | `q3` (Collections — `vec.dedup()` on non-adjacent duplicates), answer at **E**. Four lines, difficulty 2, and the `[1, 2, 3]` distractor catches everyone who reads `dedup` as `unique`. |

## Phase 1b — T-6 attendee evidence, 2026-08-12

T-6 was the last open item in Stage 1. It splits in two: what the group's own
records can answer without a room, and what only a room can answer.

**Records half — done.** `sequence/research/06-attendees.md`, a sixth dimension.
Four CSV exports aggregated locally; **no PII copied into the repo** (identities
hashed before cross-file matching, free text quoted unattributed).

| Finding | Detail |
|---|---|
| **Room size** | Regular meetups cap at **110 RSVPs** (three events, exactly 110 each, 1–31 waitlisted). Unconf 2025 approved 200 against 294 registrations. |
| **Show rate** | Median **88%**, aggregate **86%** (1,160 attendances / 1,347 yes-RSVPs, n=131 with ≥3 RSVPs). Far above the 50–60% free-meetup norm. Soft: 32 of 131 show attendances > yes-RSVPs, so the fields don't cleanly nest. |
| **Turnover** | **25–27% of each room has never attended a Rust NYC event.** Core (10+ events) is 15%. 226 unique members across the exports. |
| **Voice** | 41 unique free-text responses — the only attendee testimony in the corpus. **Zero mention a quiz, game, or interactive segment.** Five ask unprompted for beginner-friendly content, one for *"pairs up new people with more experienced engineers."* |
| **Experience mix** | Not establishable. Titles measure career seniority, not Rust fluency; 41% are unmarked "engineer". |

**Two corrections to claims made earlier this stage**, recorded because both were
stated before the data was read:

1. The Luma `survey_response_rating` / `survey_response_feedback` columns were
   described as pre-existing testimony. They are **entirely empty** — 0 of 294.
   No survey was ever collected.
2. "Four attendee exports" is **three events, four files** — the two Gametank
   files are the same event exported days apart. Also, `The_perks_and_pitfalls…`
   is a *post-event* export, so its 0% newcomer rate is an artifact of export
   timing, not a fact about the room.

**Room half — instrument built, capture SLIPPED to October.**
`mvp/FIELD-NOTES-TEMPLATE.md` is instrument and capture form in one, targeting
AC-44, AC-89, AC-91, AC-88, AC-38/78/80, AC-51, the participation ratio, and the
three open questions. Written so a **co-organizer can run it cold** — it does not
require the client. A hand-run projector deck with no phones **is** the
hands-only condition Story B5's fate depends on, so the natural experiment costs
nothing to run whenever a meetup happens.

*(Originally `mvp/2026-08-12/field-notes.md`, dated to a meetup that did not
happen for the client. Generalized and undated 2026-08-13.)*

## T-7 — AC-52 amendment proposed and WITHDRAWN, same day

**`AC-52` stands at 200 as written. No amendment.** Raised and retracted
2026-08-12 on client testimony; recorded rather than deleted because the error
is instructive.

**The proposal** was to label `AC-52`'s 200-concurrent target a hypothesis at
"2× the observed ceiling," reasoning from the 110-RSVP cap to a ~90–100-person
room.

**The error** was reading a **venue supply cap as a demand ceiling** — and the
disproof was already inside the same dossier: every event sold out with a
waitlist, 76 turned away at the Unconf alone. Client testimony supplied two
facts no export held: the group is **actively pursuing bigger venues**, and the
**Jun 26 Summer Social drew 200, sold out**. Second time Rust NYC has filled a
200-person room. Sizing the system to a constraint the organizers are actively
removing is backwards.

**Also retracted:** the claim that a ~100-person ceiling weakens the case for the
heavier realtime substrate. Dimension 5 stands unchanged — Durable Objects
remains the fit, and Turso's single-writer behaviour against a 200-write burst
remains a live concern.

**What survives, and it is sharper.** The client's own caveat — *"that was a
party not a meetup"* — is adopted: a sold-out social establishes **draw**, not
participation in a technical segment. `AC-52` counts **concurrent
participants**, and the number that sizes the deadline write burst is
*attendance × participation rate*. **Only the first factor has ever been
measured.** 200 present at 40% participation is 80 concurrent; at 90% it is 180
— a >2× spread in precisely the quantity AC-54 calls the highest-risk moment in
the system.

Field-notes Q2 (*do phones help or hurt*) is now the **only** live line of
evidence bearing on whether Stories B1/B5 shrink; the room-size line is
retracted. The field notes were amended to capture a participation count, making
tonight the first measurement of that ratio.

## Calendar reality, 2026-08-13 — attendee evidence slips to October

Client: **Aug 12 skipped** (did not attend); **Aug 20 unavailable** (Alloy
company boat party); **September is RustConf**, no meetup. **Next meetup is
October.**

**The consequence is a collision, and it is the reason this is recorded rather
than noted.** The T-2/T-4 plan was *October for the built product, with an
optional Aug 20 manual dry run to generate real user evidence for the prototype
stage*. The dry run is gone. **Evidence and ship date now land in the same
month** — the meetup that was supposed to de-risk the build is the meetup the
build was aiming at.

| # | Option | Cost |
|---|---|---|
| 1 | **Delegate Aug 20 to a co-organizer.** The exports show 3–4 organizer-role accounts besides the client, and `FIELD-NOTES-TEMPLATE.md` is written to be run cold by someone else. | Cheapest by far. Saves two months. Needs one ask. |
| 2 | **October meetup becomes the dry run**; built product targets Nov/Dec. | Honest, but slips the build a full quarter. |
| 3 | **Build ahead of evidence**, ship October. | Fastest. Designs Stories B1/B5 without ever testing whether phones are wanted — the exact risk field-notes Q2 exists to retire. |

**Resolved 2026-08-13 (T-8): option 2 — October, and the client runs it
herself.** Delegation was recommended and **declined for a reason that is
accepted without argument**: *"it's kind of my project and i don't want it taken
from me."* Authorship of the segment is not a cost to be optimised away, and the
run records it as a legitimate client call rather than an efficiency loss.

**The collision largely dissolves anyway**, because T-10 sent the arc forward:
`tone-prototype` validates `felt` criteria with the *client* driving clickable
prototypes, not with attendees; only AC-44, AC-91, the participation ratio and
Q2 need a room. Design discovery occupies the two months that would otherwise
have idled, and the October meetup lands as evidence feeding back into a design
that already exists — rather than as the first and last chance to learn anything.

**Carry phones-vs-hands as a prototype direction.** It is a core-assumption fork,
which is exactly what `tone-prototype` fans out on. Making it one of the
exploration directions turns the unanswered question into design input instead
of a blocker, and means October's answer selects between designs already built
rather than sending anyone back to the drawing board.

## Defect — AC-92 records build time, not run time

`build_deck.py:220-234` (`slot_for_meetup`) writes `answer-history.json` from
`main()` **when a deck is built**. AC-92 requires recording which question each
meetup **used**.

**Live consequence — found and repaired 2026-08-13.** The Aug 12 deck was built
and the meetup was skipped, so the ledger claimed **q3 was consumed when nobody
had seen it**. One of only 8 verified questions was wrongly retired — with one
question per meetup, a month of supply. The already-run check at
`build_deck.py:253-256` would also have fired a **false** *"q3 was already run
on 2026-08-12"* warning at the next build, pressuring whoever built it to burn a
second question to avoid a repeat that never happened.

Client confirmed at T-9 that the segment did not run. **The 2026-08-12 entry was
removed by hand and `meetups` is now empty; the bank is back to 8 and q3 is
fresh.** The rollback and its reason are recorded inside
`mvp/answer-history.json` itself. Note that rebuilding a deck for 2026-08-12
would recreate the entry — the defect below is unfixed.

**Not an oracle bug** — the ledger is correctly write-only and does not feed slot
selection; that part works as designed. This is a **provenance** bug: build time
masquerading as run time.

**Mechanism not fixed — carried to `tone-architect`.** Whether consumption is
confirmed at build, at run, or by an explicit "this ran" step is a design
decision, not a patch. It is a live candidate for a new acceptance criterion
under Story B10, since `AC-92` as written is not satisfied by any code that
exists. Data rolled back; mechanism open.

## Stage 1 — COMPLETE 2026-08-13. Handed off to `tone-prototype`.

All touchpoints T-1…T-10 answered. Carried forward, none of them blocking:

- **T-6 room half — October.** The client runs `mvp/FIELD-NOTES-TEMPLATE.md` at
  the October meetup and folds the answers back. `tone-prototype` is licensed to
  reopen `USER_STORIES.md` when they land, which is the expected path for AC-44,
  AC-91, the participation ratio and Q2.
- **Phones-vs-hands → a prototype direction**, not a blocker. See T-8 above.
- **AC-92 mechanism defect** → `tone-architect`. Likely a new criterion under
  Story B10; `AC-92` as written is satisfied by no code that exists.
- **Participation rate** remains the unmeasured variable behind AC-52/AC-54.

*Stage 1 opened 2026-08-11, closed 2026-08-13. Superseded along the way: an
earlier "Remaining" section listed T-5 as open and scoped it to AC-1…AC-70; T-5
was answered 2026-08-11 and Phase 4b took the range to AC-92.*

*(Superseded: an earlier version of this section listed T-5 as remaining and
scoped it to AC-1…AC-70. T-5 was answered 2026-08-11, and Phase 4b took the
range to AC-92.)*

---

# Stage 2 — `tone-prototype`

## Phase 0 — Intake, 2026-08-14

Corpus read cold: `run-state.md`, `PHILOSOPHY.md`, `sequence/USER_STORIES.md`
(18 stories, AC-1…AC-92, 13 `felt`), `CLAUDE.md`, `ECONOMICS.md`, the Phase-2
synthesis, and `PRD.md` §Brand And Accessibility. Inputs are complete; nothing
upstream is missing.

**Prior design asset re-assessed.** `~/Downloads/Pop Quiz - Rust NYC
(offline).html` (869 KB) carries two lanes — Organizer and Participant — across
~20 named states, including the ones nobody remembers to design: *Discord is
unreachable*, *Reconnecting*, *Room closed*, *Hint published to everyone*, *How
the room voted*, plus a reduced-motion toggle and screen-reader announcements.
Its `RustNYCDesignSystem` tokens match the `PRD.md` brand contract exactly
(`--radius:4px`, `--accent-primary:#d69e2e`, 44px targets, 480px container).

Two things it is **not**, both load-bearing for this stage:

1. **It is pre-D-7.** Its vocabulary is a multi-question round — *Next question*,
   *End quiz*, *Quiz complete*, a question count, and *Generate quiz* as a live
   action. The one-question-in-five-minutes segment invalidates that spine.
2. **It has no room-display lane.** Story B8 (AC-78…AC-81) — the surface
   `USER_STORIES.md` calls *arguably the primary one* — was never designed.
   Under D-7 the projector carries the whole segment.

So it is a strong **component and edge-state library**, not a converged take.

**Prototype scope note.** Story A4/A6/A7 (the organizer review surface, AC-20 and
AC-21 both `felt`) is a real design surface with no fan-out axis — it is a tool
for one person, and that person is the client. Carried to T-11 as a scope
question, not a direction.

**Seam mock owed.** Discord is the external system the user also lives in
(Story C1, AC-64…AC-70). Per the stage contract this needs a stand-in of
Discord's own surface plus a one-page seam diagram — what maps to what, what
crosses, what each side sees — not a black box.

## T-11 — exploration directions (open)

Put to the client 2026-08-14. Three candidate directions, each resting on a
different core assumption about **where the segment actually happens**:

| | Direction | Core assumption | What it costs if wrong |
|---|---|---|---|
| **A** | **Hands** — no participant app at all. Projector + a host control. | The room's voice beats its phones; the infrastructure was never the point. | Stories B1/B5/B6 lose most of their weight; the build shrinks to a fraction. |
| **B** | **Phone-first** — the phone carries code, answer, and reveal. | Commitment is private, and anonymity is what makes people willing to be wrong out loud. | Closest to the PRD and the prior prototype — but that prototype is pre-D-7. |
| **C** | **Projector-first, phone as buzzer** — code lives on the big screen, the phone is five letters. | Fifty people reading the *same* nine lines is the event; the phone is an input device, not a reading surface. | Makes AC-91's 30-second budget a projector-typography problem, not a phone-layout one. |

The phones-vs-hands fork was carried here deliberately from T-8: it is a
core-assumption fork, which is what this stage fans out on, and building it as a
direction means October's field notes **select between designs that already
exist** rather than sending the design back to the start.

**Answered 2026-08-14: all three, and the review surface as one take.**

## Phase 1 — the takes, 2026-08-14

`prototypes/`, five clickable HTML takes plus an index. Every one runs on the
**real verified batch** from `mvp/2026-08-12/` — real sources, real distractors,
the real long explanations, real receipt facts, and `q3`'s answer at **E**
exactly as `slot_for_day()` drew it. Room figures come from
`research/06-attendees.md` (110-RSVP cap, 88% median show → 95 present). Nothing
is shortened to flatter a layout; that is the point of prototyping on real data.

| File | Take | Frames |
|---|---|---|
| `A-hands.html` | Direction A — Hands | projector + host phone |
| `B-phone-first.html` | Direction B — Phone-first | participant phone + host phone + projector mirror |
| `C-projector-first.html` | Direction C — Projector-first, phone as buzzer | projector + buzzer phone + host phone |
| `review-surface.html` | Review surface (one take) | desktop browser |
| `seam-discord.html` | The Discord seam | Discord stand-in + our side + SVG seam diagram |
| `take-it-home.html` *(added 2026-08-14)* | Take it home — the post-meetup page | desktop browser |
| `index.html` | The takes, side by side | — |

Shared: `_shared/tokens.css` (the `PRD.md` brand contract, inherited not
reinvented), `_shared/data.js` (the real batch), `_shared/proto.js` (Rust
highlighting, the persistent `PROTOTYPE` badge, the AC-83 live region).

**Instruments built into the takes**, so `felt` criteria can be judged rather
than asserted:

- **C carries a back-of-the-room test** — front row / middle / six metres back,
  scaling the whole frame. AC-78 is the criterion this direction lives or dies
  on, and a mock only ever seen at full size passes a test the room never runs.
- **A and C carry a lights-down toggle.** AC-80 says the brand's *no dark mode
  in v1* is what gets amended if a dim room wins, so the amendment is built and
  offered for judgement instead of argued about.
- **The review surface times itself.** AC-21 targets 15 minutes for a batch and
  nobody has ever measured it; the clock in the corner is the measurement.
- **Every host surface shows the participation ratio** — the one variable behind
  AC-52/AC-54 that has never been measured.

**Findings already surfaced by building, ahead of the client's drive-through:**

1. **The prior state-explorer is not a Direction-B head start.** Its spine is a
   multi-question round; D-7 deletes that vocabulary. Its edge-state catalogue
   survives and is reused.
2. **AC-39 does not survive Direction A as written.** *Anonymous per-option
   totals* exist there only because the host types them, and a raised hand was
   never anonymous to the room. If A wins, AC-39 needs amending — recorded in
   the take itself rather than hidden.
3. **AC-32 has no surface in Direction C.** *Legible without pinch-zoom on a
   375px phone* applies to nothing when the phone carries no code. If C wins,
   AC-32 is withdrawn or rewritten against the projector.
4. **The explanation's delivery is a design fork, not a detail.** B puts the
   prose on every phone; C puts it only in the host's hand. Under C, AC-42
   (*reads well aloud*) stops being aspirational — speech is the only delivery.
5. **`q8`'s answer *is* "does not compile",** so appending AC-24's universal
   option would have shipped a five-option question with four real choices.
   Caught in the review surface and fixed there.

## T-12 ANSWERED — Direction C, 2026-08-14

*"C is it."* **Projector-first, phone as buzzer.** The code lives on the big
screen and nowhere else; the phone is five letters.

**It was not picked off a menu.** The client had already stated C's core
assumption in her own words at **A-4** — *"should not put it on the phone, only
options to vote on, code and tracing need to be on screen"* — which eliminated
**B** (its assumption is that the phone carries the code) and **A** (*"only
options to vote on"* presumes a device that A does not have). A also carried two
recorded strikes: public commitment against §9, and it cannot satisfy AC-48 as
rewritten at T-13. The drive-through then confirmed C directly rather than by
elimination.

Corroborated independently: `PROJECTOR_SPEC.md` §0 called the reveal+trace state
*"where the night lives"* and §9's build order put the wall first as *"the new
surface and the star"* — three weeks before these prototypes existed.

**What this closes.** The fan-out. `A-hands.html` and `B-phone-first.html` are
now historical takes, not candidates; both stay in the repo because October's
field notes are supposed to select against *built* comparisons rather than
hypotheses (T-8), and B's projector bug was fixed for exactly that reason.

**What it does NOT close, and the stage rule is explicit about it.** *The pick is
the start of the design, not the end.* Iteration ends when the client **loves**
the take, not when she picks it — the working test is *would she keep this if a
better option appeared tomorrow?* **No `DESIGN.md`, no `tone-architect`, no
build** until the iteration rounds on C are done.

### Round 1 — full-width layout: built, and it settles the question length

Client: *"i dont know how big but pretty big, maybe 15ft, i have to measure or
ask."* Taken as the working figure and **labelled a hypothesis**, with the
measurement routed to `mvp/FIELD-NOTES-TEMPLATE.md` so October produces it. Both
wall layouts are now built and switchable, because neither obviously dominates:
**full-width buys line width and spends line count.**

Measured at **15ft screen, back row 20ft → 14.2px smallest comfortable type**:

| Source | Code beside options | **Code full-width** |
|---|---|---|
| `q3` — 5 lines | fits | fits |
| `q1` — 16 lines | fits | fits |
| `q7` — 69ch wide | **56px off the edge** | **fits** |
| `LAYOUT-35` | 337px across, 294px down | across **fixed**, 294px down |

**Full-width fixes every real question in the bank**, including `q7`, which was
the widest thing the fan-out had. Adopt it.

**And it produces the number the project has never had: the wall holds 16
lines.** `q1`, the longest question in the bank, is **exactly 16**. There is no
headroom at all, and the 16 rests on a guessed screen width — which is precisely
why the measurement is now in the field notes.

**35 lines is not reachable here, and that is fine.** **D-6's *"no tool can
display a 35-line Rust program"* was always a statement about what off-the-shelf
tools cannot do, not a requirement this segment has** — under D-7 nobody reads 35
lines standing up in five minutes. Build-vs-borrow is unaffected.

### Correction — question length is not bounded, and 27px was never the model

**Client:** *"no question length is not bounded but it has limits, 27px is not
working, needs to shrink to smallest legible."* Both halves land, and the first
is a straight correction to what this file said an hour earlier.

**The wrong framing was mine:** *cap source length at whatever October
measures.* That is a rule in the generator that pre-rejects a perfectly good
question for being one line too long. **The right model is that the wall sizes
itself to the question and legibility is the floor** — the type grows to fill the
wall when the program is short and shrinks toward the smallest legible size when
it is long. A question is too long **only when it will not fit even at minimum
legible size**, which makes the limit *emergent and the room's*, not a constant
imposed on the pipeline. Nothing is rejected up front; a venue with a bigger
screen or a shallower room simply holds more.

**And 27px was wrong in both directions**, which the fixed size had hidden: too
small for a short program (`q3` now renders at **35.8px** and fills the wall) and
too large for a long one. Auto-fit is now the default behaviour rather than a
button.

Measured, full-width, at the 15ft/20ft assumption — every real question in the
bank fits, each at the largest size it can:

| Source | Auto type | Verdict |
|---|---|---|
| `q3` — 5 lines | **35.8px** | fits |
| `q5` — 9 lines, 50ch | 28.4px | fits |
| `q7` — 69ch wide | 22.5px | fits |
| `q1` — 16 lines | 16px | fits |
| `LAYOUT-35` | clamped to the 14.2px floor | **too long for this room** |

The same run in the split layout flags **`q7` as too long** — a real question the
old fixed-27px wall was silently truncating. Second reason to adopt full-width.

**Sizing note for `tone-architect`:** the fit is computed, not measured —
`lines × 1.5` for height, `(widest + 3.5 gutter) × 0.6em` for width, against the
usable code area. The first cut omitted the line-number gutter and the well's
padding and overflowed by 40–80px, which is worth knowing before anyone
reimplements it.

### Round 2 — the beat the one thing never had

**Found by the client asking one word: *"arguing?"*** The design had no place where
anyone worked anything out. The split was a *screen*, not a beat: the wall showed
the camps, said *"no answer yet"*, and the host clicked straight to the reveal.
The end card said **"Keep arguing."** — the product asking for the thing it never
made room for. The stage had spent its attention on whether the room could *read*
the wall and never asked what happens after they have read it.

**The first proposal was wrong and is recorded because it is the obvious one.**
*"A says `[1, 2, 3]`. E says `[1, 2, 3, 2, 1]`. Someone from each — why?"* That
asks two people to defend a position in front of the room, which is exactly what
§9 was minted to prevent. It was proposed one turn after §9 was cited as the
point. The client rejected it and replaced the frame — see **A-5**.

**Built: phase `5 · work through it`, between the split and the reveal.**

| | |
|---|---|
| **Knowledge sharing — unstaged, and that is the point** | **Second correction, same day.** The first build of this beat told the room *"find someone who picked something else and ask them what they saw."* The client cut it: *"none of this... no one needs to volunteer though we'll take it from one volunteer."* **An instruction is still an obligation**, and §9's whole subject is the person who does not want to be put on the spot. Turning to a stranger on cue is a smaller ask than defending an answer, but it is the same kind of ask. The wall now says only **"Let's walk it. Still no answer. Nobody has to say anything."** If one voice offers a reading the host takes it; if none does, nothing is missing. The sharing happens **at the bar**, unforced, which is where it was always going to happen. |
| **Repetition** | The room then walks the trace **together, before the answer**. The trace already existed but sat *after* the reveal, where it justified a settled result. Moved in front of it, the second pass through the program is what **produces** the understanding rather than what explains it. |
| **Still no answer** | The correct option is not marked and the receipt is not shown. Caught during the build: the first cut fell through to the reveal branch and put **E and the verification receipt on the wall during the beat**, which would have made the whole phase pointless. |

Order is **repeat → resolve → release**, and `take-it-home` is the second
repetition, alone, later. **The end card is the client's own line: "Let's go to
the bar."** It is better than anything written for it here — the understanding
carries on socially and unforced, which is the one thing's *"it walks out with
them"* stated as a thing a person would actually say.

**Cost, and it is real:** AC-89 gives the segment 3–5 minutes and this beat wants
60–90 seconds of it. That is the first thing in this stage that *spends* the
budget rather than saving it, and under A-5 it is the part that should be
protected when something has to give.

**Minted: AC-97 and AC-98** under Story B11, on the client's instruction the same
day. **AC-97** requires the walk-through phase to sit between the split and the
reveal with the answer unmarked and the receipt hidden — a build that runs the
trace only *after* the answer explains a settled result instead of producing one,
and does not satisfy it. **AC-98** is written as a prohibition because it is the
property two successive designs failed: nothing obliges a participant to speak or
to interact with anyone, a host may take a contribution offered unprompted, and
the product neither asks for one nor notices when none comes.

### Round 3 — the client's drive-through answers, 2026-08-14

**1. The last screen is the handover.** *"Last screen should just have link to
take it home."* `7 · released` now carries **"Let's go to the bar."**, the link at
40px, one line of what is behind it, and a QR — the link is the hero and the QR
is there because a link on a wall is useless without a way into a phone. Nothing
else competes with it. The segment can now be judged end to end.

**2. Room measurements — the client will run the field notes.** The 15ft / 20ft
figures stay labelled a hypothesis until she does.

**3. AC-80 ANSWERED — lights up. No amendment to the brand.** AC-80 said the
brand's *no dark mode in v1* is what gets amended **if a dim room wins**. It did
not. The take keeps the toggle as an instrument, and the default was already
correct.

> Evidence worth recording rather than paraphrasing: *"lights up — Cole likes it
> and he's the best designer I know, made Liberty Ferris."* **Cole is the author
> of the house brand's own imagery**, so this is not a preference collected from
> a bystander — it is the designer of record endorsing the light treatment on his
> own system. That is the strongest evidence any `felt` criterion in this arc has
> been closed with.

**4. *"The code without syntax highlighting looks bleh."*** — and the objection is
correct on the phase it was made about.

Phase 1c removed colour because `PROJECTOR_SPEC` §4.1's trace signals with
highlight-and-dim and a second colour channel underneath competes with it. **That
reason is airtight while the trace is running and vacuous when it is not.**
During `2 · question live` there is no trace at all — the room is reading nine
lines for thirty seconds — so nothing is competing with anything, and the well is
just grey.

**Built as a proposal:** colour is scoped to the phases with no trace, and
suppressed the instant the walk-through starts. Verified — the reading phases
carry keyword, macro, type, string, number and comment colour; `work` and
`reveal` render zero coloured spans. Muted values, not an editor theme: the taste
section asks for a well-made terminal.

**Deliberately NOT yet propagated to the criteria.** `PHILOSOPHY.md` §5 and
**AC-32** were amended *this morning* to say the source renders without
highlighting. If this survives, **AC-32 needs its third revision in one day** and
§5 a second — so it waits until the client has looked at it, and until **Cole has
been asked**, because this diverges from the design system at the component level
and he owns that component. A house brand is *"not up for reinvention"*; going
around its author quietly would be exactly that.

### The iteration queue on C, in priority order

| # | Round | Why it is first |
|---|---|---|
| 1 | **Source full-width, options beneath** | The options column costs ~43% of the wall. This one layout change absorbs the horizontal overflow on every source in the bank and decides whether 35 lines is reachable at all. Everything about question length depends on it. |
| 2 | **Type size defaulted to the real venue** | The derived model currently assumes a 20ft screen with the back row at 20ft. Nobody has supplied Rust NYC's actual screen size or room depth, and those two numbers move the answer by 2×. **Client input needed.** |
| 3 | **The handover into `take-it-home`** | C's `6 · released` still ends the segment with nothing. The QR and link exist only inside the take-it-home page, so the segment cannot currently be judged end to end. |
| 4 | **The `felt` judgments still unmade** | AC-80 (lights up or down), AC-93 (does split-before-answer land), AC-21 (the review-surface clock, never once measured). These are part of loving the take, not separate from it. |

### Criteria consequences now binding, none yet minted

- **AC-97 / AC-98 minted** — the walk-through phase and its no-obligation rule.
- **A sibling to AC-33: the source must FIT the wall.** Container-scroll is a
  phone escape hatch and A-4 took the phone away. This is a bound on question
  dimensions, therefore a constraint on the generation pipeline. Kin to AC-88.
- **AC-95 scoping** for the take-it-home page — it requires a count the page
  cannot carry without room state.
- **AC-32** is already rewritten (A-4) and is satisfied by C by construction.
- **AC-39** — the objection was specific to Direction A and dies with it.

## The source under test — and the wall does not fit the bank

**Client, driving C, 2026-08-14:** *"example on screen is not really complex
enough."* Correct, and the measurement is worse than the impression.

**Every take was built and judged against `q3` — the joint-shortest question in
the bank**, 5 lines and 42 characters at its widest. Real and verified, but the
prettiest member of the set, and AC-78 / AC-38 / AC-33 are precisely the criteria
a short source flatters. The stage contract names this failure directly: *a
prototype validated against pretty data validates nothing.* It was avoided for
the room numbers and the explanations and then walked into on the source itself.

| Source | Lines | Widest | Runs off the wall at 27px |
|---|---|---|---|
| `q3` *(what every take used)* | 5 | 42ch | **168px horizontally** |
| `q1` | 16 | 36ch | 87px across, **212px vertically** |
| `q5` | 9 | 50ch | **298px horizontally** |
| `q7` | 5 | 69ch | **605px horizontally** |
| `LAYOUT-35` *(fixture, not a question)* | 35 | 101ch | **1140px across, 1029px down** |

**Instrument built:** a *source under test* switcher on Direction C, worst case
first, defaulting to **`q1` rather than `q3`**. It swaps the source only, and
only before reveal — the vote, the split and the reveal stay `q3`'s, because the
room data is `q3`'s and a split shown against another question's options would be
a lie. The take now also states the overflow on screen rather than letting a
scrollbar absorb it.

### Three findings, in rising order of consequence

1. **Not one question in the bank fits the wall at 27px.** Even the 5-line `q3`
   loses 168px off the right edge. This was invisible because the source well
   scrolls horizontally — which looks like AC-33 being satisfied.
2. **AC-33 does not transfer to the wall, and A-4 is why.** *"Code that exceeds
   the viewport scrolls inside its own container"* was written for a phone, where
   the reader's thumb is the escape hatch. **AC-79** says the wall takes no
   interaction beyond host controls, and **A-4** put the source on the wall and
   nowhere else. So on the wall, overflow is not scrollable — it is **invisible**,
   and the scrollbar is the prototype quietly lying. Direction C needs a sibling
   criterion that the source **fits**, which is a bound on question dimensions,
   not a CSS fix. Candidate kin to **AC-88** (difficulty calibration) and a
   constraint on the generation pipeline.
3. **The 35-line claim has never been tested.** Dimension 1 and **D-6** rest on
   *"no mainstream live-quiz tool can display a 35-line Rust program"* — the
   entire build-vs-borrow argument. The verified bank tops out at 16 lines, so no
   artifact in this project had ever displayed one. The fixture does.

### Correction, same day — the 27px was arbitrary and the numbers above assume it

**Client:** *"it doesn't have to be 27px on a projector 20ft screen does it."*
**No, and nothing ever derived it** — it was chosen to look right in a browser
and then every overflow figure above was measured against it. A px in this mock
is not a physical size: the wall is a 1120×630 canvas standing in for the *whole
projected image*, so 27px means **each line is 4.3% of screen height** — roughly
**8.6 inches** on a 20-foot screen, far more type than any room needs.

What governs legibility is cap height against viewing distance. Rule of thumb:
cap height ≥ distance / 150 for comfortable reading of detailed content, / 200
for basic legibility. Code is read carefully, so 150 is the honest default.

| 20ft-wide screen (11.25ft tall) | back row 20ft | back row 40ft |
|---|---|---|
| Smallest comfortable type, in mock px | **≈10.7px** | ≈21px |
| Headroom over the as-built 27px | **2.5×** | 1.3× |

**Re-measured at 10.7px, the 35-line fixture nearly fits** — 118px over
horizontally and 119px over vertically, against **1140px and 1029px at 27px**.
The earlier figures stand only as *"at 27px"* and are not a verdict on the
design.

**What that leaves is a layout finding rather than a legibility one.** The wall
spends ~43% of its width on the options column, so the source gets ~620px of a
1120px wall. Full-width source with the options beneath it would absorb the
horizontal overflow outright; the vertical is tighter — 35 lines at 10.7px needs
~561px against ~426px of code area, and reclaiming the full wall height gets it
to roughly 10.1px, marginally under the comfortable minimum. So **35 lines is at
the edge, not beyond it**, and the two-column layout is what decides it.

**Instrument built:** the type size is now derived from screen width, back-row
distance and the chosen ratio, with *"shrink to the smallest legible"* on the
wall, and the take states the implied cap height in inches. It is labelled a rule
of thumb — **AC-78 is `felt`**, and October's field notes are where a real back
row settles it. But arithmetic beats a squint in a browser, and until now there
was neither.

`LAYOUT-35` carries no options, no answer and no receipt, and nothing anywhere
claims what it prints — `PHILOSOPHY.md` §3 holds. It is a layout fixture and the
take says so on screen.

## Defect found by the T-12 drive-through — the wall did not fit its window

**Found 2026-08-14 by the client, on a screenshot**, driving Direction C in a
620px pane. The projector's two columns had collapsed into each other: options
overlapping the source well, the room code sitting on top of option E, the join
strip across the middle of the code.

**Cause.** `.projector` is `width:100%; aspect-ratio:16/9`, but everything inside
it is sized in **absolute px drawn for a 1120px wall** — 27px source, 24px
options, 62px title. That is correct and deliberate: *a projector does not
reflow*. What was missing is the consequence — below 1120px the box shrank while
its contents did not, so they overflowed the 16:9 frame and collided.
`setDistance()` compounded it by scaling content *inside* an already-overflowing
box.

**Fix.** The wall now lays out once at its design size and is scaled to fit as a
single unit: a `.proj-fit` clip layer, `.proj-wrap` fixed at 1120px, and one
transform carrying **two separate factors** — `fit` (how much room the window
has) times `dist` (the back-row test). Keeping them separate is what makes the
seat buttons honest at any window size: they change how far away the wall is,
never how it is laid out.

**The instrument this bug argued for, and it is the more valuable half.** The
take now says so on screen whenever the wall is below design size — *"This wall
is at 52% of design size… the back-row test still compares honestly, but do not
settle AC-78 here. Widen the window until this line disappears."* AC-78 judged on
a shrunken mock is not judged at all, and the take should say that itself rather
than rely on whoever is driving remembering it. Same reasoning as the back-row
buttons existing at all.

**Direction B had the identical defect** (143px of overflow) and was initially
left alone as disproportionate work on an eliminated direction — **client call:
fix it**, done the same day. Ported at B's own design width (940px, not 1120px),
without the back-row factor, because in that direction the wall is a mirror and
nothing is judged on its legibility.

**Fixing it surfaced a second, independent defect in B that scaling had been
hiding.** Even at design width the reveal phase pushed ~43px out of the frame:
`.joinbox` is `flex: 0 0 auto` and its reveal stack — brand, split bars, and a
**343px block of prose** — could not fit a fixed 16:9 wall. The prose now scrolls
in its own container, which is AC-33's rule applied to the wall. **In this
direction that is the honest shape anyway**: under B the explanation belongs on
the phone and the wall is only mirroring it, so a wall that cannot hold the full
prose is telling the truth about the direction rather than failing at it.

Verified clean afterwards at **every phase of both takes** — B across all seven,
C across all six. **Direction A was unaffected throughout**; its wall is a single
column and survives shrinking.

## Phase 1b — client testimony reopens the philosophy, 2026-08-14

**The input.** Unprompted, on first sight of the takes: *"i want newbies to be
able to be wrong. the feedback we get and it's surfaced in women in rust is:
everything is very intimidating. people aren't allowed to be wrong."*

**Why it is recorded as an amendment and not a preference.** This is primary
evidence about the room, the same class as the T-2 quote that withdrew Dimension
1's verdict. **Nothing in 1,811 lines of dossier mentions intimidation.**
Dimension 6 §4b found the beginner *want* — five unprompted asks for
beginner-friendly content, one for *"pairs up new people with more experienced
engineers"*, against a 25–27% never-attended-before rate — and correctly tied it
to AC-44. It had no account of the cause. This is the cause.

**What it changed in the philosophy.** Principle 6 (*nothing per-person is
recorded*) protects the **record**. It turns out to be necessary and nowhere
near sufficient: a segment can record nothing at all and still be the most
exposing five minutes of someone's evening. New **§9** protects the **moment**.
The one thing is unchanged and is what the new principle serves — an argument is
the product, and an argument needs people willing to be wrong out loud.

**The asset the format has and nothing else does.** *Majority wrongness,
anonymous, in public.* On `q3`, **24 of 58 chose `[1, 2, 3]`** — 41% of the
room, wrong, together. No host saying it is fine to be wrong does what a room
watching 41% of itself be wrong does. The failure mode to avoid is the one that
looks like kindness: softening questions, or reassuring people.

**The prototypes were doing the opposite, and it took building them to see it.**

| Where | What the first cut did | Now |
|---|---|---|
| B and C | A red **✗** on the participant's own choice | No ✗ exists anywhere in the product |
| B | *answers closed* → *reveal*, no split beat | Split phase between them — **AC-93** |
| A, B, C | One intermediate-pitched paragraph | Three beats; the middle one is the popular wrong answer — **AC-95**, **AC-96** |
| B, C | Hint only publishable by the host to the whole room | A quiet pull-your-own hint, **badged as a proposal** (T-13) |

**Direction A takes a direct hit.** *Hands up for A*, one letter at a time, is
public commitment with your face attached. The newcomer who is unsure raises
nothing, visibly, five times. Every available mitigation — eyes closed, heads
down — removes the public commitment, and the public commitment **is** the
direction. Recorded inside the take itself rather than quietly de-emphasised;
whether it stays in the fan-out is **T-14**, the client's call.

**Minted:** Story B11 / **AC-93…AC-96**. `PHILOSOPHY.md` gains §9 and one taste
line (*the person who got it wrong is being talked to, not about*).

**Not minted, deliberately:** the private hint. It contradicts **AC-48** head-on,
and a new criterion that contradicts a standing one is a fork in the truth rather
than an addition to it. **T-13** resolves it first.

**Spec consequence for `tone-architect`.** AC-95 turns `explanation` from one
string into three fields, and the middle one cannot be empty. That is a
content-model change and a new blocking review gate alongside AC-72, not a copy
change.

### T-13 answered — AC-48 rewritten, and it deletes a host control

**The hint is now private.** Every participant can pull it on their own device
for as long as the question is open, and taking it tells nobody — not the host,
not the room, not any total. Was: *publishing a hint shows the same hint to
everyone, at once.*

The old criterion protected the fairness of a competition that does not exist —
no score, no leaderboard, nothing recorded. What it cost was one-sided: a
newcomer who wanted help had to get the host to announce that the room needed
help.

**AC-45 amended as a consequence, not a separate decision.** *An optional
published hint* is gone from the host's controls, because a hint that is always
available has nothing left to publish. **`PHILOSOPHY.md` §8 paying out a third
time** — and the first time it arrived from an accessibility argument rather
than an engineering one.

**It also rules something out that nobody had noticed.** A direction with no
participant device **cannot satisfy AC-48 as amended**: its only possible hint
is announced to the room. That is a second, independent strike against Direction
A, invisible until the hint question was asked — and a small demonstration that
the criteria are doing real work rather than decorating the design.

## Phase 1c — the design corpus, 2026-08-14

Connected to claude.ai/design at the client's instruction. Two projects, both
hers, neither in Stage 1's intake. Read: `PROJECTOR_SPEC.md`, `DESIGN.md`,
`tokens/colors.css`, `tokens/fonts.css`, and both file manifests.

### What the corpus is ahead of us on

- **The trace was already specified**, and its model is better than the one built
  here yesterday. `PROJECTOR_SPEC.md` §4.1: a step is
  `{ lines, focus, note, values }` — highlight execution lines, dim everything
  outside a focus region, anchor a callout to the trap line, and show
  debugger-style value deltas (`v.len(): 2 → 3`). **That generalises to all eight
  questions in the bank.** The tape built here only models a collection walk and
  would not survive `q1` (drop order), `q6` (shadowing) or `q8` (E0502).
- **The projector is already the star.** §0: *"the reveal+trace state is where
  the night lives"*; §9 build order puts the wall first and calls it *"the new
  surface and the star."* That is Direction C's core assumption, arrived at
  independently three weeks earlier — the strongest evidence yet in the fan-out,
  and it did not come from these prototypes.
- **A whole surface the arc has not specced:** the **static fallback** (§6) — the
  same projector views with baked data in one offline file, for when generation,
  auth, or wifi fails at the venue. Related to Story A7 but not the same thing.
- **`SourceCode` renders with no syntax highlighting.** Deliberate, and the
  prototypes here highlight. The design system wins.
- **Cascadia Mono** is the brand mono, bundled as a variable font. `PRD.md` says
  *"a system monospace stack"*; the house brand says otherwise and is newer.
- Components already exist for most of what was hand-rolled: `SourceCode`,
  `ChoiceButton`, `RoomCode`, `Timer`, `Panel`, `Alert`, `Badge`, `Button`.
- Room code is **six characters** (`KMT4XW`), not `RUST-4417`.

### Where the arc is ahead of the corpus

`PROJECTOR_SPEC.md` predates D-7 by three weeks, and Stage 1 supersedes it in
six load-bearing places. None of these are the corpus being sloppy — they are
decisions taken after it was written.

| # | Corpus says | Arc says | Standing |
|---|---|---|---|
| 1 | ~3 questions, 90s answer, 10–15 min discussion each, *"Question X of N"* | **One** question, ≤5 min, scheduled last | **D-7 wins.** Deletes the round, the counter, and the between-question transition. |
| 2 | Timer is load-bearing — big `M:SS`, amber bar, *"Final countdown"*, a `Timer` component | Timer competes with the argument for the same 30 seconds; try it without | **Arc wins, pending October.** Story B10's note. |
| 3 | Discord outage → *"keep running ≤15 min, new rooms paused"* | **AC-69**: authorized at creation only, 4-hour room lifetime | **AC-69 wins.** Dimension 2 measured real incidents at 20 min – 3 h; 15 minutes bought a degraded state that covered nothing. |
| 4 | Sessions and aggregates deleted after **30 days** | **AC-56**: no per-person record outlives the room; totals expire *with* it | **AC-56 wins** — it is strictly stronger, and §6 of the philosophy is why. |
| 5 | **3** native runs in the receipt | **AC-8**: at least **5**, non-identical output rejects | **AC-8 wins** (labelled a hypothesis, to revisit on rejection data). |
| 6 | Fifth choice is *"exhibits undefined behavior"* | **AC-24**: *"does not compile"* on **every** question so its presence signals nothing | **AC-24 wins.** A category-specific fifth option is exactly the structural tell §2 forbids. |

### The one real conflict — and it is a good one

`PROJECTOR_SPEC.md` §4.2 / §8.1, **resolved**: the trace is **host-driven only**;
participant phones stay read-only through reveal and *"never trace controls."*
Direction B here gives every participant their own stepper.

That was the right call in July and it collides with `PHILOSOPHY.md` §9, minted
yesterday from the client's own testimony: a newcomer should be able to work
through it **at their own pace, with nobody watching how many times they went
back**. A host-stepped trace runs at the pace of the person who already knows the
answer.

Not resolvable by reading either document — **T-15**.

### T-15 answered — both

**The wall is host-stepped; the phones step freely.** §4.2 keeps its narrative
build for the room, and §9 gets its private pace. The two are not the same
control and were only ever in conflict because the spec assumed one of them.

- **A and C** — the wall carries the trace, the host advances it, and the wall
  renders progress only (AC-79: no interaction beyond host controls).
- **B** — every participant steps their own copy at their own speed, and the
  wall shows the trace's **outcome** rather than an animation. A moving tape at
  the front would pull every eye off the surface where the work is happening.
  Same component, opposite call per direction, and that difference is now one of
  the things the fan-out is actually testing.

### The reconcile — decided and applied

The client's instruction was to read the package and decide. Read:
`PROJECTOR_SPEC.md`, `DESIGN.md`, `README.md`, `SUMMARY.md`, `tokens/colors.css`,
`tokens/typography.css`, `tokens/spacing.css`, `tokens/base.css`,
`components/quiz/SourceCode.jsx`, both manifests.

**Adopted:**

1. **The token CSS verbatim** — `prototypes/_shared/tokens.css` §1 is now the
   design system's four token files copied exactly, with the Phase-1 names kept
   only as a clearly-marked alias block that resolves into them. Nothing
   diverges; anything built from these takes deletes the aliases.
2. **Cascadia Mono.** `PRD.md` says *"a system monospace stack"*; the house brand
   says Cascadia Mono and is newer. The TTFs live in the design system's
   `assets/fonts/` and are not vendored here, so the takes fall back to SF Mono
   locally — a real build must ship the font.
3. **`SourceCode` with no syntax highlighting**, line numbers, white well,
   focusable scroll region. **The prototypes had highlighting and it was wrong.**
   The reason is only visible once §4.1 is read: the trace signals with
   highlight-and-dim, so a second colour channel underneath competes with the
   one thing the room is supposed to be reading. The design system had already
   resolved this and the resolution is right.
4. **The §4.1 trace model** — `{ lines, focus, note, values }` — replacing the
   bespoke tape, in all three directions. Step progress renders as **"Step N of
   M" plus dots**, the number being the accessible signal (§3.5).

**Not adopted, deliberately:** pixel-reproducing all nine components across three
directions. Two of the three get discarded at T-12, and component fidelity is
`tone-architect` and build work. Proportional, and recorded here rather than
skipped silently.

**New gap for `tone-architect`:** the **static fallback** (§6) — the same
projector views with baked data in one offline file, keyboard-driven, for when
generation, auth, or the venue wifi fails. The arc has never specced it. It is
close kin to `mvp/` and to Story A7 but is neither, and §6's design implication
is load-bearing: build the projector **view** separately from its **data
source**, so the fallback is `mode: "static"` + a fixture rather than a second
design.

Kept in the fan-out with both strikes recorded inside the take itself. It costs
nothing to keep now, and in October it is the **built** comparison for
*do phones help or hurt* rather than a hypothetical — which is exactly the job
T-8 sent the phones-vs-hands question here to do.

### The reconcile reaches the criteria — 2026-08-14, on resume

The Phase 1c reconcile was applied to the **prototypes** and stopped there. Two
items were left un-propagated overnight and were caught on resume, before T-12.
Client instruction: **amend both.**

| # | Was | Now |
|---|---|---|
| 1 | `PHILOSOPHY.md` §5 required *"real syntax highlighting"* and **AC-32** required source be *"syntax-highlighted"* — while the takes deliberately rendered none | §5 amended (legibility is the principle; highlighting was an implementation of it that works against the trace) and **AC-32 rewritten** to monospace, line-numbered, **without** highlighting. Recorded in `USER_STORIES.md` → Amendments. |
| 2 | Takes showed the pre-corpus room code `RUST-4417`; the design system's `RoomCode` is six characters (`KMT4XW`) | Six characters everywhere — `_shared/data.js` (canonical) and the two strings in `seam-discord.html`. Alphabet excludes `O`/`0` and `I`/`1`: the code is read off a projector from six metres back. No criterion changes — **AC-28** says only *"a short room code"*, **AC-49** only that it stays on screen. |

**Why it is recorded rather than fixed quietly.** For one day the corpus required
something the artifacts deliberately did not do, which is precisely the drift the
AC-lineage rule exists to catch. The lesson generalises: *the reconcile is not
done when the prototypes change — it is done when the criteria agree with them.*

Also corrected: `_shared/proto.js`'s file header still advertised *"Rust
highlighting"* — accurate about the trace's highlight-and-dim, and exactly the
stale label that would get syntax highlighting reintroduced by someone skimming.
The implementation was already correct.

**Deliberately not touched:** Phase 1 finding #3 — **AC-32 has no surface at all
in Direction C**, where the phone carries no code. That is a T-12 consequence and
waits for the direction call. This pass fixed only the highlighting clause, which
was wrong in every direction.

## Rust lunch, 2026-09-02 — first attendee contact, and a rehearsal is on offer

**What happened.** Client, reported 2026-09-02: *"i interviewed some rust
members at rust lunch … 2 people. there were only three of us at lunch, both
regulars. i didn't show them anything but they promised to help me do a
practice run."*

For the record:

- **First direct attendee contact of the arc.** D-5 put attendee interviews in
  scope on 2026-08-11; T-6 answered the *records* half and the *room* half
  slipped to October. This is neither — no room, no records — two conversations
  at lunch.
- **n = 2, both regulars, cold.** Nothing shown; the segment described at most.
  Testimony arrived later the same day — see below.
- **The offer:** a practice run with the two of them as participants.

**Why it is recorded rather than noted.** The calendar entry of 2026-08-13
wrote September off — RustConf, no meetup — and that is what put evidence and
ship date in the same month. **A rehearsal is not a meetup and needs no room,
so it can land anywhere before the October meetup** — the client confirmed
the meetup is October and left the rehearsal's date open. It retires exactly the unknowns that
need a host, a projector and someone looking, but not a crowd:

| Unknown | Standing until now |
|---|---|
| **AC-89** — one question, ≤ 5 min end to end | Never clocked with anyone watching. Every phase has been driven by the client alone, at a desk. |
| **AC-51** — no improvisation or apology | Never run before an audience. |
| **The wall-fit bound** (AC-33's sibling) | Never seen at a projector's real resolution. |
| **AC-97** — the walk-through, as a driven thing | Built and clicked through; never stepped with people looking. |
| **AC-98** — in the host's mouth, not the copy | Cut from two surfaces on 2026-08-14. Never tested in speech. |
| **The two room measurements** | Still the 15ft / 20ft **hypothesis**. Only need the venue, not a crowd — a rehearsal *at the venue* takes them a month early. |

October's one shot then goes to what only a crowd can answer — participation
rate, AC-44 / AC-96, §9's moment, Q2 — instead of being spent discovering that
the segment runs long.

**It does not reopen T-8.** The client hosts; the two are participants, not
delegates. Authorship stays where she put it.

**Costs and limits, stated up front, and written into the sheet so nobody reads
a clean rehearsal as a validated design:**

- **It spends `q3`.** C's vote, split and reveal are bound to `q3`'s mock room
  data — swapping the source swaps only the reading phases — and the two
  regulars will be in October's room. Bank **8 → 7, by hand**: nothing writes
  the ledger for a prototype run. That is the AC-92 defect in its other
  direction — a build that isn't a run wrongly retires a question, and a run
  that isn't a build retires nothing.
- **Two regulars are the wrong sample for every criterion about newcomers**,
  and they are *the people who know it* half of the one thing with nobody to
  tell. AC-44, AC-96, §9, Q2, Q1 and AC-88 are all listed in the sheet as
  things it cannot answer, with the reason each.
- **AC-91 comes back as a lower bound only.** Regulars read fast.

**Instrument:** `mvp/PRACTICE-RUN.md` — instrument and capture form, copied to
`mvp/<date>/practice-run.md` when filled, narrower than the field notes by
design. It drives **C, not the MVP deck**, because the deck predates Story B11
and has neither the split nor the walk-through.

**A stale host script, found while writing the sheet (T-19).** `mvp/README.md`'s
run-of-show still says *"Someone who said E — why?"* at 1:30 and *"go argue
about it downstairs"* at 2:30, and the argument thesis survives at lines 9, 16,
20 and 218. The first is the beat AC-98 forbids and that was cut from two
surfaces on 2026-08-14 — an instruction is an obligation however gently worded.
The rest is the framing A-5 replaced. The A-5 sweep covered the takes and the
philosophy; it never reached the MVP. The MVP is *not the product* and predates
both, so this is **flagged, not fixed** — but the README is the host script the
client actually reads before hosting, and a host who rehearses from it
rehearses the prohibited line. The practice-run sheet says not to.
**Recommended: sweep it.** The alternative — leave the MVP frozen as a
pre-A-5 artifact — is defensible only if the README stops being the thing the
host reads.

### Testimony, same day — three things, and where each lands (A-6)

Client, later on 2026-09-02: *"they remembered the quiz, liked the idea, liked
the idea of voting anonymously, the event is october."* Her report of what they
said, not their words; recorded as such.

| They said | Weight | Where it lands |
|---|---|---|
| **Remembered the quiz** | A second source that the dtolnay quiz ran and is remembered. **Not** a second source that it was memorised — nobody asked them that, and A-1 still rests on the client alone. | `06-attendees.md` §7. Ask it next time, in those words: *did you get to where you knew the answers?* |
| **Liked the idea** | The first attendee-side pull anywhere in the corpus. Weak — n = 2, regulars, in person, the organizer asking — and nonzero. | `06-attendees.md` §4a narrowed from *nobody asked for a quiz* to *nobody in the records*; §7 row updated. |
| **Liked the idea of voting anonymously** | The one with weight. §9 was minted on the client's testimony that *people aren't allowed to be wrong*, and its mechanism is *majority wrongness, anonymous, in public*. This is the first corroboration from the room's side — and from **regulars**, the people who least need the cover. Anonymity is not a newcomer accommodation; the people who know Rust want it too. | Confirms §9 and C's buzzer; no amendment. Adds a **second source to Direction A's strike** (public commitment against §9) — A stays the honest loser with more reason. Gives field-notes **Q2 a prior**, phones over hands, from two people; it is not Q2's answer, and the sheet already says a rehearsal cannot answer Q2. |
| **The event is October** | The meetup. Read as: the rehearsal precedes it, date open. | T-18 stays open on date and venue only. |

**Open (T-18):** the rehearsal's date and place — venue strongly preferred —
and confirmation that `q3` is the one to spend.

## Round 4 — the client's first clicks, 2026-09-03

Client, driving C from the T-20 guide: *"im clicking lights on but nothing"*,
then *"so many issues flipping from q3 to q1 and code beside to code full
width."* The first was not a defect — lights are up by default, so the button
had nothing to change; the guide now says so. The second was four defects and
one bad default, all reproduced in the c11 browser with screenshots before
anything was touched.

| # | What she saw | Cause | Fix |
|---|---|---|---|
| 1 | **q1 beside the options ran off the bottom of the wall**, through the join strip. | The source well is a flex item; flex items have `min-height:auto`, so a 16-line well grew past the wall and bled. The fit check measured `proj-body`'s scroll height and reported **zero** while code was drawn over the strip. | The well is bounded — `min-height:0; overflow:hidden`, scroll region `overflow:auto` — so overflow lands *inside* it, and the fit check measures that region. |
| 2 | **Full-width drew the legibility panel on top of the code.** | The type model's height constant for full-width assumed ~50px beneath the source; the options block is ~200px. Font came out at 16px, the well overflowed the space left, and the panel sat over it. | The constants are now a first guess only. After rendering, the wall is **measured** and the type shrunk until it fits or hits the legibility floor — up to six passes. `"Fits, legibly"` is now a measured claim. |
| 3 | **Nothing showed which source, layout, seat, or type mode was on.** Two clicks in and the wall's state was a mystery, and `hold at 27px` — sticky by design — looked like a broken layout. | No pressed state on any switcher button. | Every button carries `data-k` and reflects its state as `aria-pressed`; pressed is dark. |
| 4 | An orange **"98% of design size"** warning under the wall at all times. | Threshold was 99.5%; her pane is 98%. | Hidden above 90%, neutral colour below. |
| 5 | **The first screen was q1 with no options** — a dashed "legibility test" panel where the answers should be. | The page deliberately opened on q1 so the layout would meet a hard source first. Right instinct for an instrument; wrong first impression for the drive that decides whether she loves it. | Opens on **q3**, the real segment. The hard sources are one click away and now show as pressed. Panel copy shortened. |
| 6 | *(not reported — found on the way)* Title card: *"Then we all go argue about it."* | A-5 leak in the prototype itself; the sweep missed it. | *"Then we walk through it together."* |

**Honest consequence of #2, now visible instead of hidden:** under full-width
with five options beneath, `q1` (16 lines) **does not fit this room at a
legible size** — it clips at line 11 and the note says so in red. Beside the
options it fits at 14.9px, a hair above the 14.2px floor. Full-width buys line
width and spends ~200px of height on the options; that trade is what step 8 of
the guide asks the client to judge, and it was being judged against a wall
that lied.

**Verified** by re-measuring nine source × layout states after reload — zero
overflow wherever the note says *fits*, and overflow reported wherever it says
*too long* — and by screenshots of the initial load, q1 beside, q1 full-width,
q1 held at 27px, q3 beside, and the title card.

**Not changed:** `_shared/tokens.css` — the well fix is scoped to C's own
style so the other takes stay as they were judged. AC-33's sibling (the source
must *fit* the wall) is unchanged in meaning and now has a check that can see.

### The T-20 list, as it arrives

The guide asks for *everything that annoyed you*. Logged here in the order it
comes, each applied the same day so the next look at the wall is the corrected
one.

| # | Client said | Applied |
|---|---|---|
| 1 | *"One question. Five minutes. Then we walk through it together." — time for a pop quiz* | Title card hero is now **"Time for a pop quiz."**, with the two lines beneath it. The brand line already sits top-left of the wall, so the big "Pop Quiz" was saying it twice. |
| 2 | *"One question. Five minutes. Then we walk through it together." — remove* | Gone. The title card is one line: **Time for a pop quiz.** |
| 3 | *join @ bit.ly/jnfnvd* | The join strip on the wall now reads **join @ bit.ly/jnfnvd** (her placeholder URL) on the title card and during `question live`; the room code is **off the wall** in every phase. Closed says *answers are closed*; from the split on, *58 of 95 in the room answered*. The buzzer and host phone still show the code as the room's identity. **Criteria consequence, not yet minted:** AC-28 says joining requires only *a short room code*; the wall now says the way in is *a short link*. Same promise — nothing to install, nothing to sign up for — different mechanism, and the link can carry the code so nobody types it. Propose AC-28 → "a short link or code" when the list is done. |
| 4 | *"i think we're just gonna have a hard time getting the code to fit, practice will let us know how tall the screen is"* | **Recorded, not a change.** Her read agrees with the model now that the model can see: at the 15ft / 20ft guess, `q1` (16 lines) fits beside the options at 14.9px — a hair above the floor — and does not fit full-width at all; `q7`'s 69-character lines do not fit beside the options. The bound is real and it is the **generator's** to honour (AC-33's sibling, kin to AC-88), not the layout's to hide. The rehearsal's screen measurements set the number; **screen height** added to `PRACTICE-RUN.md`, because the model derives height from width at 16:9 and the venue's projector may not be. |
| 5 | *"how does dtolnay or rustlings do it"* | **Answered from the sources, 2026-09-03.** dtolnay's `rust-quiz` has an *authoring* rule, not a display trick: README — *"Aim for no more than 25 lines including blank lines, but shorter than that is better. Questions up to 35 lines may be accepted if there is no possible way to frame the same idea more concisely"* and *"Aim for no wider than 40 columns."* Measured across all 37 `.rs` files: **4–35 lines, median 19; widest line median 33 chars**, four outliers at 47–67. It is a one-reader web page (JavaScript-rendered, one question per page, the reader scrolls), so those limits are for a laptop, not a wall. Rustlings displays nothing — it is a CLI that watches files you edit in your own editor. **Consequence:** the wall's budget at the 15ft / 20ft guess — ~16 lines beside the options, fewer full-width — is **below dtolnay's median question**. The bound the generator inherits is tighter than the flagship bank's own rule, and it is the rehearsal's measurements that say by how much. Our bank is narrower in lines (5–16) but wider in characters (42–69) than dtolnay's; `q7` breaks his 40-column rule. |
| 6 | *"yes i had an opinion by 10 seconds"* | **AC-91, client's own read: 10 s on `q3`.** Recorded as a data point, not the sample — AC-91 is sampled on an attendee seeing the question for the first time, and the client has seen `q3` many times. It does say the 30-second budget is not tight for a five-line question at this size. |
| 7 | *"still readable but cut off"* at `6 metres back` | **AC-78: readable at six metres, on an 87%-scaled mock** — the note under the wall is right that this does not settle it. **The cut-off was two pressed buttons:** `hold at 27px` and `40ft` back row. At 40ft the legibility floor is **28.4px** and `q3` needs 20.7px to fit beside the options, so even the shortest question in the bank does not fit that room at a legible size — the model is telling the truth, and it was hidden below the wall where a scaled view cannot see it. **Fixed:** when the well clips, the wall now shows a **red edge on the side that lost content**. Pressing `auto` and `20ft` gives 20px, fits, no edge. |
| 8 | *"readable, syntax color readable, code readable, whole thing"* — `q3`, `auto`, 15ft / 20ft, beside the options, lights up | **The reading phase passes the client's eye at the venue guess**, colour included. For **T-17** that is the client's half — colour does not hurt legibility on this wall; Cole's half is still his. Not AC-78's sample (an 87%-scaled mock, and it is a rehearsal's measurement that sets the size), but the first time the whole reading phase has been called readable end to end. |
| 9 | *(screenshot of `4 · the split`, no words)* — five bars, all empty, counts beside them | **Defect, and an old one.** `.bar-fill` is a `span` inside a track that is not a flex container, so it stayed inline and ignored its width: every bar measured 0×0 and the split was numbers with no bars — on the split **and on the reveal**, where the correct bar's green fill was equally absent. Fixed with `display:block`. The split is §9's whole mechanism — *majority wrongness, anonymous, in public* — and until now the wall had been showing it as a column of figures. Measured after the fix: A 115 / B 25 / C 14 / D 45 / E 82 of a 284px track. |
| 10 | *"show me a hint, good"* | **AC-48 as rewritten (T-13) passes the client's eye:** the hint is pulled privately on the buzzer, the host phone does not know. No change. |
| 11 | *"question closed, good"* | **`3 · answers closed` passes.** The strip says *answers are closed* and nothing else. No change. |
| 12 | *"i like seeing the split before the answer"* | **AC-93 ANSWERED — `felt`, the client's alone, and it lands.** Split-before-answer stays as §9's mechanism, order and all. The first of the two never-answered `felt` calls is closed; **AC-21** (the review clock) remains, and is measured at the first real batch, not on this wall. |
| 13 | *"6 and 7 - love it"* | **The reveal and the release: loved.** The ✓ on the answer, the most-chosen wrong answer named and counted, the receipt, the read-aloud script; and the handover — *Let's go to the bar.*, the link, the QR. Both built since her last drive; both pass on first sight. |
| 14 | *"yes actually as the host i liked setting the pace"* | **AC-97 as built, and T-15's answer, confirmed from the host's chair:** the walk-through is host-stepped on the wall, and the host likes holding the pace. Step 5 clean — no answer, no receipt, no colour while the trace runs. |
| 15 | *"i love the options under, so code full width"* | **Layout decided: source full-width, options beneath in two columns.** Now C's default. **What it costs, measured at the 15ft / 20ft guess:** the options block takes ~200px of the wall's height, so the source gets ~250px — `q3` (5 lines) sits at 21.9px, `q5` (9 lines) at 15.5px, and **`q1` (16 lines) does not fit at a legible size**; it clips at line 11 with the red edge showing. Long lines are no longer the problem: `q7`'s 69 characters fit at 22.5px. **So the generator's bound under this layout is roughly 11 lines at the guess**, and wide is fine. That is well under dtolnay's median of 19 lines (item 5) — the rehearsal's screen height (item 4) is the number that moves it. Found on the way: a size fitted on the title card — where there is no well to measure — was the model's guess and overflowed the moment the question went live; the wall now refits on entering `live` or `closed`. |
| 16 | *"keep the colour, love it"* | **T-17 answered.** Colour stays in the reading phases, off while a trace runs. Propagated: **AC-99** minted under Story B8, `PHILOSOPHY.md` §5 amended a second time, `CLAUDE.md`'s AC range now 1…99. Recorded as **A-7**. Nothing in the prototype changes — it was already built this way as the proposal. |

### Closing the drive, 2026-09-03 — what an agent could do without her

The guide is fully driven and the list stands at sixteen. Three things did not
need the client and were done; one thing needs only her.

| # | What | Applied |
|---|---|---|
| 17 | **AC-28 rewritten** (item 3's deferred consequence). | *Joining requires only a short link or a short room code.* The link may carry the code. Original text kept in the criterion; entry in `USER_STORIES.md` → Amendments, together with AC-99's. |
| 18 | **T-19 swept.** | `mvp/README.md`: *does the room argue* → *work through it together*; *if the argument catches* → *the walk-through*; *the argument does not have to be cut off* → *the working-through*; the 1:30 row — *"Someone who said E — why?"* — is now *"Let's walk it. Nobody has to say anything."* with the host reading the program aloud a line at a time (AC-98); *go argue about it downstairs* → **"Let's go to the bar."**; *more argumentative than phones* → *everyone sees the split at once*. The *silent room at 1:30* advice became *few hands at 0:30*, because under AC-98 a quiet walk-through is not a symptom. |
| 19 | **An A-5 leak in the product**, not the docs. | The reveal script's third beat — the one the host reads aloud, and the one take-it-home ends on — was headed **"The bit worth arguing about"** in `_shared/proto.js` and `take-it-home.html`. Now **"The bit worth talking about."** The field is still called `argue` in `_shared/data.js`; renaming it is the content-model change already flagged for `tone-architect`, not a copy fix. |
| — | **`DESIGN.md` drafted.** | Records what C does — the three surfaces, the seven phases, voice, aesthetic, the type model, what is deliberately absent, the design-system divergences, the hypotheses, and what is open for the architect. Marked draft; binding on the word. `CLAUDE.md` points at it. |

**Not done, on purpose:** `tone-architect`. The handoff rule was written the day
the direction was picked and it still holds — the stage ends on *love it*, said
about the whole, not on three *love it*s about parts. Everything that could be
cleared before that word has been.

## Stage 2 — COMPLETE 2026-09-03. Handed off to `tone-architect`.

Client, on `DESIGN.md` and the drive: ***"yes i love it."*** The stage rule was
love, not approval, and the working test — *would she keep this if a better
option appeared tomorrow?* — is answered by a client who asked *"are we done
with planning now?!"* before the word was even requested.

| | |
|---|---|
| **Takes produced** | 5 clickable prototypes (A, B, C, the review surface, the Discord seam) + `take-it-home` + `index` |
| **Direction** | **C — projector-first, phone as buzzer.** Stated by the client at A-4, confirmed at T-12, loved at T-20. |
| **Rounds on C** | 4 — full-width and the type model; the walk-through beat; the drive-through answers; the T-20 list of 16 |
| **Criteria touched** | AC-93…AC-99 minted; AC-23, AC-32, AC-48, AC-28 rewritten; AC-45's host control deleted |
| **Philosophy** | The one thing rewritten (A-5); §5 amended twice; §9 added |
| **Touchpoints** | T-11…T-20, all answered; T-18 (the rehearsal) carries into Stage 3 as the client's own work |
| **Binding** | `DESIGN.md` + `prototypes/C-projector-first.html` + `prototypes/take-it-home.html` |

**Carried into Stage 3, none blocking:** T-18 (rehearsal date, venue, the
three room measurements); the AC-92 mechanism defect; the static fallback
(`PROJECTOR_SPEC` §6); the three-field explanation content model; AC-95's count;
AC-21 unmeasured; font vendoring; the real short link.
