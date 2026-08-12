# Run state — Rust NYC Pop Quiz

The resume anchor for the whole Tone arc. Every stage reads this first on invoke.

| | |
|---|---|
| **Stage** | 1 — `tone-initiation` |
| **Phase** | 4 — Stories review **complete**. Ready for handoff to `tone-prototype`. |
| **Repo** | `~/rust-nyc-pop-quiz` (canonical) |
| **Branch** | `ai-c11-cc/tone-initiation`, off `codex/valtown-fresh-quiz` @ `f6ea174` |
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

## Living-artifact amendments

| # | Artifact | Change |
|---|---|---|
| A-1 | `research/01-problem-and-people.md` | Verdict "problem unproven; solution in search of a problem" **withdrawn**. Client testimony — *"attendees had gotten all answers over and over, we stopped doing dtolnay's quiz at all"* — is primary evidence that the problem is observed, acute, and terminal. Amendment recorded in-file; original retained for the record. |

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
| D-5 | Attendee interviews in scope | Client has access. Not yet conducted — see T-6. |
| D-6 | Borrow option (C) rejected | Code display is disqualifying across all candidates. Recorded rather than escalated because the evidence is one-sided. |

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
| T-6 | 1–2 | Attendee interviews (client has access) | Not opened — recommended before Phase 3 stories |

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

## Phase 3 output

- `PHILOSOPHY.md` — the one thing (*the quiz is a pretext; the argument is the
  product*), 8 principles, taste. Referenced by `CLAUDE.md`.
- `sequence/USER_STORIES.md` — **17 stories, AC-1 … AC-88, 12 marked `felt`** after the Phase 4 review (13 stories / AC-1…AC-70 / 10 felt as first minted).
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

## Remaining in this stage

- **T-5 stories review** — client reads AC-1…AC-70, iterates, and I propose the
  stories they didn't think of.
- **T-6 attendee interviews** — not yet conducted. The Aug 12 run is the cheapest
  opportunity; four attendee CSVs sit in `~/Downloads`.
- Then handoff to `tone-prototype`.
