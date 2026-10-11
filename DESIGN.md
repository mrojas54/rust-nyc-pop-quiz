---
name: Rust NYC Pop Quiz
description: Projector-first Rust quiz, phone as buzzer.
colors:
  white: "#ffffff"
  gray-50: "#f8f9fa"
  gray-100: "#e9ecef"
  gray-200: "#dee2e6"
  gray-300: "#cbd5e0"
  gray-500: "#718096"
  gray-600: "#4a5568"
  gray-800: "#2d3748"
  amber-300: "#faf089"
  amber-500: "#d69e2e"
  amber-600: "#b7791f"
  amber-700: "#945c0a"
  green-500: "#38a169"
  red-500: "#e53e3e"
  orange-500: "#dd6b20"
  blue-500: "#3182ce"
  green-50: "#f0fff4"
  red-50: "#fef2f2"
  orange-50: "#fffaf0"
  blue-50: "#ebf8ff"
  bg-primary: "#f8f9fa"
  bg-secondary: "#e9ecef"
  bg-tertiary: "#dee2e6"
  bg-input: "#ffffff"
  text-primary: "#2d3748"
  text-secondary: "#4a5568"
  text-muted: "#718096"
  accent-primary: "#d69e2e"
  accent-secondary: "#b7791f"
  accent-hover: "#945c0a"
  accent-subtle: "#faf089"
  border-default: "#cbd5e0"
  border-focus: "#d69e2e"
  success: "#38a169"
  error: "#e53e3e"
  warning: "#dd6b20"
  info: "#3182ce"
  success-subtle: "#f0fff4"
  error-subtle: "#fef2f2"
  warning-subtle: "#fffaf0"
  info-subtle: "#ebf8ff"
  focus-ring: "#945c0a"
  success-text: "#276749"
typography:
  display:
    fontFamily: "'Instrument Serif',Georgia,'Times New Roman',serif"
    fontSize: "36px"
    fontWeight: 400
    lineHeight: 1.2
  title:
    fontFamily: "'Instrument Serif',Georgia,'Times New Roman',serif"
    fontSize: "24px"
    fontWeight: 400
    lineHeight: 1.2
  body:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "14px"
    lineHeight: 1.6
  label:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "12px"
    letterSpacing: "0.08em"
  code:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "13px"
    lineHeight: 1.6
  brand-title:
    fontFamily: "'Instrument Serif',Georgia,'Times New Roman',serif"
    fontSize: "30px"
    purpose: "Shared text-3xl token; buzzer wordmark, host heading and home masthead."
  room-code-input:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "28px"
    purpose: "Buzzer join input; 0.2em tracking, distinct from serif room identity."
  room-code-identity:
    fontFamily: "'Instrument Serif',Georgia,'Times New Roman',serif"
    fontSize: "28px"
    purpose: "Buzzer joined-room identity; tracking-wide."
  voting-letter:
    fontFamily: "'Instrument Serif',Georgia,'Times New Roman',serif"
    fontSize: "46px"
    purpose: "Buzzer choice control; line-height 1; selected weight 700."
  host-room-code:
    fontFamily: "'Instrument Serif',Georgia,'Times New Roman',serif"
    fontSize: "34px"
    purpose: "Host room-code panel; tracking-wide."
  host-step-control:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "20px"
    purpose: "Host phase control; 6px 14px padding."
  buzzer-status:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "17px"
    purpose: "Submission, pause and company-count feedback."
  buzzer-follow-up:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "19px"
    purpose: "Post-vote feedback and answer text."
  buzzer-answer-letter:
    fontFamily: "'Instrument Serif',Georgia,'Times New Roman',serif"
    fontSize: "72px"
    purpose: "Post-vote letter; line-height 1."
  split-row:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "18px"
    purpose: "Buzzer split and wall reveal row; wall reading row is 22px."
  split-count:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "16px"
    purpose: "Buzzer split and wall reveal count; wall reading count is 19px."
  keyboard-hint:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "11px"
    purpose: "Take-home kbd; 3px corner radius is component-specific."
  wall-option:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "24px"
    purpose: "Wall option text at 1120px design width; letter 22px, reveal machine option 16px."
  wall-trace-note:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "15px"
    purpose: "Wall trace note; current wall caller does not enable trace.js big option."
  wall-title:
    fontFamily: "'Instrument Serif',Georgia,'Times New Roman',serif"
    fontSize: "62px"
    purpose: "Wall title-card h2 at design width."
  wall-release-link:
    fontFamily: "'Cascadia Mono','SF Mono',Monaco,'Roboto Mono',Consolas,'Courier New',monospace"
    fontSize: "40px"
    purpose: "Wall release link at design width."
rounded:
  radius: "4px"
