# Specification — Rust NYC Pop Quiz

**Status: Stage 3 `tone-architect`, Phase 2. Written 2026-09-03.** This is the
build contract. It codifies the loved prototype (`prototypes/C-projector-first.html`,
`prototypes/take-it-home.html`, `DESIGN.md`) and the criteria
(`sequence/USER_STORIES.md`, AC-1…AC-100) so that an implementer who has never
spoken to the client can build it with **no design decision left to make**.
**Precedence:** the criteria (AC-1…AC-100) outrank both this file and the
prototype. Below them, the prototype wins on **visual and typographic detail**
— spacing, weight, colour values, layout — and this file is corrected to
match; this file wins on **behaviour, state, payloads, and copy**, because
the prototype is a mock with known shortcuts (its host phone has one screen
for `closed`/`split`/`work`, no `←`/`→` in `work`, and a `lights down`
instrument that is not a product mode). Where the prototype violates a
criterion, the build follows the criterion and the prototype is patched, not
copied. `DESIGN.md` sits beside the prototype on visual detail and below this
file on behaviour. **The source well's font size is behaviour, not visual detail** — it is
derived by §5.2 and never read off the prototype; every other size (options,
bars, beats, strips) is visual detail and is the prototype's. `EVALUATION.md` says how
each criterion is proven. `BUILDPLAN.md` says in what order, on what stack.

Criteria are cited by ID throughout; §14 indexes every one.

---

## 1. What is being built

Two decoupled components and two small pages:

| Component | What it is | Runs |
|---|---|---|
| **The pipeline** | Generate → verify → dedupe → review → a private bank; schedule one question per meetup; audit the bank. | Offline, on an organizer's laptop or a scheduled job. Never inside a room. |
| **The room** | One room per meetup, created by an organizer, joined by phones, driven through seven phases from the host phone, shown on the wall. | A hosted service, live for ≤ 4 hours. |
| **Take it home** | A public page holding the *last* meetup's question, explanation, trace and receipt. No room state. | Static, rebuilt at each release. |
| **The static fallback** | The wall's seven views with the night's question baked in, keyboard-driven, one file, no network. | Opened from a laptop when the room cannot run. |

**The segment** is one question, scheduled last, ≤ 5 minutes end to end
(AC-89, AC-90). There is no round, no navigation, no second question (§4.6).

### 1.1 Non-goals

No accounts, nicknames, scores, leaderboards, history, or per-person records of
any kind (AC-57). No dark mode (AC-80, answered). No timer on the wall. No
public question bank. No live generation. No multi-question rooms. No host
hint control (AC-45). No instruction to talk to anyone (AC-98). No syntax colour
while a trace runs (AC-99). No balancing of answer positions, ever (AC-23a).

---

## 2. Guardrails — enforced, never assumed

Each guardrail has a pass/fail criterion and **must** carry its own
enforcement or audit ticket in `BUILDPLAN.md`. Where the build sits on existing
code (`mvp/tools/`), the audit ticket audits that code too.

| G | Guardrail | Enforced by | Audit demand |
|---|---|---|---|
| **G-1** | The correct-answer position is a pure function of the date. No history, ledger, bank state, or prior position is an input. Balancing, quotas, and *never the same as last time* are prohibited. (AC-23, 23a, 23b) | `slot_for_day(date) → 0..4`, no other parameter; a static lint that no module reading `answer-history` is imported by the slot path; `bank-audit` both tails. | Audit `mvp/tools/build_deck.py` on inheritance; every PR touching the slot path re-runs the perfect-memory simulation. This has regressed four times. |
| **G-2** | No hand-written answer reaches a deck or a room. (AC-7) | The correct option is derived only from the verifier's recorded output; a provenance field names the verifier run; the build refuses a question whose answer field lacks it. | Audit `mvp/tools/verify.py` and `build_deck.py`; test that a hand-edited `correct` fails. |
| **G-3** | Nothing pre-reveal carries the **join between the options and the verified output** — which option is correct, by letter, flag, ✓, ordering, or `verified.stdout` itself — nor the explanation, the receipt, or the trace's resolving step (any `values` entry named `stdout`, and the final step), which is withheld until `reveal`. Option **texts** are public in every phase (AC-62); program state in the trace from `closed` on is allowed (that is what walking a program is). (AC-47, AC-60, AC-61, AC-79, AC-97; **D-10**) | Answer storage is a separate module/type unreachable from the public state query (compile-time or module boundary); `canary` plants secrets and scans every payload with the phase-scoped rules above. | `canary` in `test`; a ticket that proves AC-61 structurally, not by review. |
| **G-4** | Nothing per-person is stored beyond the room, and nothing per-person leaves the phone after close. (AC-56, AC-57, AC-58) | Schema has no per-participant answer table; totals carry an expiry; the recap is client-computed. | Schema audit ticket; `canary` on post-close traffic. |
| **G-5** | No participant-facing string obliges anyone to speak or interact. (AC-98) | A forbidden-copy lint over every participant-facing string in `test`; no UI element counts, prompts for, or waits on a contribution. | The lint is a ticket; the string list lives in §11 and is the only place copy is authored. |
| **G-6** | The phase order is `idle → live → closed → split → work → reveal → released`, each transition a host action, none skippable. (AC-45, AC-93, AC-97) | A phase machine with exactly these transitions; the wall, buzzer and host phone render from the same phase value (AC-81). | State-machine test; `canary` that `work` carries no ✓, receipt, or colour. |
| **G-7** | The receipt claims exactly what verification proved. (AC-43, AC-71, AC-87) | Receipt strings are generated from the `verified` record by one function with approved wording (§7.5). | String test; HC-2 reads it. |
| **G-8** | Neither source nor trace is rendered on a participant device. (AC-32) | Participant payloads have no source field; `canary`. | `canary` in `test`. |
| **G-9** | Hosting is authorized by role **ID** in `roles`, checked at room creation. (AC-64, AC-65, AC-69) | One auth function; the Administrator test; the Discord-down test. | Auth audit ticket. |
| **G-10** | A question is *used* when a room is **released**, not when a deck is built. (AC-92) | The used-question record's only writer is the release transition. Building writes nothing. | Audit and retire `build_deck.py`'s write to `answer-history.json`. |
| **G-11** | No answer-category distribution is published anywhere an attendee can read. (AC-25) | `bank-audit` scans participant-facing artifacts; organizer docs carry the tells. | Ticket for the scan. |
| **G-12** | Every question scheduled has an organizer's affirmation of its explanation. (AC-72, AC-95) | Scheduling refuses unaffirmed questions; affirmation records who and when. | Gate test. |

