# PQ-16: Accessibility sweep

BUILDPLAN.md T-13 (M2).

`a11y`: keyboard, live regions per phase and the private hint, AA contrast, 44 px, reduced motion, over every surface and phase

Criteria: AC-82–86
Harness hooks: see EVALUATION.md rows for those IDs; `just test` is the inner loop (≤ 60 s).
Depends on (BUILDPLAN): T-05–07
BUILDPLAN notes: Serialized on `justfile` (`a11y`)

Workflow mode: fast-track. Terminal pre-merge status: review. PR base: origin/main.
Standing clauses: criteria outrank SPEC on behaviour, state, payloads and copy; SPEC outranks the prototype, which wins on visual detail only. Never write down what a program prints. Answer position is a uniform draw from the date, never balanced. Miri proves absence of UB on executed paths only. Colour is never the only signal. Code never causes horizontal page scroll. Never `git add -A`. Signing goes through 1Password: retry once, then escalate.

Plan: filled in by the delegator's plan phase.

# Plan (delegator, 2026-09-29)

## What the sweep found on the branch (before any change)

Read against SPEC §4/§6/§10/§11, AC-82…86 (+AC-33/40/48/97/102), EVALUATION rows 37 and 132–136, and the code at 1263add.

1. **AC-84, focus ring.** Every focus ring is `--border-focus` / `--accent-primary` = amber-500 `#d69e2e`: 2.27:1 on `--bg-primary`, 2.39:1 on white. Below the 3:1 non-text minimum on every surface.
2. **AC-84, wall join link.** `.joinstrip b` is amber-500 on `--bg-primary` at 30 px: 2.27:1, below 3:1 even as large text.
3. **AC-84, the well header.** `.rn-src-head` (the well header *What does this program print?*, 12 px) and `.rn-src-ln` line numbers are `--text-muted` `#718096` on white: 4.0:1, below 4.5. So are comments (`.tk-c`).
4. **AC-84, the correct marker.** `.rn-correct` text is green-500 on the page: 3.08:1. That passes only as large text; the bar letters are 18 px bold on the wall's reveal and on the buzzer (not large). The wall's machine-column chip is white on green-500 at 16 px bold: 3.25:1.
5. **AC-84, the join field.** The buzzer input's 2 px `--border-default` edge is 1.4:1 on the page, and that edge is what identifies the field.
6. **AC-82, focus lost on every re-render.** The buzzer (`render()`) and the host (`paint()`) replace `innerHTML` on every event, and take-it-home replaces the walk on every step. A keyboard user who presses a letter, a host action or a trace button ends up focused on `<body>`, so the next Tab starts from the top of the page.
7. **AC-85, take-it-home's trace buttons** are 34 px (trace.js sets it inline) and home has no `.btn` rule. AC-85 says "touch targets", with no surface named.
8. **AC-49/AC-83, host.** The phase label is announced only on a *change*. A host who loads or resumes a room hears nothing, so `before the question` is never announced at all.
9. **home.css** has no focus rule and no reduced-motion block of its own. The `-Vv` block (`.vv`) scrolls sideways but can't be reached from the keyboard.
10. **Buzzer `<main aria-live="off">`.** This one is correct, and I'll explain it in a comment rather than change it. `main` is re-rendered whole on every event, and "off" keeps assistive technology from reading each repaint. The one polite region is `PQ.announce`'s, which is appended to `<body>` outside `main`.
11. **AC-86.** No stylesheet in `web/` declares an animation or transition, and no script uses `requestAnimationFrame`, `setInterval` or smooth scroll. Buzzer, host and wall carry reduce blocks already; home gets one.
12. **AC-83.** The wall and buzzer phase→string maps match §4's phases (idle announces nothing). The static fallback announces through the same `Wall.show`. The hint is announced by the buzzer's reducer alone, and it makes no request (AC-48).

## Files