spacing:
  space-1: "4px"
  space-2: "8px"
  space-3: "12px"
  space-4: "16px"
  space-6: "24px"
  space-8: "32px"
  space-12: "48px"
  space-16: "64px"
  panel-pad: "24px"
  panel-pad-compact: "16px"
components:
  button:
    backgroundColor: "{colors.bg-input}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.radius}"
    padding: "10px 18px"
  button-hover:
    backgroundColor: "{colors.bg-tertiary}"
  button-primary:
    backgroundColor: "{colors.accent-primary}"
    textColor: "{colors.gray-800}"
    rounded: "{rounded.radius}"
    padding: "10px 18px"
  button-primary-hover:
    backgroundColor: "{colors.accent-hover}"
    textColor: "{colors.white}"
  panel:
    backgroundColor: "{colors.bg-secondary}"
    rounded: "{rounded.radius}"
    padding: "{spacing.panel-pad-compact}"
  source-well:
    backgroundColor: "{colors.bg-input}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.radius}"
---
# Design — Rust NYC Pop Quiz

**Status: binding. Written 2026-09-03 at the close of the T-20 drive and made the
build's visual contract the same day on the client's word: *"yes i love it."*** The
original decision sections describe what `prototypes/C-projector-first.html` and
`prototypes/take-it-home.html` actually do; nothing here is a proposal the prototype
does not already carry. The implementation inventory added below is descriptive, not a
new visual contract. Later accepted amendments in the build contract and source comments
(including accessibility companions and trace dimming) qualify the historical prototype;
do not undo them to match a snapshot.

The converged direction is **C — projector-first, phone as buzzer.** The code
lives on the big screen and nowhere else; the phone is five letters. The client
stated the assumption in her own words before the takes were compared (A-4,
*"only options to vote on, code and tracing need to be on screen"*), then
confirmed it by driving all three: *"C is it."* Directions A (hands, no phones)
and B (phone-first) stay in `prototypes/` as built comparisons for October's
field notes, not as candidates.

## Overview

**Creative North Star: "A well-made terminal"** — the existing metaphor from
`PHILOSOPHY.md`, preserved rather than replaced.

Nostalgic and technical: muted greys, warm amber, monospace controls and data,
and serif headings. The shared system stays quiet so the Rust program and the
room’s understanding carry the experience.

**Key Characteristics:**

- Projector-first, phone as buzzer.
- Light surfaces, white source wells, restrained depth.
- Counts rather than personal verdicts.
- Host-paced trace, with colour reserved for reading phases.

Documentation pass (2026-10-02), imported in draft PR #70 (2026-10-11): canonical
sections and machine-readable values extracted from `web/shared/tokens.css`,
`web/shared/components.css`, `web/shared/fonts.css`, and the surface stylesheets. CSS
remains the executable token source; YAML resolves CSS aliases to literal values because
primitives cannot reference other primitives. Component references retain semantic
names. The original binding decisions and their evidence remain below under new
headings. Cross-references such as §5–§9 in those passages refer to `PHILOSOPHY.md`, not
this document's new section order. The added inventories do not supersede phase
behavior, voice, deliberate divergences or accepted build amendments. Frontmatter is a
scoped inventory, not an exhaustive list of permitted CSS values; component entries
describe host/buzzer defaults, not every surface or state. Surface overrides remain
intentional. This merge does not establish new aesthetic choices or claim a live visual
or venue-readability verification.

### What the design serves

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

## Colors

Muted technical greys provide the ground; warm amber marks emphasis and
interaction. The frontmatter retains the source token names and exact values.

### Primary

- **Warm amber** (`accent-primary`): primary controls, emphasis, trace edges.
- **Deep amber** (`accent-secondary`, `accent-hover`): links and hover states.
- **Pale amber** (`accent-subtle`): the trace focus region and company counts.

### Neutral

- **Paper grey** (`bg-primary`): the page ground.
- **Panel grey** (`bg-secondary`) and **inset grey** (`bg-tertiary`): containers and hover surfaces.
- **White** (`bg-input`): source wells and input fields.
- **Technical charcoal** (`text-primary`), **slate** (`text-secondary`), and
  **muted slate** (`text-muted`): body, secondary information, and metadata.
- **Soft slate** (`border-default`): boundaries and dividers.

### Semantic signals

Success, error, warning, and information each have a foreground and subtle
background token. They communicate state with words, glyphs, or border changes;
they are not extra brand accents. Rust syntax colours are component-scoped in
`web/shared/components.css`, not a global accent palette.

**The Accessible Companion Rule.** Surface styles opt into `focus-ring` (deep amber) and
`success-text` (dark green), leaving inherited house-brand tokens unchanged. The source
flags `text-muted` as unsuitable for normal small body text. The metadata label is
historical usage, not a contrast exemption: small metadata also needs AA contrast.
Current host, buzzer, home and wall styles use `text-secondary` for applicable small
text. Do not restore the muted token merely because it appears in this inventory.