---

## 3. Domain model

Every persisted field has one writer and at least one reader. A field with no
reader is deleted; a field with no writer is a defect.

### 3.1 Question (the bank)

| Field | Writer | Readers |
|---|---|---|
| `id` | generator (or the organizer, for hand-written) | everything |
| `source` — the Rust program, ≤ the room's fit bound (AC-100) | generator / organizer edit | verifier, dedupe, wall, review, take-it-home, static fallback |
| `topic`, `difficulty_requested` | generator from the run request | review, `bank-audit` (AC-26, AC-88) |
| `options[5]` — text, each with `kind ∈ {output, does_not_compile, ub, panic}`; exactly one `does_not_compile` (AC-24) | generator / organizer edit | wall, buzzer, review, take-it-home |
| `correct` — the option whose text equals the verified output, **derived** (G-2) | the verifier's output reader | reveal, review, take-it-home, `bank-audit` |
| `hint` | generator / organizer edit | buzzer (live), review |
| `explains` — `what`; `why_tempting` — **one short text per incorrect option** (why a reader would pick it); `takeaway` (AC-95, D-9) | generator / organizer edit | host phone (reveal), take-it-home, review |
| `trace` — `{steps: [{lines, focus, note, values, pivot?}]}` — §5.3 defines the semantics; the last step is the one whose `values` names `stdout` | generator / organizer edit | wall (work, reveal), take-it-home |
| `verified` — §3.2 | the verifier only | receipt, `bank-audit`, scheduling |
| `review` — `{status ∈ accepted|rejected|edited, reason, difficulty_judged, affirmed_by, affirmed_at, near_duplicate_of?}` | the review surface | scheduling (G-12), generator (rejection reasons), `bank-audit` (AC-88) |
| `used` — `{meetup_date, room_id, released_at, fit}` where `fit` is the wall's verdict **at `reveal`**, the last refit of the night | the release transition only (G-10) | reserve count, scheduling (never twice), the organizer's ledger report (`popquiz sync`), which lists any room whose `fit` was not *fits* |

The `explains` shape replaces the MVP's single `explanation` string; the MVP's
string is kept verbatim as `explains.legacy` for AC-73's quoted-output check on
inherited questions. Field names are **English words**, not the prototype's
`argue` (a Stage-2 leak).

### 3.2 Verified record

Written by `verify` and never edited: `rustc` (full `-Vv`), `edition`,
`target_triple`, `flags` (`opt-level`, `overflow-checks`, `debug-assertions`),
`runs` (count, all byte-identical: bool), `stdout`, `exit_code`, `miri`
(`{version, configs: [stacked_borrows, tree_borrows?], clean: bool, output_matched: bool, seeds}`),
`compile_error_code?`, `verified_at`, `verifier_version`. AC-6…AC-13, AC-87.
The receipt renders from this alone (AC-13).

### 3.3 Bank

An append-only store of questions plus `history` (exact hashes, normalized-AST
fingerprints, token-bigram sets) for AC-14…AC-17. Reserve = accepted ∧ affirmed ∧
unused (AC-75). Threshold and lead time are configuration (AC-76; default:
warn when reserve < 2 meetups).

### 3.4 Room