- **new** `web/test/a11y.test.js` (and `web/test/a11y/css.js`, a small CSS reader and WCAG contrast helper) holds the suite. It renders every surface at every phase from fixtures: the wall from `q3-phases.json` (7 phases, every step, the unanimous variant) and the static fallback from `q3-static.json`; the buzzer by driving its reducer through join, the six refusals, idle→released, saving/saved/failed/retry, the hint, and paused; the host through `render`, `renderCreate` and `renderSignIn`, and `boot()` on a fake window with scripted socket frames; take-it-home from `take-home.json`, including null and legacy snapshots. Tests are named `ac82_<surface>`, `ac83_<surface>`, `ac84_<surface>`, `ac85_<surface>` and `ac86_<surface>`. With `A11Y_MATRIX=1` it prints a surface × screen × criterion matrix and every contrast ratio.
- **new** `web/test/a11y/browser.html` is the browser-pass harness. It serves each surface with its real CSS and JS, driven by scripted fake room frames and real keyboard input. The static wall is the real fallback with q3's bake.
- `justfile`: the `a11y` recipe (`A11Y_MATRIX=1 node --test web/test/a11y.test.js`), its header line, `PENDING` loses `a11y:T-13`, and `test-full` depends on `a11y`. The suite is hermetic and fast, so the `test-web` glob already runs it inside `test`.
- **Fixes**, in each surface's own files:
  - `web/shared/components.css` (additive): two new custom properties, `--focus-ring: var(--amber-700)` and `--success-text: #276749` (the prototype's own machine-provenance green). No existing token value changes.
  - buzzer.css / host.css / home.css / wall.css: focus rings go to `--focus-ring`. On the wall that's the reading region; home gets a whole `:focus-visible` rule. The wall join link becomes `--accent-hover`. The well header, line numbers and comments go to `--text-secondary` on wall and home. The `.rn-correct` letter text becomes `--success-text` in the bars, and the wall's machine chip background becomes `--success-text`. The buzzer input edge becomes `--text-muted`, which is 3.8:1. `min-width` goes on every buzzer control. Home's trace buttons get 44 px with `!important`, because trace.js sets them inline.
  - buzzer.js / host.js / home.js: focus is restored across re-renders by a key (letter, act, primary, step). Fallbacks: retry falls back to the letter just tried, and the hint moves focus to the hint panel. Host announces the phase label on the first payload. Home's `.vv` gets `tabindex="0"` with its row's label.
  - buzzer/index.html gets a comment explaining `aria-live="off"`.
- `web/README.md`: a new *Accessibility* section covering what `just a11y` proves, the browser pass steps, and how to run each.

## What a stub cannot prove → the browser pass

These need a browser: the real Tab order and the ring as drawn, rendered target sizes, the live region actually spoken (the accessibility tree / VoiceOver), and reduced motion under the OS setting. I'll run them once in the c11 browser against `browser.html` served on loopback, and write the steps into web/README.md.

## Choices and contract tensions (the side I take)

- **D1. AC-84's dim-room clause:** there is no built presentation (T-16). I prove AA on *lights up* only and record the clause as a deviation (Orchestrator ruling, F-37).
- **D2. Criterion over prototype on colour:** the ring, the join link, the well header, the ✓ letter and the chip all move off the prototype's values. The criterion outranks the prototype (precedence rule 9). Every change is per-surface; no token value changes. A follow-up could set `--border-focus` to amber-700 upstream, which would let the per-surface ring rules collapse into one. I'm leaving that as a note, not a QUESTION, because I change no token value.
- **D3. Dimmed trace lines are not held to 4.5:1.** They sit at opacity .32, about 1.9:1. Highlight-and-dim *is* the trace (DESIGN, AC-97/99), and passing 4.5:1 would erase the dim. The focus region carries the step, and the full source is legible in the reading phases and on take-it-home. The suite prints the ratio and marks the pair "exempt, flagged". I'm raising it to the Orchestrator in the DONE notes.
- **D4. The split bars are supplementary.** Their fills sit on the track below 3:1 (green) but are not required, because `n · p%` beside each bar carries the value (the WCAG 1.4.11 text-alternative case). The suite prints them as supplementary.
- **D5. `aria-live="off"` on the buzzer's main is kept and explained** (see finding 10).
- **D6. AC-85 on take-it-home.** EVALUATION scopes it to the buzzer and host. AC-85's words aren't scoped, so I fix home too.
- **D7. The trace buttons' aria-labels** ("previous step", "next step") are trace.js literals that are not in §11. They predate this ticket and I leave them. The step buttons' visible `←` `→` come from host's STEPS. No new strings are authored.