## Typography

Fonts are vendored through `web/shared/fonts.css`, use `font-display: swap`,
and require no font CDN (D-14). The frontmatter includes the full fallback stacks.

- **Display:** Instrument Serif, regular, the shared h1 role.
- **Title:** Instrument Serif, regular, the shared h2/h3 role.
- **Body:** Cascadia Mono, shared body and control type.
- **Label:** Cascadia Mono, small metadata; caps tracking only where the component applies it.
- **Code:** Cascadia Mono, the shared well default for personal reading distance.
- **Brand title:** 30px (`text-3xl` exists in shared CSS); surface styles
  currently spell 30px literally. This is not a request to refactor them.

### Surface typography inventory — rechecked 2026-10-11

Frontmatter records shared defaults and scoped roles, including separate serif room
identity and monospace input roles at 28px. Semantic heading level does not select a
global visual size: host h3 and home `.sec-h` use 14px mono, while home explanation h3
uses the shared 24px serif. Code inside explanation headings uses .7em. Labels vary
tracking by component (home kicker .12em, section labels .1em); .08em is not a universal
label rule.

| Surface | Additional implemented sizes and roles |
|---|---|
| Buzzer | 30px wordmark, 24px role, 28px identity/input in different faces, 46px choices, 24px saved glyph, 17px status, 19px follow-up, 72px post-vote letter, 18px split row, 16px count |
| Host | 30px heading, 24px role, 34px room code, 20px phase control, 14px mono h3 |
| Take-home | 30px masthead, 24px explanation headings, 14px mono section headings, 13px code, 12px receipt, 11px kbd |
| Wall | 30px brand/join link, 24px options, 22px option letter/split row, 19px split count/join strip, 18px reveal row/middle, 16px reveal count/machine option, 15px receipt/trace CSS default, 13px values, 62px title, 40px release link |

Wall values are design-canvas sizes, scaled with the 1120×630 canvas; they are not phone
defaults. Source size is derived and refitted by `typemodel.js`. The wall calls
`traceNoteHtml` with `noNav`, without `big`, so its note is 15px. The shared renderer
also supports an optional 19px `big` mode; that option is not the current wall role.
Preserve this cascade and source fitting.

**The Wall Measurement Rule.** The frontmatter code size is not a projector size.
`web/shared/typemodel.js` derives and measures wall type from venue inputs. Serif is
used for headings and established display roles at 24px and above (room identities and
voting letters included). The existing aesthetic statement below remains the binding
account of the font pairing.

### Aesthetic

Inherited from the **Rust NYC Design System** (the house brand, claude.ai/design
`d608a53a…`), not reinvented. `prototypes/_shared/tokens.css` §1 is its four
token files verbatim.

- **Type.** Cascadia Mono for everything — body, controls, code, data.
  Instrument Serif for headings at 24px and above and the established
  large room-code/voting-letter roles documented above. The fonts are not
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

## Layout

The shared spacing scale and both panel padding tokens are in the frontmatter.
Observed surface containers differ by task: the buzzer caps at 375px with 12px
outer padding, the host phone uses the 480px container token and compact panel
padding, and take-it-home caps its sheet at 760px with 16px horizontal page
padding. Take-it-home has a compact layout at a 480px breakpoint. These are
surface measurements, not a replacement global grid.

### The three surfaces

| Surface | Who sees it | What it carries |
|---|---|---|
| **The wall** | the whole room | brand line top-left; the join strip; the source, line-numbered, full-width; five options beneath in two columns; the split as bars with counts; the trace; the reveal; the handover |
| **The buzzer** | a participant, on their own phone | the room code as the room's identity; five letters; *Show me a hint*; after the split, a **count** beside their letter, never a ✗; at the end, *nothing about you was recorded* |
| **The host phone** | the host | one button per phase transition; `←` `→` to step the trace; the read-aloud script at the reveal; **no answer preview** (§7) |

Neither source nor trace is rendered on the live participant buzzer (AC-32 as
rewritten). The separate released take-home reading surface carries both. The buzzer is
deliberately dull.

### The wall's type model

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

### Hypotheses this design rests on

- **Screen 15ft, back row 20ft.** A guess. The rehearsal (`mvp/PRACTICE-RUN.md`)
  measures width, height and the distance to the last occupied row, and every
  number in the type model moves with them.
- **AC-91's 30 seconds.** The client had an opinion at 10 seconds on a question
  she had seen many times; the real sample is an attendee's first look.
- **Participation rate.** Never measured; it sizes the deadline burst (AC-52).

## Elevation & Depth