| Field | Writer | Readers |
|---|---|---|
| `id`, `code` (6 chars, alphabet without `O 0 I 1`), `join_url` | creation | wall (`join @`), buzzer, host |
| `question_id` | creation (from the schedule) | every phase |
| `phase` | host transitions only (G-6) | wall, buzzer, host (AC-81) |
| `host_session` | creation; **never rotates during the room** (AC-50) | host auth |
| `host_resume_url` — a secret link shown on the host phone under *if you lose this phone* | creation | a second host device (AC-50) |
| `created_at`, `expires_at = created_at + 4h` (AC-69) | creation | lifecycle |
| `present` — count of live participant sessions | join/leave | host (AC-46), wall strip |
| `answered_live` — sessions currently holding an answer | answer upsert / clear | host phone while `live` (AC-46) |
| `answered` — sessions with an answer, **frozen at `closed`** | the close transition (from `answered_live`) | wall strip (*N of M answered*), host phone |
| `totals[A..E]` — per-option counts, the only answer data the room ever holds; die with the room (AC-56, D-12) | the close transition, computed from sessions | split, reveal, host |
| `trace_step` — in `work` bounded to `0..M-2`; `reveal` enters at `M-1` and may step the whole trace | the `work` transition (`0`), the `reveal` transition (`M-1`), host `←`/`→` | wall, host phone (the step's words) (D-10) |
| `released_at` | release | `used` writer, take-it-home rebuild |
| `fit ∈ {fits, clipped_x, clipped_y, clipped_xy}` — the wall's measured verdict after refit (§5.2) | the wall, after each refit | host phone (*fit* line, §11); the release transition copies the `reveal` verdict into `used` (AC-100 evidence) |

**Participant session** (ephemeral, dies with the room): `token`,
`answer ∈ A..E | none`. **No** hint flag (AC-48 — the hint is in
the live payload, §4.2), **no** identity, **no** device record (AC-57).
Totals are computed at `closed` and the sessions' answers are then
irrelevant to any reader; they are dropped at release.

### 3.5 Organizer

`discord_user_id`, `access_token`, `refresh_token` (rotated and persisted on
every refresh, AC-66), `token_expires_at`. Rooms reference their creating
organizer (AC-68). Nothing else.

---

## 4. The room — the phase machine

`idle → live → closed → split → work → reveal → released`. Transitions are
host-phone actions only. No timer, no auto-advance, no skipping (G-6). `←`/`→`
inside `work` and `reveal` step the trace and are not phase transitions.

| Phase | Host action to enter | Wall | Buzzer | Host phone |
|---|---|---|---|---|
| **idle** | *create room* | Title card: brand line top-left; **Time for a pop quiz.**; join strip `join @ ‹link›` | Room code; ↑ *You're in. Everything happens on the screen at the front — look up.*; foot `no account · no name · no score` | *Put it on the screen*; present count; the code |
| **live** | *Put it on the screen* | Source (colour, §5.3), options beneath in two columns, join strip `join @ ‹link› · still open`. **No timer.** | Letters A–E (tap to answer, change freely, AC-34); saving/saved/failed (AC-35); *Show me a hint* (§4.2) | *Close answers*; present and answered counts (AC-46); **no answer** (AC-47) |
| **closed** | *Close answers* | Same source and options; strip: **answers are closed** — nothing else | Letters locked; last saved answer shown | *Show the room its split* |
| **split** | *Show the room its split* | Five bars with counts (`n · p%`), *N of M in the room answered*; **no answer** | *Look up. Where the room landed.* — the count `n`, *people said X, including you.*; foot `computed on this phone · never sent anywhere` | *Let's walk it* |
| **work** | *Let's walk it* | Source **without colour**; trace at `trace_step` over steps `0..M-2` only (highlight-and-dim, *Step N of M* + dots); beat panel: **Let's walk it. / Still no answer. / Nobody has to say anything.** **No ✓, no receipt, no `stdout` value** (AC-97, D-10) | *Look up. We're walking it through.* — the count; *Nothing to do. Nobody knows the answer yet, including us.* | `←` `→`; the step's words; *Reveal* |
| **reveal** | *Reveal* | ✓ on the correct option (glyph + colour, AC-40); the most-chosen incorrect option named and counted; the receipt (§7.5) with provenance *established by the machine*; the trace **entering at its final step** `M-1` (the one that prints), steppable back through all of it, no colour | *The answer is on the screen.* — the count; **✓ It was X.**; *You and n other people read it the same way. The why is being read out now — listen, don't read.* No ✗ (AC-94) | **Read it aloud** — the three beats (§3.1 `explains`, §4.5), provenance *human*; `←` `→`; *Release the room* |
| **released** | *Release the room* | **Let's go to the bar.** The take-it-home link at 40 px, one line of what is behind it, a QR. Nothing else (T-20 item 13) | ✓ *Nothing about you was recorded. Not your answer, not your device, not that you were here.* | *Run it again* → a new room (never the same question, G-10) |

### 4.1 Joining (AC-28…AC-31)

Join via the short link (which carries the code) or by typing the six-character
code. One field, no other input. Failure states, each with its own sentence and
next step, authored in §11: **malformed**, **unknown**, **not yet open**,
**already ended**, **closed for inactivity**, **full** (AC-29). Capacity is checked before a
session is created and reserves nothing (AC-30). Capacity: 200 (AC-52),
configurable.

### 4.2 The hint (AC-48)

The hint text is part of every buzzer's `live` payload, rendered hidden.
*Show me a hint* reveals it client-side. **No request is made, so nothing can
be recorded and no signal can reach the host or the wall.** This is the smaller
mechanism (§8 of the philosophy) and it makes AC-48 provable by `canary`.
AC-60's *unpublished hint* clause is read as: before `live` no payload carries
the hint; from `live` it is published to everyone by construction. Recorded as
D-8 in `run-state.md`.

### 4.3 Answering (AC-34…AC-37)

Answers are upserted per session while `live`; the last write wins; a write
after `closed` is refused with the saved answer restated. The buzzer shows
exactly one of *saving / saved / failed*; failed says the last saved answer is
safe and offers retry. A dropped socket re-attaches with the same session
token; controls show *paused* until fresh state arrives.

### 4.4 Closing and totals

`closed` computes `totals` from sessions once and freezes them. `present`
keeps counting; *N of M in the room answered* uses the frozen `answered` count.
Zero votes on an option renders an empty bar with `0 · 0%`.

### 4.5 The most-chosen incorrect option (AC-95, D-9)

There is no prediction. Every incorrect option carries its own authored
`why_tempting` text (review refuses a question missing one, §7.4). At
`closed` the room's most-chosen incorrect option is computed from `totals`
(ties: the lower letter). At `reveal` the **wall** names it with its count,
and the **host phone's** middle beat is headed *Why ‹n› of us said ‹X›* for
the **same** option, followed by that option's `why_tempting` text. Wall and
host phone can never name different options; `test` asserts they agree.
**When no incorrect option received a vote** — everyone right, or nobody
answered — the wall's middle line and the host's middle beat are replaced by
the §11 *nobody read it another way* string, and `test` asserts that variant.
Take-it-home carries neither the option nor the count (D-12, §13).

### 4.6 Lifecycle

A room lives ≤ 4 h from creation (AC-69) and is deleted with its sessions at
release or expiry. **Closed for inactivity:** a room in `idle` with no host
action for **30 minutes**, or in any later phase with no host action for **20
minutes**, closes; joins after that return the inactivity message (AC-29).
At release or expiry the room record and its sessions are deleted whole —
`totals` included, since D-12 left them no reader. What outlives the room is
the pipeline's `used` record (G-10) and the rebuilt take-it-home page. Creating a room requires a live
Discord check; running one does not (§8). *Run it again* creates a **new**
room and the schedule refuses a used question (G-10).

---

## 5. The wall

### 5.1 Geometry

The wall lays out at a **1120 × 630 design canvas** and scales as a unit
(`transform: scale`) to the projector's actual pixels — never `width: 100%`
on the box. Brand line top-left, `PROTOTYPE` badge removed in the build,
the join strip at the bottom.

### 5.2 Layout and the type model (AC-100, AC-78)

Source full-width, options beneath in two columns. Type size is **derived**:

```
screen_h_in  = screen_height_ft * 12                        # measured; default = width × 9/16
cap_in       = back_row_ft * 12 / 150                       # cap height ≥ distance/150
font_in      = cap_in / 0.7                                 # monospace cap ratio
floor_px     = font_in / screen_h_in * 630
code_area    = reading layout (live, closed, split):  994 × 177 px   (the TEXT box)
               trace layout (work, reveal):           994 × 190 px
               Both MEASURED 2026-09-18 from the prototype at its 1120 × 630 design size
               (the well's scroll region minus its 16 px padding: 1026 × 209 → 994 × 177 in
               `live`, where the `.proj-code` box is 1028 × 247 under a 190 px options block
               and the well's 36 px header sits above; 1026 × 222 → 994 × 190 in `work`
               under a 203 px beat block). These are text boxes, not outer boxes — the
               header, borders and padding are already subtracted, so no further chrome
               is deducted below. The wall is full-width in
               every phase (T-20 item 15); nothing is ever beside the source. The block
               beneath the source is a FIXED reserve — 190 px for the options, 203 px for
               the step note (clamped to three lines) and values table — so both areas are
               constants an offline auditor can use. `bank-audit` (§7.6) fits against the
               reading layout's 177 px text box, the smaller, and reports the trace verdict beside it.
               THE OPTIONS BLOCK IS A CONSTANT because options are constrained (D-15): each
               option is ONE line of at most 29 characters at the wall's 24 px option size
               (510 px cell − 40 px letter chip − 14 px gap − 2 × 14 px padding = 428 px =
               29 chars of 14.4 px). Three rows (A B / C D / E) of 58 px with two 8 px
               gaps = 190 px. The wall has no design for a wrapped option — the loved
               prototype only ever showed q3's — so an option over 29 characters is a
               `bank-audit` failure (§7.6), never a layout the wall improvises.
               Line height in the well is 1.6 (MEASURED: `.rn-src pre { line-height: 1.6 }`).
               Worked at the 15 ft / 20 ft guess (floor 14.2 px; the reading layout holds
               floor(177 / (14.2 × 1.6)) = 7 lines at the floor, the trace layout 8):
                 q3  5 × 42 → min(177/8.0, 994/27.3) = 22.1 px  fits   (prototype measured 22.0)
                 q4  6 × 56 → min(177/9.6, 994/35.7) = 18.4 px  fits as a program
                 q7  5 × 69 → min(177/8.0, 994/43.5) = 22.1 px  fits as a program
                 q8  6 × 30 → 18.4 px                           fits as a program
                 q5  9 × 50 → 177/14.4 = 12.3 px  below the floor: too long
                 q6  9 × 32 → 12.3 px             too long
                 q2 10 × 38 → 11.1 px             too long
                 q1 16 × 36 →  6.9 px             too long
               By source length only q3, q4, q7, q8 fit. By the option rule only q3 passes as
               authored. Six questions print output that spans lines (q1, q2, q4, q5, q6,
               q8 — each has a multi-line option, none over 29 characters on any one line
               except q1 and q2's whole options at 32 and 45), and q7 has a single option
               line of 48 characters. So seven of the eight need re-authoring before they
               can run on the built wall, four of them (q1, q2, q5, q6) also being too
               long as programs. The rehearsal's screen and back-row measurements (H-5)
               are what move the floor, and with it every number here.
by_height    = code_area_h / (lines * 1.6)
by_width     = code_area_w / ((widest_chars + 3.5) * 0.6)
font_px      = max(floor_px, min(by_height, by_width, 46))   # the floor wins over the 46 cap; if floor > 46 the wall reports it
```

Then the well is **measured** after render. While anything overflows and
`font_px > floor_px`, at most six times:

```
k        = min(overX > 2 ? content_w / scroll_w : 1, overY > 2 ? content_h / scroll_h : 1)
font_px  = max(floor_px, font_px * k * 0.97)
```

If it still overflows at the floor, the well clips and shows a **red edge on
the side that lost content**, and the host phone shows the matching fit line
from §11. Room configuration (`screen_width_ft`, `screen_height_ft`,
`back_row_ft`) is organizer-set, defaults `15 / 8.44 / 20`, labelled
hypothesis until HC-1. **Refit on entering every phase that renders the
source** — `live`, `closed`, `split`, `work`, `reveal` — against that phase's
code area; the trace phases never use a hard-coded size.

### 5.3 The source well

Cascadia Mono, line numbers in a 2ch gutter, white well, 1.6 line-height (measured).
**Syntax colour** (keyword, macro, type, string, number, comment; muted
values) in `live`, `closed`, `split`; **none** in `work` and `reveal`
(AC-99). `idle` and `released` render no source. The trace renders by
highlight-and-dim with *Step N of M* and dots; the number is the accessible
signal.

**Trace model.** `trace.steps[i]`: `lines` — 1-based source lines drawn as
executing now (highlighted); `focus` — `[first, last]` 1-based inclusive
range kept at full contrast, everything outside it dimmed; `note` — the
words the host reads for this step; `values` — `[{name, was, now}]` shown as
a small table beneath the source, inside the reserved block, `was`/`now` verbatim strings, `—` for
absent; `pivot` — optional, marks the one step worth stopping on (the wall
adds *the one worth stopping on*). The last step's `values` names `stdout`
and is the resolving step (D-10).

### 5.4 The split

Five horizontal bars, one per option, `display: block` fills, count and
percentage beside each, correct option unmarked. On reveal the correct bar
gains the ✓ and the green border; no bar is ever marked wrong.

### 5.5 Released

*Let's go to the bar.* in Instrument Serif; the link in 40 px monospace; the
QR encoding the same link; one line: *the question, the walk-through and the
why — at your own pace.*

---

## 6. The buzzer and the host phone

**Buzzer:** 375 px design width, 44 px targets (AC-85), letters as
`ChoiceButton`s, the code in the header, the foot line per phase (§4). Never
source, never trace (G-8). Live region announces each phase and the hint
reveal (AC-83).

**Host phone:** one screen per phase with the phase label, **one primary
action**, present/answered counts, the room code (AC-49). In `work` and
`reveal`, `←` `→` and the step's words. In `reveal`, the three beats under
**Read it aloud**. The first screen carries the resume link (§11); opening it on
another device attaches that device to the same room; the session never
rotates during a room (AC-50). The first screen also carries §8.1's two
sentences.
No answer preview before `reveal` (AC-47, G-3).

---

## 7. The pipeline

### 7.1 Generate (AC-1…AC-5)

A CLI run: `count`, `topics[]`, `difficulty`, optional `talk {title, abstract}`,
and the **room's capacity** read from the room configuration (§5.2), which is
three facts: the most source lines the wall holds at the floor (7 at the
default guess); that **each option is a single line** — a program whose printed
output spans lines is rewritten to print on one line, never joined or wrapped —
and that no option is longer than **29 characters**. The generator is told all
three and asked to honour them; a candidate that breaks any is reported, not
silently trimmed (D-15).
Emits candidates in the §3.1 shape minus `verified` and `review`, plus a run
report: per-candidate cost, tokens, wall-clock, and which of the three
requests it could not honour. Re-runnable with no cleanup (AC-2). Talk mode
tags each candidate with a named std/core concept (AC-4). **Never invoked by
the room** (AC-1, static check).

### 7.2 Verify (AC-6…AC-13, AC-87, AC-12)

Inherits `mvp/tools/verify.py`'s procedure: pinned rustc and edition, **N=5**
native runs (byte-identical or **rejected**, AC-8), expected compile-failure
must fail with its error code recorded (AC-11). **Brownfield fact: `verify.py`
never invokes Miri** — the `miri` fields in `mvp/2026-08-12/verified.json`
were written by a pass outside the repo (`mvp/README.md` says so). The `MVP`
label on AC-9 and AC-10 therefore rests on a hand-run step, and the build must
put Miri **in the verifier**: strict provenance, output compared to native
(AC-9, AC-10), Tree Borrows as a second config for UB-intended candidates.
Adds: the flag set in the record (AC-6), a sandbox with no network, no secrets,
no host filesystem, CPU/memory/time limits (AC-12), and `target_triple` in the
receipt (AC-87). `verified.json` gains no field that code did not write (G-2's
spirit: never overstate verification).

The MVP's `content.json` keeps distractors and explanation apart from the
verified facts; the build keeps that separation — §3.1's `options`, `hint`,
`explains`, `trace` are authored content, `verified` is machine-written, and
the `correct` field is the join between them.

### 7.3 Dedupe (AC-14…AC-18)

Exact hash → reject. Normalized AST (alpha-renamed bindings, formatted) →
reject. Near-duplicates: **Jaccard similarity over the normalized token
bigrams** of two programs at or above a configured threshold (start 0.6,
labelled uncalibrated) → **review queue**, marked near-duplicate of `q`. No
embedding model in v1 — the smaller mechanism, and no external dependency
(D-13). The
history persists in the bank and its size is on the organizer's first screen.
Every uniqueness statement, on every surface, reads *no exact or normalized
duplicate found* and never *original* (AC-18).

### 7.4 Review (AC-19…AC-22, AC-72…AC-74, AC-88, AC-95)

One screen per candidate: source, five options, verified answer, three beats,
receipt, near-duplicate note, difficulty requested and a control to record
difficulty judged. Actions: accept, reject (reason required), edit (re-verify
on any source or option change). **Affirm** is separate and blocking (G-12):
records who and when; **every incorrect option must have a non-empty
`why_tempting` text** (AC-95, §4.5); quoted output is checked against `verified.stdout` (AC-73). The
surface states that the reviewer has seen the answers (AC-22).

### 7.5 The receipt (G-7)

Rendered from `verified` by one function:

> ✓ established by the machine — rustc ‹version› (‹edition›). ‹N› byte-identical
> runs on ‹triple›; that is evidence about ‹triple›, not every machine. Miri
> clean on the executed paths; output matched the native runs. **Establishes
> the answer. Not the explanation — that one is human.**

This string is normative; `EVALUATION.md` AC-43 and AC-87 quote it.

### 7.6 Bank audit (AC-23b…AC-27, AC-100)

Runs on every build and bank change: generator uniformity (20,000 synthetic
draws, χ² df=4, both tails); five options with one *does not compile*; the
enumerated tells — `unsafe` presence, source length, option text length,
option position, topic — each against the accumulated bank, failing above
**1.5× chance** (the stated margin; revisit as the bank grows); AC-27's
`unsafe` parity; fit against the configured room at the floor in the
**reading layout's** text box, 994 × 177, the smaller (AC-100, §5.2),
reporting the trace layout's verdict beside it; **option length** — every
option one line of at most 29 characters (§5.2, D-15); and
**difficulty drift** — across a run's accepted questions, `difficulty_judged`
more than one level from `difficulty_requested` on average fails the **run**
(AC-88), not the question.

### 7.7 Schedule and use (AC-89…AC-92, G-1, G-10)

One question per meetup, chosen by the organizer from the reserve. The
answer's position is `slot_for_day(meetup_date)`, a pure uniform draw from the
date. The `used` record is written at **release** and nowhere else. The
organizer's screen shows reserve count and trend (AC-75) and warns below
threshold (AC-76). A meetup runs from reserve with no generation and no
outbound network beyond serving the room (AC-77).

---

## 8. Authorization (AC-64…AC-70)

Discord OAuth2 with scopes `identify guilds.members.read`. On *create room*:
`GET /users/@me/guilds/{guild}/member` with the organizer's bearer token; host
iff `roles` contains the configured role **ID** (never a name, never the
`permissions` bitfield, AC-65). Refresh tokens rotate; every rotation is
persisted before the old one is discarded (AC-66). Participants never
authenticate (AC-67). Only the creating organizer reads or controls a room
(AC-68). No check after creation; a room runs to release with Discord down,
bounded at 4 h (AC-69). Denials say *wrong server* or *wrong role* and never
whether the user is a member (AC-70). Retries are bounded and backed off —
the 10,000-invalid-requests ban is IP-wide.

### 8.1 What is not a security guarantee (AC-62, AC-63)

The organizer runbook and the host phone's first screen carry these two
sentences verbatim: *Option text is public — the correct answer is always one
of the five visible options.* And: *A host who reads Rust can work out the
answer from the source; the host's not being shown it keeps the host honest,
it is not a security guarantee.*

---

## 9. Capacity and realtime (AC-41, AC-52…AC-55)

200 concurrent sessions per room. Answer writes p95 < 500 ms **including the
deadline burst**; the burst — 200 writes inside 2 s — is a standalone test and
a spike ticket that runs before the walking skeleton. Reveal reaches all
connected buzzers ≤ 2 s p95. Substrate is decided in Phase 3; the contract is
one authoritative state per room with server-push to the wall and buzzers.

---

## 10. Accessibility (AC-40, AC-82…AC-86)

Keyboard operable with visible focus; polite live region per state change;
AA contrast; 44 px targets; `prefers-reduced-motion` disables all animation
with every state still legible; colour never the only signal.

---

## 11. Copy — the binding strings

Authored here and only here; the forbidden-copy lint (G-5) runs over every
**authored** participant-facing string — this table, and each question's
`explains` (except `explains.legacy`, which is never rendered), `why_tempting`
and `hint` — and **not** over `source` or
`options[].text`, which are program text and may legitimately contain *argument*
in a compiler diagnostic.

| Where | String |
|---|---|
| Wall, idle | **Time for a pop quiz.** · `join @ ‹link›` |
| Wall, live | `join @ ‹link› · still open` · well header: **What does this program print?** |
| Wall, trace (work, reveal) | **Step ‹N› of ‹M›** with dots · *the one worth stopping on* on a `pivot` step |
| Buzzer, join form | label **room code** · button **join** · beneath: *or open the link on the screen* |
| Wall, closed | **answers are closed** |
| Wall, split on | `‹answered› of ‹present› in the room answered` |
| Wall, work | **Let's walk it.** / Still no answer. / Nobody has to say anything. |
| Wall, reveal | `‹n› of us said ‹X›` beside the named incorrect option |
| Wall, released | **Let's go to the bar.** · ‹link› · *the question, the walk-through and the why — at your own pace.* |
| Buzzer, idle | You're in. Everything happens on the screen at the front — look up. |
| Buzzer, foot | no account · no name · no score |
| Buzzer, split | **Look up.** Where the room landed. · `‹n›` · people said **‹X›**, including you. · Five different readings. Nobody knows what anyone picked — including us. |
| Buzzer, work | **Look up.** We're walking it through. · Nothing to do. Nobody knows the answer yet, including us. |
| Buzzer, reveal | **Look up.** The answer is on the screen. · ✓ It was **‹Y›**. · You and ‹n−1› other people read it the same way. The why is being read out now — listen, don't read. / You were the only one, which makes yours the most interesting answer in the room. |
| Buzzer, split → reveal, **no answer given** | **Look up.** · You didn't answer — that's fine. · (split) Where the room landed. / (work) We're walking it through. / (reveal) ✓ It was **‹Y›**. The why is being read out now. |
| Buzzer, join failures (AC-29) | **malformed:** That's not a room code — six letters and numbers, never O, 0, I or 1. Try again. · **unknown:** No room with that code. Check the screen at the front. · **not yet open:** That room isn't open yet. Hold on — the host will put it on the screen. · **already ended:** That room has ended. Look for the link on the screen. · **closed for inactivity:** That room went quiet and closed. If it comes back, the screen at the front will say so. · **full:** That room is full. Watch the screen — you can still play along. |
| Buzzer, foot (split→reveal) | computed on this phone · never sent anywhere |
| Buzzer, released | Nothing about you was recorded. Not your answer, not your device, not that you were here. |
| Buzzer, hint | Show me a hint · *(once shown)* Only you can see this. Nobody is told you looked. |
| Buzzer, live, submission (AC-35/36) | **saving…** · **saved — ‹X›** · **couldn't save. Your last answer, ‹X›, is safe.** [Try again] · *(no answer yet)* **tap a letter** |
| Buzzer, reconnecting (AC-37) | **paused — reconnecting…** your answer ‹X› is safe / *(no answer yet)* **paused — reconnecting…** |
| Buzzer, closed | **answers are closed** · you said **‹X›** / you didn't answer |
| Live region (AC-83), verbatim | *The question is on the screen.* · *Saving.* · *Saved, ‹X›.* · *Couldn't save; your last answer is safe.* · *Answers are closed.* · *The room's split is on the screen.* · *Walking it through on the screen.* · *Revealed: it was ‹Y›.* · *The room is released.* · *Hint shown, only to you.* |
| Host, phase labels (AC-49), one per screen | **before the question** · **question live** · **answers closed** · **the split** · **walking it through** · **the answer** · **released** |
| Take it home, headings in order | **‹date›'s question** · **What happens** · **Why you might have read it as ‹X›** (one per incorrect option) · **The bit worth talking about** · **Walk it yourself** · **How this was checked** |
| Host, actions | Create a room · Put it on the screen · Close answers · Show the room its split · Let's walk it · Reveal · Release the room · Run it again |
| Wall + host, reveal, no incorrect votes | **Nobody read it another way.** (wall) · **Why nobody said anything else** — the host reads the `takeaway` beat only (host) |
| Host, reveal | **Read it aloud** · What happens · Why ‹n› of us said ‹X› (the room's actual most-chosen incorrect option, §4.5) · The bit worth talking about |
| Host, fit line (AC-100), shown from `live` on | **fits the room** / **too long for this room — clipped at the bottom** / **too wide for this room — clipped at the right** / **too long and too wide for this room** |
| Host, first screen | If you lose this phone, open this on another one: ‹resume link› · plus §8.1's two sentences |
| Take it home | **Why you might have read it as ‹X›** — one heading per incorrect option, over its `why_tempting` text. No counts (D-12). |
| Static fallback | `Space` next phase · `←` `→` step the trace · `Esc` back a phase |
| Uniqueness (organizer-facing, AC-18) | *no exact or normalized duplicate found* — never *original* |
| Receipt | §7.5 |
| **Forbidden** anywhere participant-facing — the one normative list; the lint (G-5, AC-98) is these case-insensitive patterns and `EVALUATION.md` cites this row | `turn to`, `ask (someone\|the person\|your neighbou?r)`, `find someone`, `volunteer`, `who (said\|picked\|chose)`, `\bwrong\b`, `\bincorrect\b`, `✗`, `argu`. The wall's *‹n› of us said ‹X›* and the host phone's *Why ‹n› of us said ‹X›* do not match `who (said|picked|chose)` and are allowed. |

---

## 12. The static fallback

The wall's seven views with one question's data baked in, one HTML file,
keyboard-driven (`Space` next phase, `←`/`→` step the trace inside work and reveal, `Esc` back a phase),
no network, no phones. Built by the same wall code with `mode: "static"` and a
fixture, so it is never a second design. Produced for the scheduled question
at scheduling time; the organizer carries it to every meetup. Satisfies AC-77's
spirit when the room itself cannot run.

---

## 13. Take it home

`/last`: the last released question — source with colour, the trace
steppable both ways at the reader's pace, the three beats, the receipt, the
five options with the correct one marked. **No room state, of any kind** — AC-56 holds unamended, as the client said on
2026-08-14 (D-12): no counts, no most-chosen option. The middle beat becomes
*why you might have read it as ‹X›* for **every** incorrect option, which the
bank already carries (D-9), so the page loses nothing it could honestly have. Rebuilt at every release.
Code scrolls in its container on a phone (AC-33).

---

## 14. Criteria index

| Section | Criteria |
|---|---|
| §1 | AC-1, AC-77, AC-89, AC-90 |
| §2 | AC-7, AC-23/23a/23b, AC-25, AC-32, AC-43, AC-45, AC-47, AC-56–58, AC-60, AC-61, AC-64, AC-65, AC-69, AC-71, AC-72, AC-79, AC-87, AC-92, AC-93, AC-95, AC-97, AC-98 |
| §3 | AC-6, AC-13, AC-14–17, AC-24, AC-46, AC-50, AC-66, AC-68, AC-75, AC-76 |
| §4 | AC-28–31, AC-34–37, AC-39, AC-40, AC-46–49, AC-81, AC-93, AC-94, AC-97 |
| §5 | AC-33, AC-38, AC-78, AC-80, AC-99, AC-100 |
| §6 | AC-32, AC-49, AC-50, AC-51, AC-83, AC-85 |
| §7 | AC-2–5, AC-8–12, AC-18–22, AC-26, AC-27, AC-73, AC-74, AC-88, AC-91, AC-96 |
| §8 | AC-64–70 |
| §9 | AC-41, AC-52–55 |
| §10 | AC-40, AC-82–86 |
| §11 | AC-42, AC-44, AC-59, AC-62, AC-63, AC-98 |
| §12 | AC-77 |
| §13 | AC-33, AC-56 |

Every ID in AC-1…AC-100 appears above; `EVALUATION.md` carries the proof.

---

## 15. Decided in Phase 3, not here

Stack and substrate for the room; the sandbox for verification; the LLM and
its spend cap; where the pipeline runs; the short-link domain; hosting for
take-it-home and the static fallback. `BUILDPLAN.md` records each with the
options weighed. No design decision is deferred: D-8 (the hint rides in the
live payload), D-9 (a `why_tempting` per incorrect option, no prediction),
D-10 (the trace's resolving step is withheld until reveal), D-15 (options are
one line of at most 29 characters, so the wall's options block is a constant
190 px), D-12 (take-it-home
carries no room state) and D-13 (no embedding model; token-bigram Jaccard for
near-duplicates) are decided here and logged in `run-state.md`. **Fonts** are vendored: Cascadia Mono and
Instrument Serif ship in `web/shared/fonts/` from the design system's
`assets/fonts/`, self-hosted, no font CDN (D-14).