Product panels use one subtle card shadow (`0 1px 3px rgba(45,55,72,0.08)`)
with tonal backgrounds and fine borders. Keep it structural and restrained.
Device frames, browser chrome, and the prototype badge belong to review
artifacts; their heavier shadows and striped badge are not product tokens.

## Shapes

The shared form language uses one gently squared corner (`radius`, 4px) and fine
boundaries (`border-w`, 1px). Trace focus and provenance use stronger left edges; human-
reviewed content uses a dashed edge. Circles for progress dots, the home kbd 3px
corners, the wall projector 6px frame, and prototype device-frame corners are scoped
exceptions, not new panel radii.

## Components

### Buttons and fields

White neutral buttons use technical charcoal text and an inset-grey hover. Primary
buttons use warm amber, dark text, and a deep-amber hover with white text. These
host/buzzer `.btn` defaults use 10px × 18px padding and the shared corner radius; host
phase controls override padding to 6px × 14px. Voting letters and trace controls have
their own styles and state rules. Host and buzzer controls reserve at least a 44px touch
target in both axes. Disabled host/buzzer `.btn` controls halve opacity and use the not-
allowed cursor. Disabled voting letters use a default cursor and state-specific text; do
not impose `.btn` styling on them. Buzzer focus uses a 3px deep-amber outline; host
focus uses 2px, both offset by 2px.

The buzzer’s room-code field uses centred uppercase monospace text, a white
background, and 8px × 12px padding. It retains a visible deep-amber focus outline.

### Panels, source, and trace

Panels use the secondary ground, fine border, compact padding, and card shadow. Source
wells are white, line-numbered, with a 2ch gutter and an internal scroll region on
personal-reading surfaces. The wall measures fit instead. Trace focus uses a pale-amber
ground and a 3px left edge (deep-amber on home/wall for contrast); lines outside the
focus dim to .70 opacity in current shared CSS. This accepted AC-84/PQ-40 amendment
supersedes the prototype's .32. Progress states include a step number, not only dots or
colour.

Machine provenance uses a solid green edge; human-reviewed prose uses a dashed
orange edge. These distinguish the evidence source, not the participant’s result.

### The segment, in seven phases

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

### Voice

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
- **The verification receipt shows what was done** (§4): a short list of steps —
  compiled, ran five times, output never varied, Miri ran clean — with no
  sentence and no claim word. The rest is on take-it-home. Never more than was done.

### Take it home

The second surface, behind the link on the released wall: the last meetup's
question, alone, at the reader's own pace — the trace steppable both ways,
the three-beat explanation, the receipt. §9's private pace lives here (touchpoint T-15:
wall host-stepped, phones step freely). It carries no room state, and
no count — `tone-architect` settled the AC-95 conflict as D-12 (2026-09-03).

### The design-system divergences, all deliberate

| Divergence | Why | Who carries it |
|---|---|---|
| Syntax colour in the reading phases (`SourceCode` renders none) | the trace reason only holds while a trace runs — §5, AC-99 | the client, to Cole |
| Room code off the wall; a short link instead | the client's join strip; the link can carry the code | AC-28 rewritten |
| `explanation` becomes three fields | AC-44 needs a beat a beginner can repeat, and §9 needs the popular wrong answer named | `SPEC.md` §3.1 (`explains`), AC-95 as read by D-9 |

## Do's and Don'ts

### Do

- **Do** preserve the binding prototype decisions and their acceptance criteria.
- **Do** measure projector source fit using the venue’s dimensions and back-row distance.
- **Do** pair colour with text, a glyph, a count, or a structural signal.
- **Do** vendor the brand fonts and respect reduced-motion preferences.

### Don’t

- **Don’t** add gradients, glass, bounce, or extra panel radii.
- **Don’t** mark a participant wrong, introduce scores, or preview the answer to the host.
- **Don’t** promote prototype-only chrome or the dim-room proposal into the light product contract.
- **Don’t** balance correct-answer positions or treat a built deck as a run segment.

### What is deliberately absent

- No code or trace on the live participant buzzer; take-home is the explicit
  personal-reading exception. No timer on the wall. No scoreboard, name,
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

### For the build

The converged take is reproduced **one-to-one**: every designed control present, even if
it only toasts *not yet implemented (AC-x)*. The prototype and later accepted build
amendments are the references; the added implementation inventory does not authorize
changing either. Everything this file once listed as open for `tone-architect` is
settled in `SPEC.md` (the static fallback §12, the used-question ledger G-10, the
explanation model §3.1, fonts D-14, AC-21's measurement at HC-2); the real short link
behind the placeholder `bit.ly/jnfnvd` is the client's, `BUILDPLAN.md` H-2.

---

*Stage 2 of the Tone arc, `tone-prototype`. Referenced from `CLAUDE.md`. If the
built product or a real meetup contradicts this, the prototype and this file
are what change — propose the amendment, update, propagate.*
